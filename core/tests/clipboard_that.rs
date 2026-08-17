//! Kiểm trên clipboard **thật** — `#[ignore]` vì cần GUI session, CI headless
//! sẽ hỏng. Chạy tay: `cargo test --test clipboard_that -- --ignored`.
//!
//! Hai câu hỏi mà `FakeClipboard` không trả lời được, và cả hai đều quyết định
//! Phase 3 có chạy hay không:
//!
//! 1. Ghi ảnh xong đọc lại có ra **đúng byte PNG cũ** không. Cờ chống dội giữ
//!    `hash(PNG)`; nếu hệ điều hành trả về RGBA khác đi một chút (premultiply
//!    alpha, đổi colorspace) thì hash trượt → ảnh nhận từ peer thành item mới
//!    → PUT ngược lại → peer nhận → PUT lại. Ping-pong vô hạn, mỗi vòng vài MB.
//! 2. `nhay_cam()` có thật sự đọc được `org.nspasteboard.ConcealedType` không.
//!    Test bằng chính arboard: nó có sẵn đường **ghi** cờ đó.

use anyhow::Result;
use x2clip_core::clip::{hash_bytes, ma_hoa_png, Anh, Clipboard};
use x2clip_core::SystemClipboard;

fn anh_mau(rong: u32, cao: u32) -> Anh {
    let mut rgba = Vec::with_capacity((rong * cao * 4) as usize);
    for y in 0..cao {
        for x in 0..rong {
            rgba.extend_from_slice(&[(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255]);
        }
    }
    Anh {
        rong,
        cao,
        png: ma_hoa_png(rong, cao, &rgba).unwrap(),
    }
}

#[test]
#[ignore = "cần GUI session"]
fn ghi_anh_roi_doc_lai_phai_ra_dung_hash() -> Result<()> {
    let mut c = SystemClipboard::new()?;
    let a = anh_mau(160, 90);
    c.set_image(&a)?;
    let b = c.get_image().expect("ghi xong phải đọc lại được ảnh");
    assert_eq!((b.rong, b.cao), (a.rong, a.cao), "kích thước pixel đổi");
    assert_eq!(
        hash_bytes(&b.png),
        hash_bytes(&a.png),
        "hash lệch sau một vòng clipboard — cờ chống dội sẽ trượt, ảnh sẽ ping-pong"
    );
    Ok(())
}

#[test]
#[ignore = "cần GUI session"]
#[cfg(target_os = "macos")]
fn nhay_cam_doc_duoc_co_that_tren_pasteboard() -> Result<()> {
    let mut c = SystemClipboard::new()?;

    c.set_text("text thường")?;
    assert!(!c.nhay_cam(), "text thường không được coi là nhạy cảm");

    // arboard tự đặt `org.nspasteboard.ConcealedType` ở đường này — đúng thứ
    // password manager làm.
    use arboard::SetExtApple;
    arboard::Clipboard::new()?
        .set()
        .exclude_from_history()
        .text("gia-lam-password")?;
    assert!(
        c.nhay_cam(),
        "không đọc được ConcealedType — N22 chỉ là bản nháp"
    );
    Ok(())
}
