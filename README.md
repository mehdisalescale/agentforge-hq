# Claude Forge

Multi-agent Claude Code orchestrator. Rust + Svelte, single binary.

## What it does

- Spawn Claude Code agents and run prompts from the UI or API
- Stream agent output in real time over WebSocket
- Manage sessions and persist events to SQLite
- Export session transcripts (JSON or Markdown)

## Quick start

```bash
# Download from GitHub Releases (e.g. forge-macos-arm64), then:
chmod +x forge-macos-arm64   # needed after browser download
./forge-macos-arm64
# Open http://127.0.0.1:4173
```

## Build from source

```bash
cd frontend && pnpm install && pnpm build && cd ..
cargo build --release
./target/release/forge
```

## Data and authentication

- **Database:** The binary does **not** include a database. On first run, Forge creates an empty SQLite file at `FORGE_DB_PATH` (default `~/.claude-forge/forge.db`). So a fresh download always starts with no agents or sessions.
- **Claude Code:** Runs spawn the `claude` CLI (see `FORGE_CLI_COMMAND`). If `claude` is not installed or not authenticated, the run will fail: the session is marked "failed" and the error appears in the UI. Authenticate once with `claude` (e.g. in a terminal) before using Forge.

## Configuration

| Var | Default | Description |
|-----|---------|-------------|
| FORGE_DB_PATH | ~/.claude-forge/forge.db | SQLite database |
| FORGE_PORT | 4173 | Server port |
| FORGE_HOST | 127.0.0.1 | Bind address |
| FORGE_CORS_ORIGIN | * | CORS allowed origin |
| FORGE_CLI_COMMAND | claude | CLI executable |
| FORGE_RATE_LIMIT_MAX | 10 | Max tokens for run endpoint (token bucket) |
| FORGE_RATE_LIMIT_REFILL_MS | 1000 | Refill interval in ms (1 token per interval) |
| FORGE_BUDGET_WARN | (none) | Emit BudgetWarning event when session cost (USD) reaches this |
| FORGE_BUDGET_LIMIT | (none) | Emit BudgetExceeded event when session cost (USD) reaches this |

## Architecture

```
forge-app        -- binary: DB, API server, embedded frontend
forge-api        -- HTTP + WebSocket, routes, AppState
forge-process    -- spawn Claude CLI, stream parsing
forge-agent      -- agent presets, run request types
forge-db         -- SQLite, migrations, AgentRepo, SessionRepo, EventRepo, BatchWriter
forge-core       -- EventBus, shared types
forge-safety     -- circuit breaker, rate limiter (Phase 2)
forge-mcp        -- MCP server (Phase 2)
```

## License

Placeholder — check with maintainer.
