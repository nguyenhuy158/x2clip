//! Quality gate Phase 2 — T2, T3, T6, T8, T9, T11–T16 của docs/TEST-PLAN.md.
//! Tất cả chạy trên hộp thư giả; phần R2 thật để kiểm thủ công hai máy.

use anyhow::Result;
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use x2clip_core::clip::{hash_text, Clipboard};
use x2clip_core::config::Config;
use x2clip_core::crypto::{derive_key, SecretKey};
use x2clip_core::mailbox::{Mailbox, MailboxError, MailboxResult};
use x2clip_core::store::{SYNC_CHO_GUI, SYNC_DA_GUI, SYNC_KHONG_GUI};
use x2clip_core::sync::Payload;
use x2clip_core::{Store, Syncer, Watcher};

// ── hộp thư giả ────────────────────────────────────────────────────────────

/// Một bucket chung cho cả hai node trong cùng một process. `Clone` chia sẻ
/// cùng bộ nhớ, đúng kiểu hai máy nhìn vào một bucket.
#[derive(Clone, Default)]
struct FakeMailbox {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    so_lan_put: Arc<AtomicUsize>,
    delete_luon_hong: bool,
    offline: Arc<Mutex<bool>>,
}

impl FakeMailbox {
    fn keys_voi_prefix(&self, prefix: &str) -> Vec<String> {
        let o = self.objects.lock().unwrap();
        let mut k: Vec<String> = o
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        k.sort();
        k
    }
    fn dat_offline(&self, v: bool) {
        *self.offline.lock().unwrap() = v;
    }
    fn kiem_tra_online(&self) -> MailboxResult<()> {
        if *self.offline.lock().unwrap() {
            Err(MailboxError::Network("giả lập mất mạng".into()))
        } else {
            Ok(())
        }
    }
}

impl Mailbox for FakeMailbox {
    fn put(&self, key: &str, body: &[u8]) -> MailboxResult<()> {
        self.kiem_tra_online()?;
        self.so_lan_put.fetch_add(1, Ordering::SeqCst);
        self.objects
            .lock()
            .unwrap()
            .insert(key.to_string(), body.to_vec());
        Ok(())
    }
    fn list(&self, prefix: &str) -> MailboxResult<Vec<String>> {
        self.kiem_tra_online()?;
        Ok(self.keys_voi_prefix(prefix))
    }
    fn get(&self, key: &str) -> MailboxResult<Vec<u8>> {
        self.kiem_tra_online()?;
        self.objects
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| MailboxError::Other(format!("không có {key}")))
    }
    fn delete(&self, key: &str) -> MailboxResult<()> {
        self.kiem_tra_online()?;
        if self.delete_luon_hong {
            return Err(MailboxError::Other("giả lập DELETE hỏng".into()));
        }
        self.objects.lock().unwrap().remove(key);
        Ok(())
    }
}

const PASSPHRASE: &str = "cai-nay-la-passphrase-that-cua-toi";

fn khoa() -> SecretKey {
    // Salt cố định để hai node trong cùng test ra cùng khoá.
    derive_key(PASSPHRASE, &[7u8; 16]).unwrap()
}

/// Node A gửi cho B.
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
}
impl Clipboard for FakeClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.text.clone()
    }
    fn set_text(&mut self, text: &str) -> Result<()> {
        self.text = Some(text.to_string());
        Ok(())
    }
}

fn thu_muc_tam(ten: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("x2clip-test-{ten}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

// ── T2: một chiều, không vọng lại ──────────────────────────────────────────

/// A copy một lần → đúng 1 object trong `inbox/B/`, 0 trong `inbox/A/`,
/// B nhận xong **không PUT lại gì**, tổng số PUT có trần.
#[test]
fn t2_mot_chieu_khong_bong_ban() -> Result<()> {
    let mb = FakeMailbox::default();
    let (a, b) = (syncer(mb.clone(), "A", "B"), syncer(mb.clone(), "B", "A"));
    let (store_a, store_b) = (Store::open_memory()?, Store::open_memory()?);

    store_a.upsert_text(&hash_text("gửi đi"), "gửi đi")?;
    assert_eq!(a.push_pending(&store_a)?, 1);

    assert_eq!(mb.keys_voi_prefix("inbox/B/").len(), 1);
    assert_eq!(mb.keys_voi_prefix("inbox/A/").len(), 0);

    assert_eq!(b.ingest(&store_b)?.as_deref(), Some("gửi đi"));
    // Item nhận về không bao giờ được xếp vào hàng chờ gửi.
    assert_eq!(b.push_pending(&store_b)?, 0, "B không được PUT ngược lại");
    assert_eq!(mb.keys_voi_prefix("inbox/A/").len(), 0);

    // Vài vòng poll nữa cũng phải im lặng.
    for _ in 0..3 {
        b.ingest(&store_b)?;
        b.push_pending(&store_b)?;
        a.ingest(&store_a)?;
        a.push_pending(&store_a)?;
    }
    assert!(
        mb.so_lan_put.load(Ordering::SeqCst) <= 5,
        "số PUT phải có trần, đang là {}",
        mb.so_lan_put.load(Ordering::SeqCst)
    );
    Ok(())
}

// ── T3: echo guard qua đúng đường thật ─────────────────────────────────────

/// Nội dung nhận từ hộp thư, ghi vào clipboard qua `apply_remote`, rồi
/// watcher poll — **không** được sinh item mới và **không** được PUT lại.
#[test]
fn t3_echo_guard_tren_duong_that() -> Result<()> {
    let mb = FakeMailbox::default();
    let (a, b) = (syncer(mb.clone(), "A", "B"), syncer(mb.clone(), "B", "A"));
    let store_a = Store::open_memory()?;
    store_a.upsert_text(&hash_text("từ máy A"), "từ máy A")?;
    a.push_pending(&store_a)?;

    let mut w = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    let nhan = b.ingest(w.store())?.expect("phải có item");
    w.apply_remote(&nhan)?;

    assert_eq!(w.tick()?, None, "echo không được thành item mới");
    let truoc = mb.so_lan_put.load(Ordering::SeqCst);
    assert_eq!(b.push_pending(w.store())?, 0);
    assert_eq!(mb.so_lan_put.load(Ordering::SeqCst), truoc, "N7 — 0 echo");
    Ok(())
}

// ── T6: round-trip byte-for-byte ───────────────────────────────────────────

#[test]
fn t6_round_trip_giu_nguyen_tung_byte() -> Result<()> {
    let mb = FakeMailbox::default();
    let (a, b) = (syncer(mb.clone(), "A", "B"), syncer(mb.clone(), "B", "A"));
    let (store_a, store_b) = (Store::open_memory()?, Store::open_memory()?);

    let dai = "x".repeat(100_000);
    let mau = [
        "tiếng Việt có dấu: Ăn Quả Nhớ Kẻ Trồng Cây",
        "emoji 🇻🇳🍜 và tab\tnewline\ndòng hai",
        "  khoảng trắng đầu cuối  ",
        &dai,
    ];
    for s in mau {
        store_a.upsert_text(&hash_text(s), s)?;
    }
    a.push_pending(&store_a)?;
    b.ingest(&store_b)?;

    let lich_su = store_b.list(100)?;
    for s in mau {
        assert!(
            lich_su.iter().any(|i| i.body == s),
            "mất nguyên vẹn nội dung: {:?}",
            &s[..s.len().min(30)]
        );
    }

    // Chuỗi rỗng không bao giờ được gửi.
    let mut w = Watcher::new(FakeClipboard::default(), Store::open_memory()?);
    w.clip_mut().set_text("")?;
    assert_eq!(w.tick()?, None);
    assert_eq!(w.store().count()?, 0);
    Ok(())
}

// ── T8: payload rác ────────────────────────────────────────────────────────

#[test]
fn t8_payload_hong_thi_bo_qua_va_giu_lai() -> Result<()> {
    let mb = FakeMailbox::default();
    let b = syncer(mb.clone(), "B", "A");
    let key = khoa();
    let store = Store::open_memory()?;

    let rac: Vec<(&str, Vec<u8>)> = vec![
        ("khong-phai-json", key.encrypt(b"day khong phai json")?),
        (
            "sai-phien-ban",
            key.encrypt(br#"{"v":99,"kind":"text","hash":"x","body":"a","ts":1}"#)?,
        ),
        (
            "kind-la",
            key.encrypt(br#"{"v":1,"kind":"video/mp4","hash":"x","body":"a","ts":1}"#)?,
        ),
        (
            "thieu-truong",
            key.encrypt(br#"{"v":1,"kind":"text","ts":1}"#)?,
        ),
        (
            "hash-khong-khop",
            key.encrypt(br#"{"v":1,"kind":"text","hash":"deadbeef","body":"a","ts":1}"#)?,
        ),
        ("object-rong", Vec::new()),
        ("header-cut", vec![1u8; 10]),
    ];
    for (ten, blob) in &rac {
        mb.put(&format!("inbox/B/{ten}"), blob).unwrap();
    }

    assert_eq!(b.ingest(&store)?, None, "không có gì được lên clipboard");
    assert_eq!(store.count()?, 0, "không có gì lọt vào lịch sử");
    assert_eq!(
        mb.keys_voi_prefix("inbox/B/").len(),
        rac.len(),
        "object hỏng phải được giữ nguyên, không xoá"
    );
    Ok(())
}

// ── T9: config ─────────────────────────────────────────────────────────────

#[test]
fn t9_config() -> Result<()> {
    let dir = thu_muc_tam("config");

    // Thiếu file → tạo mặc định, chạy được ở chế độ local-only.
    let p = dir.join("config.toml");
    let cfg = Config::load_or_create(&p)?;
    assert!(p.exists());
    assert!(cfg.mailbox.is_none(), "chưa cấu hình hộp thư = local-only");
    assert_eq!(cfg.salt()?.len(), 16);

    // Sai cú pháp → lỗi có số dòng, và **không ghi đè** file người dùng.
    let hong = dir.join("hong.toml");
    let noi_dung = "machine = \"a\"\npeer = \"b\"\nday la rac\n";
    std::fs::write(&hong, noi_dung)?;

    // Quyền rộng hơn 0600 thì từ chối chạy hẳn, và nói rõ cách sửa.
    let quyen_rong = Config::load_or_create(&hong).unwrap_err().to_string();
    assert!(quyen_rong.contains("chmod 600"), "{quyen_rong}");
    std::fs::set_permissions(&hong, std::fs::Permissions::from_mode(0o600))?;

    let err = Config::load_or_create(&hong).unwrap_err().to_string();
    assert!(
        err.contains("line 3") || err.contains("3:"),
        "lỗi phải chỉ ra dòng, đang là: {err}"
    );
    assert_eq!(
        std::fs::read_to_string(&hong)?,
        noi_dung,
        "không được ghi đè"
    );

    // machine == peer + đã bật [mailbox] → tự gửi cho chính mình, vòng lặp
    // vô hạn chỉ hiện ra dưới dạng hoá đơn R2. Từ chối load.
    let vong = dir.join("vong.toml");
    let co_mailbox = "\n[mailbox]\nendpoint = \"https://x.r2.cloudflarestorage.com\"\nbucket = \"b\"\naccess_key_id = \"k\"\nsecret_access_key = \"s\"\n";
    std::fs::write(
        &vong,
        format!("machine = \"a\"\npeer = \"a\"\n{co_mailbox}"),
    )?;
    std::fs::set_permissions(&vong, std::fs::Permissions::from_mode(0o600))?;
    let err = Config::load_or_create(&vong).unwrap_err().to_string();
    assert!(err.contains("vòng lặp"), "{err}");

    // Cùng cấu hình đó nhưng chưa bật hộp thư thì local-only, không sao cả.
    let local = dir.join("local.toml");
    std::fs::write(&local, "machine = \"a\"\npeer = \"a\"\n")?;
    std::fs::set_permissions(&local, std::fs::Permissions::from_mode(0o600))?;
    Config::load_or_create(&local).expect("local-only không cần peer hợp lệ");

    // Tên còn là giá trị mẫu cũng không được bật sync.
    let mau = dir.join("mau.toml");
    std::fs::write(
        &mau,
        format!("machine = \"may-nay\"\npeer = \"peer\"\n{co_mailbox}"),
    )?;
    std::fs::set_permissions(&mau, std::fs::Permissions::from_mode(0o600))?;
    let err = Config::load_or_create(&mau).unwrap_err().to_string();
    assert!(err.contains("giá trị mẫu"), "{err}");

    // Sai access key phải phân biệt được với mất mạng.
    let auth = MailboxError::Auth("HTTP 403".into()).to_string();
    let net = MailboxError::Network("timeout".into()).to_string();
    assert!(auth.contains("access key") && !net.contains("access key"));

    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

// ── T11: máy kia tắt rồi bật lại ───────────────────────────────────────────

/// A copy ba lần khi B đang tắt → cả ba vào lịch sử B, **chỉ cái mới nhất**
/// lên clipboard B, và B không PUT gì.
#[test]
fn t11_ba_copy_khi_b_dang_tat() -> Result<()> {
    let mb = FakeMailbox::default();
    let (a, b) = (syncer(mb.clone(), "A", "B"), syncer(mb.clone(), "B", "A"));
    let (store_a, store_b) = (Store::open_memory()?, Store::open_memory()?);

    for s in ["một", "hai", "ba"] {
        store_a.upsert_text(&hash_text(s), s)?;
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    a.push_pending(&store_a)?;

    // B bật lên, poll một nhịp.
    assert_eq!(b.ingest(&store_b)?.as_deref(), Some("ba"), "chỉ mới nhất");
    assert_eq!(store_b.count()?, 3, "cả ba vào lịch sử — N8, 0 mất mát");
    assert_eq!(b.push_pending(&store_b)?, 0);
    assert_eq!(mb.keys_voi_prefix("inbox/A/").len(), 0);
    Ok(())
}

// ── T13: DELETE hỏng ───────────────────────────────────────────────────────

#[test]
fn t13_delete_hong_khong_xu_ly_lai() -> Result<()> {
    let mut mb = FakeMailbox::default();
    let a = syncer(mb.clone(), "A", "B");
    let store_a = Store::open_memory()?;
    store_a.upsert_text(&hash_text("chỉ một lần"), "chỉ một lần")?;
    a.push_pending(&store_a)?;

    mb.delete_luon_hong = true;
    let b = syncer(mb.clone(), "B", "A");
    let store_b = Store::open_memory()?;

    assert_eq!(b.ingest(&store_b)?.as_deref(), Some("chỉ một lần"));
    assert_eq!(store_b.count()?, 1);

    // B copy cái mới hơn ở local, rồi poll lại: object cũ vẫn nằm đó.
    store_b.upsert_text(&hash_text("mới hơn"), "mới hơn")?;
    assert_eq!(
        b.ingest(&store_b)?,
        None,
        "object đã xử lý không được ghi đè clipboard mới hơn của B"
    );
    assert_eq!(store_b.count()?, 2, "không sinh row trùng");
    Ok(())
}

// ── T14: giả mạo / sai khoá ────────────────────────────────────────────────

#[test]
fn t14_lat_byte_va_sai_khoa() -> Result<()> {
    let mb = FakeMailbox::default();
    let key = khoa();

    let mut blob = key.encrypt(br#"{"v":1,"kind":"text","hash":"x","body":"a","ts":1}"#)?;
    let cuoi = blob.len() - 1;
    blob[cuoi] ^= 0x01;
    mb.put("inbox/B/lat-byte", &blob).unwrap();

    let khoa_khac = derive_key("passphrase hoàn toàn khác", &[9u8; 16])?;
    let p = Payload {
        v: 1,
        kind: "text".into(),
        hash: hash_text("bí mật"),
        body: "bí mật".into(),
        ts: 1,
    };
    let blob2 = khoa_khac.encrypt(serde_json::to_string(&p)?.as_bytes())?;
    mb.put("inbox/B/sai-khoa", &blob2).unwrap();

    let b = syncer(mb.clone(), "B", "A");
    let store = Store::open_memory()?;
    assert_eq!(b.ingest(&store)?, None);
    assert_eq!(store.count()?, 0);
    assert_eq!(
        mb.keys_voi_prefix("inbox/B/").len(),
        2,
        "hai object đều phải được giữ lại"
    );
    Ok(())
}

// ── T15: object key không lộ gì ────────────────────────────────────────────

#[test]
fn t15_key_khong_lo_noi_dung() -> Result<()> {
    let mb = FakeMailbox::default();
    let a = syncer(mb.clone(), "A", "B");
    let store = Store::open_memory()?;

    let noi_dung = "nội dung bí mật";
    let h = hash_text(noi_dung);
    store.upsert_text(&h, noi_dung)?;
    a.push_pending(&store)?;

    // Lần hai: cùng nội dung, hash trùng nên phải tự tay xếp lại hàng chờ.
    let (id, _) = store.upsert_text(&h, noi_dung)?;
    store.dat_synced(id, SYNC_CHO_GUI)?;
    a.push_pending(&store)?;

    let keys = mb.keys_voi_prefix("inbox/B/");
    assert_eq!(keys.len(), 2, "cùng nội dung vẫn phải ra hai key khác nhau");
    for k in &keys {
        let ulid = k.strip_prefix("inbox/B/").expect("đúng prefix máy nhận");
        assert_eq!(ulid.len(), 26, "ULID 26 ký tự, key là {k}");
        assert!(ulid.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(!k.contains(&h) && !k.contains(noi_dung) && !k.contains("text"));
    }
    Ok(())
}

// ── T12: bí mật không rò ra ngoài ──────────────────────────────────────────

#[test]
fn t12_khong_ro_bi_mat() -> Result<()> {
    let mb = FakeMailbox::default();
    let a = syncer(mb.clone(), "A", "B");
    let store = Store::open_memory()?;
    store.upsert_text(&hash_text("dữ liệu"), "dữ liệu")?;
    a.push_pending(&store)?;

    for (k, v) in mb.objects.lock().unwrap().iter() {
        let raw = String::from_utf8_lossy(v);
        for bi_mat in [PASSPHRASE, "dữ liệu"] {
            assert!(!raw.contains(bi_mat), "{bi_mat} lộ trong object {k}");
            assert!(!k.contains(bi_mat), "{bi_mat} lộ trong key {k}");
        }
    }

    // Thông báo lỗi giải mã cũng không được nhắc tới passphrase.
    let err = khoa().decrypt(&[0u8; 50]).unwrap_err().to_string();
    assert!(!err.contains(PASSPHRASE));

    // `{cfg:?}` ở một nhánh lỗi nào đó là đủ để đưa access key vào log. Impl
    // Debug viết tay che nó — không có assert này thì nó lặng lẽ mất tác dụng
    // ngay lúc ai đó thêm lại `derive(Debug)`.
    let mb_cfg = x2clip_core::config::MailboxConfig {
        endpoint: "https://x.r2.cloudflarestorage.com".to_string(),
        bucket: "b".to_string(),
        region: "auto".to_string(),
        access_key_id: "AKIA-LO-RA-DAY".to_string(),
        secret_access_key: "SECRET-LO-RA-DAY".to_string(),
    };
    let d = format!("{mb_cfg:?}");
    assert!(!d.contains("LO-RA-DAY"), "Debug làm lộ secret: {d}");
    Ok(())
}

// ── T16: hàng chờ sống qua restart ─────────────────────────────────────────

#[test]
fn t16_hang_cho_song_qua_restart() -> Result<()> {
    let dir = thu_muc_tam("queue");
    let db = dir.join("x2clip.db");
    let mb = FakeMailbox::default();
    mb.dat_offline(true);

    {
        let store = Store::open(&db)?;
        store.upsert_text(&hash_text("chờ gửi"), "chờ gửi")?;
        let (id, _) = store.upsert_text(&hash_text("không gửi"), "không gửi")?;
        store.dat_synced(id, SYNC_KHONG_GUI)?;

        let a = syncer(mb.clone(), "A", "B");
        assert!(a.push_pending(&store).is_err(), "offline thì PUT phải lỗi");
        assert_eq!(store.cho_gui()?.len(), 1, "item lỗi vẫn nằm trong hàng chờ");
    }

    // "Khởi động lại": mở lại DB từ đầu.
    let store = Store::open(&db)?;
    let cho = store.cho_gui()?;
    assert_eq!(cho.len(), 1, "hàng chờ phải sống qua restart");
    assert_eq!(cho[0].body, "chờ gửi");
    assert!(
        !cho.iter().any(|i| i.body == "không gửi"),
        "item 'không gửi' phải phân biệt được với hàng chờ, không dùng chung synced = 0"
    );

    mb.dat_offline(false);
    let a = syncer(mb.clone(), "A", "B");
    assert_eq!(a.push_pending(&store)?, 1);
    assert_eq!(store.cho_gui()?.len(), 0);
    assert_eq!(mb.keys_voi_prefix("inbox/B/").len(), 1);

    // Đã gửi rồi thì không gửi lại.
    assert_eq!(a.push_pending(&store)?, 0);
    let _ = SYNC_DA_GUI;

    std::fs::remove_dir_all(&dir)?;
    Ok(())
}
