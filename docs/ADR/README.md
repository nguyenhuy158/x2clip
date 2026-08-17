# Architecture Decision Records — x2clip

Mỗi file là **một** quyết định kỹ thuật, kèm bối cảnh, phương án đã loại, và hệ quả.

## Quy tắc

- Đổi ý → **thêm ADR mới**, đánh ADR cũ là `Superseded by ADR-XXXX`. Không sửa lịch sử, không xoá.
- ADR trả lời **vì sao**. *Là gì* thuộc [ARCHITECTURE.md](../ARCHITECTURE.md), *bao nhiêu* thuộc [NFR.md](../NFR.md).
- Phần "Phương án đã loại" là phần giá trị nhất. Sáu tháng sau bạn sẽ hỏi lại đúng câu đó.

## Danh sách

| # | Quyết định | Trạng thái |
|---|---|---|
| [0001](0001-transport-tailscale.md) | Dùng Tailscale làm transport, không tự dựng relay | Accepted |
| [0002](0002-app-shell-tauri.md) | Tauri v2 làm app shell | Accepted |
| [0003](0003-clipboard-arboard-polling.md) | Một crate clipboard chung + poll cả hai OS | Accepted, **chờ Phase 0 xác nhận** |
| [0004](0004-storage-sqlite-local-history.md) | SQLite, lịch sử local không sync | Accepted |
| [0005](0005-no-app-layer-crypto.md) | Không mã hoá tầng app, dựa vào WireGuard | Accepted, **có điều kiện** |

## Trạng thái

- **Proposed** — đang cân nhắc
- **Accepted** — đang áp dụng
- **Superseded** — đã bị ADR khác thay, giữ lại để tra lịch sử
