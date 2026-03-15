# Agent R2-B: Connection Pooling + Busy Timeout

> Replace single `Arc<Mutex<Connection>>` with a proper read/write connection pool. Add `PRAGMA busy_timeout` to handle write contention gracefully. Currently all 16 repos + BatchWriter share one mutex-guarded connection.

## Step 1: Read Context

- `CLAUDE.md`
- `crates/forge-db/src/pool.rs` — current: `DbPool { conn: Arc<Mutex<Connection>> }` with `.connection()` and `.conn_arc()` methods
- `crates/forge-db/Cargo.toml` — no pool dependency yet
- `crates/forge-db/src/lib.rs` — re-exports, module list
- `crates/forge-db/src/batch_writer.rs` — takes `Arc<Mutex<Connection>>`, uses `conn.lock()` in `flush_to_db`
- `crates/forge-db/src/repos/` — read 3-4 repo files to understand the pattern (they all take `Arc<Mutex<Connection>>` and call `.lock()`)
- `crates/forge-app/src/main.rs` — lines 60-76: creates `conn_arc = db.conn_arc()`, passes `Arc::clone(&conn_arc)` to each repo

## Step 2: Add r2d2 Dependency

In `crates/forge-db/Cargo.toml`, add:

```toml
r2d2 = "0.8"
r2d2_sqlite = "0.25"
```

Check https://crates.io for latest compatible versions. The key is `r2d2_sqlite` which provides `SqliteConnectionManager`.

## Step 3: Replace DbPool

Rewrite `crates/forge-db/src/pool.rs`:

```rust
//! Database connection pool with separate read and write access.

use forge_core::error::{ForgeError, ForgeResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Connection pool with a single writer and multiple readers.
/// SQLite allows only one writer at a time, so the write pool has size 1.
/// Readers can be concurrent (WAL mode).
pub struct DbPool {
    write_pool: Pool<SqliteConnectionManager>,
    read_pool: Pool<SqliteConnectionManager>,
}

impl DbPool {
    pub fn new(path: &Path) -> ForgeResult<Self> {
        let manager = SqliteConnectionManager::file(path)
            .with_init(|conn| {
                conn.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA synchronous = NORMAL;
                     PRAGMA foreign_keys = ON;
                     PRAGMA cache_size = -8000;
                     PRAGMA busy_timeout = 5000;"
                )?;
                Ok(())
            });

        let write_pool = Pool::builder()
            .max_size(1)  // SQLite: one writer at a time
            .build(manager.clone())
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let read_manager = SqliteConnectionManager::file(path)
            .with_init(|conn| {
                conn.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA foreign_keys = ON;
                     PRAGMA cache_size = -8000;
                     PRAGMA busy_timeout = 5000;"
                )?;
                Ok(())
            });

        let num_readers = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(4)
            .max(2);

        let read_pool = Pool::builder()
            .max_size(num_readers)
            .build(read_manager)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(Self { write_pool, read_pool })
    }

    /// In-memory DB for testing (single shared connection via write pool).
    pub fn in_memory() -> ForgeResult<Self> {
        let manager = SqliteConnectionManager::memory()
            .with_init(|conn| {
                conn.execute_batch("PRAGMA foreign_keys = ON;")?;
                Ok(())
            });

        // For in-memory, both pools share the same manager.
        // Use size 1 for both since in-memory DBs are per-connection.
        let write_pool = Pool::builder()
            .max_size(1)
            .build(manager.clone())
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        // For testing, read pool = write pool (same in-memory DB)
        let read_pool = write_pool.clone();

        Ok(Self { write_pool, read_pool })
    }

    /// Get a write connection (used for INSERT, UPDATE, DELETE).
    pub fn writer(&self) -> ForgeResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.write_pool.get().map_err(|e| ForgeError::Database(Box::new(e)))
    }

    /// Get a read connection (used for SELECT queries).
    pub fn reader(&self) -> ForgeResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.read_pool.get().map_err(|e| ForgeError::Database(Box::new(e)))
    }

    // === BACKWARD COMPATIBILITY BRIDGE ===
    // These methods allow incremental migration. Repos can switch one at a time.

    /// Legacy: get a MutexGuard-like connection. Routes through the write pool.
    /// DEPRECATED: Use writer() or reader() instead.
    pub fn connection(&self) -> r2d2::PooledConnection<SqliteConnectionManager> {
        self.write_pool.get().expect("db pool exhausted")
    }

    /// Legacy: get an Arc<Mutex<Connection>> for BatchWriter compatibility.
    /// DEPRECATED: BatchWriter should be updated to use the write pool directly.
    pub fn conn_arc(&self) -> Arc<Mutex<Connection>> {
        let conn = Connection::open(
            self.write_pool
                .state()
                .connections
                .to_string()
                .as_str()
        ).expect("failed to open connection for legacy bridge");
        Arc::new(Mutex::new(conn))
    }
}
```

**IMPORTANT:** The `conn_arc()` backward-compat method above is a placeholder. The better approach is:

1. Keep `conn_arc()` working by opening a separate connection to the same DB path
2. OR update BatchWriter to accept the pool directly

**Recommended approach:** Store the DB path in `DbPool` and use it in `conn_arc()`:

```rust
pub struct DbPool {
    write_pool: Pool<SqliteConnectionManager>,
    read_pool: Pool<SqliteConnectionManager>,
    path: Option<std::path::PathBuf>,  // None for in-memory
}

pub fn conn_arc(&self) -> Arc<Mutex<Connection>> {
    let conn = match &self.path {
        Some(p) => Connection::open(p).expect("db open for batch writer"),
        None => Connection::open_in_memory().expect("in-memory db"),
    };
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;").ok();
    Arc::new(Mutex::new(conn))
}
```

This keeps BatchWriter working as-is without rewriting it.

## Step 4: Update Repos

Each repo currently looks like:

```rust
pub struct XyzRepo {
    conn: Arc<Mutex<Connection>>,
}
impl XyzRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self { Self { conn } }
    pub fn list(&self) -> ForgeResult<Vec<Xyz>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        // ... query ...
    }
}
```

**Migration strategy — pick ONE of these approaches:**

### Option A: Keep Arc<Mutex<Connection>> in repos (minimal change)
Leave all repos as-is. The only change is in `pool.rs` and `main.rs`. The `conn_arc()` method returns a dedicated connection for repos. This is the **safest** approach but doesn't give read/write separation.

### Option B: Pass DbPool to repos (full migration)
Change each repo to take `Arc<DbPool>` instead of `Arc<Mutex<Connection>>`:

```rust
pub struct XyzRepo {
    pool: Arc<DbPool>,
}
impl XyzRepo {
    pub fn new(pool: Arc<DbPool>) -> Self { Self { pool } }
    pub fn list(&self) -> ForgeResult<Vec<Xyz>> {
        let conn = self.pool.reader()?;
        // ... query (unchanged) ...
    }
    pub fn create(&self, ...) -> ForgeResult<Xyz> {
        let conn = self.pool.writer()?;
        // ... insert (unchanged) ...
    }
}
```

**Choose Option A** — it's much less invasive. The read/write separation still helps because BatchWriter gets its own connection instead of fighting repos for the single mutex.

### If choosing Option A:

1. Update `DbPool::new()` with r2d2 + busy_timeout (Step 3 above)
2. Keep `conn_arc()` returning a separate `Arc<Mutex<Connection>>` to the same DB file
3. Keep all repos using `Arc<Mutex<Connection>>` — **no repo changes needed**
4. Only update `main.rs` to use new DbPool constructor

## Step 5: Update main.rs

In `crates/forge-app/src/main.rs`, the repo construction stays the same if using Option A. The only change is that DbPool internally uses r2d2 now, and `busy_timeout` is set.

If `conn_arc()` needs the DB path stored, update the constructor call:

```rust
let db = DbPool::new(path)?;
```

This should work as-is since `new()` already takes a `&Path`.

## Step 6: Verify

```bash
cargo check 2>&1 | head -20
cargo test --workspace 2>&1 | tail -10
```

Key things to verify:
- `DbPool::in_memory()` still works for tests
- `conn_arc()` still works for BatchWriter
- `connection()` still works for Migrator
- `PRAGMA busy_timeout = 5000` is set on all connections

## Rules

- Touch ONLY: `crates/forge-db/src/pool.rs`, `crates/forge-db/Cargo.toml`, `crates/forge-db/src/lib.rs` (if re-exports change)
- If using Option A (recommended): do NOT touch any repo files or `crates/forge-app/src/main.rs`
- Do NOT touch `crates/forge-core/` (Agent R2-A handles that)
- Do NOT touch `crates/forge-safety/` or `crates/forge-process/` (Agent R2-C handles those)
- Do NOT touch `site-docs/`, `CLAUDE.md`, `README.md`, `.github/workflows/`, `frontend/`
- Run `cargo check` and `cargo test --workspace` before reporting done

## Report

When done, create `docs/agents/REPORT_R2B.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
APPROACH: Option A (backward compat) | Option B (full migration)
FILES_MODIFIED: [list]
DEPENDENCIES_ADDED: [r2d2 version, r2d2_sqlite version]
PRAGMAS_SET: busy_timeout=5000, journal_mode=WAL, synchronous=NORMAL, foreign_keys=ON, cache_size=-8000
POOL_SIZES: write=1, read=[N]
REPOS_UPDATED: [count] (0 if Option A)
CARGO_CHECK: pass/fail
CARGO_TEST: pass/fail
NOTES: [any issues with in_memory, conn_arc compat, etc.]
```
