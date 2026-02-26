# Phase 1 Sprint — 6 Agents (A–F) in Parallel

> **Goal:** Agent Engine — run agents with prompts, see streaming output, manage sessions. All 6 agents get the same **context** and **sprint goal**; each has one **track** so work is parallel and merge order is clear.

---

## 1. Sprint context (give this to every agent)

**Sprint:** Phase 1 — Agent Engine (Sprint 3–4 in SPRINT_PLAN).  
**Duration:** 2–3 weeks (or until Definition of Done is met).  
**Codebase:** Runnable app in **claude-forge**; planning and reference in **forge-project**. Work in the repo that has the runnable workspace (likely claude-forge); if your org uses forge-project only, work there and sync to claude-forge later.

**Phase 0 state:** 8 crates (forge-core, forge-agent, forge-db, forge-api, forge-process, forge-safety, forge-mcp, forge-app), frontend shell (SvelteKit), single binary `./forge`, health + agent CRUD + WebSocket at `/api/v1/*`. Process and session are stubs.

**Phase 1 goal:** User can pick an agent, send a prompt, see **streaming** output in the UI, and manage **sessions** (list, resume, export). Optionally multi-pane (run 2+ agents side-by-side).

**Design refs (use, don’t copy):**
- **forge-process / streaming:** `claude` CLI behavior and stream-json contract; **claude-flow** ADRs for coordination/lifecycle (see `forge-project/08-reference/TREND_26FEB_ENHANCEMENT_MAP.md`).
- **Skills/workflows later:** Agent-Skills-for-Context-Engineering, superpowers (spec→TDD). Not required for this sprint.
- **Phase 1 design notes:** `forge-project/docs/PHASE1_DESIGN_NOTES.md`.

**26-feb summary:** claude-flow (process/MCP design), deer-flow (sub-agent, sandbox), superpowers (workflow/skills). Full map: `forge-project/08-reference/TREND_26FEB_ENHANCEMENT_MAP.md`.

---

## 2. Sprint goal and definition of done

**Goal:** Implement the Agent Engine: process spawn, stream-json parsing, events to EventBus → WebSocket → frontend; session CRUD and resume/export; UI to run agents and show streaming output.

**Definition of done (Phase 1):**
- [ ] Create an agent in the UI, send a prompt; **streaming response** appears within a few seconds.
- [ ] **Session** is created and persisted; user can **resume** a session and **export** (JSON or Markdown).
- [ ] Process lifecycle (start, stream, complete/fail) emits **ForgeEvent** and events reach the frontend via WebSocket.
- [ ] `cargo test --workspace` and `cargo clippy --workspace` pass (or equivalent in your repo).

**Track A (spawn + stream-json):** Done. spawn.rs: SpawnConfig (command, args_before_prompt, working_dir, env_remove, env_set), spawn() returns ProcessHandle (take_stdout, kill, wait, id). stream_event.rs + parse.rs: StreamJsonEvent (serde tag type), ContentBlock (Text, ToolUse, ToolResult, Thinking), parse_line() → Result<Option<StreamJsonEvent>>. No EventBus in A; runner (B) maps via emit_parsed_event. Tests: parse (empty, result, assistant, system, error, unknown, invalid JSON), spawn (shell prints one line, parse). Commit 5c19ea7.

**Track B (events + EventBus):** Done. ForgeEvent process variants unchanged in forge-core. forge-process: ProcessRunner (EventBus), emit_stub_run(session_id, agent_id), emit_parsed_event (maps stream_event::StreamJsonEvent → ProcessOutput/ProcessCompleted/ProcessFailed). Tests: stub_run_emits_process_events_in_order, emit_from_stream, ProcessFailed. WebSocket in forge-api already forwards EventBus; run endpoint (D) can call ProcessRunner.

**Track E (Agents CRUD):** Done. Agents page: list (cards), create/edit modal, delete; `api.ts` + `routes/agents/+page.svelte`; `pnpm build` passes. Optional: WebSocket live updates later.

**Track C (Sessions):** Done (code audit). forge-db: SessionRepo in `repos/sessions.rs` (create, get, list, delete, update_status, update_claude_session_id). forge-api: `routes/sessions.rs` — GET/POST `/api/v1/sessions`, GET/DELETE `/api/v1/sessions/:id`, GET `/api/v1/sessions/:id/export?format=json|markdown`. Schema in `migrations/0001_init.sql`. Tests: session_crud_roundtrip, session_update_status (forge-db); session_crud_and_export (forge-api).

**Track D (Run endpoint):** Done. POST `/api/v1/run` in `forge-api/routes/run.rs`: body `{ agent_id, prompt, session_id? }`; loads agent, creates or resolves session (SessionRepo), calls ProcessRunner::emit_stub_run, returns 202 + `{ session_id, message }`. Stub can be replaced with forge_process::spawn + stream parsing later.

**Track F (Run + Sessions UI):** Done. Dashboard `/`: agent selector, prompt, Run, Resume (`?resume=<session_id>`), streaming (WebSocket, ProcessOutput filtered by session_id), status, Clear. Sessions `/sessions`: list, detail, Resume→`/?resume=`, Export JSON/Markdown. Sidebar “Run”, status bar “Phase 1”. Works end-to-end when C (session API) and D (POST /api/v1/run) are in place.

---

## 3. Six tracks (assign one per agent)

| Agent | Track | Deliverable | Deps |
|-------|--------|-------------|------|
| **A** | forge-process: spawn + stream-json | spawn.rs: SpawnConfig, spawn(config, prompt, session_id), ProcessHandle (take_stdout, kill, wait). stream_event + parse: StreamJsonEvent (System, Assistant, User, Result, Error), ContentBlock, parse_line. No EventBus. **Done** (commit 5c19ea7). | forge-core |
| **B** | forge-process: events + EventBus | ProcessRunner in runner.rs: EventBus, emit_stub_run, emit_parsed_event (maps Agent A parser output to ForgeEvent). Process variants already in forge-core. Tests: stub_run, emit_from_stream, ProcessFailed. **Done.** WebSocket unchanged. | A (or stub), forge-core |
| **C** | Sessions: DB + API | SessionRepo (forge-db `repos/sessions.rs`), session routes + export (forge-api `routes/sessions.rs`). **Done** (code audit). | forge-db, forge-api |
| **D** | Run endpoint + wiring | POST “run” (or “prompt”) endpoint: take agent_id + prompt (and optional session_id for resume); spawn process via forge-process, forward events to EventBus. WebSocket already forwards; stub emits ProcessStarted, ProcessOutput, ProcessCompleted to EventBus/BatchWriter. **Done.** `routes/run.rs`: create/resume session, ProcessRunner::emit_stub_run, 202 + session_id. | A, B, forge-api |
| **E** | Frontend: Agents CRUD | Agents page: list (cards), create/edit modal form, delete with confirm. `api.ts` + `/api/v1/agents`. **Done** (commit 0dff114). Optional later: WebSocket for AgentCreated/Updated/Deleted. | forge-api (existing) |
| **F** | Frontend: Run + streaming + sessions | Run UI (Dashboard `/`): agent selector, prompt, Run, Resume (`?resume=<id>`), streaming via WebSocket (ProcessOutput by session_id), status, Clear. Sessions (`/sessions`): list, detail, Resume→`/?resume=`, Export JSON/Markdown. **Done** (commit 91beac8). Depends on C (session API) and D (POST /api/v1/run). | C, D (or stubs) |

**Merge order:** Merge **A** and **B** first (process + events), then **C** (sessions). Then **D** (run endpoint) can use real spawn. Then **E** and **F** (frontend) can integrate with real API. If E and F start early, they can use stubs or existing agent CRUD and add run/session when D and C land.

### Implementation status (code audit)

*Last audit: 2026-02-26. Based on inspecting the forge-project codebase, not docs.*

| Track | Verified in code | Notes |
|-------|------------------|--------|
| **A** | `forge-process`: `spawn.rs`, `stream_event.rs`, `parse.rs` | SpawnConfig, spawn(), ProcessHandle; StreamJsonEvent, ContentBlock, parse_line(). |
| **B** | `forge-process/runner.rs` | ProcessRunner, emit_stub_run, emit_parsed_event (stream_event → ForgeEvent), content_block_output. |
| **C** | `forge-db/repos/sessions.rs`, `forge-api/routes/sessions.rs` | SessionRepo (create, get, list, delete, update_status, update_claude_session_id). Routes: list, create, get, delete, export (json/markdown). |
| **D** | `forge-api/routes/run.rs` | POST `/run`: RunRequest (agent_id, prompt, session_id?), create or get session, ProcessRunner::emit_stub_run, 202 + RunResponse. |
| **E** | `frontend/src/routes/agents/+page.svelte`, `api.ts` | List, create/edit modal, delete; listAgents, createAgent, updateAgent, deleteAgent. |
| **F** | `frontend/src/routes/+page.svelte`, `sessions/+page.svelte` | Dashboard: runAgent, WebSocket, streaming UI. Sessions: listSessions, getSession, exportSessionUrl, resume. |

**E2E:** Run endpoint uses **real spawn**: handler spawns CLI (SpawnConfig default), background task reads stdout, parse_line → emit_parsed_event; ProcessStarted/ProcessCompleted/ProcessFailed emitted. Create agent → run prompt → see stream → list session → resume/export.

### Agent A deliverable summary (reference)

Summary of **Agent A — forge-process: spawn + stream-json** (commit **5c19ea7**). Use this when integrating with forge-process or onboarding.

**1. Process spawn (`spawn.rs`)**  
- **SpawnConfig**: `command` (default `"claude"`), `args_before_prompt` (e.g. `--output-format stream-json --verbose`), `working_dir`, `env_remove` (e.g. `CLAUDECODE`), `env_set`.  
- **spawn(config, prompt, session_id)**: Runs command with args, `-p prompt`, optional `--resume session_id`. Stdout piped, stderr inherited. Returns **ProcessHandle** with `take_stdout()`, `kill()`, `wait()`, `id()`.

**2. Stream-json parsing (`stream_event.rs` + `parse.rs`)**  
- **StreamJsonEvent** (enum, `#[serde(tag = "type")]`): **System**, **Assistant**, **User**, **Result**, **Error** with payloads from EVENT_SYSTEM.md (session_id, message with content[], result, etc.).  
- **ContentBlock**: **Text**, **ToolUse**, **ToolResult**, **Thinking** for `message.content[]`.  
- **parse_line(line)**: One NDJSON line → `Result<Option<StreamJsonEvent>, ParseError>`. Blank lines → `Ok(None)`; unknown `type` → `ParseError::UnknownType`.

**3. No EventBus in Track A**  
- No EventBus or emit in spawn/parse. **runner.rs** (Agent B) maps parsed events to ForgeEvent in `emit_parsed_event`.

**4. Tests**  
- **parse**: empty line, result, assistant, system, error, unknown type, invalid JSON.  
- **spawn**: run shell command that prints one JSON line, read stdout, parse → Result with `result == "ok"`.  
- **Runner** (B): stub run, emit_from_stream, emit_stream_event (failed).  
- **`cargo test -p forge-process`**: 11 tests passed.

---

## 4. Copy-paste prompts for each agent

Give each agent **section 1 (Sprint context)** plus **only** the “Your task” block for their letter.

---

### Agent A — forge-process: spawn + stream-json

**Your task**
- In **forge-process**, implement **process spawn**: run the `claude` CLI (or configurable command) with working directory and env isolation. Use `claude` CLI behavior/contract as reference (see PHASE1_DESIGN_NOTES).
- Implement **stream-json parsing** on the process stdout: parse blocks, tool_calls, done (or equivalent) from the CLI’s stream-json output. Produce structured events (e.g. `StreamJsonEvent` or an enum) that Agent B will map to ForgeEvent.
- Do **not** emit to EventBus in this task; only spawn and parse. Add unit tests for parsing and (if possible) spawn with a no-op or short-lived command.
- Commit when: `cargo test -p forge-process` passes and spawn + parse are usable by the rest of the stack.

---

### Agent B — forge-process: events + EventBus

**Your task**
- Ensure **ForgeEvent** has process-related variants (e.g. ProcessStarted, OutputDelta, ProcessCompleted, ProcessFailed) in **forge-core**. If they already exist, use them.
- In **forge-process**, add a small “runner” that uses Agent A’s spawn/parse (or a stub if A isn’t merged yet) and **emits** these events to **EventBus**. The existing WebSocket in forge-api forwards EventBus events; no change needed there for this task.
- Add tests that run the runner (or a stub) and assert that the expected events are received on the EventBus.
- Commit when: process events are emitted and tests pass.

---

### Agent C — Sessions: DB + API

**Your task**
- In **forge-db**, implement **SessionRepo**: create, list, get, delete. Use the existing schema (sessions table) and follow the same patterns as AgentRepo. Add migrations if the table is missing.
- In **forge-api**, add **session routes**: e.g. GET/POST `/api/v1/sessions`, GET/DELETE `/api/v1/sessions/:id`, and **export**: GET `/api/v1/sessions/:id/export?format=json|markdown` returning JSON or Markdown.
- Add tests for SessionRepo and for the session/export endpoints.
- Commit when: `cargo test --workspace` passes and session CRUD + export work via the API.

---

### Agent D — Run endpoint + wiring

**Your task**
- Add a **run** (or **prompt**) endpoint in **forge-api**, e.g. POST `/api/v1/agents/:id/run` or POST `/api/v1/run` with body `{ "agent_id", "prompt", "session_id" (optional for resume) }`.
- The handler should: resolve the agent, optionally load session for resume, **spawn the process** via forge-process (Agent A’s spawn), and ensure process events are emitted to the **EventBus** (Agent B’s runner). The existing WebSocket will forward events to the frontend.
- If Agent A or B isn’t merged yet, use a stub that emits a few fake events so the endpoint and WebSocket path work.
- Commit when: calling the run endpoint starts a process (or stub) and clients connected to WebSocket receive the corresponding events.

---

### Agent E — Frontend: Agents CRUD

**Your task**
- Build the **Agents** page in the frontend: **list** (cards or table), **create** (form: name, model, system prompt, preset, etc.), **edit** (same form, pre-filled), **delete** (with confirm). Use existing `/api/v1/agents` (GET, POST, PUT, DELETE).
- Optional: subscribe to WebSocket and update the list when AgentCreated/AgentUpdated/AgentDeleted events arrive.
- Keep the existing layout (sidebar, main, status bar). Ensure the app still builds (e.g. `pnpm build`).
- Commit when: Agents page is usable and matches the API contract.

---

### Agent F — Frontend: Run + streaming + sessions

**Your task**
- Build **Run** UI: choose an agent (dropdown or list), prompt input, “Run” button. **Streaming output** area that shows incoming content (Markdown, code blocks) as it arrives — connect to the existing WebSocket and filter/display process output events.
- Build **Sessions** UI: session list (e.g. from GET `/api/v1/sessions`), session detail (messages or summary), **Resume** (e.g. “Run” with session_id), **Export** (link or button to download JSON/Markdown). Use session APIs from Agent C when available; otherwise stub or mock.
- Ensure the app still builds and works with the existing layout.
- Commit when: user can run an agent, see streaming output, and see sessions with resume/export.

---

## 5. Run order and merge

1. **Kickoff:** All 6 agents start with **section 1 (Sprint context)** and their **section 4** prompt. Work in the same repo (claude-forge or forge-project as decided).
2. **Merge order:**  
   - Merge **A** and **B** first (forge-process spawn + events).  
   - Then **C** (sessions).  
   - Then **D** (run endpoint; depends on A+B).  
   - Then **E** and **F** (frontend; can merge in any order after API is stable).
3. **Sync:** After each merge, run `cargo test --workspace` and fix conflicts. If D started with stubs, replace stubs with real spawn/events when A and B are in.
4. **Done:** When Definition of Done (section 2) is satisfied, run a short E2E: create agent → run prompt → see stream → list session → resume or export.

---

## 6. Where to find things

| What | Where |
|------|--------|
| Phase 1 design notes | `forge-project/docs/PHASE1_DESIGN_NOTES.md` |
| 26-feb enhancement map | `forge-project/08-reference/TREND_26FEB_ENHANCEMENT_MAP.md` |
| Sprint plan (S3–S4) | `forge-project/06-planning/SPRINT_PLAN.md` |
| North star / priorities | `forge-project/NORTH_STAR.md` |
| **Next sprint (A–F tasks)** | [NEXT_SPRINT_AGENT_TASKS.md](NEXT_SPRINT_AGENT_TASKS.md) — Phase 1 polish + Phase 2 seed |

---

## 7. After this sprint: what’s done, what remains

### When Phase 1 (this work) is done

| Area | Done |
|------|------|
| **Backend** | forge-process: real spawn (Claude CLI), stream-json parsing; process events (ProcessStarted, OutputDelta, ProcessCompleted/Failed) on EventBus → WebSocket. SessionRepo (forge-db); session CRUD + export (JSON/Markdown) in forge-api. POST run/prompt endpoint that spawns and streams. |
| **Frontend** | Agents page: list, create, edit, delete (using `/api/v1/agents`). Run UI: agent selector, prompt input, streaming output (Markdown, code blocks). Session list + detail; Resume and Export. |
| **User value** | Create agents in the UI, run a prompt, see **streaming** output; sessions are stored; user can **resume** and **export**. Core agent orchestration works. |
| **Milestone** | M1 (Agent Engine) in MILESTONES.md is satisfied. |

### What still remains (Phases 2–6)

| Phase | Name | What remains |
|-------|------|----------------|
| **2** | Workflows + Skills | Workflow engine (steps, state machine), skill catalog (1,500+ skills), FTS5 search, workflow UI, skill browser. |
| **3** | Observability + Git | Metrics, main dashboard, cost dashboard; swim lanes; git integration (status, diff, log, worktrees). |
| **4** | Safety + MCP | forge-safety: CircuitBreaker, RateLimiter, CostTracker (real logic). forge-mcp: MCP server (10 tools, 5 resources), stdio + SSE. Safety/MCP dashboards. |
| **5** | Plugins + Security | WASM plugins, security hardening, audit log, notifications, scheduler. |
| **6** | Dev Environment (post-1.0) | In-app code view/edit, terminal, file browser — IDE-like experience. |
| **1.0** | Production ready | Polish, docs, release checklist (M8). |

So after Phase 1 you have **core agent orchestration**; remaining work is workflows/skills, observability/git, safety/MCP, plugins/security, then dev environment and 1.0. See **ROADMAP.md** and **MILESTONES.md** for details and week targets.

**Next sprint:** [NEXT_SPRINT_AGENT_TASKS.md](NEXT_SPRINT_AGENT_TASKS.md) — tasks for agents A–F (Phase 1 polish + Phase 2 seed).
