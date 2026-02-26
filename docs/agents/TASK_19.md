# TASK 19 — Rate limiter

**Status:** pending
**Priority:** medium
**Track:** Phase B — safety

---

## Context

No rate limiting. A client can spam `POST /api/v1/run` and exhaust system resources. Need a token bucket rate limiter.

## Task

1. Implement in `crates/forge-safety/src/lib.rs`:

   ```rust
   pub struct RateLimiter {
       tokens: AtomicU32,
       max_tokens: u32,
       refill_interval: Duration,
       last_refill: Mutex<Instant>,
   }

   impl RateLimiter {
       pub fn new(max_tokens: u32, refill_interval: Duration) -> Self { ... }
       pub fn try_acquire(&self) -> bool { ... } // refill, then try take one token
   }
   ```

   Default: 10 tokens max, refill 1 per second.

2. Read config from env: `FORGE_RATE_LIMIT_MAX` (default 10), `FORGE_RATE_LIMIT_REFILL_MS` (default 1000).

3. Add to `AppState` as `rate_limiter: Arc<RateLimiter>`.

4. In `run_handler`, before the circuit breaker check:
   ```rust
   if !state.rate_limiter.try_acquire() {
       return Err((StatusCode::TOO_MANY_REQUESTS, Json(ErrorBody { ... })).into_response());
   }
   ```

5. Tests:
   - Allows requests up to max_tokens
   - Rejects after exhaustion
   - Refills over time

## Files to edit

- `crates/forge-safety/src/lib.rs`
- `crates/forge-api/src/state.rs`
- `crates/forge-api/src/routes/run.rs`
- `crates/forge-app/src/main.rs`
- Update test call sites for AppState::new

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
```

---

## Report

*Agent: fill this in when done.*

- [ ] What was changed:
- [ ] Tests pass: yes/no
- [ ] Clippy clean: yes/no
- [ ] Notes:
