# x2clip

Clipboard sync + history manager cho macOS và NixOS (Linux). Một codebase.

> Trạng thái: **planning** — chưa có code, chỉ có tài liệu thiết kế.

## Ý tưởng

Copy ở máy A, paste được ở máy B. Kèm lịch sử clipboard local có search và pin.

| Thành phần | Chọn |
|---|---|
| Transport | Tailscale (không backend, không key management) |
| App shell | Tauri v2 → `.app` cho macOS, binary/flake cho NixOS |
| Clipboard | `arboard` (macOS + X11 + Wayland, text + ảnh) |
| Lưu trữ | SQLite (`rusqlite`) |
| UI | Vite + TypeScript, không framework |

v1 sync *clipboard hiện tại*; history là local mỗi máy.

## Tài liệu

Mục lục đầy đủ + quy tắc cập nhật: [docs/README.md](docs/README.md).

| File | Nội dung |
|---|---|
| [docs/PRD.md](docs/PRD.md) | Vấn đề, mục tiêu, scope in/out |
| [docs/USER-STORIES.md](docs/USER-STORIES.md) | User stories + acceptance criteria |
| [docs/NFR.md](docs/NFR.md) | Ngưỡng số, giới hạn, hành vi khi lỗi, bảo mật |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Thành phần, data flow, protocol, cây file |
| [docs/ADR/](docs/ADR/) | 5 quyết định kỹ thuật + phương án đã loại |
| [docs/ROADMAP.md](docs/ROADMAP.md) | 6 phase, deliverable, exit criteria |
| [docs/TEST-PLAN.md](docs/TEST-PLAN.md) | Chiến lược test, quality gate |
| [docs/RISKS.md](docs/RISKS.md) | Risk register |
| [PLAN.md](PLAN.md) | Index (nội dung đã tách sang `docs/`) |

## Bước tiếp theo

Phase 0 spike: 0.1 (NixOS = **X11**, không DE) và 0.3 (Tailscale `nixos` ↔ `macbook`, direct 6ms) đã pass. Còn 0.2 — `arboard` đọc/ghi text + ảnh trên cả hai máy. Chi tiết: [ROADMAP](docs/ROADMAP.md).
