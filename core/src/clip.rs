//! Truy cập clipboard hệ điều hành. Trait để watcher test được không cần GUI.

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

/// Ảnh đã chuẩn hoá về PNG. Hash luôn tính trên `png` — **không** trên RGBA
/// thô: hai máy phải ra cùng hash cho cùng bức ảnh, mà chống dội (`last_written`)
/// và `hash UNIQUE` trong DB đều dựa vào đúng con số đó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anh {
    pub rong: u32,
    pub cao: u32,
    pub png: Vec<u8>,
}

pub trait Clipboard {
    /// `None` khi clipboard rỗng hoặc không phải text.
    fn get_text(&mut self) -> Option<String>;
    fn set_text(&mut self, text: &str) -> Result<()>;
    /// `None` khi clipboard không có ảnh, hoặc encode PNG hỏng.
    fn get_image(&mut self) -> Option<Anh>;
    fn set_image(&mut self, anh: &Anh) -> Result<()>;
    /// N22 — nội dung được đánh dấu nhạy cảm (password manager). Mặc định
    /// `false`: nền tảng nào không có khái niệm này thì không có gì để bỏ qua.
    fn nhay_cam(&mut self) -> bool {
        false
    }
}

pub fn hash_text(text: &str) -> String {
    hash_bytes(text.as_bytes())
}

pub fn hash_bytes(b: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b);
    format!("{:x}", h.finalize())
}

/// RGBA8 → PNG. `png` crate encode deterministic với cùng input, nên hai lần
/// copy cùng một ảnh ra cùng hash — điều kiện cần để `upsert` không đẻ row mới.
pub fn ma_hoa_png(rong: u32, cao: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let can = (rong as usize)
        .checked_mul(cao as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| anyhow!("kích thước ảnh tràn số: {rong}x{cao}"))?;
    if rgba.len() != can {
        return Err(anyhow!(
            "RGBA {} byte không khớp {rong}x{cao} (cần {can})",
            rgba.len()
        ));
    }
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, rong, cao);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(rgba)?;
    Ok(out)
}

/// PNG → RGBA8. Trả `(rộng, cao, rgba)`.
pub fn giai_ma_png(png_bytes: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
    let mut reader = png::Decoder::new(png_bytes).read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(anyhow!(
            "PNG không phải RGBA8 ({:?}/{:?}) — bên gửi phải chuẩn hoá trước",
            info.color_type,
            info.bit_depth
        ));
    }
    Ok((info.width, info.height, buf))
}

/// Thumbnail PNG, cạnh dài nhất `THUMB_MAX_EDGE`.
///
/// ponytail: nearest-neighbour, không lọc — thumbnail 128px hiện trong danh
/// sách, răng cưa không ai thấy. Đổi sang lanczos khi nào có UI phóng to nó.
pub fn thu_nho(anh: &Anh) -> Result<Vec<u8>> {
    let (rong, cao, rgba) = giai_ma_png(&anh.png)?;
    let canh = rong.max(cao).max(1);
    if canh <= crate::THUMB_MAX_EDGE {
        return Ok(anh.png.clone());
    }
    let (tr, tc) = (
        (rong * crate::THUMB_MAX_EDGE / canh).max(1),
        (cao * crate::THUMB_MAX_EDGE / canh).max(1),
    );
    let mut out = Vec::with_capacity((tr * tc * 4) as usize);
    for y in 0..tc {
        let sy = (y as u64 * cao as u64 / tc as u64) as u32;
        for x in 0..tr {
            let sx = (x as u64 * rong as u64 / tr as u64) as u32;
            let i = ((sy as usize * rong as usize) + sx as usize) * 4;
            out.extend_from_slice(&rgba[i..i + 4]);
        }
    }
    ma_hoa_png(tr, tc, &out)
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

    fn get_image(&mut self) -> Option<Anh> {
        let img = self.0.get_image().ok()?;
        let (rong, cao) = (img.width as u32, img.height as u32);
        // Encode hỏng thì coi như không có ảnh: tick sau sẽ thử lại, còn hơn
        // đẩy lỗi lên làm chết cả vòng poll clipboard.
        let png = ma_hoa_png(rong, cao, &img.bytes).ok()?;
        Some(Anh { rong, cao, png })
    }

    fn set_image(&mut self, anh: &Anh) -> Result<()> {
        let (rong, cao, rgba) = giai_ma_png(&anh.png)?;
        self.0.set_image(arboard::ImageData {
            width: rong as usize,
            height: cao as usize,
            bytes: rgba.into(),
        })?;
        Ok(())
    }

    /// macOS: password manager đánh dấu `org.nspasteboard.ConcealedType`.
    /// arboard không cho đọc type tuỳ ý nên phải hỏi thẳng NSPasteboard.
    #[cfg(target_os = "macos")]
    fn nhay_cam(&mut self) -> bool {
        use objc2_app_kit::NSPasteboard;
        use objc2_foundation::NSString;
        let pb = NSPasteboard::generalPasteboard();
        let Some(types) = pb.types() else {
            return false;
        };
        types.containsObject(&NSString::from_str("org.nspasteboard.ConcealedType"))
    }
}
