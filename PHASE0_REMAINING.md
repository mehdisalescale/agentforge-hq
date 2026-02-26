# Phase 0 Remaining Work

> Phase 0 is **~40% done**. This doc is the ordered checklist for the next 3–4 sessions.
> After Phase 0, the roadmap continues with **Phase 1: Agent Engine** (weeks 5–8) — agent CRUD API wired to UI, process spawning, real-time streaming, session management.

---

## Progress

- **Done:** forge-core, forge-agent, forge-db (3/8 crates), full schema, migrations, batch writer, AgentRepo, EventRepo, FTS5 stubs
- **Remaining:** foundational fixes → 5 new crates → frontend shell + single binary

---

## 1. Fix foundational issues first (1 session)

- [ ] **Extract rusqlite from forge-core** — Move `ForgeError::Database` to a forge-db–specific error, or use `Box<dyn Error>` in core. Unblocks clean layering for all other crates.
- [ ] **Add `[workspace.dependencies]`** — Centralize tokio, serde, chrono, rusqlite, uuid versions in root `Cargo.toml`; use in crates.
- [ ] **Export + wire `validate_update_agent`** — Add to `forge-agent` lib.rs exports; call it in `AgentRepo::update()`.
- [ ] **Fix preset serialization** — Use serde `Serialize`/`Deserialize` string representation instead of `Debug`.

---

## 2. Build remaining 5 crates (1–2 sessions)

- [ ] **forge-process** — Claude CLI process spawning, stream-json parsing (stubs OK for Phase 0).
- [ ] **forge-safety** — Circuit breaker, rate limiter, cost tracker (stubs).
- [ ] **forge-mcp** — MCP protocol types (stubs).
- [ ] **forge-api** — Axum router, health check, agent CRUD endpoints, WebSocket handler.
- [ ] **forge-app** — Binary entry point, ties everything together, serves frontend.

---

## 3. Frontend shell + single binary (1 session)

- [x] **SvelteKit project** with `adapter-static` — `frontend/`, output in `frontend/build/` (Agent B).
- [x] **Layout shell:** sidebar, main content area, status bar (Agent B).
- [x] **Empty pages:** Dashboard, Agents, Sessions, Workflows, Skills, Settings (Agent B).
- [ ] **rust-embed** to serve frontend from the binary (Agent C — use `frontend/build/`).
- [ ] **`cargo build --release`** produces working single binary.

---

## Phase 0 done when

- [x] `./forge` starts; browser shows UI shell at `http://localhost:4173`
- [x] WebSocket connects and receives heartbeat events
- [x] All 8 crates compile; `cargo test` + `cargo clippy` clean
- [x] Health check `GET /api/v1/health` returns OK

**Verification:** See [CHECKLIST_RESULTS.md](CHECKLIST_RESULTS.md). Runnable stack is in **claude-forge**; forge-project holds planning and crate stubs.

---

## After Phase 0

**Phase 1: Agent Engine** (weeks 5–8) — real functionality: agent CRUD API wired to the UI, process spawning, real-time streaming, session management.

**Phase 1 design notes:** When doing **forge-process** (real process spawn, stream-json), **MCP/server**, and **skills/workflows**, use **claude** CLI behavior and **claude-flow** ADRs/plugin design; use **Agent-Skills-for-Context-Engineering** and **superpowers** for skill content and spec→TDD workflow. Full map: [08-reference/TREND_26FEB_ENHANCEMENT_MAP.md](08-reference/TREND_26FEB_ENHANCEMENT_MAP.md). Details: [docs/PHASE1_DESIGN_NOTES.md](docs/PHASE1_DESIGN_NOTES.md).

---

## 4-agent parallel (no idle agents)

Split work so **all 4 agents** have tasks each round. Merge after each round; run `cargo build --workspace && cargo test --workspace` before starting the next.

| Round | Agent A | Agent B | Agent C | Agent D |
|-------|---------|---------|---------|---------|
| **1** | Extract rusqlite from forge-core | Add `[workspace.dependencies]` (root + forge-agent, forge-db) | Export + wire `validate_update_agent` **and** fix preset serialization | **forge-process** stub crate |
| **2** | **forge-safety** stub | **forge-mcp** stub | **forge-api** (Axum, health, agent CRUD, WebSocket) | **Frontend shell** (SvelteKit, layout, empty pages) |
| **3** | **forge-app** (binary) | **Frontend shell** (SvelteKit) | **rust-embed** + single binary | **Phase 0 checklist** + 26-feb design notes |

**Round 1:** A/B/C do foundational fixes; D adds a new crate so it isn’t blocked. C does both “wire validate_update_agent” and “preset serialization” (same file `agents.rs`).  
**Round 2:** Four crates in parallel — 3 stub crates + forge-api; D does frontend only (no Rust).  
**Round 3:** A = forge-app binary, B = frontend shell, C = rust-embed + single binary, D = Phase 0 checklist + 26-feb design notes. **Prompts:** [NEXT_PHASE_AGENT_PROMPTS.md](NEXT_PHASE_AGENT_PROMPTS.md) (includes 26-feb enhancement summary for design/skills).

Copy-paste prompts for Round 1 and Round 2 are below.

### Round 1 prompts (run all 4 in parallel)

**Agent A — Extract rusqlite from forge-core**
```
You are Agent A. In forge-project, do only: extract rusqlite from forge-core.

1. In forge-core: change ForgeError so there is no rusqlite type (e.g. ForgeError::Database(Box<dyn std::error::Error + Send + Sync>) or a String). Update any code in forge-core that constructs or matches Database.
2. Remove rusqlite from forge-core/Cargo.toml dependencies so forge-core does not depend on rusqlite.
3. In forge-db, where you currently convert rusqlite errors to ForgeError::Database, keep returning an error type that forge-db defines or map into the new core variant (e.g. .map_err(|e| ForgeError::Database(Box::new(e)))).
Run cargo build --workspace and cargo test --workspace. Commit with message that states: reason (clean layering), decision (core has no rusqlite), effect (forge-db owns DB errors).
```

**Agent B — workspace.dependencies**
```
You are Agent B. In forge-project, do only: add [workspace.dependencies] and use them in crates.

1. In root Cargo.toml add [workspace.dependencies] with: tokio, serde, serde_json, chrono, uuid, rusqlite (with features bundled, vtab, fts5 for rusqlite). Do not add forge-core to workspace members change in this task — only add the [workspace.dependencies] block and ensure members list includes existing crates.
2. In forge-agent and forge-db Cargo.toml, replace direct dependency versions with workspace = true for tokio, serde, chrono, uuid, rusqlite (forge-db only) where applicable.
3. Do not edit forge-core/Cargo.toml in this task (Agent A may be editing forge-core).
Run cargo build --workspace and cargo test --workspace. Commit with message: reason (centralize deps), decision (workspace.dependencies), effect (crates use workspace).
```

**Agent C — validate_update_agent + preset serialization**
```
You are Agent C. In forge-project, do both in one change:

1. Export validate_update_agent from forge-agent: add it to lib.rs pub use or pub fn so it is part of the public API.
2. In forge-db AgentRepo::update(), call validate_update_agent on the update payload before applying; return an error if validation fails.
3. Fix preset serialization in forge-db repos/agents.rs: store and load preset using a stable format (serde Serialize/Deserialize to string, or a fixed enum string like "CodeWriter", "Reviewer", …), not Debug format.
Run cargo build --workspace and cargo test --workspace. Commit with message: reason (validation and stable preset), decision (export + wire + serde), effect (update validated, preset round-trips).
```

**Agent D — forge-process stub**
```
You are Agent D. In forge-project, create the forge-process crate only (stub).

1. Create crates/forge-process/ with Cargo.toml that depends on forge-core (path = "../forge-core"). Add forge-process to root Cargo.toml members.
2. Add src/lib.rs with minimal public types: e.g. ProcessHandle, StreamJsonEvent (or similar) as stubs (structs with no or minimal fields). No real process spawning or stream-json parsing yet — just types so the crate compiles and can be used later.
3. Add src/error.rs or use forge_core::ForgeError if appropriate. Keep it compiling.
Run cargo build --workspace and cargo test --workspace. Commit with message: reason (Phase 0 crate set), decision (forge-process stub), effect (4th crate compiles).
```

### Round 2 prompts (run all 4 in parallel; start after Round 1 is merged)

**Agent A — forge-safety stub**
```
You are Agent A. In forge-project, create the forge-safety crate (stub). Depends on forge-core. Add to root Cargo.toml members. Minimal public types: e.g. CircuitBreaker, RateLimiter, CostTracker as empty or stub structs. No real logic — crate compiles. Commit when done.
```

**Agent B — forge-mcp stub**
```
You are Agent B. In forge-project, create the forge-mcp crate (stub). Depends on forge-core. Add to root Cargo.toml members. Minimal public types for MCP protocol (e.g. McpRequest, McpResponse or tool/resource type stubs). No real JSON-RPC — crate compiles. Commit when done.
```

**Agent C — forge-api**
```
You are Agent C. In forge-project, create the forge-api crate. Axum server with: GET /api/v1/health (returns OK), GET/POST/PUT/DELETE /api/v1/agents using forge-db::AgentRepo and forge_agent::{Agent, NewAgent, UpdateAgent}. WebSocket at GET /api/v1/ws that can broadcast events from forge_core::EventBus. Add forge-api to root Cargo.toml. No frontend yet. Commit when cargo build --workspace and cargo test --workspace pass and health check responds.
```

**Agent D — Frontend shell**
```
You are Agent D. In forge-project, create the frontend shell only (no Rust in this task). SvelteKit app with adapter-static, e.g. in frontend/. Layout: sidebar, main content area, status bar. Empty placeholder pages: Dashboard, Agents, Sessions, Workflows, Skills, Settings. Build so it produces static files (npm run build or similar). Do not wire into the binary yet — that is Round 3. Commit when the frontend builds and layout is visible.
```

### Round 3 prompts (4 agents; after Round 2 merged)

Use **[NEXT_PHASE_AGENT_PROMPTS.md](NEXT_PHASE_AGENT_PROMPTS.md)**. It has shared context (including **26-feb enhancement summary**), then one prompt each for:
- **Agent A** — forge-app binary (main, migrations, API server)
- **Agent B** — Frontend shell (SvelteKit, adapter-static, layout, empty pages)
- **Agent C** — rust-embed + single binary (serve frontend from forge-app)
- **Agent D** — Phase 0 checklist + Phase 1 design notes (26-feb: claude-flow, Agent-Skills, superpowers)

Run order: A and B in parallel → C → D.

---

## How to prompt a to-do agent (A/B/C/D or single agent)

Give **one** of the prompts below to your agent (Cursor, Claude, etc.). Run in order: Step 1 → Step 2 → Step 3. After each step, run `cargo build --workspace` and `cargo test --workspace`; commit with an explicit message.

---

### Prompt for Step 1 (foundational fixes, ~1 session)

```
Execute "1. Fix foundational issues first" from forge-project/PHASE0_REMAINING.md. Do all four items in order:

1. Extract rusqlite from forge-core — Remove ForgeError::Database from forge-core (move to forge-db or use Box<dyn Error> in core) so forge-core has no rusqlite dependency.
2. Add [workspace.dependencies] to root Cargo.toml — Centralize tokio, serde, chrono, rusqlite, uuid; have forge-core, forge-agent, forge-db use workspace deps.
3. Export validate_update_agent from forge-agent lib.rs and call it inside AgentRepo::update() in forge-db.
4. Fix preset serialization in forge-db (agents.rs) — Use serde Serialize/Deserialize (or a stable string format) for preset, not Debug.

Work in forge-project/. After each change, run cargo build --workspace and cargo test --workspace. Commit when all four are done with a clear message (reason, decision, effect).
```

---

### Prompt for Step 2 (5 crates, ~1–2 sessions)

```
Execute "2. Build remaining 5 crates" from forge-project/PHASE0_REMAINING.md. Create and wire:

1. forge-process — Stub crate: types for process spawn and stream-json (no real CLI yet). Depends on forge-core.
2. forge-safety — Stub crate: types for circuit breaker, rate limiter, cost tracker. Depends on forge-core.
3. forge-mcp — Stub crate: MCP protocol types only. Depends on forge-core.
4. forge-api — Axum router, GET /api/v1/health, agent CRUD (GET/POST/PUT/DELETE /api/v1/agents), WebSocket handler at GET /api/v1/ws. Use forge-db (AgentRepo), forge-agent (Agent, NewAgent, UpdateAgent), forge-core (EventBus). No frontend yet.
5. forge-app — Binary crate: main(), load config, run migrations, create EventBus + AgentRepo, mount forge-api routes, bind to port (e.g. 4173). Ties crates together.

Add all 5 to root Cargo.toml members. Keep [workspace.dependencies]. After changes, run cargo build --workspace and cargo test --workspace. Commit when all 5 crates compile and the binary starts (health check returns OK).
```

---

### Prompt for Step 3 (frontend shell + single binary, ~1 session)

```
Execute "3. Frontend shell + single binary" from forge-project/PHASE0_REMAINING.md:

1. Create a SvelteKit app with adapter-static (e.g. in forge-project/frontend or forge-app/frontend). Build output: static files.
2. Layout shell: sidebar, main content area, status bar. Empty pages: Dashboard, Agents, Sessions, Workflows, Skills, Settings (placeholders only).
3. In forge-app, use rust-embed (or similar) to embed the built static files and serve them at / and fallback for SPA. API stays at /api/v1/*.
4. Ensure cargo build --release produces a single binary; ./forge (or target/release/forge_app) serves UI at http://localhost:4173 and GET /api/v1/health returns OK.

Run cargo build --release and verify in browser. Commit when Phase 0 "done when" checklist is satisfied (binary starts, UI shell visible, WebSocket heartbeat optional for this step if not yet wired).
```

---

### Single combined prompt (all steps, for one agent over multiple turns)

```
We are finishing Phase 0 of Claude Forge (see forge-project/PHASE0_REMAINING.md). Do the work in strict order:

Step 1 — Foundational fixes: (a) extract rusqlite from forge-core, (b) add [workspace.dependencies], (c) export and call validate_update_agent in AgentRepo::update(), (d) fix preset serialization with serde in agents.rs. Commit after Step 1.

Step 2 — Add 5 crates: forge-process, forge-safety, forge-mcp (stubs), forge-api (Axum + health + agent CRUD + WebSocket), forge-app (binary wiring). Commit after Step 2.

Step 3 — Frontend: SvelteKit adapter-static, layout shell, empty pages, rust-embed in binary, cargo build --release produces working ./forge. Commit after Step 3.

After each step run cargo build --workspace (or --release for step 3) and cargo test --workspace. Follow git workflow: commit with explicit reason, decision, effect.
```
