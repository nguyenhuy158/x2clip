# x2clip

Clipboard sync + history manager cho macOS và NixOS (Linux). Một codebase.

> Trạng thái: **planning** — chưa có code, chỉ có tài liệu thiết kế.

## Ý tưởng

Copy ở máy A, paste được ở máy B. Kèm lịch sử clipboard local có search và pin.

| Thành phần | Chọn |
|---|---|
| Transport | Hộp thư Cloudflare R2 qua S3 API — hai máy **không** cần cùng online. Tailscale chỉ là chuông tuỳ chọn ([ADR-0006](docs/ADR/0006-r2-mailbox-store-and-forward.md)) |
| Mã hoá | AEAD tầng app (`age`/libsodium) **trước** khi lên R2. Khoá giữ local |
| App shell | Tauri v2 → `.app` cho macOS, binary/flake cho NixOS |
| Clipboard | `arboard` (macOS + X11 + Wayland, text + ảnh) |
| Lưu trữ | SQLite (`rusqlite`) |
| UI | Vite + TypeScript, không framework |

Mọi item đi qua hộp thư, nên history **tự hội tụ** giữa các máy. Đồng bộ ghim/xoá thì để sau v1.

## Tài liệu

Mục lục đầy đủ + quy tắc cập nhật: [docs/README.md](docs/README.md).

| File | Nội dung |
|---|---|
| [docs/PRD.md](docs/PRD.md) | Vấn đề, mục tiêu, scope in/out |
| [docs/USER-STORIES.md](docs/USER-STORIES.md) | User stories + acceptance criteria |
| [docs/NFR.md](docs/NFR.md) | Ngưỡng số, giới hạn, hành vi khi lỗi, bảo mật |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Thành phần, data flow, protocol, cây file |
| [docs/ADR/](docs/ADR/) | 7 quyết định kỹ thuật + phương án đã loại |
| [docs/ROADMAP.md](docs/ROADMAP.md) | 6 phase, deliverable, exit criteria |
| [docs/UI/](docs/UI/) | Wireframe, mockup, prototype bấm thử |
| [docs/TEST-PLAN.md](docs/TEST-PLAN.md) | Chiến lược test, quality gate |
| [docs/RISKS.md](docs/RISKS.md) | Risk register |
| [PLAN.md](PLAN.md) | Index (nội dung đã tách sang `docs/`) |

## Bước tiếp theo

Phase 0 ✅ xong (X11 trên NixOS, Tailscale direct 6ms, `arboard` đọc/ghi text + ảnh cả hai máy). Thiết kế UI đã có bản bấm thử. Tiếp theo: **Phase 1** — CLI local, chưa có mạng. Chi tiết: [ROADMAP](docs/ROADMAP.md).
