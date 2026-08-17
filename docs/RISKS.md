# Risk Register — x2clip

> Nguồn chính cho: rủi ro và cách giảm nhẹ. Xem lại trước mỗi phase ([ROADMAP.md](ROADMAP.md)).

Thang: **Xác suất** và **Tác động** = Cao / TB / Thấp. Chỉ có một người làm nên không có cột owner.

---

## Rủi ro đang mở

### R1 · Wayland compositor không cho đọc clipboard
**Xác suất:** Cao · **Tác động:** Cao — phá vỡ cả kế hoạch

Đọc clipboard trên Wayland cần protocol `wlr-data-control` hoặc `ext-data-control`. Hỗ trợ phụ thuộc **compositor**, không phụ thuộc distro — biết là NixOS thì chưa biết gì. GNOME/Mutter là chỗ hay thiếu.

**Dấu hiệu:** đọc clipboard trả về rỗng hoặc lỗi permission trên Linux, dù cùng code chạy ổn trên macOS.

**Giảm nhẹ:** [ROADMAP Phase 0.1–0.2](ROADMAP.md#phase-0--spike--⬜-chặn-bởi-prd-q1) kiểm trước tiên, bằng tay, trước khi viết dòng code nào của app.

**Kế hoạch B:** gọi `wl-copy`/`wl-paste` ngoài; nếu vẫn không được thì chạy X11. Cả hai đều làm [ADR-0003](ADR/0003-clipboard-arboard-polling.md) phải sửa.

---

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
**Xác suất:** Thấp · **Tác động:** Cao nếu xảy ra

Tailscale đổi giá, đổi điều khoản, hoặc coordination server không truy cập được → sync đứng.

**Giảm nhẹ:** `peer.rs` chỉ nói WebSocket tới một hostname — nó **không** biết gì về Tailscale. Đổi sang WireGuard tự dựng hoặc Headscale là đổi cấu hình mạng, không đổi code app.

**Cảnh báo:** nếu đổi sang transport **không** mã hoá thì [N18](NFR.md#4-bảo-mật) mất chỗ dựa và **bắt buộc** phải thêm mã hoá tầng app — xem [ADR-0005](ADR/0005-no-app-layer-crypto.md).

---

### R8 · Mất item khi hai máy lệch giờ online
**Xác suất:** Cao (theo thiết kế) · **Tác động:** Thấp

Tailscale là mạng, không phải kho lưu trữ. Máy kia offline thì item không tới.

Đây là **đánh đổi có ý thức**, không phải bug — [PRD § Ngoài scope](PRD.md#4-ngoài-scope).

**Kích hoạt xem lại:** nếu dùng thật mà thấy mất item thường xuyên → tính relay ở [ROADMAP § Sau v1](ROADMAP.md#sau-v1).

**Còn phải chốt:** hành vi khi nối lại — gửi bù item mới nhất, hay không gửi gì? Xem [T11](TEST-PLAN.md#t11--reconnect-không-mất-item), quyết ở Phase 2.

---

### R9 · Scope trôi sang làm lại CleanClip
**Xác suất:** TB · **Tác động:** TB

CleanClip/Paste có nhiều tính năng hấp dẫn (sync lịch sử đầy đủ, rich text, mobile, tổ chức theo pinboard). Bám theo feature list của họ là bỏ luôn cơ hội có app dùng được.

**Giảm nhẹ:** [PRD § Ngoài scope](PRD.md#4-ngoài-scope) là danh sách phải đọc lại mỗi khi định thêm tính năng. Nguyên tắc: xong Phase 2 là app đã có giá trị thật; mọi thứ sau đó là tiện nghi.

---

## Rủi ro đã đóng

Chưa có — chưa bắt đầu code.

Khi đóng một rủi ro thì chuyển xuống đây kèm lý do, đừng xoá. Biết vì sao một lo lắng đã hết cũng đáng giá.
