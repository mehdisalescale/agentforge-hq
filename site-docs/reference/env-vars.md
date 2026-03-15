# Environment Variables

All AgentForge configuration is via environment variables.

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `FORGE_HOST` | `127.0.0.1` | Bind address |
| `FORGE_PORT` | `4173` | Server port |
| `FORGE_CORS_ORIGIN` | `*` | CORS allowed origin |

## Database

| Variable | Default | Description |
|----------|---------|-------------|
| `FORGE_DB_PATH` | `~/.agentforge/forge.db` | SQLite database path |

## Execution

| Variable | Default | Description |
|----------|---------|-------------|
| `FORGE_CLI_COMMAND` | `claude` | CLI executable to spawn for agent runs |

## Safety

| Variable | Default | Description |
|----------|---------|-------------|
| `FORGE_RATE_LIMIT_MAX` | `10` | Token bucket size for run endpoint |
| `FORGE_RATE_LIMIT_REFILL_MS` | `1000` | Refill interval in milliseconds |
| `FORGE_BUDGET_WARN` | *(none)* | Warning threshold in USD |
| `FORGE_BUDGET_LIMIT` | *(none)* | Hard limit in USD — stops agent |

## MCP Server

| Variable | Default | Description |
|----------|---------|-------------|
| `FORGE_DB_PATH` | `~/.agentforge/forge.db` | Same DB as main server |

The MCP server (`forge-mcp`) reads the same database as the main `forge` binary. Set `FORGE_DB_PATH` consistently across both.
