# x2clip — Plan

Clipboard sync + history manager cho macOS và NixOS (Linux). Một codebase.

---

## 1. Quyết định đã chốt

| Vấn đề | Chọn | Lý do |
|---|---|---|
| Transport | **Tailscale** | Đóng đúng vai trò iCloud/CloudKit của CleanClip/Paste: identity + mã hoá + xuyên NAT do người khác lo. Không backend, không deploy, không quản lý key. |
| Nội dung | **Text + ảnh** | Theo yêu cầu. Text làm trước, ảnh là phase riêng. |
| App shell | **Tauri v2** | Một codebase → `.app` cho macOS, binary/flake cho NixOS. Rust core + web UI. |
| Clipboard | **crate `arboard`** | Đã cover macOS + X11 + Wayland, cả text lẫn ảnh. Không tự viết 2 adapter. |
| Lưu trữ | **SQLite** (`rusqlite`) | History cần search + pin. File đơn, không server. |
| UI | **Vite + TypeScript, không framework** | v1 chỉ là list + ô search + pin. Thêm framework khi nào thật sự chật. |

### Không làm (và khi nào thì làm)
- **Relay server riêng (Cloudflare DO)** — chỉ cần nếu sau này muốn chạy trên máy không cài được Tailscale, hoặc muốn sync khi hai máy không cùng online.
- **Sync lịch sử đầy đủ** — v1 chỉ sync *clipboard hiện tại*. History là local mỗi máy. Sync cả history cần conflict resolution, để sau nếu thực sự thiếu.
- **Mobile (iOS/Android)** — ngoài scope.
- **Máy thứ 3+** — thiết kế peer list nên không chặn, nhưng chỉ test 2 máy.
- **Mã hoá tầng app** — Tailscale (WireGuard) đã mã hoá. Thêm một lớp nữa là dư, *trừ khi* bỏ Tailscale.

---

## 2. Kiến trúc

```
┌──────────── máy A (macOS) ─────────────┐        ┌──────── máy B (NixOS) ────────┐
│  UI (webview: list, search, pin)       │        │  UI                            │
│         ↕ tauri ipc                    │        │         ↕                      │
│  ┌─ core (Rust) ──────────────────┐    │        │  ┌─ core ─────────────────┐    │
│  │  watcher: poll arboard 250ms   │    │        │  │  watcher               │    │
│  │  echo guard: last_written_hash │    │        │  │  echo guard            │    │
│  │  store: SQLite history         │    │        │  │  store                 │    │
│  │  peer: WebSocket               │◄───┼────────┼─►│  peer                  │    │
│  └────────────────────────────────┘    │        │  └────────────────────────┘    │
└────────────────────────────────────────┘  qua   └────────────────────────────────┘
                                        Tailscale
```

### Luồng dữ liệu
1. `watcher` poll clipboard mỗi 250ms → nếu hash khác lần trước → tạo `ClipItem`.
2. Nếu hash == `last_written_hash` → **bỏ qua** (đây là echo của chính mình).
3. Ghi vào SQLite → emit event lên UI → gửi tới mọi peer đang kết nối.
4. Nhận từ peer → set `last_written_hash` **trước** khi ghi clipboard → ghi clipboard → ghi SQLite.

`last_written_hash` là chỗ chống ping-pong vô tận. Không phải nice-to-have; thiếu nó app không chạy được.

### Vì sao poll cả hai bên
macOS NSPasteboard không có notification — bắt buộc poll `changeCount`. Linux có event thật (`wl-paste --watch`), nhưng `arboard` không expose. Poll 250ms cả hai bên: một code path, chi phí không đáng kể, và symmetric nên không có bug chỉ xảy ra ở một OS. Nếu sau này cần latency thấp hơn trên Linux thì thêm watch path riêng.

### Peer model
- Config `~/.config/x2clip/config.toml`: `peers = ["mac-huy", "nixos-huy"]` (tên MagicDNS của Tailscale).
- Mỗi node vừa **listen** (bind vào IP Tailscale, không phải `0.0.0.0`) vừa **dial** tất cả peer, reconnect với backoff.
- Không service discovery, không mDNS. 2 máy thì danh sách tay là đủ.
- Bind vào interface Tailscale là ranh giới tin cậy: không expose ra LAN hay internet.

### Protocol
WebSocket, mỗi message một JSON frame (ảnh: base64 hoặc binary frame — quyết ở Phase 3):
```json
{ "kind": "text", "hash": "…", "body": "…", "ts": 1234567890 }
{ "kind": "image/png", "hash": "…", "body": "<base64>", "ts": 1234567890 }
```

---

## 3. Cây file

```
x2clip/
├── PLAN.md
├── flake.nix                  # build + devShell cho aarch64-darwin + x86_64-linux
├── Cargo.toml                 # workspace
├── core/                      # lib thuần Rust, test được không cần UI
│   ├── src/lib.rs
│   ├── src/clip.rs            # arboard wrapper: read/write text+image, hash
│   ├── src/watcher.rs         # poll loop + echo guard
│   ├── src/store.rs           # SQLite: insert, search, pin, prune
│   ├── src/peer.rs            # WebSocket listen + dial + reconnect
│   └── src/config.rs
├── cli/                       # binary headless — chạy được trước khi có UI
│   └── src/main.rs
├── app/                       # Tauri v2
│   ├── src-tauri/             # tray, global hotkey, ipc commands
│   └── src/                   # index.html + main.ts + style.css
└── packaging/
    ├── com.x2clip.plist       # launchd (macOS)
    └── x2clip.service         # systemd user unit (Linux)
```

`core` là lib riêng để test logic sync bằng `cargo test`, không cần dựng GUI.

---

## 4. Các phase

Mỗi phase phải chạy được và có một kiểm chứng cụ thể trước khi qua phase sau.

### Phase 0 — Spike (làm trước tiên, có thể phá vỡ cả kế hoạch)
Ba thứ chưa xác nhận, kiểm bằng script nhỏ chứ không đoán:
1. **Compositor trên NixOS của bạn là gì** và nó có `wlr-data-control`/`ext-data-control` không. Đọc clipboard trên Wayland phụ thuộc compositor, *không* phụ thuộc distro. GNOME/Mutter là chỗ hay thiếu.
2. `arboard` đọc + ghi được text và ảnh trên đúng compositor đó.
3. Hai máy ping được nhau qua tên MagicDNS của Tailscale, mở được TCP port.

**Chốt:** nếu (1) hoặc (2) fail → phải fallback sang gọi `wl-copy`/`wl-paste` ngoài, hoặc dùng X11. Biết trước tốt hơn biết lúc đang viết UI.

### Phase 1 — Core local, một máy
`clip.rs` + `watcher.rs` + `store.rs` + `cli`.
- Poll clipboard, hash, chống trùng liên tiếp, ghi SQLite.
- `x2clip list` / `x2clip search <q>` in ra history.

**Kiểm chứng:** `cargo test` — test cho hash + echo guard (ghi clipboard rồi assert watcher *không* sinh item mới) + insert/search của store.

### Phase 2 — Sync text giữa 2 máy
`peer.rs` + `config.rs`, nối vào watcher.
- Listen trên IP Tailscale, dial peer, reconnect backoff.
- Text only.

**Kiểm chứng:** test hai instance trong một process (2 store tạm, 2 clipboard giả) — copy ở A phải hiện ở B, và **không ping-pong**: đúng 1 message mỗi chiều, không phải vô tận. Sau đó thử tay trên 2 máy thật.

### Phase 3 — Ảnh
- Đọc/ghi ảnh qua arboard, chuẩn hoá về PNG.
- Giới hạn dung lượng (mặc định 5MB, cấu hình được) — bỏ qua và log nếu vượt.
- Lưu blob vào SQLite, thumbnail cho UI.

**Kiểm chứng:** round-trip một PNG qua sync path, assert bytes decode lại ra ảnh cùng kích thước.

### Phase 4 — UI
Tauri v2: tray icon, global hotkey mở window, list history, search, pin, click để copy lại, xoá.

**Kiểm chứng:** chạy app thật trên cả hai OS, copy qua lại, xem list cập nhật.

### Phase 5 — Packaging
- macOS: `.app` bundle qua `tauri build`; launchd plist để tự chạy.
- NixOS: `flake.nix` (`flake-utils` cho cả `aarch64-darwin` + `x86_64-linux`), systemd user unit.

**Kiểm chứng:** `nix build` xong chạy được trên NixOS; `.app` mở được trên máy sạch.

> Phase 5 (nhất là Nix build cho Tauri: Rust + node deps + webkitgtk) thường mất công hơn cảm giác ban đầu. Nếu bị kẹt, fallback là `nix develop` + `cargo build` thủ công, đóng gói sạch sau.

---

## 5. Rủi ro

| Rủi ro | Mức | Xử lý |
|---|---|---|
| Wayland compositor không cho đọc clipboard | **Cao** | Phase 0 kiểm trước. Fallback: `wl-clipboard` ngoài, hoặc X11. |
| Echo loop | Cao nếu quên | `last_written_hash`, có test riêng ở Phase 2 |
| Nix build Tauri (webkitgtk, node deps) | Trung bình | Phase 5, fallback devShell thủ công |
| Ảnh format lệch giữa macOS và Wayland | Trung bình | Chuẩn hoá PNG một chiều duy nhất |
| Clipboard chứa password | Trung bình | Tailscale mã hoá sẵn; bind chỉ vào interface Tailscale. Cân nhắc: bỏ qua item từ password manager (macOS có flag `org.nspasteboard.ConcealedType`) |
| Hai máy không cùng online → mất item | Thấp | Chấp nhận. Tailscale là mạng, không phải kho. Cần bền vững thì mới tính relay. |

---

## 6. Định nghĩa "xong" (success criteria)

Không có số thì không kiểm chứng được. Đây là mức chấp nhận, không phải mục tiêu lý tưởng:

| Tiêu chí | Ngưỡng | Fail nghĩa là |
|---|---|---|
| Trễ sync text | < 1s từ lúc copy tới lúc paste được ở máy kia | Poll interval hoặc reconnect có vấn đề |
| Trễ sync ảnh 2MB | < 3s | Encode hoặc transport cần tối ưu |
| CPU lúc rảnh | < 1% mỗi máy | Poll loop viết sai (không nên decode payload nếu hash chưa đổi) |
| RAM | < 150MB | Cache ảnh không giới hạn |
| Echo loop | **0 message dư.** Copy 1 lần = đúng 1 message mỗi chiều | Bug chặn release, không phải bug tune sau |
| Mất item | 0 khi cả hai máy online | Reconnect làm rơi message trong lúc dial lại |

## 7. Non-functional budget

- **Poll interval** 250ms, cấu hình được. Chỉ hash phần metadata rẻ trước; chỉ đọc payload đầy đủ khi phát hiện đổi.
- **History**: giữ 1000 item hoặc 30 ngày, cái nào tới trước thì prune. Item đã pin **không bị prune**.
- **Ảnh**: tối đa 5MB/item, cấu hình được. Vượt thì bỏ qua + log, không cắt bớt (ảnh cắt dở tệ hơn không có ảnh).
- **Kích thước DB**: cảnh báo khi > 500MB.
- **Pin không giới hạn** — người dùng chủ động pin thì đó là ý định của họ.

## 8. Hành vi khi lỗi (nhìn từ người dùng)

Im lặng là bug tệ nhất của app sync — người dùng tưởng đã sync rồi mới phát hiện không có.

| Tình huống | App phải làm gì |
|---|---|
| Peer offline | Tray icon đổi trạng thái. History local vẫn chạy bình thường. |
| Tailscale down | Thông báo rõ "không kết nối được Tailscale", không phải "sync failed" chung chung |
| Ảnh vượt giới hạn | Item vẫn vào history local, đánh dấu "quá lớn, không sync" |
| Không đọc được clipboard (Wayland thiếu protocol) | Báo lỗi rõ **lúc khởi động**, không phải im lặng rồi không bao giờ sync |
| DB lỗi/corrupt | Không tự xoá. Báo lỗi, giữ file, cho người dùng quyết định. |
| Nhận payload không parse được | Bỏ qua item đó + log, không crash daemon |

## 9. Cần bạn xác nhận

1. **Compositor / desktop trên NixOS?** (Hyprland / Sway / GNOME / KDE / X11) — quyết định Phase 0 làm gì.
2. Tailscale đã cài trên cả hai máy chưa?
3. Duyệt thứ tự phase, hay muốn đảo (ví dụ UI sớm hơn để thấy hình trước)?
