# ADR-0004 · SQLite, và lịch sử là local không sync

**Trạng thái:** Accepted
**Ngày:** 2026-08-17

Hai quyết định gộp trong một ADR vì chúng dính nhau: chọn kho lưu trữ và chọn **không** đồng bộ kho đó.

## Bối cảnh

Lịch sử clipboard cần: tồn tại qua restart, tìm kiếm theo từ khoá, ghim, tự xoá bớt theo hạn mức, và giữ được cả text lẫn blob ảnh. Quy mô: [1000 item / 30 ngày](../NFR.md#3-giới-hạn), một người dùng, một process ghi.

Câu hỏi thứ hai: lịch sử của máy A và máy B có nên là **cùng một** lịch sử?

## Quyết định

### 4a · SQLite, một file, quyền `0600`
Schema ở [ARCHITECTURE § Lưu trữ](../ARCHITECTURE.md#7-lưu-trữ). Index trên `hash` và `updated_at` để đạt [N5](../NFR.md#1-ngưỡng-chấp-nhận).

### 4b · Lịch sử là local mỗi máy, **không** sync
v1 chỉ đồng bộ **clipboard hiện tại**. Copy ở A thì clipboard B đổi theo — nhưng lịch sử B chỉ chứa những gì đã đi qua clipboard B.

Hệ quả trực tiếp người dùng thấy: xoá một item ở máy A **không** xoá nó ở máy B ([US-B5](../USER-STORIES.md#us-b5--xoá-item) phải nói rõ điều này trong UI).

## Phương án đã loại — kho lưu trữ

### File JSON / JSONL
**Loại vì:** tìm kiếm phải load hết vào RAM; blob ảnh trong JSON là base64 (phình 33%); ghi đè cả file mỗi lần thêm item là mất dữ liệu nếu crash giữa lúc ghi. SQLite đã cho index, transaction và blob mà không thêm dependency đáng kể.

### Chỉ giữ trong RAM
**Loại vì:** [US-B1](../USER-STORIES.md#us-b1--lịch-sử-được-lưu-lại) yêu cầu sống qua restart.

### Ảnh ra file riêng, DB chỉ giữ đường dẫn
**Loại vì:** ở mức 1000 item với giới hạn 5MB thì blob trong SQLite hoàn toàn ổn, và một file duy nhất thì backup/xoá/di chuyển đều đơn giản. Hai nguồn dữ liệu là hai cơ hội lệch nhau (DB có row nhưng file mất).

**Kích hoạt xem lại:** nếu DB vượt [N12](../NFR.md#2-tài-nguyên) (500MB) thường xuyên.

### Postgres / server DB
**Loại vì:** một người dùng, một process ghi. Chạy một service DB cho việc này đi ngược [PRD G4](../PRD.md#3-mục-tiêu).

## Phương án đã loại — sync lịch sử

### Sync toàn bộ lịch sử giữa hai máy (như CleanClip/Paste làm)
Đây chính là điều hai app tham chiếu làm, và là điều dễ giả định là "đương nhiên phải có".

**Loại vì:** nó là một product thứ hai, không phải một tính năng thêm. Cần trả lời hết những câu này trước khi viết dòng đầu:
- Xoá ở A thì B có xoá theo? Nếu có, cần tombstone — và tombstone phải sống lâu hơn thời gian offline dài nhất.
- Ghim ở A thì B có ghim? Nếu có, `pinned` là state cần merge.
- Hai máy cùng offline, cùng có item mới, cùng lên mạng → trộn theo timestamp? Đồng hồ hai máy lệch thì sao?
- Máy mới cài lần đầu → đồng bộ ngược 1000 item cũ, hay bắt đầu từ rỗng?

Đó là CRDT hoặc một quy tắc merge được viết ra cẩn thận. Trong khi **giá trị chính** của app — copy ở đây paste ở kia — không cần bất kỳ điều nào ở trên.

**Kích hoạt xem lại:** dùng thật rồi thấy thường xuyên cần **item cũ** của máy kia (không phải item vừa copy). Ghi ở [ROADMAP § Sau v1](../ROADMAP.md#sau-v1).

### Sync lịch sử một chiều (A là nguồn chân lý)
**Loại vì:** hai máy đều là máy làm việc chính, không có cái nào phụ. Chọn một cái làm nguồn chân lý là sai với cách dùng thật.

## Hệ quả

### Được
- Một file, backup bằng cách copy, xoá bằng cách xoá
- Tìm kiếm và prune là SQL, không phải code tay
- Transaction: crash giữa lúc ghi không làm hỏng lịch sử
- **Không có bài toán conflict resolution nào** — đây là phần lợi lớn nhất của 4b
- `store` không biết gì về `peer` ([ARCHITECTURE § Thành phần](../ARCHITECTURE.md#2-thành-phần)), nên test được độc lập

### Mất
- Hai lịch sử khác nhau ở hai máy. Item copy lúc máy kia tắt sẽ **chỉ** có ở một bên
- Ghim ở A không ghim ở B — phải ghim hai lần nếu muốn cả hai
- Xoá phải làm hai lần nếu muốn xoá thật ở cả hai máy. **Đây là điểm bảo mật đáng lưu ý**: xoá nội dung nhạy cảm ở A không làm nó mất ở B

### Ràng buộc bắt buộc giữ
- **Prune luôn loại trừ `pinned = 1`**, bất kể hạn mức ([N14](../NFR.md#3-giới-hạn)). Có [T4](../TEST-PLAN.md#t4--prune-không-đụng-item-đã-ghim) canh.
- **DB lỗi thì không tự xoá** — báo lỗi, giữ file, để người dùng quyết ([NFR § Hành vi khi lỗi](../NFR.md#5-hành-vi-khi-lỗi)). Tự "sửa" bằng cách xoá là mất dữ liệu.
- Ảnh lưu **blob**, không base64. Base64 chỉ dùng lúc truyền qua frame.

## Xem lại 2026-08-17 — sync lịch sử

**Kích hoạt:** chủ project muốn lịch sử dùng chung giữa hai máy, và đề xuất chuyển kho lưu trữ sang **Cloudflare D1**.

### Giữ 4a. Bỏ 4b — nhưng sau v1, và không dùng D1.

**D1 bị loại làm kho chính:**

| Lý do | Chi tiết |
|---|---|
| Không offline | D1 truy cập qua HTTP tới edge. Mất mạng là mất cả lịch sử lẫn khả năng đọc item cũ. [US-B1](../USER-STORIES.md#us-b1--lịch-sử-được-lưu-lại) không đòi mạng. |
| Trễ | Mỗi lần copy = 1 round trip Internet (50–200ms) thay vì ghi local (micro giây). Phá [N9](../NFR.md#2-tài-nguyên) và cảm giác dùng. |
| Cần backend | Desktop app không nối trực tiếp D1; phải có Worker + API token. Đi ngược [PRD G4](../PRD.md#3-mục-tiêu) và chính lý do chọn [ADR-0001](0001-transport-tailscale.md). |
| Phá giả định bảo mật | [ADR-0005](0005-no-app-layer-crypto.md) bỏ mã hoá tầng app **vì** dữ liệu chỉ đi giữa hai máy qua WireGuard. Đưa clipboard (password, token) lên cloud thì phải mã hoá tầng app trước → ADR-0005 phải viết lại. |
| Ảnh | D1 không dành cho blob lớn; ảnh phải sang R2 → thêm service thứ hai. |

**Thay bằng:** SQLite local vẫn là nguồn chân lý; sync lịch sử **peer-to-peer qua kênh Tailscale đã có** — mở rộng message type của `peer.rs` từ Phase 2, không thêm hạ tầng.

### Quy tắc merge — trả lời các câu hỏi ADR này từng nêu

| Câu hỏi | Quy tắc |
|---|---|
| Khoá định danh item | `hash` nội dung. Hai máy cùng nội dung → cùng row, dedupe tự nhiên. |
| Ghim ở A có ghim ở B | Có. `pinned` merge bằng **OR**. |
| `updated_at` lệch đồng hồ | Lấy **max**. Lệch vài giây trên lịch sử clipboard không gây hại thật; không cần Lamport clock cho 2 máy. |
| Xoá ở A có xoá ở B | Có. Bảng `tombstones(hash, deleted_at)`, giữ **30 ngày** — bằng cửa sổ lịch sử ở [N14](../NFR.md#3-giới-hạn), nên tombstone không sống ngắn hơn thời gian một item còn tồn tại. |
| Máy mới cài lần đầu | Bắt đầu **rỗng**, không backfill lịch sử cũ. Backfill 1000 item là bài toán riêng, chưa cần. |

Đây là merge rule viết tay, không phải CRDT — đủ vì chỉ 2 máy, một người dùng, dữ liệu append-mostly.

### Điều kiện D1 / Durable Object mới thực sự cần

**Store-and-forward:** máy A copy khi máy B đang **tắt**, và B phải nhận được lúc bật lên. P2P không làm được (cần cả hai online cùng lúc). Nếu tình huống đó xảy ra thường xuyên khi dùng thật → mở ADR mới, và khi đó **bắt buộc** mã hoá tầng app trước khi đẩy bất kỳ byte nào lên Cloudflare.

Ghi ở [ROADMAP § Sau v1](../ROADMAP.md#sau-v1). **Không chặn v1** — v1 vẫn chỉ sync clipboard hiện tại.
