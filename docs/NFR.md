# Non-Functional Requirements — x2clip

> Nguồn chính cho: mọi con số ngưỡng, giới hạn, hành vi khi lỗi, bảo mật. File khác cần số thì link về đây, không copy số sang.

---

## 1. Ngưỡng chấp nhận

Đây là mức **chấp nhận được**, không phải mục tiêu lý tưởng. Không đạt = fail, phải sửa trước khi đóng phase.

| ID | Tiêu chí | Ngưỡng | Không đạt nghĩa là |
|---|---|---|---|
| N1 | Trễ sync text | < 1s từ lúc copy tới lúc paste được ở máy kia | Poll interval hoặc đường mạng có vấn đề |
| N2 | Trễ sync ảnh 2MB | < 3s | Encode hoặc transport cần tối ưu |
| N3 | Phát hiện mất kết nối | < 10s | Heartbeat quá thưa; người dùng tưởng đang sync |
| N4 | Reconnect sau khi peer trở lại | < 15s | Backoff quá dài |
| N5 | Tìm kiếm ở mức lịch sử tối đa | < 100ms | Thiếu index trong DB |
| N6 | Mở cửa sổ bằng phím tắt | < 200ms tới lúc gõ được | Đang khởi tạo lười lúc mở — phải làm sẵn từ trước |
| N7 | Echo loop | **0 message dư.** Copy 1 lần = đúng 1 message mỗi chiều | Bug chặn release, không phải bug tune sau |
| N8 | Mất item khi cả hai máy online | 0 | Reconnect làm rơi message trong lúc dial lại |

## 2. Tài nguyên

| ID | Tiêu chí | Ngưỡng | Ghi chú |
|---|---|---|---|
| N9 | CPU lúc rảnh | < 1% mỗi máy | Poll loop chỉ được so sánh dấu hiệu rẻ; **không** decode payload đầy đủ khi chưa phát hiện đổi |
| N10 | RAM khi chạy nền | < 150MB | Vượt là do cache ảnh không giới hạn |
| N11 | Pin (macOS laptop) | Không xuất hiện trong danh sách "Apps Using Significant Energy" | Poll 250ms là đánh đổi có ý thức — nếu tốn pin thật thì nới interval trước khi làm gì khác |
| N12 | Kích thước DB | Cảnh báo khi > 500MB | Không tự xoá, chỉ cảnh báo |

## 3. Giới hạn

| ID | Giới hạn | Mặc định | Cấu hình được |
|---|---|---|---|
| N13 | Poll interval | 250ms | Có |
| N14 | Lịch sử giữ tối đa | 1000 item **hoặc** 30 ngày, cái nào tới trước | Có |
| N15 | Dung lượng một item ảnh | 5MB | Có |
| N16 | Độ dài một item text | 1MB | Có |
| N17 | Số item đã ghim | Không giới hạn | — |

Quy tắc kèm theo:
- Item **đã ghim không bao giờ bị prune**, bất kể N14.
- Vượt N15/N16 → **bỏ qua + log**, không cắt bớt. Nội dung cắt dở tệ hơn không có nội dung.
- Vượt N15 với ảnh → item vẫn vào lịch sử local, đánh dấu "quá lớn, không sync" (xem [US-A3](USER-STORIES.md#us-a3--đồng-bộ-ảnh)).

## 4. Bảo mật

Clipboard chứa password, token, khoá riêng. Đây là ranh giới tin cậy, không được đơn giản hoá.

| ID | Yêu cầu |
|---|---|
| N18 | Mọi byte đi qua mạng phải được mã hoá. Đạt được nhờ Tailscale/WireGuard — xem [ADR-0001](ADR/0001-transport-tailscale.md) và [ADR-0005](ADR/0005-no-app-layer-crypto.md) |
| N19 | Socket lắng nghe **chỉ** bind vào địa chỉ Tailscale, không `0.0.0.0`. Không expose ra LAN hay internet |
| N20 | Chỉ nhận kết nối từ peer có trong config. Peer lạ → từ chối + log |
| N21 | DB lịch sử để quyền `0600`, trong thư mục dữ liệu của user |
| N22 | Nội dung đánh dấu nhạy cảm (macOS `org.nspasteboard.ConcealedType`) không lưu, không gửi |
| N23 | Không log nội dung clipboard. Log chỉ được chứa độ dài, hash, loại |
| N24 | Không gửi telemetry, không gọi ra ngoài ngoài peer trong config |

**Giới hạn đã biết, ghi rõ chứ không che:** Linux/Wayland không có cơ chế đánh dấu nhạy cảm tương đương macOS. Trên Linux, password copy từ password manager **sẽ** vào lịch sử. Giảm nhẹ bằng cách xoá tay ([US-B5](USER-STORIES.md#us-b5--xoá-item)).

**Nếu bỏ Tailscale** thì N18 mất chỗ dựa và bắt buộc phải thêm mã hoá tầng app. Đừng đổi transport mà quên điều này.

## 5. Hành vi khi lỗi

Im lặng là lỗi tệ nhất của app sync — người dùng tưởng đã sync rồi mới phát hiện không có. Mọi trường hợp dưới đây phải **nhìn thấy được**.

| Tình huống | App phải làm gì |
|---|---|
| Peer offline | Tray icon đổi trạng thái trong N3. Lịch sử local vẫn chạy bình thường |
| Tailscale down | Thông báo nói rõ "không kết nối được Tailscale", không phải "sync failed" chung chung |
| Peer trở lại | Tự reconnect trong N4, không cần thao tác tay |
| Không đọc được clipboard (Wayland thiếu protocol) | Báo lỗi rõ **lúc khởi động**, không phải im lặng rồi không bao giờ sync |
| Item vượt giới hạn | Vào lịch sử local, đánh dấu không sync |
| DB lỗi / corrupt | **Không tự xoá.** Báo lỗi, giữ nguyên file, để người dùng quyết định |
| Nhận payload không parse được | Bỏ qua item đó + log, **không** crash daemon |
| Phím tắt bị app khác chiếm | Báo lúc khởi động, app vẫn chạy (chỉ mất phím tắt) |
| Config sai cú pháp | Báo rõ dòng lỗi, **không** ghi đè file người dùng bằng bản mặc định |
| Peer lạ kết nối tới | Từ chối, log lại. Không cần thông báo cho người dùng |

## 6. Khả năng vận hành

| ID | Yêu cầu |
|---|---|
| N25 | Chạy nền 7 ngày liền không cần restart tay ([PRD § Thước đo](PRD.md#6-thước-đo-thành-công)) |
| N26 | Crash thì tự restart (launchd / systemd) |
| N27 | Log ra file, tự xoay vòng, mặc định giữ 7 ngày |
| N28 | Bật debug log được mà không phải build lại |
| N29 | Xem được trạng thái bằng CLI, không cần mở GUI |

## 7. Tương thích

| ID | Yêu cầu |
|---|---|
| N30 | macOS: bản hiện tại và bản trước đó, cả Apple Silicon và Intel |
| N31 | Linux: NixOS, `x86_64-linux`. Wayland là mục tiêu chính, X11 là fallback |
| N32 | Hai máy **phải cùng version** app. Version lệch → cảnh báo rõ, không cố đoán protocol |
