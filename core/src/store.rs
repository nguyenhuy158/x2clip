//! SQLite store cho lịch sử clipboard. Schema theo docs/ARCHITECTURE.md § 7.

use crate::{now_ms, MAX_AGE_MS, MAX_ITEMS};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS items (
    id         INTEGER PRIMARY KEY,
    kind       TEXT    NOT NULL,
    hash       TEXT    NOT NULL UNIQUE,
    body       TEXT,
    thumb      BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    pinned     INTEGER NOT NULL DEFAULT 0,
    synced     INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_items_updated_at ON items(updated_at DESC);
CREATE TABLE IF NOT EXISTS seen (
    key TEXT PRIMARY KEY,
    at  INTEGER NOT NULL
);
";

/// Trạng thái của `items.synced`.
///
/// Lệch với chữ nghĩa của T7 (chỗ đó chỉ có 0/1): item nhận từ hộp thư phải
/// **không bao giờ** được PUT ngược lại, mà hàng chờ lại sống trong DB qua
/// restart (T16) nên cờ chống dội cũng phải nằm trong DB. Dùng chung số 0 thì
/// mọi item nhận về sẽ bị gửi lại ngay sau lần khởi động kế tiếp.
///
/// Item quá cỡ N16 không tới được đây: `Watcher::tick` chặn từ trước khi ghi DB.
pub const SYNC_CHO_GUI: i64 = 0;
pub const SYNC_DA_GUI: i64 = 1;
pub const SYNC_KHONG_GUI: i64 = 2;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: i64,
    pub kind: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        // N21 — file lịch sử chỉ chủ máy đọc được.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Self::init(conn)
    }

    pub fn open_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch(SCHEMA)?;
        // Cột PNG đầy đủ, thêm sau Phase 1 nên phải ALTER cho DB đã tồn tại.
        // Cố ý tách khỏi `body`: `list()` SELECT body của mọi row, để ảnh ở đó
        // là mỗi lần `x2clip list` nạp cả chục MB vào RAM (N10).
        // Chạy lần hai báo "duplicate column" — đúng như mong đợi, nuốt luôn.
        let _ = conn.execute("ALTER TABLE items ADD COLUMN blob BLOB", []);
        Ok(Self { conn })
    }

    /// Ghi một item text. Trùng hash thì chỉ cập nhật `updated_at`, không sinh
    /// row mới (US-B1, T5). Trả về `(id, là_item_mới)`.
    pub fn upsert_text(&self, hash: &str, body: &str) -> Result<(i64, bool)> {
        let now = now_ms();
        let existing: Option<i64> = self
            .conn
            .query_row("SELECT id FROM items WHERE hash = ?1", [hash], |r| r.get(0))
            .ok();
        match existing {
            Some(id) => {
                self.conn.execute(
                    "UPDATE items SET updated_at = ?1 WHERE id = ?2",
                    params![now, id],
                )?;
                Ok((id, false))
            }
            None => {
                self.conn.execute(
                    "INSERT INTO items (kind, hash, body, created_at, updated_at)
                     VALUES ('text', ?1, ?2, ?3, ?3)",
                    params![hash, body, now],
                )?;
                Ok((self.conn.last_insert_rowid(), true))
            }
        }
    }

    /// Ghi một item ảnh. `body` để NULL và preview đi vào `body` dạng chữ
    /// (`"ảnh 1920x1080"`) để `x2clip list` in được mà không phải đọc blob.
    ///
    /// `synced` truyền vào chứ không mặc định 0: ảnh vượt N15 phải vào lịch sử
    /// local với `SYNC_KHONG_GUI` (US-A3) — nếu để 0 nó sẽ nằm mãi trong hàng
    /// chờ, mỗi vòng poll lại thử PUT một lần rồi lại hỏng.
    pub fn upsert_image(
        &self,
        hash: &str,
        mo_ta: &str,
        png: &[u8],
        thumb: &[u8],
        synced: i64,
        ts: i64,
    ) -> Result<(i64, bool)> {
        let existing: Option<i64> = self
            .conn
            .query_row("SELECT id FROM items WHERE hash = ?1", [hash], |r| r.get(0))
            .ok();
        match existing {
            Some(id) => {
                self.conn.execute(
                    "UPDATE items SET updated_at = MAX(updated_at, ?1) WHERE id = ?2",
                    params![ts, id],
                )?;
                Ok((id, false))
            }
            None => {
                self.conn.execute(
                    "INSERT INTO items (kind, hash, body, blob, thumb, created_at, updated_at, synced)
                     VALUES ('image', ?1, ?2, ?3, ?4, ?5, ?5, ?6)",
                    params![hash, mo_ta, png, thumb, ts, synced],
                )?;
                Ok((self.conn.last_insert_rowid(), true))
            }
        }
    }

    /// PNG đầy đủ của một item ảnh. Chỉ gọi khi thật sự cần gửi đi — đây là
    /// chỗ duy nhất nạp cả bức ảnh vào RAM.
    pub fn lay_blob(&self, id: i64) -> Result<Option<Vec<u8>>> {
        Ok(self
            .conn
            .query_row("SELECT blob FROM items WHERE id = ?1", [id], |r| r.get(0))
            .ok()
            .flatten())
    }

    /// N14 — cắt theo tuổi rồi theo số lượng. Item đã ghim **không bao giờ** bị
    /// đụng tới (US-B4, T4).
    ///
    /// Tuổi tính từ **lần dùng cuối** (`updated_at`), không phải lúc tạo: copy
    /// lại một snippet cũ là làm nó trẻ lại. Cố ý lệch với chữ nghĩa của N14.
    pub fn prune(&self) -> Result<usize> {
        let cutoff = now_ms() - MAX_AGE_MS;
        let mut n = self.conn.execute(
            "DELETE FROM items WHERE pinned = 0 AND updated_at < ?1",
            [cutoff],
        )?;
        n += self.conn.execute(
            "DELETE FROM items WHERE pinned = 0 AND id NOT IN (
                 SELECT id FROM items WHERE pinned = 0 ORDER BY updated_at DESC, id DESC LIMIT ?1
             )",
            [MAX_ITEMS as i64],
        )?;
        Ok(n)
    }

    pub fn list(&self, limit: usize) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, COALESCE(body, ''), created_at, updated_at, pinned
             FROM items ORDER BY updated_at DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |r| {
            Ok(Item {
                id: r.get(0)?,
                kind: r.get(1)?,
                body: r.get(2)?,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
                pinned: r.get::<_, i64>(5)? != 0,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Tìm không phân biệt hoa thường (US-B2). Lọc trong Rust chứ không dùng
    /// `LIKE`: `lower()` của SQLite chỉ fold ASCII, tiếng Việt sẽ trượt.
    /// ponytail: quét tuyến tính — ở mức N14 = 1000 item thì thừa sức N5.
    pub fn search(&self, query: &str) -> Result<Vec<Item>> {
        let needle = query.to_lowercase();
        Ok(self
            .list(MAX_ITEMS)?
            .into_iter()
            .filter(|i| i.kind == "text" && i.body.to_lowercase().contains(&needle))
            .collect())
    }

    pub fn set_pinned(&self, id: i64, pinned: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE items SET pinned = ?1 WHERE id = ?2",
            params![pinned as i64, id],
        )?;
        Ok(())
    }

    /// Item nhận từ hộp thư: lấy `ts` của bên gửi làm mốc thời gian để lịch
    /// sử hai máy xếp cùng thứ tự, và gắn `SYNC_KHONG_GUI` để **không bao giờ
    /// PUT ngược lại** — chốt này nằm trong DB nên sống qua cả restart.
    pub fn upsert_remote(&self, hash: &str, body: &str, ts: i64) -> Result<(i64, bool)> {
        let existing: Option<i64> = self
            .conn
            .query_row("SELECT id FROM items WHERE hash = ?1", [hash], |r| r.get(0))
            .ok();
        match existing {
            Some(id) => {
                self.conn.execute(
                    "UPDATE items SET updated_at = MAX(updated_at, ?1) WHERE id = ?2",
                    params![ts, id],
                )?;
                Ok((id, false))
            }
            None => {
                self.conn.execute(
                    "INSERT INTO items (kind, hash, body, created_at, updated_at, synced)
                     VALUES ('text', ?1, ?2, ?3, ?3, ?4)",
                    params![hash, body, ts, SYNC_KHONG_GUI],
                )?;
                Ok((self.conn.last_insert_rowid(), true))
            }
        }
    }

    /// Hàng chờ PUT, cũ trước. Nằm trong DB nên sống qua restart (T16).
    pub fn cho_gui(&self) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, COALESCE(body, ''), created_at, updated_at, pinned
             FROM items WHERE synced = ?1 ORDER BY updated_at ASC, id ASC",
        )?;
        let rows = stmt.query_map([SYNC_CHO_GUI], |r| {
            Ok(Item {
                id: r.get(0)?,
                kind: r.get(1)?,
                body: r.get(2)?,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
                pinned: r.get::<_, i64>(5)? != 0,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn dat_synced(&self, id: i64, state: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE items SET synced = ?1 WHERE id = ?2",
            params![state, id],
        )?;
        Ok(())
    }

    pub fn da_thay(&self, key: &str) -> Result<bool> {
        Ok(self
            .conn
            .query_row("SELECT 1 FROM seen WHERE key = ?1", [key], |_| Ok(()))
            .is_ok())
    }

    /// Ghi nhận đã xử lý object này. Chặn xử lý lại kể cả khi DELETE hỏng (T13).
    pub fn ghi_nhan_da_thay(&self, key: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO seen (key, at) VALUES (?1, ?2)",
            params![key, now_ms()],
        )?;
        Ok(())
    }

    /// Cùng cửa sổ 30 ngày với lifecycle rule của bucket — object đã bị R2
    /// xoá thì không cần nhớ nữa.
    pub fn prune_seen(&self) -> Result<usize> {
        Ok(self
            .conn
            .execute("DELETE FROM seen WHERE at < ?1", [now_ms() - MAX_AGE_MS])?)
    }

    pub fn count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))?)
    }
}
