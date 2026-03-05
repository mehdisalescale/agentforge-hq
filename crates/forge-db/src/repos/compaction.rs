//! Compaction CRUD repository — tracks context-window compaction events.

use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A persisted compaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compaction {
    pub id: String,
    pub session_id: String,
    pub summary: String,
    pub original_token_count: i64,
    pub compacted_token_count: i64,
    pub created_at: String,
}

pub struct CompactionRepo {
    conn: Arc<Mutex<Connection>>,
}

impl CompactionRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Create a new compaction record.
    pub fn create(
        &self,
        session_id: &str,
        summary: &str,
        original_tokens: i64,
        compacted_tokens: i64,
    ) -> ForgeResult<Compaction> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO compactions (id, session_id, summary, original_token_count, compacted_token_count)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, session_id, summary, original_tokens, compacted_tokens],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        // Re-read to get the server-generated created_at
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, summary, original_token_count, compacted_token_count, created_at
                 FROM compactions WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(rusqlite::params![id], row_to_compaction)
            .map_err(|e| ForgeError::Database(Box::new(e)))
    }

    /// List all compaction records for a session, ordered by created_at DESC.
    pub fn list_for_session(&self, session_id: &str) -> ForgeResult<Vec<Compaction>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, summary, original_token_count, compacted_token_count, created_at
                 FROM compactions WHERE session_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map(rusqlite::params![session_id], row_to_compaction)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(rows)
    }

    /// Get the most recent compaction for a session.
    pub fn get_latest(&self, session_id: &str) -> ForgeResult<Option<Compaction>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, summary, original_token_count, compacted_token_count, created_at
                 FROM compactions WHERE session_id = ?1 ORDER BY created_at DESC LIMIT 1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let result = stmt
            .query_row(rusqlite::params![session_id], row_to_compaction)
            .optional()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(result)
    }
}

fn row_to_compaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<Compaction> {
    Ok(Compaction {
        id: row.get(0)?,
        session_id: row.get(1)?,
        summary: row.get(2)?,
        original_token_count: row.get(3)?,
        compacted_token_count: row.get(4)?,
        created_at: row.get(5)?,
    })
}

/// Trait for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS compactions (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                summary TEXT NOT NULL,
                original_token_count INTEGER NOT NULL,
                compacted_token_count INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_compactions_session ON compactions(session_id);",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn compaction_crud() {
        let conn = setup_db();
        let repo = CompactionRepo::new(conn);

        // Create
        let c1 = repo
            .create("sess-1", "Summarized 10 tool calls", 5000, 1200)
            .unwrap();
        assert_eq!(c1.session_id, "sess-1");
        assert_eq!(c1.summary, "Summarized 10 tool calls");
        assert_eq!(c1.original_token_count, 5000);
        assert_eq!(c1.compacted_token_count, 1200);
        assert!(!c1.created_at.is_empty());

        // Create a second compaction for the same session
        let c2 = repo
            .create("sess-1", "Summarized 5 more calls", 3000, 800)
            .unwrap();
        assert_ne!(c1.id, c2.id);

        // Create a compaction for a different session
        let _c3 = repo
            .create("sess-2", "Other session summary", 2000, 500)
            .unwrap();

        // list_for_session returns only the matching session
        let list = repo.list_for_session("sess-1").unwrap();
        assert_eq!(list.len(), 2);
        // Should be ordered by created_at DESC (most recent first)
        // Both have the same created_at (datetime('now')), so order may vary,
        // but both belong to sess-1
        assert!(list.iter().all(|c| c.session_id == "sess-1"));

        // get_latest returns the most recent
        let latest = repo.get_latest("sess-1").unwrap();
        assert!(latest.is_some());
        let latest = latest.unwrap();
        assert_eq!(latest.session_id, "sess-1");

        // get_latest for a session with no compactions
        let none = repo.get_latest("sess-nonexistent").unwrap();
        assert!(none.is_none());

        // list_for_session for different session
        let list2 = repo.list_for_session("sess-2").unwrap();
        assert_eq!(list2.len(), 1);
        assert_eq!(list2[0].summary, "Other session summary");
    }
}
