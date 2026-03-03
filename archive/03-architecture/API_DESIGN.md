# Claude Forge -- API Design

> REST + WebSocket API specification for the multi-agent Claude Code orchestrator.
> Base URL: `http://127.0.0.1:4173` | WebSocket: `ws://127.0.0.1:4173/ws`

**Phase 0 canonical:** Implemented API uses versioned prefix `/api/v1/` (e.g. `GET /api/v1/health`, `GET/POST/PUT/DELETE /api/v1/agents`, `GET /api/v1/ws` for WebSocket). This document uses unversioned `/api/*` in examples; treat `/api/v1/` as the current standard.

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Versioning Strategy](#versioning-strategy)
3. [Authentication Model](#authentication-model)
4. [Error Response Format](#error-response-format)
5. [Endpoint Catalog](#endpoint-catalog)
   - [Agent Endpoints](#agent-endpoints)
   - [Prompt and Execution](#prompt-and-execution)
   - [Session Browser](#session-browser)
   - [Preset and Skill Endpoints](#preset-and-skill-endpoints)
   - [Workflow Endpoints](#workflow-endpoints)
   - [Git Endpoints](#git-endpoints)
   - [System Endpoints](#system-endpoints)
6. [WebSocket Protocol](#websocket-protocol)
7. [SSE Alternative](#sse-alternative)
8. [Pagination and Filtering](#pagination-and-filtering)
9. [Rate Limiting](#rate-limiting)
10. [Request and Response Examples](#request-and-response-examples)

---

## Design Principles

| Principle | Implementation |
|-----------|---------------|
| REST-ish, not REST-pure | Pragmatic use of HTTP methods. Some endpoints use POST for actions that are not pure creates (e.g., `/prompt`, `/start`) |
| JSON everywhere | All request and response bodies are `application/json` except exports (which offer Markdown) |
| Consistent error format | Every error returns `{ "error": "message" }` with appropriate HTTP status code |
| WebSocket for streaming | Real-time events flow over a single WebSocket connection with subscription filtering |
| No pagination by default | List endpoints return all items. Pagination available via query params for large collections (events) |
| Idempotent where possible | PUT and DELETE are idempotent. POST creates are not (each call creates a new resource) |

---

## Versioning Strategy

### Current: No Versioning (v0)

The API is currently unversioned. All endpoints live at `/api/*`. This is acceptable for a local-only tool where the binary ships both client and server.

### Future: URL Prefix Versioning

When breaking changes are needed:

```
/api/v1/agents          -- stable
/api/v2/agents          -- new format
/api/agents             -- alias for latest stable version
```

**Breaking change policy:**
- New fields added to responses: NOT breaking (clients should ignore unknown fields)
- Fields removed from responses: BREAKING
- New required fields in requests: BREAKING
- New optional fields in requests: NOT breaking
- Changed field types: BREAKING
- Changed URL paths: BREAKING

**Compatibility header (planned):**
```
X-Forge-API-Version: 1
```

---

## Authentication Model

### v1: No Authentication

Forge listens on `127.0.0.1` (loopback only) by default. No authentication is required. This is appropriate because:
- Forge is a local developer tool, not a cloud service
- Binding to loopback prevents network access
- The Claude CLI processes inherit the user's credentials

### v2: API Key Authentication (Planned)

When Forge is exposed on a network (e.g., team server, remote access):

```
Authorization: Bearer forge_sk_abc123def456
```

**Key management:**
- Keys stored in `config` table (hashed with argon2)
- Generated via `claude-forge keygen` CLI command
- Revoked via `claude-forge revoke <key_prefix>`
- No user accounts -- keys grant full access

### v3: Multi-User (Planned)

If Forge ever supports multi-user mode:
- OAuth2 with PKCE flow
- JWT tokens with short expiry
- Role-based access (admin, operator, viewer)

---

## Error Response Format

All errors return a JSON body with a single `error` field:

```json
{
    "error": "human-readable error message"
}
```

### HTTP Status Codes Used

| Status | Meaning | Example |
|--------|---------|---------|
| 200 | Success | GET /api/agents |
| 201 | Created | POST /api/agents |
| 202 | Accepted (async) | POST /api/agents/:id/prompt |
| 204 | No Content | DELETE /api/agents/:id, PUT /api/agents/:id/claude-md |
| 400 | Bad Request | Invalid JSON, cannot update running agent |
| 404 | Not Found | Agent ID does not exist |
| 422 | Unprocessable Entity | Schema validation failure (planned) |
| 429 | Too Many Requests | Rate limited (planned) |
| 500 | Internal Server Error | Process spawn failure, DB error |

### Error Type Mapping (from ForgeError)

| ForgeError Variant | HTTP Status | Message Pattern |
|-------------------|-------------|----------------|
| `AgentNotFound(uuid)` | 404 | "agent not found: {uuid}" |
| `AgentNotRunning(uuid)` | 400 | "agent not running: {uuid}" |
| `ProcessSpawnFailed(msg)` | 500 | "failed to spawn process: {msg}" |
| `ConfigError(msg)` | 400 | "config error: {msg}" |
| `Internal(msg)` | 500 | "internal error: {msg}" |
| `WorkflowNotFound(id)` | 404 | "workflow not found: {id}" (planned) |
| `BudgetExceeded(uuid, amt)` | 429 | "budget exceeded for agent {uuid}: ${amt}" (planned) |
| `RateLimited(uuid)` | 429 | "rate limited: retry after {n}ms" (planned) |

---

## Endpoint Catalog

### Agent Endpoints

#### POST /api/agents

Create a new agent.

**Request:**
```json
{
    "name": "Planner",
    "model": "opus",
    "system_prompt": "You are a software architect...",
    "append_system_prompt": "Additional context for this project...",
    "permission_mode": "plan",
    "allowed_tools": ["Read", "Glob", "Grep"],
    "disallowed_tools": ["Bash"],
    "mcp_servers": {
        "gitnexus": {
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "gitnexus@latest", "mcp"]
        }
    },
    "hooks": {
        "PreToolUse": [
            {
                "matcher": "Bash",
                "hooks": [
                    {
                        "type": "command",
                        "command": "echo 'Bash tool invoked'"
                    }
                ]
            }
        ]
    },
    "max_budget_usd": 5.0,
    "max_turns": 50,
    "working_directory": "/Users/bm/project",
    "additional_dirs": ["/Users/bm/shared-lib"],
    "chrome_enabled": false,
    "use_max": false,
    "use_gitnexus": true,
    "worktree": "feature-branch",
    "json_schema": null,
    "subagents": null
}
```

All fields except `name` are optional. Defaults:
- `model`: "sonnet"
- `permission_mode`: "default"
- `working_directory`: process current directory
- All arrays: empty
- All booleans: false

**Response: 201 Created**
```json
{
    "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "name": "Planner",
    "status": "idle",
    "model": "opus",
    "session_id": null,
    "usage": {
        "input_tokens": 0,
        "output_tokens": 0,
        "cache_creation_tokens": 0,
        "cache_read_tokens": 0,
        "estimated_cost_usd": 0.0
    },
    "created_at": "2026-02-25T10:30:00Z"
}
```

---

#### GET /api/agents

List all agents.

**Response: 200 OK**
```json
[
    {
        "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
        "name": "Planner",
        "status": "idle",
        "model": "opus",
        "session_id": "sess_abc123",
        "usage": {
            "input_tokens": 15420,
            "output_tokens": 3210,
            "cache_creation_tokens": 5000,
            "cache_read_tokens": 12000,
            "estimated_cost_usd": 0.1234
        },
        "created_at": "2026-02-25T10:30:00Z"
    }
]
```

---

#### GET /api/agents/{id}

Get agent detail including recent events and full configuration.

**Response: 200 OK**
```json
{
    "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "name": "Planner",
    "status": "running",
    "model": "opus",
    "session_id": "sess_abc123",
    "usage": { "..." : "..." },
    "created_at": "2026-02-25T10:30:00Z",
    "recent_events": [
        {
            "event": { "type": "system", "session_id": "sess_abc123" },
            "event_type": "system",
            "timestamp": "2026-02-25T10:31:00Z"
        },
        {
            "event": { "type": "assistant", "message": { "..." : "..." } },
            "event_type": "assistant",
            "timestamp": "2026-02-25T10:31:05Z"
        }
    ],
    "config": {
        "name": "Planner",
        "model": "opus",
        "system_prompt": "You are a software architect...",
        "..." : "..."
    }
}
```

**Notes:** `recent_events` contains the last 100 events. For full event history, use the events endpoint (planned).

---

#### PATCH /api/agents/{id}

Update agent configuration. Only provided fields are changed. Uses `Option<Option<T>>` pattern: outer `Some` means "update this field", inner `None` means "set to null".

**Request:**
```json
{
    "name": "Senior Planner",
    "model": "opus",
    "max_budget_usd": 10.0,
    "system_prompt": null
}
```

Setting `"system_prompt": null` clears the system prompt. Omitting a field leaves it unchanged.

**Response: 200 OK** (same format as AgentResponse)

**Error: 400** if agent is running.

---

#### DELETE /api/agents/{id}

Delete an agent and all its events.

**Response: 204 No Content**

**Error: 404** if agent does not exist.

---

#### GET /api/agents/{id}/export?format=json|markdown

Export agent data. Format is specified via query parameter.

**Response (format=json): 200 OK**
```json
{
    "agent": {
        "id": "f47ac10b-...",
        "name": "Planner",
        "model": "opus",
        "status": "idle",
        "session_id": "sess_abc123",
        "created_at": "2026-02-25T10:30:00Z"
    },
    "config": { "..." : "..." },
    "usage": { "..." : "..." },
    "events": [
        { "event": {...}, "event_type": "system", "timestamp": "..." },
        { "event": {...}, "event_type": "assistant", "timestamp": "..." }
    ]
}
```

Content-Disposition: `attachment; filename="Planner.json"`

**Response (format=markdown): 200 OK**
```markdown
# Agent: Planner

- **Model**: opus
- **Status**: idle
- **Created**: 2026-02-25T10:30:00Z
- **Cost**: $0.1234
- **Tokens**: 15420 in / 3210 out

---

## Conversation

**User**: Please analyze the auth module...

**Assistant**: I'll analyze the authentication module...
```

Content-Disposition: `attachment; filename="Planner.md"`

---

#### GET /api/agents/{id}/claude-md

Read the CLAUDE.md file from the agent's working directory.

**Response: 200 OK**
```json
{
    "content": "# Project Rules\n\nThis project uses...",
    "path": "/Users/bm/project/CLAUDE.md",
    "exists": true
}
```

If no CLAUDE.md exists, returns `exists: false` with empty content and the default path.

---

#### PUT /api/agents/{id}/claude-md

Write CLAUDE.md to the agent's working directory.

**Request:**
```json
{
    "content": "# Project Rules\n\nUpdated rules...",
    "path": "/Users/bm/project/.claude/CLAUDE.md"
}
```

`path` is optional. Defaults to `{working_directory}/CLAUDE.md`.

**Response: 204 No Content**

---

### Prompt and Execution

#### POST /api/agents/{id}/prompt

Send a prompt to an agent. This spawns a Claude Code child process (or resumes an existing session).

**Request:**
```json
{
    "text": "Please analyze the authentication module and identify security issues."
}
```

**Response: 202 Accepted**

The response is immediate. Results stream via WebSocket. The agent's status changes to "running" and events flow through the event bus.

**Error: 400** if agent is already running.
**Error: 500** if Claude CLI process fails to spawn.

---

### Session Browser

#### GET /api/sessions?path=/Users/bm/project

List Claude Code sessions, optionally filtered by working directory.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | No | Filter by working directory path. If omitted, returns all sessions |

**Response: 200 OK**
```json
[
    {
        "session_id": "abc123-def456-789",
        "file_path": "/Users/bm/.claude/projects/-Users-bm-project/abc123-def456-789.jsonl",
        "created_at": "2026-02-24T14:00:00Z",
        "last_timestamp": "2026-02-24T14:35:00Z",
        "first_user_message": "Please refactor the database module to use connection pooling...",
        "message_count": 42,
        "project_dir": "/Users/bm/project"
    }
]
```

Sessions are sorted by `last_timestamp` descending (most recent first).

---

#### GET /api/sessions/projects

List all project directories that have Claude Code sessions.

**Response: 200 OK**
```json
[
    {
        "name": "-Users-bm-project",
        "original_path": "/Users/bm/project",
        "dir_path": "/Users/bm/.claude/projects/-Users-bm-project"
    }
]
```

---

### Preset and Skill Endpoints

#### GET /api/presets

List all available agent presets.

**Response: 200 OK**
```json
[
    {
        "id": "planner",
        "name": "Planner / Architect",
        "description": "Plans before coding. Task breakdown, architectural decisions, dependency analysis.",
        "icon": "blueprint",
        "model": "opus",
        "system_prompt": "You are a software architect...",
        "permission_mode": "plan",
        "allowed_tools": [],
        "max_turns": null,
        "use_gitnexus": true
    },
    {
        "id": "reviewer",
        "name": "Code Reviewer",
        "description": "Objective technical assessment. Finds bugs, anti-patterns, and security issues.",
        "icon": "magnifier",
        "model": "opus",
        "..." : "..."
    }
]
```

---

#### GET /api/skills (planned)

List available skills.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `category` | string | No | Filter by category (coding, review, testing, docs, security, workflow) |

**Response: 200 OK**
```json
[
    {
        "id": "explain-code",
        "name": "Explain Code",
        "description": "Explain what a code block does in plain English",
        "category": "coding",
        "source": "builtin",
        "arguments": {
            "type": "object",
            "properties": {
                "file": { "type": "string", "description": "File path to explain" },
                "lines": { "type": "string", "description": "Line range (e.g., 10-50)" }
            }
        }
    }
]
```

---

#### POST /api/skills/{id}/invoke (planned)

Invoke a skill for a specific agent.

**Request:**
```json
{
    "agent_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "arguments": {
        "file": "src/auth.rs",
        "lines": "10-50"
    }
}
```

**Response: 202 Accepted** (triggers a prompt to the agent with the skill's template filled in)

---

### Workflow Endpoints (planned)

#### POST /api/workflows

Create a workflow definition.

**Request:**
```json
{
    "name": "Code Review Pipeline",
    "description": "Automated code review with planning, review, and summary",
    "definition": {
        "steps": [
            {
                "id": "plan",
                "name": "Architecture Analysis",
                "preset": "planner",
                "prompt": "Analyze the architecture of {{target_dir}}",
                "depends_on": []
            },
            {
                "id": "review",
                "name": "Code Review",
                "preset": "reviewer",
                "prompt": "Review the code in {{target_dir}} focusing on: {{focus_areas}}",
                "depends_on": ["plan"]
            },
            {
                "id": "security",
                "name": "Security Audit",
                "preset": "security",
                "prompt": "Audit {{target_dir}} for security vulnerabilities",
                "depends_on": ["plan"]
            },
            {
                "id": "summary",
                "name": "Summary",
                "preset": "planner",
                "prompt": "Summarize findings from the review and security audit",
                "depends_on": ["review", "security"]
            }
        ],
        "variables": {
            "target_dir": { "type": "string", "required": true },
            "focus_areas": { "type": "string", "default": "correctness, performance, maintainability" }
        }
    }
}
```

**Response: 201 Created**
```json
{
    "id": "wf-abc123",
    "name": "Code Review Pipeline",
    "step_count": 4,
    "created_at": "2026-02-25T10:30:00Z"
}
```

---

#### POST /api/workflows/{id}/start

Start executing a workflow.

**Request:**
```json
{
    "variables": {
        "target_dir": "/Users/bm/project/src",
        "focus_areas": "error handling, auth bypass"
    },
    "working_directory": "/Users/bm/project"
}
```

**Response: 202 Accepted**
```json
{
    "run_id": "run-xyz789",
    "workflow_id": "wf-abc123",
    "status": "running",
    "started_at": "2026-02-25T10:35:00Z"
}
```

---

#### GET /api/workflows/{id}/runs/{run_id}

Get workflow run details.

**Response: 200 OK**
```json
{
    "id": "run-xyz789",
    "workflow_id": "wf-abc123",
    "status": "running",
    "started_at": "2026-02-25T10:35:00Z",
    "completed_at": null,
    "total_cost_usd": 0.0567,
    "steps": [
        {
            "id": "plan",
            "name": "Architecture Analysis",
            "status": "completed",
            "agent_id": "agent-111",
            "cost_usd": 0.0234,
            "started_at": "2026-02-25T10:35:00Z",
            "completed_at": "2026-02-25T10:36:30Z"
        },
        {
            "id": "review",
            "name": "Code Review",
            "status": "running",
            "agent_id": "agent-222",
            "cost_usd": 0.0333,
            "started_at": "2026-02-25T10:36:31Z",
            "completed_at": null
        },
        {
            "id": "security",
            "name": "Security Audit",
            "status": "running",
            "agent_id": "agent-333",
            "cost_usd": 0.0,
            "started_at": "2026-02-25T10:36:31Z",
            "completed_at": null
        },
        {
            "id": "summary",
            "name": "Summary",
            "status": "pending",
            "agent_id": null,
            "cost_usd": 0.0,
            "started_at": null,
            "completed_at": null
        }
    ]
}
```

---

### Git Endpoints (planned)

#### GET /api/agents/{id}/git/status

Get git status for the agent's working directory.

**Response: 200 OK**
```json
{
    "branch": "main",
    "remote": "origin/main",
    "ahead": 2,
    "behind": 0,
    "staged": [
        { "path": "src/auth.rs", "status": "modified" }
    ],
    "unstaged": [
        { "path": "src/db.rs", "status": "modified" },
        { "path": "src/new_file.rs", "status": "untracked" }
    ],
    "is_clean": false
}
```

---

#### GET /api/agents/{id}/git/diff?base=HEAD~1

Get git diff.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `base` | string | No | Base ref for diff. Default: staged diff |
| `file` | string | No | Filter to specific file |

**Response: 200 OK**
```json
{
    "files_changed": 3,
    "insertions": 42,
    "deletions": 15,
    "hunks": [
        {
            "file": "src/auth.rs",
            "old_start": 10,
            "old_count": 5,
            "new_start": 10,
            "new_count": 8,
            "content": "@@ -10,5 +10,8 @@\n-old line\n+new line\n..."
        }
    ]
}
```

---

#### POST /api/agents/{id}/git/worktrees (planned)

Create a git worktree for an agent.

**Request:**
```json
{
    "name": "feature-auth-refactor",
    "branch": "auth-refactor",
    "base": "main"
}
```

**Response: 201 Created**
```json
{
    "name": "feature-auth-refactor",
    "path": "/Users/bm/project/.claude/worktrees/feature-auth-refactor",
    "branch": "auth-refactor"
}
```

---

### System Endpoints

#### GET /api/cost

Get cost summary across all agents.

**Response: 200 OK**
```json
{
    "total_spent_usd": 1.2345,
    "total_input_tokens": 150000,
    "total_output_tokens": 35000,
    "agents": {
        "f47ac10b-58cc-4372-a567-0e02b2c3d479": {
            "input_tokens": 15420,
            "output_tokens": 3210,
            "cache_creation_tokens": 5000,
            "cache_read_tokens": 12000,
            "estimated_cost_usd": 0.1234
        }
    }
}
```

---

#### GET /api/fs/browse?path=/Users/bm

Browse filesystem directories.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | No | Directory to browse. Default: current working directory |

**Response: 200 OK**
```json
{
    "current": "/Users/bm",
    "parent": "/Users",
    "entries": [
        { "name": "project", "path": "/Users/bm/project", "is_dir": true },
        { "name": "Documents", "path": "/Users/bm/Documents", "is_dir": true }
    ]
}
```

Only directories are returned. Hidden directories (starting with `.`) are excluded.

---

#### GET /api/config (planned)

Get current Forge configuration.

**Response: 200 OK**
```json
{
    "default_model": "sonnet",
    "global_max_budget_usd": null,
    "event_retention_days": 90,
    "port": 4173,
    "bind": "127.0.0.1"
}
```

---

#### PUT /api/config (planned)

Update Forge configuration.

**Request:**
```json
{
    "default_model": "opus",
    "global_max_budget_usd": 50.0
}
```

**Response: 200 OK**

---

#### GET /api/search?q=refactor+auth (planned)

Full-text search across all agent events.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query (FTS5 syntax) |
| `agent_id` | UUID | No | Filter to specific agent |
| `event_type` | string | No | Filter by event type |
| `limit` | integer | No | Max results (default 20, max 100) |
| `offset` | integer | No | Pagination offset |

**Response: 200 OK**
```json
{
    "total": 42,
    "results": [
        {
            "agent_id": "f47ac10b-...",
            "agent_name": "Planner",
            "event_type": "assistant",
            "snippet": "...I recommend **refactoring** the **auth** module to use...",
            "timestamp": "2026-02-25T10:31:05Z",
            "event_id": 12345
        }
    ]
}
```

---

## WebSocket Protocol

### Connection

Connect to `ws://127.0.0.1:4173/ws`. No authentication required for v1.

### Client Messages

All client messages are JSON with a `type` field.

#### Subscribe

Set a filter for which events to receive.

```json
{
    "type": "subscribe",
    "filter": "all"
}
```

```json
{
    "type": "subscribe",
    "filter": {
        "agent_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
    }
}
```

**Filter options:**
- `"all"` -- receive events from all agents
- `{ "agent_id": "uuid" }` -- receive events from one agent only

Default filter on connect: `"all"`.

#### Prompt

Send a prompt to an agent via WebSocket (alternative to POST /api/agents/:id/prompt).

```json
{
    "type": "prompt",
    "agent_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "text": "Please analyze the auth module."
}
```

#### Ping

Keepalive ping.

```json
{
    "type": "ping"
}
```

### Server Messages

#### Event

A Claude Code event for a specific agent.

```json
{
    "type": "event",
    "agent_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "event_type": "assistant",
    "event": {
        "type": "assistant",
        "message": {
            "id": "msg_abc123",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "I'll analyze the authentication module. Let me start by reading the relevant files."
                }
            ],
            "usage": {
                "input_tokens": 5140,
                "output_tokens": 1070,
                "cache_creation_input_tokens": 0,
                "cache_read_input_tokens": 4800
            }
        }
    }
}
```

#### Agent Status

Status change notification.

```json
{
    "type": "agent_status",
    "agent_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "status": "running"
}
```

#### Error

Error message (e.g., from a failed prompt).

```json
{
    "type": "error",
    "message": "failed to spawn process: claude: command not found"
}
```

#### Pong

Response to client ping.

```json
{
    "type": "pong"
}
```

### Event Types from Claude Code

Events forwarded through the WebSocket maintain Claude Code's stream-json format. The `event_type` field is extracted from the event's `type` field:

| event_type | Description | Key Fields |
|-----------|-------------|------------|
| `system` | Session initialization | `session_id`, `tools`, `model` |
| `assistant` | LLM response | `message.content[]` (text blocks), `message.usage` |
| `user` | User message echo | `message.content[]` |
| `tool_use` | Tool invocation | `name`, `input` (tool-specific params) |
| `tool_result` | Tool execution result | `content` (output text), `is_error` |
| `result` | Final result | `result` (completion text), `cost_usd`, `duration_ms` |

### Connection Lifecycle

```
Client                                  Server
  |                                        |
  |------ WebSocket Upgrade Request ------>|
  |<----- 101 Switching Protocols ---------|
  |                                        |
  |  (default filter: "all")               |
  |                                        |
  |------ { type: "subscribe", ... } ----->|
  |                                        |
  |<----- { type: "event", ... } ----------|
  |<----- { type: "event", ... } ----------|
  |<----- { type: "event", ... } ----------|
  |                                        |
  |------ { type: "ping" } -------------->|
  |<----- { type: "pong" } ---------------|
  |                                        |
  |------ Close Frame ------------------->|
  |<----- Close Frame -------------------|
```

---

## SSE Alternative

For clients that cannot use WebSocket (e.g., simple scripts, curl), a Server-Sent Events endpoint is planned:

#### GET /api/agents/{id}/events/stream (planned)

```
GET /api/agents/{id}/events/stream HTTP/1.1
Accept: text/event-stream
```

**Response:**
```
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive

event: system
data: {"type":"system","session_id":"sess_abc123"}

event: assistant
data: {"type":"assistant","message":{"content":[{"type":"text","text":"Analyzing..."}]}}

event: result
data: {"type":"result","result":"Completed."}
```

**curl example:**
```bash
curl -N http://127.0.0.1:4173/api/agents/f47ac10b-.../events/stream
```

---

## Pagination and Filtering

### Standard Query Parameters

For endpoints that support pagination:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 50 | Maximum items to return (max 1000) |
| `offset` | integer | 0 | Number of items to skip |
| `sort` | string | varies | Sort field (e.g., `created_at`, `name`) |
| `order` | string | `desc` | Sort direction (`asc` or `desc`) |

### Response Envelope (for paginated endpoints, planned)

```json
{
    "data": [ ... ],
    "pagination": {
        "total": 150,
        "limit": 50,
        "offset": 0,
        "has_more": true
    }
}
```

### Filtering Examples

```
GET /api/agents?status=running
GET /api/agents?model=opus
GET /api/sessions?path=/Users/bm/project
GET /api/search?q=refactor&agent_id=f47ac10b-...&event_type=assistant
GET /api/skills?category=security
GET /api/workflows?status=completed
```

### Cursor-Based Pagination (planned for events)

For large event queries, cursor-based pagination is more efficient:

```
GET /api/agents/{id}/events?after=12345&limit=50
```

The cursor is the `id` of the last event seen. This avoids the O(n) skip cost of OFFSET.

**Response:**
```json
{
    "data": [ ... ],
    "cursor": {
        "after": 12395,
        "has_more": true
    }
}
```

---

## Rate Limiting

### v1: No Rate Limiting

For local single-user mode, no rate limiting is applied.

### v2: Configurable Rate Limits (Planned)

When Forge is exposed on a network:

**Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1709129400
```

**429 Response:**
```json
{
    "error": "rate limited: retry after 5000ms",
    "retry_after_ms": 5000
}
```

**Default Limits:**

| Endpoint Category | Rate Limit | Window |
|------------------|-----------|--------|
| Agent CRUD | 60 req/min | 1 minute |
| Prompt submission | 10 req/min per agent | 1 minute |
| Event queries | 120 req/min | 1 minute |
| Search | 30 req/min | 1 minute |
| WebSocket connections | 10 concurrent | -- |

**Implementation:** Token bucket algorithm using DashMap with per-key counters. Cleanup of stale buckets via background task every 5 minutes.

---

## Request and Response Examples

### Complete Workflow: Create Agent, Send Prompt, Stream Results

```bash
# 1. Create an agent
curl -X POST http://127.0.0.1:4173/api/agents \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Security Auditor",
    "model": "opus",
    "permission_mode": "plan",
    "working_directory": "/Users/bm/project",
    "use_gitnexus": true
  }'
# Response: { "id": "abc-123", "status": "idle", ... }

# 2. Open WebSocket and subscribe to this agent
# (using websocat for example)
echo '{"type":"subscribe","filter":{"agent_id":"abc-123"}}' | \
  websocat ws://127.0.0.1:4173/ws

# 3. Send a prompt
curl -X POST http://127.0.0.1:4173/api/agents/abc-123/prompt \
  -H 'Content-Type: application/json' \
  -d '{"text": "Audit this project for security vulnerabilities"}'
# Response: 202 Accepted

# 4. Events stream through WebSocket:
#    {"type":"event","agent_id":"abc-123","event_type":"system",...}
#    {"type":"event","agent_id":"abc-123","event_type":"assistant",...}
#    {"type":"event","agent_id":"abc-123","event_type":"tool_use",...}
#    {"type":"event","agent_id":"abc-123","event_type":"result",...}

# 5. Export results
curl http://127.0.0.1:4173/api/agents/abc-123/export?format=markdown \
  -o audit-results.md

# 6. Check cost
curl http://127.0.0.1:4173/api/cost
# Response: { "total_spent_usd": 0.45, ... }
```

---

### Full Endpoint Summary Table

| Method | Path | Status | Context |
|--------|------|--------|---------|
| POST | `/api/agents` | Implemented | Agent Mgmt |
| GET | `/api/agents` | Implemented | Agent Mgmt |
| GET | `/api/agents/{id}` | Implemented | Agent Mgmt |
| PATCH | `/api/agents/{id}` | Implemented | Agent Mgmt |
| DELETE | `/api/agents/{id}` | Implemented | Agent Mgmt |
| POST | `/api/agents/{id}/prompt` | Implemented | Process Exec |
| GET | `/api/agents/{id}/export` | Implemented | Agent Mgmt |
| GET | `/api/agents/{id}/claude-md` | Implemented | Agent Mgmt |
| PUT | `/api/agents/{id}/claude-md` | Implemented | Agent Mgmt |
| GET | `/api/sessions` | Implemented | Session History |
| GET | `/api/sessions/projects` | Implemented | Session History |
| GET | `/api/presets` | Implemented | Preset Library |
| GET | `/api/cost` | Implemented | Safety & Limits |
| GET | `/api/fs/browse` | Implemented | Presentation |
| GET | `/ws` | Implemented | Presentation |
| GET | `/api/agents/{id}/git/status` | Planned | Git Integration |
| GET | `/api/agents/{id}/git/diff` | Planned | Git Integration |
| GET | `/api/agents/{id}/git/log` | Planned | Git Integration |
| GET | `/api/agents/{id}/git/worktrees` | Planned | Git Integration |
| POST | `/api/agents/{id}/git/worktrees` | Planned | Git Integration |
| DELETE | `/api/agents/{id}/git/worktrees/{name}` | Planned | Git Integration |
| GET | `/api/skills` | Planned | Skill Catalog |
| GET | `/api/skills/{id}` | Planned | Skill Catalog |
| POST | `/api/skills` | Planned | Skill Catalog |
| POST | `/api/skills/{id}/invoke` | Planned | Skill Catalog |
| POST | `/api/workflows` | Planned | Workflow Orch |
| GET | `/api/workflows` | Planned | Workflow Orch |
| GET | `/api/workflows/{id}` | Planned | Workflow Orch |
| POST | `/api/workflows/{id}/start` | Planned | Workflow Orch |
| POST | `/api/workflows/{id}/pause` | Planned | Workflow Orch |
| POST | `/api/workflows/{id}/resume` | Planned | Workflow Orch |
| DELETE | `/api/workflows/{id}` | Planned | Workflow Orch |
| GET | `/api/workflows/{id}/runs` | Planned | Workflow Orch |
| GET | `/api/workflows/{id}/runs/{run_id}` | Planned | Workflow Orch |
| GET | `/api/config` | Planned | System |
| PUT | `/api/config` | Planned | System |
| GET | `/api/search` | Planned | System |
| GET | `/api/agents/{id}/events/stream` | Planned | SSE |
| GET | `/api/agents/{id}/budget` | Planned | Safety & Limits |

---

*Next: [MCP_INTERFACE.md](MCP_INTERFACE.md) for the complete MCP server specification.*
