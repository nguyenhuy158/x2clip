//! Vòng poll clipboard + echo guard. Luồng theo docs/ARCHITECTURE.md § 3.

use crate::clip::{hash_text, Clipboard};
use crate::{Store, MAX_TEXT_BYTES, POLL_INTERVAL};
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Ô nhớ giữ hash của giá trị **chính app vừa ghi** vào clipboard.
pub type EchoCell = Arc<Mutex<Option<String>>>;

pub struct Watcher<C: Clipboard> {
    clip: C,
    store: Store,
    last_seen: Option<String>,
    last_written: EchoCell,
}

impl<C: Clipboard> Watcher<C> {
    pub fn new(clip: C, store: Store) -> Self {
        Self {
            clip,
            store,
            last_seen: None,
            last_written: Arc::new(Mutex::new(None)),
        }
    }

    pub fn echo_cell(&self) -> EchoCell {
        Arc::clone(&self.last_written)
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Ghi một nội dung nhận từ nơi khác vào clipboard local.
    /// Phase 1 chỉ có test gọi; Phase 2 là đường về từ hộp thư.
    ///
    /// Ràng buộc thứ tự **bắt buộc**: set cờ trước, ghi clipboard sau.
    /// Đảo lại là có race — watcher có thể đọc được giá trị mới trước khi cờ kịp set.
    pub fn apply_remote(&mut self, text: &str) -> Result<()> {
        *self.last_written.lock().unwrap() = Some(hash_text(text));
        self.clip.set_text(text)
    }

    /// Một nhịp poll. Trả về id của item vừa thêm, `None` nếu không có gì mới.
    pub fn tick(&mut self) -> Result<Option<i64>> {
        let Some(text) = self.clip.get_text() else {
            return Ok(None);
        };
        if text.is_empty() {
            return Ok(None);
        }

        let hash = hash_text(&text);
        if self.last_seen.as_deref() == Some(hash.as_str()) {
            return Ok(None);
        }
        self.last_seen = Some(hash.clone());

        // Echo guard: đây là thứ chính mình vừa ghi, không phải người dùng copy.
        let mut written = self.last_written.lock().unwrap();
        if written.as_deref() == Some(hash.as_str()) {
            *written = None;
            return Ok(None);
        }
        drop(written);

        if text.len() > MAX_TEXT_BYTES {
            eprintln!(
                "x2clip: bỏ qua item {} byte, vượt N16 ({} byte) — không cắt bớt",
                text.len(),
                MAX_TEXT_BYTES
            );
            return Ok(None);
        }

        let (id, is_new) = self.store.upsert_text(&hash, &text)?;
        if is_new {
            self.store.prune()?;
        }
        Ok(Some(id))
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if let Err(e) = self.tick() {
                eprintln!("x2clip: lỗi khi poll clipboard: {e}");
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }
}
