//! Vòng poll clipboard + echo guard. Luồng theo docs/ARCHITECTURE.md § 3.

use crate::clip::{hash_bytes, hash_text, thu_nho, Anh, Clipboard};
use crate::store::{SYNC_CHO_GUI, SYNC_KHONG_GUI};
use crate::{now_ms, Store, MAX_IMAGE_BYTES, MAX_TEXT_BYTES, POLL_INTERVAL};
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

    /// Chỉ để test soi clipboard giả.
    pub fn clip_mut(&mut self) -> &mut C {
        &mut self.clip
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

    /// Đường về từ hộp thư cho ảnh. Cùng ràng buộc thứ tự với `apply_remote`,
    /// và hash phải là hash của **PNG** — đúng thứ `tick` sẽ tính lại khi đọc
    /// clipboard, nếu không thì ảnh nhận về sẽ dội ngược lại peer (N7).
    pub fn apply_remote_image(&mut self, anh: &Anh) -> Result<()> {
        *self.last_written.lock().unwrap() = Some(hash_bytes(&anh.png));
        self.clip.set_image(anh)
    }

    /// `true` nếu hash này đáng xử lý tiếp: chưa thấy ở nhịp trước, và không
    /// phải thứ chính app vừa ghi vào clipboard.
    fn moi(&mut self, hash: &str) -> bool {
        if self.last_seen.as_deref() == Some(hash) {
            return false;
        }
        self.last_seen = Some(hash.to_string());

        let mut written = self.last_written.lock().unwrap();
        if written.as_deref() == Some(hash) {
            *written = None;
            return false;
        }
        true
    }

    /// Một nhịp poll. Trả về id của item vừa thêm, `None` nếu không có gì mới.
    pub fn tick(&mut self) -> Result<Option<i64>> {
        // N22 trước mọi thứ khác: nội dung nhạy cảm không được chạm vào DB,
        // kể cả hash. Không cập nhật `last_seen` — không có gì để nhớ.
        if self.clip.nhay_cam() {
            return Ok(None);
        }

        match self.clip.get_text() {
            Some(text) if !text.is_empty() => self.tick_text(text),
            // Text rỗng vẫn có thể kèm ảnh: nhiều app đặt cả hai flavour.
            _ => self.tick_anh(),
        }
    }

    fn tick_text(&mut self, text: String) -> Result<Option<i64>> {
        let hash = hash_text(&text);
        if !self.moi(&hash) {
            return Ok(None);
        }

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

    fn tick_anh(&mut self) -> Result<Option<i64>> {
        let Some(anh) = self.clip.get_image() else {
            return Ok(None);
        };
        let hash = hash_bytes(&anh.png);
        if !self.moi(&hash) {
            return Ok(None);
        }

        // Vượt N15 thì **vẫn lưu**, chỉ không gửi (US-A3). Khác hẳn text: text
        // quá cỡ bị chặn hẳn, ảnh quá cỡ vẫn là lịch sử dùng được ở máy này.
        let synced = if anh.png.len() > MAX_IMAGE_BYTES {
            eprintln!(
                "x2clip: ảnh {} byte vượt N15 ({MAX_IMAGE_BYTES} byte) — giữ ở lịch sử local, không sync",
                anh.png.len()
            );
            SYNC_KHONG_GUI
        } else {
            SYNC_CHO_GUI
        };

        let thumb = thu_nho(&anh).unwrap_or_default();
        let mo_ta = format!("ảnh {}x{}", anh.rong, anh.cao);
        let (id, is_new) =
            self.store
                .upsert_image(&hash, &mo_ta, &anh.png, &thumb, synced, now_ms())?;
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
