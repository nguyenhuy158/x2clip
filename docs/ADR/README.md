# Architecture Decision Records — x2clip

Mỗi file là **một** quyết định kỹ thuật, kèm bối cảnh, phương án đã loại, và hệ quả.

## Quy tắc

- Đổi ý → **thêm ADR mới**, đánh ADR cũ là `Superseded by ADR-XXXX`. Không sửa lịch sử, không xoá.
- ADR trả lời **vì sao**. *Là gì* thuộc [ARCHITECTURE.md](../ARCHITECTURE.md), *bao nhiêu* thuộc [NFR.md](../NFR.md).
- Phần "Phương án đã loại" là phần giá trị nhất. Sáu tháng sau bạn sẽ hỏi lại đúng câu đó.

## Danh sách

| # | Quyết định | Trạng thái |
|---|---|---|
| [0001](0001-transport-tailscale.md) | Dùng Tailscale làm transport, không tự dựng relay | **Một phần Superseded by 0006** — phần "không tự dựng relay" còn hiệu lực |
| [0002](0002-app-shell-tauri.md) | Tauri v2 làm app shell | Accepted |
| [0003](0003-clipboard-arboard-polling.md) | Một crate clipboard chung + poll cả hai OS | Accepted (Phase 0 đã xác nhận) |
| [0004](0004-storage-sqlite-local-history.md) | SQLite, lịch sử local không sync | Accepted phần SQLite; **4b (không sync lịch sử) Superseded by 0006** |
| [0005](0005-no-app-layer-crypto.md) | Không mã hoá tầng app, dựa vào WireGuard | **Superseded by 0006** — mã hoá tầng app giờ bắt buộc; **phân phối khoá superseded by 0007** |
| [0006](0006-r2-mailbox-store-and-forward.md) | R2 làm hộp thư (store-and-forward), Tailscale làm chuông | Accepted; **cách phát access key superseded by 0007** |
| [0007](0007-dang-nhap-va-khoa-tu-passphrase.md) | Đăng nhập để lấy quyền, khoá dẫn xuất từ passphrase | Accepted (làm sau v1) |

## Trạng thái

- **Proposed** — đang cân nhắc
- **Accepted** — đang áp dụng
- **Superseded** — đã bị ADR khác thay, giữ lại để tra lịch sử
