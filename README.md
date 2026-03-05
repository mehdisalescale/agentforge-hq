# Claude Forge

Multi-agent Claude Code orchestrator. Rust + Svelte 5, single binary.

## What it does

- Spawn Claude Code agents with 10 specialized presets (CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer, Coordinator)
- Run multiple sub-agents in parallel with semaphore-limited concurrency
- Stream agent output in real time over WebSocket with sub-agent progress tracking
- 8-middleware pipeline: rate limiting, circuit breaker, cost check, skill injection, persistence, spawn, exit gate, quality gate
- Cross-session memory: extract facts from transcripts, inject relevant context into new runs
- Git worktree isolation for multi-agent safety
- Event hooks: run shell commands on any of 35 event types (pre/post)
- Cron scheduler: schedule recurring agent runs with cron expressions
- Usage analytics: daily costs, agent breakdown, session stats (P90), projected monthly cost
- Loop detection: sliding-window hash dedup catches stuck agents producing repetitive output
- Manage sessions with status tracking, cost visibility, and export (JSON/Markdown/HTML)
- 10 skill templates loaded from Markdown files with YAML frontmatter
- Safety controls: circuit breaker, rate limiter, budget enforcement, exit gates, quality gates
- MCP server mode (stdio) with 10 tools via rmcp

## Quick start

```bash
# Download from GitHub Releases (e.g. forge-macos-arm64), then:
chmod +x forge-macos-arm64
./forge-macos-arm64
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

- **Database**: Created automatically at `~/.claude-forge/forge.db` (SQLite WAL mode). Fresh install starts empty.
- **No cloud**: Everything runs locally. No data leaves your machine (except Claude API calls made by the agents).

## Configuration

| Var | Default | Description |
|-----|---------|-------------|
| `FORGE_DB_PATH` | `~/.claude-forge/forge.db` | SQLite database path |
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
├── forge-db       SQLite WAL, 5 migrations, 8 repos (Agent, Session, Event, Skill, Memory, Hook, Schedule, Analytics), BatchWriter
├── forge-core     ForgeEvent (35 variants), EventBus broadcast, shared types
├── forge-safety   circuit breaker (3-state FSM), rate limiter (token bucket), CostTracker
├── forge-git      git worktree create/remove/list for multi-agent isolation
└── forge-mcp-bin  MCP stdio server (rmcp, 10 tools)
```

## Status

- **v0.1.0**: Agent CRUD, process spawn, sessions, streaming, embedded UI
- **v0.2.0**: MCP server rewrite with rmcp, bug fixes, doc consolidation
- **v0.4.0**: 6-middleware chain, skill system, memory extract/inject, git worktree isolation, sub-agent parallel spawning, Coordinator preset, full frontend (10 pages), 118 tests
- **v0.5.0**: Current — cron scheduler, usage analytics (daily/agent/P90/projected), loop detection, quality/exit gates, session HTML export, 12 frontend pages, 150 tests

See `NORTH_STAR.md` for full roadmap and `docs/RESEARCH_FINDINGS_2026_03_05.md` for patterns from 67 community repos.

## Reference Hub

This project was built on research from **[claude-parent](https://github.com/mbaneshi/claude-parent)** — a reference hub that maps 61 Claude Code community repos into mini-books with extracted patterns.

| Resource | What it provides |
|----------|-----------------|
| [Capability Map & Forge Roadmap](https://github.com/mbaneshi/claude-parent/blob/main/docs/CLAUDE_CODE_CAPABILITY_MAP_AND_FORGE_ROADMAP.md) | Claude Code capabilities mapped to Forge features |
| [AI Documentation Landscape (2026)](https://github.com/mbaneshi/claude-parent/blob/main/docs/ai-documentation-landscape.md) | AGENTS.md standard, AI context files, best practices |
| [Reference Repos Overview](https://github.com/mbaneshi/claude-parent/blob/main/docs/reference/index.md) | 61 repos across 13 categories |
| [Borrowed Ideas](docs/BORROWED_IDEAS.md) | Patterns adopted from DeerFlow, Claude-Flow, and community repos |

Forge was originally developed inside claude-parent and split into its own repository on 2026-03-02.

## License

Placeholder — check with maintainer.
