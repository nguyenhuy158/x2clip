# Prototype — x2clip

> Nguồn chính cho: **hợp đồng tương tác** kiểm chứng được bằng tay. Cấu trúc ở [WIREFRAMES.md](WIREFRAMES.md), hình ảnh ở [MOCKUPS.md](MOCKUPS.md).

## Chạy thế nào

```bash
open docs/UI/prototype.html
```

Một file HTML, không dependency, không build, không server. Trên NixOS: `xdg-open docs/UI/prototype.html`.

## Nó là gì, không là gì

**Là:** bản giả để *dùng thử bằng bàn phím* trước khi viết Rust. Dữ liệu cứng trong file, gồm một hàng ảnh, một item ghim, một item vượt giới hạn, và một chuỗi unicode + emoji.

**Không là:** code sẽ dùng lại. Phase 4 viết trong Tauri, không copy file này vào `app/`. Nó **không** đụng clipboard thật, không có SQLite, không có mạng.

## Nó kiểm cái gì

Prototype tồn tại để trả lời những câu mà đọc doc không trả lời được. Bấm thử và tự chấm:

| Kiểm | Cách thử | Đạt là |
|---|---|---|
| [US-C1](../USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt) — gõ được ngay | Mở file, gõ luôn `doc` | Danh sách lọc mà không phải click chỗ nào |
| Chọn bằng bàn phím | `↑` `↓` | Đúng một hàng sáng, cuộn theo |
| Chép rồi đóng | `⏎` | Cửa sổ biến mất, hiện "Đã chép …" |
| **`esc` không đụng clipboard** | `esc` | Đóng, hiện "clipboard không đổi" |
| Ghim không đóng cửa sổ | `⌘P` | Item nhảy lên đầu, cửa sổ còn nguyên |
| Xoá một item không hỏi | `⌘⌫` | Mất một hàng, không có dialog |
| Mở lại bằng phím tắt | `⌘⇧V` (hoặc `Ctrl⇧V`) | Cửa sổ hiện, ô tìm **rỗng** và có con trỏ |
| Ma trận trạng thái | Bấm 6 nút ở bảng "Thử trạng thái" | Banner + chấm + chữ trong tray đổi cùng lúc, và nói đúng nguyên nhân |
| Rỗng vs. không khớp | Nút "lịch sử rỗng"; gõ `zzz` | Hai câu khác nhau, không phải một khung trắng |

Ô đáng chú ý nhất là **`esc` không đụng clipboard** và **`⌘P` không đóng cửa sổ**. Hai cái này dễ code sai vì "đóng sau mỗi hành động" nghe hợp lý — dùng thử ba mươi giây là thấy sai.

## Lệch với wireframe (2026-08-17)

Prototype dựng theo ma trận trạng thái **trước** [ADR-0007](../ADR/0007-dang-nhap-va-khoa-tu-passphrase.md). Sáu nút trạng thái trong file vẫn đúng với những gì chúng mô tả, nhưng **thiếu** bốn trạng thái mới: chưa đăng nhập, token hết hạn, passphrase sai, access key hết hạn. Nút "Tailscale" trong prototype cũng còn nói *mất kết nối* — [WIREFRAMES § 3](WIREFRAMES.md#3-ma-trận-trạng-thái) đã sửa thành *chỉ chậm hơn, sync vẫn chạy*.

Chưa cập nhật vì Phase 6 nằm sau v1 và prototype là đồ bỏ. **Nguồn chân lý là WIREFRAMES.md**, không phải file HTML này.

## Chưa dựng

Màn hình cấu hình, hộp xác nhận xoá toàn bộ, lazy-load khi cuộn quá 50 item. Đã có wireframe ở [WIREFRAMES.md § 4–5](WIREFRAMES.md#4-cấu-hình) — dựng khi thật sự cần bấm thử, không phải để cho đủ bộ.

## Sau khi xem xong

Prototype này chính là câu trả lời cho [PRD Q3](../PRD.md#9-câu-hỏi-mở) ("muốn thấy hình trước không?"). Xem xong thì chốt Q3 trong PRD, rồi [Phase 1](../ROADMAP.md#phase-1--core-local-một-máy--) hết bị chặn — thứ tự phase không đổi: UI vẫn ở Phase 4, chỉ là đã biết trước sẽ dựng cái gì.
