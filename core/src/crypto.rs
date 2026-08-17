//! Mã hoá đầu-cuối cho hộp thư R2. ADR-0005 C1–C4, ADR-0007 § 7b.
//!
//! Thư viện: `dryoc` (libsodium thuần Rust). Không tự chọn cipher, không tự
//! sinh nonce bằng tay, không lắp primitive.
//!
//! Cấu trúc: Argon2id(passphrase, salt) → khoá 32 byte → `crypto_secretbox`
//! (XSalsa20-Poly1305, AEAD) cho từng message.
//!
//! **Vì sao không dùng `age`** như N18b gợi ý đầu tiên: recipient passphrase
//! của `age` chạy scrypt **mỗi message**, tốn ~1s CPU cho mỗi lần copy — vỡ
//! N1b và N9. N18f vốn đã chốt "dẫn xuất khoá bằng Argon2id", nên dẫn xuất
//! một lần lúc khởi động rồi AEAD từng message mới là thứ NFR yêu cầu.

use anyhow::{anyhow, Result};
use dryoc::classic::crypto_pwhash::{crypto_pwhash, PasswordHashAlgorithm};
use dryoc::classic::crypto_secretbox::{
    crypto_secretbox_easy, crypto_secretbox_open_easy, Key, Nonce,
};
use dryoc::constants::{
    CRYPTO_PWHASH_MEMLIMIT_INTERACTIVE, CRYPTO_PWHASH_OPSLIMIT_INTERACTIVE,
    CRYPTO_PWHASH_SALTBYTES, CRYPTO_SECRETBOX_MACBYTES, CRYPTO_SECRETBOX_NONCEBYTES,
};
use dryoc::rng::copy_randombytes;

/// Khoá phiên, dẫn xuất một lần lúc mở app.
pub struct SecretKey(Key);

/// Argon2id với **tham số mặc định của thư viện** (INTERACTIVE) — ADR-0007
/// cấm tự chỉnh. Cùng passphrase + cùng salt = cùng khoá trên cả hai máy.
pub fn derive_key(passphrase: &str, salt: &[u8]) -> Result<SecretKey> {
    if salt.len() != CRYPTO_PWHASH_SALTBYTES {
        return Err(anyhow!(
            "salt phải đúng {CRYPTO_PWHASH_SALTBYTES} byte, đang có {}",
            salt.len()
        ));
    }
    let mut key: Key = [0u8; 32];
    crypto_pwhash(
        &mut key,
        passphrase.as_bytes(),
        salt,
        CRYPTO_PWHASH_OPSLIMIT_INTERACTIVE,
        CRYPTO_PWHASH_MEMLIMIT_INTERACTIVE,
        PasswordHashAlgorithm::Argon2id13,
    )
    .map_err(|e| anyhow!("Argon2id thất bại: {e}"))?;
    Ok(SecretKey(key))
}

pub fn random_salt() -> [u8; CRYPTO_PWHASH_SALTBYTES] {
    let mut salt = [0u8; CRYPTO_PWHASH_SALTBYTES];
    copy_randombytes(&mut salt);
    salt
}

impl SecretKey {
    /// Trả về `nonce || ciphertext`. Nonce ngẫu nhiên mỗi message — cũng vì
    /// thế hai lần mã hoá cùng nội dung cho ra hai object khác nhau (T15).
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce: Nonce = [0u8; CRYPTO_SECRETBOX_NONCEBYTES];
        copy_randombytes(&mut nonce);

        let mut out =
            vec![0u8; CRYPTO_SECRETBOX_NONCEBYTES + plaintext.len() + CRYPTO_SECRETBOX_MACBYTES];
        out[..CRYPTO_SECRETBOX_NONCEBYTES].copy_from_slice(&nonce);
        crypto_secretbox_easy(
            &mut out[CRYPTO_SECRETBOX_NONCEBYTES..],
            plaintext,
            &nonce,
            &self.0,
        )
        .map_err(|e| anyhow!("mã hoá thất bại: {e}"))?;
        Ok(out)
    }

    /// Sai khoá, sai salt, hay lật một byte đều rơi vào đây — Poly1305 bắt
    /// hết (T14). Người gọi **giữ lại object**, không xoá (ADR-0005 C4).
    pub fn decrypt(&self, blob: &[u8]) -> Result<Vec<u8>> {
        let min = CRYPTO_SECRETBOX_NONCEBYTES + CRYPTO_SECRETBOX_MACBYTES;
        if blob.len() < min {
            return Err(anyhow!("object cụt: {} byte, tối thiểu {min}", blob.len()));
        }
        let (nonce_bytes, ct) = blob.split_at(CRYPTO_SECRETBOX_NONCEBYTES);
        let mut nonce: Nonce = [0u8; CRYPTO_SECRETBOX_NONCEBYTES];
        nonce.copy_from_slice(nonce_bytes);

        let mut out = vec![0u8; ct.len() - CRYPTO_SECRETBOX_MACBYTES];
        crypto_secretbox_open_easy(&mut out, ct, &nonce, &self.0)
            .map_err(|_| anyhow!("giải mã thất bại — passphrase không khớp hoặc dữ liệu hỏng"))?;
        Ok(out)
    }
}
