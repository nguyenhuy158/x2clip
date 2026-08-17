//! T1, T4, T5 của docs/TEST-PLAN.md — quality gate Phase 1.

use anyhow::Result;
use x2clip_core::clip::{hash_text, Clipboard};
use x2clip_core::watcher::EchoCell;
use x2clip_core::{Store, Watcher, MAX_ITEMS};

#[derive(Default)]
struct FakeClipboard {
    text: Option<String>,
    /// Nếu gắn, `set_text` sẽ soi cờ echo **ngay lúc ghi** để bắt lỗi đảo thứ tự.
    cell: Option<EchoCell>,
    co_set_truoc_khi_ghi: bool,
}

impl Clipboard for FakeClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.text.clone()
    }
    fn set_text(&mut self, text: &str) -> Result<()> {
        if let Some(cell) = &self.cell {
            self.co_set_truoc_khi_ghi =
                cell.lock().unwrap().as_deref() == Some(hash_text(text).as_str());
        }
        self.text = Some(text.to_string());
        Ok(())
    }
    // Phase 1 chỉ có text; ảnh nằm ở phase3.rs với fake riêng.
    fn get_image(&mut self) -> Option<x2clip_core::clip::Anh> {
        None
    }
    fn set_image(&mut self, _: &x2clip_core::clip::Anh) -> Result<()> {
        unreachable!("test phase 1 không ghi ảnh")
    }
}

/// T1 — echo guard: nội dung do chính app ghi vào clipboard không được
/// quay lại thành item mới.
#[test]
fn t1_echo_guard_khong_sinh_item() -> Result<()> {
    let mut w = Watcher::new(FakeClipboard::default(), Store::open_memory()?);

    w.apply_remote("nhận từ hộp thư")?;
    assert_eq!(w.tick()?, None, "item nhận về không được coi là copy mới");
    assert_eq!(w.store().count()?, 0);

    // Nhiều lượt nhận liên tiếp cũng không được lọt cái nào.
    let mut w2 = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    w2.apply_remote("a")?;
    w2.tick()?;
    w2.apply_remote("b")?;
    w2.tick()?;
    assert_eq!(w2.store().count()?, 0, "không có echo nào lọt qua");
    Ok(())
}

/// T1 (phần thứ tự) — `apply_remote` phải set cờ **trước** khi ghi clipboard.
/// Đảo lại thì test này đỏ; các assert count() ở trên thì không bắt được.
#[test]
fn t1c_apply_remote_set_co_truoc_khi_ghi() -> Result<()> {
    let mut w = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    let cell = w.echo_cell();
    w.clip_mut().cell = Some(cell);

    w.apply_remote("nhận từ hộp thư")?;
    assert!(
        w.clip_mut().co_set_truoc_khi_ghi,
        "cờ echo phải có sẵn tại thời điểm set_text — nếu không là race"
    );
    Ok(())
}

#[test]
fn t1b_copy_that_su_thi_duoc_ghi() -> Result<()> {
    let mut clip = FakeClipboard::default();
    clip.set_text("người dùng copy")?;
    let mut w = Watcher::new(clip, Store::open_memory()?);

    assert!(w.tick()?.is_some());
    assert_eq!(w.store().count()?, 1);
    assert_eq!(w.tick()?, None, "clipboard không đổi thì không thêm gì");
    Ok(())
}

/// T4 — prune không đụng item đã ghim.
#[test]
fn t4_prune_khong_dung_pinned() -> Result<()> {
    let store = Store::open_memory()?;

    let (pinned_id, _) = store.upsert_text(&hash_text("ghim"), "ghim")?;
    store.set_pinned(pinned_id, true)?;
    let (oldest_id, _) = store.upsert_text(&hash_text("cũ nhất"), "cũ nhất")?;

    for i in 0..MAX_ITEMS {
        let body = format!("item {i}");
        store.upsert_text(&hash_text(&body), &body)?;
    }

    store.prune()?;

    let ids: Vec<i64> = store.list(MAX_ITEMS + 10)?.iter().map(|i| i.id).collect();
    assert!(ids.contains(&pinned_id), "item đã ghim phải sống sót");
    assert!(
        !ids.contains(&oldest_id),
        "item cũ nhất chưa ghim phải bị xoá"
    );
    assert_eq!(ids.len(), MAX_ITEMS + 1, "1000 chưa ghim + 1 đã ghim");
    Ok(())
}

/// T5 — copy trùng: một row, `updated_at` tăng, `created_at` giữ nguyên.
#[test]
fn t5_copy_trung_khong_sinh_row_moi() -> Result<()> {
    let store = Store::open_memory()?;
    let h = hash_text("cùng nội dung");

    let (id1, is_new) = store.upsert_text(&h, "cùng nội dung")?;
    assert!(is_new);
    let before = store.list(1)?[0].clone();

    std::thread::sleep(std::time::Duration::from_millis(2));

    let (id2, is_new) = store.upsert_text(&h, "cùng nội dung")?;
    assert_eq!(id1, id2);
    assert!(!is_new);

    let after = store.list(1)?[0].clone();
    assert_eq!(store.count()?, 1, "chỉ một entry");
    assert_eq!(after.created_at, before.created_at);
    assert!(after.updated_at > before.updated_at);
    Ok(())
}

#[test]
fn search_khong_phan_biet_hoa_thuong_va_moi_nhat_len_dau() -> Result<()> {
    let store = Store::open_memory()?;
    for body in ["Xin chào", "không liên quan", "xin CHÀO lần hai"] {
        store.upsert_text(&hash_text(body), body)?;
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    let hits = store.search("XIN chào")?;
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].body, "xin CHÀO lần hai", "mới nhất lên đầu");
    assert!(store.search("zzz")?.is_empty());
    Ok(())
}
