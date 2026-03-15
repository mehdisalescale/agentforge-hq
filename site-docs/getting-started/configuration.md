# Configuration

All configuration is via environment variables. No config files needed.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `FORGE_DB_PATH` | `~/.agentforge/forge.db` | SQLite database path |
| `FORGE_PORT` | `4173` | Server port |
| `FORGE_HOST` | `127.0.0.1` | Bind address |
| `FORGE_CLI_COMMAND` | `claude` | CLI executable to spawn |
| `FORGE_CORS_ORIGIN` | `*` | CORS allowed origin |
| `FORGE_RATE_LIMIT_MAX` | `10` | Token bucket size for run endpoint |
| `FORGE_RATE_LIMIT_REFILL_MS` | `1000` | Refill interval (ms) |
| `FORGE_BUDGET_WARN` | *(none)* | Warning threshold (USD) |
| `FORGE_BUDGET_LIMIT` | *(none)* | Hard limit (USD) |

## Examples

```bash
# Custom port and database
FORGE_PORT=8080 FORGE_DB_PATH=/data/forge.db ./forge

# With budget controls
FORGE_BUDGET_WARN=50 FORGE_BUDGET_LIMIT=100 ./forge

# Use a different CLI
FORGE_CLI_COMMAND=/usr/local/bin/claude ./forge
```

## Health Check

AgentForge checks for the CLI at startup. If `claude` (or your configured `FORGE_CLI_COMMAND`) isn't found, a warning banner appears in the UI.

Check health programmatically:

```bash
curl http://127.0.0.1:4173/api/v1/health
```

```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_secs": 42,
  "cli_available": true,
  "cli_command": "claude"
}
```
