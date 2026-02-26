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

- [x] What was changed: Implemented token-bucket RateLimiter in forge-safety (tokens, max_tokens, refill_interval, last_refill); try_acquire() refills then takes one token; FORGE_RATE_LIMIT_MAX (10), FORGE_RATE_LIMIT_REFILL_MS (1000) in forge-app; added rate_limiter to AppState; run_handler returns 429 via rate_limit_exceeded() before circuit breaker; 3 tests (allows up to max, rejects after exhaustion, refills over time); all AppState test call sites updated.
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [ ] Notes:
