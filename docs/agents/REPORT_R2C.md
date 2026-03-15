STATUS: COMPLETE
FILES_MODIFIED:
  - crates/forge-safety/src/lib.rs
  - crates/forge-safety/Cargo.toml
  - crates/forge-db/src/migrations.rs
  - crates/forge-db/src/repos/mod.rs
  - crates/forge-db/src/lib.rs
  - crates/forge-process/src/spawn.rs
  - crates/forge-process/src/lib.rs
  - crates/forge-process/src/concurrent.rs (test fix for new SpawnConfig fields)
  - crates/forge-app/src/main.rs
FILES_CREATED:
  - migrations/0013_safety_state.sql
  - crates/forge-db/src/repos/safety.rs
MIGRATION_ADDED: safety_state table (yes)
CIRCUIT_BREAKER_PERSISTENCE: export/restore (yes)
SPAWN_LIMITER: max_concurrent default=4, env var FORGE_MAX_CONCURRENT (yes)
CARGO_CHECK: pass
CARGO_TEST: pass (271 tests, 0 failures)
NOTES:
  - CircuitBreakerState uses serde for JSON serialization of state, failure_count, last_failure_epoch_ms
  - On startup, if circuit breaker was Open and timeout has passed, it transitions to HalfOpen automatically
  - SafetyRepo is a simple key-value store using the safety_state table
  - SpawnLimiter wraps a tokio::sync::Semaphore; caller holds OwnedSemaphorePermit until process completes
  - SpawnConfig gains max_concurrent (default 4) and max_output_bytes (default 10MB) fields
  - Safety state is saved before BatchWriter shutdown to ensure persistence
  - concurrent.rs test updated to include new SpawnConfig fields
