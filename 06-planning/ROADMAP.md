# Claude Forge -- Roadmap

> **From scratch, informed by everything.** 62 reference repos, 34 design docs, existing Forge as reference.
> 7 phases. Phases 0-5 = 1.0 release (~27 weeks). Phase 6 = post-1.0.
> Each phase delivers a shippable binary. No phase depends on old code.

---

## Phase Overview

```
Week   1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27
       [------- Phase 0 -------]
                                [---------- Phase 1 ----------]
                                                              [------------- Phase 2 ------
Phase 2 (cont'd) ------]
                        [---------- Phase 4 ----------]    (parallel with Phase 2/3)
                                                        [------------- Phase 3 -------------]
                                                                          [------ Phase 5 --------
Phase 5 (cont'd) ------]

Post-1.0:
Week   28 29 30 31 32
       [---- Phase 6 ----]
```

| Phase | Name | Duration | Weeks | Cumulative |
|-------|------|----------|-------|-----------|
| 0 | Foundation Build | 4 weeks | 1-4 | 4 |
| 1 | Agent Engine + Process Management | 4 weeks | 5-8 | 8 |
| 2 | Workflows + Skills | 5 weeks | 9-13 | 13 |
| 3 | Observability + Git | 5 weeks | 14-18 | 18 |
| 4 | Safety + MCP | 4 weeks | 9-12 | (parallel) |
| 5 | Plugins + Security + Polish | 6 weeks | 19-24 | 24 |
| 6 | Dev Environment (post-1.0) | 5 weeks | 25-29 | 29 |

---

## What's Shippable After Each Phase

| After Phase | Users Can... | Key Value |
|-------------|-------------|-----------|
| 0 | Build the binary, see the UI, hit API endpoints | **Skeleton works end-to-end** |
| 1 | Create agents, run prompts, see streaming output, manage sessions | **Core agent orchestration** |
| 2 | Define multi-step workflows, search 1,500+ skills, chain agents | **Workflow automation** |
| 3 | View dashboards, track costs, use git operations, manage worktrees | **Operational visibility** |
| 4 | Use safety controls, connect MCP clients, rate-limit costs | **Safe multi-agent operation** |
| 5 | Install WASM plugins, audit actions, control permissions | **Extensibility + trust** |
| 6 | View/edit code, use terminal, browse files -- all in Forge | **IDE-like experience** |

---

## Phase 0: Foundation Build (Weeks 1-4)

### Goal
Build the workspace structure, core types, event bus, database, API skeleton, and embedded frontend shell from scratch. At the end of Phase 0, `cargo build --release` produces a single binary that serves a working (but empty) web UI.

### What We're Building New (Not Refactoring)
This is a greenfield build. The existing Forge code and 62 reference repos are **reference material**, not starting points.

### Reference Material
- **Architecture**: `03-architecture/SYSTEM_ARCHITECTURE.md` (crate map, event types, data model)
- **Data model**: `03-architecture/DATA_MODEL.md` (SQLite schema)
- **Existing Forge**: process spawning patterns, WebSocket streaming approach (study, don't copy)
- **claude-code-tools**: session management patterns, Tantivy FTS approach
- **ralph-claude-code**: circuit breaker pattern, autonomous loop design

### Tasks

1. **Workspace structure** (Week 1)
   - Create workspace `Cargo.toml` with `[workspace.dependencies]`
   - Build 8 initial crates:
     - `forge-core`: types, events, traits, IDs, validation, error types
     - `forge-db`: SQLite connection pool, migrations, repositories
     - `forge-api`: Axum server, routes, middleware, WebSocket handler
     - `forge-agent`: agent types, presets, CRUD operations
     - `forge-process`: Claude CLI process spawning, stream-json parsing
     - `forge-safety`: circuit breaker, rate limiter, cost tracker (stubs)
     - `forge-mcp`: MCP protocol types (stubs)
     - `forge-app`: binary entry point, ties everything together
   - Shared workspace dependencies: tokio, serde, axum, rusqlite, uuid, chrono
   - `cargo build` and `cargo test` pass
   - `cargo clippy` clean with pedantic lints

2. **Event system** (Week 2)
   - Define `ForgeEvent` enum in `forge-core`: AgentCreated, AgentUpdated, AgentDeleted, ProcessStarted, ProcessOutput, ProcessCompleted, SessionCreated, Error
   - `EventBus` struct wrapping `tokio::sync::broadcast::Sender`
   - `EventSink` trait for testable event emission
   - Event filtering: subscribe to specific event kinds
   - Batch writer: accumulate events, flush to SQLite every 50 events or 2 seconds
   - Use `crossbeam-channel` for writer thread to avoid async in SQLite writes

3. **Database schema** (Week 3)
   - Design schema for ALL future features upfront (no migrations later):
     - `agents` (id, name, model, system_prompt, config_json, preset, created_at, updated_at)
     - `sessions` (id, agent_id, directory, status, created_at, updated_at)
     - `events` (id, session_id, event_type, data_json, timestamp)
     - `workflows` (id, name, definition_json, created_at)
     - `workflow_runs` (id, workflow_id, status, started_at, completed_at)
     - `skills` (id, name, description, category, content, source_repo, parameters_json)
     - `schedules` (id, name, cron_expr, job_type, job_config_json, enabled)
     - `audit_log` (id, actor, action, target_type, target_id, details_json, timestamp)
     - `config` (scope, key, value_json)
   - FTS5 virtual tables for search: `skills_fts`, `sessions_fts`, `events_fts`
   - Migration runner with version tracking (start at v1, design for future v2+)
   - `TestDb` helper: in-memory SQLite for tests
   - Schema documentation inline in migration SQL

4. **API skeleton + embedded frontend** (Week 4)
   - Axum router with versioned API: `/api/v1/...`
   - Health check: `GET /api/v1/health`
   - Agent CRUD stubs: `GET/POST/PUT/DELETE /api/v1/agents`
   - WebSocket endpoint: `GET /api/v1/ws` (broadcasts ForgeEvents)
   - CORS middleware (restrictive defaults)
   - Request ID middleware (tracing)
   - SvelteKit project in `frontend/`:
     - `adapter-static` for `rust-embed` integration
     - Svelte 5 with runes ($state, $derived, $effect)
     - TailwindCSS 4
     - Layout shell: sidebar nav, main content area, status bar
     - Empty pages: Dashboard, Agents, Sessions, Workflows, Skills, Settings
   - `rust-embed` serves frontend from binary
   - Single binary serves UI at `http://localhost:4173`

### Deliverables
- [x] ~~8~~ 3 workspace crates compile and pass tests (forge-core, forge-agent, forge-db)
- [x] Event bus operational with batch writer
- [x] Full database schema with FTS5 (designed for all phases)
- [ ] Remaining 5 crates (forge-api, forge-app, forge-process, forge-safety, forge-mcp)
- [ ] Fix forge-core rusqlite layering violation
- [ ] Add [workspace.dependencies] for shared dep versions
- [ ] API skeleton with health check and WebSocket
- [ ] Frontend shell served from single binary
- [ ] CI pipeline green (`cargo test`, `cargo clippy`, `cargo build --release`)

### Success Criteria
- `cargo build --release` produces a single binary < 15 MB (no frontend yet embedded is OK)
- `./forge` starts server, browser shows working UI shell
- WebSocket connection established, receives heartbeat events
- All 8 crates have at least one unit test
- Schema supports all planned features (no schema changes needed in Phase 1-5)

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Schema design misses future needs | Medium | High | Study all phase requirements before writing schema; design for extension |
| Crate boundary mistakes (wrong abstraction) | Medium | Medium | Start with fewer crates, split later if needed; keep interfaces narrow |
| Frontend build pipeline complexity | Low | Medium | Use proven SvelteKit + adapter-static + rust-embed chain |
| Scope creep in "skeleton" phase | Medium | Medium | Strict rule: stubs only, no business logic beyond CRUD |

---

## Phase 1: Agent Engine + Process Management (Weeks 5-8)

### Goal
Build the core agent orchestration engine: create agents, spawn Claude Code processes, stream output in real-time, manage sessions. At the end of Phase 1, a user can create an agent, send it a prompt, and watch the response stream live.

### Reference Material
- **Existing Forge**: process spawning with `claude -p`, stream-json parsing, `--resume` sessions
- **ralph-claude-code**: autonomous dev loop, exit detection, rate limiting
- **claude-code-hooks-mastery**: 13 hook types, builder/validator agent pattern
- **awesome-claude-code-subagents**: 127+ agent definitions, model routing patterns

### Tasks

1. **Agent CRUD** (Week 5)
   - Implement `forge-agent` crate:
     - `Agent` struct: id, name, model, system_prompt, allowed_tools, max_turns, config
     - 9 built-in presets: CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer
     - CRUD operations backed by `forge-db`
     - Agent validation (name uniqueness, model allowlist)
   - API endpoints: `GET/POST/PUT/DELETE /api/v1/agents`, `GET /api/v1/agents/:id`
   - Frontend: Agent list page, Agent create/edit form, Agent detail view
   - Emit events: AgentCreated, AgentUpdated, AgentDeleted

2. **Process spawning** (Week 6)
   - Implement `forge-process` crate:
     - Spawn `claude -p "<prompt>" --output-format stream-json --verbose`
     - Parse stream-json events: `assistant`, `tool_use`, `tool_result`, `result`, `error`
     - Handle process lifecycle: start, running, completed, failed, killed
     - `--resume <session_id>` for session continuity
     - Environment handling: `env_remove("CLAUDECODE")` to avoid nested-session guard
     - `env_remove("ANTHROPIC_API_KEY")` when `use_max=true`
     - Working directory per agent
   - Process handle with kill/cancel support
   - Timeout enforcement (configurable per agent)

3. **Real-time streaming** (Week 7)
   - Connect process output to event bus
   - WebSocket broadcasts process events to all connected clients
   - Frontend: streaming output renderer
     - Markdown rendering for assistant messages
     - Code block syntax highlighting
     - Tool use/result collapsible panels
     - Thinking/reasoning blocks (collapsible)
     - Progress indicators
   - Multi-pane layout: run multiple agents side-by-side
   - Tab management: open/close/reorder agent tabs

4. **Session management** (Week 8)
   - Session CRUD in `forge-db`
   - Session list API: `GET /api/v1/sessions`, `GET /api/v1/sessions/:id`
   - Session history: all events for a session, paginated
   - Session resume: `--resume` flag passes Claude session ID
   - Session browser frontend: list sessions by project, filter by status/date
   - Export: JSON and Markdown export of session history
   - Scan existing Claude sessions from `~/.claude/projects/`

### Deliverables
- [ ] Agent CRUD with 9 presets
- [ ] Process spawning with stream-json parsing
- [ ] Real-time WebSocket streaming to frontend
- [ ] Multi-pane tab layout
- [ ] Session management with resume
- [ ] Session browser
- [ ] Export (JSON/Markdown)

### Success Criteria
- Create agent, send prompt, see streaming response in < 2 seconds
- Process output renders Markdown, code blocks, tool calls correctly
- Session resume works: continue a previous conversation
- 3 agents running in parallel, each streaming independently
- Export produces valid JSON and readable Markdown

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| stream-json format changes in Claude CLI | Low | High | Pin Claude Code version, abstract parser |
| Process zombie on crash | Medium | Medium | SIGTERM on drop, reaper thread, PID tracking |
| WebSocket backpressure with fast output | Medium | Low | Bounded channel, drop oldest if full |

### Dependency Chain
- Requires Phase 0 (crate structure, event bus, DB schema, API skeleton)

---

## Phase 2: Workflows + Skills (Weeks 9-13)

### Goal
Build a 4-level workflow engine (sequential, parallel, conditional, loop) and index a catalog of 1,500+ skills with full-text search.

### Reference Material
- **Claude-Code-Workflow**: JSON-driven 4-level workflows, CLI orchestration
- **claude-code-spec-workflow**: spec-driven workflow (requirements -> design -> tasks -> implementation)
- **claude-code-skills**: 38 production skills, marketplace manifest.json, YAML frontmatter
- **claude-code-plugins-plus-skills**: 1,500+ skills, CCPI package manager
- **claude-code-infrastructure-showcase**: auto-skill activation hooks, skill-rules.json

### Tasks

1. **Workflow engine core** (Weeks 9-10)
   - `forge-workflow` crate:
     - Workflow DSL (YAML/JSON format)
     - Step types: Prompt, Parallel, Conditional, Loop, Handoff
     - State machine: Pending -> Running -> Completed/Failed/Cancelled
     - Step result passing (output of step N is input to step N+1)
     - Error handling: retry, skip, abort strategies per step
     - Parallel step execution with `tokio::JoinSet`
     - Workflow persistence (resume after restart)
   - API: `POST /api/v1/workflows`, `POST /api/v1/workflows/:id/run`
   - 5 built-in workflow templates: review-code, refactor, test-write, doc-gen, debug

2. **Workflow UI** (Week 11)
   - Workflow list page
   - Workflow builder: form-based step ordering
   - Run visualization: step progress, current step highlighting
   - Run history with outcome and duration

3. **Skill catalog** (Week 12)
   - `forge-skills` crate:
     - Skill schema: name, description, category, parameters, examples, source_repo
     - Import skills from reference repos (scan `refrence-repo/` for skill definitions)
     - FTS5 indexing for search
     - Category hierarchy (13 top-level categories from INDEX.md)
   - Skill detail view with parameter documentation
   - Slash-command autocomplete integration

4. **Skill execution** (Week 13)
   - Skill-to-prompt compilation: skill + parameters -> system prompt + user prompt
   - Skill chaining: output of one skill as input to another
   - Skill result formatting (Markdown, JSON, code blocks)
   - Skill usage tracking (most-used, recently-used)
   - Integration tests: skill -> agent -> result pipeline

### Deliverables
- [ ] Workflow engine executing 4-level workflows
- [ ] 5 built-in workflow templates
- [ ] Workflow builder UI
- [ ] 1,500+ skills indexed and searchable
- [ ] Slash-command autocomplete
- [ ] Skill-to-prompt compilation
- [ ] Skill usage tracking

### Success Criteria
- 5-step sequential workflow completes successfully
- 3-branch parallel workflow executes concurrently
- Conditional workflow takes correct branch based on prior step output
- Skill search returns relevant results in < 20ms
- Slash-command `/` shows autocomplete with fuzzy matching

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Workflow state machine bugs | Medium | High | Exhaustive state transition tests, property-based testing |
| Skill catalog quality varies across repos | Medium | Medium | Quality scoring during import, user rating system |
| Workflow UI complexity | Medium | Medium | Start form-based, add drag-and-drop later |

### Dependency Chain
- Requires Phase 0 (crate structure, DB schema with FTS5)
- Requires Phase 1 (agent engine for workflow step execution)

---

## Phase 3: Observability + Git (Weeks 14-18)

### Goal
Build operational dashboards and deep git integration. Users can see what every agent is doing, how much it costs, and what it changed in the codebase.

### Reference Material
- **claude-code-hooks-multi-agent-observability**: real-time agent monitoring via hooks
- **Claude-Code-Usage-Monitor**: usage tracking with predictions, warnings
- **1code**: git UI, worktree management, Kanban-style agent tracking

### Tasks

1. **Metrics collection** (Week 14)
   - Instrument all crates with `metrics` counters/gauges/histograms
   - Token usage per agent, per model, per session
   - Response latency percentiles
   - Event throughput (events/second)
   - Active agent count, WebSocket connection count

2. **Dashboards** (Weeks 15-16)
   - Main dashboard: active agents, recent runs, system health
   - Cost dashboard: daily/weekly/monthly spend, per-agent breakdown, budget usage
   - Agent swim lanes: parallel timeline of agent activities
   - Session timeline: event sequence visualization
   - Real-time updates via WebSocket

3. **Git integration** (Weeks 17-18)
   - `forge-git` crate wrapping `git2`:
     - Status: modified, staged, untracked files
     - Diff: file-level and hunk-level diffs with syntax highlighting
     - Log: commit history with author, date, message, diff stats
     - Branch: list, create, checkout, delete
     - Worktree: list, create, remove (for agent isolation)
   - Git panel in frontend: status summary, diff viewer, commit log
   - Auto-detect working directory changes

### Deliverables
- [ ] Metrics collection across all crates
- [ ] Main dashboard with real-time updates
- [ ] Cost tracking dashboard
- [ ] Agent swim lane visualization
- [ ] Git status, diff, log operations via libgit2
- [ ] Git panel in frontend
- [ ] Worktree management for agent isolation

### Success Criteria
- Dashboard loads in < 1 second with 100 agents' history
- Cost tracking accurate within 5% of actual API costs
- Git status matches `git status` output
- Diff viewer renders hunks with syntax highlighting
- Worktree creation takes < 2 seconds

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| libgit2 edge cases (submodules, LFS) | Medium | Medium | Start with core operations, iterate |
| Dashboard performance with large history | Medium | Medium | Pagination, time-window queries, indexing |
| Swim lane rendering complexity | Low | Low | Start with simple timeline, enhance later |

### Dependency Chain
- Requires Phase 0 (crate structure, event bus)
- Requires Phase 1 (process events for metrics)
- `forge-git` has no phase dependency (can start anytime after Phase 0)

---

## Phase 4: Safety + MCP (Weeks 9-12, parallel with Phase 2)

### Goal
Add safety controls and an MCP server. This phase runs **in parallel** with Phase 2 because it depends only on Phase 0+1, not on workflows/skills.

### Reference Material
- **ralph-claude-code**: circuit breaker pattern, rate limiting
- **claude-code-tools**: safety hooks, env-safe, vault
- **claude-code-mcp**: Claude Code as MCP server pattern
- **claude-code-hub**: API proxy, load balancing, monitoring

### Tasks

1. **Circuit breaker** (Week 9)
   - Implement in `forge-safety` crate:
     - Three-state machine: Closed -> Open -> HalfOpen -> Closed
     - Configurable failure threshold, timeout, success threshold
     - Per-agent and global circuit breakers
   - Dashboard widget showing circuit states
   - Unit tests for all state transitions

2. **Rate limiter + cost tracking** (Week 10)
   - Token bucket rate limiter (per-agent, per-model, global)
   - Cost tracking: token counts * model pricing
   - Budget enforcement: hard limit (reject) and soft limit (warn)
   - Cost dashboard with time-series chart

3. **MCP server** (Weeks 11-12)
   - Implement in `forge-mcp` crate:
     - MCP protocol (JSON-RPC over stdio/SSE)
     - 10 initial tools: agent_create, agent_list, agent_run, session_list, session_get, workflow_create, workflow_run, skill_search, config_get, config_set
     - 5 initial resources: agent://list, session://list, skill://catalog, config://current, status://health
   - Protocol compliance tests against MCP spec
   - Multiple simultaneous client connections

### Deliverables
- [ ] Circuit breaker protecting all agent operations
- [ ] Rate limiter with configurable limits
- [ ] Cost tracking with budget enforcement
- [ ] MCP server accepting connections
- [ ] 10 MCP tools + 5 resources functional
- [ ] Safety dashboard in frontend

### Success Criteria
- Circuit breaker opens after N failures, prevents cascading failures
- Rate limiter correctly throttles at configured rate
- Cost tracking within 5% of actual API costs
- MCP server passes protocol compliance tests
- Claude Desktop can connect and use Forge tools via MCP

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| MCP spec changes | Medium | Medium | Pin protocol version, abstract protocol layer |
| Cost calculation inaccuracy | Medium | Low | Conservative estimates, user-configurable pricing |

### Dependency Chain
- Requires Phase 0 (crate structure, DB) + Phase 1 (agent engine)
- **No dependency on Phase 2 or 3** -- runs in parallel

---

## Phase 5: Plugins + Security + Polish (Weeks 19-24)

### Goal
Enable community extensibility via WASM plugins, add audit logging and permissions, and polish for production. This is the 1.0 release phase.

### Reference Material
- **claude-code-plugins-plus-skills**: CCPI package manager, plugin architecture
- **claude-code-skill-factory**: toolkit for building/deploying skills
- **claude-code-security-review**: security review patterns
- **claude-code-action**: GitHub Action automation patterns

### Tasks

1. **WASM plugin host** (Weeks 19-21)
   - `forge-plugins` crate with Wasmtime runtime
   - Plugin API (WIT interface): access to agents, events, skills, config
   - Resource limits: memory (64 MB), fuel (1M instructions), filesystem (scoped)
   - Plugin lifecycle: install, enable, disable, uninstall
   - Plugin manifest (TOML): name, version, author, permissions, entry point
   - Plugin registry UI: browse installed, install new, configure
   - 3 example plugins: custom-formatter, slack-integration, metrics-exporter
   - Plugin SDK (Rust template project)

2. **Security hardening** (Weeks 22-23)
   - Audit log: all state-changing operations logged with actor, timestamp, details
   - Audit log viewer in frontend with filtering
   - Permission model: define what agents/plugins can access
   - Secret management: encrypted storage for API keys, tokens
   - Input validation audit: all API endpoints
   - CORS, CSP headers, rate limiting on all endpoints

3. **1.0 polish** (Week 24)
   - Performance profiling and optimization
   - Error message review (user-friendly, actionable)
   - Loading states and skeleton screens for all pages
   - Keyboard shortcuts (global and per-page)
   - Binary size optimization (< 35 MB target)
   - Documentation: user guide, API reference, plugin guide

### Deliverables
- [ ] WASM plugin runtime operational
- [ ] Plugin API with WIT interface
- [ ] 3 example plugins + SDK
- [ ] Audit log with viewer
- [ ] Permission model + secret management
- [ ] Security hardening complete
- [ ] Performance optimized
- [ ] User documentation
- [ ] **1.0 release binary for macOS, Linux, Windows**

### Success Criteria
- Plugin loads and executes within 100ms
- Plugin cannot exceed memory/fuel limits (tested)
- Audit log captures all state changes with < 5ms overhead
- All API endpoints have input validation
- Binary size < 35 MB (< 50 MB acceptable)
- Lighthouse accessibility score > 90

### Dependency Chain
- Requires all prior phases
- Security hardening touches all crates (cross-cutting)

---

## Phase 6: Dev Environment (Post-1.0, Weeks 25-29)

### Goal
Transform Forge from an agent orchestrator into a full development environment. This is a post-1.0 enhancement.

### Reference Material
- **1code**: multi-agent desktop, code viewer, file tree
- **claude_code_bridge**: split-pane terminal, multi-CLI
- **claude-code-viewer**: web-based client with interactive features

### Tasks

1. **Code viewer** (Weeks 25-26)
   - File read API endpoint with line range support
   - Syntax highlighting via `shiki` (50+ languages)
   - Side-by-side diff view (using git diff from Phase 3)
   - File tabs

2. **Embedded terminal** (Weeks 27-28)
   - PTY allocation and management (server-side)
   - `xterm.js` frontend with WebSocket connection
   - Terminal tabs, resizing, shell detection
   - Working directory sync with agent's project

3. **File explorer** (Week 29)
   - Directory tree API (lazy-loading)
   - File tree component, file type icons
   - File search (fuzzy find by path)
   - Integration: click file -> opens in code viewer

### Dependency Chain
- Requires Phase 3 (git integration for diff viewer)
- Self-contained frontend components

---

## Critical Path

```
Phase 0 (4w) -> Phase 1 (4w) -> Phase 2 (5w) -> Phase 3 (5w) -> Phase 5 (6w)
                                  |
                                  +-> Phase 4 (4w, parallel with Phase 2)

Critical path: 24 weeks (Phases 0->1->2->3->5)
```

### Parallelization

Phase 4 (Safety + MCP) has **no dependency on Phase 2 or 3**. It only needs Phase 0+1. This means:

```
Weeks  1-4:   Phase 0 (Foundation Build)
Weeks  5-8:   Phase 1 (Agent Engine)
Weeks  9-13:  Phase 2 (Workflows + Skills)  ← parallel
Weeks  9-12:  Phase 4 (Safety + MCP)        ← parallel
Weeks  14-18: Phase 3 (Observability + Git)
Weeks  19-24: Phase 5 (Plugins + Polish + 1.0 Release)
```

**With parallel Claude Code sessions:**
- Session A: Phase 0 -> 1 -> 2 -> 3 -> 5
- Session B: Phase 4 (starts week 9, after Phase 1)
- Session C: Phase 6 (starts week 25, post-1.0)

**Single-track critical path: 24 weeks** (under the 26-week vision target).

---

## How Reference Repos Feed Each Phase

| Phase | Key Repos to Study | What to Extract |
|-------|-------------------|-----------------|
| 0 | `claude-code-tools` | Workspace structure, Tantivy FTS patterns |
| 1 | `ralph-claude-code`, `awesome-claude-code-subagents` | Process spawning, agent presets, exit detection |
| 2 | `Claude-Code-Workflow`, `claude-code-skills`, `claude-code-spec-workflow` | Workflow DSL, skill schema, 1,500+ skill definitions |
| 3 | `claude-code-hooks-multi-agent-observability`, `1code` | Dashboard patterns, git UI, worktree management |
| 4 | `claude-code-mcp`, `claude-code-hub` | MCP protocol, proxy patterns, safety hooks |
| 5 | `claude-code-plugins-plus-skills`, `claude-code-action` | Plugin architecture, GitHub Actions, security patterns |
| 6 | `1code`, `claude_code_bridge`, `claude-code-viewer` | Code viewer, terminal, file explorer patterns |

---

## Decision Log

| Decision | Rationale | Date |
|----------|-----------|------|
| Start from scratch (not refactor) | 62-repo analysis revealed architecture needs new foundation; existing code designed before full knowledge | 2026-02-25 |
| 8 crates initially (not 12) | Fewer crates = faster builds, cleaner boundaries; split later if needed | 2026-02-25 |
| Phase 4 parallel with Phase 2 | Safety/MCP has no workflow dependency; saves 4 weeks on critical path | 2026-02-25 |
| Schema designed upfront (Phase 0) | Avoid migration pain; study all phase requirements first | 2026-02-25 |
| Phase 6 is post-1.0 | IDE features are additive; core value is orchestration + safety | 2026-02-25 |
| Use existing Forge as reference only | Study patterns (process spawn, WebSocket), don't copy code wholesale | 2026-02-25 |
