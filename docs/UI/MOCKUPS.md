# Mockups — x2clip

> Nguồn chính cho: **ngôn ngữ hình ảnh** — màu, chữ, khoảng cách, hình dạng trạng thái. Bố cục ở [WIREFRAMES.md](WIREFRAMES.md). Bản chạy được ở [PROTOTYPE.md](PROTOTYPE.md).

Mockup trả lời câu "trông nó ra sao", sau khi wireframe đã chốt "nó gồm những gì". Ở đây chỉ có **một** giao diện — không có hệ thống theme. [USER-STORIES](../USER-STORIES.md) đã xếp tuỳ biến giao diện ra ngoài scope.

## 1. Token

Đặt ở `:root`, dùng lại nguyên vẹn trong [prototype](prototype.html) — hai file lệch nhau là lỗi.

| Token | Sáng | Tối | Dùng cho |
|---|---|---|---|
| `--bg` | `#f7f7f8` | `#1c1c1e` | nền cửa sổ |
| `--bg-row-sel` | `#e4e7ec` | `#2f3033` | hàng đang chọn |
| `--fg` | `#111114` | `#f2f2f4` | text chính |
| `--fg-dim` | `#77777e` | `#8b8b93` | thời gian, metadata, gợi ý phím |
| `--line` | `#e2e2e6` | `#333337` | đường kẻ |
| `--accent` | `#2f6feb` | `#4c8bf5` | viền ô tìm khi focus, ghim |
| `--warn` | `#b25e09` | `#e0913a` | vượt giới hạn, không sync |
| `--error` | `#c0392b` | `#e5645a` | banner lỗi |

Theo `prefers-color-scheme` của hệ điều hành. **Không có nút đổi theme** — chưa ai xin.

## 2. Chữ

Font hệ thống, không nhúng font ngoài: `-apple-system, "Segoe UI", Inter, system-ui, sans-serif`.

Nội dung item dùng font đơn cách: `ui-monospace, "SF Mono", "JetBrains Mono", monospace` — lịch sử clipboard của người này chủ yếu là lệnh, path, key. Đơn cách khiến nhận diện nhanh hơn hẳn.

| Chỗ | Cỡ | Đậm |
|---|---|---|
| Ô tìm kiếm | 15px | 400 |
| Nội dung item | 13px | 400 |
| Thời gian / metadata | 11px | 400 |
| Thanh phím ở đáy | 11px | 500 |

## 3. Mật độ

| | Giá trị |
|---|---|
| Chiều cao hàng text | 34px |
| Chiều cao hàng ảnh | 64px |
| Thumbnail | 48×48, `object-fit: cover`, bo 4px |
| Padding ngang | 12px |
| Bo góc cửa sổ | 10px |
| Bo góc hàng | 6px |

34px cho khoảng 9 hàng lọt vào chiều cao 420px. Đủ để thấy hết những gì vừa copy mà không phải cuộn — chín item gần nhất là gần như toàn bộ nhu cầu thật.

## 4. Trạng thái từng phần tử

**Hàng**

| Trạng thái | Hình ảnh |
|---|---|
| thường | nền trong suốt |
| hover | nền `--bg-row-sel` ở 50% |
| đang chọn | nền `--bg-row-sel` đặc + vạch `--accent` 2px bên trái |
| ghim | icon 📌 `--accent` ở đầu hàng |
| vượt giới hạn | icon ⚠ + nội dung màu `--warn` |

Chỉ đúng **một** hàng có trạng thái "đang chọn", kể cả khi chuột đang hover ở hàng khác — bàn phím là nguồn chân lý, chuột không cướp ô chọn.

**Ô tìm kiếm.** Không viền lúc thường; focus thì viền 2px `--accent`. Vì ô này *luôn* có focus lúc mở cửa sổ, viền focus mặc định là hình ảnh bình thường của app.

**Chấm trạng thái ở góc phải thanh tìm.** 8px. ● đặc = kết nối, ○ rỗng = mất kết nối, ⏸ = tạm dừng. Cùng bộ hình dạng với tray icon, cùng nghĩa ở hai chỗ.

**Banner.** Một dòng 28px, dưới thanh tìm. Nền = màu trạng thái ở 12% opacity, chữ = màu trạng thái đặc. Không có nút X đóng — banner biến mất khi vấn đề hết, không phải khi người dùng gạt đi.

## 5. Tray icon

Vẽ bằng SVG đơn sắc 16×16 (`template image` trên macOS, để hệ thống tự đảo màu theo menu bar sáng/tối). Đó là lý do ba trạng thái phải khác nhau về **hình dạng**:

```
 ●  đã kết nối   — vòng tròn tô đặc
 ○  mất kết nối  — vòng tròn viền 1.5px, rỗng ruột
 ⏸  tạm dừng     — hai thanh dọc
 ⚠  lỗi          — tam giác
```

## 6. Chuyển động

Gần như không có. Cửa sổ hiện: fade 80ms. Không có slide, không có scale, không có animation trên hàng.

[N6](../NFR.md#1-ngưỡng-chấp-nhận) cho **200ms** từ lúc bấm phím tắt tới lúc gõ được. Một animation 150ms "cho mượt" ăn hết 3/4 ngân sách đó. Với công cụ mở hàng chục lần mỗi ngày, tức thì *chính là* cảm giác cao cấp.

## 7. Chưa làm

Theme tuỳ chỉnh, đổi màu nhấn, chế độ compact/comfortable, icon riêng cho từng loại nội dung, avatar theo máy nguồn. Thêm khi dùng thật rồi thấy thiếu — không phải trước đó.
