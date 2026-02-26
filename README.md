# Claude Forge

Multi-agent Claude Code orchestrator. Rust + Svelte, single binary.

## What it does

- Spawn Claude Code agents and run prompts from the UI or API
- Stream agent output in real time over WebSocket
- Manage sessions and persist events to SQLite
- Export session transcripts (JSON or Markdown)

## Quick start

```bash
# Download from GitHub Releases (or build from source)
./forge
# Open http://127.0.0.1:4173
```

## Build from source

```bash
cd frontend && pnpm install && pnpm build && cd ..
cargo build --release
./target/release/forge
```

## Configuration

| Var | Default | Description |
|-----|---------|-------------|
| FORGE_DB_PATH | ~/.claude-forge/forge.db | SQLite database |
| FORGE_PORT | 4173 | Server port |
| FORGE_HOST | 127.0.0.1 | Bind address |
| FORGE_CORS_ORIGIN | * | CORS allowed origin |
| FORGE_CLI_COMMAND | claude | CLI executable |

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
