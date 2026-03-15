# Agent R2-C: Safety State Persistence + Process Spawn Limits

> Circuit breaker and cost tracker reset on restart — making safety features ineffective across restarts. Process spawn has no concurrency limits. Fix both.

**IMPORTANT: Run this agent AFTER R2-A and R2-B complete. This agent depends on R2-B's pool changes.**

## Step 1: Read Context

- `CLAUDE.md`
- `crates/forge-safety/src/lib.rs` — CircuitBreaker (in-memory AtomicU8 + Mutex<Instant>), RateLimiter (AtomicU32), CostTracker (warn/limit f64)
- `crates/forge-db/src/pool.rs` — updated by R2-B (may have new API)
- `crates/forge-db/src/lib.rs` — repo re-exports, Migrator
- `crates/forge-db/src/migrations.rs` — existing migration pattern
- `crates/forge-process/src/spawn.rs` — SpawnConfig struct, spawn() function, ProcessHandle
- `crates/forge-process/src/lib.rs` — module exports
- `crates/forge-process/Cargo.toml` — current dependencies
- `crates/forge-app/src/main.rs` — safety initialization (lines 121-139), spawn usage

## Step 2: Add Safety State Database Table

Create a new migration in `crates/forge-db/src/migrations.rs`. Find the existing migration pattern (look for `Migration` struct or `apply_pending` method) and add a new one:

```sql
CREATE TABLE IF NOT EXISTS safety_state (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

This is a simple key-value store for:
- `circuit_breaker` → `{"state": "Open", "failure_count": 3, "last_failure": "2026-03-15T..."}`
- `cost_tracker_budget_used` → `{"total_cost": 42.50}`

## Step 3: Create SafetyRepo

Create `crates/forge-db/src/repos/safety.rs`:

```rust
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
```

Add `use rusqlite::OptionalExtension;` at the top for the `.optional()` call.

Register in `crates/forge-db/src/repos/mod.rs` and re-export from `crates/forge-db/src/lib.rs`.

## Step 4: Add Persistence to CircuitBreaker

In `crates/forge-safety/src/lib.rs`, add serialization support:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub state: String,       // "Closed", "Open", "HalfOpen"
    pub failure_count: u32,
    pub last_failure_epoch_ms: Option<u64>,
}

impl CircuitBreaker {
    /// Export current state for persistence.
    pub fn export_state(&self) -> CircuitBreakerState {
        let state_name = match self.state() {
            CircuitState::Closed => "Closed",
            CircuitState::Open => "Open",
            CircuitState::HalfOpen => "HalfOpen",
        };
        let last_failure_ms = self.last_failure.lock().unwrap()
            .map(|i| i.elapsed().as_millis() as u64); // milliseconds ago
        CircuitBreakerState {
            state: state_name.to_string(),
            failure_count: self.failure_count.load(Ordering::SeqCst),
            last_failure_epoch_ms: last_failure_ms,
        }
    }

    /// Restore state from persistence. Call on startup.
    pub fn restore_state(&self, saved: &CircuitBreakerState) {
        let state_val = match saved.state.as_str() {
            "Open" => CircuitState::Open as u8,
            "HalfOpen" => CircuitState::HalfOpen as u8,
            _ => CircuitState::Closed as u8,
        };
        self.state.store(state_val, Ordering::SeqCst);
        self.failure_count.store(saved.failure_count, Ordering::SeqCst);

        if state_val == CircuitState::Open as u8 {
            if let Some(ms_ago) = saved.last_failure_epoch_ms {
                let elapsed = Duration::from_millis(ms_ago);
                if elapsed < self.timeout {
                    // Still within timeout — keep Open
                    *self.last_failure.lock().unwrap() = Some(Instant::now() - (self.timeout - elapsed));
                } else {
                    // Timeout has passed — transition to HalfOpen
                    self.state.store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
        }
    }
}
```

Add `serde` to `crates/forge-safety/Cargo.toml` if not already there:
```toml
serde = { workspace = true }
serde_json = { workspace = true }
```

## Step 5: Add Concurrency Limits to Process Spawn

In `crates/forge-process/src/spawn.rs`, add concurrency control:

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

/// Enforces max concurrent process spawns.
pub struct SpawnLimiter {
    semaphore: Arc<Semaphore>,
    pub config: SpawnConfig,
}

impl SpawnLimiter {
    pub fn new(config: SpawnConfig, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            config,
        }
    }

    /// Spawn with concurrency control. Waits for a permit if at max.
    pub async fn spawn_limited(
        &self,
        prompt: &str,
        session_id: Option<&str>,
    ) -> Result<(ProcessHandle, tokio::sync::OwnedSemaphorePermit), SpawnError> {
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|_| SpawnError::CommandMissing)?; // Semaphore closed
        let handle = spawn(&self.config, prompt, session_id).await?;
        Ok((handle, permit))
        // Caller holds permit until process completes, then drops it
    }

    /// Current number of running processes.
    pub fn active_count(&self) -> usize {
        let max = self.semaphore.available_permits() + self.active_running();
        max - self.semaphore.available_permits()
    }

    fn active_running(&self) -> usize {
        // This is a rough count — exact tracking would need a counter
        0
    }
}
```

Also add `max_concurrent` to SpawnConfig:

```rust
pub struct SpawnConfig {
    // ... existing fields ...
    /// Maximum concurrent agent processes. Default: 4.
    pub max_concurrent: usize,
    /// Maximum stdout bytes before truncation. Default: 10MB.
    pub max_output_bytes: usize,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            max_concurrent: 4,
            max_output_bytes: 10 * 1024 * 1024, // 10MB
        }
    }
}
```

Add env var support in `from_env()`:
```rust
if let Ok(max) = std::env::var("FORGE_MAX_CONCURRENT") {
    if let Ok(n) = max.parse::<usize>() {
        config.max_concurrent = n;
    }
}
```

## Step 6: Wire Safety Persistence in main.rs

In `crates/forge-app/src/main.rs`, add safety state load on startup and save on shutdown.

**After creating safety state (after line 139):**
```rust
// Load persisted safety state
let safety_repo = SafetyRepo::new(Arc::clone(&conn_arc));
if let Ok(Some(cb_json)) = safety_repo.get("circuit_breaker") {
    if let Ok(saved) = serde_json::from_str::<forge_safety::CircuitBreakerState>(&cb_json) {
        safety.circuit_breaker.restore_state(&saved);
        info!("restored circuit breaker state: {}", saved.state);
    }
}
```

**Before shutdown (before BatchWriter shutdown, around line 176):**
```rust
// Persist safety state before shutdown
let cb_state = safety.circuit_breaker.export_state();
if let Ok(json) = serde_json::to_string(&cb_state) {
    if let Err(e) = safety_repo.set("circuit_breaker", &json) {
        tracing::warn!("failed to persist circuit breaker state: {}", e);
    }
}
```

**Note:** You'll need access to `safety` after `AppState::new()` consumes it. Either:
- Clone the `Arc<CircuitBreaker>` before passing to AppState
- Or save state in AppState and extract after serve returns

## Step 7: Verify

```bash
cargo check 2>&1 | head -20
cargo test --workspace 2>&1 | tail -10
```

## Rules

- Touch ONLY: `crates/forge-safety/src/lib.rs`, `crates/forge-safety/Cargo.toml`, `crates/forge-db/src/repos/safety.rs` (new), `crates/forge-db/src/repos/mod.rs`, `crates/forge-db/src/lib.rs`, `crates/forge-db/src/migrations.rs`, `crates/forge-process/src/spawn.rs`, `crates/forge-process/src/lib.rs`, `crates/forge-process/Cargo.toml`, `crates/forge-app/src/main.rs` (safety init + shutdown sections only)
- Do NOT touch `crates/forge-core/src/event_bus.rs` (Agent R2-A handled that)
- Do NOT touch `crates/forge-db/src/pool.rs` (Agent R2-B handled that)
- Do NOT touch `site-docs/`, `CLAUDE.md`, `README.md`, `.github/workflows/`, `frontend/`
- Run `cargo check` and `cargo test --workspace` before reporting done

## Report

When done, create `docs/agents/REPORT_R2C.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
FILES_MODIFIED: [list]
FILES_CREATED: [list]
MIGRATION_ADDED: safety_state table (yes/no)
CIRCUIT_BREAKER_PERSISTENCE: export/restore (yes/no)
SPAWN_LIMITER: max_concurrent default=[N], env var FORGE_MAX_CONCURRENT (yes/no)
CARGO_CHECK: pass/fail
CARGO_TEST: pass/fail
NOTES: [any design decisions, compat issues]
```
