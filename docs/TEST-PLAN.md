# Test Plan — x2clip

> Nguồn chính cho: chiến lược test và quality gate. Ngưỡng số ở [NFR.md](NFR.md). Exit criteria từng phase ở [ROADMAP.md](ROADMAP.md).

---

## 1. Nguyên tắc

- **`core` là library thuần Rust** để test logic sync bằng `cargo test`, không cần dựng GUI. Đây là lý do tách `core` khỏi `app` ngay từ đầu ([ARCHITECTURE § Thành phần](ARCHITECTURE.md#2-thành-phần)).
- **Clipboard giả (fake) cho automated test.** Clipboard thật là tài nguyên toàn cục của máy — test song song sẽ đấu nhau, và CI không có clipboard. `clip` phải là trait có hai impl: thật và fake trong bộ nhớ.
- **Hộp thư giả (fake) cũng vậy.** `mailbox` phải là trait (`put`/`list`/`get`/`delete`) có hai impl: R2 thật và một `HashMap` trong bộ nhớ. Không có nó thì mọi test sync đều cần mạng, cần tiền, và **không mô phỏng được ca "máy kia đang tắt"** — tức là không test được đúng cái [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md) sinh ra để giải quyết.
- **Không test cái mà compiler đã đảm bảo.** Không viết test cho getter/setter.
- **Mỗi bug tìm được ngoài test → thêm một test.** Nhất là bug echo loop và bug mất item.
- **Test thủ công là hợp lệ** cho phần chạm hệ điều hành thật (clipboard thật, tray, phím tắt, Wayland). Nhưng phải có checklist ghi lại, không phải "tôi thử rồi thấy ok".

## 2. Tầng test

| Tầng | Test gì | Công cụ | Số lượng |
|---|---|---|---|
| Unit | hash, echo guard, prune, parse config, parse plaintext object, mã hoá/giải mã | `cargo test` | Nhiều nhất |
| Integration (trong process) | Hai instance `core` nói chuyện qua clipboard giả + **hộp thư giả** | `cargo test` | Vài chục |
| Thủ công, một máy | Clipboard thật, ảnh thật, UI | Checklist | Mỗi phase |
| Thủ công, hai máy | Sync thật qua bucket R2 thật | Checklist | Mỗi phase từ 2 |
| Chạy dài | Ổn định 7 ngày | Quan sát + log | Trước khi coi là xong |

**CI chỉ có từ Phase 5, và chỉ để chặn release.** `.github/workflows/release.yml` chạy `cargo test` như job mà release `needs:` ([ROADMAP Phase 5](ROADMAP.md#phase-5--đóng-gói--)) — mục đích duy nhất là không publish bản có [T2](#t2--không-có-echo-loop-hai-node)/[T3](#t3--thứ-tự-set-cờ-trước-khi-ghi) đỏ.

Phase 1–4 thì chạy tay trước mỗi lần đóng phase. Không có ai nhắc, và CI cũng **không** cứu được phần lớn danh sách này: hộp thư giả chạy được trên CI, nhưng checklist hai máy thì không — không runner nào có hai máy của bạn và một bucket R2 thật. **Kỷ luật chạy checklist là của bạn.**

## 3. Test bắt buộc phải có

Đây là những test mà thiếu chúng thì app hỏng theo cách khó thấy. Không được bỏ vì "gọn hơn".

### T1 — Echo guard, một node
Ghi clipboard qua đường "nhận từ hộp thư" → poll watcher → **assert không sinh item mới nào**.

Đây là test rẻ nhất và chặn được bug tệ nhất.

### T2 — Không có echo loop, hai node
Hai instance `core` trong một process, hai clipboard giả, hai store tạm, **một hộp thư giả dùng chung**.

- Copy một lần ở A
- **Assert đúng 1 object trong `inbox/B/` và 0 object trong `inbox/A/`**
- Sau khi B ingest xong, assert B **không** PUT lại gì cả — đây là chỗ echo loop sẽ hiện ra dưới dạng hoá đơn R2, không phải dưới dạng máy treo
- Assert tổng số PUT là hữu hạn (đặt trần: vượt 5 thì fail ngay, không chờ timeout)

Test này ứng với [N7](NFR.md#1-ngưỡng-chấp-nhận) và [US-A2](USER-STORIES.md#us-a2--không-có-vòng-lặp-echo). **Fail = chặn release.**

### T3 — Thứ tự set cờ trước khi ghi
Assert `last_written_hash` được set **trước** khi clipboard bị ghi. Đảo thứ tự là có race ([ARCHITECTURE § Echo guard](ARCHITECTURE.md#echo-guard--chỗ-dễ-sai-nhất)).

Cách làm: clipboard giả ghi lại trạng thái cờ tại thời điểm `set()` được gọi.

### T4 — Prune không đụng item đã ghim
Nhồi quá hạn mức [N14](NFR.md#3-giới-hạn), trong đó có item đã ghim, prune, assert item ghim **còn nguyên** và item cũ nhất chưa ghim đã mất.

### T5 — Copy trùng liên tiếp không sinh row mới
Copy cùng nội dung hai lần → một row, `updated_at` tăng, `created_at` không đổi.

### T6 — Round-trip nội dung
Đi qua **cả** mã hoá lẫn hộp thư giả, không chỉ qua serialize.

- Text: Unicode, emoji, xuống dòng, tab, chuỗi rất dài → nhận được **byte-for-byte** giống
- Ảnh: PNG → mã hoá → object → giải mã → decode lại ra ảnh cùng kích thước pixel
- Chuỗi rỗng → bị bỏ qua, không tạo item, không PUT

### T7 — Vượt giới hạn dung lượng
Item vượt [N15](NFR.md#3-giới-hạn)/[N16](NFR.md#3-giới-hạn) → vào store với `synced = 0`, **không bị cắt bớt**, không PUT, không crash.

### T8 — Object lỗi không làm chết daemon
Nạp plaintext JSON sai cú pháp, `v` lạ, `kind` lạ, base64 hỏng, thiếu field → mỗi trường hợp: log + bỏ qua, daemon vẫn sống ([NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi)).

Thêm hai case của tầng ngoài, vì chúng đi trước cả JSON: object rỗng 0 byte, và object dài đúng bằng cỡ header AEAD rồi hết. Cả hai phải **giữ object**, không xoá ([T14](#t14--giải-mã-fail-thì-giữ-object)).

### T9 — Config
- Thiếu file → tạo mặc định, chạy được
- Sai cú pháp → báo lỗi có số dòng, **không ghi đè** file người dùng
- Không cấu hình hộp thư (thiếu bucket/khoá) → chạy ở chế độ local-only, không crash
- Có bucket nhưng access key sai → báo lỗi **phân biệt được với mất mạng** ([NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi)), lịch sử local vẫn chạy

### T10 — Peer lạ bị từ chối
Kết nối từ địa chỉ không có trong config → từ chối + log ([N20](NFR.md#4-bảo-mật)).

Chỉ áp dụng cho kênh chuông ([Phase 2b](ROADMAP.md#phase-2b--kênh-chuông-tailscale--)) — Phase 2 không mở socket nào nên test này chưa có gì để chạy.

### T11 — Máy kia đang tắt thì item vẫn tới
Không còn là câu hỏi mở: [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md) chốt là **gửi bù cả hàng đợi**, và [N8](NFR.md#1-ngưỡng-chấp-nhận) cam kết 0 mất mát *kể cả khi máy kia đang tắt lúc copy*.

Với hộp thư giả, "B đang tắt" = không chạy vòng ingest của B. Kịch bản:

- B chưa chạy. Copy **ba** nội dung khác nhau ở A → assert 3 object nằm trong `inbox/B/`
- Cho B chạy ingest **một lần**
- Assert **cả ba** vào lịch sử của B (đây là chỗ "clipboard là một giá trị" **không** áp dụng — lịch sử là hàng đợi)
- Assert **chỉ một** item được ghi vào clipboard của B: item có `ts` lớn nhất ([ARCHITECTURE § Nhận từ hộp thư](ARCHITECTURE.md#nhận-từ-hộp-thư)). Ghi cả ba là bug thấy được bằng mắt: clipboard cuối cùng lại là nội dung cũ nhất
- Assert B **không** PUT lại gì (nối với [T2](#t2--không-có-echo-loop-hai-node))

### T12 — Không log nội dung clipboard
Grep log sau khi chạy: không được xuất hiện nội dung, chỉ được có độ dài/hash/loại ([N23](NFR.md#4-bảo-mật)).

Từ Phase 2, grep thêm: **khoá mã hoá, access key R2, passphrase** đều không được có trong log ([N18c](NFR.md#4-bảo-mật), [N18g](NFR.md#4-bảo-mật)). Chuỗi tìm là **giá trị thật** của khoá trong config test, không phải tên biến.

### T13 — DELETE fail không gây xử lý hai lần
Hộp thư giả với `delete` luôn trả lỗi. Ingest cùng một object **hai lần**:

- Lần đầu: vào lịch sử, ghi clipboard
- Copy một nội dung mới ở B (clipboard B giờ là nội dung mới)
- Lần hai: sổ `seen` chặn → assert **không** thêm row, và **assert clipboard B vẫn là nội dung mới**, không bị object cũ ghi đè

Đây là lý do bảng `seen` tồn tại. Không có test này thì bug chỉ hiện lúc R2 lỗi thật, tức là lúc khó gỡ nhất.

### T14 — Giải mã fail thì giữ object
- Sửa **1 byte** ciphertext → giải mã fail, **không** vào store, **không** vào clipboard, object **không bị xoá**, daemon sống ([N18d](NFR.md#4-bảo-mật))
- Object mã bằng khoá khác → fail sạch, cùng các assert trên

"Không xoá" là phần dễ quên nhất: xoá cái mình không giải mã được là tự tay phá bằng chứng.

### T15 — Object key không rò rỉ gì
Assert key khớp `inbox/<máy>/<ULID>` và **không** chứa hash, không chứa nội dung, không chứa `kind` ([N18e](NFR.md#4-bảo-mật)). Hai lần copy **cùng** nội dung → hai key **khác nhau**.

Key trùng theo hash thì người xem bucket biết được "hai máy này copy lại cùng thứ", dù không đọc được thứ đó.

### T16 — Hàng chờ PUT sống qua restart
Mất mạng lúc copy → item nằm trong `store`, chưa `synced`. Kill process, dựng lại từ **cùng file DB** → assert item vẫn được PUT.

Hàng chờ chỉ nằm trong RAM thì mất mạng + restart = mất item, mà [N8](NFR.md#1-ngưỡng-chấp-nhận) nói 0.

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

Ca **quan trọng nhất** nằm ở đầu danh sách, không phải cuối: hai máy hiếm khi bật cùng lúc ([PRD § Ràng buộc](PRD.md#5-ràng-buộc)).

- [ ] **Tắt hẳn B. Copy 3 nội dung ở A. Bật B lên → cả 3 vào lịch sử B, clipboard B là nội dung mới nhất** ([N1c](NFR.md#1-ngưỡng-chấp-nhận), [T11](#t11--máy-kia-đang-tắt-thì-item-vẫn-tới))
- [ ] Chiều ngược lại: tắt A, copy ở B, bật A
- [ ] Copy text A khi **cả hai đang bật** → paste được ở B trong [N1](NFR.md#1-ngưỡng-chấp-nhận) (có chuông) hoặc [N1b](NFR.md#1-ngưỡng-chấp-nhận) (không chuông)
- [ ] Copy ảnh 2MB A → paste ở B, dưới 3s
- [ ] Copy liên tục 10 lần nhanh → không rác, không loop, thứ tự đúng
- [ ] Rút mạng ở A, copy, cắm lại → item lên hộp thư trong [N4](NFR.md#1-ngưỡng-chấp-nhận), không cần thao tác tay
- [ ] Đổi access key thành chuỗi sai → tray nói rõ **sai khoá**, không nói "mất mạng"
- [ ] `tailscale down` → **sync vẫn chạy** qua poll, chỉ chậm hơn; tray nói rõ là chậm, không nói lỗi
- [ ] Xem bucket sau khi B nhận xong → prefix `inbox/B/` **rỗng** (đã DELETE), key không chứa gì đọc được ([T15](#t15--object-key-không-rò-rỉ-gì))
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
| [N1](NFR.md#1-ngưỡng-chấp-nhận), [N2](NFR.md#1-ngưỡng-chấp-nhận) trễ sync | Log timestamp lúc phát hiện và lúc ghi ở đầu nhận; so hiệu. Ghi rõ lúc đo **có chuông hay không** — hai con số khác nhau ([N1b](NFR.md#1-ngưỡng-chấp-nhận)) |
| [N1c](NFR.md#1-ngưỡng-chấp-nhận) trễ lúc vừa bật máy | Đo từ lúc daemon start tới lúc clipboard được ghi. Đây là ca dùng chính, phải có số thật |
| [N13b](NFR.md#3-giới-hạn) chi phí R2 | Xem dashboard R2 sau 1 tuần chạy thật: số request Class A/B. Đối chiếu [R12](RISKS.md#r12--chi-phí-r2-vượt-dự-kiến) — con số free tier trong đó **chưa tra cứu** |
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
| 2 | **T2, T3 xanh (chặn release)** · T6 (text), T8, T9, T11–T16 xanh · checklist hai máy phần text, **gồm ca tắt-B-rồi-bật** · N1b, N1c đạt |
| 2b | T10 xanh · `tailscale down` mà sync vẫn chạy · N1 đạt (đây là phase duy nhất N1 áp dụng) |
| 3 | T6 (ảnh), T7 xanh · checklist ảnh hai máy · N2, N10 đạt |
| 4 | Checklist một máy đủ trên **cả hai** OS · mọi lỗi ở [NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi) đều nhìn thấy · N5, N6 đạt |
| 5 | Tự chạy được sau restart · crash tự hồi · chạy dài 7 ngày đạt |

## 7. Không test

Ghi ra để không ai tưởng là quên:

- **Test UI tự động** (Playwright/WebDriver) — một cửa sổ, một người dùng. Chi phí dựng cao hơn giá trị. Checklist thủ công thay thế.
- **Test load / nhiều node** — v1 hai máy. Máy thứ 3+ đã vào scope nhưng **sau v1** ([PRD § Ngoài scope](PRD.md#4-ngoài-scope)), test lúc đó.
- **Fuzzing** — trừ parse plaintext object, đã có T8 với các case cụ thể.
- **Test chống lại Cloudflare** — không mô phỏng R2 down, chậm, hay trả 500 lung tung. Hộp thư giả chỉ giả hai ca đã biết là gây bug: `delete` fail ([T13](#t13--delete-fail-không-gây-xử-lý-hai-lần)) và không có mạng. Ca còn lại thì để log nói ([R10](RISKS.md#r10--phụ-thuộc-cloudflare-r2)).
- **Test endpoint `auth`** — chưa có endpoint nào, cả Epic D nằm sau v1 ([ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md)).
- **Đo test coverage %** — chỉ số coverage khuyến khích viết test rỗng. Danh sách T1–T16 ở trên là chuẩn thật.
