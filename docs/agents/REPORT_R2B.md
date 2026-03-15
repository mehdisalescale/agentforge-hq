STATUS: COMPLETE
APPROACH: Option A (backward compat)
FILES_MODIFIED: [crates/forge-db/Cargo.toml, crates/forge-db/src/pool.rs]
DEPENDENCIES_ADDED: [r2d2 0.8, r2d2_sqlite 0.25]
PRAGMAS_SET: busy_timeout=5000, journal_mode=WAL, synchronous=NORMAL, foreign_keys=ON, cache_size=-8000
POOL_SIZES: write=1, read=available_parallelism (min 2)
REPOS_UPDATED: 0 (Option A — no repo changes)
CARGO_CHECK: pass (forge-db clean; forge-process has pre-existing errors from other uncommitted work)
CARGO_TEST: pass (81/81 forge-db tests green)
NOTES:
- conn_arc() now opens a SEPARATE connection for file-backed DBs (with busy_timeout + WAL),
  so BatchWriter no longer contends on the same mutex as repos. For in-memory DBs it still
  clones the original Arc to keep test isolation working.
- connection() return type (MutexGuard<Connection>) is unchanged — Migrator and tests work as before.
- New writer()/reader() methods available for future Option B migration of individual repos.
- SqliteConnectionManager does not implement Clone, so in-memory read_pool = write_pool.clone()
  (Pool itself is Clone via Arc internals).
