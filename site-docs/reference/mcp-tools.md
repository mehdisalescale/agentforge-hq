# MCP Tools Reference

AgentForge exposes 19 MCP tools. All accessible via stdio transport.

## Workforce (7 tools)

### agent_list
List all agents.

### agent_get
Get an agent by ID.

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | UUID of the agent |

### agent_create
Create a new agent.

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | string | Agent name |
| `model` | string? | Model identifier |
| `system_prompt` | string? | System prompt |
| `preset` | string? | Preset name (CodeWriter, Reviewer, etc.) |

### agent_update
Update an agent.

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | UUID of the agent |
| `name` | string? | New name |
| `model` | string? | New model |
| `system_prompt` | string? | New system prompt |

### agent_delete
Delete an agent by ID.

### forge_list_personas
List available AI personas from the catalog.

| Parameter | Type | Description |
|-----------|------|-------------|
| `division` | string? | Filter by division (engineering, security, etc.) |
| `search` | string? | Search by name or description |

### forge_hire_persona
Hire a persona into a company.

| Parameter | Type | Description |
|-----------|------|-------------|
| `persona_id` | string | UUID of the persona |
| `company_id` | string | UUID of the company |
| `department_id` | string? | UUID of the department |

## Sessions (4 tools)

### session_list
List all sessions.

### session_get
Get a session by ID.

### session_create
Create a new session for an agent.

| Parameter | Type | Description |
|-----------|------|-------------|
| `agent_id` | string | UUID of the agent |
| `directory` | string? | Working directory |
| `claude_session_id` | string? | Claude session ID for resume |

### session_export
Export a session with events.

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | UUID of the session |
| `format` | string? | "json" or "markdown" (default: json) |

## Governance (4 tools)

### forge_get_budget
Get budget status for a company.

| Parameter | Type | Description |
|-----------|------|-------------|
| `company_id` | string | UUID of the company |

Returns: company name, budget limit, used, remaining, status (ok/warning/exhausted).

### forge_request_approval
Request an approval from company governance.

| Parameter | Type | Description |
|-----------|------|-------------|
| `company_id` | string | UUID of the company |
| `approval_type` | string | budget_increase, run_authorization, deployment, other |
| `description` | string | What needs approval |

### forge_check_approval
Check status of an approval request.

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | UUID of the approval |

### forge_list_goals
List goals for a company.

| Parameter | Type | Description |
|-----------|------|-------------|
| `company_id` | string | UUID of the company |

## Intelligence (1 tool)

### forge_classify_task
Classify a prompt into a task type with recommended skills.

| Parameter | Type | Description |
|-----------|------|-------------|
| `prompt` | string | Task description to classify |

Returns: task type (BugFix, Feature, Refactor, Test, Review, etc.), recommended skills, confidence level.

## Observability (1 tool)

### forge_get_analytics
Get usage analytics.

| Parameter | Type | Description |
|-----------|------|-------------|
| `company_id` | string? | Filter by company |
| `start` | string? | Start date (YYYY-MM-DD) |
| `end` | string? | End date (YYYY-MM-DD) |

Returns: total cost, daily costs, agent breakdown, session stats, projected monthly cost.

### forge_get_session_events
Get all events for a session.

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | UUID of the session |
