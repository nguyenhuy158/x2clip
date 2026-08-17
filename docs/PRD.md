# PRD — x2clip

> Nguồn chính cho: vấn đề, người dùng, mục tiêu, scope. Con số hiệu năng ở [NFR.md](NFR.md), kiến trúc ở [ARCHITECTURE.md](ARCHITECTURE.md).

**Trạng thái:** Draft, chờ duyệt
**Cập nhật:** 2026-08-17

---

## 1. Vấn đề

Người dùng làm việc song song trên một máy macOS và một máy NixOS (Linux), hai máy **không cùng mạng**. Copy nội dung ở máy này, muốn paste ở máy kia thì hiện tại phải đi đường vòng: gửi qua chat, qua file, hoặc đánh lại tay.

Đồng thời clipboard của cả hai OS chỉ giữ **một** giá trị — copy cái mới là mất cái cũ, không tìm lại được.

## 2. Người dùng

Một người: chính chủ project. Dev, dùng terminal nhiều, tự quản trị cả hai máy, cài được software tuỳ ý.

Không có người dùng thứ hai trong scope này. Điều đó cho phép bỏ multi-tenant, phân quyền, chia sẻ — xem [§4](#4-ngoài-scope).

**Một người, nhiều máy** (đổi scope 2026-08-17). Số máy không cố định ở 2. Cài app lên máy mới phải **đăng nhập là dùng được**, không chép file secret bằng tay. Kéo theo [G4](#3-mục-tiêu) và [ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md).

## 3. Mục tiêu

| # | Mục tiêu | Vì sao |
|---|---|---|
| G1 | Copy ở máy nào thì paste được ở máy kia, không thao tác thêm | Đây là lý do tồn tại của app |
| G2 | Giữ lịch sử clipboard, tìm lại và dùng lại được | Clipboard chỉ giữ 1 giá trị là mất mát thật |
| G3 | Chạy được cả text và ảnh | Ảnh (screenshot) là phần lớn nhu cầu copy chéo máy |
| ~~G4~~ | ~~Không tự vận hành backend nào~~ → **thu hẹp 2026-08-17**: chỉ được có **một** thành phần tự vận hành, là endpoint đăng nhập/cấp quyền. Nó **không** chở nội dung clipboard và **không** giữ khoá giải mã. | Đổi lấy việc thêm máy mới không phải chép secret tay ([ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md)) |
| G5 | Một codebase cho cả hai OS | Fix một lần chạy cả hai máy |
| G6 | Máy mới: đăng nhập + passphrase là có lịch sử. Không chép file nào bằng tay | Mục tiêu trực tiếp của lần đổi scope này |

### Sản phẩm tham chiếu
[CleanClip](https://cleanclip.cc/), [Paste](https://pasteapp.io/) — cùng ý tưởng (history + sync), nhưng cả hai **chỉ chạy trên hệ Apple** và sync bằng iCloud/CloudKit. Cách đó không dùng được vì NixOS không có CloudKit. x2clip lấy *ý tưởng*, không lấy feature list.

## 4. Ngoài scope

| Không làm | Vì sao | Khi nào tính lại |
|---|---|---|
| Mobile (iOS/Android) | Không phải nhu cầu hiện tại | Có nhu cầu thật |
| Nhiều người dùng, chia sẻ clipboard | Một người dùng | Không dự kiến |
| ~~Máy thứ 3+~~ | **Vào scope 2026-08-17.** Thêm máy = đăng nhập trên máy đó ([G6](#3-mục-tiêu)) | — |
| Nhiều tài khoản trên một máy | Vẫn một người. Đăng nhập là để **nhận diện máy**, không phải để chuyển người dùng | Không dự kiến |
| Khôi phục khi quên passphrase | Server không giữ khoá nên **không khôi phục được**. Đây là hệ quả cố ý, không phải thiếu sót ([RISKS R11](RISKS.md#r11--mất-khoá-mã-hoá)) | Không dự kiến |
| ~~Sync **cả lịch sử** giữa 2 máy~~ | **Không còn là việc riêng 2026-08-17.** Mọi item đều đi qua hộp thư nên lịch sử **tự hội tụ**. Còn thiếu: đồng bộ *state* (ghim, xoá) — merge rule đã viết ở [ADR-0004 § Xem lại](ADR/0004-storage-sqlite-local-history.md#quy-tắc-merge--trả-lời-các-câu-hỏi-adr-này-từng-nêu), làm sau v1 | — |
| ~~Sync khi hai máy không cùng online~~ | **Vào scope 2026-08-17** — hoá ra đây là ca dùng **chính**, không phải ca biên: mac ở công ty, nixos ở nhà. Hộp thư R2 giữ item tới khi máy kia bật ([ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md), [R8 đã đóng](RISKS.md#r8--mất-item-khi-hai-máy-lệch-giờ-online--đóng-2026-08-17)) | — |
| ~~Relay server riêng~~ | **Không cần dựng** — R2 là hộp thư sẵn có, dùng qua S3 API, không có Worker nào chở nội dung. Endpoint `auth` ở [ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md) **không** phải relay: nó cấp quyền, không thấy nội dung | — |
| Sync file / thư mục | Đó là Syncthing, không phải clipboard | Không dự kiến |
| Đồng bộ định dạng rich text (RTF/HTML) | Plain text + ảnh phủ gần hết. Rich text mỗi OS một kiểu | Paste mất format gây khó chịu thật |

## 5. Ràng buộc

- **Hai OS bất đối xứng**: macOS không có event clipboard, Linux thì có. Chi tiết ở [ARCHITECTURE.md](ARCHITECTURE.md).
- **Máy NixOS chạy X11**, không DE. Clipboard X11 là *owner-based*: process ghi phải còn sống. (Wayland từng là rủi ro cao nhất — [R1](RISKS.md#r1--wayland-compositor-không-cho-đọc-clipboard--đóng-2026-08-17) đã đóng.)
- **Clipboard chứa dữ liệu nhạy cảm** (password, token). Ràng buộc bảo mật ở [NFR.md § Bảo mật](NFR.md#4-bảo-mật).
- **Hai máy ít khi bật cùng lúc** (mac ở công ty, nixos ở nhà). Đây là ràng buộc nền, không phải ca biên: mọi thiết kế đòi "cả hai online" đều không dùng được ([ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md)).
- **Nội dung clipboard nằm tạm trên hạ tầng của người khác** (Cloudflare R2). Kéo theo: mã hoá tầng app là **bắt buộc**, và Cloudflare thấy metadata — số item, thời điểm, kích thước ([NFR N18, N24b](NFR.md#4-bảo-mật)).
- **Tailscale là tuỳ chọn**, chỉ để bớt trễ. Không có nó thì sync vẫn chạy qua poll, chỉ chậm hơn ([N1b](NFR.md#1-ngưỡng-chấp-nhận)).
- **Người dùng phải nhớ một passphrase.** Đây là cái giá của việc server không đọc được clipboard. Quên = mất hết lịch sử đã sync, không có đường khôi phục ([ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md)).

## 6. Thước đo thành công

Đây là app cá nhân, không có DAU/retention. Thành công = ba điều kiện:

1. Người dùng **ngừng** gửi text cho chính mình qua chat để chuyển máy.
2. Đạt hết ngưỡng ở [NFR.md § Ngưỡng chấp nhận](NFR.md#1-ngưỡng-chấp-nhận).
3. Chạy nền một tuần liền không cần restart tay.

## 7. Yêu cầu chức năng

Đây là danh sách rút gọn để tra cứu. Acceptance criteria đầy đủ ở [USER-STORIES.md](USER-STORIES.md).

| ID | Yêu cầu | Ưu tiên |
|---|---|---|
| FR1 | Theo dõi clipboard và phát hiện nội dung mới | Must |
| FR2 | Đẩy nội dung mới vào hộp thư của máy còn lại, **kể cả khi máy đó đang tắt** | Must |
| FR3 | Lấy từ hộp thư và ghi vào clipboard local. Nhiều item một lúc → chỉ item **mới nhất** vào clipboard, cả lô vào lịch sử | Must |
| FR4 | Không tạo vòng lặp echo | Must |
| FR5 | Lưu lịch sử, tìm kiếm được | Must |
| FR6 | Ghim (pin) item để không bị xoá tự động | Must |
| FR7 | Click item trong lịch sử → copy lại | Must |
| FR8 | Hỗ trợ ảnh | Must |
| FR9 | Global hotkey mở cửa sổ lịch sử | Must |
| FR10 | Tray icon hiện trạng thái kết nối | Must |
| FR11 | Xoá item khỏi lịch sử | Must |
| FR12 | Tự chạy khi đăng nhập | Should |
| FR13 | Bỏ qua item được đánh dấu nhạy cảm (password manager) | Should |
| FR14 | Cấu hình được: bucket + endpoint, tên máy, poll interval clipboard và hộp thư, giới hạn dung lượng, peer chuông | Should |
| FR15 | Tạm dừng sync | Could |
| FR16 | Đăng nhập trên máy mới → lấy được quyền vào hộp thư, không nhập access key tay | Must (sau v1) |
| FR17 | Nhập passphrase một lần mỗi máy → dẫn xuất khoá giải mã, lưu trong keychain của OS | Must (sau v1) |
| FR18 | Xem danh sách máy đã đăng nhập, thu hồi một máy | Should (sau v1) |
| FR19 | Đăng xuất: xoá token + khoá khỏi máy này, **giữ nguyên** lịch sử local | Should (sau v1) |

## 8. Phát hành

**Đổi 2026-08-17:** repo đã public, và có release công khai qua **GitHub Release** (tag `v*`). Nhưng "công khai" chỉ là *tải về được*, không kèm gì khác:

| Có | Không có |
|---|---|
| Repo public, source đọc được | Không lên store nào (App Store, Homebrew, nixpkgs) |
| `.dmg` cho macOS attach vào Release | **Không notarize** — không có Apple Developer account. Máy mac tải về bị Gatekeeper chặn, phải `xattr -dr com.apple.quarantine` một lần. Đây là hệ quả cố ý, không phải bug |
| NixOS cài bằng `nix build github:nguyenhuy158/x2clip` | Không upload binary Linux precompiled — flake tự build, binary rời thường không chạy trên NixOS (linker path) |
| — | Không hỗ trợ, không nhận bug report, không cam kết tương thích. Vẫn là app một người dùng ([§2](#2-người-dùng)) |

"Ship" vẫn là: chạy được trên máy của chính chủ. Release chỉ để khỏi build tay khi cài lại máy.

Đóng gói chi tiết ở [ROADMAP.md](ROADMAP.md) Phase 5.

## 9. Câu hỏi mở

| # | Câu hỏi | Chặn gì |
|---|---|---|
| ~~Q1~~ | **Compositor / desktop trên NixOS là gì?** → **X11**, không có DE (`XDG_SESSION_TYPE=x11`, `XDG_CURRENT_DESKTOP` rỗng). Không phải lo `wlr-data-control`; `arboard` dùng backend X11. Quyết định: **giữ X11**, không chuyển sang GNOME/Wayland (GNOME không implement `data_control`). | ✅ trả lời 2026-08-17 |
| ~~Q2~~ | Tailscale đã cài trên cả hai máy chưa? → **Rồi.** `nixos` ↔ `macbook`, `tailscale ping` = pong 6ms, kết nối trực tiếp qua LAN (không qua DERP). | ✅ trả lời 2026-08-17 |
| ~~Q3~~ | Duyệt thứ tự phase, hay muốn UI sớm hơn để thấy hình trước? → **Làm tài liệu thiết kế UI trước khi code**: [UI/WIREFRAMES.md](UI/WIREFRAMES.md), [UI/MOCKUPS.md](UI/MOCKUPS.md), [UI/PROTOTYPE.md](UI/PROTOTYPE.md) + bản bấm thử. Thứ tự phase **giữ nguyên** (Phase 1 CLI trước, UI ở Phase 4) — tài liệu thiết kế là artifact trước Phase 1, không phải phase mới. | ✅ trả lời 2026-08-17 |
