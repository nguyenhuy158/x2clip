//! x2clip core — Phase 1: lịch sử clipboard local, một máy. Chưa có mạng.
//! Ngưỡng lấy từ docs/NFR.md.

pub mod clip;
pub mod config;
pub mod crypto;
pub mod mailbox;
pub mod store;
pub mod sync;
pub mod watcher;

pub use clip::{Clipboard, SystemClipboard};
pub use config::Config;
pub use mailbox::Mailbox;
pub use store::{Item, Store};
pub use sync::Syncer;
pub use watcher::Watcher;

use std::time::Duration;

/// N13 — chu kỳ poll.
pub const POLL_INTERVAL: Duration = Duration::from_millis(250);
/// N14 — giữ tối đa 1000 item hoặc 30 ngày, cái nào tới trước.
pub const MAX_ITEMS: usize = 1000;
pub const MAX_AGE_MS: i64 = 30 * 24 * 60 * 60 * 1000;
/// N16 — item text tối đa 1 MB. Vượt thì bỏ qua + log, không cắt bớt.
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
/// N15 — item ảnh tối đa 5 MB (đo trên PNG đã chuẩn hoá). Vượt thì **vẫn vào
/// lịch sử local**, đánh dấu không sync (US-A3), không cắt, không resize.
pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;
/// Cạnh dài nhất của thumbnail.
pub const THUMB_MAX_EDGE: u32 = 128;
/// Trần khi tải object về. Ảnh N15 5 MB đi trong JSON dưới dạng hex nên nở gấp
/// đôi; 16 MB cho rộng tay phần nonce/mac/JSON.
/// ponytail: hex chứ không base64 — `hex` đã có sẵn, base64 chỉ tiết kiệm 33%
/// băng thông. Đổi khi nào hoá đơn R2 thấy được.
pub const MAX_OBJECT_BYTES: usize = 16 * 1024 * 1024;

/// Đường dẫn DB mặc định. Phase 2 (US-C5) mới cho cấu hình.
pub fn default_db_path() -> anyhow::Result<std::path::PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("không tìm được thư mục data của user"))?
        .join("x2clip");
    std::fs::create_dir_all(&dir)?;
    // N21 — khoá cả thư mục: WAL sinh thêm `-wal`/`-shm` chứa nội dung vừa copy,
    // chmod riêng file .db là hụt.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(dir.join("x2clip.db"))
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock trước 1970")
        .as_millis() as i64
}
