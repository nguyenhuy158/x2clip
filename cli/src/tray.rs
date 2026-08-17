//! Tray + phím tắt toàn cục (US-C1, US-C2). **macOS-only**, cố ý.
//!
//! Trên X11 clipboard là owner-based nên `copy` cần tiến trình sống — tray sẽ
//! giải quyết được chuyện đó, nhưng chưa test trên NixOS nên chưa mở cfg.
//!
//! Vì sao là cửa sổ eframe chứ không phải mở thẳng menu tray khi bấm phím tắt:
//! `tray-icon` 0.24 / `muda` 0.19 chỉ cho mở context menu qua `hwnd`,
//! `gtk_window` hoặc `nsview` — **không** có đường mở bằng code từ NSApp. Muốn
//! có ô tìm kiếm (US-B3) thì đằng nào cũng phải có cửa sổ, nên dùng luôn.
//!
//! Cửa sổ **không bao giờ đóng**, chỉ ẩn: đóng là event loop chết, phím tắt
//! chỉ chạy được đúng một lần. Và vì ẩn thì egui không vẽ nữa, mỗi frame phải
//! `request_repaint_after` để còn kịp đọc kênh phím tắt — đây là chỗ hỏng im
//! lặng duy nhất của file này.

use anyhow::{Context, Result};
use eframe::egui;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::path::Path;
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};
use x2clip_core::clip::{giai_ma_png, Anh, Clipboard};
use x2clip_core::config::default_config_path;
use x2clip_core::store::Item;
use x2clip_core::{Config, Store, SystemClipboard};

/// Nhịp quét kênh phím tắt/menu khi cửa sổ đang ẩn.
const NHIP: std::time::Duration = std::time::Duration::from_millis(200);
const SO_ITEM: usize = 200;

/// US-C2 — ba trạng thái phải nhìn là biết, không cần mở menu.
#[derive(PartialEq, Clone, Copy)]
enum TrangThai {
    DangChay,
    TamDung,
    ChuaCauHinh,
}

impl TrangThai {
    fn doc(co_mailbox: bool, co: &Path) -> Self {
        if !co_mailbox {
            Self::ChuaCauHinh
        } else if co.exists() {
            Self::TamDung
        } else {
            Self::DangChay
        }
    }

    /// Chấm tròn 16x16 vẽ bằng tay. Không dùng file PNG: thêm một file ảnh là
    /// thêm một thứ phải nhớ copy khi đóng gói `.app`.
    fn icon(self) -> Result<Icon> {
        let mau: [u8; 3] = match self {
            Self::DangChay => [46, 160, 67],      // xanh
            Self::TamDung => [219, 171, 10],      // vàng
            Self::ChuaCauHinh => [128, 128, 128], // xám
        };
        let (n, r) = (16i32, 7i32);
        let mut rgba = Vec::with_capacity((n * n * 4) as usize);
        for y in 0..n {
            for x in 0..n {
                let trong = (x - 8) * (x - 8) + (y - 8) * (y - 8) <= r * r;
                rgba.extend_from_slice(&[mau[0], mau[1], mau[2], if trong { 255 } else { 0 }]);
            }
        }
        Icon::from_rgba(rgba, n as u32, n as u32).context("dựng icon tray")
    }

    fn nhan(self) -> &'static str {
        match self {
            Self::DangChay => "x2clip — đang đồng bộ",
            Self::TamDung => "x2clip — tạm dừng đồng bộ",
            Self::ChuaCauHinh => "x2clip — chưa cấu hình hộp thư (local-only)",
        }
    }
}

pub fn chay() -> Result<()> {
    let db = x2clip_core::default_db_path()?;
    let co = db.with_file_name("paused");
    let co_mailbox = Config::load_or_create(&default_config_path()?)?
        .mailbox
        .is_some();

    // Đăng ký phím tắt **trước** event loop: hỏng thì báo ngay rồi thoát, chứ
    // không để người dùng bấm ⌥⌘V mãi mà không hiểu vì sao im.
    let hotkey_mgr = GlobalHotKeyManager::new().context(
        "không tạo được bộ quản lý phím tắt — kiểm tra quyền Accessibility trong System Settings",
    )?;
    hotkey_mgr
        .register(HotKey::new(
            Some(Modifiers::ALT | Modifiers::SUPER),
            Code::KeyV,
        ))
        .context("không đăng ký được ⌥⌘V — có app khác đang giữ phím tắt này")?;

    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 420.0])
            .with_visible(false)
            .with_title("x2clip"),
        ..Default::default()
    };

    eframe::run_native(
        "x2clip",
        opts,
        Box::new(move |_cc| {
            // Tray phải dựng **trong** đây: NSApplication chỉ tồn tại sau khi
            // eframe khởi tạo xong, dựng sớm hơn là icon không hiện.
            Ok(Box::new(Ung::moi(co, co_mailbox, hotkey_mgr)?) as Box<dyn eframe::App>)
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe: {e}"))
}

struct Ung {
    store: Store,
    clip: SystemClipboard,
    co: std::path::PathBuf,
    co_mailbox: bool,
    trang_thai: TrangThai,
    tray: TrayIcon,
    mi_mo: MenuItem,
    mi_dung: CheckMenuItem,
    mi_thoat: MenuItem,
    _hotkey_mgr: GlobalHotKeyManager,
    tim: String,
    items: Vec<Item>,
    hien: bool,
    vua_mo: bool,
    loi: Option<String>,
}

impl Ung {
    fn moi(
        co: std::path::PathBuf,
        co_mailbox: bool,
        hotkey_mgr: GlobalHotKeyManager,
    ) -> Result<Self> {
        let trang_thai = TrangThai::doc(co_mailbox, &co);
        let menu = Menu::new();
        let mi_mo = MenuItem::new("Mở x2clip  (⌥⌘V)", true, None);
        // Chưa cấu hình hộp thư thì không có gì để dừng — disable, đừng để
        // người dùng bấm rồi tự hỏi sao không đổi gì.
        let mi_dung = CheckMenuItem::new(
            "Tạm dừng đồng bộ",
            co_mailbox,
            trang_thai == TrangThai::TamDung,
            None,
        );
        let mi_thoat = MenuItem::new("Thoát", true, None);
        menu.append(&mi_mo)?;
        menu.append(&mi_dung)?;
        menu.append(&mi_thoat)?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(trang_thai.icon()?)
            .with_tooltip(trang_thai.nhan())
            .build()
            .context("không dựng được tray icon")?;

        let store = Store::open(&x2clip_core::default_db_path()?)?;
        let items = store.list(SO_ITEM)?;
        Ok(Self {
            store,
            clip: SystemClipboard::new()?,
            co,
            co_mailbox,
            trang_thai,
            tray,
            mi_mo,
            mi_dung,
            mi_thoat,
            _hotkey_mgr: hotkey_mgr,
            tim: String::new(),
            items,
            hien: false,
            vua_mo: false,
            loi: None,
        })
    }

    fn bao(&mut self, msg: String) {
        eprintln!("x2clip: {msg}");
        self.loi = Some(msg);
    }

    fn nap_lai(&mut self) {
        let r = if self.tim.trim().is_empty() {
            self.store.list(SO_ITEM)
        } else {
            self.store.search(&self.tim)
        };
        match r {
            Ok(v) => self.items = v,
            Err(e) => self.bao(format!("không đọc được lịch sử: {e}")),
        }
    }

    fn mo(&mut self, ctx: &egui::Context) {
        self.tim.clear();
        self.nap_lai();
        self.hien = true;
        self.vua_mo = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    fn an(&mut self, ctx: &egui::Context) {
        self.hien = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    /// Đúng đường mà `x2clip copy <id>` đi — sửa một chỗ thì nhớ sửa cả hai.
    fn dan(&mut self, id: i64) -> Result<()> {
        let item = self
            .store
            .lay(id)?
            .ok_or_else(|| anyhow::anyhow!("item {id} biến mất khỏi lịch sử"))?;
        if item.kind == "image" {
            let png = self
                .store
                .lay_blob(id)?
                .ok_or_else(|| anyhow::anyhow!("item {id} là ảnh nhưng mất blob — DB hỏng"))?;
            let (rong, cao, _) = giai_ma_png(&png)?;
            self.clip.set_image(&Anh { rong, cao, png })?;
        } else {
            self.clip.set_text(&item.body)?;
        }
        Ok(())
    }

    fn dat_tam_dung(&mut self, dung: bool) {
        let r = if dung {
            std::fs::write(&self.co, b"")
        } else {
            match std::fs::remove_file(&self.co) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                r => r,
            }
        };
        if let Err(e) = r {
            self.bao(format!("không đổi được trạng thái đồng bộ: {e}"));
        }
    }

    /// `x2clip pause` ở terminal đổi được trạng thái sau lưng tray, nên đọc lại
    /// từ file mỗi frame chứ không giữ trong đầu.
    fn dong_bo_icon(&mut self) {
        let moi = TrangThai::doc(self.co_mailbox, &self.co);
        if moi == self.trang_thai {
            return;
        }
        self.trang_thai = moi;
        self.mi_dung.set_checked(moi == TrangThai::TamDung);
        match moi.icon() {
            Ok(i) => {
                if let Err(e) = self.tray.set_icon(Some(i)) {
                    self.bao(format!("không đổi được icon tray: {e}"));
                }
                if let Err(e) = self.tray.set_tooltip(Some(moi.nhan())) {
                    self.bao(format!("không đổi được tooltip tray: {e}"));
                }
            }
            Err(e) => self.bao(format!("không dựng được icon tray: {e}")),
        }
    }
}

impl eframe::App for Ung {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        while let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv() {
            // Không lọc Pressed là mỗi lần bấm chạy hai lần: nhấn rồi nhả.
            if ev.state == HotKeyState::Pressed {
                if self.hien {
                    self.an(ctx);
                } else {
                    self.mo(ctx);
                }
            }
        }
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if ev.id == *self.mi_mo.id() {
                self.mo(ctx);
            } else if ev.id == *self.mi_dung.id() {
                let dung = self.trang_thai != TrangThai::TamDung;
                self.dat_tam_dung(dung);
            } else if ev.id == *self.mi_thoat.id() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.dong_bo_icon();

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.an(ctx);
        }

        let mut chon: Option<i64> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let o = ui.add(
                egui::TextEdit::singleline(&mut self.tim)
                    .hint_text("Tìm trong lịch sử… (Esc để đóng)")
                    .desired_width(f32::INFINITY),
            );
            // N6 — con trỏ phải sẵn trong ô tìm kiếm. Chỉ xin focus một lần mỗi
            // lần mở; xin mọi frame là người dùng không click đi đâu được.
            if self.vua_mo {
                o.request_focus();
                self.vua_mo = false;
            }
            if o.changed() {
                self.nap_lai();
            }

            if let Some(e) = self.loi.clone() {
                ui.colored_label(egui::Color32::from_rgb(200, 60, 60), e);
            }
            ui.separator();

            if self.items.is_empty() {
                ui.weak("Không có item nào.");
                return;
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for it in &self.items {
                    let mut d: String = it.body.chars().take(90).collect();
                    if it.body.chars().count() > 90 {
                        d.push('…');
                    }
                    let d = d.replace('\n', "⏎");
                    let nhan = if it.pinned {
                        format!("📌 {d}")
                    } else {
                        format!("　 {d}")
                    };
                    if ui.selectable_label(false, nhan).clicked() {
                        chon = Some(it.id);
                    }
                }
            });
        });

        if let Some(id) = chon {
            match self.dan(id) {
                Ok(()) => {
                    self.loi = None;
                    self.an(ctx);
                }
                Err(e) => self.bao(format!("không copy được item {id}: {e}")),
            }
        }

        // Ẩn thì egui ngủ, không ai đọc kênh phím tắt nữa → bấm ⌥⌘V không lên
        // gì và chẳng có log nào báo. Phải tự đánh thức.
        ctx.request_repaint_after(NHIP);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Ba trạng thái phải ra ba nhãn khác nhau và đọc đúng từ file cờ — trùng
    /// nhau là US-C2 hỏng mà nhìn màn hình không thấy.
    #[test]
    fn ba_trang_thai_phan_biet_duoc() {
        let d = std::env::temp_dir().join("x2clip-test-paused");
        let _ = std::fs::remove_file(&d);
        assert!(matches!(TrangThai::doc(false, &d), TrangThai::ChuaCauHinh));
        assert!(matches!(TrangThai::doc(true, &d), TrangThai::DangChay));
        std::fs::write(&d, b"").unwrap();
        assert!(matches!(TrangThai::doc(true, &d), TrangThai::TamDung));
        std::fs::remove_file(&d).unwrap();

        let n = [
            TrangThai::DangChay,
            TrangThai::TamDung,
            TrangThai::ChuaCauHinh,
        ]
        .map(TrangThai::nhan);
        assert_eq!(n.iter().collect::<std::collections::HashSet<_>>().len(), 3);
    }
}
