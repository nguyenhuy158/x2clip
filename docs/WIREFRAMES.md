# Wireframes — x2clip

> Nguồn chính cho: **layout và trạng thái** của từng màn hình. Visual (màu, font, icon) ở [MOCKUPS.md](MOCKUPS.md). Bản bấm được ở [prototype/index.html](prototype/index.html). Story ở [USER-STORIES.md](USER-STORIES.md), con số ở [NFR.md](NFR.md).

Low-fidelity, cố ý. Mục đích: chốt *có những gì trên màn hình* và *khi nào thấy gì*, trước khi viết code UI ([ROADMAP Phase 4](ROADMAP.md#phase-4--ui--)).

**Quy tắc:** mỗi màn hình / mỗi trạng thái phải gắn được vào một US. Không có US thì không vẽ.

Chỉ có **một** cửa sổ và **một** tray menu. Không có settings window (config là file — [US-C5](USER-STORIES.md#us-c5--cấu-hình-được)), không có onboarding (một người dùng — [PRD §2](PRD.md#2-người-dùng)).

---

## W1 · Cửa sổ lịch sử — trạng thái thường

Story: [US-C1](USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt) (mở), [US-B2](USER-STORIES.md#us-b2--tìm-trong-lịch-sử) (ô tìm), [US-B3](USER-STORIES.md#us-b3--dùng-lại-một-item) (danh sách)

```
┌──────────────────────────────────────────────────┐
│ 🔍 [                                   ]  ● sync │  ← ô tìm có con trỏ sẵn [N6]
├──────────────────────────────────────────────────┤
│ 📌 │ ssh-key-prod-2026                    3 ngày │  ← ghim luôn nằm trên
│    │ https://github.com/huy/x2clip/pull/…  2 phút│  ← đang chọn (row 1 mặc định)
│    │ [🖼 320×200]                          5 phút│  ← ảnh: thumb + kích thước
│    │ docker compose up -d --build          1 giờ │
│    │ nội dung dài sẽ bị cắt một dòng, không …    │
│    │ ⚠ [🖼 4000×3000] quá lớn, không sync  2 giờ │  ← synced = 0
├──────────────────────────────────────────────────┤
│ ↑↓ chọn · ⏎ dùng lại · ⌘P ghim · ⌫ xoá · esc     │
└──────────────────────────────────────────────────┘
```

| Vùng | Nội dung | Nguồn dữ liệu |
|---|---|---|
| Ô tìm | Rỗng khi mở, **con trỏ đã ở trong** | — |
| Chỉ báo sync | Một dấu tròn + chữ, xem [W6](#w6--trạng-thái-kết-nối) | [US-C2](USER-STORIES.md#us-c2--biết-được-sync-có-đang-chạy-hay-không) |
| Mỗi hàng | cột ghim · nội dung 1 dòng · thời gian tương đối | `pinned`, `body`/`thumb`, `updated_at` ([ARCHITECTURE §7](ARCHITECTURE.md#7-lưu-trữ)) |
| Chân | Nhắc phím tắt, luôn hiện | — |

**Thứ tự:** `pinned` trước, rồi `updated_at` giảm dần. Copy lại nội dung cũ thì item nhảy lên đầu, **không** tạo hàng mới.

**Một dòng mỗi item, không preview pane.** Muốn xem đủ thì dùng lại rồi paste — đó là việc của app này.

## W2 · Đang tìm

Story: [US-B2](USER-STORIES.md#us-b2--tìm-trong-lịch-sử)

```
┌──────────────────────────────────────────────────┐
│ 🔍 [dock                              ]  ● sync  │
├──────────────────────────────────────────────────┤
│    │ **dock**er compose up -d --build      1 giờ │  ← khớp được tô
│    │ ~/.config/**dock**er/daemon.json      2 ngày│
└──────────────────────────────────────────────────┘
```

Lọc ngay khi gõ, không cần Enter. Ảnh không có text nên bị lọc ra hết khi ô tìm khác rỗng. Ngưỡng ở [N5](NFR.md#1-ngưỡng-chấp-nhận).

## W3 · Không có kết quả / lịch sử rỗng

Story: [US-B2](USER-STORIES.md#us-b2--tìm-trong-lịch-sử), [US-B1](USER-STORIES.md#us-b1--lịch-sử-được-lưu-lại)

```
┌──────────────────────────────────────────────────┐   ┌──────────────────────────┐
│ 🔍 [zzz                               ]  ● sync  │   │ 🔍 [        ]     ● sync │
├──────────────────────────────────────────────────┤   ├──────────────────────────┤
│                                                  │   │                          │
│         Không có gì khớp "zzz"                   │   │   Chưa có gì trong       │
│                                                  │   │   lịch sử. Copy thử một  │
└──────────────────────────────────────────────────┘   │   đoạn text.             │
                                                        └──────────────────────────┘
```

Hai câu khác nhau. "Rỗng vì chưa dùng" và "rỗng vì lọc" là hai tình huống khác nhau; dùng chung một câu là làm người dùng đoán.

## W4 · Ghim

Story: [US-B4](USER-STORIES.md#us-b4--ghim-item)

Toggle tại chỗ, không dialog. Item nhảy lên nhóm ghim ngay, cột 📌 sáng lên. Bỏ ghim thì rơi về đúng vị trí theo `updated_at`.

Item đã ghim thì **prune không đụng tới** ([N14](NFR.md#3-giới-hạn)) — UI không cần nói, nhưng đây là lý do tính năng này tồn tại.

## W5 · Xoá

Story: [US-B5](USER-STORIES.md#us-b5--xoá-item)

```
┌──────────────────────────────────────────────────┐
│    │ ssh-key-prod-2026                           │
│    ├──────────────────────────────────────────┐  │
│    │ Xoá khỏi lịch sử máy này?                │  │
│    │ Máy kia vẫn còn bản của nó.              │  │  ← bắt buộc có câu này
│    │                      [Huỷ]  [Xoá]        │  │
│    └──────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

Câu **"Máy kia vẫn còn bản của nó"** là yêu cầu, không phải trang trí: lịch sử là local mỗi máy, xoá ở A không xoá ở B ([ADR-0004](ADR/0004-storage-sqlite-local-history.md)). Bỏ câu đó là UI nói dối về mô hình dữ liệu.

Xoá là **cứng**, không undo, không trash — clipboard có thể chứa password ([R5](RISKS.md#r5--password-vào-lịch-sử-trên-linux)), xoá phải xoá thật.

## W6 · Trạng thái kết nối

Story: [US-C2](USER-STORIES.md#us-c2--biết-được-sync-có-đang-chạy-hay-không), [US-A4](USER-STORIES.md#us-a4--tạm-dừng-sync)

Ba trạng thái, đúng ba, khớp [ROADMAP Phase 4](ROADMAP.md#phase-4--ui--):

| Trạng thái | Trong cửa sổ | Trên tray |
|---|---|---|
| Đã kết nối | `● sync` | icon thường |
| Mất kết nối | `○ mất kết nối` | icon mờ |
| Tạm dừng | `⏸ tạm dừng` | icon có gạch |

Phân biệt bằng **hình dạng + chữ**, không chỉ bằng màu (xem [MOCKUPS § Màu không phải kênh duy nhất](MOCKUPS.md#màu-không-phải-kênh-duy-nhất)).

Lỗi khác (config sai, DB không mở được…) hiện thành một dải trên đầu danh sách, không phải dialog — không được có lỗi im lặng ([NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi)):

```
├──────────────────────────────────────────────────┤
│ ⚠ Config sai dòng 4: peers phải là mảng          │
├──────────────────────────────────────────────────┤
```

## W7 · Tray menu

Story: [US-C2](USER-STORIES.md#us-c2--biết-được-sync-có-đang-chạy-hay-không), [US-A4](USER-STORIES.md#us-a4--tạm-dừng-sync), [US-C1](USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt)

```
┌──────────────────────────┐
│ ● nixos — đã kết nối     │  ← không bấm được, chỉ hiện
├──────────────────────────┤
│ Mở lịch sử        ⌘⇧V    │
│ Tạm dừng sync            │  ← toggle, đang dừng thì thành "Tiếp tục sync"
├──────────────────────────┤
│ Thoát                    │
└──────────────────────────┘
```

Bốn dòng. Không nhét lịch sử vào tray menu — đó là việc của cửa sổ.

---

## Bàn phím

Story: [US-C1](USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt), [US-B3](USER-STORIES.md#us-b3--dùng-lại-một-item)

| Phím | Việc |
|---|---|
| `⌘⇧V` / `Ctrl⇧V` | Mở cửa sổ ở bất kỳ đâu ([N6](NFR.md#1-ngưỡng-chấp-nhận)) |
| gõ chữ | Vào ô tìm (con trỏ đã ở đó) |
| `↑` `↓` | Chọn |
| `⏎` | Dùng lại item đang chọn → ghi clipboard → **đóng cửa sổ** |
| `⌘P` / `Ctrl+P` | Ghim / bỏ ghim |
| `⌫` | Xoá (qua [W5](#w5--xoá)) |
| `esc` | Đóng, không làm gì |

`⏎` xong là đóng cửa sổ. Người dùng mở app này để đi paste chỗ khác, giữ cửa sổ mở là bắt họ đóng tay.

**Không** có chọn nhiều item, không drag-drop, không phím số nhảy nhanh — chưa có US nào cần.

## Chỗ cố tình không vẽ

| Không có | Vì sao |
|---|---|
| Settings window | Config là file, sửa bằng editor ([US-C5](USER-STORIES.md#us-c5--cấu-hình-được)) |
| Onboarding / welcome | Một người dùng, chính chủ project |
| Preview pane | Một dòng là đủ để nhận ra item |
| Nhóm / tag / pinboard | [R9](RISKS.md#r9--scope-trôi-sang-làm-lại-cleanclip) — đó là feature list của CleanClip |
| Màn hình lịch sử của máy kia | v1 không sync lịch sử ([PRD §4](PRD.md#4-ngoài-scope)) |
