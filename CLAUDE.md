# Claude Forge — Project Context

> For AI agents and humans starting a new session.

## What This Is

Multi-agent Claude Code orchestrator. Rust/Axum backend + Svelte 5 frontend, shipped as a single binary. The only Rust-native tool in this space.

## Tech Stack

- **Backend:** Rust, Axum, SQLite (WAL mode, rusqlite), tokio
- **Frontend:** SvelteKit 5, adapter-static, TailwindCSS 4, embedded via rust-embed
- **MCP Server:** rmcp v0.17 (official Rust MCP SDK), stdio transport
- **Safety:** Circuit breaker (3-state FSM), rate limiter (token bucket), CostTracker (budget warn/limit)

## Workspace Crates (8)

```
forge-app          binary: DB setup, API server, embedded frontend, graceful shutdown
├── forge-api      Axum HTTP + WebSocket, routes, CORS, TraceLayer, rust-embed SPA
├── forge-process  spawn Claude CLI, stream-json parsing, process lifecycle
├── forge-agent    agent model, 9 presets, validation
├── forge-db       SQLite WAL, migrations, AgentRepo, SessionRepo, EventRepo, BatchWriter
├── forge-core     ForgeEvent (20 variants), EventBus broadcast, shared types
├── forge-safety   CircuitBreaker, RateLimiter, CostTracker
└── forge-mcp-bin  MCP stdio server (rmcp, 10 tools)
```

## Build & Test

```bash
# Frontend (must build first — rust-embed needs frontend/build/)
cd frontend && pnpm install && pnpm build && cd ..

# Backend
cargo build --release
cargo test              # 59 tests, all should pass
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
| `FORGE_BUDGET_WARN` | *(none)* | Warning threshold (USD) |
| `FORGE_BUDGET_LIMIT` | *(none)* | Hard limit (USD) |

## Conventions

- **Zero warnings policy:** `cargo check` must produce zero warnings before committing
- **All tests pass:** `cargo test` must be green
- **Frontend state:** Svelte 5 runes (`$state`, `$derived`) — some pages still use `let` (legacy)
- **Error handling:** `ForgeError` hierarchy in forge-core, propagated via `ForgeResult<T>`
- **IDs:** Newtype wrappers (`AgentId`, `SessionId`) around `uuid::Uuid`
- **Events:** All state changes emit `ForgeEvent` variants through `EventBus` (broadcast channel)
- **Persistence:** `BatchWriter` batches events (50 or 2s flush) in transactions

## Active Docs (read these)

| File | What |
|------|------|
| `NORTH_STAR.md` | Vision, current state, sprint plan |
| `MASTER_TASK_LIST.md` | Sprint tasks with What/Where/How/Verify |
| `docs/ENHANCEMENT_PROPOSAL.md` | Focused 3-sprint plan (this session's output) |
| `docs/FORGE_AUDIT_2026_03_02.md` | Full audit: per-crate grades, gap analysis |
| `docs/DOC_INDEX.md` | What's current vs archived |

## Current Phase

Sprint 1 (v0.2.0) is nearly complete. Bug fixes (F1-F3), CostTracker (B2), and MCP rewrite (B1) are done. Remaining: CLAUDE.md (this file) and doc consolidation.

Next: Sprint 2 (v0.3.0) — git worktree isolation for multi-agent safety.

## Don't

- Don't update files in `00-08/` directories (frozen reference from Feb 2026)
- Don't update files in `docs/planning/` (archived)
- Don't treat the old 305-feature roadmap as current — use MASTER_TASK_LIST.md
- Don't add features beyond what's in the current sprint
