# Mockups — x2clip

> Nguồn chính cho: **visual** — token màu, chữ, khoảng cách, icon tray. Layout ở [WIREFRAMES.md](WIREFRAMES.md), bản bấm được ở [prototype/index.html](prototype/index.html).

Không có Figma. Bản mockup thật là [prototype/index.html](prototype/index.html) — nó vừa là mockup vừa là prototype, cùng CSS sẽ chuyển thẳng vào `app/src/style.css`. File này là phần **quyết định**, không phải phần render.

Style: spotlight-like — cửa sổ nổi, bo góc, nền mờ. Giống cách macOS mở Spotlight và cách CleanClip mở panel. Lý do: app này mở ra 2 giây rồi đóng, không phải app để ngồi trong đó.

---

## Token

Đặt bằng CSS custom property, một chỗ duy nhất, để đổi theme không phải sửa rải rác.

```css
:root {
  --bg:        #1c1c1e;   /* nền cửa sổ */
  --bg-row:    #2c2c2e;   /* hàng đang chọn */
  --fg:        #f2f2f7;   /* chữ chính */
  --fg-dim:    #8e8e93;   /* thời gian, chú thích, chân cửa sổ */
  --accent:    #0a84ff;   /* khớp tìm kiếm, focus ring */
  --pin:       #ffd60a;   /* ghim */
  --warn:      #ff9f0a;   /* dải cảnh báo, item không sync */
  --ok:        #30d158;   /* đã kết nối */
  --radius:    10px;
  --gap:       8px;
}
```

**Dark là mặc định**, không phải tuỳ chọn — cả hai máy đang dùng dark. Light theme: chưa làm, thêm khi thấy cần (`prefers-color-scheme` đổi token là đủ, không sửa markup).

## Chữ

| Chỗ | Kiểu |
|---|---|
| Ô tìm | system UI, 15px |
| Nội dung item | system UI, 13px, một dòng, `text-overflow: ellipsis` |
| Thời gian | system UI, 12px, `--fg-dim` |
| Nội dung là code/đường dẫn | monospace 13px — nhận ra ngay đây là thứ paste vào terminal |
| Chân cửa sổ | 11px, `--fg-dim` |

Font: `-apple-system, system-ui, sans-serif` và `ui-monospace, monospace`. **Không nhúng font** — mỗi OS tự dùng font của nó, và bundle nhỏ hơn.

## Khoảng cách

- Cửa sổ: rộng 560px, cao tối đa 480px, bo `--radius`, nền `--bg` + backdrop blur.
- Hàng: cao 32px, padding ngang 12px. Đủ để bấm chuột, đủ để thấy ~12 item một lần.
- Danh sách cuộn, ô tìm và chân **cố định** — cuộn không được làm mất ô tìm.

## Màu không phải kênh duy nhất

Ba trạng thái kết nối ([W6](WIREFRAMES.md#w6--trạng-thái-kết-nối)) khác nhau **cả hình dạng, cả chữ**, màu chỉ là lớp thứ ba:

| Trạng thái | Ký hiệu | Chữ | Màu |
|---|---|---|---|
| Đã kết nối | `●` đầy | "sync" | `--ok` |
| Mất kết nối | `○` rỗng | "mất kết nối" | `--fg-dim` |
| Tạm dừng | `⏸` | "tạm dừng" | `--warn` |

Cùng lý do đó: item không sync được có cả icon `⚠` và chữ "quá lớn, không sync", không chỉ đổi màu chữ.

Đây không phải chỗ để tiết kiệm — người dùng nhìn cái này bằng khoé mắt trong nửa giây.

## Icon tray

Ba biến thể, một hình gốc (hai hình chữ nhật lệch nhau — hình clipboard tối giản):

| Trạng thái | Icon |
|---|---|
| Đã kết nối | nét đầy, opacity 1 |
| Mất kết nối | nét đầy, opacity 0.4 |
| Tạm dừng | có gạch chéo |

- **macOS:** template image đơn sắc, để OS tự đảo màu theo menubar sáng/tối. Icon màu ở menubar macOS trông như app lạc chỗ.
- **Linux (X11, không DE):** tray phụ thuộc panel đang chạy. Nếu panel không có system tray thì **hotkey vẫn phải mở được cửa sổ** — tray là tiện nghi, không phải đường vào duy nhất ([US-C1](USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt) không phụ thuộc tray).

Định dạng: SVG một file, export PNG @1x/@2x lúc build.

## Animation

Mở cửa sổ: fade + scale từ 0.98, ~120ms. Đóng: fade 80ms. Hết.

Không animate hàng, không stagger, không spring. Cửa sổ phải hiện trong [N6](NFR.md#1-ngưỡng-chấp-nhận) — animation dài hơn budget đó thì chính nó là cái làm app cảm giác chậm.

## Ảnh trong danh sách

Thumbnail 40×24, `object-fit: cover`, bo 4px, kèm chữ kích thước thật (`320×200`). Thumb lấy từ cột `thumb` ([ARCHITECTURE §7](ARCHITECTURE.md#7-lưu-trữ)) — **không** decode blob gốc để render list, đó là đường ngắn nhất tới việc phá [N5](NFR.md#1-ngưỡng-chấp-nhận) và [N10](NFR.md#2-tài-nguyên).

## Chỗ cố tình không có

| Không có | Vì sao |
|---|---|
| Light theme | Hai máy đang dark. Đổi token là xong khi cần |
| Font nhúng | System font đủ, bundle nhỏ hơn |
| Icon set ngoài (Lucide, SF Symbols…) | Cần ~6 glyph. Emoji + SVG tay là đủ, không thêm dependency |
| Logo / branding | Không phát hành công khai ([PRD §8](PRD.md#8-phát-hành)) |
| Skeleton loading | Đọc SQLite local, không có trạng thái "đang tải" đủ lâu để thấy |
