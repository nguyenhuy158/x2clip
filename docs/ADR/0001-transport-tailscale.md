# ADR-0001 · Dùng Tailscale làm transport, không tự dựng relay

**Trạng thái:** **Một phần Superseded by [ADR-0006](0006-r2-mailbox-store-and-forward.md)** (2026-08-17)
**Ngày:** 2026-08-17

> Giả định nền của ADR này — *hai máy cùng online* — đã được xác nhận **sai**: mac ở công ty, nixos ở nhà, ít khi mở cùng lúc. Đúng cái ADR này ghi ở § Mất ("Tailscale là mạng, không phải kho") đã thành vấn đề thật.
>
> **Còn hiệu lực:** không tự dựng relay; không tự làm identity / NAT traversal.
> **Hết hiệu lực:** Tailscale chở nội dung clipboard. Nội dung giờ đi qua hộp thư R2; Tailscale hạ cấp thành kênh thông báo tuỳ chọn. Xem [ADR-0006](0006-r2-mailbox-store-and-forward.md).

## Bối cảnh

Hai máy (macOS + NixOS) **không cùng mạng**. Cần đưa nội dung clipboard từ máy này sang máy kia. Clipboard chứa password và token, nên mọi byte qua mạng phải được mã hoá ([NFR N18](../NFR.md#4-bảo-mật)).

Sản phẩm tham chiếu — CleanClip, Paste — giải bài này bằng **iCloud/CloudKit**: không server riêng, Apple lo identity, mã hoá, xuyên NAT. Nhưng cả hai chỉ chạy trên hệ Apple. NixOS không có CloudKit, nên cách đó không dùng lại được.

Bài toán thật vì vậy là: **tìm thứ đóng đúng vai trò CloudKit** — hạ tầng có sẵn lo identity, mã hoá và xuyên NAT, để app không phải tự làm.

## Quyết định

Dùng **Tailscale** làm transport. App chỉ mở WebSocket tới tên MagicDNS của peer.

App **không** biết gì về Tailscale: `peer.rs` chỉ thấy một hostname và một port.

## Phương án đã loại

### Tự dựng relay server (Cloudflare Worker + Durable Object)
DO giữ WebSocket cho cả hai client, client mã hoá E2E bằng passphrase chung nên relay không đọc được.

**Loại vì:** phải tự viết crypto E2E, tự quản khoá, tự vận hành và tự trả tiền cho một service — trong khi chỉ có hai máy của cùng một người. Tự viết crypto là chỗ dễ sai nhất trong toàn bộ project này.

**Ưu điểm bị bỏ:** chạy được ở máy không cài được Tailscale; hai máy không cần cùng online.

**Kích hoạt xem lại:** cần dùng ở máy không cài được Tailscale, hoặc [R8](../RISKS.md#r8--mất-item-khi-hai-máy-lệch-giờ-online--đóng-2026-08-17) xảy ra thường xuyên thật.

### SSH làm transport
`ssh peer pbcopy` — auth + mã hoá miễn phí, không viết protocol.

**Loại vì:** vẫn cần xuyên NAT khi khác mạng, tức vẫn cần một lớp mạng bên dưới. Và spawn process mỗi lần copy thì không đạt [N1](../NFR.md#1-ngưỡng-chấp-nhận). Nếu chỉ cùng LAN thì đây mới là phương án lười nhất.

### MQTT / broker công khai
**Loại vì:** thêm một service phải tin, vẫn phải tự mã hoá E2E, không được gì so với DO relay.

### Tự dựng WireGuard hoặc Headscale
Cùng tính chất kỹ thuật với Tailscale, không phụ thuộc nhà cung cấp.

**Loại vì:** phải tự vận hành coordination server và quản khoá. Vì app không biết gì về Tailscale, **đổi sang cái này về sau là đổi cấu hình mạng, không đổi code** — nên chưa cần làm bây giờ.

### Syncthing / thư mục chia sẻ
**Loại vì:** đồng bộ file, không phải clipboard. Trễ sai bậc, và biến clipboard thành file là mô hình sai.

## Hệ quả

### Được
- **Không backend nào để viết, deploy hay bảo trì** ([PRD G4](../PRD.md#3-mục-tiêu))
- Mã hoá và identity coi như xong: WireGuard + tài khoản Tailscale ([N18](../NFR.md#4-bảo-mật))
- Xuyên NAT không phải việc của app
- Bind vào interface Tailscale là ranh giới tin cậy tự nhiên ([N19](../NFR.md#4-bảo-mật))
- Thêm máy thứ ba là thêm một dòng config

### Mất
- **Phải cài Tailscale trên cả hai máy** — một dependency ngoài, không phải app tự đủ
- **Hai máy phải cùng online.** Tailscale là mạng, không phải kho. Item copy lúc máy kia tắt thì không tới ([R8](../RISKS.md#r8--mất-item-khi-hai-máy-lệch-giờ-online--đóng-2026-08-17))
- Phụ thuộc một nhà cung cấp ([R7](../RISKS.md#r7--phụ-thuộc-tailscale))
- Nếu sau này đổi sang transport **không** mã hoá thì [N18](../NFR.md#4-bảo-mật) mất chỗ dựa, bắt buộc thêm crypto tầng app — xem [ADR-0005](0005-no-app-layer-crypto.md)

### Xác nhận ở Phase 0
[ROADMAP 0.3](../ROADMAP.md#phase-0--spike--): hai máy ping được nhau qua tên MagicDNS và mở được TCP port.
