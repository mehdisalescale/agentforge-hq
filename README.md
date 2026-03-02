# Claude Forge

Multi-agent Claude Code orchestrator. Rust + Svelte 5, single binary.

## What it does

- Spawn Claude Code agents with 9 specialized presets (CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer)
- Stream agent output in real time over WebSocket
- Manage sessions with status tracking and cost visibility
- Export session transcripts (JSON or Markdown)
- Safety controls: circuit breaker, rate limiter, budget enforcement

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
forge-app          binary: DB setup, API server, embedded frontend, graceful shutdown
├── forge-api      Axum HTTP + WebSocket, routes, CORS, TraceLayer, rust-embed SPA
├── forge-process  spawn Claude CLI, stream-json parsing, process lifecycle
├── forge-agent    agent model, 9 presets, validation
├── forge-db       SQLite WAL, migrations, AgentRepo, SessionRepo, EventRepo, BatchWriter
├── forge-core     ForgeEvent (20 variants), EventBus broadcast, shared types
├── forge-safety   circuit breaker (3-state FSM), rate limiter (token bucket)
├── forge-mcp      MCP protocol types
└── forge-mcp-bin  MCP stdio server (JSON-RPC)
```

## Status

- **v0.1.0**: Shipped — agent CRUD, process spawn, sessions, streaming, embedded UI
- **v0.2.0**: In progress — MCP server rewrite with rmcp, bug fixes
- **v0.3.0**: Planned — middleware chain, skill system, git worktree isolation
- **v0.4.0**: Planned — sub-agent parallel spawning, agent domains

See `NORTH_STAR.md` for full roadmap and `docs/FORGE_AUDIT_2026_03_02.md` for the latest audit.

## License

Placeholder — check with maintainer.
