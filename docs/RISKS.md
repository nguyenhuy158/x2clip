# Risk Register — x2clip

> Nguồn chính cho: rủi ro và cách giảm nhẹ. Xem lại trước mỗi phase ([ROADMAP.md](ROADMAP.md)).

Thang: **Xác suất** và **Tác động** = Cao / TB / Thấp. Chỉ có một người làm nên không có cột owner.

---

## Rủi ro đang mở

### R2 · Vòng lặp echo
**Xác suất:** Cao nếu quên · **Tác động:** Cao — app không dùng được

Hai đầu vừa theo dõi vừa ghi clipboard. Đây là lỗi kinh điển của mọi project clipboard sync.

**Dấu hiệu:** CPU tăng vọt, lịch sử đầy entry trùng, máy chậm ngay sau khi copy một lần.

**Giảm nhẹ:** `last_written_hash` với ràng buộc thứ tự (set cờ **trước** khi ghi clipboard) — [ARCHITECTURE § Echo guard](ARCHITECTURE.md#echo-guard--chỗ-dễ-sai-nhất). Có [T2, T3](TEST-PLAN.md#3-test-bắt-buộc-phải-có) là gate chặn release.

**Không được** coi đây là bug tune sau. Đã có cơ chế ngay từ Phase 2.

---

### R3 · Nix build cho app GUI mất công hơn dự kiến
**Xác suất:** TB · **Tác động:** TB — chậm Phase 5, không chặn việc dùng app

Rust + node deps + webkitgtk trong một derivation là chỗ hay kẹt.

**Dấu hiệu:** `nix build` fail vì thiếu system lib, hoặc vì node_modules cần mạng lúc build.

**Giảm nhẹ:** để ở Phase 5, sau khi app đã chạy được. Fallback: `nix develop` + `cargo build` tay, đóng gói sạch sau.

---

### R4 · Định dạng ảnh lệch giữa macOS và Linux
**Xác suất:** TB · **Tác động:** TB

macOS thường đưa ảnh lên clipboard dưới dạng TIFF; Linux mong PNG. Chuyển qua lại có thể mất alpha hoặc đổi kích thước.

**Dấu hiệu:** paste ra ảnh đen, ảnh mất trong suốt, hoặc kích thước lệch.

**Giảm nhẹ:** chuẩn hoá **một chiều duy nhất** về PNG trước khi gửi ([ARCHITECTURE § Protocol](ARCHITECTURE.md#6-protocol)). [T6](TEST-PLAN.md#t6--round-trip-nội-dung) assert cùng kích thước pixel.

---

### R5 · Password vào lịch sử trên Linux
**Xác suất:** Cao · **Tác động:** TB

macOS có `org.nspasteboard.ConcealedType` để đánh dấu nội dung nhạy cảm. Wayland/Linux **không có** cơ chế tương đương.

**Giảm nhẹ:** ghi rõ giới hạn này trong tài liệu ([NFR § Bảo mật](NFR.md#4-bảo-mật)) chứ không giả vờ đã xử lý. Người dùng xoá tay ([US-B5](USER-STORIES.md#us-b5--xoá-item)).

**Không giảm nhẹ bằng heuristic đoán password** (kiểu "chuỗi ngắn không có khoảng trắng") — đoán sai theo cả hai chiều đều tệ hơn là nói thẳng giới hạn.

---

### R6 · Poll 250ms tốn pin trên laptop
**Xác suất:** TB · **Tác động:** Thấp

**Dấu hiệu:** macOS liệt app vào "Apps Using Significant Energy" ([N11](NFR.md#2-tài-nguyên)).

**Giảm nhẹ:** poll chỉ so sánh dấu hiệu rẻ, **không** decode payload khi chưa phát hiện đổi ([N9](NFR.md#2-tài-nguyên)). Nếu vẫn tốn thì nới interval trước — nó đã cấu hình được ([N13](NFR.md#3-giới-hạn)).

---

### R7 · Phụ thuộc Tailscale
**Xác suất:** Thấp · **Tác động:** **Thấp** — hạ từ Cao sau [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md)

Tailscale giờ chỉ chở tiếng chuông, không chở nội dung. Tailscale đứng → sync **vẫn chạy** qua poll R2, chỉ chậm hơn ([N1b](NFR.md#1-ngưỡng-chấp-nhận) thay cho [N1](NFR.md#1-ngưỡng-chấp-nhận)).

**Giảm nhẹ:** không cần làm gì thêm. `notify.rs` là thành phần tuỳ chọn, bỏ hẳn cũng chạy ([ROADMAP 2b](ROADMAP.md#phase-2b--kênh-chuông-tailscale--⬜-tuỳ-chọn)).

---

### R10 · Phụ thuộc Cloudflare R2
**Xác suất:** Thấp · **Tác động:** Cao — đây là **đường duy nhất** chở nội dung

R2 down, đổi giá, khoá account, hoặc access key hết hạn → không sync được. Rủi ro này **thay chỗ** R7: đổi một dependency ngoài để lấy store-and-forward.

**Dấu hiệu:** PUT/LIST lỗi liên tục. Phải phân biệt được "sai access key" với "mất mạng" ([NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi)) — báo sai nguyên nhân là gỡ cả buổi.

**Giảm nhẹ:**
- Lịch sử local **không** phụ thuộc R2. R2 chết thì mất sync, **không** mất lịch sử ([ADR-0004](ADR/0004-storage-sqlite-local-history.md) 4a nguyên vẹn).
- `mailbox.rs` dùng **S3 API chuẩn**, không dùng thứ riêng của Cloudflare → đổi sang MinIO tự dựng / Backblaze B2 / S3 là đổi endpoint trong config, không sửa code. Đây là lý do chọn S3 API thay vì Worker.
- Item chưa PUT được nằm trong hàng chờ, có mạng lại thì gửi ([N4](NFR.md#1-ngưỡng-chấp-nhận)).

---

### R11 · Mất khoá mã hoá
**Xác suất:** Thấp · **Tác động:** Cao — không có recovery

v1 có **một** khoá, sinh tay, copy tay. Mất là mất: object trong hộp thư thành rác, và không có rotation để cứu ([ADR-0005 § Mô hình đe doạ](ADR/0005-no-app-layer-crypto.md#mô-hình-đe-doạ--cái-gì-được-bảo-vệ-cái-gì-không)).

**Giảm nhẹ:** khoá là thứ **duy nhất** phải backup tay (password manager). Thiệt hại có giới hạn: hộp thư tự hết hạn sau 30 ngày ([N13c](NFR.md#3-giới-hạn)), không tích rác vĩnh viễn, và lịch sử local vẫn đọc được vì nó **không** mã hoá at-rest.

**Kích hoạt xem lại:** thêm máy thứ 3, hoặc nghi khoá bị lộ → cần rotation, [ROADMAP § Sau v1](ROADMAP.md#sau-v1).

---

### R12 · Chi phí R2 vượt dự kiến
**Xác suất:** Thấp · **Tác động:** Thấp

Poll 30s = một lệnh LIST mỗi 30s mỗi máy, chạy 24/7. Con số free tier ghi trong tài liệu này là **áng, chưa tra cứu** — đừng tin nó.

**Giảm nhẹ:** [N13b](NFR.md#3-giới-hạn) cấu hình được, nới interval là xong. Có kênh chuông thì nới được mạnh mà vẫn nhạy lúc cả hai máy bật. **Việc phải làm ở Phase 2:** tra bảng giá thật trước khi chốt interval.

---

### R9 · Scope trôi sang làm lại CleanClip
**Xác suất:** TB · **Tác động:** TB

CleanClip/Paste có nhiều tính năng hấp dẫn (sync lịch sử đầy đủ, rich text, mobile, tổ chức theo pinboard). Bám theo feature list của họ là bỏ luôn cơ hội có app dùng được.

**Giảm nhẹ:** [PRD § Ngoài scope](PRD.md#4-ngoài-scope) là danh sách phải đọc lại mỗi khi định thêm tính năng. Nguyên tắc: xong Phase 2 là app đã có giá trị thật; mọi thứ sau đó là tiện nghi.

---

## Rủi ro đã đóng

### ~~R8~~ · Mất item khi hai máy lệch giờ online — đóng 2026-08-17
Từng là **Cao (theo thiết kế) / Thấp**, và từng được ghi là "đánh đổi có ý thức, không phải bug": Tailscale là mạng, không phải kho, nên máy kia offline thì item không tới.

**Đóng vì rủi ro đã xảy ra, không phải vì hết lo.** Hoá ra nó không phải ca biên mà là **ca dùng chính**: mac ở công ty, nixos ở nhà, hai máy ít khi mở cùng lúc. Với hình dạng đó thì "chỉ sync khi cả hai online" gần như không bao giờ sync.

**Cách giải:** [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md) — hộp thư R2 giữ item cho tới khi máy kia bật lên. Điều kiện xem lại ở [ADR-0001](ADR/0001-transport-tailscale.md#tự-dựng-relay-server-cloudflare-worker--durable-object) ("R8 xảy ra thường xuyên thật") đã kích hoạt đúng như dự phòng.

**Bù lại:** thêm [R10](#r10--phụ-thuộc-cloudflare-r2) (phụ thuộc R2) và [R11](#r11--mất-khoá-mã-hoá) (mất khoá). Đổi một rủi ro Cao-xác-suất lấy hai rủi ro Thấp-xác-suất.

**Bài học đáng giữ:** chỗ sai không phải quyết định transport, mà là **giả định chưa kiểm về cách dùng thật**. "Hai máy có thường cùng bật không" là câu đáng hỏi ở Phase 0, cùng chỗ với compositor và Tailscale ping.

---

### ~~R1~~ · Wayland compositor không cho đọc clipboard — đóng 2026-08-17
Từng là **Cao/Cao**, rủi ro số một của cả kế hoạch: đọc clipboard trên Wayland cần `wlr-data-control` hoặc `ext-data-control`, mà hỗ trợ thì tuỳ compositor (GNOME/Mutter hay thiếu).

**Đóng vì:** máy nixos chạy **X11** (Xorg 21.1.23), không có DE — không có compositor Wayland nào trong hình. [Phase 0.1–0.2](ROADMAP.md#kết-quả-02-2026-08-17) xác nhận `arboard` dùng backend X11 đọc/ghi được cả text lẫn ảnh. Kế hoạch B (`wl-copy`/`wl-paste`) không cần dùng.

**Mở lại khi:** chuyển máy nixos sang Wayland, hoặc thêm máy Linux thứ hai chạy Wayland. Lúc đó [ADR-0003](ADR/0003-clipboard-arboard-polling.md) phải xem lại trước khi sửa code.

---

Khi đóng một rủi ro thì chuyển xuống đây kèm lý do, đừng xoá. Biết vì sao một lo lắng đã hết cũng đáng giá.
