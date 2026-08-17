# Kiến trúc — x2clip

> Nguồn chính cho: thành phần, data flow, protocol, cây file. *Vì sao* chọn các công nghệ này ở [ADR/](ADR/). Con số ở [NFR.md](NFR.md).

---

## 1. Toàn cảnh

```
┌──────────── máy A (macOS) ─────────────┐        ┌──────── máy B (NixOS) ────────┐
│  UI (webview: list, search, pin)       │        │  UI                            │
│         ↕ tauri ipc                    │        │         ↕                      │
│  ┌─ core (Rust) ──────────────────┐    │        │  ┌─ core ─────────────────┐    │
│  │  watcher: poll clipboard 250ms │    │        │  │  watcher               │    │
│  │  echo guard: last_written_hash │    │        │  │  echo guard            │    │
│  │  store: SQLite history         │    │        │  │  store                 │    │
│  │  peer: WebSocket               │◄───┼────────┼─►│  peer                  │    │
│  └────────────────────────────────┘    │        │  └────────────────────────┘    │
└────────────────────────────────────────┘  qua   └────────────────────────────────┘
                                        Tailscale
```

Không có server. Hai node ngang hàng, cấu hình giống nhau, chỉ khác nội dung file config.

## 2. Thành phần

| Thành phần | Trách nhiệm | Không làm |
|---|---|---|
| `clip` | Đọc/ghi clipboard (text + ảnh), tính hash | Không biết gì về mạng hay DB |
| `watcher` | Poll `clip`, phát hiện đổi, giữ echo guard, phát event | Không tự ghi clipboard |
| `store` | SQLite: insert, search, pin, prune | Không biết gì về peer |
| `peer` | WebSocket listen + dial + reconnect + parse frame | Không chạm clipboard trực tiếp |
| `config` | Đọc/validate file config | — |
| `cli` | Binary headless, chạy được trước khi có UI | — |
| `app` | Tauri: tray, phím tắt, cửa sổ, IPC | Không chứa logic sync |

`core` là library thuần Rust để test được logic sync bằng `cargo test`, không cần dựng GUI. Xem [TEST-PLAN.md](TEST-PLAN.md).

## 3. Data flow

### Clipboard đổi ở local (người dùng copy)
1. `watcher` poll `clip` mỗi 250ms ([N13](NFR.md#3-giới-hạn)).
2. Tính hash. Nếu hash == hash lần trước → không có gì xảy ra.
3. **Nếu hash == `last_written_hash` → bỏ qua và xoá cờ.** Đây là echo của chính mình.
4. Nếu nội dung đánh dấu nhạy cảm ([N22](NFR.md#4-bảo-mật)) → bỏ qua hoàn toàn.
5. Nếu vượt giới hạn dung lượng → ghi vào `store` với cờ "không sync", dừng ở đây.
6. Ghi vào `store` → emit event cho UI → `peer` gửi tới mọi peer đang kết nối.

### Nhận từ peer
1. `peer` parse frame. Parse lỗi → log + bỏ qua item, không crash ([NFR § Hành vi khi lỗi](NFR.md#5-hành-vi-khi-lỗi)).
2. Nếu hash đã có trong `store` như item mới nhất → bỏ qua (đã đồng bộ).
3. **Set `last_written_hash` = hash — làm việc này TRƯỚC khi ghi clipboard.**
4. Ghi clipboard qua `clip`.
5. Ghi vào `store` → emit event cho UI.
6. **Không** phát lại cho peer khác.

### Echo guard — chỗ dễ sai nhất
Hai đầu vừa theo dõi vừa ghi clipboard. Thiếu bước 3 ở trên thì mỗi lần copy sinh vòng lặp vô tận: A ghi → watcher A thấy "nội dung mới" → gửi lại B → B ghi → …

`last_written_hash` là một ô nhớ đơn: hash của giá trị **mình vừa ghi**. Lần poll tiếp theo thấy đúng hash đó thì bỏ qua và xoá cờ.

Ràng buộc thứ tự **bắt buộc**: set cờ trước, ghi clipboard sau. Đảo lại là có race — watcher có thể poll đúng khoảnh khắc giữa hai bước.

Đây không phải tối ưu; thiếu nó app không dùng được. Có test riêng, xem [US-A2](USER-STORIES.md#us-a2--không-có-vòng-lặp-echo) và [N7](NFR.md#1-ngưỡng-chấp-nhận).

## 4. Vì sao poll cả hai bên

macOS NSPasteboard **không có** notification — bắt buộc poll `changeCount`. Linux có event thật (`wl-paste --watch`, hoặc X11 selection-owner notify), nhưng crate clipboard đang dùng không expose ([ADR-0003](ADR/0003-clipboard-arboard-polling.md)).

Chọn poll cả hai bên: một code path duy nhất, symmetric, nên không có class bug chỉ xảy ra ở một OS. Chi phí nằm trong [N9](NFR.md#2-tài-nguyên).

Nếu sau này Linux cần latency thấp hơn thì thêm watch path riêng cho Linux — nhưng echo guard vẫn dùng chung, đừng nhân đôi nó.

## 5. Peer model

- Config liệt kê peer bằng **tên MagicDNS** của Tailscale, ví dụ `peers = ["mac-huy", "nixos-huy"]`.
- Mỗi node vừa **listen** vừa **dial** tất cả peer trong danh sách, reconnect với exponential backoff (giới hạn bởi [N4](NFR.md#1-ngưỡng-chấp-nhận)).
- Listen socket bind **chỉ** vào địa chỉ Tailscale ([N19](NFR.md#4-bảo-mật)), không `0.0.0.0`.
- Chỉ nhận kết nối từ peer trong config ([N20](NFR.md#4-bảo-mật)).
- **Không** service discovery, không mDNS. Hai máy thì danh sách tay là đủ.

Vì cả hai đầu đều dial, sẽ có hai kết nối giữa một cặp máy. Chấp nhận: dedupe bằng hash ở tầng nhận là đủ, thêm leader election cho 2 node là dư. Nếu lên 3+ máy thì tính lại.

## 6. Protocol

WebSocket, mỗi message là một JSON frame.

```json
{ "v": 1, "kind": "text",      "hash": "<sha256 hex>", "body": "nội dung",  "ts": 1755400000000 }
{ "v": 1, "kind": "image/png", "hash": "<sha256 hex>", "body": "<base64>",  "ts": 1755400000000 }
```

| Field | Kiểu | Ghi chú |
|---|---|---|
| `v` | int | Version protocol. Lệch → cảnh báo rõ, không đoán ([N32](NFR.md#7-tương-thích)) |
| `kind` | `"text"` \| `"image/png"` | Ảnh luôn chuẩn hoá về PNG trước khi gửi |
| `hash` | string | SHA-256 hex của payload gốc. Dùng để dedupe và cho echo guard |
| `body` | string | Text: nguyên văn UTF-8. Ảnh: base64 của PNG bytes |
| `ts` | int | Epoch millisecond, thời điểm phát hiện ở máy gửi |

Ảnh dùng base64 trong JSON frame cho v1 — đơn giản, một code path. Nếu [N2](NFR.md#1-ngưỡng-chấp-nhận) không đạt thì đổi sang binary frame.

## 7. Lưu trữ

SQLite, một file, quyền `0600` ([N21](NFR.md#4-bảo-mật)).

Bảng `items`:

| Cột | Kiểu | Ghi chú |
|---|---|---|
| `id` | INTEGER PK | |
| `kind` | TEXT | `text` \| `image/png` |
| `hash` | TEXT, index | Dedupe |
| `body` | TEXT hoặc BLOB | Ảnh lưu blob, không base64 (base64 chỉ dùng khi truyền) |
| `thumb` | BLOB, nullable | Thumbnail cho UI, chỉ với ảnh |
| `created_at` | INTEGER | Epoch ms |
| `updated_at` | INTEGER | Copy lại cùng nội dung thì cập nhật cột này, không tạo row mới |
| `pinned` | INTEGER | 0/1. Đã ghim thì prune bỏ qua |
| `synced` | INTEGER | 0 nếu bị bỏ qua vì vượt giới hạn |

Index trên `hash` và `updated_at` để đạt [N5](NFR.md#1-ngưỡng-chấp-nhận).

Prune theo [N14](NFR.md#3-giới-hạn), **luôn** loại trừ `pinned = 1`.

## 8. Cây file

```
x2clip/
├── PLAN.md                    # index, trỏ sang docs/
├── docs/                      # tài liệu (file này nằm ở đây)
├── flake.nix                  # build + devShell cho aarch64-darwin + x86_64-linux
├── Cargo.toml                 # workspace
├── core/                      # lib thuần Rust, test không cần UI
│   └── src/
│       ├── lib.rs
│       ├── clip.rs            # đọc/ghi clipboard text+ảnh, hash
│       ├── watcher.rs         # poll loop + echo guard
│       ├── store.rs           # SQLite
│       ├── peer.rs            # WebSocket listen + dial + reconnect
│       └── config.rs
├── cli/
│   └── src/main.rs            # binary headless
├── app/                       # Tauri v2
│   ├── src-tauri/             # tray, phím tắt, IPC command
│   └── src/                   # index.html + main.ts + style.css
└── packaging/
    ├── com.x2clip.plist       # launchd (macOS)
    └── x2clip.service         # systemd user unit (Linux)
```

## 9. Chỗ khác nhau giữa hai OS

Chỉ được có ba chỗ. Nhiều hơn là dấu hiệu abstraction đang rò rỉ:

1. **`clip.rs`** — backend clipboard (`arboard` lo hết: macOS NSPasteboard + X11, đã đo ở [Phase 0.2](ROADMAP.md#kết-quả-02-2026-08-17)).
2. **Đánh dấu nhạy cảm** — macOS có, Linux không ([N22](NFR.md#4-bảo-mật)).
3. **File tự chạy** — launchd plist vs systemd user unit.
