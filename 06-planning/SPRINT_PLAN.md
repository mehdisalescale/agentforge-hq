# Claude Forge -- Sprint Plan

> From-scratch build. 12 two-week sprints aligned with ROADMAP (Phase 0→1→2∥4→3→5→6).
> Each sprint has clear goals, specific tasks, expected deliverables, and a definition of done.

---

## Sprint Calendar (aligned with ROADMAP)

| Sprint | Weeks | Phase | Focus |
|--------|-------|-------|-------|
| S1 | 1-2 | Phase 0 | Workspace structure + core types + event bus |
| S2 | 3-4 | Phase 0 | DB schema (all phases), batch writer, Agent CRUD, frontend shell |
| S3 | 5-6 | Phase 1 | Agent Engine: process spawn, streaming, session management |
| S4 | 7-8 | Phase 1 | Multi-pane UI, session browser, export |
| S5 | 9-10 | Phase 2 | Workflow engine core + Safety/MCP (parallel start) |
| S6 | 11-12 | Phase 2 | Workflow UI + skill catalog; Safety: circuit breaker, rate limiter |
| S7 | 13 | Phase 2 / 4 | Skill execution; MCP server (10 tools, 5 resources) |
| S8 | 14-15 | Phase 3 | Metrics, main dashboard, cost dashboard |
| S9 | 16-17 | Phase 3 | Swim lanes, git integration (status, diff, log) |
| S10 | 18 | Phase 3 | Git panel, worktrees |
| S11 | 19-21 | Phase 5 | WASM plugins, security hardening, audit log |
| S12 | 22-24 | Phase 5 | Polish, docs, 1.0 release |

Note: Phase 4 (Safety + MCP) runs in parallel with Phase 2 (weeks 9-12). We build from scratch; no migration from a previous codebase.

---

## Sprint 1: Workspace Structure + Core Types

**Weeks 1-2 | Phase 0**

### Goals
- Establish the 8-crate workspace structure (Phase 0: forge-core, forge-db, forge-api, forge-agent, forge-process, forge-safety, forge-mcp, forge-app)
- Implement core types and traits in `forge-core` (IDs, ForgeError, ForgeEvent, EventBus, EventSink)
- Implement database layer in `forge-db` (pool, migrations, batch writer)

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 1.1 | Create workspace `Cargo.toml` with `[workspace.dependencies]` | 2h | root |
| 1.2 | Create `forge-core` crate: Agent, Session, Event types | 8h | forge-core |
| 1.3 | Define core traits: EventSink, Repository | 4h | forge-core |
| 1.4 | Define error types with thiserror for forge-core | 3h | forge-core |
| 1.5 | Create `forge-db` crate: SQLite connection, pool, WAL config | 8h | forge-db |
| 1.6 | Implement repository layer in forge-db (AgentRepo, migrations) | 6h | forge-db |
| 1.7 | Create `forge-api` crate: Axum server, routes, middleware | 8h | forge-api |
| 1.8 | Create stub crates for forge-process, forge-safety, forge-mcp (Phase 1/4) | 4h | all |
| 1.9 | Create forge-app binary crate (entry point, wiring) | 3h | forge-app |
| 1.10 | Add `#![forbid(unsafe_code)]` to all workspace crates | 1h | all |
| 1.11 | Configure clippy lints in workspace Cargo.toml | 2h | root |
| 1.12 | Add TestDb helper and tests in forge-db / forge-core | 6h | forge-db, forge-core |
| 1.13 | Write unit tests for event bus, agent validation, migration runner | 8h | all |
| 1.14 | Verify frontend build + rust-embed still works | 2h | forge-api |
| 1.15 | Set up CI pipeline (fmt, clippy, nextest, build) | 4h | root |

### Expected Deliverables
- 8 workspace crates all compile
- `cargo test --workspace` passes
- `cargo build --release` produces binary (frontend optional for Phase 0)
- Agent CRUD via API, events via EventBus and batch writer
- CI pipeline green

### Definition of Done
- [ ] `cargo build --workspace` succeeds with no warnings
- [ ] `cargo test --workspace` passes 100%
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] Binary starts and serves UI at localhost:4173 when frontend is embedded
- [ ] Agent CRUD and WebSocket streaming functional (Phase 0 scope)
- [ ] All crates have `#![forbid(unsafe_code)]`

---

## Sprint 2: Event Bus + DB Schema + API + Frontend Shell

**Weeks 3-4 | Phase 0 (completion)**

### Goals
- Implement broadcast-based event bus (ForgeEvent enum, EventBus, EventSink)
- Implement full database schema (all phases) and FTS5 in one migration
- Add batch writer (50 events / 2s flush)
- Implement forge-api (health, agent CRUD, WebSocket) and forge-app wiring
- Add SvelteKit frontend shell with Agents CRUD page

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 2.1 | Implement ForgeEvent enum with all variants (typed, serde tag+content) | 6h | forge-core |
| 2.2 | Build broadcast channel event bus (emit, subscribe) | 8h | forge-core |
| 2.3 | Implement batch writer (crossbeam-channel, 50 events / 2s flush) | 6h | forge-db |
| 2.4 | Implement WebSocket handler subscribing to event bus | 4h | forge-api |
| 2.5 | Implement full schema migration (agents, sessions, events, workflows, skills, FTS5, etc.) | 4h | forge-db |
| 2.6 | Implement migration runner (version tracking, apply pending) | 6h | forge-db |
| 2.7 | Add FTS5 virtual tables (skills_fts, sessions_fts, events_fts) | 4h | forge-db |
| 2.8 | Implement migration runner with version tracking | 3h | forge-db |
| 2.9 | Implement AgentRepo CRUD and tests | 2h | forge-db |
| 2.10 | Write batch writer tests (threshold flush, timer flush, error handling) | 4h | forge-db |
| 2.11 | Write event bus tests (subscribe, multiple receivers, serialization) | 4h | forge-core |
| 2.12 | Write migration test (empty DB -> full schema, idempotent) | 2h | forge-db |
| 2.13 | Implement forge-api routes and forge-app CLI + rust-embed | 4h | forge-api, forge-app |
| 2.14 | Implement frontend shell (layout, Agents page, WebSocket store) | 3h | frontend |

### Expected Deliverables
- Event bus operational with WebSocket integration
- Full database schema with FTS5 (single migration, no v1)
- Batch writer flushing events reliably
- Phase 0 complete: binary, API, agent CRUD, frontend

### Definition of Done
- [ ] Event bus delivers events to WebSocket clients in < 10ms
- [ ] Batch writer flushes 50 events in < 50ms
- [ ] FTS5 search returns results in < 20ms
- [ ] Migration applies cleanly to empty DB and is idempotent
- [ ] All Phase 0 acceptance criteria from MILESTONES.md are met
- [ ] Phase 0 milestone signed off

---

## Sprint 3: Agent Engine (Process + Streaming)

**Weeks 5-6 | Phase 1**

### Goals
- Implement process spawning (Claude CLI, stream-json parsing)
- Event bus → WebSocket → frontend streaming
- Session CRUD, resume (--resume), export (JSON/Markdown)

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 3.1 | Implement process spawn: Claude CLI, working dir, env isolation | 6h | forge-process |
| 3.2 | Implement stream-json parsing (blocks, tool_calls, done) | 8h | forge-process |
| 3.3 | Emit process events to EventBus (ProcessStarted, OutputDelta, ProcessCompleted/Failed) | 4h | forge-process, forge-core |
| 3.4 | Wire WebSocket to forward event stream to frontend | 2h | forge-api |
| 3.5 | Session CRUD: create, list, get, delete; persist to DB | 4h | forge-db |
| 3.6 | Session resume: --resume flag, load history, append to conversation | 4h | forge-process |
| 3.7 | Session export: JSON and Markdown formats | 3h | forge-api |
| 3.8 | Frontend: streaming output (Markdown, code blocks, tool calls) | 8h | frontend |
| 3.9 | Frontend: session list and session detail views | 4h | frontend |
| 3.10 | Write process lifecycle tests (start, stream, complete, fail) | 6h | forge-process |
| 3.11 | Write session resume and export tests | 3h | forge-db, forge-api |

### Expected Deliverables
- Run agent with prompt; streaming output in UI within 2s
- Session resume continues previous conversation
- Session export produces valid JSON and Markdown

### Definition of Done
- [ ] Create agent, send prompt; streaming response appears within 2 seconds
- [ ] Session resume loads history and appends correctly
- [ ] Export produces valid JSON and readable Markdown
- [ ] Process lifecycle (start, complete, fail) emits correct events
- [ ] Unit test coverage for forge-process

---

## Sprint 4: Multi-pane UI + Session Browser

**Weeks 7-8 | Phase 1 (completion)**

### Goals
- Multi-pane layout: run multiple agents side-by-side
- Session browser with search and filters
- Phase 1 complete: Agent Engine + process + sessions + multi-pane

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 4.1 | Frontend: split-pane layout (2–4 panes, resizable) | 6h | frontend |
| 4.2 | Frontend: each pane has agent selector, prompt input, streaming output | 4h | frontend |
| 4.3 | API: support multiple concurrent process runs (scope by session/agent) | 4h | forge-api, forge-process |
| 4.4 | Session browser: list with filters (agent, date, status) | 4h | frontend |
| 4.5 | Session browser: FTS5 search (sessions_fts) | 3h | forge-db, forge-api |
| 4.6 | Frontend: session detail (messages, export button) | 3h | frontend |
| 4.7 | 9 agent presets usable from UI (select preset, run) | 2h | frontend |
| 4.8 | E2E test: create 2 agents, run in 2 panes, both stream | 4h | integration |
| 4.9 | Phase 1 stabilization and regression testing | 4h | all |

### Expected Deliverables
- Multi-pane UI: 2+ agents run side-by-side with independent streams
- Session browser with search and export
- Phase 1 milestone complete

### Definition of Done
- [ ] Three agents run in parallel, each streaming independently
- [ ] Session browser shows sessions with search/filter
- [ ] All Phase 1 acceptance criteria from MILESTONES.md met
- [ ] Phase 1 milestone signed off

---

## Sprint 5: Workflow Engine Core

**Weeks 8-9 | Phase 2**

### Goals
- Build the core workflow engine with all step types
- Implement workflow state machine and persistence
- Define workflow DSL format

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 5.1 | Create `forge-workflow` crate: types, error types | 3h | forge-workflow |
| 5.2 | Define Workflow, WorkflowStep, WorkflowRun types | 4h | forge-workflow |
| 5.3 | Define workflow YAML/JSON format with schema validation | 4h | forge-workflow |
| 5.4 | Implement WorkflowEngine.execute() entry point | 4h | forge-workflow |
| 5.5 | Implement PromptStep: send prompt to agent, collect result | 6h | forge-workflow |
| 5.6 | Implement ParallelStep: JoinSet for concurrent branches | 8h | forge-workflow |
| 5.7 | Implement ConditionalStep: evaluate condition, choose branch | 6h | forge-workflow |
| 5.8 | Implement LoopStep: iterate with break condition | 6h | forge-workflow |
| 5.9 | Implement HandoffStep: pass context between agents | 4h | forge-workflow |
| 5.10 | Implement step result passing (output -> next step input) | 4h | forge-workflow |
| 5.11 | Implement error handling strategies: retry, skip, abort | 4h | forge-workflow |
| 5.12 | Implement run state machine: Pending/Running/Completed/Failed/Cancelled | 4h | forge-workflow |
| 5.13 | Persist workflow definitions and runs to DB | 4h | forge-db |
| 5.14 | API endpoints: POST /api/workflows, GET /api/workflows, POST /api/workflows/:id/run | 4h | forge-api |
| 5.15 | Write state machine tests (all transitions, edge cases) | 6h | forge-workflow |
| 5.16 | Write step execution tests (each type, success + failure) | 8h | forge-workflow |

### Expected Deliverables
- Workflow engine executing all 5 step types
- Workflow persistence and resume
- Workflow API endpoints
- Comprehensive state machine tests

### Definition of Done
- [ ] Sequential workflow with 5 steps completes with correct output passing
- [ ] Parallel workflow runs 3 branches concurrently
- [ ] Conditional workflow takes correct branch based on condition
- [ ] Loop workflow iterates and breaks on condition
- [ ] Handoff step passes context between agents
- [ ] Failed step triggers configured error strategy (retry/skip/abort)
- [ ] Workflow run persists to DB and survives restart
- [ ] Unit test coverage > 90% for forge-workflow
- [ ] API endpoints return correct responses

---

## Sprint 6: Workflow UI + Skill Catalog + Safety (Circuit Breaker, Rate Limiter)

**Weeks 11-12 | Phase 2 + Phase 4 (parallel)**

### Goals
- Build the workflow builder and run visualization UI
- Import and index the 1,500+ skill catalog; skill search and browsing UI
- Implement Safety: circuit breaker and rate limiter (Phase 4)

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 6.1 | Create `forge-skills` crate: Skill type, SkillCategory, SkillCatalog | 4h | forge-skills |
| 6.2 | Define skill JSON schema: name, description, category, parameters, examples | 3h | forge-skills |
| 6.3 | Import 1,500+ skills from ecosystem repos | 6h | forge-skills |
| 6.4 | Index skills in FTS5 virtual table | 4h | forge-db |
| 6.5 | Implement skill search (FTS5 query builder, ranking) | 4h | forge-skills |
| 6.6 | Define 13+ top-level skill categories with descriptions | 2h | forge-skills |
| 6.7 | API endpoints: GET /api/skills, GET /api/skills/search, GET /api/skills/:id | 3h | forge-api |
| 6.8 | Frontend: workflow list page | 4h | frontend |
| 6.9 | Frontend: workflow builder (form-based step creation) | 12h | frontend |
| 6.10 | Frontend: workflow run visualization (step progress, status) | 8h | frontend |
| 6.11 | Frontend: workflow run history | 4h | frontend |
| 6.12 | Create 5 built-in workflow templates | 4h | forge-workflow |
| 6.13 | Frontend: skill browser (category navigation, search) | 8h | frontend |
| 6.14 | Frontend: skill detail view (parameters, examples, usage) | 4h | frontend |
| 6.15 | Create `forge-safety` crate: CircuitBreaker (Closed/Open/HalfOpen), RateLimiter (token bucket) | 8h | forge-safety |
| 6.16 | Integrate circuit breaker and rate limiter into agent execution path; 429 + Retry-After | 4h | forge-api |
| 6.17 | Frontend: safety indicators (circuit state, rate limit usage); GET /api/safety/status | 4h | frontend, forge-api |
| 6.18 | Write skill search tests (relevance, ranking, edge cases) | 4h | forge-skills |
| 6.19 | Write workflow UI component tests | 4h | frontend |

### Expected Deliverables
- Workflow builder UI (form-based)
- Workflow run visualization
- 5 built-in workflow templates
- 1,500+ skills indexed and searchable
- Skill browser and detail views
- Circuit breaker and rate limiter protecting agent execution

### Definition of Done
- [ ] Workflow builder creates valid workflow definitions
- [ ] Run visualization shows step progress in real-time
- [ ] All 5 templates load and execute successfully
- [ ] All 1,500+ skills appear in the skill browser
- [ ] Skill search returns relevant results in < 20ms
- [ ] Circuit breaker and rate limiter integrated; 429 + Retry-After when limited
- [ ] Frontend component tests pass

---

## Sprint 7: Skill Execution + MCP Server + Cost Tracking

**Week 13 | Phase 2 (completion) + Phase 4**

### Goals
- Implement skill-to-prompt compilation and execution; slash-command autocomplete
- MCP server: 10 tools, 5 resources (stdio + SSE)
- Cost tracking and budget enforcement; cost dashboard

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 7.1 | Implement skill-to-prompt compilation (skill + params -> prompts) | 6h | forge-skills |
| 7.2 | Implement skill chaining (output of one -> input of another) | 4h | forge-skills |
| 7.3 | Implement skill result formatting (Markdown, JSON, code blocks) | 3h | forge-skills |
| 7.4 | Implement skill usage tracking (most-used, recently-used) | 3h | forge-db |
| 7.5 | Frontend: slash-command autocomplete (`/` trigger, fuzzy match) | 8h | frontend |
| 7.6 | Frontend: skill parameter form (dynamic based on skill schema) | 6h | frontend |
| 7.7 | Create `forge-mcp` crate: protocol types, JSON-RPC, stdio + SSE transports | 6h | forge-mcp |
| 7.8 | Implement 10 MCP tools (agent CRUD, session, workflow, skill, config) | 12h | forge-mcp |
| 7.9 | Implement 5 MCP resources (agent, session, skill, config, status) | 6h | forge-mcp |
| 7.10 | Implement CostTracker + CostBudget (soft/hard limit); persist to DB | 6h | forge-safety, forge-db |
| 7.11 | Frontend: cost dashboard (daily/weekly/monthly, per-agent) | 6h | frontend |
| 7.12 | Integration test: skill search -> select -> execute -> result | 4h | integration |
| 7.13 | Integration test: MCP client -> tool execution -> result | 4h | integration |
| 7.14 | Phase 2 stabilization; Phase 2 documentation (workflow guide, skill catalog) | 6h | all, docs |

### Expected Deliverables
- Skill-to-prompt compilation working
- Slash-command autocomplete in prompt input
- MCP server with 10 tools and 5 resources (stdio + SSE)
- Cost tracking and budget enforcement; cost dashboard
- Phase 2 fully integrated and stable

### Definition of Done
- [ ] Skill compilation produces valid agent prompts
- [ ] Slash-command `/` opens autocomplete within 100ms
- [ ] MCP server responds to initialize; all 10 tools and 5 resources work
- [ ] Cost tracker and budget enforcement block at hard limit
- [ ] Cost dashboard renders with real data
- [ ] All Phase 2 acceptance criteria from MILESTONES.md met
- [ ] Phase 2 milestone signed off

---

## Sprint 8: Metrics Collection + Main Dashboard

**Weeks 13-14 | Phase 3**

### Goals
- Instrument all crates with metrics
- Build the main operational dashboard
- Create the `forge-observe` crate

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 8.1 | Create `forge-observe` crate: MetricsCollector, Dashboard types | 4h | forge-observe |
| 8.2 | Add token usage metrics to agent execution path | 4h | forge-core/api |
| 8.3 | Add response latency metrics to all API endpoints | 3h | forge-api |
| 8.4 | Add event throughput metrics to event bus | 2h | forge-core |
| 8.5 | Add active agent count gauge | 2h | forge-api |
| 8.6 | Add WebSocket connection count gauge | 2h | forge-api |
| 8.7 | Add DB batch writer queue depth metric | 2h | forge-db |
| 8.8 | Add circuit breaker state metrics (per-agent) | 2h | forge-safety |
| 8.9 | Add rate limiter metrics (requests allowed/rejected) | 2h | forge-safety |
| 8.10 | Implement metrics aggregation (time windows: 1min, 5min, 1h, 24h) | 6h | forge-observe |
| 8.11 | API endpoints: GET /api/dashboard, GET /api/metrics | 4h | forge-api |
| 8.12 | Frontend: main dashboard layout (grid of widgets) | 6h | frontend |
| 8.13 | Frontend: active agents widget (count, names, status) | 3h | frontend |
| 8.14 | Frontend: recent runs widget (last 10 runs with status) | 3h | frontend |
| 8.15 | Frontend: system health widget (uptime, memory, event rate) | 3h | frontend |
| 8.16 | Frontend: real-time updates via WebSocket | 4h | frontend |
| 8.17 | Write metrics collection tests | 4h | forge-observe |

### Expected Deliverables
- Metrics instrumentation across all crates
- Main dashboard with 3+ widgets
- Real-time dashboard updates

### Definition of Done
- [ ] All metric types (counters, gauges, histograms) recording data
- [ ] Main dashboard loads in < 1 second
- [ ] Active agents widget shows correct count and statuses
- [ ] Recent runs widget shows correct history
- [ ] System health widget shows accurate data
- [ ] Dashboard updates in real-time when events occur
- [ ] Metrics aggregation works for all time windows

---

## Sprint 9: Cost Dashboard + Swim Lanes

**Weeks 15-16 | Phase 3**

### Goals
- Build the cost tracking dashboard with time-series visualization
- Implement agent swim lane visualization
- Add session timeline view

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 9.1 | Implement cost aggregation: per-agent, per-model, per-day | 6h | forge-observe |
| 9.2 | Implement budget utilization calculation (current vs. limit) | 3h | forge-safety |
| 9.3 | API endpoints: GET /api/costs (time range, grouping params) | 4h | forge-api |
| 9.4 | Frontend: cost dashboard page layout | 4h | frontend |
| 9.5 | Frontend: daily/weekly/monthly cost chart (chart.js) | 6h | frontend |
| 9.6 | Frontend: per-agent cost breakdown (bar chart) | 4h | frontend |
| 9.7 | Frontend: budget utilization meter | 3h | frontend |
| 9.8 | Frontend: cost alerts (approaching/exceeded budget) | 3h | frontend |
| 9.9 | Implement swim lane data: parallel agent timeline data | 6h | forge-observe |
| 9.10 | API endpoint: GET /api/dashboard/swimlane (time range) | 3h | forge-api |
| 9.11 | Frontend: swim lane visualization (d3-hierarchy or custom SVG) | 12h | frontend |
| 9.12 | Frontend: swim lane event markers (click to see details) | 4h | frontend |
| 9.13 | Frontend: session timeline view (event sequence) | 6h | frontend |
| 9.14 | Write cost aggregation tests | 4h | forge-observe |

### Expected Deliverables
- Cost dashboard with time-series charts
- Per-agent cost breakdown
- Budget utilization display
- Agent swim lane visualization
- Session timeline view

### Definition of Done
- [ ] Cost dashboard shows accurate daily/weekly/monthly data
- [ ] Per-agent breakdown matches individual agent cost sums
- [ ] Budget meter shows correct utilization percentage
- [ ] Swim lane shows parallel agent activity with correct timing
- [ ] Event markers on swim lane are clickable
- [ ] Session timeline shows event sequence in order
- [ ] All visualizations handle empty data gracefully

---

## Sprint 10: Git Integration (Status, Diff, Log)

**Weeks 17-18 | Phase 3**

### Goals
- Build `forge-git` crate wrapping libgit2
- Implement core git operations: status, diff, log, branches
- Add API endpoints for git operations

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 10.1 | Create `forge-git` crate: GitRepo wrapper, error types | 4h | forge-git |
| 10.2 | Implement git status: modified, staged, untracked, ignored | 6h | forge-git |
| 10.3 | Implement git diff: file-level diff, hunk-level diff | 8h | forge-git |
| 10.4 | Implement diff formatting: line numbers, +/- markers, context | 4h | forge-git |
| 10.5 | Implement git log: commit history with author, date, message, stats | 6h | forge-git |
| 10.6 | Implement git branch: list, create, delete | 4h | forge-git |
| 10.7 | Implement branch info: current branch, tracking branch, ahead/behind | 3h | forge-git |
| 10.8 | API endpoints: GET /api/git/status, /diff, /log, /branches | 6h | forge-api |
| 10.9 | API: working directory parameter (support multiple repos) | 2h | forge-api |
| 10.10 | Write git status tests (with temp repos) | 4h | forge-git |
| 10.11 | Write git diff tests (various change types) | 4h | forge-git |
| 10.12 | Write git log tests (merge commits, linear history) | 3h | forge-git |
| 10.13 | Write git branch tests | 2h | forge-git |
| 10.14 | Performance test: status/diff/log on large repo (10K files) | 3h | forge-git |

### Expected Deliverables
- `forge-git` crate with full status, diff, log, branch operations
- API endpoints for all git operations
- Performance validated on large repos

### Definition of Done
- [ ] Git status matches `git status` output for 5 test repos
- [ ] Git diff shows correct file and hunk changes
- [ ] Git log shows correct commit history
- [ ] Branch list/create/delete work correctly
- [ ] All operations complete in < 500ms on 10K-file repos
- [ ] API endpoints return correctly formatted JSON
- [ ] Tests use temporary git repos (no external dependencies)

---

## Sprint 11: Git UI + Worktrees + Notifications Start

**Weeks 19-20 | Phase 3/4 overlap**

### Goals
- Build the git panel in the frontend
- Implement worktree management
- Begin notification system (`forge-notify` crate)

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 11.1 | Frontend: git panel layout (status summary, file list) | 6h | frontend |
| 11.2 | Frontend: diff viewer with syntax highlighting (shiki) | 8h | frontend |
| 11.3 | Frontend: commit log view with pagination | 4h | frontend |
| 11.4 | Frontend: branch selector dropdown | 3h | frontend |
| 11.5 | Implement git worktree: list, create, remove | 6h | forge-git |
| 11.6 | API endpoints: GET /api/git/worktrees, POST, DELETE | 3h | forge-api |
| 11.7 | Frontend: worktree management panel | 4h | frontend |
| 11.8 | Create `forge-notify` crate: types, NotificationService | 4h | forge-notify |
| 11.9 | Implement WebSocket notification channel | 4h | forge-notify |
| 11.10 | Implement desktop notification channel (native) | 4h | forge-notify |
| 11.11 | Implement notification routing (per-agent preferences) | 4h | forge-notify |
| 11.12 | API endpoints: GET /api/notifications, POST /api/notifications/preferences | 3h | forge-api |
| 11.13 | Phase 3 stabilization and sign-off | 4h | all |
| 11.14 | Git integration tests (full workflow: status -> diff -> log) | 4h | integration |

### Expected Deliverables
- Git panel in frontend with diff viewer
- Worktree management working
- Notification system foundation (WebSocket + desktop channels)
- Phase 3 complete

### Definition of Done
- [ ] Git panel shows accurate status, clickable files
- [ ] Diff viewer renders hunks with syntax highlighting
- [ ] Commit log displays with correct data and pagination
- [ ] Worktree create/list/remove work end-to-end
- [ ] WebSocket notifications delivered in < 2 seconds
- [ ] Desktop notifications appear for agent completion events
- [ ] All Phase 3 acceptance criteria from MILESTONES.md met
- [ ] Phase 3 milestone signed off

---

## Sprint 12: Notifications + Scheduler

**Weeks 21-22 | Phase 4**

### Goals
- Complete the notification system (webhook, email channels)
- Build the scheduler with cron support
- Implement configuration management

### Tasks

| # | Task | Est. Hours | Crate |
|---|------|-----------|-------|
| 12.1 | Implement webhook notification channel (HTTP POST) | 4h | forge-notify |
| 12.2 | Implement email notification channel (SMTP) | 6h | forge-notify |
| 12.3 | Implement notification templates (Markdown-based) | 4h | forge-notify |
| 12.4 | Implement notification history + read/unread state | 3h | forge-db |
| 12.5 | Implement notification rate limiting (per-channel) | 3h | forge-notify |
| 12.6 | Frontend: notification bell icon + unread count | 3h | frontend |
| 12.7 | Frontend: notification center panel | 6h | frontend |
| 12.8 | Create `forge-scheduler` crate: Scheduler, Job types | 4h | forge-scheduler |
| 12.9 | Implement cron expression parsing and next-run calculation | 4h | forge-scheduler |
| 12.10 | Implement job execution: agent run, workflow execution | 4h | forge-scheduler |
| 12.11 | Implement job persistence (DB) and restart recovery | 3h | forge-db |
| 12.12 | Frontend: scheduler dashboard (upcoming, history) | 6h | frontend |
| 12.13 | Implement hierarchical config: defaults < global < project < agent | 6h | forge-core |
| 12.14 | Frontend: settings page with sections | 6h | frontend |
| 12.15 | Integration tests: notification delivery, job scheduling | 4h | integration |
| 12.16 | Phase 4 stabilization | 4h | all |

### Expected Deliverables
- 4 notification channels complete
- Notification center in frontend
- Cron-based scheduler operational
- Scheduler dashboard
- Configuration management with UI
- Phase 4 substantially complete

### Definition of Done
- [ ] Webhook delivers within 1 second of trigger
- [ ] Email sends via configured SMTP
- [ ] Notification templates render correctly
- [ ] Notification center shows history with read/unread
- [ ] Scheduler executes jobs at correct cron times
- [ ] Jobs survive binary restart
- [ ] Scheduler dashboard shows upcoming and past jobs
- [ ] Config hierarchy resolves correctly
- [ ] Settings UI saves and applies immediately

---

## Velocity Tracking Template

After each sprint, record actuals vs. estimates:

| Sprint | Planned Points | Completed Points | Velocity | Blockers |
|--------|---------------|-----------------|----------|----------|
| S1 | -- | -- | -- | -- |
| S2 | -- | -- | -- | -- |
| S3 | -- | -- | -- | -- |
| ... | | | | |

Use rolling 3-sprint average velocity to forecast remaining work.

---

## Buffer Allocation

| Buffer Type | Amount | Purpose |
|-------------|--------|---------|
| Phase boundary buffer | 2-3 days per phase | Stabilization, regression testing, documentation |
| Technical debt sprint | 1 sprint per 6 sprints | Refactoring, dependency updates, performance work |
| Risk buffer | 10% of total timeline | Unforeseen issues, scope discovery |

The sprint plan assumes approximately 40 productive hours per week for a single developer. Adjust sprint scope if actual velocity differs significantly from estimates.
