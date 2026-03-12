# Claude Forge — Project Context

> For AI agents and humans starting a new session.

## What This Is

Multi-agent Claude Code orchestrator. Rust/Axum backend + Svelte 5 frontend, shipped as a single binary. The only Rust-native tool in this space.

## Tech Stack

- **Backend:** Rust, Axum, SQLite (WAL mode, rusqlite), tokio
- **Frontend:** SvelteKit 5, adapter-static, TailwindCSS 4, embedded via rust-embed
- **MCP Server:** rmcp v0.17 (official Rust MCP SDK), stdio transport
- **Safety:** Circuit breaker (3-state FSM), rate limiter (token bucket), CostTracker (budget warn/limit)

## Workspace Crates (9)

```
forge-app          binary: DB setup, API server, embedded frontend, graceful shutdown, cron scheduler
├── forge-api      Axum HTTP + WebSocket, routes, CORS, TraceLayer, rust-embed SPA
├── forge-process  spawn Claude CLI, stream-json parsing, ConcurrentRunner, LoopDetector
├── forge-agent    agent model, 10 presets (incl. Coordinator), validation
├── forge-db       SQLite WAL, 5 migrations, AgentRepo, SessionRepo, EventRepo, SkillRepo, MemoryRepo, HookRepo, ScheduleRepo, AnalyticsRepo, BatchWriter
├── forge-core     ForgeEvent (35 variants), EventBus broadcast, shared types
├── forge-safety   CircuitBreaker, RateLimiter, CostTracker
├── forge-git      git worktree create/remove/list for multi-agent isolation
└── forge-mcp-bin  MCP stdio server (rmcp, 10 tools)
```

## Build & Test

```bash
# Frontend (must build first — rust-embed needs frontend/build/)
cd frontend && pnpm install && pnpm build && cd ..

# Backend
cargo build --release
cargo test              # 150 tests, all should pass
cargo check             # should be zero warnings

# Run
./target/release/forge  # serves at http://127.0.0.1:4173
```

## Key Environment Variables

| Var | Default | Purpose |
|-----|---------|---------|
| `FORGE_DB_PATH` | `~/.claude-forge/forge.db` | SQLite database path |
| `FORGE_PORT` | `4173` | Server port |
| `FORGE_HOST` | `127.0.0.1` | Bind address |
| `FORGE_CLI_COMMAND` | `claude` | CLI executable to spawn |
| `FORGE_CORS_ORIGIN` | `*` | CORS allowed origin |
| `FORGE_RATE_LIMIT_MAX` | `10` | Rate limiter max tokens |
| `FORGE_RATE_LIMIT_REFILL_MS` | `1000` | Rate limiter refill interval (ms) |
| `FORGE_BUDGET_WARN` | *(none)* | Warning threshold (USD) |
| `FORGE_BUDGET_LIMIT` | *(none)* | Hard limit (USD) |

## Conventions

- **Zero warnings policy:** `cargo check` must produce zero warnings before committing
- **All tests pass:** `cargo test` must be green
- **Frontend state:** Svelte 5 runes (`$state`, `$derived`) across all pages
- **Error handling:** `ForgeError` hierarchy in forge-core, propagated via `ForgeResult<T>`
- **IDs:** Newtype wrappers (`AgentId`, `SessionId`, `ScheduleId`) around `uuid::Uuid`
- **Events:** All state changes emit `ForgeEvent` variants (35 types) through `EventBus` (broadcast channel)
- **Persistence:** `BatchWriter` batches events (50 or 2s flush) in transactions

## Active Docs (read these)

| File | What |
|------|------|
| `NORTH_STAR.md` | Vision, current state, sprint plan (for forge-project) |
| `MASTER_TASK_LIST.md` | Sprint tasks with What/Where/How/Verify (for v0.5/v0.6) |
| `docs/V060_SPRINT_PLAN.md` | v0.6.0 sprint plan (7 agents, 3 waves) |
| `docs/agents/V060_WAVE_PROMPTS.md` | Copy-paste prompts for v0.6.0 parallel agents |
| `docs/RESEARCH_FINDINGS_2026_03_05.md` | Patterns from 67 repos (historical) |
| `docs/DOC_INDEX.md` | What's current vs archived for forge-project v0.5/v0.6 |

## Documentation Map (AgentForge)

> AgentForge planning now lives one level up, in the workspace `/docs` folder.  
> Use this table as the single jumping-off point for new sessions.

| Topic | Primary doc | Notes |
|-------|-------------|-------|
| Product vision & strategy | `/docs/strategy/EXECUTIVE_SUMMARY.md` | What/why/how in ~2 pages |
| Full proposal & architecture | `/docs/strategy/PROPOSAL.md` | End-to-end AgentForge proposal |
| Build phases & releases | `/docs/strategy/BUILD_PLAN.md` | Phase-by-phase plan and releases |
| Epics & product requirements | `/docs/product/EPIC_INDEX.md` | E1–E9 with scope and status |
| Expansion plan (8 repos → AgentForge) | `/docs/engineering/EXPANSION_PLAN.md` | Master expansion plan |
| Multi-agent development process | `/docs/engineering/MULTI_AGENT_DEVELOPMENT_SYSTEM.md` | How to run many agents safely |
| Current sprint plan | `/docs/sprints/SPRINT_PLAN.md` | Points to the active sprint (e.g. S1) |
| Forge implementation tasks (Epic 1) | `/docs/engineering/EPIC1_FOUNDATION_TASKS.md` | Story-level tasks for org + personas |

When in doubt:

1. Start at `/docs/INDEX.md` to understand the **global AgentForge plan**.  
2. Then use this `CLAUDE.md` and `forge-project/docs/README.md` to see how that plan maps into the Rust + Svelte codebase.

## Epic 1 Baseline (Org + Personas + Governance)

The following slices are now implemented and safe for agents/humans to rely on:

- **Backend APIs**
  - `GET /api/v1/companies`, `POST /api/v1/companies`
  - `POST /api/v1/departments`, `GET /api/v1/departments?company_id=...`
  - `POST /api/v1/org-positions`, `GET /api/v1/org-positions?company_id=...`
  - `GET /api/v1/org-chart?company_id=...`
  - `GET /api/v1/personas`, `GET /api/v1/personas/:id`
  - `POST /api/v1/personas/:id` — hire persona → creates `agent` + `org_position`, links back via `agents.persona_id`
  - `GET /api/v1/goals?company_id=...`, `POST /api/v1/goals`, `PATCH /api/v1/goals/:id/status`
  - `GET /api/v1/approvals?company_id=...&status=...`, `POST /api/v1/approvals`, `PATCH /api/v1/approvals/:id`

- **Frontend pages**
  - `/companies` — manage companies (name, mission, budget)
  - `/org-chart` — visualize org hierarchy per company
  - `/personas` — browse persona catalog and hire into a company/org
  - `/goals` — define and update per-company goals
  - `/approvals` — review and resolve approval requests

A typical Epic 1 flow is:

1. Create a **company** in `/companies`.  
2. Use `/personas` to **hire personas** into that company (creates agents + org positions).  
3. Inspect hierarchy in `/org-chart`.  
4. Capture intent in `/goals` and keep status updated.  
5. Use `/approvals` as the thin governance layer for decisions that need an explicit yes/no.

## Current Phase

**v0.5.0 shipped** — 150 tests, 12 pages, 35 events. **v0.6.0 planned** — 7 agents, 3 waves.

Next sprint (v0.6.0): Best-of-N selection, context pruner, pipeline engine, OpenAPI docs, typed memory, auto-skills, swim-lane dashboard, pipeline builder UI.

See `docs/V060_SPRINT_PLAN.md` for plan, `docs/agents/V060_WAVE_PROMPTS.md` for agent prompts.

## Don't

- Don't update files in `00-08/` directories (frozen reference from Feb 2026)
- Don't update files in `docs/planning/` (archived)
- Don't treat the old 305-feature roadmap as current — use MASTER_TASK_LIST.md
- Don't add features beyond what's in the current sprint
