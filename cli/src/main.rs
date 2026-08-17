//! x2clip CLI — Phase 1. Không có `copy <id>`: trên X11 clipboard là
//! owner-based, tiến trình ghi rồi thoát là mất nội dung (spike 0.2).
//! Dùng lại item là việc của daemon, Phase 4.

use anyhow::{bail, Result};
use x2clip_core::{default_db_path, store::Item, Store, SystemClipboard, Watcher};

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
            eprintln!("x2clip: đang theo dõi clipboard, lịch sử ở {}", db.display());
            Watcher::new(SystemClipboard::new()?, store).run()
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
