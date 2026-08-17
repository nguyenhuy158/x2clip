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
/// Trần khi tải object về: 1 MB text + JSON + nonce/mac, cho rộng tay gấp bốn.
/// Chặn ở đây để một object rác khổng lồ không nuốt hết RAM.
pub const MAX_OBJECT_BYTES: usize = 4 * 1024 * 1024;

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
