# AgentForge Codebase Architecture Audit

**Date:** 2026-03-16
**Auditor:** Staff Engineer (Code-Reviewer)
**Scope:** Full codebase — Rust backend (13 crates), SvelteKit frontend, CI/CD, migrations, tests

---

## 1. Architecture Overview

### 1.1 High-Level Structure

AgentForge is a **monolithic Rust server** with an **embedded SvelteKit SPA** for managing autonomous AI agent teams. It exposes a REST + WebSocket API and an MCP (Model Context Protocol) stdio binary.

```
┌─────────────────────────────────────────────────────┐
│                    forge-app                         │
│  (binary: HTTP server, scheduler, initialization)    │
├─────────────────────────────────────────────────────┤
│                    forge-api                          │
│  (Axum routes, CORS, OpenAPI, embedded SPA, WS)      │
├──────────┬──────────┬───────────┬───────────────────┤
│forge-    │forge-    │forge-     │forge-safety       │
│process   │db        │persona    │(circuit breaker,  │
│(backends,│(SQLite,  │(catalog,  │ rate limiter,     │
│ runners, │ repos,   │ parser,   │ cost tracker,     │
│ pipeline)│ batch)   │ mapper)   │ scanner)          │
├──────────┴──────────┴───────────┴───────────────────┤
│  forge-agent  │  forge-org  │  forge-governance      │
│  (models,     │  (companies,│  (goals, approvals)    │
│   presets,    │  depts, org │                         │
│   strategy)   │  chart)     │                         │
├─────────────────────────────────────────────────────┤
│                   forge-core                          │
│  (ForgeError, IDs, Events, EventBus, EventSink)      │
├─────────────────────────────────────────────────────┤
│  forge-git       │  forge-mcp       │  forge-mcp-bin │
│  (worktrees)     │  (JSON-RPC stubs)│  (MCP server)  │
└──────────────────┴──────────────────┴────────────────┘
```

### 1.2 Rust Crates (13 total)

| Crate | Role | Key Types |
|-------|------|-----------|
| **forge-core** | Foundation: errors, IDs, events, event bus | `ForgeError`, `ForgeEvent`, `EventBus`, `AgentId`, `SessionId` |
| **forge-agent** | Agent model, 10 presets, best-of-N strategies, validation | `Agent`, `AgentPreset`, `Strategy`, `NewAgent` |
| **forge-db** | SQLite persistence: pool, 18 repos, batch writer, migrations | `DbPool`, `UnitOfWork`, `Migrator`, `BatchWriter` |
| **forge-process** | Process execution: backends, spawning, streaming, pipelines | `ProcessBackend` trait, `ClaudeBackend`, `ProcessRunner`, `Pipeline` |
| **forge-safety** | Production safety: circuit breaker, rate limiter, cost tracker, scanner | `CircuitBreaker`, `RateLimiter`, `CostTracker`, `SecurityScanner` |
| **forge-api** | Axum HTTP server: 25+ REST endpoints, WebSocket, OpenAPI, embedded SPA | `AppState`, `app()`, `serve()` |
| **forge-org** | Org structure models | `Company`, `Department`, `OrgPosition`, `OrgChartNode` |
| **forge-governance** | Governance models | `Goal`, `Approval` |
| **forge-persona** | Persona catalog: parsing, mapping to agents | `Persona`, `PersonaDivision`, `PersonaCatalog`, `PersonaParser` |
| **forge-git** | Git worktree isolation per session | `WorktreeInfo`, `create_worktree()` |
| **forge-mcp** | MCP JSON-RPC type stubs (no implementation) | `McpRequest`, `McpResponse`, `McpTool` |
| **forge-mcp-bin** | MCP stdio server binary (20+ tools via rmcp) | Agent/session/org/approval MCP tools |
| **forge-app** | Main binary: startup, migration, scheduler, graceful shutdown | `main()`, `Scheduler` |

### 1.3 SvelteKit Frontend (17 routes)

| Route | Purpose |
|-------|---------|
| `/` | Dashboard: run agents, stream output via WebSocket, swim-lane sub-agent view |
| `/agents` | Agent CRUD with preset selection and stats |
| `/sessions` | Session history (list + kanban), export (JSON/Markdown/HTML) |
| `/skills` | Skill catalog browser with category filter |
| `/workflows` | Workflow editor (sequential/fanout step pipelines) |
| `/memory` | Memory bank: CRUD, search, confidence scoring |
| `/hooks` | Event hooks: shell commands on process lifecycle |
| `/schedules` | Cron-based scheduled agent execution |
| `/companies` | Company management with budgets |
| `/personas` | 100+ pre-built personas, hire into org |
| `/org-chart` | Recursive org hierarchy visualization |
| `/goals` | Hierarchical goal tracking |
| `/approvals` | Approval workflow (pending/approved/rejected) |
| `/analytics` | Usage analytics: daily costs, agent breakdown, success rates |
| `/backends` | Backend health and capabilities |
| `/settings` | System config, health, environment |

### 1.4 Shared Components

| Component | Purpose |
|-----------|---------|
| `EmptyState.svelte` | Reusable empty state with icon, title, action |
| `ErrorMessage.svelte` | Error alert with optional retry |
| `Markdown.svelte` | Safe rendering via `marked` + `DOMPurify` |
| `Skeleton.svelte` | Loading placeholders (card/table/text) |
| `OrgNode.svelte` | Recursive org chart node with collapse/expand |
| `Onboarding.svelte` | Getting started guide |
| `focusTrap.ts` | Modal accessibility (Tab key constraint) |

---

## 2. Data Flow

### 2.1 Request Lifecycle (Frontend → Backend)

```
SvelteKit Page
  │  fetch('/api/v1/agents')
  ▼
lib/api.ts (type-safe client, 880 lines)
  │  handleResponse<T>() error extraction
  ▼
Axum Router (forge-api)
  │  CORS middleware, JSON extraction
  ▼
Route Handler (e.g., routes/agents.rs)
  │  Extract AppState
  ▼
UnitOfWork → Repository (forge-db)
  │  rusqlite query → SQLite
  ▼
JSON response → SvelteKit → $state() → Render
```

### 2.2 Real-Time Streaming (WebSocket)

```
Frontend: new WebSocket('/api/v1/events/stream')
  ▲
  │  JSON messages (ForgeEvent variants)
  │
forge-api/routes/ws.rs
  │  EventBus.ui_receiver() → broadcast channel
  │
EventBus (forge-core)
  │  Fan-out: persistence channel (mpsc, guaranteed)
  │           + UI broadcast channel (best-effort)
  │
ProcessRunner (forge-process)
  │  Spawns child process, parses streaming JSON output
  │  Emits: ProcessStarted, ProcessOutput, ProcessCompleted,
  │         SubAgentRequested, SubAgentStarted, etc.
  │
ClaudeBackend → claude CLI (child process)
```

### 2.3 Event Persistence

```
ForgeEvent
  │  EventBus.emit()
  ▼
BatchWriter (crossbeam channel, NOT Tokio)
  │  Accumulates events
  │  Flushes on: BATCH_SIZE=50 or FLUSH_INTERVAL=2s
  ▼
EventRepo → SQLite (append-only events table)
```

### 2.4 Agent Execution Flow

```
POST /api/v1/run { agent_id, prompt, session_id?, working_dir }
  ▼
routes/run.rs → tokio::spawn (async)
  ▼
ProcessRunner.run()
  │  1. Load agent config
  │  2. Select backend (ClaudeBackend)
  │  3. Build SpawnConfig (command, args, env, timeout)
  │  4. Spawn child process
  │  5. Parse streaming stdout (newline-delimited JSON)
  │  6. Map to ForgeEvent, emit to EventBus
  │  7. Detect loops (LoopDetector)
  │  8. Check exit gates (ExitGateConfig)
  ▼
EventBus → BatchWriter → SQLite
         → WebSocket → Frontend (live updates)
```

### 2.5 MCP Binary (Separate Process)

```
External MCP Client (Claude Desktop, etc.)
  │  stdin: JSON-RPC request
  ▼
forge-mcp-bin (stdio transport via rmcp)
  │  Routes to tool handler (e.g., create-agent)
  ▼
UnitOfWork → SQLite (direct DB access, no HTTP)
  │
  ▼  stdout: JSON-RPC response
External MCP Client
```

**Note:** `forge-mcp-bin` accesses the same SQLite database directly — it does NOT go through the HTTP API. This means the MCP binary and HTTP server can conflict if running simultaneously on the same DB.

---

## 3. Integration Points

### 3.1 API Boundary (HTTP REST)

The primary integration surface between frontend and backend. All endpoints under `/api/v1/`:

| Category | Endpoints | Verbs |
|----------|-----------|-------|
| Health | `/health` | GET |
| Agents | `/agents`, `/agents/:id`, `/agents/presets`, `/agents/:id/stats` | GET, POST, PUT, DELETE |
| Sessions | `/sessions`, `/sessions/:id`, `/sessions/:id/events`, `/sessions/:id/export/*` | GET, POST, DELETE |
| Run | `/run` | POST |
| Skills | `/skills` | GET |
| Workflows | `/workflows`, `/workflows/:id`, `/workflows/:id/run` | GET, POST, PUT, DELETE |
| Memory | `/memory`, `/memory/:id`, `/memory/search` | GET, POST, PUT, DELETE |
| Hooks | `/hooks`, `/hooks/:id` | GET, POST, PUT, DELETE |
| Schedules | `/schedules`, `/schedules/:id` | GET, POST, PUT, DELETE |
| Personas | `/personas`, `/personas/divisions`, `/personas/:id/hire` | GET, POST |
| Companies | `/companies`, `/companies/:id` | GET, POST, PUT, DELETE |
| Departments | `/companies/:id/departments` | GET, POST |
| Org Positions | `/companies/:id/positions` | GET, POST |
| Org Chart | `/companies/:id/org-chart` | GET |
| Goals | `/goals` | GET, POST, PUT |
| Approvals | `/approvals`, `/approvals/:id` | GET, POST, PUT |
| Analytics | `/analytics/usage` | GET |
| Backends | `/backends`, `/backends/:id/health` | GET |
| Settings | `/settings` | GET |
| WebSocket | `/events/stream` | WS |
| OpenAPI | `/openapi.json` | GET |

### 3.2 Shared Type Contract

There is **no formal shared type contract** between Rust and TypeScript. The frontend `api.ts` defines TypeScript interfaces that mirror the Rust structs manually:

| Rust (forge-api response) | TypeScript (api.ts) | Sync Method |
|--------------------------|--------------------|-|
| `Agent` struct | `Agent` interface | Manual |
| `Session` struct | `Session` interface | Manual |
| `Workflow` struct | `Workflow` interface | Manual |
| `ForgeEvent` enum | Event type checks | Manual |
| `UsageReport` struct | `UsageReport` interface | Manual |

**Risk:** Type drift between Rust and TypeScript. OpenAPI schema exists (`/api/v1/openapi.json` via utoipa) but is not used for TypeScript codegen.

### 3.3 WebSocket Protocol

- Endpoint: `/api/v1/events/stream`
- Format: JSON-serialized `ForgeEvent` variants
- No formal schema or versioning for WS messages
- Frontend filters by event type (string matching)

### 3.4 Process/CLI Coupling

`ClaudeBackend` spawns `claude` CLI as a child process:
- Communication: stdout streaming (newline-delimited JSON)
- Parsed via `StreamJsonEvent` → mapped to `ForgeEvent`
- **Tight coupling** to Claude CLI output format

### 3.5 Database Coupling

- **forge-app** and **forge-mcp-bin** both access the same SQLite database directly
- No coordination mechanism between them (no file locks beyond SQLite's WAL)
- forge-db uses WAL mode with busy_timeout=5000ms

### 3.6 File System Coupling

| Path | Used By | Purpose |
|------|---------|---------|
| `~/.agentforge/forge.db` | forge-app, forge-mcp-bin | SQLite database |
| `~/.claude-forge/` | forge-mcp-bin (legacy fallback) | Legacy DB path |
| `.worktrees/<session_id>` | forge-git | Git worktree isolation |
| `personas/` directory | forge-persona parser | Built-in persona definitions |
| `frontend/build/` | forge-api (rust-embed) | Embedded SPA assets |

### 3.7 Environment Variables

| Variable | Default | Used By |
|----------|---------|---------|
| `FORGE_HOST` | `127.0.0.1` | forge-app |
| `FORGE_PORT` | `3000` | forge-app |
| `FORGE_DB_PATH` | `~/.agentforge/forge.db` | forge-app |
| `CORS_ORIGIN` | `*` | forge-api |
| `RUST_LOG` | (unset) | tracing |
| `VITE_API_URL` | (same-origin) | frontend |

---

## 4. Inconsistencies Found

### 4.1 CRITICAL: Error Handling Divergence

**Pattern A (canonical):** `forge-core::ForgeError` — stratified errors with HTTP status mapping.

**Pattern B (divergent):** `forge-process/src/parse.rs` defines its own `ParseError` via thiserror instead of wrapping into `ForgeError`. This breaks the error hierarchy.

**Pattern C (panic-prone):** 40+ instances of `.lock().expect("db mutex poisoned")` and `.lock().unwrap()` across:
- `forge-db/src/pool.rs:128, 140`
- `forge-db/src/batch_writer.rs:90`
- `forge-db/src/repos/schedules.rs` (7 instances)
- `forge-db/src/repos/analytics.rs`, `compaction.rs`, `departments.rs` (all repos)
- `forge-safety/src/lib.rs:78, 113, 119, 142, 167, 183, 208`

These will **panic and crash the server** if a mutex is ever poisoned.

### 4.2 HIGH: Missing Database Layer Tests

The entire `forge-db/src/repos/` directory (18 repositories) has **zero unit tests**. This is the most critical persistence layer and has no test coverage for:
- agents, sessions, events, skills, workflows, memory, hooks, schedules
- analytics, compaction, companies, departments, org_positions, goals, approvals, personas, safety

Only `forge-api` has integration tests that exercise repos indirectly through HTTP handlers.

### 4.3 HIGH: No Shared Type Contract

Frontend TypeScript interfaces in `api.ts` are maintained **manually** in parallel with Rust structs. The OpenAPI schema is generated via utoipa but not consumed for TypeScript codegen. Any field rename, type change, or new variant in Rust can silently break the frontend.

### 4.4 MEDIUM: Dual Database Access Without Coordination

Both `forge-app` (HTTP server) and `forge-mcp-bin` (MCP stdio server) access the same SQLite database directly. While SQLite's WAL mode handles concurrent reads, concurrent writes from both processes could cause `SQLITE_BUSY` errors.

### 4.5 MEDIUM: Logging Inconsistency

`forge-app/src/main.rs` and `forge-mcp-bin/src/main.rs` use `eprintln!()` for startup errors instead of structured `tracing` logging. All other crates consistently use tracing.

### 4.6 MEDIUM: Excessive Cloning in Event Construction

`forge-process/src/runner.rs` has **18+ `.clone()` calls** in ~100 lines, repeatedly cloning `SessionId` and `AgentId` for each `ForgeEvent`. These are UUID wrappers (16 bytes) so the cost is small, but the pattern is noisy and could use references or `Copy`.

`forge-process/src/stream_event.rs` similarly clones content strings (potentially large) at lines 129, 142, 149, 152, 168, 171.

### 4.7 MEDIUM: Silent Error Swallowing in Frontend

Several frontend pages silently catch and discard API errors:
- `+page.svelte:163` — `.catch(() => { /* ignore metadata fetch errors */ })`
- `sessions/+page.svelte:107` — `try { ... } catch { continue; }`
- `agents/+page.svelte:70` — `.catch(() => {})`
- `sessions/+page.svelte:141` — `.then(...).catch(() => {})`
- `api.ts:667` — `.catch(() => 'Delete failed')`

Some pages properly display errors via `ErrorMessage` component; others don't. No consistent error boundary pattern.

### 4.8 MEDIUM: Regex Unwrap at Initialization

`forge-safety/src/scanner.rs` constructs 9 regex patterns with `.unwrap()` at initialization time (lines 48-96). If any regex is invalid, the entire server panics on startup. Should use `lazy_static` or `once_cell` with proper error handling, or compile-time regex validation.

### 4.9 LOW: Migration Numbering Gap

Migrations jump from `0009_personas.sql` to `0011_org_charts.sql` — there is no `0010_*.sql`. This doesn't break anything (the Migrator applies by version number) but is confusing.

### 4.10 LOW: forge-mcp Crate is Nearly Empty

`forge-mcp/src/lib.rs` contains only lightweight JSON-RPC type stubs (`McpRequest`, `McpResponse`, `McpTool`, `McpResource`). Meanwhile, `forge-mcp-bin` uses the external `rmcp` crate for actual MCP implementation. The `forge-mcp` crate appears to serve no purpose — `forge-mcp-bin` doesn't depend on it.

### 4.11 LOW: Inconsistent Serde Derive Patterns

Some structs use `#[serde(rename_all = "...")]` and some don't. The JSON field naming convention is not documented or enforced project-wide. Most use Rust's default snake_case, which aligns with the TypeScript interfaces, but this is coincidental rather than intentional.

### 4.12 LOW: No Max-Depth Guard on Org Chart Recursion

`forge-org/src/service.rs` builds org chart trees recursively via `build_org_chart()`. The `OrgNode.svelte` component renders them recursively. Neither has a max-depth guard — a circular `reports_to` reference would cause infinite recursion/stack overflow.

---

## 5. Summary

### What Works Well

- **Clean layered architecture**: Core → Models → DB → Process → API → App separation is solid
- **Event-driven design**: EventBus with fan-out to persistence and UI channels
- **Safety infrastructure**: Circuit breaker, rate limiter, cost tracker, security scanner
- **Pluggable backends**: `ProcessBackend` trait allows swappable execution engines
- **Git worktree isolation**: Concurrent agent sessions don't conflict
- **Frontend quality**: Svelte 5 runes, DOMPurify sanitization, accessibility, responsive design
- **Comprehensive CI**: clippy, cargo-deny, cargo-audit, multi-platform releases

### Top Risks

1. **Panic-prone mutex handling** in database and safety layers (40+ instances)
2. **No database repo tests** — the most critical layer is untested
3. **Manual type sync** between Rust and TypeScript — high drift risk
4. **Dual DB access** from forge-app and forge-mcp-bin without coordination
5. **Silent frontend error swallowing** — user-facing failures are hidden

### Crate Dependency Graph

```
forge-core ─────────────────────────────┐
  ├── forge-agent                        │
  ├── forge-process ── forge-agent       │
  ├── forge-safety                       │
  ├── forge-git                          │
  └── forge-db ── forge-agent            │
       │          forge-persona          │
       │                                 │
forge-api ── forge-db                    │
             forge-process               │
             forge-safety                │
             forge-org                   │
             forge-persona               │
                                         │
forge-app ── forge-api ──────────────────┘
             forge-db
             forge-process
             forge-safety
             forge-persona

forge-mcp-bin ── forge-db
                 forge-agent
                 forge-process
                 forge-persona
                 rmcp (external)

forge-org ── (standalone, serde only)
forge-governance ── (standalone, serde only)
forge-mcp ── (standalone, serde only, unused?)
```
