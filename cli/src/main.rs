//! x2clip CLI — Phase 1. Không có `copy <id>`: trên X11 clipboard là
//! owner-based, tiến trình ghi rồi thoát là mất nội dung (spike 0.2).
//! Dùng lại item là việc của daemon, Phase 4.

use anyhow::{bail, Result};
use x2clip_core::config::default_config_path;
use x2clip_core::crypto::derive_key;
use x2clip_core::mailbox::R2Mailbox;
use x2clip_core::sync::NhanVe;
use x2clip_core::{
    default_db_path, store::Item, Config, Store, Syncer, SystemClipboard, Watcher, POLL_INTERVAL,
};

const USAGE: &str = "\
x2clip — lịch sử clipboard

    x2clip watch          theo dõi clipboard và ghi lịch sử (chạy nền)
    x2clip list [n]       liệt kê n item mới nhất (mặc định 20)
    x2clip search <từ>    tìm trong lịch sử, không phân biệt hoa thường
";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let db = default_db_path()?;

    match args.first().map(String::as_str) {
        Some("watch") => {
            let store = Store::open(&db)?;
            eprintln!(
                "x2clip: đang theo dõi clipboard, lịch sử ở {}",
                db.display()
            );
            watch(store)
        }
        Some("list") => {
            let n = args.get(1).map(|s| s.parse()).transpose()?.unwrap_or(20);
            let items = Store::open(&db)?.list(n)?;
            if items.is_empty() {
                println!("Lịch sử trống — chưa copy gì.");
            } else {
                print(&items);
            }
            Ok(())
        }
        Some("search") => {
            let Some(q) = args.get(1) else {
                bail!("thiếu từ khoá\n\n{USAGE}");
            };
            let items = Store::open(&db)?.search(q)?;
            if items.is_empty() {
                println!("Không tìm thấy item nào chứa \"{q}\".");
            } else {
                print(&items);
            }
            Ok(())
        }
        _ => {
            print!("{USAGE}");
            Ok(())
        }
    }
}

/// Một vòng lặp, hai nhịp: clipboard mỗi `POLL_INTERVAL` (N13), hộp thư mỗi
/// `poll_secs` (N13b). Không thread riêng — cả hai đều rẻ và cùng đụng một DB.
fn watch(store: Store) -> Result<()> {
    let cfg = Config::load_or_create(&default_config_path()?)?;
    let mut w = Watcher::new(SystemClipboard::new()?, store);

    let syncer = match &cfg.mailbox {
        None => {
            eprintln!("x2clip: chưa cấu hình [mailbox] — chạy local-only, không đồng bộ");
            None
        }
        Some(mb) => {
            let key = derive_key(&cfg.passphrase()?, &cfg.salt()?)?;
            Some(Syncer::new(
                R2Mailbox::new(mb)?,
                key,
                cfg.inbox_cua_minh(),
                cfg.inbox_cua_peer(),
            ))
        }
    };

    let nhip_hop_thu = (cfg.poll_secs * 1000 / POLL_INTERVAL.as_millis() as u64).max(1);
    let mut n: u64 = 0;
    loop {
        if let Err(e) = w.tick() {
            eprintln!("x2clip: lỗi khi poll clipboard: {e}");
        }
        if let Some(s) = &syncer {
            // Nhịp đầu tiên chạy ngay để N1c (< 10s kể từ lúc khởi động) đạt.
            if n.is_multiple_of(nhip_hop_thu) {
                if let Err(e) = s.push_pending(w.store()) {
                    eprintln!("x2clip: chưa gửi được, giữ hàng chờ: {e}");
                }
                match s.ingest(w.store()) {
                    // Set cờ echo trước khi ghi clipboard — `apply_remote*` lo.
                    Ok(Some(nhan)) => {
                        let r = match &nhan {
                            NhanVe::Text(t) => w.apply_remote(t),
                            NhanVe::Anh(a) => w.apply_remote_image(a),
                        };
                        if let Err(e) = r {
                            eprintln!("x2clip: không ghi được clipboard: {e}");
                        }
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("x2clip: không đọc được hộp thư: {e}"),
                }
            }
        }
        n += 1;
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn print(items: &[Item]) {
    for i in items {
        let mut preview: String = i.body.chars().take(70).collect();
        if i.body.chars().count() > 70 {
            preview.push('…');
        }
        let preview = preview.replace('\n', "⏎");
        let pin = if i.pinned { "📌" } else { "  " };
        println!("{:>5} {pin} {preview}", i.id);
    }
}
