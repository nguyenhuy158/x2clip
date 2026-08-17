# ADR-0003 · Một crate clipboard chung + poll cả hai OS

**Trạng thái:** Accepted — **chờ [Phase 0](../ROADMAP.md#phase-0--spike--⬜-chặn-bởi-prd-q1) xác nhận**
**Ngày:** 2026-08-17

## Bối cảnh

Cần đọc và ghi clipboard (text + ảnh) trên macOS và Linux. Đây là chỗ hai hệ điều hành **bất đối xứng nhất**:

| | macOS | Linux |
|---|---|---|
| API | NSPasteboard | Wayland `data-control`, hoặc X11 selection |
| Thông báo khi đổi | **Không có.** Phải poll `changeCount` | Có event thật (`wl-paste --watch`, X11 selection-owner notify) |
| Ảnh | Thường là TIFF | Thường là PNG |
| Đánh dấu nhạy cảm | Có (`org.nspasteboard.ConcealedType`) | Không có |
| Có sẵn hay không | Luôn có | **Phụ thuộc compositor** |

## Quyết định

Hai phần:

1. **Một crate clipboard đa nền tảng** (`arboard`) làm backend cho `clip.rs`, thay vì tự viết hai adapter. Crate này đã cover NSPasteboard, X11 và Wayland, cả text lẫn ảnh.

2. **Poll cả hai OS** ở 250ms ([N13](../NFR.md#3-giới-hạn)), dù Linux có event thật.

`clip` là một **trait**, không phải struct — để có impl fake cho test ([TEST-PLAN § Nguyên tắc](../TEST-PLAN.md#1-nguyên-tắc)).

## Vì sao poll cả hai bên, dù Linux có event

macOS **không có** lựa chọn nào khác ngoài poll. Vậy câu hỏi thật là: Linux có nên dùng đường riêng?

**Không, chưa nên.** Một code path duy nhất và đối xứng nghĩa là không có class bug chỉ xảy ra ở một OS — và loại bug đó là loại tốn nhất, vì nó chỉ hiện ra trên máy bạn không đang ngồi. Chi phí poll nằm trong [N9](../NFR.md#2-tài-nguyên) và interval đã cấu hình được.

Nếu thiết kế một interface `watch()` rồi bên macOS thì giả lập bằng poll bên dưới, ta được một abstraction **nói dối** về cái nó làm. Thà thừa nhận "poll hoặc watch, cả hai đều phát ra `(hash, payload)`" cho đúng.

**Kích hoạt xem lại:** nếu trễ 250ms thấy rõ khi dùng thật, thêm watch path riêng cho Linux. Khi đó echo guard vẫn **dùng chung** — đừng nhân đôi nó, đó là chỗ dễ sai nhất ([R2](../RISKS.md#r2--vòng-lặp-echo)).

## Phương án đã loại

### Tự viết hai adapter native (objc2 cho macOS, wayland-client cho Linux)
**Loại vì:** viết lại đúng thứ crate đã làm. Sẽ là hàng trăm dòng unsafe FFI mà không có gì mới.

**Ưu điểm bị bỏ:** kiểm soát hoàn toàn, đọc được `org.nspasteboard.ConcealedType` mà không chờ crate hỗ trợ, dùng được event thật trên Wayland.

**Ghi chú:** [US-C4](../USER-STORIES.md#us-c4--không-lưu-password) (bỏ qua password) có thể cần một ít code native riêng cho macOS **bên cạnh** crate, nếu crate không expose UTI. Đây là ngoại lệ có giới hạn, không phải lý do viết lại cả adapter.

### Gọi tool ngoài (`pbcopy`/`pbpaste`, `wl-copy`/`wl-paste`, `xclip`)
**Loại vì:** spawn process mỗi 250ms là không đạt [N9](../NFR.md#2-tài-nguyên). Với ảnh còn phải qua file tạm.

**Nhưng đây là kế hoạch B của [R1](../RISKS.md#r1--wayland-compositor-không-cho-đọc-clipboard).** Nếu crate không chạy được trên compositor thực tế thì chuyển sang cách này — kèm nới poll interval để bù chi phí spawn.

### Chỉ hỗ trợ X11 trên Linux
**Loại vì:** X11 clipboard đơn giản và ổn định hơn hẳn, nhưng bắt người dùng rời Wayland là bắt họ đổi desktop vì một app clipboard. Sai chiều.

**Là fallback cuối** nếu compositor không hỗ trợ đọc clipboard theo bất kỳ cách nào.

## Hệ quả

### Được
- Một `clip.rs`, không phải hai adapter — chỗ khác nhau giữa hai OS giữ được ở mức tối thiểu ([ARCHITECTURE § 9](../ARCHITECTURE.md#9-chỗ-khác-nhau-giữa-hai-os))
- Ảnh cũng do crate lo, không tự xử lý TIFF/PNG ở tầng platform
- Một code path poll → không có bug chỉ-xảy-ra-ở-một-OS

### Mất
- **Không dùng được event thật trên Linux** → trễ tối đa bằng poll interval
- Poll tốn pin trên laptop ([R6](../RISKS.md#r6--poll-250ms-tốn-pin-trên-laptop))
- **Phụ thuộc crate cho phần rủi ro nhất** (Wayland). Crate không hỗ trợ compositor thì hết đường đi qua nó
- Đánh dấu nhạy cảm có thể cần code native riêng bên cạnh

### Điều kiện bắt buộc trước khi tin quyết định này
[ROADMAP 0.1–0.2](../ROADMAP.md#phase-0--spike--⬜-chặn-bởi-prd-q1): xác nhận compositor thật trên máy NixOS có hỗ trợ đọc clipboard, và crate đọc + ghi được **cả text lẫn ảnh** trên đó.

Fail → ADR này phải sửa **trước** khi viết `clip.rs`. Đó là toàn bộ lý do Phase 0 tồn tại.
