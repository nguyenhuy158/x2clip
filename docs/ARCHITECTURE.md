# Kiến trúc — x2clip

> Nguồn chính cho: thành phần, data flow, protocol, cây file. *Vì sao* chọn các công nghệ này ở [ADR/](ADR/). Con số ở [NFR.md](NFR.md).

---

## 1. Toàn cảnh

Hai máy **ít khi online cùng lúc** (mac ở công ty, nixos ở nhà) → nội dung đi qua **hộp thư R2**, không đi trực tiếp. Xem [ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md).

```
┌───────── máy A (macOS, công ty) ───────┐         ┌────── máy B (NixOS, nhà) ──────┐
│  UI (webview: list, search, pin)       │         │  UI                            │
│         ↕ tauri ipc                    │         │         ↕                      │
│  ┌─ core (Rust) ──────────────────┐    │         │  ┌─ core ─────────────────┐    │
│  │  watcher: poll clipboard 250ms │    │         │  │  watcher               │    │
│  │  echo guard: last_written_hash │    │         │  │  echo guard            │    │
│  │  store: SQLite (nguồn chân lý) │    │         │  │  store                 │    │
│  │  crypto: mã hoá / giải mã      │    │         │  │  crypto                │    │
│  │  mailbox: PUT/LIST/GET/DELETE  │    │         │  │  mailbox               │    │
│  │  notify: WebSocket (tuỳ chọn)  │    │         │  │  notify                │    │
│  └───────────┬────────────────────┘    │         │  └──────────┬─────────────┘    │
└──────────────┼─────────────────────────┘         └─────────────┼──────────────────┘
               │        ┌──────────────────────┐                 │
   ciphertext  └──PUT──►│    Cloudflare R2     │◄───LIST/GET─────┘ poll 30s
                        │  inbox/<máy>/<ulid>  │                   + lúc vừa bật
                        └──────────────────────┘
        ╌╌╌ "có item mới, key=X" (~100B, chỉ khi cả hai cùng online) ╌╌╌
                    qua Tailscale — mất cũng chỉ chậm hơn
```

Không có server **của mình**. Hai node ngang hàng, cấu hình giống nhau, chỉ khác tên máy trong config. R2 là hộp thư thụ động — không có code của mình chạy trên đó.

**Nội dung chỉ đi một đường (R2).** Tailscale chở thông báo, không chở nội dung → một chỗ mã hoá, một chỗ ingest, không cần dedupe giữa hai đường.

## 2. Thành phần

| Thành phần | Trách nhiệm | Không làm |
|---|---|---|
| `clip` | Đọc/ghi clipboard (text + ảnh), tính hash | Không biết gì về mạng hay DB |
| `watcher` | Poll `clip`, phát hiện đổi, giữ echo guard, phát event | Không tự ghi clipboard |
| `store` | SQLite: insert, search, pin, prune, sổ `seen` | Không biết gì về mạng |
| `crypto` | Mã hoá/giải mã payload bằng thư viện có sẵn ([ADR-0005 § Xem lại](ADR/0005-no-app-layer-crypto.md#xem-lại-2026-08-17--mã-hoá-tầng-app-thành-bắt-buộc)) | Không tự ghép primitive, không quản rotation (v1) |
| `mailbox` | R2 qua S3 API: PUT / LIST / GET / DELETE, retry, backoff | Không giải mã, không chạm clipboard |
| `notify` | WebSocket chở **object key**, listen + dial + reconnect | Không chở nội dung. Hỏng thì chỉ mất tốc độ |
| `config` | Đọc/validate file config | — |
| `auth` *(sau v1)* | Đăng nhập, giữ token, xin credential R2 tạm thời, dẫn xuất khoá từ passphrase ([ADR-0007](ADR/0007-dang-nhap-va-khoa-tu-passphrase.md)) | **Không** chở nội dung clipboard. **Không** gửi passphrase đi đâu ([N18g](NFR.md#4-bảo-mật)). Hỏng thì chỉ chặn thêm máy mới |
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
6. Ghi vào `store` → emit event cho UI.
7. `crypto` mã hoá → `mailbox` PUT lên `inbox/<máy kia>/<ulid>`.
8. Nếu `notify` đang có kết nối: bắn frame `{"key": "<object key>"}`. Không có kết nối → **không sao**, máy kia sẽ thấy ở lần poll tới.

PUT lỗi (mất mạng, R2 down) → item **vẫn nằm trong `store` local**, đưa vào hàng chờ gửi lại với backoff. Không mất item, không chặn clipboard.

### Nhận từ hộp thư

Chạy khi: tới chu kỳ poll ([N13b](NFR.md#3-giới-hạn), 30s), hoặc vừa bật máy/vừa có mạng, hoặc `notify` báo có key mới.

1. `mailbox` LIST prefix `inbox/<máy mình>/`. Rỗng → xong.
2. Với mỗi object: bỏ qua nếu key đã có trong sổ `seen` (idempotent — cùng object có thể tới hai lần nếu DELETE thất bại).
3. GET → `crypto` giải mã. **Giải mã lỗi → log + GIỮ object, không xoá, không ghi vào đâu** ([ADR-0006](ADR/0006-r2-mailbox-store-and-forward.md) C4).
4. Ghi vào `store` + ghi key vào `seen`.
5. DELETE object. DELETE lỗi → bỏ qua, `seen` đã chặn xử lý lại; lifecycle rule 30 ngày dọn nốt.
6. **Sau khi xử lý hết lô:** lấy item có `ts` lớn nhất → **set `last_written_hash` TRƯỚC** → ghi clipboard. Các item còn lại chỉ vào history.

Bước 6 là [ADR-0006 § 6c](ADR/0006-r2-mailbox-store-and-forward.md#6c--chỉ-item-mới-nhất-được-ghi-vào-clipboard): bật máy sau 8 tiếng có 30 item trong hộp thư, ghi hết thì clipboard cuối cùng lại là item cũ nhất trong lô.

7. **Không** PUT lại bất cứ thứ gì vừa nhận.

### Echo guard — chỗ dễ sai nhất
Hai đầu vừa theo dõi vừa ghi clipboard. Thiếu bước 3 ở trên thì mỗi lần copy sinh vòng lặp vô tận: A ghi → watcher A thấy "nội dung mới" → gửi lại B → B ghi → …

`last_written_hash` là một ô nhớ đơn: hash của giá trị **mình vừa ghi**. Lần poll tiếp theo thấy đúng hash đó thì bỏ qua và xoá cờ.

Ràng buộc thứ tự **bắt buộc**: set cờ trước, ghi clipboard sau. Đảo lại là có race — watcher có thể poll đúng khoảnh khắc giữa hai bước.

Đây không phải tối ưu; thiếu nó app không dùng được. Có test riêng, xem [US-A2](USER-STORIES.md#us-a2--không-có-vòng-lặp-echo) và [N7](NFR.md#1-ngưỡng-chấp-nhận).

## 4. Vì sao poll cả hai bên

macOS NSPasteboard **không có** notification — bắt buộc poll `changeCount`. Linux có event thật (`wl-paste --watch`, hoặc X11 selection-owner notify), nhưng crate clipboard đang dùng không expose ([ADR-0003](ADR/0003-clipboard-arboard-polling.md)).

Chọn poll cả hai bên: một code path duy nhất, symmetric, nên không có class bug chỉ xảy ra ở một OS. Chi phí nằm trong [N9](NFR.md#2-tài-nguyên).

Nếu sau này Linux cần latency thấp hơn thì thêm watch path riêng cho Linux — nhưng echo guard vẫn dùng chung, đừng nhân đôi nó.

## 5. Danh sách máy

- Config liệt kê **tên máy** (`mac`, `nixos`) — đây là `recipient` trong object key, không phải hostname mạng.
- Gửi = PUT vào `inbox/<tên máy kia>/`. Nhận = LIST `inbox/<tên mình>/`. Không cần biết máy kia có đang chạy hay không.
- **Không** service discovery. Hai máy thì danh sách tay là đủ.

Kênh thông báo (tuỳ chọn, [ADR-0006 § 6b](ADR/0006-r2-mailbox-store-and-forward.md#6b--tailscale-hạ-cấp-thành-kênh-thông-báo)) mới cần địa chỉ mạng: **tên MagicDNS** của Tailscale, mỗi node vừa listen vừa dial, reconnect backoff ([N4](NFR.md#1-ngưỡng-chấp-nhận)). Ở kênh đó vẫn bắt buộc bind **chỉ** vào địa chỉ Tailscale ([N19](NFR.md#4-bảo-mật)) và whitelist peer ([N20](NFR.md#4-bảo-mật)) — object key là thứ dùng để GET, để lộ là mời người khác đọc hộp thư.

Cả hai đầu đều dial → hai kết nối giữa một cặp máy. Chấp nhận: frame chỉ chở key, xử lý idempotent qua sổ `seen`, nên trùng là vô hại.

## 6. Định dạng object

Một object = một item. Body là **ciphertext nhị phân**, không phải JSON.

Plaintext trước khi mã hoá:

```json
{ "v": 1, "kind": "text",      "hash": "<sha256 hex>", "body": "nội dung", "ts": 1755400000000 }
{ "v": 1, "kind": "image/png", "hash": "<sha256 hex>", "body": "<base64>", "ts": 1755400000000 }
```

| Field | Kiểu | Ghi chú |
|---|---|---|
| `v` | int | Version. Lệch → cảnh báo rõ, không đoán ([N32](NFR.md#7-tương-thích)) |
| `kind` | `"text"` \| `"image/png"` | Ảnh luôn chuẩn hoá về PNG trước khi gửi |
| `hash` | string | SHA-256 hex của payload gốc. Dedupe + echo guard |
| `body` | string | Text: nguyên văn UTF-8. Ảnh: base64 của PNG bytes |
| `ts` | int | Epoch ms, thời điểm phát hiện ở máy gửi. Quyết định item nào được ghi clipboard |

`kind`, `hash`, `ts` nằm **trong** ciphertext, **không** làm object metadata — metadata trên R2 là plaintext.

Object key: `inbox/<recipient>/<ulid>`, ULID random. **Không** đưa `hash` vào key ([ADR-0005 C6](ADR/0005-no-app-layer-crypto.md#xem-lại-2026-08-17--mã-hoá-tầng-app-thành-bắt-buộc)).

Frame của kênh thông báo — chỉ có key, không có nội dung:

```json
{ "v": 1, "key": "inbox/nixos/01J8XK2..." }
```

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
| `synced` | INTEGER | 0 nếu bị bỏ qua vì vượt giới hạn, hoặc PUT chưa thành công |

Index trên `hash` và `updated_at` để đạt [N5](NFR.md#1-ngưỡng-chấp-nhận).

Bảng `seen` — sổ object key đã xử lý:

| Cột | Kiểu | Ghi chú |
|---|---|---|
| `key` | TEXT PK | Object key trên R2 |
| `at` | INTEGER | Epoch ms lúc xử lý xong |

Lý do tồn tại: DELETE trên R2 có thể fail sau khi đã ghi vào `store`. Không có `seen` thì lần poll sau xử lý lại object đó và **ghi đè clipboard hiện tại** bằng nội dung cũ. Prune cùng cửa sổ với lifecycle rule ([N14](NFR.md#3-giới-hạn), 30 ngày) — key cũ hơn thế thì object đã không còn tồn tại để tới lần hai.

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
│       ├── store.rs           # SQLite: items + seen
│       ├── crypto.rs          # mã hoá/giải mã payload (thư viện có sẵn)
│       ├── mailbox.rs         # R2 qua S3 API: PUT/LIST/GET/DELETE + retry
│       ├── notify.rs          # kênh chuông tuỳ chọn: WebSocket chở object key
│       ├── auth.rs            # (sau v1) đăng nhập, token, credential tạm, Argon2id
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
