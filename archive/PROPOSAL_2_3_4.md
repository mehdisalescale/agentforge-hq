# Implementation Proposal: Items 2, 3, 4 (P2/P3, MCP, Budget)

> **Per workflow: exact implementation detail for your explicit authorization before execution.**

---

## 2. P2/P3 code fixes

### 2.1 BatchWriter: persist event timestamp (P2)

- **Where:** `crates/forge-core/src/events.rs`, `crates/forge-db/src/batch_writer.rs`
- **Change:**
  1. Add to `ForgeEvent` in `forge_core::events`: a method `pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc>` that matches on each variant and returns its `timestamp` field.
  2. In `batch_writer::flush_to_db`, replace `let timestamp = chrono::Utc::now().to_rfc3339();` with `let timestamp = event.timestamp().to_rfc3339();` (use the event’s embedded timestamp).
- **Effect:** Stored event timestamps reflect when the event occurred, not flush time.

### 2.2 BatchWriter: use `transaction()` (P2)

- **Where:** `crates/forge-db/src/batch_writer.rs`
- **Change:** In `flush_to_db`, replace `conn.unchecked_transaction()` with `conn.transaction()`.
- **Effect:** Proper transaction semantics. If anything in the app relies on unchecked (e.g. nested transaction), tests will catch it; we can document and revert only if necessary.

### 2.3 Agents: preset serialization (P2)

- **Where:** `crates/forge-db/src/repos/agents.rs`
- **Current:** Create/update already use `serde_json::to_string(p).ok()` for preset. Read uses `serde_json::from_str(s).ok().or_else(|| parse_preset(s))`.
- **Change:** No code change for write path. Optionally add a short comment that preset is stored as JSON (serde) and `parse_preset` is fallback for legacy Debug-formatted rows. If you prefer, we can remove `parse_preset` and use only `serde_json::from_str` (breaking for any existing DB with old format).
- **Recommendation:** Comment only; keep `parse_preset` for backward compatibility.

### 2.4 Agents: UUID / timestamp parse errors (P2)

- **Where:** `crates/forge-db/src/repos/agents.rs`
- **Change:**
  1. Change `row_to_agent` signature from `fn row_to_agent(row: &rusqlite::Row<'_>) -> Result<Agent, rusqlite::Error>` to `Result<Agent, forge_core::ForgeError>`.
  2. For `id_str`: use `uuid::Uuid::parse_str(&id_str).map_err(|_| ForgeError::Validation(format!("invalid agent id: {}", id_str)))?` (and wrap in `AgentId`).
  3. For `created_at` / `updated_at`: use `DateTime::parse_from_rfc3339(...).map_err(|_| ForgeError::Validation(format!("invalid timestamp: {}", created_at)))?` (and `.with_timezone(&Utc)`).
  4. In `get` and `list`: use closure `|row| row_to_agent(&row).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))`; in `.map_err` map `InvalidParameterName(s)` to `ForgeError::Validation(s)` (e.g. pass through `ForgeError`, and map only `rusqlite::Error` from `query_row`/`query_map` to `ForgeError::Database`). So `row_to_agent` returns `Result<Agent, ForgeError>`, and call sites that use `query_row`/`query_map` need to distinguish: for `query_row` we have `stmt.query_row(..., row_to_agent).map_err(|e| match e { rusqlite::Error::QueryReturnedNoRows => ForgeError::AgentNotFound(...), other => ForgeError::Database(...) })` but now row_to_agent returns ForgeError. So we need to change to: get row, then row_to_agent(row).map_err(...). So we'll need to use `query_row` that returns Row and then call row_to_agent, or have row_to_agent return Result<Agent, Box<dyn Error>> or a unified error. Simpler: keep row_to_agent returning Result<Agent, rusqlite::Error> but map UUID parse to a different rusqlite error. Looking at rusqlite, there is no "invalid value" - we have InvalidParameterName. So the audit wants to avoid misusing InvalidParameterName. The only way is to have row_to_agent return something that can be ForgeError. So return Result<Agent, ForgeError>, and in get(): use stmt.query_row(...).map_err(...) to get Row, then row_to_agent(&row).map_err(identity)? - but query_row consumes the row and returns T from the closure. So we need a closure that returns Result<Agent, _>. If row_to_agent returns Result<Agent, ForgeError>, then we need get() to do something like: let row = stmt.query_row(..., |r| r)?; row_to_agent(&row).map_err(...)? But query_row doesn't work that way - it takes a fn(Row) -> Result<T, E>. So we do: stmt.query_row(params, |row| row_to_agent(row).map_err(|e| rusqlite::Error::ToSqlFailure(...)))? - that would convert ForgeError into rusqlite::Error which we then map to ForgeError::Database. That's ugly. Clean approach: change row_to_agent to return Result<Agent, ForgeError>. Then in get(), we do: let row = stmt.query_row(rusqlite::params![...], |r| Ok(r))?; row_to_agent(&row).map_err(api_error)? - but query_row expects the closure to return Result<T, rusqlite::Error>. So we can't return Ok(row) and then call row_to_agent outside - we need the closure to do the full conversion. So closure could be |row| row_to_agent(row).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string())) - then in get we map_err that to ForgeError. So we keep row_to_agent returning Result<Agent, ForgeError>, and in the query_row closure we call row_to_agent(row).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string())) so that query_row still returns Result<Agent, rusqlite::Error>. Then in get/list we map InvalidParameterName to the inner message... Actually the simplest is: row_to_agent returns Result<Agent, ForgeError>. get() does: let agent = stmt.query_row(..., |row| row_to_agent(row))?; But query_row requires the error type to be rusqlite::Error (from the closure). So we need a type that can hold both - or we use a wrapper. Easiest: row_to_agent returns Result<Agent, ForgeError>. In get we use: let mut stmt = ...; let row = stmt.query_row(..., |r| Ok(r))?; row_to_agent(&row).map_err(identity) - but query_row(..., |r| Ok(r)) returns Result<Row, _> and Row is not Clone and we need to pass a reference. Actually in rusqlite, the closure is FnOnce(Row) -> Result<T, Error>. So we get one Row. If we do query_row(..., |row| row_to_agent(&row)), we need row_to_agent to take &Row and return Result<Agent, ForgeError>. Then query_row expects Result<Agent, rusqlite::Error>. So we'd have to convert ForgeError to rusqlite::Error in the closure. So: row_to_agent(&row).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string())) and then in get().map_err(|e| match e { rusqlite::Error::QueryReturnedNoRows => ForgeError::AgentNotFound(id.clone()), rusqlite::Error::InvalidParameterName(msg) => ForgeError::Validation(msg), other => ForgeError::Database(Box::new(other)) }). That way we don't abuse InvalidParameterName for "invalid uuid" - we use it as a carrier for Validation message and then map it in get/list. So the fix: (1) row_to_agent returns Result<Agent, ForgeError>; (2) in row_to_agent, uuid parse error -> ForgeError::Validation("invalid agent id: ..."), datetime parse -> ForgeError::Validation("invalid timestamp: ..."); (3) in get and list, closure does row_to_agent(&row).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string())); map_err at get/list: InvalidParameterName(s) => try parse as Validation or keep as Database for simplicity - actually we can use a convention: if we have InvalidParameterName we could check if it starts with "invalid" and return ForgeError::Validation(s) else Database. Simpler: just map InvalidParameterName to ForgeError::Validation(s) in get/list when we get it from row_to_agent. So get: .map_err(|e| match e { QueryReturnedNoRows => AgentNotFound(id.clone()), InvalidParameterName(s) => ForgeError::Validation(s), other => Database(Box::new(other)) }). Same for list when we collect. Good.
- **Effect:** Callers get Validation errors for bad UUID/timestamp instead of misleading Database/InvalidParameterName.

### 2.5 Agent name validation: character set (P3)

- **Where:** `crates/forge-agent/src/validation.rs`
- **Change:** In `validate_new_agent` and `validate_update_agent`, add a check: agent name must contain only alphanumeric ASCII, hyphen, underscore (and optionally space). Per WHAT_TO_DO_NEXT: "alphanumeric, hyphen, underscore". So: `name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')` — if false, return `Err(ForgeError::Validation("Agent name may only contain letters, numbers, hyphens, and underscores".into()))`.
- **Effect:** Prevents problematic characters in agent names.

### 2.6 Re-export `validate_update_agent` (P3)

- **Where:** `crates/forge-agent/src/lib.rs`
- **Current:** Already has `pub use validation::{validate_new_agent, validate_update_agent};`
- **Change:** None.
- **Effect:** No change; already done.

---

## 3. MCP server (design + implement)

### 3.1 Design doc

- **Deliverable:** `docs/MCP_DESIGN.md`
- **Contents:**
  - Transport: stdio only (stdin read, stdout write, line-delimited or length-prefixed JSON-RPC 2.0).
  - Protocol: JSON-RPC 2.0 (method, params, id; result or error).
  - Initial tools (~10): `agent_list`, `agent_create`, `agent_get`, `agent_update`, `agent_delete`, `session_list`, `session_get`, `session_export`, `run_create` (start a run; optional: `run_status`, `run_cancel`). Exact list and params/result schemas (mirror existing API types where possible).
  - Error handling: map ForgeError to JSON-RPC error code and message.
  - No HTTP in v0.2.0; optional later: HTTP endpoint or spawn MCP binary per connection.

### 3.2 Implementation

- **Where:** `crates/forge-mcp` (extend) and optionally a new binary `crates/forge-mcp-bin` (or add `forge-mcp` as a binary in the same crate).
- **Dependencies:** forge-mcp will depend on forge-db, forge-agent, forge-core (and for run_create, forge-process + spawn; or defer run to a later iteration and implement only CRUD + session list/get/export first to avoid pulling in tokio/process in the MCP binary).
- **Approach A (minimal):** One binary `forge-mcp-bin` that: opens DB (FORGE_DB_PATH), creates AgentRepo + SessionRepo + EventRepo, runs a stdio loop: read line (or read until newline), parse JSON-RPC request, dispatch by method to repo methods, serialize response, write line to stdout. Implements agent_list, agent_get, agent_create, agent_update, agent_delete, session_list, session_get, session_export (and optionally run_create if we add spawn there). Sync I/O for simplicity (stdin/stdout).
- **Approach B:** Same but inside `forge-mcp` as a `run_server()` that takes a DB path and runs the loop; binary in forge-app that can run as `forge --mcp` or a separate `forge-mcp` binary. Prefer separate binary so MCP is runnable standalone (e.g. by IDE).
- **Concrete steps:**
  1. Add `docs/MCP_DESIGN.md` with tool list and request/response shapes.
  2. Add binary crate `forge-mcp-bin` (or binary target in forge-mcp) that links forge-mcp, forge-db, forge-agent, forge-core. No forge-api/forge-process initially (no run_create in first cut, or add run_create with spawn in same process).
  3. In forge-mcp: add `server` module with `run_stdio_loop(conn: Arc<Mutex<Connection>>)` or similar: loop { let line = read_line(stdin); let req: McpRequest = serde_json::from_str(&line)?; let result = dispatch(&req, &repos); let resp = McpResponse { ... }; write_line(stdout, serde_json::to_string(&resp)?); }
  4. Dispatch: match req.method, call agent_repo.list/create/get/update/delete or session_repo.list/get, session_export (from event_repo or existing export logic). Return JSON-RPC result or error.
  5. For session_export: reuse existing export logic (e.g. from forge-api routes or forge-db) so the MCP server can return JSON or Markdown.
- **Defer:** run_create (spawn) in MCP to a follow-up if it pulls in too much (tokio, process); first deliver CRUD + session list/get/export.

- **Effect:** IDE/CLI can drive Forge via stdio MCP with agents and sessions; run_create can be added next.

---

## 4. Budget enforcement (optional stretch)

- **Where:** `crates/forge-app/src/main.rs` (read env), `crates/forge-api` (run handler: after update_cost, compare and emit / block). Optionally `crates/forge-safety` for a small `BudgetChecker` type.
- **Env:** `FORGE_BUDGET_WARN` (f64, optional), `FORGE_BUDGET_LIMIT` (f64, optional). Default: none (disabled).
- **Change:**
  1. **forge-app:** Read `FORGE_BUDGET_WARN` and `FORGE_BUDGET_LIMIT` (parse as f64); pass into AppState (e.g. add `budget_warn: Option<f64>`, `budget_limit: Option<f64>` to a new `BudgetConfig` in state, or add to SafetyState).
  2. **forge-api state:** Add optional budget limits to `AppState` (e.g. `budget_warn: Option<f64>`, `budget_limit: Option<f64>`).
  3. **run handler (forge-api/routes/run.rs):** In the spawned task, after `session_repo.update_cost(&sid, cost)`: get current session (or use `cost` as current); if `state.budget_limit` is Some and cost >= limit, emit `ForgeEvent::BudgetExceeded { current_cost: cost, limit, timestamp }` and optionally block further runs (e.g. set a "budget exceeded" flag or return error on next run). If `state.budget_warn` is Some and cost >= warn (and cost < limit), emit `ForgeEvent::BudgetWarning { current_cost, limit: warn, timestamp }`.
  4. **README:** Add `FORGE_BUDGET_WARN` and `FORGE_BUDGET_LIMIT` to the configuration table.
- **Effect:** Operators can set soft (warn) and hard (limit) cost thresholds; events flow to UI/analytics; optional block when over limit.

---

## Execution order (after approval)

1. **P2 (BatchWriter + agents):** 2.1, 2.2, 2.3 (comment), 2.4, then 2.5. One commit per logical change (e.g. one commit for BatchWriter, one for agents row_to_agent, one for validation).
2. **P3:** 2.6 (no-op). 2.5 in same batch as P2.
3. **MCP:** 3.1 design doc, then 3.2 implementation (design commit, then impl commits).
4. **Budget:** 4 (single feature commit or split app/state + run handler + README).

---

## Authorization

If you approve this proposal, reply with explicit authorization (e.g. "Approved, proceed with 2, 3, and 4") and I will execute in the order above, with one commit per task and commit messages that state reason, decision, effect, and purpose.
