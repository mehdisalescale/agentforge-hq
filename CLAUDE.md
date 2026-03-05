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
| `NORTH_STAR.md` | Vision, current state, sprint plan |
| `MASTER_TASK_LIST.md` | Sprint tasks with What/Where/How/Verify |
| `docs/V050_SPRINT_PLAN.md` | v0.5.0 sprint plan (10 agents, 3 waves) |
| `docs/RESEARCH_FINDINGS_2026_03_05.md` | Patterns from 67 repos |
| `docs/DOC_INDEX.md` | What's current vs archived |

## Current Phase

**v0.5.0** — 150 tests pass, 12 frontend pages, 35 event types.

New in v0.5.0:
- Cron scheduler (ScheduleRepo, background tick, CRUD API + UI)
- Usage analytics (AnalyticsRepo, daily costs, agent breakdown, P90, projected monthly, dashboard)
- Loop detection (sliding-window hash detector, exit gate config)
- Quality gate + exit gate middleware variants
- Session HTML export
- 8 new ForgeEvent variants (schedule lifecycle, exit/quality gates)

Previous: v0.4.0 (Waves 1-4, 13 agents, middleware chain, memory, hooks, sub-agents, 10 pages)

## Don't

- Don't update files in `00-08/` directories (frozen reference from Feb 2026)
- Don't update files in `docs/planning/` (archived)
- Don't treat the old 305-feature roadmap as current — use MASTER_TASK_LIST.md
- Don't add features beyond what's in the current sprint
