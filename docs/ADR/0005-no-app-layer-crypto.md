# ADR-0005 · Không mã hoá tầng app, dựa vào WireGuard

**Trạng thái:** Accepted — **có điều kiện**
**Ngày:** 2026-08-17

Đây là ADR duy nhất trong tập này mà **sai là hỏng thật**. Đọc hết trước khi đổi transport.

## Bối cảnh

Clipboard chứa password, token, khoá riêng. [NFR N18](../NFR.md#4-bảo-mật) đặt ràng buộc: **mọi byte đi qua mạng phải được mã hoá.** Đây là ranh giới tin cậy, không phải chỗ để đơn giản hoá.

Câu hỏi: app có nên tự mã hoá payload trước khi gửi, **thêm** vào lớp mã hoá của transport?

## Quyết định

**Không.** Payload đi trên WebSocket dưới dạng JSON không mã hoá thêm. Bảo mật đường truyền do **WireGuard (Tailscale)** đảm nhiệm ([ADR-0001](0001-transport-tailscale.md)).

Kèm hai biện pháp bắt buộc, **không** phải tuỳ chọn:
- **[N19](../NFR.md#4-bảo-mật):** socket lắng nghe bind **chỉ** vào địa chỉ Tailscale, không `0.0.0.0`. Không bao giờ expose ra LAN hay internet.
- **[N20](../NFR.md#4-bảo-mật):** chỉ nhận kết nối từ peer có trong config; peer lạ → từ chối + log ([T10](../TEST-PLAN.md#t10--peer-lạ-bị-từ-chối) canh).

## Điều kiện — đọc kỹ

Quyết định này **chỉ đúng khi** transport tự nó đã mã hoá và xác thực. Nó dựa trực tiếp vào ADR-0001.

> **Nếu đổi transport sang thứ không mã hoá đầu-cuối — LAN thuần, TCP trần, một relay công khai, MQTT broker — thì [N18](../NFR.md#4-bảo-mật) mất chỗ dựa và mã hoá tầng app trở thành BẮT BUỘC.**

Cùng mạng LAN **không phải** một ranh giới bảo mật. "Chỉ trong mạng nhà" không phải lý do bỏ mã hoá.

Cụ thể: nếu sau này chuyển sang relay Cloudflare DO ([ADR-0001 § Phương án đã loại](0001-transport-tailscale.md#tự-dựng-relay-server-cloudflare-worker--durable-object)), relay đó **phải** không đọc được nội dung → phải có E2E ở tầng app. ADR này khi đó bị supersede, không phải "điều chỉnh nhẹ".

## Phương án đã loại

### Mã hoá E2E tầng app bằng passphrase chung, thêm lên trên WireGuard
**Loại vì:** với transport hiện tại, đây là mã hoá hai lần cho cùng một đoạn đường. Nó không chống được thêm mối đe doạ nào có thật trong mô hình này — kẻ tấn công đã vào được máy thì đọc clipboard trực tiếp, không cần bắt gói.

Đổi lại nó thêm một chỗ **có thể sai lặng lẽ**: tự chọn cipher, tự sinh nonce, tự quản khoá, tự lo rotation. Crypto tự viết sai thì không kêu — nó vẫn chạy, vẫn trông như đã mã hoá.

**Kích hoạt xem lại:** đổi transport (xem § Điều kiện). Khi đó **không** tự viết — dùng thư viện đã kiểm chứng, giao thức đã có tên (ví dụ Noise), không tự ghép primitive.

### Mã hoá nội dung khi lưu trong DB (encryption at rest)
**Loại vì:** khoá phải nằm đâu đó trên cùng máy đó, nên nó chỉ chống được kẻ đọc file DB mà **không** chạy được code với quyền của bạn — một mô hình tấn công hẹp. Full-disk encryption của hệ điều hành đã phủ đúng chỗ này, tốt hơn và không phải app lo.

Thay vào đó dùng quyền file `0600` ([N21](../NFR.md#4-bảo-mật)) và cho phép xoá nhanh ([US-B5](../USER-STORIES.md#us-b5--xoá-item)).

### Xác thực peer bằng token riêng của app
**Loại vì:** Tailscale đã xác thực máy. Thêm một tầng token là thêm một secret phải quản. [N20](../NFR.md#4-bảo-mật) (whitelist peer theo config) là đủ ở tầng này.

## Hệ quả

### Được
- **Không có code crypto nào để viết sai** — đây là lợi ích chính, không phải phụ
- Không có khoá để sinh, phân phối, lưu, rotate
- Frame debug được bằng mắt lúc phát triển
- Không tốn CPU mã hoá cho item ảnh lớn

### Mất
- **Bảo mật đường truyền phụ thuộc hoàn toàn vào việc Tailscale được cấu hình đúng.** Ai đó tự `tailscale down` rồi chạy app trên LAN thường là chạy không mã hoá — mà app **không tự biết**
- Quyết định này ràng buộc [ADR-0001](0001-transport-tailscale.md): hai ADR phải đổi cùng nhau

### Việc phải làm ở Phase 2
Vì "app không tự biết mình đang không được bảo vệ" là chỗ hở duy nhất còn lại:

- [ ] Bind chỉ vào địa chỉ Tailscale ([N19](../NFR.md#4-bảo-mật)). Không tìm được địa chỉ Tailscale → **từ chối listen**, báo lỗi rõ; **không** fallback sang `0.0.0.0`
- [ ] Whitelist peer theo config ([N20](../NFR.md#4-bảo-mật)), có [T10](../TEST-PLAN.md#t10--peer-lạ-bị-từ-chối)
- [ ] Không log nội dung clipboard ([N23](../NFR.md#4-bảo-mật)), có [T12](../TEST-PLAN.md#t12--không-log-nội-dung-clipboard)

Điểm quan trọng nhất là dòng đầu: **fallback im lặng sang `0.0.0.0` sẽ biến quyết định này từ "hợp lý" thành "lỗ bảo mật"**, và không có gì nhìn thấy được để cảnh báo.
