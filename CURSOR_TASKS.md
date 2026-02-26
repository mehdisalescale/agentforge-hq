# Forge Project — Cursor Agent Task List

> Generated: 2026-02-26
> Context: Phase 0 complete, Phase 1 ~70% done. 8 crates exist, frontend functional.
> Goal: Fix integration gaps left by parallel agent development, then finish Phase 1.

---

## How to Use This File

Each task is self-contained with:
- **What**: The problem
- **Where**: Exact file paths and line references
- **Why**: Impact if not fixed
- **How**: Step-by-step implementation instructions
- **Verify**: How to confirm the fix works

Work tasks in order (Critical → Medium → Low). Each task is independent unless noted.

---

## CRITICAL — Must Fix Before Phase 1 Ships

### C1: Remove rusqlite from forge-core (layering violation)

**What:** `forge-core` is the contract crate — every other crate depends on it. It currently pulls in `rusqlite` via `ForgeError::Database(#[from] rusqlite::Error)`. This means every crate that uses `ForgeError` transitively depends on rusqlite, even crates that have nothing to do with databases (forge-process, forge-mcp, forge-safety).

**Where:**
- `crates/forge-core/Cargo.toml` — line 14: `rusqlite = { version = "0.32", features = ["bundled"] }`
- `crates/forge-core/src/error.rs` — line 9: `Database(#[from] rusqlite::Error)`

**How:**
1. In `crates/forge-core/src/error.rs`, change the Database variant:
   ```rust
   // BEFORE
   #[error("Database error: {0}")]
   Database(#[from] rusqlite::Error),

   // AFTER
   #[error("Database error: {0}")]
   Database(String),
   ```
2. Remove `rusqlite` from `crates/forge-core/Cargo.toml` dependencies entirely.
3. In `crates/forge-db/src/repos/agents.rs`, update all places that return rusqlite errors:
   - `row_to_agent()` errors: wrap with `ForgeError::Database(e.to_string())`
   - `AgentRepo` methods that use `?` on rusqlite calls: add `.map_err(|e| ForgeError::Database(e.to_string()))`
4. In `crates/forge-db/src/repos/sessions.rs`, same treatment.
5. In `crates/forge-db/src/repos/events.rs`, same treatment.
6. In `crates/forge-db/src/pool.rs`, same treatment.
7. In `crates/forge-db/src/migrations.rs`, same treatment.
8. In `crates/forge-db/src/batch_writer.rs`, same treatment.
9. Search all `.rs` files for `ForgeError::Database` usage and ensure they pass `String` not `rusqlite::Error`.

**Verify:**
- `cargo build --workspace` succeeds
- `cargo test --workspace` passes
- `forge-core` no longer has rusqlite in its dependency tree: `cargo tree -p forge-core | grep rusqlite` returns nothing

---

### C2: Add Drop impl to ProcessHandle (prevent zombie processes)

**What:** `ProcessHandle` wraps a `tokio::process::Child` but has no `Drop` implementation. If a `ProcessHandle` is dropped without calling `kill()` or `wait()`, the child process becomes a zombie — it keeps running with no parent tracking it.

**Where:**
- `crates/forge-process/src/spawn.rs` — `ProcessHandle` struct

**How:**
1. Add a `Drop` impl that kills the child process:
   ```rust
   impl Drop for ProcessHandle {
       fn drop(&mut self) {
           if let Err(e) = self.child.start_kill() {
               tracing::warn!("failed to kill child process on drop: {}", e);
           }
       }
   }
   ```
2. This ensures that even if error handling skips explicit cleanup, the process is terminated.

**Verify:**
- Write a test that spawns a long-running process (`sleep 60`), drops the handle, and confirms the process is no longer running (check via `id()` before drop, then verify PID is gone).

---

### C3: Add configurable process timeout

**What:** There is no timeout on Claude CLI process execution. A runaway or hung process will block the session forever. The agent specified `max_turns` and `use_max` fields on agents but nothing enforces a time limit.

**Where:**
- `crates/forge-process/src/spawn.rs` — `SpawnConfig` struct
- `crates/forge-process/src/runner.rs` — `ProcessRunner`

**How:**
1. Add `timeout: Option<Duration>` to `SpawnConfig` with a default of 5 minutes:
   ```rust
   pub struct SpawnConfig {
       // ... existing fields ...
       pub timeout: Option<Duration>,
   }

   impl Default for SpawnConfig {
       fn default() -> Self {
           Self {
               // ... existing defaults ...
               timeout: Some(Duration::from_secs(300)),
           }
       }
   }
   ```
2. In the runner, when reading stdout lines, wrap the read loop with `tokio::time::timeout`:
   ```rust
   use tokio::time::timeout;

   let deadline = config.timeout.unwrap_or(Duration::from_secs(300));
   match timeout(deadline, read_and_process_loop).await {
       Ok(result) => result,
       Err(_) => {
           handle.kill().await?;
           self.emit(ForgeEvent::ProcessFailed {
               session_id: session_id.clone(),
               error: format!("Process timed out after {}s", deadline.as_secs()),
               timestamp: Utc::now(),
           });
           Err(ForgeError::Internal("process timeout".into()))
       }
   }
   ```

**Verify:**
- Test with a process that sleeps longer than the timeout. Confirm it is killed and a `ProcessFailed` event is emitted with a timeout message.

---

### C4: Fix WebSocket task coordination

**What:** The WebSocket handler uses `tokio::select!` on two spawned tasks (send and recv). If the recv task completes (client disconnects), the send task is aborted mid-flight — potentially losing buffered data. Neither task is explicitly cancelled on the other's completion.

**Where:**
- `crates/forge-api/src/routes/ws.rs` — `handle_socket()` function

**How:**
1. Replace the current pattern with proper abort handles:
   ```rust
   async fn handle_socket(socket: WebSocket, state: AppState) {
       let (mut sender, mut receiver) = socket.split();
       let mut bus_rx = state.event_bus.subscribe();

       let send_task = tokio::spawn(async move {
           while let Ok(event) = bus_rx.recv().await {
               match serde_json::to_string(&event) {
                   Ok(json) => {
                       if sender.send(Message::Text(json)).await.is_err() {
                           break;
                       }
                   }
                   Err(e) => {
                       tracing::warn!("failed to serialize event: {}", e);
                   }
               }
           }
       });

       let recv_task = tokio::spawn(async move {
           while let Some(Ok(_msg)) = receiver.next().await {
               // Consume client messages to detect disconnect
           }
       });

       // Wait for either task to finish, then abort the other
       tokio::select! {
           _ = send_task => {
               recv_task.abort();
           }
           _ = recv_task => {
               send_task.abort();
           }
       }

       tracing::debug!("WebSocket connection closed");
   }
   ```
2. The key change: explicitly `.abort()` the other task instead of just dropping it.
3. Add logging for serialization failures (was previously silent).

**Verify:**
- Connect a WebSocket client, disconnect it, confirm no error logs about panicked tasks.
- Connect a client, send events through EventBus, confirm they arrive as JSON.

---

## MEDIUM — Should Fix Before Phase 1 Complete

### M1: Add SessionRepo::update_status()

**What:** SessionRepo can create and delete sessions, but has no method to update status. The session status field (`created`, `running`, `completed`, `failed`, `cancelled`) is set to `created` on insert and can never change. The Run page creates sessions but can't transition them through their lifecycle.

**Where:**
- `crates/forge-db/src/repos/sessions.rs` — `SessionRepo` impl

**How:**
1. Add an `update_status` method:
   ```rust
   pub fn update_status(&self, id: &SessionId, status: &str) -> ForgeResult<()> {
       let valid = ["created", "running", "completed", "failed", "cancelled"];
       if !valid.contains(&status) {
           return Err(ForgeError::Validation(format!("invalid session status: {}", status)));
       }
       let conn = self.conn.lock().expect("db mutex poisoned");
       let now = Utc::now().to_rfc3339();
       let rows = conn.execute(
           "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
           rusqlite::params![status, now, id.0.to_string()],
       )?;
       if rows == 0 {
           return Err(ForgeError::SessionNotFound(id.clone()));
       }
       Ok(())
   }
   ```
2. Also add `update_claude_session_id` for storing the Claude CLI session ID after process start:
   ```rust
   pub fn update_claude_session_id(&self, id: &SessionId, claude_id: &str) -> ForgeResult<()> {
       let conn = self.conn.lock().expect("db mutex poisoned");
       let now = Utc::now().to_rfc3339();
       conn.execute(
           "UPDATE sessions SET claude_session_id = ?1, updated_at = ?2 WHERE id = ?3",
           rusqlite::params![claude_id, now, id.0.to_string()],
       )?;
       Ok(())
   }
   ```
3. Wire these into the process runner lifecycle:
   - On `ProcessStarted` → `update_status(id, "running")`
   - On `ProcessCompleted` → `update_status(id, "completed")`
   - On `ProcessFailed` → `update_status(id, "failed")`

**Verify:**
- Add a test: create session, update status to "running", fetch it, assert status changed.
- Test invalid status string is rejected.

---

### M2: Add WebSocket auto-reconnect on frontend

**What:** The dashboard declares `wsReconnect` but never assigns it. If the WebSocket connection drops (server restart, network blip), the user sees a dead UI with no recovery.

**Where:**
- `frontend/src/routes/+page.svelte` — `connectWs()` function

**How:**
1. Add reconnect logic with exponential backoff:
   ```typescript
   let reconnectAttempts = 0;
   const MAX_RECONNECT_DELAY = 16000;

   function connectWs() {
       const url = wsUrl('/api/v1/ws');
       streamStatusDetail = 'Connecting...';
       ws = new WebSocket(url);

       ws.onopen = () => {
           reconnectAttempts = 0;
           streamStatusDetail = 'Connected';
       };

       ws.onclose = () => {
           ws = null;
           scheduleReconnect();
       };

       ws.onerror = () => {
           streamStatusDetail = 'Connection error';
       };

       ws.onmessage = (event) => {
           // ... existing message handling ...
       };
   }

   function scheduleReconnect() {
       const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), MAX_RECONNECT_DELAY);
       reconnectAttempts++;
       streamStatusDetail = `Reconnecting in ${delay / 1000}s...`;
       setTimeout(connectWs, delay);
   }
   ```
2. Call `connectWs()` on mount (already done).
3. Clean up WebSocket on component destroy.

**Verify:**
- Start the app, connect WebSocket, stop the server, confirm UI shows "Reconnecting..." with increasing delays. Restart server, confirm reconnection succeeds.

---

### M3: Log event emission failures

**What:** All agent CRUD handlers use `let _ = state.event_bus.emit(...)` which silently swallows any emission failure. If the event bus is broken, no one knows.

**Where:**
- `crates/forge-api/src/routes/agents.rs` — lines with `let _ = state.event_bus.emit`

**How:**
1. Replace all occurrences:
   ```rust
   // BEFORE
   let _ = state.event_bus.emit(ForgeEvent::AgentCreated { ... });

   // AFTER
   if let Err(e) = state.event_bus.emit(ForgeEvent::AgentCreated { ... }) {
       tracing::warn!("failed to emit AgentCreated event: {}", e);
   }
   ```
2. Do the same in any other file that uses `let _ = ... .emit(...)`.

**Verify:**
- `cargo clippy` should no longer warn about unused results (if it did before).
- Add `tracing` to the imports if not already present.

---

### M4: Extract UUID parsing helper

**What:** Every handler in agents.rs and sessions.rs has inline UUID parsing with duplicate error response construction. This is copy-paste code that should be a shared helper.

**Where:**
- `crates/forge-api/src/routes/agents.rs` — get_agent, update_agent, delete_agent
- `crates/forge-api/src/routes/sessions.rs` — get_session, delete_session, export_session

**How:**
1. Create a helper in `crates/forge-api/src/error.rs`:
   ```rust
   use uuid::Uuid;

   pub fn parse_uuid(s: &str) -> Result<Uuid, Response> {
       Uuid::parse_str(s).map_err(|_| {
           (
               StatusCode::BAD_REQUEST,
               Json(ErrorBody {
                   error: format!("invalid uuid: {}", s),
                   code: "invalid_id".to_string(),
               }),
           )
               .into_response()
       })
   }
   ```
2. Replace all inline UUID parsing in handlers with `let id = parse_uuid(&id_str)?;`.
3. Remove the per-file `bad_request()` helper in sessions.rs (consolidate into error.rs).

**Verify:**
- All handlers still return 400 with `{"error": "invalid uuid: ...", "code": "invalid_id"}` for bad UUIDs.
- `cargo test` passes.

---

### M5: Surface ToolUse and ToolResult events

**What:** The process runner's `content_block_text()` returns `None` for `ToolUse` and `ToolResult` blocks. This means when Claude calls a tool, the user sees nothing in the stream — tool invocations are invisible.

**Where:**
- `crates/forge-process/src/runner.rs` — `content_block_text()` function and `emit_parsed_event()`

**How:**
1. Update `emit_parsed_event` to handle tool blocks separately:
   ```rust
   // In the Assistant arm of emit_parsed_event, after collecting text blocks:
   for block in content_blocks {
       match block {
           ContentBlock::Text { text } => {
               self.emit(ForgeEvent::ProcessOutput {
                   session_id: session_id.clone(),
                   kind: OutputKind::Assistant,
                   content: text.clone(),
                   timestamp: Utc::now(),
               });
           }
           ContentBlock::Thinking { thinking } => {
               self.emit(ForgeEvent::ProcessOutput {
                   session_id: session_id.clone(),
                   kind: OutputKind::Thinking,
                   content: thinking.clone(),
                   timestamp: Utc::now(),
               });
           }
           ContentBlock::ToolUse { id, name, input } => {
               let content = format!("Tool: {} ({})\n{}", name, id, serde_json::to_string_pretty(&input).unwrap_or_default());
               self.emit(ForgeEvent::ProcessOutput {
                   session_id: session_id.clone(),
                   kind: OutputKind::ToolUse,
                   content,
                   timestamp: Utc::now(),
               });
           }
           ContentBlock::ToolResult { tool_use_id, content, is_error } => {
               let label = if *is_error { "Tool Error" } else { "Tool Result" };
               let content = format!("{} ({}): {}", label, tool_use_id, content);
               self.emit(ForgeEvent::ProcessOutput {
                   session_id: session_id.clone(),
                   kind: OutputKind::ToolResult,
                   content,
                   timestamp: Utc::now(),
               });
           }
       }
   }
   ```
2. Remove the old `content_block_text()` helper function (no longer needed).

**Verify:**
- Run an agent that uses tools (e.g., Reviewer preset reading a file). Confirm `ToolUse` and `ToolResult` events appear in the WebSocket stream and on the frontend Run page.

---

### M6: Add error logging to api_error()

**What:** The `api_error()` function converts `ForgeError` to HTTP JSON responses but never logs the error server-side. When debugging production issues, there's no trace of what went wrong.

**Where:**
- `crates/forge-api/src/error.rs` — `api_error()` function

**How:**
1. Add a tracing log before returning:
   ```rust
   pub fn api_error(e: ForgeError) -> Response {
       let (status, code) = match &e {
           // ... existing match arms ...
       };

       tracing::error!(
           status = status.as_u16(),
           code = code,
           error = %e,
           "API error"
       );

       (
           status,
           Json(ErrorBody {
               error: e.to_string(),
               code: code.to_string(),
           }),
       )
           .into_response()
   }
   ```

**Verify:**
- Hit an endpoint with an invalid agent ID. Confirm the error appears in server logs AND in the JSON response.

---

## LOW — Cleanup and Polish

### L1: Increase EventBus capacity

**Where:** `crates/forge-app/src/main.rs` — `EventBus::new(16)`

**Change:** Increase to 1024:
```rust
let event_bus = Arc::new(EventBus::new(1024));
```

**Why:** 16 is too small. If a process emits events faster than subscribers consume, events are dropped silently via broadcast channel lag.

---

### L2: Add configurable port and host

**Where:** `crates/forge-app/src/main.rs` — hardcoded `127.0.0.1:4173`

**How:** Read from env vars with defaults:
```rust
let host = std::env::var("FORGE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
let port = std::env::var("FORGE_PORT").unwrap_or_else(|_| "4173".to_string());
let addr = format!("{}:{}", host, port);
```

---

### L3: Remove unused RunnerStubEvent export

**Where:** `crates/forge-process/src/lib.rs` — line 10

**Change:** Remove `StreamJsonEvent as RunnerStubEvent` from the pub use line. It's exported but never imported anywhere.

---

### L4: Add TraceLayer middleware

**Where:** `crates/forge-api/src/lib.rs` — router construction

**How:** Add tower-http tracing:
```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .nest("/api/v1", api_routes)
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());  // ADD THIS
```

**Why:** Every HTTP request/response gets logged with method, path, status, and duration. Essential for debugging.

---

### L5: Add session list filtering by agent_id

**Where:** `crates/forge-db/src/repos/sessions.rs` — `SessionRepo`

**How:** Add a `list_by_agent` method:
```rust
pub fn list_by_agent(&self, agent_id: &AgentId) -> ForgeResult<Vec<Session>> {
    let conn = self.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, claude_session_id, directory, status, created_at, updated_at
         FROM sessions WHERE agent_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![agent_id.0.to_string()], row_to_session)?;
    let sessions = rows.collect::<Result<Vec<_>, _>>().map_err(|e| ForgeError::Database(e.to_string()))?;
    Ok(sessions)
}
```

**Why:** The sessions page and agent detail view will need to filter sessions by agent. Currently `list()` returns all sessions with no filtering.

---

### L6: Add index on sessions.agent_id

**Where:** `migrations/0001_init.sql`

**How:** Add after the sessions table:
```sql
CREATE INDEX IF NOT EXISTS idx_sessions_agent ON sessions(agent_id);
```

**Note:** Since the migration is v1 and may already be applied to existing databases, either:
- Add as migration v2 (new file `migrations/0002_session_index.sql`)
- Or add to v1 if no production databases exist yet

---

### L7: Log System and User events instead of silently dropping

**Where:** `crates/forge-process/src/runner.rs` — `emit_parsed_event()`

**Change:**
```rust
// BEFORE
ParsedEvent::System(_) => return Ok(()),
ParsedEvent::User(_) => return Ok(()),

// AFTER
ParsedEvent::System(payload) => {
    tracing::debug!(session_id = %session_id, "system event (session init): {:?}", payload.session_id);
    return Ok(());
}
ParsedEvent::User(_) => {
    tracing::debug!(session_id = %session_id, "user event (ignored in stream)");
    return Ok(());
}
```

---

## Post-Fix Verification Checklist

After completing all tasks, run these checks:

```bash
# 1. Build succeeds
cargo build --workspace

# 2. All tests pass
cargo test --workspace

# 3. No clippy warnings
cargo clippy --workspace -- -D warnings

# 4. forge-core has no rusqlite dependency
cargo tree -p forge-core | grep rusqlite
# Should return nothing

# 5. Binary runs and serves UI
cargo run -p forge-app
# Open http://127.0.0.1:4173 — UI should load

# 6. Agent CRUD works
curl http://127.0.0.1:4173/api/v1/health
curl -X POST http://127.0.0.1:4173/api/v1/agents \
  -H 'Content-Type: application/json' \
  -d '{"name":"Test","model":"claude-sonnet-4-20250514"}'

# 7. WebSocket streams events
websocat ws://127.0.0.1:4173/api/v1/ws
# Should receive JSON events
```

---

## Architecture Reference

```
forge-app (binary)
  ├── forge-api (Axum HTTP + WebSocket)
  │    ├── routes/health.rs
  │    ├── routes/agents.rs
  │    ├── routes/sessions.rs
  │    └── routes/ws.rs
  ├── forge-process (Claude CLI spawning)
  │    ├── spawn.rs (ProcessHandle, SpawnConfig)
  │    ├── parse.rs (stream-json line parser)
  │    ├── runner.rs (ProcessRunner, EventBus integration)
  │    └── stream_event.rs (StreamJsonEvent types)
  ├── forge-db (SQLite persistence)
  │    ├── pool.rs (DbPool, WAL mode)
  │    ├── migrations.rs (schema versioning)
  │    ├── batch_writer.rs (buffered event writes)
  │    └── repos/ (agents.rs, sessions.rs, events.rs)
  ├── forge-agent (domain model)
  │    ├── model.rs (Agent, NewAgent, UpdateAgent)
  │    ├── preset.rs (9 agent presets)
  │    └── validation.rs (input validation)
  ├── forge-core (shared contract)
  │    ├── ids.rs (AgentId, SessionId, etc.)
  │    ├── events.rs (20 ForgeEvent variants)
  │    ├── event_bus.rs (broadcast channel)
  │    └── error.rs (ForgeError, ForgeResult)
  ├── forge-safety (stub — Phase 4)
  └── forge-mcp (stub — Phase 4)
```
