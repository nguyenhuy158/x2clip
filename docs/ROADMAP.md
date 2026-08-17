# Roadmap — x2clip

> Nguồn chính cho: thứ tự phase, deliverable, exit criteria. Cách kiểm chứng chi tiết ở [TEST-PLAN.md](TEST-PLAN.md). Story tương ứng ở [USER-STORIES.md](USER-STORIES.md).

Không có deadline — đây là project cá nhân. Thứ tự thì có, và nó **có ràng buộc**: mỗi phase phải chạy được và đóng được exit criteria trước khi qua phase sau.

**Trạng thái hiện tại (2026-08-17):** Phase 0 ✅. Phase 1 🟡 code xong, còn chờ chạy thử NixOS. Phase 2 🟡 code + 11 test xanh trên hộp thư giả, **chưa gọi R2 thật lần nào**. Việc kế tiếp không phải viết code mà là [4 mục checklist R2](#phase-2--hộp-thư-r2--mã-hoá--) — tạo bucket, access key, lifecycle, tra giá. Sau đó chọn giữa **Phase 2b** (chuông Tailscale, tuỳ chọn) và **pause của [US-A4](USER-STORIES.md#us-a4--tạm-dừng-sync)** (docs đã tả, code chưa có).

Phase 0 chi tiết: 0.1, 0.2, 0.3 đều pass ([PRD Q1, Q2](PRD.md#9-câu-hỏi-mở): NixOS chạy **X11**, không DE; Tailscale `nixos` ↔ `macbook` ping trực tiếp 6ms; `arboard` đọc/ghi text + ảnh OK cả hai máy).

**Đổi kiến trúc 2026-08-17:** hai máy **ít khi online cùng lúc** (mac ở công ty, nixos ở nhà). Nội dung giờ đi qua hộp thư R2 thay vì P2P Tailscale — [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md). Phase 2 viết lại; Phase 1 không đổi.

---

## Phase 0 — Spike · ✅

**Mục đích:** xác nhận ba giả định có thể phá vỡ cả kế hoạch. Làm bằng script nhỏ, không đoán.

| # | Kiểm gì | Fail thì sao |
|---|---|---|
| 0.1 | Compositor trên NixOS là gì, có `wlr-data-control`/`ext-data-control` không | Phải fallback: gọi `wl-copy`/`wl-paste` ngoài, hoặc chuyển X11 |
| 0.2 | Crate clipboard đọc + ghi được text **và ảnh** trên đúng compositor đó | Đổi sang gọi tool ngoài → sửa [ADR-0003](ADR/0003-clipboard-arboard-polling.md) |
| 0.3 | Hai máy ping được nhau qua tên MagicDNS, mở được TCP port | Kiểm lại Tailscale ACL; nếu không xong thì [ADR-0001](ADR/0001-transport-tailscale.md) phải xem lại |

**Deliverable:** một ghi chú kết quả + ADR được cập nhật nếu có giả định sai.

**Exit criteria**
- [x] Biết chính xác compositor và protocol nó hỗ trợ — X11 (Xorg 21.1.23), không DE
- [x] Đọc *và* ghi được cả text lẫn ảnh trên **cả hai** máy, bằng tay
- [x] Mở được TCP giữa hai máy qua tên Tailscale
- [x] ADR nào bị chứng minh sai thì đã cập nhật — không cái nào sai, [ADR-0003](ADR/0003-clipboard-arboard-polling.md) chuyển sang Accepted

### Kết quả 0.2 (2026-08-17)

Spike: crate tạm `spikes/clip-probe` dùng `arboard` 3.6.1, đã xoá sau khi đo.

| | text đọc | text ghi | ảnh đọc | ảnh ghi |
|---|---|---|---|---|
| macbook (macOS) | ✅ | ✅ | ✅ ảnh thật từ `screencapture` 400x240 | ✅ |
| nixos (X11 `:0`) | ✅ | ✅ | ✅ | ✅ |

Hai điều phát hiện thêm, ảnh hưởng thiết kế:

1. **X11 clipboard là owner-based** — process ghi phải *còn sống* thì nội dung mới còn. Không phải vấn đề với daemon chạy nền, nhưng CLI kiểu `x2clip paste` một phát rồi thoát sẽ mất nội dung trên Linux. Ghi nhận cho [Phase 1](#phase-1--core-local-một-máy--).
2. Máy nixos có **hai X display**: `:0` (thật) và `:10` (xrdp). Chỉ `:0` kết nối được; `:10` timeout. Daemon phải chốt `DISPLAY=:0`, đừng đoán từ env.
3. Process **ngoài session đồ hoạ không thấy clipboard**. Qua SSH `XDG_SESSION_TYPE=tty`, phải export cả `DISPLAY=:0` **và** `XAUTHORITY=/home/huy/.Xauthority` mới đọc được. systemd user unit chạy đúng trong điều kiện đó → [Phase 5](#phase-5--đóng-gói--) phải set hai biến này trong unit (hoặc `After=`/`PartOf=graphical-session.target`), không thì app im lặng không thấy clipboard.

> Đừng bỏ phase này vì "chắc chạy được". Phát hiện Wayland không cho đọc clipboard lúc đang viết UI là đắt gấp nhiều lần.

## Phase 1 — Core local, một máy · 🟡 (xong trên macOS, chờ chạy thử NixOS)

**Story:** [US-B1](USER-STORIES.md#us-b1--lịch-sử-được-lưu-lại), [US-B2](USER-STORIES.md#us-b2--tìm-trong-lịch-sử) (phần CLI)

**Phạm vi:** `clip.rs`, `watcher.rs`, `store.rs`, `cli`. Chưa có mạng, chưa có UI.

- Poll clipboard, hash, chống trùng liên tiếp, ghi SQLite
- Prune theo [N14](NFR.md#3-giới-hạn), loại trừ item đã ghim
- `x2clip list` / `x2clip search <q>` in ra lịch sử

**Deliverable:** binary CLI chạy được trên cả hai máy.

**Exit criteria**
- [x] `cargo test` xanh, gồm test echo guard (T1) và test prune-không-đụng-pinned (T4) — 5 test
- [x] Lịch sử còn nguyên sau restart — daemon tắt, `x2clip list` vẫn ra đủ
- [x] Copy hai lần cùng nội dung → một entry, `updated_at` được cập nhật (T5)
- [x] CPU lúc rảnh đạt [N9](NFR.md#2-tài-nguyên) — 0.0%, RSS 13.9 MB trên macBook
- [ ] **Còn lại:** chạy thử trên NixOS (`DISPLAY=:0` + `XAUTHORITY`, mục 2–3 Phase 0.2)

Ghi chú: chưa có `x2clip copy <id>` — trên X11 clipboard là owner-based, tiến trình ghi rồi thoát là mất nội dung. Dùng lại item là việc của daemon ở [Phase 4](#phase-4--ui--).

## Phase 2 — Hộp thư R2 + mã hoá · 🟡 (code + test xong, chờ bucket thật)

**Story:** [US-A1](USER-STORIES.md#us-a1--copy-ở-máy-này-paste-ở-máy-kia), [US-A2](USER-STORIES.md#us-a2--không-có-vòng-lặp-echo), [US-C5](USER-STORIES.md#us-c5--cấu-hình-được)

**Phạm vi:** `crypto.rs`, `mailbox.rs`, `config.rs`, bảng `seen`, nối vào watcher. **Text only** — ảnh để phase sau.

Kiến trúc: [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md). Hai máy **không** cần cùng online.

**Trước khi viết dòng code đầu** ([ADR-0006 § Checklist](ADR/0006-r2-mailbox-store-and-forward.md)):
- [ ] Tạo bucket R2 + access key giới hạn đúng bucket đó
- [ ] Sinh khoá mã hoá, copy tay sang hai máy, `chmod 0600`, xác nhận không nằm trong `git status`
- [ ] Đặt lifecycle rule 30 ngày cho prefix `inbox/`
- [ ] Tra bảng giá R2 để chốt chu kỳ poll ([N13b](NFR.md#3-giới-hạn))

> Chép secret bằng tay ở đây là **cách tạm cho v1 hai máy**. [ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md) thay nó bằng đăng nhập + passphrase sau v1 — nên đừng nhúng access key vào code, đọc từ config/Keychain ngay từ đầu.

Việc chính:
- Mã hoá **trước** khi PUT, giải mã sau khi GET ([ADR-0005 C1–C6](ADR/0005-no-app-layer-crypto.md#xem-lại-2026-08-17--mã-hoá-tầng-app-thành-bắt-buộc))
- PUT vào `inbox/<máy kia>/<ulid>`; LIST/GET/DELETE prefix của mình
- Poll định kỳ + poll khi vừa bật máy / vừa có mạng
- Chỉ item `ts` lớn nhất trong lô được ghi clipboard; cả lô vào history
- Echo guard đi qua đường thật

**Deliverable:** copy ở máy này, paste được ở máy kia — **kể cả khi lúc copy máy kia đang tắt**.

**Exit criteria**
- [ ] Sync hai chiều chạy trên hai máy thật
- [ ] **Copy ở A khi B tắt → bật B lên thì nhận được.** Đây là lý do phase này tồn tại — T11 xanh trên hộp thư giả, còn chờ máy thật
- [x] **Echo loop = 0 message dư** ([N7](NFR.md#1-ngưỡng-chấp-nhận)) — T2, T3 trong `core/tests/phase2.rs`
- [ ] Trễ đạt [N1](NFR.md#1-ngưỡng-chấp-nhận) — đo được khi có bucket thật
- [ ] Round-trip mã hoá → giải mã pass trên **cả hai** OS — T6 xanh trên macOS, chưa chạy NixOS
- [x] Sửa 1 byte ciphertext → giải mã **fail**, object **không** bị xoá — T14
- [x] Object mã khoá khác → fail sạch, daemon không crash — T14
- [x] Không log nội dung clipboard, khoá, access key ([N23](NFR.md#4-bảo-mật)) — T12
- [x] Mất mạng lúc copy → item vẫn trong `store`, có mạng lại thì PUT lên, không mất — T16
- [x] DELETE fail → object không bị xử lý hai lần (sổ `seen`), clipboard không bị ghi đè bằng item cũ — T13
- [x] Unicode/emoji/xuống dòng giống byte-for-byte — T6
- [x] Config sai cú pháp → báo lỗi rõ, không ghi đè file người dùng — T9

### Trạng thái 2026-08-17

Code xong: `core/src/{crypto,config,mailbox,sync}.rs`, bảng `seen`, `x2clip watch` gọi cả hai nhịp. 11 test Phase 2 xanh trên **hộp thư giả trong process**.

Rà lại sau khi code xong, đã siết bốn chỗ (commit `6ab908a`) và hai chỗ nữa (`94566ba`):
- `MailboxConfig` impl `Debug` viết tay che access key; T12 có assert `format!("{cfg:?}")` để nó không lặng lẽ mất tác dụng lúc ai đó thêm lại `derive(Debug)`
- File secret rộng hơn `0600` → **từ chối chạy** ([US-C5](USER-STORIES.md#us-c5--cấu-hình-được), [N18c](NFR.md#4-bảo-mật)), kiểm **trước** khi parse, áp cho **cả** `config.toml` **và** `passphrase_file`. Tạo file bằng `OpenOptions().mode(0o600)` để không hở khoảnh khắc world-readable
- `passphrase_file` nở `~` — file mẫu tự gợi ý `~/.config/x2clip/...` mà TOML không bung hộ
- `machine == peer` (hoặc còn là tên mẫu) + đã bật `[mailbox]` → từ chối load. Máy tự gửi cho chính nó là vòng lặp hiện ra dưới dạng **hoá đơn R2**, không phải app treo

Còn lại, đều là việc cần bucket thật hoặc máy thật:
- 4 mục checklist "trước khi viết dòng code đầu" ở trên — chưa làm cái nào. **Đây là việc kế tiếp**, không phải viết thêm code
- `R2Mailbox` (rusty-s3 + ureq) **chưa từng gọi R2 thật một lần nào**
- Chạy thử hai máy, kể cả trường hợp tắt B rồi bật lại
- Đo N1b/N1c

Hai đầu dây chưa quyết:
- **[US-A4](USER-STORIES.md#us-a4--tạm-dừng-sync) pause** — [T16](TEST-PLAN.md) đòi item copy lúc tạm dừng không được PUT. `SYNC_KHONG_GUI` đã có trong `store.rs`, nhưng `x2clip watch` chưa có đường bật/tắt nào. Docs đi trước code ở chỗ này
- **Phase 2b** — chuông Tailscale, [ADR-0006 § 6b](ADR/0006-r2-mailbox-store-and-forward.md#6b--tailscale-hạ-cấp-thành-kênh-thông-báo) nói làm sau khi hộp thư đã chạy thật

Chốt mã hoá, lệch với chữ nghĩa của [N18b](NFR.md#4-bảo-mật) (chỗ đó gợi ý `age` trước): dùng `dryoc` (libsodium thuần Rust) — Argon2id tham số mặc định → khoá 32 byte → `crypto_secretbox` mỗi message. Recipient passphrase của `age` chạy scrypt **mỗi message**, ~1s CPU cho mỗi lần copy, vỡ N1b lẫn N9; mà [N18f](NFR.md#4-bảo-mật) vốn đã chốt dẫn xuất bằng Argon2id nên dẫn xuất một lần rồi AEAD từng message mới đúng ý. Vẫn là thư viện có kiểm chứng, vẫn không tự lắp primitive — đúng tinh thần [ADR-0005 C1](ADR/0005-no-app-layer-crypto.md).

Một cái bẫy đã gặp khi thiết kế, ghi ra đây trước khi nó cắn lúc chạy hai máy: **salt phải copy y hệt sang máy kia** cùng với passphrase. Cùng passphrase + khác salt = khác khoá, và triệu chứng trông hệt như sai passphrase. File config mẫu có sẵn cảnh báo này ngay trên dòng `salt`.

> Đây là phase quan trọng nhất. Xong phase 2 là app đã có giá trị thật, phần còn lại là tiện nghi.

## Phase 2b — Kênh chuông Tailscale · ⬜

**Tuỳ chọn.** **Phạm vi:** `notify.rs`. Không chặn phase nào. Bỏ hẳn cũng được — mất tốc độ, không mất đúng.

Không có kênh này thì trễ = chu kỳ poll (30–60s). Có thì về mức [N1](NFR.md#1-ngưỡng-chấp-nhận) khi cả hai máy đang bật.

- WebSocket chở **object key**, ~100 byte, **không** chở nội dung ([ADR-0006 § 6b](ADR/0006-r2-mailbox-store-and-forward.md#6b--tailscale-hạ-cấp-thành-kênh-thông-báo))
- Nhận key → chạy đúng luồng ingest của Phase 2, không thêm code path thứ hai

**Exit criteria**
- [ ] Bind **chỉ** vào địa chỉ Tailscale ([N19](NFR.md#4-bảo-mật)); không tìm được địa chỉ → **từ chối listen**, không fallback `0.0.0.0`
- [ ] Chỉ nhận peer trong config ([N20](NFR.md#4-bảo-mật)), có [T10](TEST-PLAN.md#t10--peer-lạ-bị-từ-chối)
- [ ] Tắt Tailscale → sync **vẫn chạy** qua poll, chỉ chậm hơn
- [ ] Chuông trùng / key lạ → vô hại, `seen` chặn

## Phase 3 — Ảnh · 🟡

**Story:** [US-A3](USER-STORIES.md#us-a3--đồng-bộ-ảnh), [US-C4](USER-STORIES.md#us-c4--không-lưu-password)

- Đọc/ghi ảnh, chuẩn hoá về PNG
- Giới hạn dung lượng [N15](NFR.md#3-giới-hạn), vượt thì bỏ qua + đánh dấu, **không cắt**
- Lưu blob + thumbnail
- Bỏ qua nội dung đánh dấu nhạy cảm ([N22](NFR.md#4-bảo-mật))

**Exit criteria**
- [ ] Screenshot macOS → paste được ở NixOS, cùng kích thước pixel
      (round-trip nguyên byte đã xanh trên hộp thư giả — `phase3.rs`; ghi/đọc
      lại trên clipboard macOS thật cũng giữ nguyên hash — `clipboard_that.rs`;
      còn thiếu hai máy thật, cùng chỗ tắc với Phase 2)
- [ ] Chiều ngược lại cũng được
- [ ] Trễ đạt [N2](NFR.md#1-ngưỡng-chấp-nhận)
- [x] Ảnh vượt giới hạn → vào lịch sử local, đánh dấu không sync, không crash
- [x] Password từ password manager (macOS) không vào lịch sử
      (`org.nspasteboard.ConcealedType`, đọc được trên pasteboard **thật** —
      `clipboard_that.rs`, chạy với `--ignored`)
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
- `.github/workflows/release.yml`: tag `v*` → build `.dmg` trên `macos-14`, `nix build` trên `ubuntu-latest`, tạo GitHub Release ([PRD §8](PRD.md#8-phát-hành))

**CD bất đối xứng giữa hai máy** — đừng cố làm cho giống nhau:

| Máy | Giao hàng |
|---|---|
| macbook | `.dmg` tải từ Release. **Không notarize** → lần đầu phải `xattr -dr com.apple.quarantine /Applications/x2clip.app`, README bắt buộc ghi câu này |
| nixos | `nix build github:nguyenhuy158/x2clip` — flake fetch source rồi build. **Không** upload binary Linux vào Release; job Linux trong CD chỉ để chứng minh `nix build` xanh |

Release job phải `needs:` job test — không publish bản có [T2/T3](TEST-PLAN.md) (echo guard) đỏ.

**Exit criteria**
- [ ] `nix build` xong chạy được trên NixOS
- [ ] `.app` mở được trên máy sạch
- [ ] Tag `v*` → Release tự có `.dmg`, và `.dmg` đó mở được trên máy mac sạch sau khi bỏ quarantine
- [ ] README có đúng câu lệnh bỏ quarantine, đã thử trên máy chưa từng cài app
- [ ] Restart máy → app tự chạy nền, không hiện cửa sổ
- [ ] Kill process → tự restart
- [ ] Chạy nền 7 ngày không cần can thiệp ([N25](NFR.md#6-khả-năng-vận-hành))

> Nix build cho app GUI (Rust + node deps + webkitgtk) thường mất công hơn cảm giác ban đầu. Kẹt thì fallback `nix develop` + build tay, đóng gói sạch sau — đừng để nó chặn việc dùng app.

---

## Sau v1

Chưa cam kết. Chỉ làm khi dùng thật rồi thấy thiếu:

| Ý tưởng | Điều kiện kích hoạt |
|---|---|
| ~~Sync cả lịch sử~~ | **Không còn là việc riêng.** Phase 2 đưa mọi item qua hộp thư → lịch sử tự hội tụ. Cần merge rule (ghim, xoá) thì lấy ở [ADR-0004 § Xem lại](ADR/0004-storage-sqlite-local-history.md#xem-lại-2026-08-17--sync-lịch-sử). |
| ~~Relay server (D1 / Durable Object)~~ | **Đã giải bằng R2** ở [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md), không cần Worker. |
| Xoá / ghim đồng bộ hai máy — tombstone + merge `pinned` bằng OR ([ADR-0004 § Xem lại](ADR/0004-storage-sqlite-local-history.md#quy-tắc-merge--trả-lời-các-câu-hỏi-adr-này-từng-nêu)) | Ghim hoặc xoá hai lần thấy khó chịu thật. Nội dung đã hội tụ; đây là phần *state* chưa. |
| **Backup toàn bộ DB lên R2** — khác với hộp thư: hộp thư chở item lẻ và tự xoá sau 30 ngày, backup giữ cả file. `sqlite3 .backup` → mã hoá → PUT, ~1 lần/ngày, lỗi thì log rồi bỏ qua. | Mất history một lần và thấy tiếc. Trước đó `cp` file `.db` là đủ. |
| **Đăng nhập + khoá từ passphrase** ([ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md)) — endpoint `auth`, credential R2 tạm thời theo prefix, Argon2id từ passphrase. FR16–FR19 ở [PRD](PRD.md#7-yêu-cầu-chức-năng) | **Đã quyết** (2026-08-17). Cần khi thêm máy thứ 3, hoặc khi chép secret tay thành phiền thật. Không chặn v1. |
| Máy thứ 3+ | Có máy thứ ba. Cần [ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md) trước, và tính lại `recipient`: PUT vào N-1 hộp thư thay vì 1. |
| Rotation khoá mã hoá | v1 không có. Mất khoá = mất hết ([ADR-0005 § Mô hình đe doạ](ADR/0005-no-app-layer-crypto.md#mô-hình-đe-doạ--cái-gì-được-bảo-vệ-cái-gì-không)). Cần khi có máy thứ 3 hoặc nghi khoá bị lộ. |
| Watch path riêng cho Linux | Trễ 250ms thấy rõ khi dùng |
| Rich text / RTF | Paste mất format gây khó chịu thật |
| Mobile | Không dự kiến |

## Bảng phase ↔ story

| Phase | Story |
|---|---|
| 0 | — (spike kỹ thuật) |
| 1 | US-B1, US-B2 (CLI) |
| 2 | US-A1, US-A2, US-C5 |
| 2b | — (tối ưu trễ, tuỳ chọn) |
| 3 | US-A3, US-C4 |
| 4 | US-A4, US-B2 (UI), US-B3, US-B4, US-B5, US-C1, US-C2 |
| 5 | US-C3 |
| 6 (sau v1) | US-D1, US-D2, US-D3, US-D4 |
