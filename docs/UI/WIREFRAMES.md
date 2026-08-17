# Wireframes — x2clip

> Nguồn chính cho: **cấu trúc** các màn hình và **ma trận trạng thái**. Màu sắc/typography ở [MOCKUPS.md](MOCKUPS.md). Bản bấm được ở [PROTOTYPE.md](PROTOTYPE.md).
>
> Đây là tài liệu chuẩn bị cho [Phase 4](../ROADMAP.md#phase-4--ui--) — chưa code.

Wireframe = bố cục và thứ bậc thông tin, cố tình **không** có màu và font. Nếu bố cục sai thì tô màu đẹp cũng vô ích.

## 0. Có bao nhiêu bề mặt

App này không phải "một cửa sổ". Có 6 bề mặt, mỗi cái gắn với story riêng:

| # | Bề mặt | Story |
|---|---|---|
| 1 | Cửa sổ lịch sử (mở bằng phím tắt) | [US-B2](../USER-STORIES.md#us-b2--tìm-trong-lịch-sử), [US-B3](../USER-STORIES.md#us-b3--dùng-lại-một-item), [US-B4](../USER-STORIES.md#us-b4--ghim-item), [US-B5](../USER-STORIES.md#us-b5--xoá-item), [US-C1](../USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt) |
| 2 | Tray icon + menu | [US-C2](../USER-STORIES.md#us-c2--biết-được-sync-có-đang-chạy-hay-không), [US-A4](../USER-STORIES.md#us-a4--tạm-dừng-sync) |
| 3 | Cửa sổ cấu hình | [US-C5](../USER-STORIES.md#us-c5--cấu-hình-được) |
| 4 | Xác nhận xoá toàn bộ | [US-B5](../USER-STORIES.md#us-b5--xoá-item) |
| 5 | Trạng thái rỗng + lỗi | [NFR § Hành vi khi lỗi](../NFR.md#5-hành-vi-khi-lỗi) |
| 6 | Đăng nhập + passphrase *(sau v1)* | [US-D1](../USER-STORIES.md#us-d1--đăng-nhập-trên-máy-mới), [US-D4](../USER-STORIES.md#us-d4--đăng-xuất-không-mất-lịch-sử) |

Bề mặt 5 hay bị bỏ quên nhất, và nó chính là chỗ [exit criteria Phase 4](../ROADMAP.md#phase-4--ui--) yêu cầu "không có lỗi im lặng".

---

## 1. Cửa sổ lịch sử

Kích thước cố định 640×420, hiện giữa màn hình đang focus. Không có title bar hệ điều hành — cửa sổ này là overlay, không phải app window.

```
┌────────────────────────────────────────────────────────────┐
│ 🔍  tìm...                                        ● nixos  │ ← thanh tìm, con trỏ sẵn ở đây
├────────────────────────────────────────────────────────────┤
│ 📌 ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbq…    2 ngày  │ ← ghim luôn nằm trên cùng
├────────────────────────────────────────────────────────────┤
│    docker compose up -d --build                   3 phút   │ ← hàng đang chọn
│  ┌────┐                                                    │
│  │▓▓▓▓│ Ảnh · 1284×892 · PNG · 412 KB              8 phút   │ ← hàng ảnh, cao gấp đôi
│  └────┘                                                    │
│    https://github.com/huy/x2clip/pull/12          12 phút  │
│    Lorem ipsum dolor sit amet, consectetur adip…  1 giờ    │
│  ⚠ Ảnh 14 MB — vượt giới hạn, không sync          2 giờ    │
├────────────────────────────────────────────────────────────┤
│ ↑↓ chọn   ⏎ chép   ⌘P ghim   ⌘⌫ xoá   esc đóng             │ ← thanh phím, luôn hiện
└────────────────────────────────────────────────────────────┘
```

**Quy tắc bố cục**

- **Ô tìm kiếm nằm trên cùng và luôn có con trỏ.** [US-C1](../USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt) yêu cầu gõ được ngay, không phải click trước. Đây là ràng buộc bố cục, không phải chi tiết trang trí — để ô tìm ở dưới thì mắt và tay lệch nhau.
- **Item ghim nổi lên đầu**, có đường kẻ ngăn với phần còn lại. Ghim mà vẫn phải cuộn tìm thì ghim vô nghĩa.
- **Một dòng = một item.** Text cắt ở một dòng, `…` ở cuối. Không preview nhiều dòng — người dùng nhận ra item bằng đoạn đầu.
- **Hàng ảnh cao gấp đôi**, thumbnail 48×48 bên trái, metadata bên phải. Thumbnail lấy từ cột `thumb` ([ADR-0004](../ADR/0004-storage-sqlite-local-history.md)) — **không** decode blob gốc để vẽ list.
- **Thanh phím ở đáy luôn hiện.** Đây là app dùng bằng bàn phím; giấu phím tắt trong menu là bắt người dùng nhớ.
- **Nút ghim/xoá không hiện sẵn** — chỉ hiện khi hover hoặc khi hàng đang được chọn. Sáu icon nhân N hàng = nhiễu.

**Hợp đồng bàn phím** (phần quan trọng nhất của màn hình này)

| Phím | Việc |
|---|---|
| gõ chữ | lọc ngay, không cần Enter |
| `↑` `↓` | di chuyển ô chọn, cuộn theo |
| `⏎` | chép item đang chọn vào clipboard **rồi đóng cửa sổ** |
| `⌘P` / `Ctrl+P` | ghim/bỏ ghim item đang chọn, cửa sổ **không** đóng |
| `⌘⌫` / `Ctrl+Del` | xoá item đang chọn, cửa sổ **không** đóng |
| `esc` | đóng, **không đụng vào clipboard** |

`esc` không đụng clipboard là acceptance criteria, không phải tuỳ chọn: mở nhầm rồi thoát mà clipboard bị đổi thì lần paste sau ra sai nội dung.

---

## 2. Tray icon + menu

```
    ●            ○            ⏸
  đã kết nối   mất kết nối   tạm dừng
```

```
┌──────────────────────────┐
│ x2clip · đã kết nối      │ ← dòng trạng thái, không bấm được
│ nixos · 12s trước        │
├──────────────────────────┤
│ Mở lịch sử       ⌘⇧V     │
│ Tạm dừng sync            │
├──────────────────────────┤
│ Cấu hình…                │
│ Thoát                    │
└──────────────────────────┘
```

Ba trạng thái phải phân biệt được **bằng hình dạng, không chỉ bằng màu** — tray icon trên macOS là đơn sắc theo menu bar, màu đỏ/xanh sẽ biến mất. Dùng: chấm đặc / chấm rỗng / biểu tượng pause.

Dòng thứ hai trong menu là **lần bắt tay gần nhất với peer**. Không có nó thì "đã kết nối" là lời hứa không kiểm chứng được.

---

## 3. Ma trận trạng thái

Đây là phần khiến bộ tài liệu này có ích thật. Mỗi ô là thứ phải code ở Phase 4.

| Tình huống | Tray | Cửa sổ lịch sử | Nguồn |
|---|---|---|---|
| Bình thường | ● + tên peer + "12s trước" | Danh sách đầy đủ | — |
| Peer offline | ○ + "nixos offline · 4 phút" | Vẫn dùng bình thường, banner mảnh trên đầu: "Không thấy nixos — lịch sử local vẫn chạy" | [N3](../NFR.md#1-ngưỡng-chấp-nhận) |
| Tailscale không chạy | ● (sync **vẫn chạy**) | Banner mảnh: "Mất kênh báo nhanh — sync vẫn chạy, chỉ chậm hơn" | [ADR-0006 § 6b](../ADR/0006-r2-mailbox-store-and-forward.md) — Tailscale chỉ là chuông. Gọi đây là "mất sync" là **sai** |
| Không tới được R2 | ○ + "không tới được R2" | Banner: "Không kết nối được R2 — item vẫn lưu local, sẽ gửi khi có mạng" | [NFR § lỗi](../NFR.md#5-hành-vi-khi-lỗi) — nói chữ *R2*, không phải "sync failed" |
| Access key sai / hết hạn | ⚠ | Banner đỏ: "Không vào được hộp thư — key sai hoặc hết hạn" + nút "Đăng nhập lại". **Phân biệt hẳn với mất mạng** | [NFR § lỗi](../NFR.md#5-hành-vi-khi-lỗi) |
| Chưa đăng nhập (máy mới) | ○ + "chưa đăng nhập" | Màn hình đăng nhập ở [§ 3b](#3b-đăng-nhập-và-passphrase), không phải danh sách rỗng | [US-D1](../USER-STORIES.md#us-d1--đăng-nhập-trên-máy-mới) |
| Token hết hạn | ○ + "cần đăng nhập lại" | **Danh sách vẫn dùng bình thường** + banner: "Cần đăng nhập lại để sync — lịch sử local vẫn tra được" | [N33](../NFR.md#6-khả-năng-vận-hành), [US-D2](../USER-STORIES.md#us-d2--máy-cũ-không-chết-theo-lỗi-đăng-nhập) |
| Passphrase sai | ⚠ | "Passphrase không khớp — không giải mã được hộp thư". **Không** phải "đăng nhập thất bại" | [N18i](../NFR.md#4-bảo-mật) |
| Tạm dừng | ⏸ + "Đã tạm dừng" | Banner: "Đang tạm dừng — vẫn lưu local, không gửi đi" | [US-A4](../USER-STORIES.md#us-a4--tạm-dừng-sync) |
| Phím tắt bị chiếm | ● (sync vẫn chạy) | Báo một lần lúc khởi động: "⌘⇧V đang bị app khác dùng. Mở lịch sử từ tray, hoặc đổi phím trong Cấu hình" | [NFR § lỗi](../NFR.md#5-hành-vi-khi-lỗi) |
| Không đọc được clipboard | ⚠ | Banner đỏ: "Không đọc được clipboard" + lý do cụ thể (thiếu `DISPLAY`/`XAUTHORITY`, hoặc compositor không hỗ trợ) | [Phase 0.2 ghi chú 3](../ROADMAP.md#kết-quả-02-2026-08-17) |
| DB lỗi | ⚠ | Banner đỏ: "Không mở được lịch sử — file DB có vấn đề. **Chưa xoá gì cả.**" + đường dẫn file | [NFR § lỗi](../NFR.md#5-hành-vi-khi-lỗi) — không tự xoá |
| Lịch sử rỗng (lần đầu) | ● | "Chưa có gì. Copy một thứ gì đó đi." | — |
| Tìm không ra | ● | "Không có item nào khớp *docker*." | — |

Banner là **một dòng, nằm ngay dưới thanh tìm**, không phải dialog. Dialog chặn đường; lỗi sync không đáng chặn người dùng lấy item cũ.

---

## 3b. Đăng nhập và passphrase

Bề mặt thứ 6, thêm 2026-08-17 cùng [ADR-0007](../ADR/0007-dang-nhap-va-khoa-tu-passphrase.md). **Chỉ hiện trên máy chưa từng đăng nhập** — không phải màn hình mở app hằng ngày.

```
┌─────────────────────────────────────────────┐
│ x2clip                                      │
│                                             │
│ Máy này chưa được thêm vào tài khoản.       │
│                                             │
│            [ Đăng nhập ]                    │
└─────────────────────────────────────────────┘

        ↓ sau khi đăng nhập xong

┌─────────────────────────────────────────────┐
│ Passphrase                                  │
│ [ ••••••••••••••••••••             ]        │
│                                             │
│ Cùng passphrase với các máy khác của bạn.   │
│ Không ai khôi phục được nếu quên — kể cả    │
│ chính x2clip.                               │
│                                             │
│                      [ Tiếp tục ]           │
└─────────────────────────────────────────────┘
```

**Quy tắc**

- **Đúng hai bước, không có bước ba.** Nếu bản dựng nào bắt chép file thì bản đó sai [G6](../PRD.md#3-mục-tiêu).
- **Câu cảnh báo mất-là-mất phải nằm ngay cạnh ô nhập**, không nhét vào tooltip hay trang trợ giúp. Đó là lúc duy nhất người dùng còn cơ hội đi cất passphrase vào password manager.
- **Không có nút "Quên passphrase?"** — không có luồng đó, và một nút dẫn tới ngõ cụt còn tệ hơn không có nút. Xem [R11](../RISKS.md#r11--mất-khoá-mã-hoá).
- Passphrase hỏi **một lần mỗi máy**. Mở app lần sau đi thẳng vào lịch sử.
- Nhập sai → chữ dưới ô: *"Passphrase không khớp — không giải mã được hộp thư"*. Đây là câu bắt buộc ([N18i](../NFR.md#4-bảo-mật)); "sai mật khẩu" là câu sai, nó chỉ người dùng đi gỡ nhầm chỗ.

Danh sách máy + nút thu hồi ([US-D3](../USER-STORIES.md#us-d3--xem-và-thu-hồi-máy)) nằm trong [§ 4 Cấu hình](#4-cấu-hình), không phải màn hình riêng.

---

## 4. Cấu hình

Không làm cửa sổ nhiều tab. Một trang, cuộn được.

```
┌─────────────────────────────────────────────┐
│ Cấu hình                                    │
├─────────────────────────────────────────────┤
│ Peer            [ nixos              ]      │
│ Cổng            [ 47231              ]      │
│ Phím tắt        [ ⌘⇧V   ]  [ Đổi… ]         │
│ Poll interval   [ 250   ] ms                │
│ Giữ tối đa      [ 1000  ] item              │
│ Ảnh tối đa      [ 10    ] MB                │
│ Tự chạy lúc đăng nhập          [x]          │
├─────────────────────────────────────────────┤
│ Sửa trực tiếp: ~/.config/x2clip/config.toml │
├─────────────────────────────────────────────┤
│ Máy đã đăng nhập              (sau v1)      │
│   macbook   · vừa xong                      │
│   nixos     · 2 giờ trước                   │
│   thinkpad  · 14 ngày trước    [ Thu hồi ]  │
├─────────────────────────────────────────────┤
│  [ Đăng xuất máy này ]                      │ ← giữ nguyên lịch sử local
│              [ Xoá toàn bộ lịch sử ]        │ ← hành động phá huỷ, tách riêng dưới cùng
└─────────────────────────────────────────────┘
```

**Đăng xuất và xoá lịch sử là hai việc khác nhau, phải trông khác nhau.** Đăng xuất giữ nguyên lịch sử local ([US-D4](../USER-STORIES.md#us-d4--đăng-xuất-không-mất-lịch-sử)); gộp hai nút cạnh nhau cùng kiểu là cách chắc chắn để bấm nhầm cái không hoàn tác được.

UI cấu hình **ghi vào cùng file TOML** người dùng sửa tay, không phải kho riêng. Hai nguồn sự thật cho config là cách chắc chắn để chúng lệch nhau.

## 5. Xác nhận xoá toàn bộ

```
┌────────────────────────────────────────┐
│ Xoá toàn bộ 1000 item?                 │
│                                        │
│ Kể cả 3 item đã ghim.                  │
│ Chỉ xoá trên máy này — máy kia giữ     │
│ nguyên lịch sử của nó.                 │
│                                        │
│              [ Huỷ ]  [ Xoá hết ]      │
└────────────────────────────────────────┘
```

Ba thông tin bắt buộc: **số lượng**, **có đụng item ghim không**, **local-only**. Thiếu cái thứ ba thì người dùng tưởng đã xoá cả hai máy.

Xoá **một** item thì không hỏi — đúng một hàng, đảo ngược được bằng cách copy lại.

---

## 6. Ghi chú ràng buộc kỹ thuật

**X11 giữ selection.** [Phase 0.2](../ROADMAP.md#kết-quả-02-2026-08-17) đã chứng minh clipboard X11 là owner-based. Nghĩa là [US-B3](../USER-STORIES.md#us-b3--dùng-lại-một-item) (bấm item → chép) trên Linux **không phải** "ghi rồi xong": tiến trình app phải sống và giữ selection. Đóng cửa sổ ≠ thoát app nên vẫn ổn — nhưng đừng thiết kế luồng nào chép-rồi-thoát-tiến-trình.

**Ngân sách 200ms.** [N6](../NFR.md#1-ngưỡng-chấp-nhận) tính từ lúc bấm phím tắt đến lúc gõ được. Hệ quả bố cục: cửa sổ phải **giữ sẵn ở trạng thái ẩn** rồi hiện lên, không tạo mới mỗi lần. Trang đầu chỉ query ~50 item; phần còn lại nạp khi cuộn.

**Chưa làm:** theme, đổi màu, đổi bố cục, mật độ danh sách. [USER-STORIES](../USER-STORIES.md) đã xếp ra ngoài scope — chưa dùng thật thì chưa biết cần gì.
