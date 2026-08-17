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

| File | Nội dung |
|---|---|
| [PLAN.md](PLAN.md) | Quyết định kỹ thuật, kiến trúc, phase |
| [docs/PRD.md](docs/PRD.md) | Product requirements |
| [docs/USER-STORIES.md](docs/USER-STORIES.md) | User stories + acceptance criteria |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Kiến trúc chi tiết |
| [docs/NFR.md](docs/NFR.md) | Non-functional requirements |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Roadmap theo milestone |
| [docs/TEST-PLAN.md](docs/TEST-PLAN.md) | Test plan |

## License

[MIT](LICENSE)
