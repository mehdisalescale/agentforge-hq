# Claude Forge — Master Task List

> **Created:** 2026-02-27 | **Scope:** Everything remaining from current state to v0.3.0+
> **For:** Cursor agents working in parallel. Each task is self-contained.
> **Rule:** Complete tasks in order within each phase. Don't skip phases.

---

## How To Use This File

1. Pick a task. Read its **What/Where/How/Verify** section.
2. Mark it `[x]` when done.
3. Run `cargo test --workspace && cargo clippy --workspace` after every task.
4. Commit after each task (or batch of related tasks).
5. Don't update planning docs — write code.

---

## PHASE 0: Current Sprint — Integration Wiring

> **Goal:** Connect the pieces that were built in isolation. All code exists but isn't wired together.
> **Estimated effort:** 4-6 hours total

### S1: Wire BatchWriter to EventBus

- [ ] **Done**

**What:** Events broadcast on the EventBus vanish after delivery. BatchWriter exists in forge-db (tested, working) but is never connected. Session export returns empty because events aren't persisted.

**Where:** `crates/forge-app/src/main.rs`

**How:**
1. Add `use forge_db::BatchWriter;` to imports.
2. After creating `conn_arc` and before creating `AppState`, spawn the BatchWriter and subscribe it:
```rust
let batch_writer = BatchWriter::spawn(Arc::clone(&conn_arc));
let event_bus = EventBus::new(1024);  // also fix S2 here
let mut persist_rx = event_bus.subscribe();
let bw = batch_writer.clone(); // if BatchWriter is Clone, otherwise Arc it
tokio::spawn(async move {
    while let Ok(event) = persist_rx.recv().await {
        if let Err(e) = bw.write(event) {
            tracing::warn!(error = %e, "failed to persist event");
        }
    }
});
```
3. Note: `BatchWriter::spawn` returns a `BatchWriter` with a `write()` method that takes `ForgeEvent`. Check if it needs `Arc` wrapping or if it's already `Clone` (it uses `crossbeam_channel::Sender` internally which is `Clone`).
4. On shutdown, call `batch_writer.shutdown()` — see S8 for graceful shutdown.

**Verify:**
- `cargo test --workspace` passes
- Start the app, create an agent, run a prompt, then check: `GET /api/v1/sessions/:id/export?format=json` — events array should be non-empty.

---

### S2: Increase EventBus Capacity

- [ ] **Done**

**What:** `EventBus::new(16)` is too small. Fast process runs overflow the broadcast channel, causing `RecvError::Lagged` — events silently dropped for WebSocket clients and BatchWriter.

**Where:** `crates/forge-app/src/main.rs` line 45

**How:** Change `EventBus::new(16)` to `EventBus::new(1024)`.

**Verify:** `cargo test --workspace` passes.

---

### S3: Add `directory` Field to Frontend Run Form

- [ ] **Done**

**What:** The backend `POST /api/v1/run` accepts an optional `directory` field, but the frontend Run form doesn't include it. Users can't specify a working directory.

**Where:**
- `frontend/src/lib/api.ts` — `RunRequest` interface and `runAgent()` function
- `frontend/src/routes/+page.svelte` — Run form

**How:**
1. In `api.ts`, add `directory?: string` to the `RunRequest` interface (or whatever the run request type is called).
2. In `+page.svelte`, add an input field:
```svelte
<label for="directory-input">Directory (optional)</label>
<input
  id="directory-input"
  type="text"
  bind:value={directory}
  placeholder="/path/to/project"
  disabled={running}
/>
```
3. Add `let directory = '';` to the script variables.
4. Include `directory: directory.trim() || undefined` in the `runAgent()` call.

**Verify:** `pnpm build` succeeds. Run form shows directory field. Submitting with a directory sends it in the request body.

---

### S4: Show `status` in Sessions UI

- [ ] **Done**

**What:** The API returns `status` for each session, but the frontend Session type doesn't include it and the UI doesn't display it.

**Where:**
- `frontend/src/lib/api.ts` — `Session` interface
- `frontend/src/routes/sessions/+page.svelte` — session list and detail views

**How:**
1. In `api.ts`, add `status: string;` to the `Session` interface.
2. In `sessions/+page.svelte`, display the status in the session list items and detail view. Use a badge/tag style:
```svelte
<span class="status-badge" class:running={s.status === 'running'} class:completed={s.status === 'completed'} class:failed={s.status === 'failed'}>
  {s.status}
</span>
```

**Verify:** Sessions page shows status for each session.

---

### S5: Replace `.expect()` With Error Propagation

- [ ] **Done**

**What:** 13 calls to `.expect("db mutex poisoned")` across repo files. If any thread panics while holding the DB lock, all subsequent requests crash the server.

**Where:**
- `crates/forge-db/src/repos/agents.rs` — 4 calls
- `crates/forge-db/src/repos/sessions.rs` — 5 calls
- `crates/forge-db/src/repos/events.rs` — 3 calls
- `crates/forge-db/src/batch_writer.rs` — 1 call (inside the flush thread)

**How:**
Replace every instance of:
```rust
let conn = self.conn.lock().expect("db mutex poisoned");
```
With:
```rust
let conn = self.conn.lock().map_err(|_| ForgeError::Internal("database lock poisoned".into()))?;
```
For the batch_writer (non-Result context), use:
```rust
let conn = match self.conn.lock() {
    Ok(c) => c,
    Err(_) => {
        tracing::error!("batch writer: db mutex poisoned");
        return;
    }
};
```

**Verify:** `cargo test --workspace` passes. `cargo clippy --workspace` no new warnings.

---

### S6: Add Prompt Length Validation

- [ ] **Done**

**What:** `RunRequest.prompt` is unbounded. A 1GB prompt causes OOM.

**Where:** `crates/forge-api/src/routes/run.rs` — `run_handler` function

**How:** Add after the agent lookup:
```rust
if body.prompt.len() > 100_000 {
    return Err(api_error(ForgeError::Validation("prompt exceeds 100KB limit".into())));
}
```

**Verify:** `cargo test --workspace` passes. `curl -X POST /api/v1/run` with a huge prompt returns 400.

---

### S7: Fix Clippy Warnings

- [ ] **Done**

**What:** Two clippy warnings: `doc_lazy_continuation` in spawn.rs and `result_large_err` in error.rs.

**Where:**
- `crates/forge-process/src/spawn.rs` line 59 — doc comment indentation
- `crates/forge-api/src/error.rs` — `parse_uuid` returns `Result<Uuid, Response>` (128 bytes)

**How:**
1. In `spawn.rs`, indent the doc continuation line:
```rust
/// Read from environment variables.
///   Backward compatible: if env vars are unset, defaults are used.
```
2. In `error.rs`, either Box the error or allow it:
```rust
#[allow(clippy::result_large_err)]
pub fn parse_uuid(s: &str) -> Result<uuid::Uuid, Response> {
```

**Verify:** `cargo clippy --workspace -- -D warnings` passes clean.

---

### S8: Fix CORS for Production

- [ ] **Done**

**What:** `CorsLayer::new().allow_origin(Any)` allows any website to make requests to the API.

**Where:** `crates/forge-api/src/lib.rs` lines 17-20

**How:**
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)  // TODO: restrict in production via FORGE_CORS_ORIGIN env var
    .allow_methods(Any)
    .allow_headers(Any);
```
For now, add a TODO comment. Full fix: read `FORGE_CORS_ORIGIN` env var and parse as `HeaderValue`. Fall back to `Any` if unset (dev mode).

**Verify:** `cargo test --workspace` passes.

---

## PHASE A: Ship v0.1.0

> **Goal:** Real users can download a binary and use it.
> **Prerequisite:** All Phase 0 tasks complete.
> **Estimated effort:** 10-15 hours total

### A1: Embed Frontend in Binary (rust-embed)

- [ ] **Done**

**What:** The binary currently serves API only. The frontend must be built with `pnpm build` and served from the binary via `rust-embed`.

**Where:**
- `crates/forge-api/Cargo.toml` — add `rust-embed` dependency
- `crates/forge-api/src/lib.rs` — add static file handler as fallback
- New file: `crates/forge-api/src/static_files.rs`

**How:**
1. Add to `crates/forge-api/Cargo.toml`:
```toml
rust-embed = { workspace = true }
mime_guess = "2"
```
2. Create `crates/forge-api/src/static_files.rs`:
```rust
use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../../frontend/build"]
struct Asset;

pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    // Try exact path, then index.html for SPA routing
    if let Some(content) = Asset::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
    } else if let Some(content) = Asset::get("index.html") {
        Html(std::str::from_utf8(&content.data).unwrap_or("")).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
```
3. In `lib.rs`, add the fallback:
```rust
pub mod static_files;
// ...
Router::new()
    .nest("/api/v1", routes::router())
    .fallback(static_files::static_handler)
    .layer(cors)
    .with_state(state)
```
4. Update `forge-app/src/main.rs` log message: remove "no frontend" comment.
5. Add build step: `cd frontend && pnpm build` before `cargo build --release`.

**Verify:**
- `cargo build --release` succeeds
- Run binary, open `http://127.0.0.1:4173` in browser — UI loads
- Navigate to `/agents` — page renders (SPA routing works)

---

### A2: Graceful Shutdown

- [ ] **Done**

**What:** The binary has no signal handling. SIGTERM/SIGINT leaves BatchWriter unflushed and connections open.

**Where:** `crates/forge-app/src/main.rs`

**How:**
1. Use `tokio::signal` for shutdown:
```rust
use tokio::signal;

// Replace the simple serve call with:
let server = axum::serve(listener, app(state));
let graceful = server.with_graceful_shutdown(async {
    signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    info!("shutdown signal received");
});
graceful.await?;

// After server stops:
info!("flushing batch writer...");
batch_writer.shutdown().expect("batch writer shutdown failed");
info!("shutdown complete");
```

**Verify:** Start the binary, hit Ctrl+C, confirm "shutdown complete" appears in logs.

---

### A3: Add TraceLayer Middleware

- [ ] **Done**

**What:** No HTTP request/response logging. Can't debug what requests hit the server.

**Where:** `crates/forge-api/src/lib.rs`

**How:**
```rust
use tower_http::trace::TraceLayer;

Router::new()
    .nest("/api/v1", routes::router())
    .fallback(static_files::static_handler)
    .layer(TraceLayer::new_for_http())  // ADD THIS
    .layer(cors)
    .with_state(state)
```

**Verify:** Start the app, hit `/api/v1/health`, confirm request log appears in terminal.

---

### A4: Configurable Host and Port

- [ ] **Done**

**What:** Server hardcoded to `127.0.0.1:4173`.

**Where:** `crates/forge-app/src/main.rs` line 53

**How:**
```rust
let host = env::var("FORGE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
let port = env::var("FORGE_PORT").unwrap_or_else(|_| "4173".into());
let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
```

**Verify:** `FORGE_PORT=8080 cargo run -p forge-app` starts on port 8080.

---

### A5: E2E Smoke Test Script

- [ ] **Done**

**What:** No way to verify the full flow works end-to-end.

**Where:** New file: `scripts/e2e-smoke.sh`

**How:** Write a bash script that:
```bash
#!/usr/bin/env bash
set -euo pipefail

BASE="http://127.0.0.1:4173/api/v1"

echo "=== Health check ==="
curl -sf "$BASE/health" | jq .

echo "=== Create agent ==="
AGENT=$(curl -sf -X POST "$BASE/agents" \
  -H 'Content-Type: application/json' \
  -d '{"name":"SmokeTest","model":"claude-sonnet-4-20250514"}')
AGENT_ID=$(echo "$AGENT" | jq -r .id)
echo "Agent ID: $AGENT_ID"

echo "=== Run agent ==="
RUN=$(curl -sf -X POST "$BASE/run" \
  -H 'Content-Type: application/json' \
  -d "{\"agent_id\":\"$AGENT_ID\",\"prompt\":\"Say hello in one word.\"}")
SESSION_ID=$(echo "$RUN" | jq -r .session_id)
echo "Session ID: $SESSION_ID"

echo "=== Wait for process ==="
sleep 10

echo "=== List sessions ==="
curl -sf "$BASE/sessions" | jq '.[0]'

echo "=== Export session ==="
curl -sf "$BASE/sessions/$SESSION_ID/export?format=json" | jq '.events | length'

echo "=== Export markdown ==="
curl -sf "$BASE/sessions/$SESSION_ID/export?format=markdown" | head -20

echo "=== Cleanup ==="
curl -sf -X DELETE "$BASE/agents/$AGENT_ID"

echo "=== SMOKE TEST PASSED ==="
```

**Verify:** Start the binary in one terminal, run `bash scripts/e2e-smoke.sh` in another. Should print "SMOKE TEST PASSED".

---

### A6: GitHub Actions CI

- [ ] **Done**

**What:** No CI pipeline. Tests, clippy, and build aren't automated.

**Where:** New file: `.github/workflows/ci.yml`

**How:**
```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
      - run: cargo build --release
```
Add a second job for frontend build if frontend exists:
```yaml
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - run: cd frontend && pnpm install && pnpm build
```

**Verify:** Push to GitHub, Actions tab shows green checks.

---

### A7: GitHub Release Binary

- [ ] **Done**

**What:** Users need a downloadable binary.

**Where:** New file: `.github/workflows/release.yml`

**How:**
- Trigger on tag push (`v*`)
- Build for macOS arm64, macOS x86_64, Linux x86_64
- Use `cross` for cross-compilation or native runners
- Upload binaries as release assets via `softprops/action-gh-release`

**Verify:** Tag `v0.1.0`, push, release appears with downloadable binaries.

---

### A8: README.md

- [ ] **Done**

**What:** GitHub landing page. Users need to know what this is and how to use it.

**Where:** Root `README.md` (or `forge-project/README.md`)

**How:** Include:
- One-line description
- Screenshot/GIF of the UI
- Install instructions (download binary or `cargo install`)
- Quick start (run binary, open browser, create agent, run prompt)
- Architecture diagram (text, not image)
- Link to NORTH_STAR.md for contributors

**Verify:** Looks good on GitHub.

---

### A9: Update NORTH_STAR.md Honestly

- [ ] **Done**

**What:** NORTH_STAR claims features that don't exist ("MCP server editor", "Hooks editor", "Multi-pane tab layout"). Needs honest update.

**Where:** `NORTH_STAR.md` — "Current State" section

**How:** Replace the "What Works" list with what actually exists in code:
- 8 Rust crates (forge-core/agent/db/api/process/safety/mcp/app)
- Agent CRUD + 9 presets (API + frontend)
- Process spawning with stream-json parsing + --resume
- Real-time WebSocket event streaming
- Session CRUD + export (JSON/Markdown)
- Run endpoint with real Claude CLI spawn
- Frontend: Dashboard (Run), Agents (CRUD), Sessions (list/detail/export)
- Frontend: Skills, Workflows, Settings (placeholder pages)

Remove references to: MCP server editor, Hooks editor, Multi-pane tab layout, Directory picker, CLAUDE.md editor, split view.

---

## PHASE B: Core Loop + MCP → v0.2.0

> **Goal:** MCP server so Claude Desktop can drive Forge. Safety controls to prevent runaway costs.
> **Prerequisite:** v0.1.0 shipped, user feedback collected.
> **Estimated effort:** 25-35 hours total

### B1: Auto-Update Session Status on Process Events

- [ ] **Done**

**What:** Sessions stay in "created" status forever. Status should update to "running" when ProcessStarted fires, "completed" on ProcessCompleted, "failed" on ProcessFailed.

**Where:** `crates/forge-api/src/routes/run.rs` — the background `tokio::spawn` task

**How:** After emitting ProcessStarted, call:
```rust
if let Err(e) = state.session_repo.update_status(&sid, "running") {
    tracing::warn!(error = %e, "failed to update session status to running");
}
```
Similarly after ProcessCompleted → "completed", ProcessFailed → "failed".

Note: `update_status` already exists on `SessionRepo` (added in previous sprint). The run task needs access to `session_repo` — clone the `Arc<SessionRepo>` into the spawned task.

**Verify:** Run an agent. Check `GET /api/v1/sessions` — status should progress: created → running → completed.

---

### B2: Markdown Rendering in Output Stream

- [ ] **Done**

**What:** Assistant output renders as raw text. Should render as formatted Markdown with code blocks, headings, lists.

**Where:** `frontend/src/routes/+page.svelte` — stream output area

**How:**
1. Add a Markdown renderer. Options:
   - `marked` (npm package, lightweight)
   - `svelte-markdown` component
2. Replace `<pre><code>{streamContent}</code></pre>` with rendered HTML:
```svelte
{@html renderMarkdown(streamContent)}
```
3. Sanitize the HTML output to prevent XSS (use `DOMPurify` or `marked` with sanitize option).

**Verify:** Run an agent that produces Markdown. Code blocks, headings, and lists render correctly.

---

### B3: Tool Use/Result Collapsible Panels

- [ ] **Done**

**What:** When Claude uses tools, the output should show tool calls as expandable blocks, not inline text.

**Where:** `frontend/src/routes/+page.svelte` — WebSocket message handler

**How:**
1. Instead of appending all content to one `streamContent` string, maintain an array of output blocks:
```typescript
interface OutputBlock {
    kind: 'assistant' | 'tool_use' | 'tool_result' | 'thinking';
    content: string;
}
let outputBlocks: OutputBlock[] = [];
```
2. In the WebSocket handler, check `ev.data?.kind` and push to the array.
3. Render each block type differently:
   - `assistant`: Markdown rendered
   - `tool_use`: Collapsible `<details>` with tool name as summary
   - `tool_result`: Collapsible with "Result" as summary, red border if error
   - `thinking`: Collapsible with "Thinking..." as summary, dimmed text

**Verify:** Run an agent that uses tools. Tool calls appear as collapsible panels.

---

### B4: Circuit Breaker

- [ ] **Done**

**What:** No protection against cascading failures. If Claude CLI is down, every request spawns a process that immediately fails.

**Where:** `crates/forge-safety/src/lib.rs` — replace stub

**How:**
Implement a 3-state machine:
```rust
pub enum CircuitState { Closed, Open, HalfOpen }

pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    config: CircuitBreakerConfig,
}

pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,    // default: 5
    pub timeout: Duration,         // default: 60s
    pub success_threshold: u32,    // default: 2
}

impl CircuitBreaker {
    pub fn check(&self) -> Result<(), CircuitBreakerError> { ... }
    pub fn record_success(&self) { ... }
    pub fn record_failure(&self) { ... }
}
```
- Closed: allow requests. After `failure_threshold` consecutive failures → Open
- Open: reject immediately. After `timeout` → HalfOpen
- HalfOpen: allow one request. If succeeds → Closed. If fails → Open

Wire into run handler: check circuit breaker before spawning.

**Verify:** Unit tests for all state transitions. Integration: fail 5 runs, 6th returns 503 immediately.

---

### B5: Rate Limiter

- [ ] **Done**

**What:** No rate limiting. Single client can spam `/run` and exhaust resources.

**Where:** `crates/forge-safety/src/lib.rs` — replace stub

**How:**
Token bucket rate limiter:
```rust
pub struct RateLimiter {
    tokens: AtomicU32,
    max_tokens: u32,
    refill_rate: Duration,  // one token per this duration
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    pub fn try_acquire(&self) -> bool { ... }
}
```
Add as tower middleware layer on the `/run` route (or globally):
```rust
use tower::limit::RateLimitLayer;
// Or custom middleware wrapping forge_safety::RateLimiter
```

**Verify:** Rapid-fire 20 requests to `/run`. After limit, responses return 429.

---

### B6: Cost Tracking

- [ ] **Done**

**What:** No visibility into how much agent runs cost.

**Where:** `crates/forge-safety/src/lib.rs` — replace CostTracker stub

**How:**
1. Parse `cost_usd` from `ResultPayload` in stream-json output (field already exists in `stream_event.rs`).
2. Track per-session cost and aggregate per-agent.
3. Budget enforcement:
   - `FORGE_BUDGET_WARN` env var (soft limit — emit BudgetWarning event)
   - `FORGE_BUDGET_LIMIT` env var (hard limit — emit BudgetExceeded, stop agent)
4. Store cost in sessions table (add `cost_usd REAL` column via migration v2).
5. API endpoint: `GET /api/v1/sessions/:id` already returns session — add cost field.

**Verify:** Run an agent. Session export includes cost. Exceed budget → agent stops.

---

### B7: MCP Server — Stdio Transport

- [ ] **Done**

**What:** No MCP server. Claude Desktop can't connect to Forge.

**Where:** `crates/forge-mcp/src/lib.rs` — replace stubs

**How:**
1. Implement JSON-RPC dispatcher over stdin/stdout:
```rust
pub struct McpServer {
    state: AppState, // or subset of it
}

impl McpServer {
    pub async fn run_stdio(&self) -> Result<(), McpError> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        // Read JSON-RPC requests from stdin, dispatch, write responses to stdout
    }
}
```
2. Add CLI flag to forge-app: `forge --mcp` starts in MCP mode (stdio) instead of HTTP mode.
3. Implement protocol handshake: `initialize` → `initialized` → ready for tool calls.

**Verify:** Pipe JSON-RPC `initialize` to stdin, get valid response on stdout.

---

### B8: MCP Tools (10)

- [ ] **Done**

**What:** Tools that Claude Desktop can call to interact with Forge.

**Where:** `crates/forge-mcp/src/` — new module `tools.rs`

**How:** Implement 10 tools:

| Tool | Method | Description |
|------|--------|-------------|
| `forge_agent_create` | Create agent | Takes name, model, preset |
| `forge_agent_list` | List agents | Returns all agents |
| `forge_agent_get` | Get agent | Takes agent_id |
| `forge_agent_delete` | Delete agent | Takes agent_id |
| `forge_run` | Run agent | Takes agent_id, prompt, directory |
| `forge_session_list` | List sessions | Optional agent_id filter |
| `forge_session_get` | Get session | Takes session_id |
| `forge_session_export` | Export session | Takes session_id, format |
| `forge_config_get` | Get config | Takes key |
| `forge_health` | Health check | Returns status |

Each tool returns JSON matching the HTTP API response format.

**Verify:** Protocol compliance test: call each tool via stdin JSON-RPC, verify response.

---

### B9: MCP Resources (5)

- [ ] **Done**

**What:** Resources that MCP clients can read from Forge.

**Where:** `crates/forge-mcp/src/` — new module `resources.rs`

**How:** Implement 5 resources:

| URI | Description |
|-----|-------------|
| `forge://agents` | List of all agents |
| `forge://sessions` | List of recent sessions |
| `forge://config` | Current configuration |
| `forge://health` | System health status |
| `forge://skills` | Skill catalog (empty for now) |

**Verify:** MCP `resources/list` returns all 5. `resources/read` for each returns valid JSON.

---

### B10: Safety Dashboard Widget

- [ ] **Done**

**What:** Frontend visibility into circuit breaker state, rate limit usage, budget.

**Where:** `frontend/src/routes/+page.svelte` — new section on Dashboard

**How:**
1. Add API endpoint: `GET /api/v1/status` returning `{ circuit_state, rate_limit_remaining, budget_used, budget_limit }`.
2. Dashboard widget showing:
   - Circuit breaker: green (closed), yellow (half-open), red (open)
   - Rate limit: X/Y remaining
   - Budget: progress bar with current/limit

**Verify:** Dashboard shows safety status. Trip the circuit breaker → widget turns red.

---

## PHASE C: Differentiate → v0.3.0

> **Goal:** Pick ONE feature. Decision deferred until Phase A user feedback.
> **Prerequisite:** v0.2.0 shipped.
> **Estimated effort:** 20-30 hours total

### Option 1: Multi-Agent Observability

- [ ] C1-1: Metrics collection (`metrics` crate) across all crates
- [ ] C1-2: Token usage tracking per agent/model/session
- [ ] C1-3: Main dashboard: active agents, recent runs, system health
- [ ] C1-4: Cost dashboard: daily/weekly/monthly, per-agent breakdown
- [ ] C1-5: Agent swim lanes: parallel timeline visualization
- [ ] C1-6: Session timeline: event sequence visualization

### Option 2: Worktree-Per-Agent Isolation

- [ ] C2-1: `forge-git` crate wrapping `git2`
- [ ] C2-2: Git status, diff, log operations
- [ ] C2-3: Worktree create/remove per agent
- [ ] C2-4: Git panel in frontend (status, diff viewer)
- [ ] C2-5: Auto-detect working directory changes

### Option 3: Workflow DAG Execution

- [ ] C3-1: `forge-workflow` crate: Workflow DSL (YAML/JSON)
- [ ] C3-2: Step types: Prompt, Parallel, Conditional, Loop
- [ ] C3-3: State machine: Pending → Running → Completed/Failed
- [ ] C3-4: 5 built-in templates (review, refactor, test, doc, debug)
- [ ] C3-5: Workflow builder UI
- [ ] C3-6: Run visualization: step progress

---

## PHASE D: User-Driven Iteration

> **Goal:** Build what users ask for. No pre-planned scope.
> **Prerequisite:** v0.3.0 shipped, user community exists.

### Likely Requests (Pre-populate as needed)

- [ ] D1: Skill catalog (import 10-20 skills from reference repos)
- [ ] D2: Webhook notifications (POST to URL on events)
- [ ] D3: Session search / filtering (by agent, status, date)
- [ ] D4: Agent config templates (save/load configurations)
- [ ] D5: Keyboard shortcuts (global and per-page)
- [ ] D6: Dark/light theme toggle
- [ ] D7: Session scan from `~/.claude/projects/`
- [ ] D8: Pagination on list endpoints
- [ ] D9: Input validation hardening (all endpoints)
- [ ] D10: More agent presets (per user request)

---

## PARKED (Not in scope)

These are explicitly cut. Don't build them unless users demand them:

| Feature | Why Parked |
|---------|------------|
| WASM plugin runtime | MCP is the extension mechanism |
| 1,500+ skills catalog | Ship with 0; add when asked |
| Plugin marketplace | Need users first |
| Multi-CLI (Codex, Gemini) | Claude Code only for now |
| Telegram/Discord/email | Webhooks enough |
| ML usage prediction | Simple budget threshold |
| Cron scheduler | Manual for now |
| Kanban session view | Simple list with filters |
| Semantic security scanning | Glob pattern file protection |
| Dev environment | Post-1.0 if ever |
| Audit log + permissions | Post-1.0 |
| Secret management | Post-1.0 |

---

## Architecture Reference

```
forge-app (binary, serves everything)
  ├── forge-api (Axum HTTP + WebSocket + static files)
  │    ├── routes/health.rs     GET /api/v1/health
  │    ├── routes/agents.rs     CRUD /api/v1/agents
  │    ├── routes/sessions.rs   CRUD /api/v1/sessions + export
  │    ├── routes/run.rs        POST /api/v1/run
  │    ├── routes/ws.rs         GET /api/v1/ws (WebSocket)
  │    ├── error.rs             api_error(), parse_uuid()
  │    ├── state.rs             AppState (repos + event_bus)
  │    └── static_files.rs      rust-embed frontend serving (Phase A)
  ├── forge-process (Claude CLI spawning)
  │    ├── spawn.rs             ProcessHandle, SpawnConfig
  │    ├── parse.rs             stream-json line parser
  │    ├── runner.rs            ProcessRunner, EventBus integration
  │    └── stream_event.rs      StreamJsonEvent types
  ├── forge-db (SQLite persistence)
  │    ├── pool.rs              DbPool (WAL mode)
  │    ├── migrations.rs        Schema versioning
  │    ├── batch_writer.rs      Buffered event writes (50/2s)
  │    └── repos/               agents.rs, sessions.rs, events.rs
  ├── forge-agent (domain model)
  │    ├── model.rs             Agent, NewAgent, UpdateAgent
  │    ├── preset.rs            9 agent presets
  │    └── validation.rs        Input validation
  ├── forge-core (shared contract)
  │    ├── ids.rs               AgentId, SessionId, etc.
  │    ├── events.rs            22 ForgeEvent variants
  │    ├── event_bus.rs         Broadcast channel
  │    └── error.rs             ForgeError, ForgeResult
  ├── forge-safety (Phase B)    CircuitBreaker, RateLimiter, CostTracker
  └── forge-mcp (Phase B)       MCP server, 10 tools, 5 resources
```

---

## Post-Task Verification Checklist

Run after every task batch:

```bash
# 1. Rust compiles clean
cargo build --workspace

# 2. All tests pass
cargo test --workspace

# 3. No clippy warnings
cargo clippy --workspace -- -D warnings

# 4. Frontend builds (if frontend was touched)
cd frontend && pnpm build && cd ..

# 5. Binary runs
cargo run -p forge-app &
sleep 2
curl -sf http://127.0.0.1:4173/api/v1/health | jq .
kill %1
```
