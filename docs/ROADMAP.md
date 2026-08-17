# Roadmap — x2clip

> Nguồn chính cho: thứ tự phase, deliverable, exit criteria. Cách kiểm chứng chi tiết ở [TEST-PLAN.md](TEST-PLAN.md). Story tương ứng ở [USER-STORIES.md](USER-STORIES.md).

Không có deadline — đây là project cá nhân. Thứ tự thì có, và nó **có ràng buộc**: mỗi phase phải chạy được và đóng được exit criteria trước khi qua phase sau.

**Trạng thái hiện tại:** Phase 0 đang làm. 0.1 và 0.3 đã pass ([PRD Q1, Q2](PRD.md#9-câu-hỏi-mở) đã trả lời: NixOS chạy **X11**, không DE; Tailscale `nixos` ↔ `macbook` ping trực tiếp 6ms). Còn 0.2 — thử `arboard` đọc/ghi text + ảnh trên cả hai máy.

---

## Phase 0 — Spike · 🟡 Đang làm (0.1 ✅, 0.3 ✅, 0.2 còn lại)

**Mục đích:** xác nhận ba giả định có thể phá vỡ cả kế hoạch. Làm bằng script nhỏ, không đoán.

| # | Kiểm gì | Fail thì sao |
|---|---|---|
| 0.1 | Compositor trên NixOS là gì, có `wlr-data-control`/`ext-data-control` không | Phải fallback: gọi `wl-copy`/`wl-paste` ngoài, hoặc chuyển X11 |
| 0.2 | Crate clipboard đọc + ghi được text **và ảnh** trên đúng compositor đó | Đổi sang gọi tool ngoài → sửa [ADR-0003](ADR/0003-clipboard-arboard-polling.md) |
| 0.3 | Hai máy ping được nhau qua tên MagicDNS, mở được TCP port | Kiểm lại Tailscale ACL; nếu không xong thì [ADR-0001](ADR/0001-transport-tailscale.md) phải xem lại |

**Deliverable:** một ghi chú kết quả + ADR được cập nhật nếu có giả định sai.

**Exit criteria**
- [ ] Biết chính xác compositor và protocol nó hỗ trợ
- [ ] Đọc *và* ghi được cả text lẫn ảnh trên **cả hai** máy, bằng tay
- [ ] Mở được TCP giữa hai máy qua tên Tailscale
- [ ] ADR nào bị chứng minh sai thì đã cập nhật

> Đừng bỏ phase này vì "chắc chạy được". Phát hiện Wayland không cho đọc clipboard lúc đang viết UI là đắt gấp nhiều lần.

## Phase 1 — Core local, một máy · ⬜

**Story:** [US-B1](USER-STORIES.md#us-b1--lịch-sử-được-lưu-lại), [US-B2](USER-STORIES.md#us-b2--tìm-trong-lịch-sử) (phần CLI)

**Phạm vi:** `clip.rs`, `watcher.rs`, `store.rs`, `cli`. Chưa có mạng, chưa có UI.

- Poll clipboard, hash, chống trùng liên tiếp, ghi SQLite
- Prune theo [N14](NFR.md#3-giới-hạn), loại trừ item đã ghim
- `x2clip list` / `x2clip search <q>` in ra lịch sử

**Deliverable:** binary CLI chạy được trên cả hai máy.

**Exit criteria**
- [ ] `cargo test` xanh, gồm test echo guard và test prune-không-đụng-pinned
- [ ] Lịch sử còn nguyên sau restart
- [ ] Copy hai lần cùng nội dung → một entry, `updated_at` được cập nhật
- [ ] CPU lúc rảnh đạt [N9](NFR.md#2-tài-nguyên)

## Phase 2 — Sync text giữa hai máy · ⬜

**Story:** [US-A1](USER-STORIES.md#us-a1--copy-ở-máy-này-paste-ở-máy-kia), [US-A2](USER-STORIES.md#us-a2--không-có-vòng-lặp-echo), [US-C5](USER-STORIES.md#us-c5--cấu-hình-được)

**Phạm vi:** `peer.rs`, `config.rs`, nối vào watcher. **Text only** — ảnh để phase sau.

- Listen bind vào IP Tailscale, dial peer, reconnect backoff
- Chỉ nhận peer trong config
- Echo guard đi qua đường mạng thật

**Deliverable:** copy ở máy này paste được ở máy kia.

**Exit criteria**
- [ ] Sync hai chiều chạy trên hai máy thật
- [ ] **Echo loop = 0 message dư** ([N7](NFR.md#1-ngưỡng-chấp-nhận)) — có automated test, không chỉ thử tay
- [ ] Trễ đạt [N1](NFR.md#1-ngưỡng-chấp-nhận)
- [ ] Rút mạng rồi cắm lại → tự reconnect trong [N4](NFR.md#1-ngưỡng-chấp-nhận), không mất item
- [ ] Unicode/emoji/xuống dòng giống byte-for-byte
- [ ] Config sai cú pháp → báo lỗi rõ, không ghi đè file người dùng

> Đây là phase quan trọng nhất. Xong phase 2 là app đã có giá trị thật, phần còn lại là tiện nghi.

## Phase 3 — Ảnh · ⬜

**Story:** [US-A3](USER-STORIES.md#us-a3--đồng-bộ-ảnh), [US-C4](USER-STORIES.md#us-c4--không-lưu-password)

- Đọc/ghi ảnh, chuẩn hoá về PNG
- Giới hạn dung lượng [N15](NFR.md#3-giới-hạn), vượt thì bỏ qua + đánh dấu, **không cắt**
- Lưu blob + thumbnail
- Bỏ qua nội dung đánh dấu nhạy cảm ([N22](NFR.md#4-bảo-mật))

**Exit criteria**
- [ ] Screenshot macOS → paste được ở NixOS, cùng kích thước pixel
- [ ] Chiều ngược lại cũng được
- [ ] Trễ đạt [N2](NFR.md#1-ngưỡng-chấp-nhận)
- [ ] Ảnh vượt giới hạn → vào lịch sử local, đánh dấu không sync, không crash
- [ ] Password từ password manager (macOS) không vào lịch sử
- [ ] RAM đạt [N10](NFR.md#2-tài-nguyên) sau khi sync ~20 ảnh

## Phase 4 — UI · ⬜

**Story:** [US-B3](USER-STORIES.md#us-b3--dùng-lại-một-item), [US-B4](USER-STORIES.md#us-b4--ghim-item), [US-B5](USER-STORIES.md#us-b5--xoá-item), [US-C1](USER-STORIES.md#us-c1--mở-lịch-sử-bằng-phím-tắt), [US-C2](USER-STORIES.md#us-c2--biết-được-sync-có-đang-chạy-hay-không), [US-A4](USER-STORIES.md#us-a4--tạm-dừng-sync)

- Tray icon với trạng thái kết nối
- Global hotkey mở cửa sổ
- Danh sách lịch sử, tìm kiếm, ghim, xoá, click để dùng lại
- Tạm dừng sync

**Exit criteria**
- [ ] Chạy thật trên **cả hai** OS, không chỉ macOS
- [ ] Phím tắt mở cửa sổ trong [N6](NFR.md#1-ngưỡng-chấp-nhận), ô tìm kiếm sẵn con trỏ
- [ ] Tray phân biệt được đã kết nối / mất kết nối / tạm dừng
- [ ] Mọi trường hợp ở [NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi) đều nhìn thấy được — không có lỗi im lặng
- [ ] Tìm kiếm ở mức 1000 item đạt [N5](NFR.md#1-ngưỡng-chấp-nhận)

## Phase 5 — Đóng gói · ⬜

**Story:** [US-C3](USER-STORIES.md#us-c3--tự-chạy-khi-đăng-nhập)

- macOS: `.app` bundle + launchd plist
- NixOS: `flake.nix` (cả `aarch64-darwin` và `x86_64-linux`) + systemd user unit
- README hướng dẫn cài

**Exit criteria**
- [ ] `nix build` xong chạy được trên NixOS
- [ ] `.app` mở được trên máy sạch
- [ ] Restart máy → app tự chạy nền, không hiện cửa sổ
- [ ] Kill process → tự restart
- [ ] Chạy nền 7 ngày không cần can thiệp ([N25](NFR.md#6-khả-năng-vận-hành))

> Nix build cho app GUI (Rust + node deps + webkitgtk) thường mất công hơn cảm giác ban đầu. Kẹt thì fallback `nix develop` + build tay, đóng gói sạch sau — đừng để nó chặn việc dùng app.

---

## Sau v1

Chưa cam kết. Chỉ làm khi dùng thật rồi thấy thiếu:

| Ý tưởng | Điều kiện kích hoạt |
|---|---|
| Sync cả lịch sử giữa hai máy | Thường xuyên cần item cũ ở máy kia |
| Relay server (Cloudflare DO) | Cần chạy trên máy không cài được Tailscale, hoặc hai máy hay lệch giờ online |
| Máy thứ 3+ | Có máy thứ ba |
| Watch path riêng cho Linux | Trễ 250ms thấy rõ khi dùng |
| Rich text / RTF | Paste mất format gây khó chịu thật |
| Mobile | Không dự kiến |

## Bảng phase ↔ story

| Phase | Story |
|---|---|
| 0 | — (spike kỹ thuật) |
| 1 | US-B1, US-B2 (CLI) |
| 2 | US-A1, US-A2, US-C5 |
| 3 | US-A3, US-C4 |
| 4 | US-A4, US-B2 (UI), US-B3, US-B4, US-B5, US-C1, US-C2 |
| 5 | US-C3 |
