# User Stories & Acceptance Criteria — x2clip

> Nguồn chính cho: story + điều kiện chấp nhận. Yêu cầu chức năng gốc (FR*) ở [PRD.md § 7](PRD.md#7-yêu-cầu-chức-năng). Con số ngưỡng ở [NFR.md](NFR.md).

Chỉ có một persona: **tôi** — chủ máy, dev, dùng cả macOS và NixOS. Nên story viết ngắn, không diễn giải persona.

Cột *Phase* trỏ sang [ROADMAP.md](ROADMAP.md).

---

## Epic A — Sync clipboard giữa hai máy

### US-A1 · Copy ở máy này, paste ở máy kia
**FR:** FR1, FR2, FR3 · **Phase:** 2 · **Ưu tiên:** Must

> Là người dùng, tôi muốn nội dung vừa copy ở máy A xuất hiện trong clipboard máy B, để paste luôn mà không phải gửi qua chat.

**Acceptance criteria**
- Given cả hai máy đang chạy app và kết nối được nhau
  When tôi copy một đoạn text ở máy A
  Then trong vòng thời gian ở [NFR § trễ text](NFR.md#1-ngưỡng-chấp-nhận), paste ở máy B ra đúng đoạn text đó
- Given tôi copy ở máy B
  Then chiều ngược lại hoạt động y hệt (sync hai chiều, không phải một chiều)
- Given nội dung có ký tự Unicode, emoji, xuống dòng, tab
  Then nội dung nhận được giống **byte-for-byte**, không bị chuẩn hoá hay cắt
- Given nội dung là chuỗi rỗng
  Then app bỏ qua, không tạo item lịch sử và không gửi đi

### US-A2 · Không có vòng lặp echo
**FR:** FR4 · **Phase:** 2 · **Ưu tiên:** Must — **chặn release**

> Là người dùng, tôi muốn app không tự đấu clipboard với chính nó, để máy không treo và lịch sử không đầy rác.

Hai đầu vừa theo dõi vừa ghi clipboard → nếu không chặn, mỗi lần copy sẽ sinh vòng lặp vô tận. Đây là lỗi kinh điển của mọi project clipboard sync.

**Acceptance criteria**
- Given tôi copy một lần ở máy A
  When sync hoàn tất
  Then đúng **một** message được gửi A→B và **không có** message nào quay lại B→A
- Given máy B ghi giá trị nhận được vào clipboard local
  Then watcher của B **không** coi đó là nội dung mới của người dùng
- Given tôi copy lại **đúng cùng** nội dung đó một lần nữa (chủ động)
  Then app vẫn nhận đó là hành động mới và cập nhật thời điểm — nhưng không gửi lặp vô hạn
- Có automated test khẳng định số message là hữu hạn và bằng 1 mỗi chiều

### US-A3 · Đồng bộ ảnh
**FR:** FR8 · **Phase:** 3 · **Ưu tiên:** Must

> Là người dùng, tôi muốn copy screenshot ở máy này và paste được ở máy kia, vì phần lớn nhu cầu chuyển chéo là ảnh.

**Acceptance criteria**
- Given tôi copy một ảnh ở máy A
  Then paste ở máy B ra ảnh cùng kích thước pixel, nội dung nhìn như nhau
- Given ảnh vượt giới hạn dung lượng ở [NFR § Giới hạn](NFR.md#3-giới-hạn)
  Then item vẫn vào lịch sử local, đánh dấu "quá lớn, không sync", và **không** bị cắt bớt
- Given ảnh copy từ macOS (thường là TIFF)
  Then được chuẩn hoá về PNG trước khi gửi, và máy Linux paste ra được
- Given clipboard chứa cả text và ảnh cùng lúc
  Then app chọn ảnh, và quyết định này ổn định (không lúc này lúc khác)

### US-A4 · Tạm dừng sync
**FR:** FR15 · **Phase:** 4 · **Ưu tiên:** Could

> Là người dùng, tôi muốn tắt sync tạm thời khi đang làm việc với dữ liệu không muốn rời máy.

**Acceptance criteria**
- Given tôi bật "tạm dừng" từ tray
  When tôi copy nội dung
  Then không có gì được gửi đi, nhưng lịch sử local vẫn ghi
- Given đang tạm dừng
  Then tray icon hiện rõ trạng thái đó, không im lặng
- Tạm dừng **không** tự bật lại sau khi restart app (trạng thái sau restart phải là trạng thái người dùng chọn gần nhất)

---

## Epic B — Lịch sử clipboard

### US-B1 · Lịch sử được lưu lại
**FR:** FR5 · **Phase:** 1 · **Ưu tiên:** Must

> Là người dùng, tôi muốn thấy lại những gì đã copy, vì clipboard OS chỉ giữ một giá trị.

**Acceptance criteria**
- Given tôi copy nhiều nội dung khác nhau
  Then tất cả xuất hiện trong lịch sử, mới nhất lên đầu
- Given tôi copy **hai lần liền** cùng một nội dung
  Then lịch sử chỉ có một entry, thời điểm được cập nhật (không sinh entry trùng liên tiếp)
- Given app restart
  Then lịch sử vẫn còn (lưu trên đĩa, không phải trong RAM)
- Given lịch sử vượt hạn mức ở [NFR § Giới hạn](NFR.md#3-giới-hạn)
  Then item cũ nhất **chưa ghim** bị xoá tự động

### US-B2 · Tìm trong lịch sử
**FR:** FR5 · **Phase:** 1 (CLI) → 4 (UI) · **Ưu tiên:** Must

> Là người dùng, tôi muốn tìm nội dung đã copy hôm qua bằng cách gõ một từ khoá.

**Acceptance criteria**
- Given tôi gõ từ khoá
  Then chỉ hiện item text chứa từ khoá đó, mới nhất lên đầu
- Tìm kiếm **không** phân biệt hoa thường
- Given không có kết quả
  Then hiện thông báo rõ ràng, không phải danh sách trống không giải thích
- Given lịch sử ở mức hạn mức tối đa
  Then tìm kiếm vẫn phản hồi dưới ngưỡng ở [NFR](NFR.md#1-ngưỡng-chấp-nhận)

### US-B3 · Dùng lại một item
**FR:** FR7 · **Phase:** 4 · **Ưu tiên:** Must

> Là người dùng, tôi muốn click một item cũ để nó vào clipboard, rồi paste bình thường.

**Acceptance criteria**
- Given tôi click một item text
  Then clipboard local chứa đúng nội dung đó và paste được ngay
- Given tôi click một item ảnh
  Then clipboard chứa ảnh đó, paste được vào app khác
- Given tôi click một item
  Then hành động này **cũng** sync sang máy kia (dùng lại là một hành động copy)
- Cửa sổ tự đóng sau khi click, để paste được ngay mà không phải tắt tay

### US-B4 · Ghim item
**FR:** FR6 · **Phase:** 4 · **Ưu tiên:** Must

> Là người dùng, tôi muốn giữ vài nội dung dùng thường xuyên để chúng không bị xoá tự động.

**Acceptance criteria**
- Given tôi ghim một item
  Then nó **không bao giờ** bị xoá bởi cơ chế prune tự động
- Item đã ghim hiện thành nhóm riêng hoặc có dấu hiệu nhìn thấy rõ
- Số lượng ghim không giới hạn — người dùng chủ động ghim thì đó là ý định của họ
- Bỏ ghim thì item quay lại luồng prune bình thường

### US-B5 · Xoá item
**FR:** FR11 · **Phase:** 4 · **Ưu tiên:** Must

> Là người dùng, tôi muốn xoá nội dung nhạy cảm khỏi lịch sử ngay lập tức.

**Acceptance criteria**
- Given tôi xoá một item
  Then nó mất khỏi lịch sử **local** ngay và không quay lại sau restart
- Có cách xoá **toàn bộ** lịch sử trong một hành động
- Xoá toàn bộ phải có bước xác nhận (không hoàn tác được)
- Rõ ràng với người dùng: xoá chỉ tác động máy này, không xoá ở máy kia (vì lịch sử là local — [PRD § Ngoài scope](PRD.md#4-ngoài-scope))

---

## Epic C — Vận hành hằng ngày

### US-C1 · Mở lịch sử bằng phím tắt
**FR:** FR9 · **Phase:** 4 · **Ưu tiên:** Must

> Là người dùng, tôi muốn mở lịch sử bằng một phím tắt toàn cục, vì phải rời chuột đi tìm cửa sổ là mất mục đích.

**Acceptance criteria**
- Phím tắt hoạt động khi app **không** đang focus
- Cửa sổ mở ra với ô tìm kiếm đã sẵn con trỏ — gõ được ngay
- `Esc` đóng cửa sổ, không thay đổi clipboard
- Given phím tắt đã bị app khác chiếm
  Then app báo lỗi rõ lúc khởi động, không im lặng bỏ qua
- Phím tắt đổi được trong config

### US-C2 · Biết được sync có đang chạy hay không
**FR:** FR10 · **Phase:** 4 · **Ưu tiên:** Must

> Là người dùng, tôi muốn nhìn là biết sync còn sống, vì tưởng đã sync xong mà thật ra không có là lỗi tệ nhất của app loại này.

**Acceptance criteria**
- Tray icon phân biệt được ít nhất: **đã kết nối** / **mất kết nối** / **tạm dừng**
- Given máy kia offline
  Then trạng thái đổi trong vòng thời gian ở [NFR](NFR.md#1-ngưỡng-chấp-nhận), và lịch sử local **vẫn chạy bình thường**
- Given Tailscale không chạy
  Then thông báo nói rõ là Tailscale, không phải "sync failed" chung chung
- Chi tiết mọi trường hợp lỗi ở [NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi)

### US-C3 · Tự chạy khi đăng nhập
**FR:** FR12 · **Phase:** 5 · **Ưu tiên:** Should

> Là người dùng, tôi không muốn phải nhớ bật app mỗi lần khởi động máy.

**Acceptance criteria**
- Given tôi restart máy và đăng nhập
  Then app tự chạy nền, không hiện cửa sổ
- Chạy được trên cả macOS (launchd) và Linux (systemd user unit)
- Bật/tắt tự chạy được, không phải sửa file hệ thống bằng tay
- Given app crash
  Then được restart tự động (đây là daemon, không phải app mở một lần)

### US-C4 · Không lưu password
**FR:** FR13 · **Phase:** 3 · **Ưu tiên:** Should

> Là người dùng, tôi không muốn password từ password manager bị lưu vào lịch sử hay gửi qua mạng.

**Acceptance criteria**
- Given nội dung được đánh dấu nhạy cảm (macOS: `org.nspasteboard.ConcealedType`)
  Then **không** lưu lịch sử và **không** gửi đi
- Given Linux không có dấu hiệu tương đương
  Then giới hạn này được ghi rõ trong tài liệu, không giả vờ là đã xử lý
- Bối cảnh bảo mật ở [NFR § Bảo mật](NFR.md#4-bảo-mật)

### US-C5 · Cấu hình được
**FR:** FR14 · **Phase:** 2 · **Ưu tiên:** Should

> Là người dùng, tôi muốn sửa peer, poll interval, giới hạn dung lượng mà không phải build lại app.

**Acceptance criteria**
- Config là một file text ở đường dẫn chuẩn của từng OS
- Given file config không tồn tại
  Then app tạo file mặc định và chạy được, không crash
- Given config sai cú pháp
  Then báo lỗi chỉ rõ dòng nào, và **không** ghi đè file của người dùng bằng bản mặc định
- Đổi peer list thì chỉ cần restart app, không cần cài lại

---

## Epic D — Thêm máy mới

Thêm 2026-08-17 cùng [PRD G6](PRD.md#3-mục-tiêu). Cả epic này nằm **sau v1** — Phase 1–5 không đụng tới. Cơ sở kỹ thuật ở [ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md).

### US-D1 · Đăng nhập trên máy mới
**FR:** FR16, FR17 · **Phase:** 6 · **Ưu tiên:** Must

> Là người dùng, tôi muốn cài app lên máy thứ ba rồi **đăng nhập** là có lịch sử, không phải scp file khoá từ máy cũ sang.

**Acceptance criteria**
- Given tôi cài app lần đầu trên một máy mới
  Then app hỏi đăng nhập, rồi hỏi passphrase — **đúng hai bước, không có bước chép file nào**
- Given tôi nhập đúng passphrase
  Then item từ máy khác giải mã được và vào lịch sử máy này
- Given tôi nhập sai passphrase
  Then app nói **"passphrase không khớp — không giải mã được hộp thư"**, không nói "đăng nhập thất bại" ([N18i](NFR.md#4-bảo-mật))
- Passphrase chỉ hỏi **một lần mỗi máy** — sau đó khoá nằm trong Keychain / file `0600` ([N18h](NFR.md#4-bảo-mật))
- Given endpoint `auth` không tới được
  Then báo rõ "không thêm được máy lúc này", **không** im lặng treo

### US-D2 · Máy cũ không chết theo lỗi đăng nhập
**FR:** FR16 · **Phase:** 6 · **Ưu tiên:** Must

> Là người dùng, tôi không muốn một sự cố đăng nhập làm hỏng cái app tôi mở ba mươi lần mỗi ngày.

**Acceptance criteria**
- Given token hết hạn, hoặc mất mạng, hoặc endpoint `auth` chết
  Then cửa sổ lịch sử **vẫn mở được**, tìm kiếm vẫn chạy, dùng lại item vẫn chạy ([N33](NFR.md#6-khả-năng-vận-hành))
  And tray nói rõ "cần đăng nhập lại để sync", không phải một lỗi chung chung
- Given `auth` chết nhưng credential R2 còn hạn
  Then sync **vẫn chạy bình thường** — ràng buộc thiết kế, không phải may mắn

### US-D3 · Xem và thu hồi máy
**FR:** FR18 · **Phase:** 6 · **Ưu tiên:** Should

> Là người dùng, tôi muốn thấy máy nào đang đăng nhập, và bỏ được máy đã bán đi.

**Acceptance criteria**
- Màn hình cấu hình liệt kê từng máy + lần đăng nhập cuối
- Given tôi thu hồi một máy
  Then credential máy đó hết hiệu lực, máy đó không LIST/GET được nữa
  And **không** phải đổi passphrase, **không** phải làm gì trên các máy còn lại

### US-D4 · Đăng xuất không mất lịch sử
**FR:** FR19 · **Phase:** 6 · **Ưu tiên:** Should

> Là người dùng, tôi muốn đăng xuất mà không mất những gì đã lưu trên máy này.

**Acceptance criteria**
- Given tôi đăng xuất
  Then token và khoá bị xoá khỏi máy, sync dừng
  And **lịch sử local còn nguyên**, kể cả item đã ghim
- Given tôi đăng nhập lại bằng đúng passphrase cũ
  Then sync chạy tiếp, không sinh ra lịch sử trùng lặp

---

## Không có story cho

Đây là phần cố ý bỏ trống, để không ai tưởng là quên:

- Onboarding / wizard cài đặt — một người dùng, tự cài được
- ~~Đăng nhập, tài khoản~~ — **đã có, xem Epic D** (đổi 2026-08-17). Vẫn **không** có: đăng ký tài khoản, quên mật khẩu, nhiều tài khoản trên một máy ([ADR-0007 § Phương án đã loại](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md#phương-án-đã-loại))
- Chia sẻ clipboard cho người khác — [PRD § Ngoài scope](PRD.md#4-ngoài-scope)
- Sync lịch sử đầy đủ giữa hai máy — cùng chỗ trên
- Theme / tuỳ biến giao diện — chưa dùng thật thì chưa biết cần gì
