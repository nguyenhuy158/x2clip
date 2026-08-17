//! x2clip CLI.
//!
//! `copy <id>` **chỉ chạy đúng trên macOS**. Trên X11 clipboard là owner-based:
//! tiến trình ghi rồi thoát là nội dung bay theo (spike 0.2). NSPasteboard thì
//! giữ hộ dữ liệu, nên một lệnh chạy-rồi-thoát là đủ. Trên Linux muốn dùng lại
//! item thì phải qua daemon — chưa làm.
//!
//! `copy` không đụng được cờ chống dội (cờ đó nằm trong `Watcher` của tiến
//! trình `watch` khác). Nên item vừa copy lại sẽ được `watch` thấy như một lần
//! copy mới: **không đẻ row mới** (`hash` UNIQUE, chỉ bump `updated_at`) nhưng
//! **có gửi lại cho máy kia**. Đó cũng đúng ý người dùng: copy lại là muốn nó
//! sang máy bên kia.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use x2clip_core::clip::{giai_ma_png, Anh, Clipboard};
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
    x2clip copy <id>      đưa item lên clipboard (hiện chỉ đúng trên macOS)
    x2clip pin <id>       ghim — không bị dọn tự động
    x2clip unpin <id>     bỏ ghim
    x2clip rm <id>        xoá hẳn, kể cả item đã ghim
    x2clip pause          tạm dừng đồng bộ (lịch sử local vẫn chạy)
    x2clip resume         chạy đồng bộ lại
    x2clip status         xem đang đồng bộ hay tạm dừng
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
            let co = co_tam_dung(&db);
            watch(store, co)
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
        Some("copy") => {
            let id = doc_id(&args)?;
            let store = Store::open(&db)?;
            let Some(item) = store.lay(id)? else {
                bail!("không có item {id}");
            };
            let mut clip = SystemClipboard::new()?;
            if item.kind == "image" {
                let Some(png) = store.lay_blob(id)? else {
                    bail!("item {id} là ảnh nhưng mất blob — DB hỏng");
                };
                let (rong, cao, _) = giai_ma_png(&png)?;
                clip.set_image(&Anh { rong, cao, png })?;
                println!("đã copy ảnh {rong}x{cao} (item {id})");
            } else {
                clip.set_text(&item.body)?;
                println!("đã copy item {id}");
            }
            Ok(())
        }
        Some(v @ ("pin" | "unpin")) => {
            let id = doc_id(&args)?;
            if !Store::open(&db)?.set_pinned(id, v == "pin")? {
                bail!("không có item {id}");
            }
            println!(
                "{} item {id}",
                if v == "pin" {
                    "đã ghim"
                } else {
                    "đã bỏ ghim"
                }
            );
            Ok(())
        }
        Some("rm") => {
            let id = doc_id(&args)?;
            if !Store::open(&db)?.xoa(id)? {
                bail!("không có item {id}");
            }
            println!("đã xoá item {id}");
            Ok(())
        }
        Some(v @ ("pause" | "resume")) => {
            let co = co_tam_dung(&db);
            if v == "pause" {
                std::fs::write(&co, b"")?;
                println!("đã tạm dừng đồng bộ — `x2clip resume` để chạy lại");
            } else {
                // Chưa dừng thì `remove_file` báo NotFound; không phải lỗi.
                match std::fs::remove_file(&co) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
                println!("đã bật lại đồng bộ");
            }
            Ok(())
        }
        Some("status") => {
            let cfg = Config::load_or_create(&default_config_path()?)?;
            println!(
                "hộp thư: {}\nđồng bộ: {}\nlịch sử: {} item",
                if cfg.mailbox.is_some() {
                    "đã cấu hình"
                } else {
                    "chưa cấu hình (local-only)"
                },
                if co_tam_dung(&db).exists() {
                    "TẠM DỪNG"
                } else {
                    "đang chạy"
                },
                Store::open(&db)?.count()?,
            );
            Ok(())
        }
        _ => {
            print!("{USAGE}");
            Ok(())
        }
    }
}

fn doc_id(args: &[String]) -> Result<i64> {
    let Some(s) = args.get(1) else {
        bail!("thiếu <id>\n\n{USAGE}");
    };
    s.parse().map_err(|_| anyhow::anyhow!("id phải là số: {s}"))
}

/// Cờ tạm dừng là một file rỗng cạnh DB, không phải một dòng trong config:
/// `watch` đọc lại nó mỗi vòng nên `pause` có tác dụng ngay, không cần restart,
/// và không có nguy cơ hai tiến trình cùng ghi đè file config.
fn co_tam_dung(db: &Path) -> PathBuf {
    db.with_file_name("paused")
}

/// Một vòng lặp, hai nhịp: clipboard mỗi `POLL_INTERVAL` (N13), hộp thư mỗi
/// `poll_secs` (N13b). Không thread riêng — cả hai đều rẻ và cùng đụng một DB.
fn watch(store: Store, co: PathBuf) -> Result<()> {
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
    let mut da_bao_dung = false;
    loop {
        if let Err(e) = w.tick() {
            eprintln!("x2clip: lỗi khi poll clipboard: {e}");
        }
        // US-A4 — tạm dừng chỉ chặn hộp thư, lịch sử local vẫn ghi. Hàng chờ
        // giữ nguyên, `resume` là gửi tiếp, không mất item (N8).
        let dung = co.exists();
        if dung != da_bao_dung {
            eprintln!(
                "x2clip: {}",
                if dung {
                    "tạm dừng đồng bộ"
                } else {
                    "đồng bộ chạy lại"
                }
            );
            da_bao_dung = dung;
        }
        if let Some(s) = syncer.as_ref().filter(|_| !dung) {
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
