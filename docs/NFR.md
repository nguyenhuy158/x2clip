# Non-Functional Requirements — x2clip

> Nguồn chính cho: mọi con số ngưỡng, giới hạn, hành vi khi lỗi, bảo mật. File khác cần số thì link về đây, không copy số sang.

---

## 1. Ngưỡng chấp nhận

Đây là mức **chấp nhận được**, không phải mục tiêu lý tưởng. Không đạt = fail, phải sửa trước khi đóng phase.

| ID | Tiêu chí | Ngưỡng | Không đạt nghĩa là |
|---|---|---|---|
| N1 | Trễ sync text, **cả hai máy đang bật và có kênh chuông** ([ROADMAP 2b](ROADMAP.md#phase-2b--kênh-chuông-tailscale--)) | < 1s | Poll interval hoặc đường mạng có vấn đề |
| N1b | Trễ sync text, **không có kênh chuông** | ≤ 1 chu kỳ poll + 1s ([N13b](#3-giới-hạn)) | Poll không chạy, hoặc PUT thất bại lặng lẽ |
| N1c | Trễ khi máy nhận **vừa bật lên / vừa có mạng** | < 10s | Không poll lúc khởi động và lúc wake. Đây là ca dùng **chính**, không phải ca biên |
| N2 | Trễ sync ảnh 2MB | < 3s (cùng điều kiện N1) | Encode hoặc transport cần tối ưu |
| N3 | Phát hiện không tới được R2 | < 10s | Người dùng tưởng đã sync |
| N4 | Có mạng lại → gửi xong hàng chờ PUT | < 15s | Backoff quá dài |
| N5 | Tìm kiếm ở mức lịch sử tối đa | < 100ms | Thiếu index trong DB |
| N6 | Mở cửa sổ bằng phím tắt | < 200ms tới lúc gõ được | Đang khởi tạo lười lúc mở — phải làm sẵn từ trước |
| N7 | Echo loop | **0 message dư.** Copy 1 lần = đúng 1 message mỗi chiều | Bug chặn release, không phải bug tune sau |
| N8 | Mất item | 0, **kể cả khi máy kia đang tắt lúc copy** | Hộp thư không giữ được item — chính lý do [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md) tồn tại |

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
| N13 | Poll clipboard | 250ms | Có |
| N13b | Poll hộp thư R2 | 30s | Có. **Tra bảng giá R2 trước khi hạ số này** — mỗi lần LIST là một request có phí ([ADR-0006 § Hệ quả](ADR/0006-r2-mailbox-store-and-forward.md)) |
| N13c | Giữ object trong hộp thư | 30 ngày (lifecycle rule của R2) | Bằng cửa sổ N14, để object không chết trước khi máy kia bật lên |
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
| N18 | Mọi byte đi qua mạng phải được mã hoá. **Nội dung clipboard phải được mã hoá ở tầng app trước khi PUT lên R2** — TLS một mình không đủ, vì dữ liệu *nằm lại* trên đĩa của Cloudflare. Xem [ADR-0005 § Xem lại](ADR/0005-no-app-layer-crypto.md#xem-lại-2026-08-17--mã-hoá-tầng-app-thành-bắt-buộc) |
| N18b | Dùng thư viện AEAD đã kiểm chứng (`age` / libsodium). Không tự chọn cipher, tự sinh nonce, tự ghép primitive |
| N18c | Khoá mã hoá và access key R2 giữ local, quyền `0600` (hoặc Keychain). Không vào repo, không lên R2, không vào log |
| N18d | Giải mã fail → log + **giữ** object, không xoá, không ghi vào clipboard/store |
| N18e | Object key **không** chứa hash plaintext — dùng ULID random |
| N18f | Khoá mã hoá **dẫn xuất trên máy** bằng Argon2id từ passphrase người dùng gõ. Server **không** cấp khoá, không giữ khoá ([ADR-0007 § 7b](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md)) |
| N18g | Passphrase không rời khỏi máy — không gửi đi dạng gốc, dạng băm, hay dạng nào khác. Không ghi ra đĩa, không vào log |
| N18h | Token đăng nhập và credential R2 tạm thời: Keychain / file `0600`, như N18c. Credential phải **có hạn** và **chỉ đủ quyền cho prefix của máy này** |
| N18i | Sai passphrase phải báo khác lỗi đăng nhập. "Không giải mã được" và "không xác thực được" là hai vấn đề, sửa bằng hai cách |
| N19 | Socket lắng nghe của kênh chuông **chỉ** bind vào địa chỉ Tailscale, không `0.0.0.0`. Không tìm được địa chỉ Tailscale → **từ chối listen**, không fallback |
| N20 | Chỉ nhận kết nối từ peer có trong config. Peer lạ → từ chối + log |
| N21 | DB lịch sử để quyền `0600`, trong thư mục dữ liệu của user |
| N22 | Nội dung đánh dấu nhạy cảm (macOS `org.nspasteboard.ConcealedType`) không lưu, không gửi |
| N23 | Không log nội dung clipboard. Log chỉ được chứa độ dài, hash, loại |
| N24 | Không gửi telemetry. Chỉ gọi ra **bốn** đích: bucket R2 trong config, peer trong config, endpoint `auth`, và identity provider — **lúc đăng nhập hoặc làm mới token, không phải liên tục** |
| N24c | Endpoint `auth` **không** được nhận nội dung clipboard, dù đã mã hoá. Nội dung chỉ đi qua R2 ([ADR-0007 § 7a](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md)) |
| N24b | **Rò rỉ metadata là đánh đổi có ý thức:** Cloudflare thấy số item, thời điểm, kích thước — không thấy nội dung. Ghi rõ ở [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md), không che |

**Giới hạn đã biết, ghi rõ chứ không che:** Linux/Wayland không có cơ chế đánh dấu nhạy cảm tương đương macOS. Trên Linux, password copy từ password manager **sẽ** vào lịch sử. Giảm nhẹ bằng cách xoá tay ([US-B5](USER-STORIES.md#us-b5--xoá-item)).

**Quên passphrase = mất hết.** Không có recovery, không có rotation. Đây là hệ quả trực tiếp của N18f: server không giữ khoá nên không có ai để hỏi xin. Passphrase là thứ duy nhất phải nhớ hoặc cất vào password manager.

## 5. Hành vi khi lỗi

Im lặng là lỗi tệ nhất của app sync — người dùng tưởng đã sync rồi mới phát hiện không có. Mọi trường hợp dưới đây phải **nhìn thấy được**.

| Tình huống | App phải làm gì |
|---|---|
| Máy kia đang tắt | **Không phải lỗi.** Item nằm trong hộp thư, tray hiện "đã gửi, chờ máy kia nhận" |
| Không tới được R2 | Tray đổi trạng thái trong N3, nói rõ "không kết nối được R2". Item vẫn vào lịch sử local, vào hàng chờ PUT |
| Access key R2 sai / hết hạn | Thử **làm mới bằng token đăng nhập trước**. Chỉ khi làm mới cũng fail mới báo, và phải phân biệt với "mất mạng". Sai key mà báo "mất mạng" là gỡ cả buổi |
| Token đăng nhập hết hạn | App **vẫn mở, lịch sử local vẫn tra được** (N33). Tray hiện "cần đăng nhập lại để sync". Không chặn người dùng lấy item cũ |
| Endpoint `auth` không tới được | Máy đã đăng nhập: **không phải lỗi**, chạy tiếp bằng credential còn hạn. Máy mới: báo rõ "không thêm được máy lúc này", không im lặng |
| Passphrase sai | "Passphrase không khớp — không giải mã được hộp thư" (N18i). **Không** xoá gì, không ghi đè hộp thư |
| Có mạng lại | Tự PUT hết hàng chờ trong N4 + poll ngay, không cần thao tác tay |
| Kênh chuông (Tailscale) down | **Không phải lỗi sync.** Ghi log, tray hiện "chậm hơn bình thường". Sync vẫn chạy qua poll |
| Giải mã fail | Log + giữ object (N18d). Không ghi clipboard, không xoá, không crash |
| Không đọc được clipboard (Wayland thiếu protocol) | Báo lỗi rõ **lúc khởi động**, không phải im lặng rồi không bao giờ sync |
| Item vượt giới hạn | Vào lịch sử local, đánh dấu không sync |
| DB lỗi / corrupt | **Không tự xoá.** Báo lỗi, giữ nguyên file, để người dùng quyết định |
| Nhận payload không parse được | Bỏ qua item đó + log, **không** crash daemon |
| DELETE object trên R2 fail | Bỏ qua — sổ `seen` chặn xử lý lại, lifecycle rule dọn nốt. **Không** được xử lý object hai lần rồi ghi đè clipboard hiện tại bằng item cũ |
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
| N33 | **Đăng nhập chỉ cần lúc thêm máy hoặc làm mới token.** Hết hạn, mất mạng, hay `auth` chết đều không được chặn: lịch sử local vẫn mở, tìm và dùng lại item vẫn chạy. Sync dừng, phần còn lại thì không |

## 7. Tương thích

| ID | Yêu cầu |
|---|---|
| N30 | macOS: bản hiện tại và bản trước đó, cả Apple Silicon và Intel |
| N31 | Linux: NixOS, `x86_64-linux`. Wayland là mục tiêu chính, X11 là fallback |
| N32 | Hai máy **phải cùng version** app. Version lệch → cảnh báo rõ, không cố đoán protocol |
