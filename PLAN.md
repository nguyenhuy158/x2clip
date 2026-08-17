# x2clip

Clipboard sync + history manager cho macOS và NixOS (Linux). Một codebase.

> Nội dung kế hoạch đã tách sang [`docs/`](docs/). File này chỉ còn là điểm vào — để không có thông tin nào tồn tại ở hai chỗ rồi lệch nhau.

## Bắt đầu từ đâu

| Muốn biết | Đọc |
|---|---|
| App này làm gì, không làm gì | [docs/PRD.md](docs/PRD.md) |
| Thế nào là xong một tính năng | [docs/USER-STORIES.md](docs/USER-STORIES.md) |
| Con số ngưỡng, giới hạn, hành vi khi lỗi | [docs/NFR.md](docs/NFR.md) |
| Code nằm ở đâu, chạy thế nào | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |
| Vì sao chọn Tailscale / Tauri / SQLite | [docs/ADR/](docs/ADR/) |
| Giờ làm gì tiếp | [docs/ROADMAP.md](docs/ROADMAP.md) |
| Kiểm chứng thế nào | [docs/TEST-PLAN.md](docs/TEST-PLAN.md) |
| Chỗ nào dễ vỡ | [docs/RISKS.md](docs/RISKS.md) |

Mục lục đầy đủ kèm quy tắc cập nhật: [docs/README.md](docs/README.md).

## Trạng thái

**[Phase 0](docs/ROADMAP.md#phase-0--spike--✅-xong) ✅ xong.** 0.1 (NixOS = X11, không DE), 0.3 (Tailscale `nixos` ↔ `macbook`, direct 6ms), 0.2 (`arboard` đọc/ghi text + ảnh, cả hai máy) đều pass. Không ADR nào phải đổi.

Còn [Q3](docs/PRD.md#9-câu-hỏi-mở) chưa trả lời — duyệt thứ tự phase hay muốn UI sớm hơn — chặn Phase 1.
