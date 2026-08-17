# ADR-0006 · R2 làm hộp thư, Tailscale làm chuông

**Trạng thái:** Accepted — **§ Ràng buộc bảo mật hàng 2–3 bị [ADR-0007](0007-dang-nhap-va-khoa-tu-passphrase.md) supersede** (2026-08-17). Hộp thư, object layout và ràng buộc mã hoá giữ nguyên; chỉ **cách secret đến được máy** là đổi.
**Ngày:** 2026-08-17
**Supersede:** [ADR-0001](0001-transport-tailscale.md) (phần "Tailscale là đường truyền nội dung"), [ADR-0004 § 4b](0004-storage-sqlite-local-history.md) (phần "không sync lịch sử")
**Kéo theo:** [ADR-0005](0005-no-app-layer-crypto.md) bị supersede — mã hoá tầng app trở thành **bắt buộc**

## Bối cảnh

Giả định nền của [ADR-0001](0001-transport-tailscale.md) là **hai máy cùng online**. Ngày 2026-08-17 chủ project cho biết giả định đó sai:

> Máy mac ở công ty, máy nixos ở nhà. **Hai máy ít khi nào mở cùng lúc.**

Tailscale là *mạng*, không phải *kho*. Không có kho ở giữa thì:

- Copy ở công ty cả ngày → về nhà bật nixos, không có gì cả. Mất trắng.
- G1 ở [PRD](../PRD.md#3-mục-tiêu) — "copy máy này paste máy kia" — không đạt trong đúng tình huống dùng chính.

Đây là điều kiện kích hoạt đã ghi trước ở [ADR-0004 § Xem lại](0004-storage-sqlite-local-history.md#xem-lại-2026-08-17--sync-lịch-sử): *store-and-forward*.

## Quyết định

### 6a · R2 là hộp thư, và là đường **duy nhất** chở nội dung

Mỗi item clipboard được mã hoá rồi `PUT` lên một bucket R2. Object nằm đó tới khi máy kia lấy. Máy nhận `LIST` + `GET` + `DELETE`.

Dùng **S3 API của R2** với access key → app gọi trực tiếp, **không cần dựng Worker**. Giữ được [G4](../PRD.md#3-mục-tiêu) "không tự vận hành backend": R2 là dịch vụ quản lý, không phải service mình deploy và trực.

### 6b · Tailscale hạ cấp thành kênh thông báo

Khi cả hai máy tình cờ cùng online, máy gửi bắn một frame ~100 byte: *"có item mới, key = X"*. Máy nhận `GET` ngay thay vì chờ hết chu kỳ poll.

**Frame thông báo không chứa nội dung clipboard.** Đây là điểm cốt lõi của thiết kế:

| | Nếu Tailscale cũng chở nội dung | Thiết kế này |
|---|---|---|
| Đường chở nội dung | 2 | **1** |
| Chỗ mã hoá | 2 | 1 |
| Chỗ ingest / ghi store | 2 | 1 |
| Phải dedupe giữa 2 đường | Có | Không |
| Mất Tailscale | Rơi sang đường khác, khác hành vi | Chỉ chậm hơn, cùng một đường |

Mất Tailscale không phải lỗi — chỉ là mất chuông, hộp thư vẫn chạy. Vì vậy **kênh thông báo là tuỳ chọn**, làm sau khi hộp thư đã chạy ([ROADMAP](../ROADMAP.md) Phase 2b).

### 6c · Chỉ item mới nhất được ghi vào clipboard

Bật máy sau 8 tiếng, hộp thư có 30 item. Ghi hết vào clipboard theo thứ tự thì clipboard cuối cùng giữ item **cũ nhất trong lô**, và Cmd+V ra thứ copy từ sáng.

Quy tắc: cả lô vào **history**; chỉ item có `ts` lớn nhất được ghi vào **clipboard hệ thống**.

### 6d · Lịch sử tự hội tụ

Vì mọi item đều đi qua hộp thư và mọi máy đều ingest hết vào `store`, hai lịch sử tự giống nhau — không cần cơ chế sync lịch sử riêng. [ADR-0004 § 4b](0004-storage-sqlite-local-history.md) ("lịch sử local mỗi máy") bị supersede; merge rule ở § Xem lại của ADR đó vẫn dùng (hash làm khoá, `pinned` OR, `updated_at` max, tombstone 30 ngày).

## Object layout

```
<bucket>/inbox/<recipient>/<ulid>       ← ciphertext, body là blob nhị phân
```

| Thành phần | Quy tắc |
|---|---|
| `recipient` | Tên máy nhận (`mac`, `nixos`). Máy chỉ LIST prefix của chính mình. |
| `<ulid>` | ULID sinh **random**, sắp được theo thời gian. |
| Metadata | Không có. `kind`, `ts`, `hash` nằm **trong** ciphertext. |

**Object key không được chứa hash nội dung.** Hash plaintext trong tên object cho phép Cloudflare (hoặc ai đọc được bucket) xác nhận một nội dung đoán trước có đi qua hay không — dictionary attack trên chính cái vừa mã hoá. ULID random không rò gì.

**Xoá:** máy nhận `DELETE` sau khi ingest thành công (ghi `store` xong, không phải trước). Kèm lifecycle rule xoá object > 30 ngày để rác không tích lại nếu một máy chết hẳn.

## Ràng buộc bảo mật — không phải tuỳ chọn

Chi tiết ở [ADR-0005 § Xem lại](0005-no-app-layer-crypto.md). Tóm:

| # | Ràng buộc |
|---|---|
| 1 | **Mã hoá tầng app trước khi PUT.** Cloudflare không được đọc clipboard. Dùng thư viện đã kiểm chứng (`age` / libsodium), **không** tự ghép primitive. |
| ~~2~~ | ~~Khoá chia sẻ trước, copy tay sang hai máy~~ → **[ADR-0007 § 7b](0007-dang-nhap-va-khoa-tu-passphrase.md)**: khoá dẫn xuất từ passphrase, không chép tay. Phần "quyền `0600`, không bao giờ vào repo hay lên R2" **vẫn giữ**. |
| ~~3~~ | ~~Access key R2 chép tay~~ → **[ADR-0007 § 7a](0007-dang-nhap-va-khoa-tu-passphrase.md)**: credential tạm thời lấy sau khi đăng nhập. Phần "là secret, Keychain / file `0600`, không hardcode" **vẫn giữ**. |
| 4 | Giải mã lỗi → log + **giữ** object, không xoá. Object không giải được có thể là bug, không phải rác. |

**Rò rỉ metadata phải chấp nhận:** Cloudflare thấy số item, thời điểm, và kích thước mỗi item. Không thấy nội dung. Với mô hình một người dùng đây là đánh đổi chấp nhận được — nhưng nó *có thật*.

## Phương án đã loại

### Giữ Tailscale P2P thuần (nguyên trạng)
**Loại vì:** chính là cái vừa vỡ. Cần cả hai máy online cùng lúc.

### D1 làm hộp thư
**Loại vì:** hộp thư chở blob (ảnh tới [5MB](../NFR.md#3-giới-hạn)); D1 không dành cho blob lớn. Và hộp thư chỉ cần PUT/LIST/GET/DELETE — không cần SQL. Chi tiết ở [ADR-0004 § Xem lại](0004-storage-sqlite-local-history.md#xem-lại-2026-08-17--sync-lịch-sử).

### Durable Object làm hộp thư
**Loại vì:** đúng về kỹ thuật (WebSocket + storage + store-and-forward trong một), nhưng phải viết và deploy code Worker — có version, có log, có thứ để hỏng lúc 3 giờ sáng. R2 qua S3 API là 4 lệnh HTTP, không có code của mình chạy trên server nào.

**Kích hoạt xem lại:** cần push tức thời mà không có Tailscale, hoặc lên 3+ máy khiến fan-out object thành vấn đề.

### R2 làm kho chính, bỏ SQLite local
**Loại vì:** mất mạng là mất cả lịch sử. [US-B1](../USER-STORIES.md#us-b1--lịch-sử-được-lưu-lại) không đòi mạng. SQLite local vẫn là nguồn chân lý; R2 chỉ trung chuyển.

### Syncthing
**Loại vì:** cũng là P2P, cũng cần hai máy cùng online → không giải quyết được vấn đề. Ngoài ra thêm một daemon nữa để trực.

## Hệ quả

### Được
- Copy ở công ty, về nhà vẫn nhận — **đúng tình huống dùng chính**, thứ kiến trúc cũ không làm được
- Lịch sử hai máy tự hội tụ, không cần cơ chế riêng ([6d](#6d--lịch-sử-tự-hội-tụ))
- Không có server nào của mình để trực
- Tailscale hỏng / firewall công ty chặn → app vẫn chạy, chỉ chậm hơn

### Mất
- **Trễ 30–60s** khi không có kênh thông báo, thay vì <1s. Chấp nhận được vì hai máy vốn ít online cùng lúc — nhưng [N1](../NFR.md#1-ngưỡng-chấp-nhận) phải sửa lại cho đúng thực tế.
- **Phụ thuộc Cloudflare.** R2 down = không sync (history local vẫn đọc được).
- **Phải viết crypto** — dùng thư viện, nhưng vẫn là code có thể sai. Trước đây WireGuard lo hộ.
- Rò rỉ metadata cho Cloudflare (số lượng, thời điểm, kích thước).
- Có chi phí, dù nhỏ. Poll 30s ≈ 90k lệnh LIST/tháng/máy. **Kiểm lại bảng giá R2 trước khi chốt interval** — con số free tier ở đây là áng, không phải tra cứu.

### Việc phải làm trước khi viết dòng code sync đầu tiên
- [ ] Tạo bucket + access key, quyền hẹp nhất có thể (chỉ bucket đó)
- [ ] Sinh khoá mã hoá, copy sang hai máy, `chmod 0600`
- [ ] Đặt lifecycle rule 30 ngày trên bucket
- [ ] Xác nhận bảng giá cho poll interval đã chọn
