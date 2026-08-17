//! Quality gate Phase 3 — ảnh (US-A3, US-C4) của docs/ROADMAP.md.
//!
//! Cái mà bộ test này canh, theo đúng thứ tự dễ hỏng nhất:
//!
//! 1. **Pixel không đổi.** Exit criterion nói "cùng kích thước pixel" — nếu
//!    encode/decode PNG lệch một byte thì hash lệch, và hash lệch thì mọi thứ
//!    khác (chống dội, `hash UNIQUE`) hỏng theo mà không kêu.
//! 2. **Ảnh quá N15 vẫn vào lịch sử.** Khác hẳn text: text quá cỡ bị chặn hẳn.
//!    Nếu nó lỡ nằm ở `synced = 0` thì mỗi vòng poll lại thử PUT một lần —
//!    hỏng mãi, im lặng.
//! 3. **Ảnh nhận về không dội ngược lại peer** (N7 = 0 echo).
//! 4. **N22** — nội dung đánh dấu nhạy cảm không chạm vào DB, kể cả hash.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use x2clip_core::clip::{giai_ma_png, hash_bytes, ma_hoa_png, thu_nho, Anh, Clipboard};
use x2clip_core::crypto::{derive_key, SecretKey};
use x2clip_core::mailbox::{Mailbox, MailboxError, MailboxResult};
use x2clip_core::store::{SYNC_CHO_GUI, SYNC_KHONG_GUI};
use x2clip_core::sync::NhanVe;
use x2clip_core::{Store, Syncer, Watcher, MAX_IMAGE_BYTES, THUMB_MAX_EDGE};

// ── đồ giả ─────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct FakeMailbox {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    so_lan_put: Arc<AtomicUsize>,
}

impl Mailbox for FakeMailbox {
    fn put(&self, key: &str, body: &[u8]) -> MailboxResult<()> {
        self.so_lan_put.fetch_add(1, Ordering::SeqCst);
        self.objects
            .lock()
            .unwrap()
            .insert(key.to_string(), body.to_vec());
        Ok(())
    }
    fn list(&self, prefix: &str) -> MailboxResult<Vec<String>> {
        let o = self.objects.lock().unwrap();
        let mut k: Vec<String> = o
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        k.sort();
        Ok(k)
    }
    fn get(&self, key: &str) -> MailboxResult<Vec<u8>> {
        self.objects
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| MailboxError::Other(format!("không có {key}")))
    }
    fn delete(&self, key: &str) -> MailboxResult<()> {
        self.objects.lock().unwrap().remove(key);
        Ok(())
    }
}

fn khoa() -> SecretKey {
    derive_key("cai-nay-la-passphrase-that-cua-toi", &[7u8; 16]).unwrap()
}

fn syncer(mb: FakeMailbox, minh: &str, peer: &str) -> Syncer<FakeMailbox> {
    Syncer::new(
        mb,
        khoa(),
        format!("inbox/{minh}/"),
        format!("inbox/{peer}/"),
    )
}

#[derive(Default)]
struct FakeClipboard {
    text: Option<String>,
    anh: Option<Anh>,
    nhay_cam: bool,
}

impl Clipboard for FakeClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.text.clone()
    }
    fn set_text(&mut self, text: &str) -> Result<()> {
        self.text = Some(text.to_string());
        self.anh = None;
        Ok(())
    }
    fn get_image(&mut self) -> Option<Anh> {
        self.anh.clone()
    }
    fn set_image(&mut self, anh: &Anh) -> Result<()> {
        self.anh = Some(anh.clone());
        self.text = None;
        Ok(())
    }
    fn nhay_cam(&mut self) -> bool {
        self.nhay_cam
    }
}

/// Ảnh gradient `rong x cao` — nội dung khác nhau theo pixel nên sai một pixel
/// là so sánh phát hiện được.
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

// ── 1. pixel không đổi ─────────────────────────────────────────────────────

#[test]
fn png_round_trip_giu_nguyen_tung_pixel() -> Result<()> {
    for (r, c) in [(1, 1), (7, 3), (64, 64), (321, 197)] {
        let anh = anh_mau(r, c);
        let (rr, cc, rgba) = giai_ma_png(&anh.png)?;
        assert_eq!((rr, cc), (r, c), "kích thước lệch ở {r}x{c}");
        assert_eq!(rgba.len(), (r * c * 4) as usize);
        // Encode lại phải ra đúng byte cũ — nếu không thì cùng một bức ảnh sẽ
        // ra hai hash khác nhau và mỗi lần copy lại đẻ một row mới.
        assert_eq!(ma_hoa_png(r, c, &rgba)?, anh.png, "encode không ổn định");
    }
    Ok(())
}

#[test]
fn ma_hoa_png_tu_choi_rgba_sai_do_dai() {
    assert!(ma_hoa_png(4, 4, &[0u8; 10]).is_err());
}

#[test]
fn thumbnail_khong_vuot_canh_toi_da() -> Result<()> {
    let (tr, tc, _) = giai_ma_png(&thu_nho(&anh_mau(1000, 400))?)?;
    assert!(
        tr.max(tc) <= THUMB_MAX_EDGE,
        "thumbnail {tr}x{tc} vượt {THUMB_MAX_EDGE}"
    );
    // Ảnh vốn đã nhỏ thì giữ nguyên, không phóng to.
    let nho = anh_mau(20, 10);
    assert_eq!(thu_nho(&nho)?, nho.png);
    Ok(())
}

// ── 2. N15 — quá cỡ vẫn vào lịch sử, chỉ không gửi ─────────────────────────

#[test]
fn anh_qua_n15_vao_lich_su_nhung_khong_gui() -> Result<()> {
    let mut w = Watcher::new(FakeClipboard::default(), Store::open_memory()?);

    // Nhiễu để PNG không nén được xuống dưới N15.
    let (r, c) = (1400u32, 1400u32);
    let mut rgba = Vec::with_capacity((r * c * 4) as usize);
    let mut s: u32 = 12345;
    for _ in 0..(r * c) {
        s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        rgba.extend_from_slice(&[(s >> 24) as u8, (s >> 16) as u8, (s >> 8) as u8, 255]);
    }
    let png = ma_hoa_png(r, c, &rgba)?;
    assert!(
        png.len() > MAX_IMAGE_BYTES,
        "test tự hỏng: ảnh mẫu chỉ {} byte, chưa vượt N15",
        png.len()
    );

    w.clip_mut().set_image(&Anh {
        rong: r,
        cao: c,
        png,
    })?;
    let id = w.tick()?.expect("ảnh quá cỡ vẫn phải vào lịch sử");

    assert_eq!(w.store().list(10)?.len(), 1);
    assert!(
        w.store().cho_gui()?.is_empty(),
        "ảnh quá N15 không được nằm trong hàng chờ — sẽ thử PUT lại mỗi vòng poll"
    );
    assert!(
        w.store().lay_blob(id)?.is_some(),
        "blob vẫn phải giữ nguyên"
    );
    Ok(())
}

#[test]
fn anh_vua_co_thi_vao_hang_cho() -> Result<()> {
    let mut w = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    let anh = anh_mau(200, 150);
    w.clip_mut().set_image(&anh)?;
    w.tick()?.expect("phải vào lịch sử");

    let cho = w.store().cho_gui()?;
    assert_eq!(cho.len(), 1);
    assert_eq!(cho[0].kind, "image");
    assert_eq!(
        cho[0].body, "ảnh 200x150",
        "preview phải in được không cần blob"
    );
    Ok(())
}

// ── 3. round-trip qua hộp thư, không dội ngược ─────────────────────────────

#[test]
fn anh_di_qua_hop_thu_nguyen_ven_va_khong_doi_lai() -> Result<()> {
    let mb = FakeMailbox::default();
    let (a, b) = (syncer(mb.clone(), "A", "B"), syncer(mb.clone(), "B", "A"));

    let anh = anh_mau(160, 90);
    let mut wa = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    wa.clip_mut().set_image(&anh)?;
    wa.tick()?;
    assert_eq!(a.push_pending(wa.store())?, 1);

    let mut wb = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    let nhan = b.ingest(wb.store())?.expect("B phải nhận được ảnh");
    let NhanVe::Anh(nhan_anh) = nhan else {
        panic!("phải là ảnh, không phải text");
    };
    assert_eq!(nhan_anh, anh, "PNG phải nguyên vẹn từng byte");
    assert_eq!(
        hash_bytes(&nhan_anh.png),
        hash_bytes(&anh.png),
        "hash hai đầu phải trùng"
    );

    // N7 — ghi vào clipboard local rồi poll: không item mới, không PUT ngược.
    wb.apply_remote_image(&nhan_anh)?;
    assert_eq!(wb.tick()?, None, "ảnh nhận về không được thành item mới");
    let truoc = mb.so_lan_put.load(Ordering::SeqCst);
    assert_eq!(b.push_pending(wb.store())?, 0, "B không được PUT ngược lại");
    assert_eq!(mb.so_lan_put.load(Ordering::SeqCst), truoc, "N7 — 0 echo");

    let item = &wb.store().list(10)?[0];
    assert_eq!(item.kind, "image");
    assert_ne!(SYNC_KHONG_GUI, SYNC_CHO_GUI);
    Ok(())
}

// ── 4. N22 ─────────────────────────────────────────────────────────────────

#[test]
fn n22_noi_dung_nhay_cam_khong_vao_lich_su() -> Result<()> {
    let mut w = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    w.clip_mut().set_text("mat-khau-sieu-bi-mat")?;
    w.clip_mut().nhay_cam = true;

    assert_eq!(w.tick()?, None);
    assert_eq!(w.store().count()?, 0, "password không được vào DB");

    // Bỏ đánh dấu thì lại lưu bình thường — chứng minh test không xanh nhờ
    // một lý do khác (clipboard rỗng chẳng hạn).
    w.clip_mut().nhay_cam = false;
    assert!(w.tick()?.is_some());
    assert_eq!(w.store().count()?, 1);
    Ok(())
}
