# PRD — x2clip

> Nguồn chính cho: vấn đề, người dùng, mục tiêu, scope. Con số hiệu năng ở [NFR.md](NFR.md), kiến trúc ở [ARCHITECTURE.md](ARCHITECTURE.md).

**Trạng thái:** Draft, chờ duyệt
**Cập nhật:** 2026-08-17

---

## 1. Vấn đề

Người dùng làm việc song song trên một máy macOS và một máy NixOS (Linux), hai máy **không cùng mạng**. Copy nội dung ở máy này, muốn paste ở máy kia thì hiện tại phải đi đường vòng: gửi qua chat, qua file, hoặc đánh lại tay.

Đồng thời clipboard của cả hai OS chỉ giữ **một** giá trị — copy cái mới là mất cái cũ, không tìm lại được.

## 2. Người dùng

Một người: chính chủ project. Dev, dùng terminal nhiều, tự quản trị cả hai máy, cài được software tuỳ ý.

Không có người dùng thứ hai trong scope này. Điều đó cho phép bỏ nhiều thứ (onboarding, multi-tenant, phân quyền, chia sẻ) — xem [§4](#4-ngoài-scope).

## 3. Mục tiêu

| # | Mục tiêu | Vì sao |
|---|---|---|
| G1 | Copy ở máy nào thì paste được ở máy kia, không thao tác thêm | Đây là lý do tồn tại của app |
| G2 | Giữ lịch sử clipboard, tìm lại và dùng lại được | Clipboard chỉ giữ 1 giá trị là mất mát thật |
| G3 | Chạy được cả text và ảnh | Ảnh (screenshot) là phần lớn nhu cầu copy chéo máy |
| G4 | Không tự vận hành backend nào | Một người dùng, không đáng để bảo trì service |
| G5 | Một codebase cho cả hai OS | Fix một lần chạy cả hai máy |

### Sản phẩm tham chiếu
[CleanClip](https://cleanclip.cc/), [Paste](https://pasteapp.io/) — cùng ý tưởng (history + sync), nhưng cả hai **chỉ chạy trên hệ Apple** và sync bằng iCloud/CloudKit. Cách đó không dùng được vì NixOS không có CloudKit. x2clip lấy *ý tưởng*, không lấy feature list.

## 4. Ngoài scope

| Không làm | Vì sao | Khi nào tính lại |
|---|---|---|
| Mobile (iOS/Android) | Không phải nhu cầu hiện tại | Có nhu cầu thật |
| Nhiều người dùng, chia sẻ clipboard | Một người dùng | Không dự kiến |
| Máy thứ 3+ | Chỉ có 2 máy. Thiết kế peer list không chặn việc thêm | Có máy thứ 3 |
| Sync **cả lịch sử** giữa 2 máy | Cần conflict resolution — là product thứ hai. v1 chỉ sync clipboard hiện tại, history local mỗi máy | Dùng thật rồi thấy thiếu |
| Sync khi hai máy không cùng online | Tailscale là mạng, không phải kho lưu trữ. Xem [ADR-0001](ADR/0001-transport-tailscale.md) | Thấy mất item thường xuyên |
| Relay server riêng | Không cần khi có Tailscale | Cần chạy trên máy không cài được Tailscale |
| Sync file / thư mục | Đó là Syncthing, không phải clipboard | Không dự kiến |
| Đồng bộ định dạng rich text (RTF/HTML) | Plain text + ảnh phủ gần hết. Rich text mỗi OS một kiểu | Paste mất format gây khó chịu thật |

## 5. Ràng buộc

- **Hai OS bất đối xứng**: macOS không có event clipboard, Linux thì có. Chi tiết ở [ARCHITECTURE.md](ARCHITECTURE.md).
- **Wayland đọc clipboard phụ thuộc compositor**, không phụ thuộc distro. Đây là rủi ro cao nhất — xem [RISKS.md](RISKS.md) R1.
- **Clipboard chứa dữ liệu nhạy cảm** (password, token). Ràng buộc bảo mật ở [NFR.md § Bảo mật](NFR.md#4-bảo-mật).
- **Phải cài Tailscale trên cả hai máy.** Đánh đổi có ý thức: bớt hẳn một backend, đổi lấy một dependency ngoài.

## 6. Thước đo thành công

Đây là app cá nhân, không có DAU/retention. Thành công = ba điều kiện:

1. Người dùng **ngừng** gửi text cho chính mình qua chat để chuyển máy.
2. Đạt hết ngưỡng ở [NFR.md § Ngưỡng chấp nhận](NFR.md#1-ngưỡng-chấp-nhận).
3. Chạy nền một tuần liền không cần restart tay.

## 7. Yêu cầu chức năng

Đây là danh sách rút gọn để tra cứu. Acceptance criteria đầy đủ ở [USER-STORIES.md](USER-STORIES.md).

| ID | Yêu cầu | Ưu tiên |
|---|---|---|
| FR1 | Theo dõi clipboard và phát hiện nội dung mới | Must |
| FR2 | Đẩy nội dung mới sang máy còn lại | Must |
| FR3 | Nhận từ máy kia và ghi vào clipboard local | Must |
| FR4 | Không tạo vòng lặp echo | Must |
| FR5 | Lưu lịch sử, tìm kiếm được | Must |
| FR6 | Ghim (pin) item để không bị xoá tự động | Must |
| FR7 | Click item trong lịch sử → copy lại | Must |
| FR8 | Hỗ trợ ảnh | Must |
| FR9 | Global hotkey mở cửa sổ lịch sử | Must |
| FR10 | Tray icon hiện trạng thái kết nối | Must |
| FR11 | Xoá item khỏi lịch sử | Must |
| FR12 | Tự chạy khi đăng nhập | Should |
| FR13 | Bỏ qua item được đánh dấu nhạy cảm (password manager) | Should |
| FR14 | Cấu hình được: peer, poll interval, giới hạn dung lượng | Should |
| FR15 | Tạm dừng sync | Could |

## 8. Phát hành

Không có release công khai, không có store. "Ship" = chạy được trên hai máy của chính chủ. Đóng gói chi tiết ở [ROADMAP.md](ROADMAP.md) Phase 5.

## 9. Câu hỏi mở

| # | Câu hỏi | Chặn gì |
|---|---|---|
| Q1 | **Compositor / desktop trên NixOS là gì?** (Hyprland / Sway / GNOME / KDE / X11) | Chặn Phase 0 — quyết định clipboard adapter có chạy được không |
| Q2 | Tailscale đã cài trên cả hai máy chưa? | Chặn Phase 2 |
| Q3 | Duyệt thứ tự phase, hay muốn UI sớm hơn để thấy hình trước? | Chặn Phase 1 |
