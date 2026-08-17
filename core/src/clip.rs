//! Truy cập clipboard hệ điều hành. Trait để watcher test được không cần GUI.

use anyhow::Result;
use sha2::{Digest, Sha256};

pub trait Clipboard {
    /// `None` khi clipboard rỗng hoặc không phải text.
    fn get_text(&mut self) -> Option<String>;
    fn set_text(&mut self, text: &str) -> Result<()>;
}

pub fn hash_text(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("{:x}", h.finalize())
}

pub struct SystemClipboard(arboard::Clipboard);

impl SystemClipboard {
    /// Giữ instance này sống suốt đời daemon: trên X11 clipboard là owner-based,
    /// tiến trình ghi mà thoát là mất nội dung (spike 0.2).
    pub fn new() -> Result<Self> {
        Ok(Self(arboard::Clipboard::new()?))
    }
}

impl Clipboard for SystemClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.0.get_text().ok()
    }

    fn set_text(&mut self, text: &str) -> Result<()> {
        self.0.set_text(text.to_string())?;
        Ok(())
    }
}
