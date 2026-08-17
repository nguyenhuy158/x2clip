# ADR-0002 · Tauri v2 làm app shell

**Trạng thái:** Accepted
**Ngày:** 2026-08-17

## Bối cảnh

Cần một app có tray icon, global hotkey và một cửa sổ danh sách lịch sử, chạy trên **cả** macOS và NixOS, từ **một** codebase ([PRD G5](../PRD.md#3-mục-tiêu)).

Logic sync là Rust ([ADR-0003](0003-clipboard-arboard-polling.md), [ADR-0004](0004-storage-sqlite-local-history.md)), nên app shell nên nói được với Rust mà không cần cầu FFI thủ công.

## Quyết định

**Tauri v2** cho phần `app/`. Logic sync nằm trong `core/` — một Rust library thuần, **không** phụ thuộc Tauri.

UI viết bằng **Vite + TypeScript, không framework**: v1 chỉ là một danh sách, một ô tìm kiếm và một nút ghim.

## Phương án đã loại

### SwiftUI (macOS) + GTK/Qt (Linux) — hai UI riêng
**Loại vì:** hai codebase UI cho một người dùng. Mỗi tính năng phải làm hai lần, mỗi bug phải fix hai lần. Đi ngược [PRD G5](../PRD.md#3-mục-tiêu).

### Electron
**Loại vì:** kéo cả Chromium vào cho một cửa sổ danh sách. RAM khó đạt [N10](../NFR.md#2-tài-nguyên), và logic sync là Rust nên vẫn phải cầu qua native module — mất luôn ưu điểm "toàn JS".

### Rust GUI thuần (egui / iced)
Không cần webview, không cần node, `nix build` dễ hơn hẳn — bớt luôn [R3](../RISKS.md#r3--nix-build-cho-app-gui-mất-công-hơn-dự-kiến).

**Loại vì:** tray icon và global hotkey đa nền tảng vẫn phải tự ghép thư viện ngoài, và đó chính là phần khó. Cộng với việc layout danh sách + tìm kiếm bằng CSS nhanh hơn nhiều.

**Kích hoạt xem lại:** nếu [R3](../RISKS.md#r3--nix-build-cho-app-gui-mất-công-hơn-dự-kiến) thành vấn đề thật và không giải được, đây là phương án B. Vì `core/` không phụ thuộc Tauri, đổi shell **không** phải viết lại logic sync.

### Chỉ CLI, không GUI
**Loại vì:** người dùng đã chọn có history UI. CLI-only không đáp ứng [US-B3](../USER-STORIES.md#us-b3--dùng-lại-một-item) và [US-C1](../USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt).

Tuy nhiên CLI **vẫn** được xây ở Phase 1 — nó là cách test logic sync trước khi có UI, và là công cụ xem trạng thái ([N29](../NFR.md#6-khả-năng-vận-hành)).

### Có framework frontend (React / Svelte / Vue)
**Loại vì:** một danh sách và một ô tìm kiếm. Framework thêm build step và dependency mà không giải quyết gì ở quy mô này.

**Kích hoạt xem lại:** khi UI có nhiều state lồng nhau tới mức DOM tay bắt đầu rối — không phải trước đó.

## Hệ quả

### Được
- Một codebase, một lần build config, ra `.app` cho macOS và binary cho Linux
- Rust core dùng trực tiếp qua IPC, không cầu FFI thủ công
- Tray + global hotkey có plugin sẵn cho cả hai OS
- Bundle nhỏ hơn Electron rõ rệt (dùng webview của hệ điều hành)

### Mất
- **Webview khác nhau giữa hai OS** (WebKit trên macOS, WebKitGTK trên Linux) → CSS/JS có thể lệch. Bắt buộc kiểm UI trên **cả hai** OS, đã ghi ở [ROADMAP Phase 4](../ROADMAP.md#phase-4--ui--⬜)
- **`nix build` phức tạp**: Rust + node deps + webkitgtk trong một derivation ([R3](../RISKS.md#r3--nix-build-cho-app-gui-mất-công-hơn-dự-kiến))
- Cần Node trong môi trường dev, dù logic là Rust

### Ràng buộc bắt buộc giữ
**`core/` không được phụ thuộc Tauri.** Đây là điều làm cho quyết định này đảo được, và là điều cho phép test logic sync mà không dựng GUI ([TEST-PLAN § Nguyên tắc](../TEST-PLAN.md#1-nguyên-tắc)). Nếu `core` bắt đầu `use tauri::` thì cả hai lợi ích mất cùng lúc.
