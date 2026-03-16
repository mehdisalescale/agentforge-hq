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
| GET | `/agents/stats` | Bulk stats for all agents |
| GET | `/agents/:id` | Get agent by ID |
| PUT | `/agents/:id` | Update agent |
| DELETE | `/agents/:id` | Delete agent |
| GET | `/agents/:id/stats` | Get agent run stats (count, cost, last run) |

## Sessions

| Method | Path | Description |
|--------|------|-------------|
| GET | `/sessions` | List all sessions |
| POST | `/sessions` | Create session |
| GET | `/sessions/:id` | Get session by ID |
| DELETE | `/sessions/:id` | Delete session |
| GET | `/sessions/:id/events` | Get session events |
| GET | `/sessions/:id/export` | Export session (JSON or Markdown) |

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
| GET | `/companies/:id` | Get company by ID |
| PATCH | `/companies/:id` | Update company (name, mission, budget_limit) |
| DELETE | `/companies/:id` | Delete company |

## Departments

| Method | Path | Description |
|--------|------|-------------|
| GET | `/departments?company_id=...` | List departments by company |
| POST | `/departments` | Create department |
| GET | `/departments/:id` | Get department by ID |
| PATCH | `/departments/:id` | Update department (name, description) |
| DELETE | `/departments/:id` | Delete department |

## Org Chart

| Method | Path | Description |
|--------|------|-------------|
| GET | `/org-positions?company_id=...` | List positions by company |
| POST | `/org-positions` | Create position |
| GET | `/org-chart?company_id=...` | Full org chart tree |

## Personas

| Method | Path | Description |
|--------|------|-------------|
| GET | `/personas` | List personas (filter: `division_slug`, `q`) |
| GET | `/personas/divisions` | List all persona divisions |
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

## Skills

| Method | Path | Description |
|--------|------|-------------|
| GET | `/skills` | List loaded skills |
| GET | `/skills/:id` | Get skill by ID |

## Workflows

| Method | Path | Description |
|--------|------|-------------|
| GET | `/workflows` | List workflows |
| POST | `/workflows` | Create workflow |
| GET | `/workflows/:id` | Get workflow by ID |
| PUT | `/workflows/:id` | Update workflow |
| DELETE | `/workflows/:id` | Delete workflow |
| POST | `/workflows/:id/run` | Run a workflow |

## Memory

| Method | Path | Description |
|--------|------|-------------|
| GET | `/memory` | List memories (params: `limit`, `offset`, `q`) |
| POST | `/memory` | Create memory |
| GET | `/memory/:id` | Get memory by ID |
| PUT | `/memory/:id` | Update memory |
| DELETE | `/memory/:id` | Delete memory |

## Hooks

| Method | Path | Description |
|--------|------|-------------|
| GET | `/hooks` | List hook configurations |
| POST | `/hooks` | Create hook |
| GET | `/hooks/:id` | Get hook by ID |
| PUT | `/hooks/:id` | Update hook |
| DELETE | `/hooks/:id` | Delete hook |

### HookReceiver (Claude Code event capture)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/hooks/pre-tool` | Claude Code PreToolUse hook |
| POST | `/hooks/post-tool` | Claude Code PostToolUse hook |
| POST | `/hooks/stop` | Claude Code Stop hook |

## Schedules

| Method | Path | Description |
|--------|------|-------------|
| GET | `/schedules` | List schedules |
| POST | `/schedules` | Create schedule |
| GET | `/schedules/:id` | Get schedule by ID |
| PUT | `/schedules/:id` | Update schedule |
| DELETE | `/schedules/:id` | Delete schedule |

## Analytics

| Method | Path | Description |
|--------|------|-------------|
| GET | `/analytics/usage?start=...&end=...&company_id=...` | Usage report |

## Settings

| Method | Path | Description |
|--------|------|-------------|
| GET | `/settings` | Runtime configuration |

## Backends

| Method | Path | Description |
|--------|------|-------------|
| GET | `/backends` | List available backends and capabilities |
| GET | `/backends/health` | Health check all backends |

## WebSocket

| Path | Description |
|------|-------------|
| `/ws` | Real-time event stream (ForgeEvents) |
