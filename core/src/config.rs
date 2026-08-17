//! Cấu hình đọc từ `~/.config/x2clip/config.toml` (US-C5, T9).
//!
//! Thiếu file → tạo file mặc định. Sai cú pháp → báo lỗi **kèm số dòng** và
//! **không ghi đè** file của người dùng.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Tên máy này — prefix hộp thư của mình là `inbox/<machine>/`.
    pub machine: String,
    /// Tên máy kia — nơi mình PUT tới.
    pub peer: String,
    /// N13b — chu kỳ poll hộp thư, giây.
    #[serde(default = "default_poll_secs")]
    pub poll_secs: u64,
    /// Thiếu hẳn section này = chế độ local-only, không đồng bộ.
    pub mailbox: Option<MailboxConfig>,
    pub crypto: Option<CryptoConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MailboxConfig {
    pub endpoint: String,
    pub bucket: String,
    #[serde(default = "default_region")]
    pub region: String,
    /// ADR-0005 C5. ponytail: để thẳng trong config.toml `0600` nằm trong thư
    /// mục `0700` — cùng mức bảo vệ với "file 0600" mà ADR yêu cầu. Chuyển
    /// sang Keychain khi nào có máy thứ ba hoặc nhiều người dùng chung máy.
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Viết tay thay vì `derive` — T12 grep log tìm **giá trị thật** của access
/// key. Một `{cfg:?}` ở nhánh lỗi nào đó là đủ để đỏ, và lúc đó nó đã nằm
/// trong log rồi. Che ở đây, một lần, thay vì soi từng chỗ log.
impl std::fmt::Debug for MailboxConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailboxConfig")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("region", &self.region)
            .field("access_key_id", &"***")
            .field("secret_access_key", &"***")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// Salt Argon2id, 16 byte dạng hex.
    ///
    /// **Phải copy y hệt sang máy kia** cùng với passphrase. Cùng passphrase
    /// nhưng khác salt = khác khoá = mọi object đều giải mã hỏng, và triệu
    /// chứng nhìn hệt như "sai passphrase".
    pub salt: String,
    /// File chứa passphrase, `0600`. Trống thì đọc biến môi trường
    /// `X2CLIP_PASSPHRASE`. Passphrase không bao giờ được ghi vào config.
    #[serde(default)]
    pub passphrase_file: Option<PathBuf>,
}

fn default_poll_secs() -> u64 {
    30
}
fn default_region() -> String {
    "auto".to_string()
}

pub fn default_config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("không tìm được thư mục config của user"))?
        .join("x2clip");
    std::fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(dir.join("config.toml"))
}

/// File mặc định: chạy được ngay ở chế độ local-only, các mục cần điền để
/// bật đồng bộ đều đã có sẵn dạng comment.
fn mau_mac_dinh(machine: &str, peer: &str, salt_hex: &str) -> String {
    format!(
        r#"# x2clip — sửa xong nhớ giữ file này ở chmod 0600.
machine = "{machine}"
peer = "{peer}"
poll_secs = 30

# Bỏ comment để bật đồng bộ. Access key phải giới hạn đúng bucket này.
# [mailbox]
# endpoint = "https://<account-id>.r2.cloudflarestorage.com"
# bucket = "x2clip"
# region = "auto"
# access_key_id = ""
# secret_access_key = ""

[crypto]
# COPY Y HỆT DÒNG NÀY SANG MÁY KIA. Khác salt là khác khoá, mọi thứ nhận về
# sẽ giải mã hỏng và trông y như sai passphrase.
salt = "{salt_hex}"
# Đường dẫn file passphrase (chmod 0600). Bỏ trống thì đọc $X2CLIP_PASSPHRASE.
# passphrase_file = "~/.config/x2clip/passphrase"
"#
    )
}

/// Tạo file với quyền `0600` **ngay lúc tạo**. Ghi trước rồi `chmod` sau là
/// để hở một khoảnh khắc file đọc được bởi cả máy.
fn ghi_moi_0600(path: &Path, noi_dung: &str) -> Result<()> {
    use std::io::Write;
    let mut opt = std::fs::OpenOptions::new();
    opt.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opt.mode(0o600);
    }
    opt.open(path)?.write_all(noi_dung.as_bytes())?;
    Ok(())
}

/// [US-C5](../../docs/USER-STORIES.md) + N18c: file chứa secret rộng hơn
/// `0600` thì **từ chối chạy**, không phải cảnh báo rồi chạy tiếp. Cảnh báo
/// cuộn mất trong log; access key thì vẫn nằm đó cho mọi user trên máy đọc.
fn kiem_quyen_0600(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)?.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(anyhow!(
                "{} đang ở quyền {mode:o}, phải là 600 (có access key trong đó): chmod 600 {}",
                path.display(),
                path.display()
            ));
        }
    }
    Ok(())
}

/// `~/...` trong TOML là chuỗi thường, không ai bung hộ.
fn no_dau_ngã(p: &Path) -> PathBuf {
    match p.strip_prefix("~") {
        Ok(duoi) => match dirs::home_dir() {
            Some(home) => home.join(duoi),
            None => p.to_path_buf(),
        },
        Err(_) => p.to_path_buf(),
    }
}

impl Config {
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if !path.exists() {
            let host = hostname();
            let noi_dung = mau_mac_dinh(&host, "peer", &hex::encode(crate::crypto::random_salt()));
            ghi_moi_0600(path, &noi_dung)?;
            eprintln!(
                "x2clip: đã tạo config mặc định tại {} — chạy local-only cho tới khi điền [mailbox]",
                path.display()
            );
            return Ok(toml::from_str(&noi_dung)?);
        }

        kiem_quyen_0600(path)?;
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("không đọc được {}", path.display()))?;
        // Lỗi của `toml` có sẵn dòng:cột; chỉ cần đừng nuốt nó (T9).
        let cfg: Config =
            toml::from_str(&raw).map_err(|e| anyhow!("config {} lỗi: {e}", path.display()))?;
        cfg.kiem_tra_ten_may()?;
        Ok(cfg)
    }

    /// `machine == peer` nghĩa là daemon PUT vào hộp thư của chính nó rồi tự
    /// ingest lại — vòng lặp không bao giờ dừng, và nó hiện ra dưới dạng hoá
    /// đơn R2 chứ không phải app treo. Chỉ chặn khi đã bật `[mailbox]`: chưa
    /// bật thì `peer` còn là placeholder của file mẫu, chạy local-only vẫn đúng.
    fn kiem_tra_ten_may(&self) -> Result<()> {
        if self.mailbox.is_none() {
            return Ok(());
        }
        if self.machine == self.peer {
            return Err(anyhow!(
                "machine và peer đều là \"{}\" — máy sẽ tự gửi cho chính nó, vòng lặp vô hạn",
                self.machine
            ));
        }
        for (ten, gia_tri) in [("machine", &self.machine), ("peer", &self.peer)] {
            if gia_tri.is_empty() || gia_tri == "peer" || gia_tri == "may-nay" {
                return Err(anyhow!(
                    "{ten} còn là giá trị mẫu (\"{gia_tri}\") — đặt tên thật cho hai máy trước khi bật [mailbox]"
                ));
            }
        }
        Ok(())
    }

    /// Passphrase lấy từ env hoặc file — không bao giờ nằm trong config.
    pub fn passphrase(&self) -> Result<String> {
        if let Ok(p) = std::env::var("X2CLIP_PASSPHRASE") {
            return Ok(p);
        }
        let f = self
            .crypto
            .as_ref()
            .and_then(|c| c.passphrase_file.as_ref())
            .ok_or_else(|| {
                anyhow!("chưa có passphrase: đặt $X2CLIP_PASSPHRASE hoặc crypto.passphrase_file")
            })?;
        let f = no_dau_ngã(f);
        Ok(std::fs::read_to_string(&f)
            .with_context(|| format!("không đọc được passphrase file {}", f.display()))?
            .trim()
            .to_string())
    }

    pub fn salt(&self) -> Result<Vec<u8>> {
        let c = self
            .crypto
            .as_ref()
            .ok_or_else(|| anyhow!("config thiếu section [crypto]"))?;
        hex::decode(&c.salt).map_err(|e| anyhow!("crypto.salt không phải hex hợp lệ: {e}"))
    }

    /// Prefix hộp thư của chính máy này — nơi mình LIST/GET/DELETE.
    pub fn inbox_cua_minh(&self) -> String {
        format!("inbox/{}/", self.machine)
    }
    /// Prefix của máy kia — nơi mình PUT. Không bao giờ đọc.
    pub fn inbox_cua_peer(&self) -> String {
        format!("inbox/{}/", self.peer)
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "may-nay".to_string())
}
