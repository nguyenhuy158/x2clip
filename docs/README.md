# x2clip — Tài liệu

Clipboard sync + history manager cho macOS và NixOS (Linux). Một codebase.

## Bảng phân quyền nội dung

Mỗi thông tin chỉ có **một** file là nguồn chính. File khác muốn nhắc thì link, không copy — copy là nguyên nhân số một khiến doc lệch nhau sau vài tuần.

| File | Nguồn chính cho | Đọc khi |
|---|---|---|
| [PRD.md](PRD.md) | Vấn đề, người dùng, mục tiêu, scope in/out | Muốn biết app này để làm gì và *không* làm gì |
| [USER-STORIES.md](USER-STORIES.md) | Story + acceptance criteria | Sắp code một tính năng, cần biết thế nào là xong |
| [NFR.md](NFR.md) | Ngưỡng hiệu năng, giới hạn, hành vi khi lỗi, bảo mật | Cần con số cụ thể |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Kiến trúc, data flow, protocol, cây file | Cần biết code nằm ở đâu, chạy thế nào |
| [ADR/](ADR/) | Quyết định kỹ thuật + lý do | Muốn biết *vì sao* chọn thứ này thay vì thứ kia |
| [ROADMAP.md](ROADMAP.md) | Thứ tự phase, deliverable, exit criteria | Hỏi "giờ làm gì tiếp" |
| [TEST-PLAN.md](TEST-PLAN.md) | Chiến lược test, quality gate mỗi phase | Sắp đóng một phase |
| [RISKS.md](RISKS.md) | Risk register | Có gì bất ngờ xảy ra, hoặc review trước mỗi phase |
| [UI/WIREFRAMES.md](UI/WIREFRAMES.md) | Bố cục màn hình + ma trận trạng thái UI | Sắp dựng giao diện, cần biết màn hình gồm gì |
| [UI/MOCKUPS.md](UI/MOCKUPS.md) | Màu, chữ, khoảng cách, hình dạng trạng thái | Cần token cụ thể để code UI |
| [UI/PROTOTYPE.md](UI/PROTOTYPE.md) | Hợp đồng tương tác + bản bấm thử [prototype.html](UI/prototype.html) | Muốn *dùng thử* trước khi code |

## Trạng thái

**[Phase 0](ROADMAP.md#phase-0--spike--✅-xong) xong, chưa bắt đầu code.** Còn [Q3](PRD.md#9-câu-hỏi-mở) chưa chốt — mở [prototype](UI/prototype.html) xem rồi trả lời.

## Quy tắc cập nhật

- Đổi quyết định kỹ thuật → thêm ADR mới, đánh ADR cũ là `Superseded`. Không sửa lịch sử ADR.
- Đổi scope → sửa [PRD.md](PRD.md) trước, rồi mới lan sang story/roadmap.
- Đóng một phase → tick exit criteria trong [ROADMAP.md](ROADMAP.md).
