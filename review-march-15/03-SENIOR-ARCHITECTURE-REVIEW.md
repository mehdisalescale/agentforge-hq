# Senior Architecture & Engineering Review

> **Date:** 2026-03-15
> **Perspective:** Staff/Principal engineer reviewing for production readiness
> **Codebase:** AgentForge HQ — 13 Rust crates + SvelteKit frontend

---

## Executive Assessment

AgentForge has strong architectural bones: a well-factored crate structure, event-driven design, and a clear product vision. The code is honest about what it is — an orchestration layer, not an agent runtime. But several structural decisions, left uncorrected, will create escalating pain as the system scales from demo to production.

This review covers 8 areas, ordered by impact.

---

## 1. State Management: The `Arc<Repo>` Proliferation Problem

### Current State
`forge-app/main.rs` creates 16 individual `Arc<XyzRepo>` instances and passes them into `AppState`. Every new domain concept requires a new repo, a new Arc, and wiring through AppState.

### Problem
This is a classic "bag of services" anti-pattern. It works at 16 repos. It collapses at 30. Every handler function signature grows with each new dependency. Testing requires constructing the full AppState even when testing a single endpoint.

### Recommendation

Introduce a **Unit of Work** pattern:

```rust
pub struct UnitOfWork {
    conn: Connection,  // Single SQLite connection per request
}

impl UnitOfWork {
    pub fn agents(&self) -> AgentRepo<'_> { AgentRepo::new(&self.conn) }
    pub fn sessions(&self) -> SessionRepo<'_> { SessionRepo::new(&self.conn) }
    pub fn companies(&self) -> CompanyRepo<'_> { CompanyRepo::new(&self.conn) }
    // ... repos are lightweight views over the same connection
}
```

Benefits: single connection per request (SQLite WAL performs best this way), repos become zero-cost wrappers, handlers take `UnitOfWork` instead of 5 individual repos, and transactions naturally scope to the request lifecycle.

**Priority:** HIGH — do this before adding more domain concepts
**Effort:** 2-3 days refactor

---

## 2. Event Bus: Broadcast Channel Limitations

### Current State
`EventBus` uses `tokio::sync::broadcast` with capacity 16. All 35 event types flow through a single channel. WebSocket consumers and BatchWriter both subscribe.

### Problems

1. **Capacity 16 is dangerously low.** A burst of tool-use events from a single agent run can easily produce 20+ events in under a second. When the channel is full, slow receivers get `Lagged` errors and lose events. For an observability platform, losing events is a critical failure.

2. **No backpressure.** Broadcast channels drop messages rather than applying backpressure. The BatchWriter (which does I/O) can fall behind during bursts.

3. **No event filtering.** Every subscriber gets every event. The WebSocket handler must filter client-side. As event volume grows, this wastes CPU.

### Recommendation

Replace broadcast with a **multi-consumer event pipeline**:

```rust
// Option A: Sized broadcast with overflow buffer
// Use tokio::sync::broadcast with capacity 1024 + an overflow ring buffer

// Option B (better): Fan-out with dedicated channels per consumer type
pub struct EventPipeline {
    persistence_tx: mpsc::Sender<ForgeEvent>,   // Guaranteed delivery
    websocket_tx: broadcast::Sender<ForgeEvent>, // Best-effort for UI
    analytics_tx: mpsc::Sender<ForgeEvent>,      // Batched aggregation
}
```

The key insight: persistence must never lose events (use mpsc with backpressure), but WebSocket streaming can tolerate drops. Different consumers need different delivery guarantees.

**Priority:** HIGH — silent event loss undermines the entire observability story
**Effort:** 1-2 days

---

## 3. SQLite WAL Under Concurrent Agent Load

### Current State
Single SQLite database in WAL mode. 16 repos share connections. BatchWriter flushes every 50 events or 2 seconds.

### Problems

1. **Write contention.** SQLite WAL allows concurrent readers but only ONE writer at a time. When multiple agents run simultaneously, event writes from BatchWriter compete with session status updates, cost tracking writes, and approval status changes. Under load, you'll see `SQLITE_BUSY` errors.

2. **No connection pooling.** Each repo appears to hold its own connection strategy. Without a pool, connection creation overhead adds up.

3. **No read replicas.** Analytics queries (which scan large time ranges) compete with hot-path writes.

### Recommendation

1. **Add `r2d2-sqlite` or `deadpool-sqlite` connection pool** with separate read and write pools:

```rust
pub struct DbPool {
    writer: Pool<SqliteConnectionManager>,  // Size: 1 (SQLite only allows 1 writer)
    readers: Pool<SqliteConnectionManager>, // Size: num_cpus
}
```

2. **Set `busy_timeout`** to 5000ms to handle transient write contention gracefully:

```sql
PRAGMA busy_timeout = 5000;
```

3. **Consider WAL2 mode** (SQLite 3.37+) which allows concurrent writers to different tables.

4. **Long-term:** If multi-agent concurrency becomes the primary use case, evaluate moving events to an append-only log (e.g., `redb` or `sled`) and keeping SQLite for CRUD-only operations.

**Priority:** MEDIUM-HIGH — fine for single-agent demo, breaks under real multi-agent load
**Effort:** 2-3 days for connection pooling, 1 week for architecture change

---

## 4. Error Handling: ForgeError Needs Stratification

### Current State
`ForgeError` in `forge-core` is a flat enum covering everything from database errors to rate limiting to CLI spawn failures. Handlers map errors to HTTP status codes.

### Problem
Not all errors are equal. Currently there's no distinction between:
- **Retriable errors** (CLI temporarily unavailable, rate limited)
- **Client errors** (bad input, missing fields)
- **System errors** (database corruption, OOM)
- **Domain errors** (budget exceeded, approval denied)

This matters for: retry logic in MCP clients, error reporting in the UI, circuit breaker decisions, and alerting thresholds.

### Recommendation

Stratify errors into behavioral categories:

```rust
pub enum ForgeError {
    // Client errors (4xx) — caller's fault, don't retry
    Validation(String),
    NotFound { entity: &'static str, id: String },
    Conflict(String),

    // Domain errors (4xx) — business logic rejection
    BudgetExceeded { cost: f64, limit: f64 },
    ApprovalRequired { approval_type: String },
    RateLimited { retry_after_ms: u64 },

    // Retriable errors (5xx) — try again later
    CliUnavailable(String),
    CircuitOpen { reset_in_ms: u64 },
    Timeout(String),

    // System errors (5xx) — something is broken
    Database(String),
    Internal(String),
}

impl ForgeError {
    pub fn is_retriable(&self) -> bool { ... }
    pub fn http_status(&self) -> StatusCode { ... }
    pub fn error_code(&self) -> &'static str { ... }  // Machine-readable
}
```

**Priority:** MEDIUM — improves every consumer's error handling
**Effort:** 2 days refactor

---

## 5. The Middleware Pipeline Is Not Actually Middleware

### Current State
The "middleware" is described as an 8-stage pipeline, but looking at the code, it's implemented as sequential function calls in the run handler rather than as composable Axum middleware layers or a proper chain-of-responsibility pattern.

### Problem
This makes it impossible to:
- Reorder stages without modifying the handler
- Add conditional stages (e.g., skip SkillInjection for MCP calls)
- Test individual stages in isolation
- Share the pipeline between HTTP and MCP transports

### Recommendation

Implement a proper **pipeline pattern**:

```rust
pub trait Middleware: Send + Sync {
    async fn process(
        &self,
        ctx: &mut RunContext,
        next: &dyn Middleware,
    ) -> Result<RunResponse, MiddlewareError>;
}

pub struct Pipeline {
    stages: Vec<Box<dyn Middleware>>,
}

impl Pipeline {
    pub fn new() -> Self { Self { stages: vec![] } }
    pub fn add(mut self, stage: impl Middleware + 'static) -> Self {
        self.stages.push(Box::new(stage));
        self
    }
    pub async fn execute(&self, ctx: &mut RunContext) -> Result<RunResponse, MiddlewareError> {
        // Chain through stages
    }
}

// Usage in forge-app:
let http_pipeline = Pipeline::new()
    .add(RateLimiter::new(config))
    .add(CircuitBreaker::new(config))
    .add(CostChecker::new(config))
    .add(SkillInjector::new(catalog))
    .add(Persister::new(event_repo))
    .add(Spawner::new(cli_config))
    .add(ExitGate::new(quality_config));

let mcp_pipeline = Pipeline::new()
    .add(RateLimiter::new(config))
    .add(CostChecker::new(config))
    .add(Spawner::new(cli_config));
    // MCP skips skill injection and exit gate
```

**Priority:** MEDIUM — blocks MCP and HTTP parity
**Effort:** 3-4 days

---

## 6. Safety Layer: Circuit Breaker Needs Persistence

### Current State
`CircuitBreaker` in `forge-safety` is an in-memory 3-state FSM. If the server restarts, the circuit breaker resets to Closed even if the CLI was failing.

### Problem
A crash-restart cycle could hammer a broken CLI repeatedly. The circuit breaker forgets its state on every restart, providing no protection against persistent failures.

### Recommendation

1. Persist circuit breaker state to SQLite (just the state + failure count + last failure timestamp):

```sql
CREATE TABLE safety_state (
    key TEXT PRIMARY KEY,
    value_json TEXT,
    updated_at TEXT
);
```

2. On startup, load previous state. If it was Open and the reset timeout hasn't elapsed, stay Open.

3. Same for `CostTracker` — budget_used should survive restarts. Currently it resets to 0 on restart, which means budget enforcement is ineffective across server restarts.

**Priority:** MEDIUM — safety features that reset on restart aren't really safety features
**Effort:** 1 day

---

## 7. Process Spawn: No Resource Limits

### Current State
`forge-process` spawns Claude CLI as a child process. No CPU limits, no memory limits, no timeout, no maximum concurrent processes.

### Problem
A runaway agent or a prompt that triggers infinite tool loops can consume unbounded resources. The `LoopDetector` helps with detection but doesn't enforce limits at the OS level.

### Recommendation

```rust
pub struct SpawnConfig {
    pub max_concurrent: usize,         // Semaphore-limited
    pub timeout: Duration,             // Kill after N minutes
    pub max_output_bytes: usize,       // Truncate stdout after limit
    pub working_dir_allowed: Vec<PathBuf>, // Sandbox paths
}

// Use tokio::sync::Semaphore for concurrency control
let permit = self.semaphore.acquire().await?;
let child = Command::new(&self.cli_command)
    .timeout(self.timeout)
    .spawn()?;
// permit drops when child exits
```

Also consider using `cgroups` (Linux) or `rlimit` for hard OS-level enforcement.

**Priority:** MEDIUM — fine for local dev, dangerous for any shared/hosted deployment
**Effort:** 2 days

---

## 8. Testing Strategy Gaps

### Current State
`cargo test` runs unit tests. Integration tests exist but many are `#[ignore]` because they require a running `claude` CLI. No end-to-end tests for the HTTP API. No frontend tests.

### Gaps

1. **No API integration tests.** The Axum router can be tested with `axum::test::TestClient` without spawning a server. Every route should have at least a happy-path test.

2. **No database migration tests.** Migrations should be tested by applying them to an in-memory SQLite and verifying schema.

3. **No frontend tests.** No Vitest, no Playwright, no component tests.

4. **No load tests.** For a system that claims multi-agent orchestration, there should be a basic load test showing how many concurrent agents the system supports before SQLite contention degrades performance.

### Recommendation

```
tests/
  api/           # Axum TestClient tests for every route
  db/            # Migration and repo tests on in-memory SQLite
  safety/        # Circuit breaker, rate limiter state machine tests
  e2e/           # Full flow: create company → hire persona → run agent
  load/          # k6 or criterion benchmarks for concurrency limits
frontend/
  tests/         # Vitest for component logic, Playwright for flows
```

**Priority:** HIGH for API tests, MEDIUM for the rest
**Effort:** 1 week for API tests, ongoing for others

---

## 9. Dependency Hygiene

### Observations

1. **`rusqlite` bundled feature** compiles SQLite from source. This is correct for single-binary deployment but adds ~30s to clean builds. Consider caching the compiled artifact.

2. **`utoipa` + `utoipa-scalar`** for OpenAPI — good choice, but the annotations need to be added to all handlers to be useful. Currently partial.

3. **`rust-embed`** for the frontend — works but means any frontend change requires a full Rust recompile. Consider an alternative for dev mode: serve frontend from filesystem during development, embedded only in release.

4. **No `cargo-deny`** for license/vulnerability auditing. Add to CI.

5. **`cron 0.13`** is maintained but consider `croner` which has better timezone support.

### Recommendation
Add to CI:
```bash
cargo deny check advisories licenses
cargo audit
cargo clippy -- -D warnings
```

**Priority:** LOW — no immediate risk, good hygiene
**Effort:** 2 hours for CI setup

---

## 10. Observability: Missing Structured Logging

### Current State
Uses `tracing` for logging but no structured log format (JSON), no trace IDs, no span context propagation.

### Problem
When multiple agents run concurrently, log lines from different sessions interleave. Without session-scoped spans, debugging a single agent run requires manual correlation.

### Recommendation

```rust
// In run handler, create a span per request
let span = tracing::info_span!(
    "agent_run",
    session_id = %ctx.session_id,
    agent_id = %ctx.agent_id,
    agent_name = %agent.name,
);
let _guard = span.enter();

// In main.rs, add JSON formatter for production
if cfg!(not(debug_assertions)) {
    tracing_subscriber::fmt()
        .json()
        .with_span_events(FmtSpan::CLOSE)
        .init();
}
```

**Priority:** MEDIUM — essential before any production deployment
**Effort:** 1 day

---

## Summary: Top 5 Actions by Impact

| Rank | Action | Why |
|------|--------|-----|
| 1 | Event bus capacity + delivery guarantees | Silent event loss kills observability |
| 2 | Unit of Work pattern for repos | Blocks further domain expansion |
| 3 | API integration test suite | No tests = no confidence in changes |
| 4 | Connection pooling + busy_timeout | Multi-agent load will cause SQLITE_BUSY |
| 5 | Proper middleware pipeline | Blocks MCP/HTTP parity |
