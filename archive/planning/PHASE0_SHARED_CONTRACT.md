# Phase 0 — Shared Contract (Multi-Agent)

> **Every Cursor agent working on Phase 0 must read this document first and adhere to it.**
> This is the single source of truth. Do not change types, route shapes, or table names without updating this contract and syncing with other tracks.

---

## 1. Purpose

When multiple agents work Phase 0 in parallel, they need one shared contract so that:

- **forge-core** defines types that **forge-db**, **forge-api**, and **forge-agent** use without drift.
- **forge-api** and **frontend** agree on request/response JSON.
- **forge-db** uses the approved schema; no one else edits migration SQL.
- No two agents edit the same file or crate without a defined handoff.

If you are an agent assigned to a track: **read this contract, then read PHASE0_PARALLEL_TRACKS.md for your track. Do not invent new variants, routes, or columns.**

---

## 2. Crate Layout (Approved)

```
Cargo.toml
Makefile
crates/
  forge-core/
  forge-db/
  forge-api/
  forge-agent/
  forge-process/
  forge-safety/
  forge-mcp/
  forge-app/
frontend/
migrations/
  0001_init.sql
```

Workspace `members = ["crates/*"]`. Shared deps in `[workspace.dependencies]`: tokio, axum, serde, serde_json, rusqlite (bundled, vtab, fts5), uuid, chrono, thiserror, tracing, clap, rust-embed, crossbeam-channel (and any in PHASE0_IMPLEMENTATION_PLAN workspace snippet).

---

## 3. Type Contract (forge-core owns)

All crates that need these types **depend on forge-core** and use these definitions. Do not duplicate or redefine.

### 3.1 ID types (newtype wrappers around Uuid)

| Type      | Use |
|-----------|-----|
| `AgentId` | Agents, sessions |
| `SessionId` | Sessions, events |
| `EventId` | Events (if needed for persistence ID) |
| `WorkflowId` | Workflows (Phase 2) |
| `SkillId` | Skills (Phase 2) |

All: `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`. String representation for JSON/DB: same as inner Uuid.

### 3.2 ForgeError (thiserror, in forge-core)

Variants (exact): `Database(rusqlite::Error)`, `Serialization(serde_json::Error)`, `AgentNotFound(AgentId)`, `SessionNotFound(SessionId)`, `Validation(String)`, `EventBus(String)`, `Io(std::io::Error)`, `Internal(String)`.

Type alias: `ForgeResult<T> = Result<T, ForgeError>`.

Use `anyhow` only in **forge-app** for `main()`; library crates use `ForgeResult` or map into `ForgeError`.

### 3.3 ForgeEvent enum (exact variants)

Serde: `#[serde(tag = "type", content = "data")]`. All variants include `timestamp: DateTime<Utc>` where applicable.

| Variant | Phase 0 emit? | Payload (in addition to timestamp where noted) |
|---------|----------------|------------------------------------------------|
| SystemStarted | Yes | version: String |
| SystemStopped | Yes | — |
| Heartbeat | Yes | — |
| AgentCreated | Yes | agent_id, name |
| AgentUpdated | Yes | agent_id, name |
| AgentDeleted | Yes | agent_id |
| ProcessStarted | No | session_id, agent_id |
| ProcessOutput | No | session_id, kind (OutputKind), content |
| ProcessCompleted | No | session_id, exit_code |
| ProcessFailed | No | session_id, error |
| SessionCreated | No | session_id, agent_id, directory |
| SessionResumed | No | session_id |
| WorkflowStarted | No | workflow_id |
| WorkflowStepCompleted | No | workflow_id, step |
| WorkflowCompleted | No | workflow_id |
| WorkflowFailed | No | workflow_id, error |
| CircuitBreakerTripped | No | agent_id, reason |
| BudgetWarning | No | current_cost, limit |
| BudgetExceeded | No | current_cost, limit |
| Error | Yes | message, context: Option<String> |

`OutputKind`: Assistant | ToolUse | ToolResult | Thinking | Result.

### 3.4 EventBus and EventSink

- **EventBus:** `new(capacity: usize)`, `emit(&self, event: ForgeEvent) -> ForgeResult<()>`, `subscribe(&self) -> broadcast::Receiver<ForgeEvent>`.
- **EventSink:** trait `fn handle(&self, event: &ForgeEvent)` (Send + Sync). Used for batch writer and tests.

---

## 4. API Contract (forge-api owns implementation; frontend consumes)

Base path: `/api/v1`. All JSON. CORS and request-id middleware applied.

| Method | Path | Request body | Response | Status |
|--------|------|--------------|----------|--------|
| GET | /health | — | `{ status, version, uptime?, db_ok? }` | 200 |
| GET | /agents | — | `Agent[]` | 200 |
| POST | /agents | `NewAgent` | `Agent` | 201 |
| GET | /agents/:id | — | `Agent` | 200 / 404 |
| PUT | /agents/:id | `UpdateAgent` | `Agent` | 200 / 404 |
| DELETE | /agents/:id | — | (empty) | 204 / 404 |
| GET | /ws | — | WebSocket upgrade | 101 |

**Agent (response / GET):** id (string UUID), name, model, system_prompt?, allowed_tools?, max_turns?, use_max, preset?, config?, created_at (RFC 3339), updated_at (RFC 3339).

**NewAgent (POST):** name (required), model?, system_prompt?, preset?, allowed_tools?, max_turns?, use_max?.

**UpdateAgent (PUT):** name?, model?, system_prompt?, preset?, allowed_tools?, max_turns?, use_max? (all optional; omit field = no change).

**WebSocket:** Each message is a JSON-serialized `ForgeEvent` (same tag+content as in §3.3). First message after connect: `SystemStarted`. Then stream of events; optional heartbeat `Heartbeat` every ~30s.

Validation errors (e.g. empty name): 422 with body `{ error: string }` or similar. 404: no body or minimal.

---

## 5. Schema Contract (forge-db owns)

Single migration: `migrations/0001_init.sql`. WAL, foreign_keys ON.

**Tables (names and purpose; exact columns in approved DDL):** schema_version, agents, sessions, events, workflows, workflow_runs, skills, schedules, audit_log, config. FTS5: skills_fts, sessions_fts, events_fts.

**agents:** id (TEXT PK), name (TEXT UNIQUE NOT NULL), model, system_prompt, allowed_tools (JSON), max_turns, use_max (BOOLEAN), preset, config_json, created_at, updated_at.

**events:** id (TEXT PK), session_id (FK), agent_id (FK), event_type (TEXT), data_json (TEXT), timestamp. event_type must match ForgeEvent variant name; data_json = serialized event payload.

Only **forge-db** track creates or edits `migrations/` and repository code. Other crates never write SQL.

---

## 6. Agent Type (forge-agent owns; must match API and DB)

**Agent:** id (AgentId), name, model, system_prompt?, allowed_tools?, max_turns?, use_max, preset?, config?, created_at, updated_at.

**AgentPreset (enum):** CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer.

**NewAgent / UpdateAgent:** As in §4. Validation rules: name non-empty, length limit, model in allowlist. Validation errors map to `ForgeError::Validation`.

forge-db **AgentRepo** maps Agent to/from `agents` table; forge-api handlers use AgentRepo and return Agent as JSON per §4.

---

## 7. File Ownership (do not cross without handoff)

| Crate / area | Owner track | Other agents |
|--------------|-------------|--------------|
| crates/forge-core/* | Track Core | Only depend on it; do not edit |
| crates/forge-db/*, migrations/* | Track DB | Only depend on forge-db; do not edit |
| crates/forge-agent/* | Track Agent | Only depend on it; do not edit |
| crates/forge-api/* | Track API | Frontend consumes API contract only |
| frontend/* | Track Frontend | Do not edit from backend tracks |
| crates/forge-app/* | Track App | Edits after other crates exist |
| Cargo.toml (workspace), Makefile | Gate 1 / bootstrap | One agent or human; then frozen for Phase 0 |

---

## 8. When to Update This Contract

- **Before parallel work starts:** Contract is filled (and Gates 1 & 2 approved). All agents clone or read the same version.
- **If a track needs a new variant or route:** Propose change in this doc; get alignment; then implement. Do not add a ForgeEvent variant or API route without adding it here first.
- **After Phase 0:** This doc can be versioned or superseded by OpenAPI/schema docs; for Phase 0 it is the source of truth.

---

**Summary for agents:** Read this. Implement only what your track owns. Use types and routes exactly as above. Do not edit another track’s files. Refer to PHASE0_PARALLEL_TRACKS.md for your track and dependencies.
