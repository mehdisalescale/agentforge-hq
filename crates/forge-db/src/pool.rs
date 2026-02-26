//! Database connection management.

use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(path: &Path) -> ForgeResult<Self> {
        let conn = Connection::open(path).map_err(|e| ForgeError::Database(Box::new(e)))?;

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA cache_size = -8000;
        ",
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// In-memory DB for testing
    pub fn in_memory() -> ForgeResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| ForgeError::Database(Box::new(e)))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("db mutex poisoned")
    }

    /// Shared connection for BatchWriter and repos (same DB, concurrent access via mutex).
    pub fn conn_arc(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
}
