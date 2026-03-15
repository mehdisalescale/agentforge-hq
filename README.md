# AgentForge HQ

Self-improving AI workforce platform. Rust + Svelte 5, single binary.

**Repo:** [mehdisalescale/agentforge-hq](https://github.com/mehdisalescale/agentforge-hq)

## What it does

- Browse **100+ pre-built agent personas** across 11 divisions (engineering, design, marketing, product, testing, support, etc.)
- Arrange agents in **org charts** with companies, departments, reporting lines, and budgets
- **Hire personas** into companies — creates agents + org positions with one click
- Spawn Claude Code agents with 10 specialized presets + parallel sub-agent execution
- Stream agent output in real time over WebSocket with sub-agent progress tracking
- 8-middleware pipeline: rate limiting, circuit breaker, cost check, skill injection, persistence, spawn, exit gate, quality gate
- Cross-session memory: extract facts from transcripts, inject relevant context into new runs
- Git worktree isolation for multi-agent safety
- **Governance layer:** goals, approvals (explicit yes/no decisions)
- Cron scheduler, usage analytics (daily costs, P90, projected monthly), loop detection
- MCP server mode (stdio) with 19 tools via rmcp
- Safety controls: circuit breaker, rate limiter, budget enforcement, exit gates, quality gates

## Quick start

```bash
# Download from GitHub Releases (e.g. agentforge-macos-arm64), then:
chmod +x agentforge-macos-arm64
./agentforge-macos-arm64
# Open http://127.0.0.1:4173
```

## Build from source

```bash
cd frontend && pnpm install && pnpm build && cd ..
cargo build --release
./target/release/forge
```

## Prerequisites

- **Claude Code CLI**: Must be installed and authenticated. Run `claude` in a terminal first to set up.
- **Rust toolchain**: For building from source (`rustup`, stable channel).
- **Node.js 22 + pnpm**: For building the frontend.

## Data

- **Database**: Created automatically at `~/.agentforge/forge.db` (SQLite WAL mode). Fresh install starts empty.
- **No cloud**: Everything runs locally. No data leaves your machine (except Claude API calls made by the agents).

## Configuration

| Var | Default | Description |
|-----|---------|-------------|
| `FORGE_DB_PATH` | `~/.agentforge/forge.db` | SQLite database path |
| `FORGE_PORT` | `4173` | Server port |
| `FORGE_HOST` | `127.0.0.1` | Bind address |
| `FORGE_CORS_ORIGIN` | `*` | CORS allowed origin |
| `FORGE_CLI_COMMAND` | `claude` | CLI executable to spawn |
| `FORGE_RATE_LIMIT_MAX` | `10` | Token bucket size for run endpoint |
| `FORGE_RATE_LIMIT_REFILL_MS` | `1000` | Refill interval (1 token per interval) |
| `FORGE_BUDGET_WARN` | *(none)* | Emit warning when session cost (USD) reaches this |
| `FORGE_BUDGET_LIMIT` | *(none)* | Stop agent when session cost (USD) reaches this |

## Architecture

```
forge-app          binary: DB setup, API server, embedded frontend, graceful shutdown, cron scheduler
├── forge-api      Axum HTTP + WebSocket, 8-middleware chain, CORS, TraceLayer, rust-embed SPA
├── forge-process  spawn Claude CLI, stream-json parsing, ConcurrentRunner, LoopDetector
├── forge-agent    agent model, 10 presets (incl. Coordinator), validation
├── forge-db       SQLite WAL, 12 migrations, 16 repos, BatchWriter
├── forge-core     ForgeEvent (43 variants), EventBus broadcast, shared types
├── forge-safety   circuit breaker (3-state FSM), rate limiter (token bucket), CostTracker
├── forge-git      git worktree create/remove/list for multi-agent isolation
├── forge-org      company, department, org position models + org chart builder
├── forge-persona  100+ persona catalog, division taxonomy, hire flow
├── forge-governance  goals, approvals (governance layer)
├── forge-mcp      MCP protocol stubs
└── forge-mcp-bin  MCP stdio server (rmcp, 19 tools)
```

## Frontend (16+ pages)

| Page | Purpose |
|------|---------|
| Dashboard | Agent selector, prompt, WebSocket streaming, markdown rendering |
| Agents | Full CRUD, 10 presets, domain badges |
| Sessions | Two-pane layout, export (JSON/Markdown/HTML), cost display |
| Memory | CRUD, search, confidence bars, category badges |
| Hooks | CRUD, event type select, timing badges |
| Skills | Tag pills, category filter, expandable content |
| Workflows | Visual diagram, card layout |
| Schedules | Cron CRUD, preset dropdown, run count, last/next run |
| Analytics | Summary cards, bar chart (daily costs), agent breakdown, P90 |
| **Companies** | Create/manage companies (name, mission, budget) |
| **Org Chart** | Visualize org hierarchy per company |
| **Personas** | Browse 100+ personas, hire into company/org |
| **Goals** | Define and track per-company goals |
| **Approvals** | Governance: review and resolve approval requests |
| Settings | Config dashboard, health endpoint |

## Roadmap (9 Epics → v1.0.0)

| Epic | What | Status |
|------|------|--------|
| E1 | Persona Workforce Catalog | Baseline implemented |
| E2 | Dev Methodology Engine | Planning |
| E3 | Hexagonal Backend Architecture | Planning |
| E4 | Org Structure & Governance | Planning |
| E5 | Multi-Backend Execution (Hermes/OpenClaw) | Not started |
| E6 | Knowledge Base | Not started |
| E7 | Multi-Platform Messaging (16+) | Not started |
| E8 | Native Desktop Client | Not started |
| E9 | Production Hardening → v1.0.0 | Not started |

See `../docs/INDEX.md` for the full documentation index.

## License

Placeholder — check with maintainer.
