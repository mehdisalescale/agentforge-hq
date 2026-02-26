# Claude Forge -- MCP Interface

> Complete MCP server design: tools, resources, prompts, transport, and security.
> Forge both consumes external MCP servers AND exposes itself as one.

---

## Table of Contents

1. [Overview](#overview)
2. [Transport Options](#transport-options)
3. [Server Capabilities](#server-capabilities)
4. [Tools Catalog (30+)](#tools-catalog)
5. [Resources Catalog (10+)](#resources-catalog)
6. [Prompts Catalog (7+)](#prompts-catalog)
7. [Security Model](#security-model)
8. [Client Configuration Examples](#client-configuration-examples)
9. [Meta-Agent Orchestration Patterns](#meta-agent-orchestration-patterns)

---

## Overview

Claude Forge has a dual MCP relationship:

```
+-------------------------+       +-------------------------+
|  External MCP Servers   |       |  External MCP Clients   |
| (GitNexus, filesystem)  |       | (Claude Code, VS Code)  |
+----------+--------------+       +----------+--------------+
           |                                  |
     Forge CONSUMES                    Forge EXPOSES
     (via --mcp-config                 (as MCP server
      passed to claude CLI)             via stdio/SSE)
           |                                  |
+----------v--------------+                   |
|                          |<-----------------+
|     Claude Forge         |
|     MCP Server           |
|                          |
|  30+ tools               |
|  10+ resources           |
|  7+ prompts              |
+-------------------------+
```

**Key design principle:** Each tool has one clear purpose. Tools are named with a `forge_` prefix to avoid conflicts with other MCP servers.

---

## Transport Options

### Stdio Transport (Primary)

The default transport for local use. Forge starts as an MCP server when invoked with the `--mcp` flag:

```bash
claude-forge --mcp
```

This mode:
- Reads JSON-RPC messages from stdin
- Writes JSON-RPC responses to stdout
- Logs to stderr (not mixed with protocol)
- No HTTP server started (pure MCP mode)

### SSE Transport (Planned)

For remote access or when the HTTP server is already running:

```bash
claude-forge --mcp-sse --port 4174
```

This mode:
- Exposes MCP over Server-Sent Events at `http://host:4174/mcp/sse`
- Shares state with the main HTTP server if both are running
- Supports multiple concurrent MCP clients

### Dual Mode (Planned)

Run both the web UI and MCP server simultaneously:

```bash
claude-forge --port 4173 --mcp-sse --mcp-port 4174
```

---

## Server Capabilities

```json
{
    "protocolVersion": "2024-11-05",
    "capabilities": {
        "tools": {
            "listChanged": true
        },
        "resources": {
            "subscribe": true,
            "listChanged": true
        },
        "prompts": {
            "listChanged": true
        },
        "logging": {}
    },
    "serverInfo": {
        "name": "claude-forge",
        "version": "0.1.0"
    }
}
```

---

## Tools Catalog

### Agent Management Tools

#### forge_create_agent

Create a new agent with configuration.

```json
{
    "name": "forge_create_agent",
    "description": "Create a new Claude Code agent with specified configuration. Returns the agent ID for subsequent operations.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Human-readable name for the agent"
            },
            "model": {
                "type": "string",
                "enum": ["opus", "sonnet", "haiku"],
                "description": "Claude model to use. Default: sonnet",
                "default": "sonnet"
            },
            "system_prompt": {
                "type": "string",
                "description": "Custom system prompt (replaces Claude Code default)"
            },
            "append_system_prompt": {
                "type": "string",
                "description": "Additional system prompt (appended to Claude Code default)"
            },
            "permission_mode": {
                "type": "string",
                "enum": ["default", "plan", "acceptEdits", "dontAsk", "bypassPermissions"],
                "description": "Permission mode for tool usage",
                "default": "default"
            },
            "working_directory": {
                "type": "string",
                "description": "Working directory for the agent"
            },
            "max_budget_usd": {
                "type": "number",
                "description": "Maximum budget in USD for this agent"
            },
            "max_turns": {
                "type": "integer",
                "description": "Maximum conversation turns"
            },
            "use_gitnexus": {
                "type": "boolean",
                "description": "Enable GitNexus MCP server for this agent",
                "default": false
            },
            "preset": {
                "type": "string",
                "description": "Preset ID to use as base configuration (planner, reviewer, bug-hunter, refactor, security, fullstack, tester, docs, quick)"
            }
        },
        "required": ["name"]
    }
}
```

#### forge_list_agents

```json
{
    "name": "forge_list_agents",
    "description": "List all agents with their current status, model, usage statistics, and session IDs.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["idle", "running", "stopped", "error"],
                "description": "Filter by agent status"
            },
            "model": {
                "type": "string",
                "description": "Filter by model name"
            }
        }
    }
}
```

#### forge_get_agent

```json
{
    "name": "forge_get_agent",
    "description": "Get detailed information about a specific agent including configuration, recent events, and usage statistics.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent"
            }
        },
        "required": ["agent_id"]
    }
}
```

#### forge_update_agent

```json
{
    "name": "forge_update_agent",
    "description": "Update an agent's configuration. Only provided fields are changed. Agent must not be running.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent to update"
            },
            "name": { "type": "string" },
            "model": { "type": "string" },
            "system_prompt": { "type": ["string", "null"] },
            "append_system_prompt": { "type": ["string", "null"] },
            "permission_mode": { "type": "string" },
            "max_budget_usd": { "type": ["number", "null"] },
            "max_turns": { "type": ["integer", "null"] },
            "working_directory": { "type": "string" }
        },
        "required": ["agent_id"]
    }
}
```

#### forge_delete_agent

```json
{
    "name": "forge_delete_agent",
    "description": "Delete an agent and all its event history. This action is irreversible.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent to delete"
            }
        },
        "required": ["agent_id"]
    }
}
```

### Execution Tools

#### forge_send_prompt

```json
{
    "name": "forge_send_prompt",
    "description": "Send a prompt to an agent. The agent will spawn a Claude Code process to handle the prompt. Results stream asynchronously. Use forge_get_agent or forge_wait_for_result to check completion.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent to prompt"
            },
            "text": {
                "type": "string",
                "description": "The prompt text to send"
            }
        },
        "required": ["agent_id", "text"]
    }
}
```

#### forge_stop_agent

```json
{
    "name": "forge_stop_agent",
    "description": "Stop a running agent's Claude Code process.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent to stop"
            }
        },
        "required": ["agent_id"]
    }
}
```

#### forge_wait_for_result

```json
{
    "name": "forge_wait_for_result",
    "description": "Wait for an agent to complete its current prompt and return the result. Polls status until the agent returns to idle. Times out after the specified duration.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent to wait for"
            },
            "timeout_seconds": {
                "type": "integer",
                "description": "Maximum time to wait in seconds",
                "default": 300
            }
        },
        "required": ["agent_id"]
    }
}
```

#### forge_send_and_wait

```json
{
    "name": "forge_send_and_wait",
    "description": "Send a prompt to an agent and wait for the result. Combines forge_send_prompt and forge_wait_for_result into a single synchronous call. Ideal for scripted workflows.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent"
            },
            "text": {
                "type": "string",
                "description": "The prompt text"
            },
            "timeout_seconds": {
                "type": "integer",
                "description": "Maximum wait time in seconds",
                "default": 300
            }
        },
        "required": ["agent_id", "text"]
    }
}
```

### Preset and Skill Tools

#### forge_list_presets

```json
{
    "name": "forge_list_presets",
    "description": "List available agent presets with their configurations. Presets include Planner, Reviewer, Bug Hunter, Refactoring Expert, Security Auditor, Full-Stack Dev, Test Engineer, Documentation Writer, and Quick Task.",
    "inputSchema": {
        "type": "object",
        "properties": {}
    }
}
```

#### forge_create_from_preset

```json
{
    "name": "forge_create_from_preset",
    "description": "Create an agent using a preset configuration as the base. Optionally override any preset field.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "preset_id": {
                "type": "string",
                "enum": ["planner", "reviewer", "bug-hunter", "refactor", "security", "fullstack", "tester", "docs", "quick"],
                "description": "ID of the preset to use"
            },
            "name": {
                "type": "string",
                "description": "Override the agent name (default: preset name)"
            },
            "working_directory": {
                "type": "string",
                "description": "Working directory for the agent"
            },
            "model": {
                "type": "string",
                "description": "Override the model (default: preset model)"
            }
        },
        "required": ["preset_id"]
    }
}
```

#### forge_list_skills

```json
{
    "name": "forge_list_skills",
    "description": "List available skills (reusable prompt templates) in the skill catalog.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "category": {
                "type": "string",
                "enum": ["coding", "review", "testing", "docs", "security", "workflow", "custom"],
                "description": "Filter by category"
            }
        }
    }
}
```

#### forge_invoke_skill

```json
{
    "name": "forge_invoke_skill",
    "description": "Invoke a skill on a specific agent. The skill's prompt template is filled with the provided arguments and sent to the agent.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "skill_id": {
                "type": "string",
                "description": "ID of the skill to invoke"
            },
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent to run the skill on"
            },
            "arguments": {
                "type": "object",
                "description": "Arguments to fill into the skill's prompt template",
                "additionalProperties": true
            }
        },
        "required": ["skill_id", "agent_id"]
    }
}
```

### Workflow Tools

#### forge_create_workflow

```json
{
    "name": "forge_create_workflow",
    "description": "Define a multi-step workflow as a DAG. Each step specifies an agent preset and prompt template. Dependencies determine execution order.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Workflow name"
            },
            "description": {
                "type": "string",
                "description": "What this workflow does"
            },
            "steps": {
                "type": "array",
                "description": "Ordered list of workflow steps",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Unique step identifier" },
                        "name": { "type": "string", "description": "Step display name" },
                        "preset": { "type": "string", "description": "Agent preset to use" },
                        "prompt": { "type": "string", "description": "Prompt template with {{variable}} placeholders" },
                        "depends_on": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Step IDs that must complete before this step"
                        }
                    },
                    "required": ["id", "name", "preset", "prompt"]
                }
            },
            "variables": {
                "type": "object",
                "description": "Variable definitions for prompt templates",
                "additionalProperties": {
                    "type": "object",
                    "properties": {
                        "type": { "type": "string" },
                        "required": { "type": "boolean" },
                        "default": { "type": "string" }
                    }
                }
            }
        },
        "required": ["name", "steps"]
    }
}
```

#### forge_run_workflow

```json
{
    "name": "forge_run_workflow",
    "description": "Execute a previously defined workflow with the specified variable values.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "workflow_id": {
                "type": "string",
                "description": "ID of the workflow to execute"
            },
            "variables": {
                "type": "object",
                "description": "Variable values to fill into step prompt templates",
                "additionalProperties": true
            },
            "working_directory": {
                "type": "string",
                "description": "Working directory for all agents in this run"
            }
        },
        "required": ["workflow_id"]
    }
}
```

#### forge_workflow_status

```json
{
    "name": "forge_workflow_status",
    "description": "Get the current status of a workflow run including each step's progress, agent assignments, and costs.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "run_id": {
                "type": "string",
                "description": "ID of the workflow run"
            }
        },
        "required": ["run_id"]
    }
}
```

### Coordination Tools

#### forge_handoff

```json
{
    "name": "forge_handoff",
    "description": "Transfer context from one agent to another. Extracts the source agent's latest result and sends it as context to the target agent with a follow-up prompt.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "from_agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "Agent to take context from"
            },
            "to_agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "Agent to send context to"
            },
            "prompt": {
                "type": "string",
                "description": "Prompt for the target agent. The source agent's last result is prepended as context."
            }
        },
        "required": ["from_agent_id", "to_agent_id", "prompt"]
    }
}
```

#### forge_broadcast

```json
{
    "name": "forge_broadcast",
    "description": "Send a message to all agents in a group or all active agents.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "message": {
                "type": "string",
                "description": "Message to broadcast"
            },
            "group_id": {
                "type": "string",
                "description": "Agent group ID (omit for all agents)"
            },
            "exclude": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Agent IDs to exclude from broadcast"
            }
        },
        "required": ["message"]
    }
}
```

### Session and History Tools

#### forge_list_sessions

```json
{
    "name": "forge_list_sessions",
    "description": "List Claude Code sessions from the local session history. Sessions include those created by Forge and directly by Claude Code CLI.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "working_directory": {
                "type": "string",
                "description": "Filter sessions by working directory path"
            },
            "limit": {
                "type": "integer",
                "description": "Maximum sessions to return",
                "default": 50
            }
        }
    }
}
```

#### forge_search_events

```json
{
    "name": "forge_search_events",
    "description": "Full-text search across all agent event history. Uses FTS5 with porter stemming.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query (supports phrases, boolean operators, prefix matching)"
            },
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "Filter to a specific agent"
            },
            "event_type": {
                "type": "string",
                "enum": ["system", "assistant", "user", "tool_use", "tool_result", "result"],
                "description": "Filter by event type"
            },
            "limit": {
                "type": "integer",
                "description": "Maximum results",
                "default": 20
            }
        },
        "required": ["query"]
    }
}
```

### Git Tools

#### forge_git_status

```json
{
    "name": "forge_git_status",
    "description": "Get git repository status for a directory. Shows branch, staged/unstaged changes, and remote tracking info.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "working_directory": {
                "type": "string",
                "description": "Path to the git repository"
            }
        },
        "required": ["working_directory"]
    }
}
```

#### forge_git_diff

```json
{
    "name": "forge_git_diff",
    "description": "Get git diff for a directory. Shows file changes, insertions, deletions, and diff hunks.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "working_directory": {
                "type": "string",
                "description": "Path to the git repository"
            },
            "base": {
                "type": "string",
                "description": "Base ref for comparison (e.g., HEAD~1, main, commit SHA)"
            },
            "file": {
                "type": "string",
                "description": "Filter to a specific file path"
            }
        },
        "required": ["working_directory"]
    }
}
```

#### forge_create_worktree

```json
{
    "name": "forge_create_worktree",
    "description": "Create a git worktree for isolated agent work. Returns the worktree path that can be used as a working directory.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "working_directory": {
                "type": "string",
                "description": "Path to the main git repository"
            },
            "name": {
                "type": "string",
                "description": "Worktree name (used for directory and branch)"
            },
            "base_branch": {
                "type": "string",
                "description": "Branch to base the worktree on",
                "default": "HEAD"
            }
        },
        "required": ["working_directory", "name"]
    }
}
```

### System Tools

#### forge_get_cost

```json
{
    "name": "forge_get_cost",
    "description": "Get cost summary across all agents including total spend, token counts, and per-agent breakdown.",
    "inputSchema": {
        "type": "object",
        "properties": {}
    }
}
```

#### forge_get_config

```json
{
    "name": "forge_get_config",
    "description": "Get current Forge configuration including default model, budget limits, and retention settings.",
    "inputSchema": {
        "type": "object",
        "properties": {}
    }
}
```

#### forge_update_config

```json
{
    "name": "forge_update_config",
    "description": "Update Forge configuration. Only provided fields are changed.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "default_model": {
                "type": "string",
                "description": "Default model for new agents"
            },
            "global_max_budget_usd": {
                "type": ["number", "null"],
                "description": "Global budget limit in USD (null for no limit)"
            },
            "event_retention_days": {
                "type": "integer",
                "description": "Days to retain events before cleanup"
            }
        }
    }
}
```

#### forge_export_agent

```json
{
    "name": "forge_export_agent",
    "description": "Export an agent's full data including configuration, events, and usage statistics.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID of the agent to export"
            },
            "format": {
                "type": "string",
                "enum": ["json", "markdown"],
                "description": "Export format",
                "default": "json"
            }
        },
        "required": ["agent_id"]
    }
}
```

#### forge_browse_files

```json
{
    "name": "forge_browse_files",
    "description": "Browse filesystem directories. Returns directory entries (no files, no hidden dirs).",
    "inputSchema": {
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Directory path to browse"
            }
        },
        "required": ["path"]
    }
}
```

### Tools Summary

| # | Tool Name | Category | Status |
|---|-----------|----------|--------|
| 1 | `forge_create_agent` | Agent Mgmt | Planned |
| 2 | `forge_list_agents` | Agent Mgmt | Planned |
| 3 | `forge_get_agent` | Agent Mgmt | Planned |
| 4 | `forge_update_agent` | Agent Mgmt | Planned |
| 5 | `forge_delete_agent` | Agent Mgmt | Planned |
| 6 | `forge_send_prompt` | Execution | Planned |
| 7 | `forge_stop_agent` | Execution | Planned |
| 8 | `forge_wait_for_result` | Execution | Planned |
| 9 | `forge_send_and_wait` | Execution | Planned |
| 10 | `forge_list_presets` | Presets | Planned |
| 11 | `forge_create_from_preset` | Presets | Planned |
| 12 | `forge_list_skills` | Skills | Planned |
| 13 | `forge_invoke_skill` | Skills | Planned |
| 14 | `forge_create_workflow` | Workflow | Planned |
| 15 | `forge_run_workflow` | Workflow | Planned |
| 16 | `forge_workflow_status` | Workflow | Planned |
| 17 | `forge_handoff` | Coordination | Planned |
| 18 | `forge_broadcast` | Coordination | Planned |
| 19 | `forge_list_sessions` | Sessions | Planned |
| 20 | `forge_search_events` | Search | Planned |
| 21 | `forge_git_status` | Git | Planned |
| 22 | `forge_git_diff` | Git | Planned |
| 23 | `forge_create_worktree` | Git | Planned |
| 24 | `forge_get_cost` | System | Planned |
| 25 | `forge_get_config` | System | Planned |
| 26 | `forge_update_config` | System | Planned |
| 27 | `forge_export_agent` | System | Planned |
| 28 | `forge_browse_files` | System | Planned |
| 29 | `forge_read_claude_md` | Agent Mgmt | Planned |
| 30 | `forge_write_claude_md` | Agent Mgmt | Planned |
| 31 | `forge_list_worktrees` | Git | Planned |
| 32 | `forge_remove_worktree` | Git | Planned |

---

## Resources Catalog

MCP resources provide read-only access to Forge state. Resources use URI patterns that can contain variables.

### forge://agents

List all agents as a resource.

```json
{
    "uri": "forge://agents",
    "name": "Forge Agents",
    "description": "List of all agents with their status and configuration",
    "mimeType": "application/json"
}
```

### forge://agents/{id}

Single agent detail.

```json
{
    "uri": "forge://agents/{id}",
    "name": "Agent Detail",
    "description": "Detailed information about a specific agent",
    "mimeType": "application/json"
}
```

### forge://agents/{id}/events

Event history for an agent.

```json
{
    "uri": "forge://agents/{id}/events",
    "name": "Agent Events",
    "description": "Event history for a specific agent. Last 100 events by default.",
    "mimeType": "application/json"
}
```

### forge://agents/{id}/config

Agent configuration (read-only view).

```json
{
    "uri": "forge://agents/{id}/config",
    "name": "Agent Configuration",
    "description": "Current configuration for an agent including model, permissions, MCP servers, and hooks",
    "mimeType": "application/json"
}
```

### forge://presets

Available agent presets.

```json
{
    "uri": "forge://presets",
    "name": "Agent Presets",
    "description": "All available agent presets with descriptions and configurations",
    "mimeType": "application/json"
}
```

### forge://sessions

Session history from Claude Code.

```json
{
    "uri": "forge://sessions",
    "name": "Session History",
    "description": "All Claude Code sessions from ~/.claude/projects/",
    "mimeType": "application/json"
}
```

### forge://sessions/{project}

Sessions for a specific project.

```json
{
    "uri": "forge://sessions/{project}",
    "name": "Project Sessions",
    "description": "Sessions for a specific project directory",
    "mimeType": "application/json"
}
```

### forge://cost

Cost summary.

```json
{
    "uri": "forge://cost",
    "name": "Cost Summary",
    "description": "Total cost, token usage, and per-agent cost breakdown",
    "mimeType": "application/json"
}
```

### forge://workflows

Workflow definitions.

```json
{
    "uri": "forge://workflows",
    "name": "Workflows",
    "description": "All defined workflows with their step graphs",
    "mimeType": "application/json"
}
```

### forge://workflows/{id}/runs

Workflow run history.

```json
{
    "uri": "forge://workflows/{id}/runs",
    "name": "Workflow Runs",
    "description": "Execution history for a specific workflow",
    "mimeType": "application/json"
}
```

### forge://skills

Skill catalog.

```json
{
    "uri": "forge://skills",
    "name": "Skill Catalog",
    "description": "Available skills organized by category",
    "mimeType": "application/json"
}
```

### forge://config

Forge configuration.

```json
{
    "uri": "forge://config",
    "name": "Forge Configuration",
    "description": "Current Forge configuration settings",
    "mimeType": "application/json"
}
```

### Resource Subscriptions

Clients can subscribe to resource changes. Forge notifies subscribers when:

| Resource | Notification Trigger |
|----------|---------------------|
| `forge://agents` | Agent created, updated, deleted, or status changed |
| `forge://agents/{id}` | Specific agent updated or status changed |
| `forge://agents/{id}/events` | New events for the agent |
| `forge://cost` | Usage/cost updated (batched, max 1/second) |
| `forge://workflows/{id}/runs` | Workflow run status changed |

---

## Prompts Catalog

MCP prompts provide reusable prompt templates that clients can fill and send to agents.

### forge_plan_project

```json
{
    "name": "forge_plan_project",
    "description": "Create a comprehensive project plan with architecture analysis, task breakdown, and dependency mapping.",
    "arguments": [
        {
            "name": "project_directory",
            "description": "Path to the project to analyze",
            "required": true
        },
        {
            "name": "goal",
            "description": "What you want to accomplish",
            "required": true
        },
        {
            "name": "constraints",
            "description": "Any constraints or requirements (technology choices, deadlines, etc.)",
            "required": false
        }
    ]
}
```

**Generated prompt:**
```
Analyze the project at {{project_directory}} and create a plan for: {{goal}}

Steps:
1. Read and understand the project structure, key files, and architecture
2. Identify what needs to change to accomplish the goal
3. Break down the work into ordered tasks with dependencies
4. Identify risks and edge cases
5. Estimate scope (minimal viable change vs optional enhancements)

{{#if constraints}}Constraints: {{constraints}}{{/if}}

Output a numbered task list with dependencies, estimated complexity, and any questions.
```

### forge_review_code

```json
{
    "name": "forge_review_code",
    "description": "Perform a thorough code review on recent changes or a specific directory.",
    "arguments": [
        {
            "name": "target",
            "description": "Directory path or file pattern to review",
            "required": true
        },
        {
            "name": "focus",
            "description": "Specific areas to focus on (security, performance, correctness, style)",
            "required": false
        },
        {
            "name": "severity_threshold",
            "description": "Minimum severity to report: critical, warning, or suggestion",
            "required": false
        }
    ]
}
```

### forge_debug_issue

```json
{
    "name": "forge_debug_issue",
    "description": "Investigate and fix a bug using hypothesis-driven debugging.",
    "arguments": [
        {
            "name": "description",
            "description": "Description of the bug or error message",
            "required": true
        },
        {
            "name": "reproduction_steps",
            "description": "How to reproduce the issue",
            "required": false
        },
        {
            "name": "suspected_area",
            "description": "Files or modules where the bug might be",
            "required": false
        }
    ]
}
```

### forge_security_audit

```json
{
    "name": "forge_security_audit",
    "description": "Run a security audit following the OWASP checklist.",
    "arguments": [
        {
            "name": "target",
            "description": "Directory or module to audit",
            "required": true
        },
        {
            "name": "threat_model",
            "description": "Description of the threat model (who are the attackers, what are the assets)",
            "required": false
        }
    ]
}
```

### forge_write_tests

```json
{
    "name": "forge_write_tests",
    "description": "Generate tests for a module or function.",
    "arguments": [
        {
            "name": "target",
            "description": "File or module to test",
            "required": true
        },
        {
            "name": "framework",
            "description": "Test framework to use (auto-detected if omitted)",
            "required": false
        },
        {
            "name": "coverage_goal",
            "description": "Focus areas: happy path, edge cases, error handling, integration",
            "required": false
        }
    ]
}
```

### forge_refactor

```json
{
    "name": "forge_refactor",
    "description": "Safely refactor code with full reference checking.",
    "arguments": [
        {
            "name": "target",
            "description": "What to refactor (file, function, module)",
            "required": true
        },
        {
            "name": "goal",
            "description": "What the refactoring should achieve",
            "required": true
        },
        {
            "name": "preserve",
            "description": "Behavior or interfaces that must not change",
            "required": false
        }
    ]
}
```

### forge_document

```json
{
    "name": "forge_document",
    "description": "Generate or update documentation for code.",
    "arguments": [
        {
            "name": "target",
            "description": "What to document (module, API, README, architecture)",
            "required": true
        },
        {
            "name": "audience",
            "description": "Target audience (developers, users, operators)",
            "required": false
        },
        {
            "name": "format",
            "description": "Documentation format (markdown, inline comments, docstrings)",
            "required": false
        }
    ]
}
```

### forge_pipeline

```json
{
    "name": "forge_pipeline",
    "description": "Run a multi-agent pipeline: plan, implement, review, test.",
    "arguments": [
        {
            "name": "task",
            "description": "What to accomplish",
            "required": true
        },
        {
            "name": "working_directory",
            "description": "Project directory",
            "required": true
        },
        {
            "name": "stages",
            "description": "Which stages to run: plan, implement, review, test (comma-separated)",
            "required": false
        }
    ]
}
```

---

## Security Model

### What Is Exposed

| Category | Exposed | Rationale |
|----------|---------|-----------|
| Agent CRUD | Yes | Core functionality |
| Prompt submission | Yes | Core functionality |
| Event history | Yes (read-only) | Observability |
| Session history | Yes (read-only) | Browsing past work |
| File browsing | Yes (directories only) | Directory picker for agent config |
| File read/write | No | Agents do this via Claude Code, not Forge |
| CLAUDE.md read/write | Yes | Agent configuration |
| System commands | No | Never exposed |
| Environment variables | No | Never exposed |
| API keys | No | Never exposed; keys are in env, not in Forge |

### Permission Boundaries

```
MCP Client (e.g., Claude Code)
     |
     | JSON-RPC call
     v
+----+----+
| Validate | -- Is the tool name valid?
| Schema   | -- Do the arguments match the schema?
+----+----+
     |
+----+----+
| Authorize| -- (v1: always allow, v2: check API key scope)
+----+----+
     |
+----+----+
| Budget   | -- Would this exceed the agent's or global budget?
| Check    |
+----+----+
     |
+----+----+
| Execute  | -- Delegate to internal Forge operation
+----+----+
     |
+----+----+
| Sanitize | -- Remove internal fields, temp file paths, etc.
| Response |
+----+----+
     |
     v
MCP Client
```

### What Is NOT Exposed

- Raw database access
- Internal DashMap state
- Process PIDs or OS-level details
- Filesystem write operations (except CLAUDE.md)
- Forge configuration file paths
- tokio runtime internals

### Rate Limiting (Planned)

MCP tool calls will be rate-limited independently of HTTP API rate limits:
- 30 tool calls per minute per MCP client connection
- 5 concurrent agent prompts
- Workflow starts limited to 3 per minute

---

## Client Configuration Examples

### Claude Code (settings.json or .mcp.json)

**Stdio transport:**
```json
{
    "mcpServers": {
        "forge": {
            "command": "claude-forge",
            "args": ["--mcp"],
            "env": {}
        }
    }
}
```

**SSE transport (when Forge is already running):**
```json
{
    "mcpServers": {
        "forge": {
            "type": "sse",
            "url": "http://127.0.0.1:4174/mcp/sse"
        }
    }
}
```

### VS Code (MCP extension)

```json
{
    "mcp.servers": {
        "forge": {
            "command": "claude-forge",
            "args": ["--mcp"],
            "transport": "stdio"
        }
    }
}
```

### Cursor

```json
{
    "mcpServers": {
        "forge": {
            "command": "claude-forge",
            "args": ["--mcp"]
        }
    }
}
```

### Custom Script (using MCP SDK)

```python
from mcp import ClientSession, StdioServerParameters
import asyncio

async def main():
    server_params = StdioServerParameters(
        command="claude-forge",
        args=["--mcp"]
    )

    async with ClientSession(server_params) as session:
        # Create an agent
        result = await session.call_tool("forge_create_agent", {
            "name": "My Reviewer",
            "preset": "reviewer",
            "working_directory": "/path/to/project"
        })
        agent_id = result["agent_id"]

        # Send a prompt and wait for result
        result = await session.call_tool("forge_send_and_wait", {
            "agent_id": agent_id,
            "text": "Review the latest changes",
            "timeout_seconds": 120
        })
        print(result["text"])

asyncio.run(main())
```

---

## Meta-Agent Orchestration Patterns

These patterns show how an external AI agent (e.g., Claude Code with Forge as MCP) can orchestrate multiple Forge agents.

### Pattern 1: Supervisor

One Claude Code session creates and manages multiple Forge agents.

```
Claude Code (supervisor)
     |
     +-- forge_create_from_preset("planner", dir="/project")
     |        |
     |        +-- forge_send_and_wait("Analyze auth module")
     |        |        |
     |        |        v
     |        |   Plan result: "3 tasks identified..."
     |        |
     +-- forge_create_from_preset("fullstack", dir="/project")
     |        |
     |        +-- forge_send_and_wait("Implement task 1 from plan: ...")
     |        |
     +-- forge_create_from_preset("reviewer", dir="/project")
              |
              +-- forge_send_and_wait("Review the changes just made")
```

### Pattern 2: Fan-Out / Fan-In

Supervisor creates multiple agents for parallel work, then aggregates.

```
Claude Code (supervisor)
     |
     +-- Create 3 reviewers (different focus areas)
     |        |
     |        +-- Agent A: forge_send_prompt("Review security")
     |        +-- Agent B: forge_send_prompt("Review performance")
     |        +-- Agent C: forge_send_prompt("Review correctness")
     |
     +-- Wait for all: forge_wait_for_result(A), (B), (C)
     |
     +-- Aggregate results into a summary
```

### Pattern 3: Workflow Pipeline

Use Forge's workflow engine for complex multi-step processes.

```
Claude Code
     |
     +-- forge_create_workflow({
     |       name: "Full Review Pipeline",
     |       steps: [plan, review, security, tests, summary]
     |   })
     |
     +-- forge_run_workflow(workflow_id, {
     |       target_dir: "/project/src",
     |       focus: "auth module refactor"
     |   })
     |
     +-- Poll: forge_workflow_status(run_id)
     |         -> "step 1 complete, steps 2-3 running..."
     |         -> "all steps complete"
     |
     +-- Read results from each step's agent
```

### Pattern 4: Iterative Refinement

Agent A produces output, Agent B critiques, Agent A revises.

```
Claude Code (coordinator)
     |
     +-- Agent A (coder): forge_send_and_wait("Implement feature X")
     |        |
     |        v result_v1
     |
     +-- Agent B (reviewer): forge_send_and_wait("Review this: {result_v1}")
     |        |
     |        v feedback
     |
     +-- Agent A (coder): forge_send_and_wait("Revise based on: {feedback}")
     |        |
     |        v result_v2
     |
     +-- (repeat until reviewer approves)
```

### Pattern 5: Context Handoff

Use forge_handoff for structured context transfer between specialized agents.

```
Claude Code
     |
     +-- Agent A (planner): forge_send_and_wait("Create implementation plan")
     |
     +-- forge_handoff(from=A, to=B, prompt="Implement the plan above")
     |        |
     |        | (Forge extracts A's result, prepends to B's prompt)
     |        v
     |   Agent B (coder): receives plan + implementation prompt
     |
     +-- forge_handoff(from=B, to=C, prompt="Write tests for the implementation")
              |
              v
         Agent C (tester): receives implementation context + testing prompt
```

---

*Next: [EVENT_SYSTEM.md](EVENT_SYSTEM.md) for the complete event architecture.*
