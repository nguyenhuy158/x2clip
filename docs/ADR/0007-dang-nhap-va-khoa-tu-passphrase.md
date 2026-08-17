# ADR-0007 · Đăng nhập để lấy quyền, khoá dẫn xuất từ passphrase

**Trạng thái:** Accepted (2026-08-17)
**Supersede:** [ADR-0006 § Ràng buộc bảo mật](0006-r2-mailbox-store-and-forward.md) hàng 2 và 3, và [ADR-0005 § C2](0005-no-app-layer-crypto.md) — phần *phân phối* khoá. Phần *mã hoá* của cả hai giữ nguyên.

## Bối cảnh

[ADR-0006](0006-r2-mailbox-store-and-forward.md) bắt cài máy mới phải làm ba việc bằng tay: tạo access key R2, chép nó sang máy, rồi chép cả khoá mã hoá sang máy. Với hai máy dựng một lần thì chịu được. Với [G6](../PRD.md#3-mục-tiêu) ("máy mới đăng nhập là dùng được") thì không.

Chép secret bằng tay còn có vấn đề thật ngoài chuyện phiền: nó đi qua clipboard, qua chat, qua scp — đúng những chỗ mà app này sinh ra để tránh.

## Quyết định

Tách **quyền truy cập** khỏi **khả năng đọc**. Hai thứ này lâu nay bị gộp làm một trong ADR-0006, và đó chính là lý do phải chép hai secret.

### 7a · Đăng nhập → quyền vào hộp thư

Một endpoint nhỏ (`auth`) do mình vận hành. Máy đăng nhập bằng OIDC/passkey qua một identity provider có sẵn — **không tự làm signup, không tự lưu mật khẩu**. Đổi lại, endpoint trả về **credential R2 phạm vi hẹp, có hạn**:

| | Cũ (ADR-0006) | Mới |
|---|---|---|
| Access key | Vĩnh viễn, chép tay, quyền cả bucket | Tạm thời, tự lấy, chỉ prefix của máy này |
| Lộ key = | Đọc được cả bucket, mãi mãi | Đọc được ciphertext của một máy, tới khi hết hạn |
| Thu hồi | Xoá key, chép key mới sang mọi máy | Bỏ máy khỏi danh sách, key hết hạn là xong |

Đây là **thành phần tự vận hành duy nhất** được phép tồn tại ([G4 đã thu hẹp](../PRD.md#3-mục-tiêu)). Nó không nằm trên đường đi của nội dung clipboard: hỏng thì máy cũ vẫn sync bình thường, chỉ không thêm được máy mới.

### 7b · Khoá mã hoá dẫn xuất từ passphrase — server không giữ

```
khoá = Argon2id(passphrase người dùng gõ, salt của account)
```

`salt` lấy từ endpoint `auth` sau khi đăng nhập; **passphrase không bao giờ rời khỏi máy**, cả dạng gốc lẫn dạng băm. Endpoint không có gì để giao nộp và không có gì để lộ.

Dẫn xuất xong thì cất khoá vào Keychain (macOS) / file `0600` (NixOS) — gõ passphrase **một lần mỗi máy**, không phải mỗi lần mở app.

Ràng buộc: **cùng một passphrase trên mọi máy**, vì chung một hộp thư. Gõ sai không phải "sai mật khẩu" mà là **giải mã fail** — app phải nói đúng như vậy, xem [§ Nhiệm vụ bắt buộc](#nhiệm-vụ-bắt-buộc).

### 7c · Cái server biết và không biết

| Server giữ | Server **không** giữ |
|---|---|
| Danh tính account (từ provider) | Passphrase, hay bất cứ thứ gì dẫn ra được nó |
| Danh sách máy + lần đăng nhập cuối | Khoá mã hoá |
| `salt` của account | Nội dung clipboard, kể cả đã mã hoá (nội dung nằm trên R2) |

`salt` không phải secret — nó chống rainbow table, lộ ra không cho ai đọc được gì.

## Cái này **không** đổi

- [N18](../NFR.md#4-bảo-mật) mã hoá tầng app trước khi PUT: **giữ nguyên**, giờ còn quan trọng hơn.
- Hộp thư R2, object layout, ULID random, quy tắc xoá sau khi ingest: **giữ nguyên** ([ADR-0006](0006-r2-mailbox-store-and-forward.md)).
- Tailscale vẫn chỉ là chuông. Đăng nhập không thay thế nó.
- Rò rỉ metadata sang Cloudflare (số item, thời điểm, kích thước): **vẫn còn**.

## Phương án đã loại

### Server giữ luôn khoá — đăng nhập là xong, không gõ gì

Đúng cái người ta hình dung khi nói "kiểu SaaS", và nó **tiện hơn thật**.

**Loại vì:** lúc đó server đọc được clipboard — và ai chiếm được server cũng vậy. Clipboard này chứa password và khoá riêng ([NFR § Bảo mật](../NFR.md#4-bảo-mật)). Đổi một câu passphrase gõ một lần mỗi máy lấy việc đó là đắt.

**Bật lại khi:** chủ project nói rõ chấp nhận đánh đổi. Đây là quyết định của người dùng, không phải của tài liệu — sửa 7b, giữ nguyên phần còn lại.

### Ghép máy trực tiếp, không cần đăng nhập

Máy mới hiện mã 8 chữ số, máy cũ nhập vào, khoá đi qua kênh đã mã hoá sẵn. **Không cần server nào**, giữ nguyên G4 gốc.

**Loại vì:** cần **một máy cũ đang bật** lúc thêm máy mới. Mất hết máy = mất hết. Và nó không giải quyết access key R2 — vẫn phải chép tay cái đó.

**Bật lại khi:** không muốn vận hành endpoint `auth` nữa. Nó rẻ hơn hẳn, chỉ hẹp hơn.

### Tự làm signup, mật khẩu, session

**Loại vì:** thêm một bảng user, một luồng quên-mật-khẩu, một chỗ nữa để rò rỉ — cho đúng **một** người dùng. Provider có sẵn làm việc đó tốt hơn và không phải trực.

## Hệ quả

| | |
|---|---|
| Được | Máy mới không chép file nào. Access key hết hạn được, thu hồi được. Số máy không còn giới hạn |
| Mất | Một service phải vận hành. Một dependency identity provider. Một passphrase phải nhớ |
| Quên passphrase | **Mất toàn bộ lịch sử đã sync.** Không có recovery — đó chính là ý nghĩa của việc server không giữ khoá |
| `auth` chết | Máy cũ **vẫn chạy bình thường**, chỉ không thêm được máy mới. Đây là ràng buộc thiết kế, không phải may mắn |

## Nhiệm vụ bắt buộc

- [ ] Sai passphrase → báo **"passphrase không khớp — không giải mã được hộp thư"**, không phải "đăng nhập thất bại". Hai lỗi khác nhau, sửa bằng hai cách khác nhau
- [ ] Token hết hạn → app vẫn mở, lịch sử local vẫn tra được. Chỉ dừng sync
- [ ] Token và khoá dẫn xuất: Keychain / file `0600`. Không plaintext, không vào log ([N23](../NFR.md#4-bảo-mật))
- [ ] Argon2id dùng tham số mặc định của thư viện, **không tự chỉnh** — [ADR-0005](0005-no-app-layer-crypto.md) đã cấm tự ghép primitive
- [ ] Test: đăng xuất rồi mở lại → lịch sử local còn nguyên
- [ ] Test: hai máy khác passphrase → máy kia fail sạch, giữ object, không crash daemon
