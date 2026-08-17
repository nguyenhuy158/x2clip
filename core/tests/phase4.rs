//! Quality gate Phase 4 — các thao tác trên một item (US-B3, US-B4, US-B5).
//!
//! Ba chỗ dễ hỏng im lặng, và cả ba đều là "UPDATE/DELETE không khớp row nào":
//!
//! 1. `pin`/`rm` với id không tồn tại. SQLite không coi đó là lỗi, `execute`
//!    trả về 0 row rồi thôi. Nếu CLI không đọc con số đó thì gõ nhầm id sẽ in
//!    "đã xoá item 99" trong khi chẳng xoá gì — đúng loại lỗi im lặng mà
//!    NFR § hành vi khi lỗi cấm.
//! 2. Ghim chặn `prune` tự động nhưng **không** được chặn người dùng tự xoá.
//! 3. `copy <id>` nạp lại đúng nội dung cũ — ở đây kiểm phần DB (`lay`,
//!    `lay_blob`); phần clipboard thật nằm ở `clipboard_that.rs`.

use anyhow::Result;
use x2clip_core::clip::{giai_ma_png, hash_bytes, hash_text, ma_hoa_png};
use x2clip_core::store::SYNC_CHO_GUI;
use x2clip_core::Store;

fn them_text(s: &Store, body: &str) -> Result<i64> {
    Ok(s.upsert_text(&hash_text(body), body)?.0)
}

#[test]
fn lay_tra_ve_dung_item_va_none_khi_khong_co() -> Result<()> {
    let s = Store::open_memory()?;
    let id = them_text(&s, "xin chào")?;

    let item = s.lay(id)?.expect("vừa thêm xong phải lấy được");
    assert_eq!(item.body, "xin chào");
    assert_eq!(item.kind, "text");
    assert!(!item.pinned);

    assert!(s.lay(id + 999)?.is_none(), "id không có phải ra None");
    Ok(())
}

#[test]
fn pin_va_rm_bao_false_khi_id_khong_ton_tai() -> Result<()> {
    let s = Store::open_memory()?;
    let id = them_text(&s, "có thật")?;

    assert!(s.set_pinned(id, true)?);
    assert!(s.lay(id)?.unwrap().pinned);
    assert!(s.set_pinned(id, false)?);
    assert!(!s.lay(id)?.unwrap().pinned);

    // Con số này là thứ duy nhất ngăn CLI in "đã xoá" cho một id ma.
    assert!(!s.set_pinned(9999, true)?, "id ma mà báo thành công");
    assert!(!s.xoa(9999)?, "id ma mà báo xoá được");
    Ok(())
}

#[test]
fn xoa_duoc_ca_item_da_ghim() -> Result<()> {
    let s = Store::open_memory()?;
    let id = them_text(&s, "ghim rồi vẫn xoá được")?;
    s.set_pinned(id, true)?;

    assert!(s.xoa(id)?);
    assert!(s.lay(id)?.is_none());
    assert_eq!(s.count()?, 0);
    Ok(())
}

#[test]
fn copy_lai_anh_nap_dung_byte_cu() -> Result<()> {
    let s = Store::open_memory()?;
    let (r, c) = (40u32, 24u32);
    let rgba: Vec<u8> = (0..r * c)
        .flat_map(|i| [(i % 256) as u8, 9, 200, 255])
        .collect();
    let png = ma_hoa_png(r, c, &rgba)?;
    let id = s
        .upsert_image(
            &hash_bytes(&png),
            &format!("ảnh {r}x{c}"),
            &png,
            &png,
            SYNC_CHO_GUI,
            1_700_000_000_000,
        )?
        .0;

    // Đúng đường mà `x2clip copy <id>` đi.
    let blob = s.lay_blob(id)?.expect("phải còn blob");
    assert_eq!(blob, png, "blob lệch byte — dán ra sẽ khác ảnh gốc");
    let (rr, cc, _) = giai_ma_png(&blob)?;
    assert_eq!((rr, cc), (r, c));
    Ok(())
}
