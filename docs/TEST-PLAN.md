# Test Plan — x2clip

> Nguồn chính cho: chiến lược test và quality gate. Ngưỡng số ở [NFR.md](NFR.md). Exit criteria từng phase ở [ROADMAP.md](ROADMAP.md).

---

## 1. Nguyên tắc

- **`core` là library thuần Rust** để test logic sync bằng `cargo test`, không cần dựng GUI. Đây là lý do tách `core` khỏi `app` ngay từ đầu ([ARCHITECTURE § Thành phần](ARCHITECTURE.md#2-thành-phần)).
- **Clipboard giả (fake) cho automated test.** Clipboard thật là tài nguyên toàn cục của máy — test song song sẽ đấu nhau, và CI không có clipboard. `clip` phải là trait có hai impl: thật và fake trong bộ nhớ.
- **Không test cái mà compiler đã đảm bảo.** Không viết test cho getter/setter.
- **Mỗi bug tìm được ngoài test → thêm một test.** Nhất là bug echo loop và bug mất item.
- **Test thủ công là hợp lệ** cho phần chạm hệ điều hành thật (clipboard thật, tray, phím tắt, Wayland). Nhưng phải có checklist ghi lại, không phải "tôi thử rồi thấy ok".

## 2. Tầng test

| Tầng | Test gì | Công cụ | Số lượng |
|---|---|---|---|
| Unit | hash, echo guard, prune, parse config, parse frame | `cargo test` | Nhiều nhất |
| Integration (trong process) | Hai instance `core` nói chuyện qua clipboard giả + WebSocket loopback | `cargo test` | Vài chục |
| Thủ công, một máy | Clipboard thật, ảnh thật, UI | Checklist | Mỗi phase |
| Thủ công, hai máy | Sync thật qua Tailscale | Checklist | Mỗi phase từ 2 |
| Chạy dài | Ổn định 7 ngày | Quan sát + log | Trước khi coi là xong |

Không có CI. Một người, hai máy — dựng CI cho việc này là chi phí thuần. `cargo test` chạy tay trước mỗi lần đóng phase là đủ. Điều này đổi lại: **kỷ luật chạy test là của bạn**, không có ai nhắc.

## 3. Test bắt buộc phải có

Đây là những test mà thiếu chúng thì app hỏng theo cách khó thấy. Không được bỏ vì "gọn hơn".

### T1 — Echo guard, một node
Ghi clipboard qua đường "nhận từ peer" → poll watcher → **assert không sinh item mới nào**.

Đây là test rẻ nhất và chặn được bug tệ nhất.

### T2 — Không có echo loop, hai node
Hai instance `core` trong một process, hai clipboard giả, hai store tạm, nối bằng channel.

- Copy một lần ở A
- **Assert đúng 1 message A→B và 0 message B→A**
- Assert số message là hữu hạn (đặt trần: nếu vượt 5 thì fail ngay, không chờ timeout)

Test này ứng với [N7](NFR.md#1-ngưỡng-chấp-nhận) và [US-A2](USER-STORIES.md#us-a2--không-có-vòng-lặp-echo). **Fail = chặn release.**

### T3 — Thứ tự set cờ trước khi ghi
Assert `last_written_hash` được set **trước** khi clipboard bị ghi. Đảo thứ tự là có race ([ARCHITECTURE § Echo guard](ARCHITECTURE.md#echo-guard--chỗ-dễ-sai-nhất)).

Cách làm: clipboard giả ghi lại trạng thái cờ tại thời điểm `set()` được gọi.

### T4 — Prune không đụng item đã ghim
Nhồi quá hạn mức [N14](NFR.md#3-giới-hạn), trong đó có item đã ghim, prune, assert item ghim **còn nguyên** và item cũ nhất chưa ghim đã mất.

### T5 — Copy trùng liên tiếp không sinh row mới
Copy cùng nội dung hai lần → một row, `updated_at` tăng, `created_at` không đổi.

### T6 — Round-trip nội dung
- Text: Unicode, emoji, xuống dòng, tab, chuỗi rất dài → nhận được **byte-for-byte** giống
- Ảnh: PNG → qua frame → decode lại ra ảnh cùng kích thước pixel
- Chuỗi rỗng → bị bỏ qua, không tạo item, không gửi

### T7 — Vượt giới hạn dung lượng
Item vượt [N15](NFR.md#3-giới-hạn)/[N16](NFR.md#3-giới-hạn) → vào store với `synced = 0`, **không bị cắt bớt**, không gửi đi, không crash.

### T8 — Frame lỗi không làm chết daemon
Nạp JSON sai cú pháp, `v` lạ, `kind` lạ, base64 hỏng, thiếu field → mỗi trường hợp: log + bỏ qua, daemon vẫn sống ([NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi)).

### T9 — Config
- Thiếu file → tạo mặc định, chạy được
- Sai cú pháp → báo lỗi có số dòng, **không ghi đè** file người dùng
- Peer list rỗng → chạy ở chế độ local-only, không crash

### T10 — Peer lạ bị từ chối
Kết nối từ địa chỉ không có trong config → từ chối + log ([N20](NFR.md#4-bảo-mật)).

### T11 — Reconnect không mất item
Ngắt kết nối, copy ở A trong lúc mất kết nối, nối lại → hành vi phải **khớp với tài liệu**.

> Cần chốt khi làm Phase 2: item copy trong lúc peer offline thì có gửi bù khi nối lại, hay bỏ? [NFR N8](NFR.md#1-ngưỡng-chấp-nhận) chỉ cam kết "0 mất mát khi **cả hai** online". Đề xuất: chỉ gửi item **mới nhất** khi nối lại, không gửi bù cả hàng đợi — đúng tinh thần "clipboard là một giá trị, không phải hàng đợi". Chốt xong thì sửa NFR cho rõ.

### T12 — Không log nội dung clipboard
Grep log sau khi chạy: không được xuất hiện nội dung, chỉ được có độ dài/hash/loại ([N23](NFR.md#4-bảo-mật)).

## 4. Checklist thủ công

Chạy trên **cả hai** OS. Ghi ngày + kết quả, không chỉ tick trong đầu.

### Một máy
- [ ] Copy text ở app khác → hiện trong lịch sử
- [ ] Copy ảnh (screenshot) → hiện trong lịch sử, có thumbnail
- [ ] Click item cũ → clipboard đổi, paste ra đúng
- [ ] Ghim → prune không xoá
- [ ] Xoá một item → mất luôn sau restart
- [ ] Xoá toàn bộ → có bước xác nhận
- [ ] Phím tắt mở cửa sổ khi app không focus, ô tìm kiếm sẵn con trỏ
- [ ] `Esc` đóng, clipboard không đổi
- [ ] Password copy từ password manager → **không** vào lịch sử (chỉ macOS; Linux đã ghi rõ là không hỗ trợ)

### Hai máy
- [ ] Copy text A → paste được ở B, dưới 1s
- [ ] Copy text B → paste được ở A
- [ ] Copy ảnh 2MB A → paste ở B, dưới 3s
- [ ] Copy liên tục 10 lần nhanh → không rác, không loop, thứ tự đúng
- [ ] Tắt app ở B → tray A đổi trạng thái trong 10s
- [ ] Bật lại B → tự nối lại trong 15s, không cần thao tác
- [ ] `tailscale down` → thông báo nói rõ Tailscale
- [ ] Tạm dừng ở A → copy không rời máy, lịch sử local vẫn ghi

### Chạy dài (trước khi coi là xong)
- [ ] 7 ngày liền không restart tay
- [ ] RAM không tăng dần (kiểm ngày 1 vs ngày 7)
- [ ] macOS không báo app tốn pin
- [ ] Log tự xoay vòng, không phình vô hạn

## 5. Đo hiệu năng

Ngưỡng nào cũng phải đo được, không phải cảm nhận:

| Ngưỡng | Đo thế nào |
|---|---|
| [N1](NFR.md#1-ngưỡng-chấp-nhận), [N2](NFR.md#1-ngưỡng-chấp-nhận) trễ sync | Log timestamp lúc phát hiện và lúc ghi ở đầu nhận; so hiệu |
| [N5](NFR.md#1-ngưỡng-chấp-nhận) tìm kiếm | Nhồi 1000 item, đo query |
| [N6](NFR.md#1-ngưỡng-chấp-nhận) mở cửa sổ | Bấm phím tắt, đo tới lúc gõ được |
| [N9](NFR.md#2-tài-nguyên) CPU rảnh | Activity Monitor / `top`, quan sát 10 phút |
| [N10](NFR.md#2-tài-nguyên) RAM | Đo sau khi sync 20 ảnh |

## 6. Quality gate mỗi phase

Không qua phase sau khi chưa xong:

| Phase | Gate |
|---|---|
| 0 | Ba spike kết luận rõ; ADR sai thì đã sửa |
| 1 | T1, T4, T5 xanh · CPU đạt N9 · lịch sử sống qua restart |
| 2 | **T2, T3 xanh (chặn release)** · T8–T11 xanh · checklist hai máy phần text · N1 đạt |
| 3 | T6 (ảnh), T7 xanh · checklist ảnh hai máy · N2, N10 đạt |
| 4 | Checklist một máy đủ trên **cả hai** OS · mọi lỗi ở [NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi) đều nhìn thấy · N5, N6 đạt |
| 5 | Tự chạy được sau restart · crash tự hồi · chạy dài 7 ngày đạt |

## 7. Không test

Ghi ra để không ai tưởng là quên:

- **Test UI tự động** (Playwright/WebDriver) — một cửa sổ, một người dùng. Chi phí dựng cao hơn giá trị. Checklist thủ công thay thế.
- **Test load / nhiều node** — hai máy, ngoài scope ([PRD § Ngoài scope](PRD.md#4-ngoài-scope)).
- **Fuzzing** — trừ parse frame, đã có T8 với các case cụ thể.
- **Đo test coverage %** — chỉ số coverage khuyến khích viết test rỗng. Danh sách T1–T12 ở trên là chuẩn thật.
