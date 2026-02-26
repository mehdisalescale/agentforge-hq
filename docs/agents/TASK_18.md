# TASK 18 — Circuit breaker

**Status:** pending
**Priority:** high
**Track:** Phase B — safety

---

## Context

`crates/forge-safety/src/lib.rs` has empty stubs for `CircuitBreaker`, `RateLimiter`, and `CostTracker`. If Claude CLI is down, every `/run` request spawns a process that immediately fails — wasting resources and flooding logs.

## Task

Implement a 3-state circuit breaker in `crates/forge-safety/src/lib.rs`:

```
Closed → (N consecutive failures) → Open → (timeout expires) → HalfOpen → (success) → Closed
                                                                         → (failure) → Open
```

1. **Struct:**
   ```rust
   use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
   use std::sync::Mutex;
   use std::time::{Duration, Instant};

   #[derive(Debug, Clone, Copy, PartialEq)]
   #[repr(u8)]
   pub enum CircuitState { Closed = 0, Open = 1, HalfOpen = 2 }

   pub struct CircuitBreaker {
       state: AtomicU8,
       failure_count: AtomicU32,
       success_count: AtomicU32,
       last_failure: Mutex<Option<Instant>>,
       failure_threshold: u32,   // default: 5
       success_threshold: u32,   // default: 2
       timeout: Duration,        // default: 60s
   }
   ```

2. **Methods:**
   - `new(failure_threshold, success_threshold, timeout)` + `Default`
   - `check() -> Result<(), CircuitBreakerError>` — returns Ok if request allowed, Err if circuit open
   - `record_success()` — in HalfOpen: increment success count, if >= threshold → Closed
   - `record_failure()` — in Closed: increment failure count, if >= threshold → Open; in HalfOpen → Open
   - `state() -> CircuitState` — current state
   - `reset()` — force back to Closed

3. **Wire into run handler** (`crates/forge-api/src/routes/run.rs`):
   - Add `circuit_breaker: Arc<CircuitBreaker>` to `AppState`
   - Before spawning: `state.circuit_breaker.check().map_err(|_| api_error(...))?`
   - In the spawned task: `record_success()` on ProcessCompleted, `record_failure()` on ProcessFailed

4. **Tests** (at least 5):
   - Closed allows requests
   - Opens after N failures
   - Open rejects immediately
   - Transitions to HalfOpen after timeout
   - Closes after success in HalfOpen

## Files to read first

- `crates/forge-safety/src/lib.rs` — current stubs
- `crates/forge-api/src/routes/run.rs` — where to wire it
- `crates/forge-api/src/state.rs` — AppState

## Files to edit

- `crates/forge-safety/src/lib.rs`
- `crates/forge-safety/Cargo.toml` (if new deps needed)
- `crates/forge-api/src/state.rs`
- `crates/forge-api/src/routes/run.rs`
- `crates/forge-app/src/main.rs` (create and pass CircuitBreaker)
- Update test call sites for AppState::new

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: Implemented 3-state CircuitBreaker in forge-safety (Closed/Open/HalfOpen, failure_threshold 5, success_threshold 2, timeout 60s); check(), record_success(), record_failure(), state(), reset(); 7 tests. Wired into AppState, run handler (check before spawn; record_success on ProcessCompleted, record_failure on ProcessFailed); forge-app creates default and passes to AppState; all forge-api tests updated.
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [ ] Notes:
