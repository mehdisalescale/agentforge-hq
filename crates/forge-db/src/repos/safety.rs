//! Persistence for safety state (circuit breaker, cost tracking).

use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct SafetyRepo {
    conn: Arc<Mutex<Connection>>,
}

impl SafetyRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Load a safety state value by key. Returns None if not found.
    pub fn get(&self, key: &str) -> ForgeResult<Option<String>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT value_json FROM safety_state WHERE key = ?1")
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let result = stmt
            .query_row(rusqlite::params![key], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(result)
    }

    /// Save a safety state value. Upserts (insert or update).
    pub fn set(&self, key: &str, value_json: &str) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO safety_state (key, value_json, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value_json = ?2, updated_at = datetime('now')",
            rusqlite::params![key, value_json],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(())
    }
}

/// Trait for optional query results (mirrors rusqlite pattern).
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
