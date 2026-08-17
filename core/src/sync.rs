//! Luồng gửi/nhận qua hộp thư — docs/ARCHITECTURE.md § 3 và § 6.

use crate::clip::{giai_ma_png, hash_bytes, hash_text, thu_nho, Anh};
use crate::crypto::SecretKey;
use crate::mailbox::Mailbox;
use crate::store::{Store, SYNC_DA_GUI, SYNC_KHONG_GUI};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Nội dung **trước khi mã hoá**. `kind`/`hash`/`ts` nằm trong ciphertext,
/// không bao giờ làm metadata của object (ADR-0005 C6).
///
/// Với `kind = "image"`, `body` là PNG dạng hex và `hash` là hash của **PNG
/// đã giải hex**, không phải của chuỗi hex. Hai máy phải ra cùng con số với
/// cùng bức ảnh, và đó cũng đúng là hash `Watcher` tính khi đọc clipboard —
/// lệch một chút là ảnh nhận về dội ngược lại peer.
#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub v: u32,
    pub kind: String,
    pub hash: String,
    pub body: String,
    pub ts: i64,
}

pub const PHIEN_BAN: u32 = 1;

/// Thứ `ingest` muốn đưa lên clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NhanVe {
    Text(String),
    Anh(Anh),
}

impl NhanVe {
    pub fn text(&self) -> Option<&str> {
        match self {
            NhanVe::Text(t) => Some(t),
            NhanVe::Anh(_) => None,
        }
    }
}

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

    fn anh(png: &[u8], ts: i64) -> Self {
        Self {
            v: PHIEN_BAN,
            kind: "image".to_string(),
            hash: hash_bytes(png),
            body: hex::encode(png),
            ts,
        }
    }

    /// Trả về PNG đã giải hex khi là ảnh — giải một lần ở đây thay vì để người
    /// gọi giải lại lần nữa sau khi đã kiểm hash.
    fn kiem_tra(&self) -> Result<Option<Vec<u8>>> {
        if self.v != PHIEN_BAN {
            return Err(anyhow!("phiên bản payload lạ: {}", self.v));
        }
        match self.kind.as_str() {
            "text" => {
                if self.hash != hash_text(&self.body) {
                    return Err(anyhow!("hash không khớp nội dung"));
                }
                Ok(None)
            }
            "image" => {
                let png =
                    hex::decode(&self.body).map_err(|e| anyhow!("body không phải hex: {e}"))?;
                if self.hash != hash_bytes(&png) {
                    return Err(anyhow!("hash không khớp nội dung"));
                }
                Ok(Some(png))
            }
            k => Err(anyhow!("kind không hiểu: {k}")),
        }
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
            let payload = if item.kind == "image" {
                let Some(png) = store.lay_blob(item.id)? else {
                    // Row ảnh mà không có blob = DB hỏng. Đừng thử lại mãi.
                    eprintln!("x2clip: item ảnh {} mất blob — bỏ khỏi hàng chờ", item.id);
                    store.dat_synced(item.id, SYNC_KHONG_GUI)?;
                    continue;
                };
                Payload::anh(&png, item.updated_at)
            } else {
                Payload::text(&item.body, item.updated_at)
            };
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
    pub fn ingest(&self, store: &Store) -> Result<Option<NhanVe>> {
        let mut moi_nhat: Option<(i64, NhanVe)> = None;

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
            let (payload, png) = match self
                .key
                .decrypt(&blob)
                .and_then(|pt| Ok(serde_json::from_slice::<Payload>(&pt)?))
                .and_then(|p| p.kiem_tra().map(|png| (p, png)))
            {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("x2clip: bỏ qua {key}: {e} — giữ lại object, không xoá");
                    continue;
                }
            };

            let nhan = match png {
                None => {
                    store.upsert_remote(&payload.hash, &payload.body, payload.ts)?;
                    NhanVe::Text(payload.body)
                }
                Some(png) => {
                    // PNG hỏng thì cùng luật với giải mã hỏng: giữ object, không
                    // đánh dấu `seen`, không ghi gì vào DB (C4).
                    let (rong, cao) = match giai_ma_png(&png) {
                        Ok((r, c, _)) => (r, c),
                        Err(e) => {
                            eprintln!("x2clip: bỏ qua {key}: PNG hỏng ({e}) — giữ lại object");
                            continue;
                        }
                    };
                    let anh = Anh { rong, cao, png };
                    store.upsert_image(
                        &payload.hash,
                        &format!("ảnh {rong}x{cao}"),
                        &anh.png,
                        &thu_nho(&anh).unwrap_or_default(),
                        // Nhận về thì **không bao giờ** PUT ngược lại.
                        SYNC_KHONG_GUI,
                        payload.ts,
                    )?;
                    NhanVe::Anh(anh)
                }
            };

            store.ghi_nhan_da_thay(&key)?;
            // DELETE hỏng cũng không sao, `seen` đã chặn xử lý lại (T13).
            let _ = self.mailbox.delete(&key);

            if moi_nhat.as_ref().is_none_or(|(ts, _)| payload.ts > *ts) {
                moi_nhat = Some((payload.ts, nhan));
            }
        }

        store.prune()?;
        store.prune_seen()?;
        Ok(moi_nhat.map(|(_, nhan)| nhan))
    }
}
