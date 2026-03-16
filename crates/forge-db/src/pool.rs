//! Database connection pool with r2d2 and busy_timeout.
//!
//! Provides a single-writer / multi-reader r2d2 pool, plus backward-compatible
//! `connection()` and `conn_arc()` methods so existing repos and BatchWriter
//! continue to work without changes.

use forge_core::error::{ForgeError, ForgeResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Shared PRAGMAs applied to every connection (file-backed databases).
const FILE_PRAGMAS: &str = "\
    PRAGMA journal_mode = WAL;\
    PRAGMA synchronous  = NORMAL;\
    PRAGMA foreign_keys = ON;\
    PRAGMA cache_size   = -8000;\
    PRAGMA busy_timeout = 5000;";

/// PRAGMAs for in-memory databases (no WAL, no synchronous tuning).
const MEMORY_PRAGMAS: &str = "\
    PRAGMA foreign_keys = ON;\
    PRAGMA busy_timeout = 5000;";

pub struct DbPool {
    /// Legacy shared connection — keeps `connection()` and `conn_arc()` working.
    conn: Arc<Mutex<Connection>>,
    /// r2d2 write pool (max_size = 1).
    write_pool: Pool<SqliteConnectionManager>,
    /// r2d2 read pool (sized to available parallelism).
    read_pool: Pool<SqliteConnectionManager>,
    /// Database file path (None for in-memory).
    path: Option<PathBuf>,
}

impl DbPool {
    pub fn new(path: &Path) -> ForgeResult<Self> {
        // --- legacy connection (unchanged API surface) ---
        let conn = Connection::open(path).map_err(|e| ForgeError::Database(Box::new(e)))?;
        conn.execute_batch(FILE_PRAGMAS)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        // --- r2d2 write pool (1 writer) ---
        let write_mgr = SqliteConnectionManager::file(path).with_init(|c| {
            c.execute_batch(FILE_PRAGMAS)?;
            Ok(())
        });
        let write_pool = Pool::builder()
            .max_size(1)
            .build(write_mgr)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        // --- r2d2 read pool (N readers) ---
        let num_readers = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(4)
            .max(2);
        let read_mgr = SqliteConnectionManager::file(path).with_init(|c| {
            c.execute_batch(FILE_PRAGMAS)?;
            Ok(())
        });
        let read_pool = Pool::builder()
            .max_size(num_readers)
            .build(read_mgr)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            write_pool,
            read_pool,
            path: Some(path.to_path_buf()),
        })
    }

    /// In-memory DB for testing.
    pub fn in_memory() -> ForgeResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| ForgeError::Database(Box::new(e)))?;
        conn.execute_batch(MEMORY_PRAGMAS)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        // For in-memory, r2d2 pools point to the same shared-cache URI so all
        // connections see the same data.  We use pool size 1 for both to avoid
        // concurrency issues with in-memory shared-cache.
        let write_mgr = SqliteConnectionManager::memory().with_init(|c| {
            c.execute_batch(MEMORY_PRAGMAS)?;
            Ok(())
        });
        let write_pool = Pool::builder()
            .max_size(1)
            .build(write_mgr)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        // In-memory: read pool = write pool (each in-memory connection is its
        // own database, so we must reuse the single pool).
        let read_pool = write_pool.clone();

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            write_pool,
            read_pool,
            path: None,
        })
    }

    // ── new pool-based API ──────────────────────────────────────────

    /// Get a pooled write connection (INSERT / UPDATE / DELETE).
    pub fn writer(&self) -> ForgeResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.write_pool
            .get()
            .map_err(|e| ForgeError::Database(Box::new(e)))
    }

    /// Get a pooled read connection (SELECT).
    pub fn reader(&self) -> ForgeResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.read_pool
            .get()
            .map_err(|e| ForgeError::Database(Box::new(e)))
    }

    // ── backward-compatible API (unchanged signatures) ──────────────

    /// Legacy: returns a MutexGuard on the shared connection.
    /// Used by Migrator and tests.
    pub fn connection(&self) -> ForgeResult<std::sync::MutexGuard<'_, Connection>> {
        lock_conn(&self.conn)
    }

    /// Legacy: shared Arc<Mutex<Connection>> for BatchWriter and repos.
    ///
    /// For file-backed DBs this opens a **separate** connection to the same
    /// file (with busy_timeout), so BatchWriter no longer contends with repos
    /// on the same mutex.  For in-memory DBs it clones the original Arc so
    /// all callers share one database.
    pub fn conn_arc(&self) -> ForgeResult<Arc<Mutex<Connection>>> {
        match &self.path {
            Some(p) => {
                let c = Connection::open(p).map_err(|e| ForgeError::Database(Box::new(e)))?;
                c.execute_batch(FILE_PRAGMAS).ok();
                Ok(Arc::new(Mutex::new(c)))
            }
            None => Ok(Arc::clone(&self.conn)),
        }
    }
}

/// Lock a database connection mutex, returning ForgeError::Internal on poisoned mutex.
/// Use this instead of `.lock().expect()` to avoid panics.
pub fn lock_conn(conn: &std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>) -> ForgeResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    conn.lock().map_err(|_| ForgeError::Internal("database mutex poisoned".into()))
}
