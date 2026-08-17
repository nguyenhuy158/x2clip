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
";

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
                self.conn
                    .execute("UPDATE items SET updated_at = ?1 WHERE id = ?2", params![now, id])?;
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

    /// N14 — cắt theo tuổi rồi theo số lượng. Item đã ghim **không bao giờ** bị
    /// đụng tới (US-B4, T4).
    ///
    /// Tuổi tính từ **lần dùng cuối** (`updated_at`), không phải lúc tạo: copy
    /// lại một snippet cũ là làm nó trẻ lại. Cố ý lệch với chữ nghĩa của N14.
    pub fn prune(&self) -> Result<usize> {
        let cutoff = now_ms() - MAX_AGE_MS;
        let mut n = self
            .conn
            .execute("DELETE FROM items WHERE pinned = 0 AND updated_at < ?1", [cutoff])?;
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

    pub fn count(&self) -> Result<i64> {
        Ok(self.conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))?)
    }
}
