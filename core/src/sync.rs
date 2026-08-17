//! Luồng gửi/nhận qua hộp thư — docs/ARCHITECTURE.md § 3 và § 6.

use crate::clip::hash_text;
use crate::crypto::SecretKey;
use crate::mailbox::Mailbox;
use crate::store::{Store, SYNC_DA_GUI};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Nội dung **trước khi mã hoá**. `kind`/`hash`/`ts` nằm trong ciphertext,
/// không bao giờ làm metadata của object (ADR-0005 C6).
#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub v: u32,
    pub kind: String,
    pub hash: String,
    pub body: String,
    pub ts: i64,
}

pub const PHIEN_BAN: u32 = 1;

impl Payload {
    fn text(body: &str, ts: i64) -> Self {
        Self {
            v: PHIEN_BAN,
            kind: "text".to_string(),
            hash: hash_text(body),
            body: body.to_string(),
            ts,
        }
    }

    fn kiem_tra(&self) -> Result<()> {
        if self.v != PHIEN_BAN {
            return Err(anyhow!("phiên bản payload lạ: {}", self.v));
        }
        if self.kind != "text" {
            return Err(anyhow!("kind chưa hỗ trợ ở phase này: {}", self.kind));
        }
        if self.hash != hash_text(&self.body) {
            return Err(anyhow!("hash không khớp nội dung"));
        }
        Ok(())
    }
}

pub struct Syncer<M: Mailbox> {
    mailbox: M,
    key: SecretKey,
    /// `inbox/<máy mình>/` — chỉ mình LIST/GET/DELETE ở đây.
    hop_thu_minh: String,
    /// `inbox/<máy kia>/` — chỉ PUT, không bao giờ đọc.
    hop_thu_peer: String,
}

impl<M: Mailbox> Syncer<M> {
    pub fn new(mailbox: M, key: SecretKey, hop_thu_minh: String, hop_thu_peer: String) -> Self {
        Self {
            mailbox,
            key,
            hop_thu_minh,
            hop_thu_peer,
        }
    }

    pub fn mailbox(&self) -> &M {
        &self.mailbox
    }

    /// Đẩy hàng chờ lên hộp thư của máy kia. Lỗi thì **dừng và giữ nguyên
    /// hàng chờ** — không mất item, lần poll sau gửi tiếp (N8).
    pub fn push_pending(&self, store: &Store) -> Result<usize> {
        let mut n = 0;
        for item in store.cho_gui()? {
            let payload = Payload::text(&item.body, item.updated_at);
            let blob = self
                .key
                .encrypt(serde_json::to_string(&payload)?.as_bytes())?;
            // ULID ngẫu nhiên: key không được lộ hash hay nội dung (C6, T15).
            let key = format!("{}{}", self.hop_thu_peer, Ulid::generate());
            self.mailbox.put(&key, &blob)?;
            store.dat_synced(item.id, SYNC_DA_GUI)?;
            n += 1;
        }
        Ok(n)
    }

    /// Đọc hết hộp thư của mình. Trả về nội dung **duy nhất** cần đưa lên
    /// clipboard — item có `ts` lớn nhất trong lô (T11). Cả lô đều vào lịch sử.
    ///
    /// Người gọi có trách nhiệm set cờ echo **trước** khi ghi clipboard.
    pub fn ingest(&self, store: &Store) -> Result<Option<String>> {
        let mut moi_nhat: Option<(i64, String)> = None;

        for key in self.mailbox.list(&self.hop_thu_minh)? {
            if store.da_thay(&key)? {
                // Lần trước DELETE hỏng nên nó còn nằm đây. Dọn lại, đừng xử lý lại.
                let _ = self.mailbox.delete(&key);
                continue;
            }

            let blob = match self.mailbox.get(&key) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("x2clip: không tải được {key}: {e} — giữ lại, thử lần sau");
                    continue;
                }
            };

            // Giải mã hỏng hay payload hỏng: log, **giữ object**, không ghi đi
            // đâu cả (ADR-0005 C4, T8, T14). Cũng không đánh dấu `seen` — biết
            // đâu chỉ là sai passphrase tạm thời.
            let payload = match self
                .key
                .decrypt(&blob)
                .and_then(|pt| Ok(serde_json::from_slice::<Payload>(&pt)?))
                .and_then(|p| p.kiem_tra().map(|_| p))
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("x2clip: bỏ qua {key}: {e} — giữ lại object, không xoá");
                    continue;
                }
            };

            store.upsert_remote(&payload.hash, &payload.body, payload.ts)?;
            store.ghi_nhan_da_thay(&key)?;
            // DELETE hỏng cũng không sao, `seen` đã chặn xử lý lại (T13).
            let _ = self.mailbox.delete(&key);

            if moi_nhat.as_ref().is_none_or(|(ts, _)| payload.ts > *ts) {
                moi_nhat = Some((payload.ts, payload.body));
            }
        }

        store.prune()?;
        store.prune_seen()?;
        Ok(moi_nhat.map(|(_, body)| body))
    }
}
