# API Reference

Base URL: `http://127.0.0.1:4173/api/v1`

## Health

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Server health, version, CLI availability |

## Agents

| Method | Path | Description |
|--------|------|-------------|
| GET | `/agents` | List all agents |
| POST | `/agents` | Create agent |
| GET | `/agents/:id` | Get agent by ID |
| PUT | `/agents/:id` | Update agent |
| DELETE | `/agents/:id` | Delete agent |
| GET | `/agents/:id/stats` | Get agent run stats (count, cost, last run) |
| GET | `/agents/stats` | Bulk stats for all agents |

## Sessions

| Method | Path | Description |
|--------|------|-------------|
| GET | `/sessions` | List all sessions |
| POST | `/sessions` | Create session |
| GET | `/sessions/:id` | Get session by ID |
| DELETE | `/sessions/:id` | Delete session |
| GET | `/sessions/:id/events` | Get session events |

## Run

| Method | Path | Description |
|--------|------|-------------|
| POST | `/run` | Start an agent run |

```json
{
  "agent_id": "uuid",
  "prompt": "Review auth module for vulnerabilities",
  "session_id": "uuid (optional, for resume)",
  "directory": "/path/to/project (optional)"
}
```

Returns `202 Accepted` with `session_id`.

## Companies

| Method | Path | Description |
|--------|------|-------------|
| GET | `/companies` | List companies |
| POST | `/companies` | Create company |

## Departments

| Method | Path | Description |
|--------|------|-------------|
| GET | `/departments?company_id=...` | List departments |
| POST | `/departments` | Create department |

## Org Chart

| Method | Path | Description |
|--------|------|-------------|
| GET | `/org-positions?company_id=...` | List positions |
| POST | `/org-positions` | Create position |
| GET | `/org-chart?company_id=...` | Full org chart tree |

## Personas

| Method | Path | Description |
|--------|------|-------------|
| GET | `/personas` | List all personas (112) |
| GET | `/personas/:id` | Get persona detail |
| POST | `/personas/:id` | Hire persona into company |

## Goals

| Method | Path | Description |
|--------|------|-------------|
| GET | `/goals?company_id=...` | List goals |
| POST | `/goals` | Create goal |
| PATCH | `/goals/:id/status` | Update goal status |

## Approvals

| Method | Path | Description |
|--------|------|-------------|
| GET | `/approvals?company_id=...&status=...` | List approvals |
| POST | `/approvals` | Create approval request |
| PATCH | `/approvals/:id` | Approve/reject |

## Analytics

| Method | Path | Description |
|--------|------|-------------|
| GET | `/analytics/usage?start=...&end=...` | Usage report |

## Settings

| Method | Path | Description |
|--------|------|-------------|
| GET | `/settings` | Runtime configuration |

## Skills

| Method | Path | Description |
|--------|------|-------------|
| GET | `/skills` | List loaded skills |

## HookReceiver

| Method | Path | Description |
|--------|------|-------------|
| POST | `/hooks/pre-tool` | Claude Code PreToolUse hook |
| POST | `/hooks/post-tool` | Claude Code PostToolUse hook |
| POST | `/hooks/stop` | Claude Code Stop hook |

## WebSocket

| Path | Description |
|------|-------------|
| `/ws` | Real-time event stream (ForgeEvents) |
