# Claude Forge -- Bounded Contexts

> Domain-Driven Design bounded contexts for a multi-agent Claude Code orchestrator.
> 12 bounded contexts with clear boundaries, event contracts, and anti-corruption layers.

---

## Table of Contents

1. [Context Overview](#context-overview)
2. [Context Map](#context-map)
3. [Context Details](#context-details)
   - [1. Agent Management](#1-agent-management)
   - [2. Process Execution](#2-process-execution)
   - [3. Event Streaming](#3-event-streaming)
   - [4. Persistence](#4-persistence)
   - [5. Session History](#5-session-history)
   - [6. Preset Library](#6-preset-library)
   - [7. Safety and Limits](#7-safety-and-limits)
   - [8. Workflow Orchestration](#8-workflow-orchestration)
   - [9. MCP Interface](#9-mcp-interface)
   - [10. Skill Catalog](#10-skill-catalog)
   - [11. Git Integration](#11-git-integration)
   - [12. Presentation](#12-presentation)
4. [Anti-Corruption Layers](#anti-corruption-layers)
5. [Ubiquitous Language](#ubiquitous-language)

---

## Context Overview

```
+-----------------------------------------------------------------------+
|                        Claude Forge Contexts                           |
|                                                                        |
|  CORE DOMAIN (what makes Forge unique)                                 |
|  +-------------------+  +-------------------+  +-------------------+   |
|  | 1. Agent          |  | 2. Process        |  | 8. Workflow       |   |
|  |    Management     |  |    Execution      |  |    Orchestration  |   |
|  +-------------------+  +-------------------+  +-------------------+   |
|                                                                        |
|  SUPPORTING DOMAIN (enables core but not differentiating)              |
|  +-------------------+  +-------------------+  +-------------------+   |
|  | 3. Event          |  | 4. Persistence    |  | 7. Safety &       |   |
|  |    Streaming      |  |                   |  |    Limits         |   |
|  +-------------------+  +-------------------+  +-------------------+   |
|                                                                        |
|  +-------------------+  +-------------------+  +-------------------+   |
|  | 5. Session        |  | 6. Preset         |  | 10. Skill         |   |
|  |    History        |  |    Library        |  |     Catalog       |   |
|  +-------------------+  +-------------------+  +-------------------+   |
|                                                                        |
|  GENERIC DOMAIN (could be a library, not Forge-specific)               |
|  +-------------------+  +-------------------+  +-------------------+   |
|  | 9. MCP            |  | 11. Git           |  | 12. Presentation  |   |
|  |    Interface      |  |     Integration   |  |     (Frontend)    |   |
|  +-------------------+  +-------------------+  +-------------------+   |
+-----------------------------------------------------------------------+
```

---

## Context Map

Relationships between bounded contexts. Arrows show upstream (provider) to downstream (consumer).

```
                        +-------------------+
                        |  12. Presentation |
                        |    (Frontend)     |
                        +--------+----------+
                                 |
                    consumes API & WebSocket
                                 |
         +-----------+-----------+-----------+-----------+
         |           |           |           |           |
         v           v           v           v           v
  +------+---+ +-----+----+ +---+-----+ +---+-----+ +--+-------+
  | 1. Agent | | 3. Event | | 5. Sess | | 6. Pre- | | 11. Git  |
  |   Mgmt   | |  Stream  | |  Hist.  | |  sets   | |  Integ.  |
  +----+-----+ +----+-----+ +---------+ +---------+ +----------+
       |             |
       |  publishes  |  subscribes
       |  events to  |  to events
       v             v
  +----+-------------+----+
  |   3. Event Streaming   |<------------ 2. Process Execution
  |   (shared kernel)      |              (publishes raw events)
  +----+--+----------------+
       |  |
       |  +-------> 7. Safety & Limits (consumes events for budget tracking)
       |
       +----------> 4. Persistence (consumes events for batch writes)

  +-------------------+           +-------------------+
  | 8. Workflow       |---------->| 2. Process        |
  |    Orchestration  |  spawns   |    Execution      |
  +--------+----------+  agents   +-------------------+
           |
           +---------->  1. Agent Mgmt (creates/queries agents)
           +---------->  3. Event Streaming (monitors step completion)

  +-------------------+
  | 9. MCP Interface  |---------->  1. Agent Mgmt (CRUD via tools)
  |                   |---------->  8. Workflow Orch. (workflow tools)
  |                   |---------->  3. Event Streaming (subscribe resources)
  +-------------------+

  +-------------------+
  | 10. Skill Catalog |---------->  6. Preset Library (skills reference presets)
  +-------------------+
```

### Relationship Types

| Upstream | Downstream | Relationship | Notes |
|----------|-----------|-------------|-------|
| Agent Management | Event Streaming | Published Language | Agent lifecycle events |
| Process Execution | Event Streaming | Published Language | Raw stream-json events |
| Event Streaming | Persistence | Customer-Supplier | Persistence batches events from stream |
| Event Streaming | Safety & Limits | Customer-Supplier | Safety monitors spend via events |
| Event Streaming | Presentation | Published Language | WebSocket forwards events to frontend |
| Agent Management | Persistence | Customer-Supplier | Agent CRUD persisted immediately |
| Workflow Orchestration | Agent Management | Conformist | Workflows use agent API as-is |
| Workflow Orchestration | Process Execution | Conformist | Workflows trigger prompts via process exec |
| MCP Interface | Agent Management | Anti-Corruption Layer | MCP translates JSON-RPC to internal calls |
| Preset Library | Agent Management | Shared Kernel | Presets produce AgentConfig values |
| Skill Catalog | Preset Library | Customer-Supplier | Skills may reference preset configurations |

---

## Context Details

### 1. Agent Management

**Purpose:** Owns the agent lifecycle -- creation, configuration, updates, deletion. The central concept in Forge.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| Agent | AgentHandle | AgentConfig, AgentStatus, TokenUsage |
| AgentConfig | (value object) | PermissionMode, McpServerConfig, HooksConfig |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `AgentCreated` | POST /api/agents | agent_id, name, model |
| `AgentUpdated` | PATCH /api/agents/:id | agent_id, changed_fields |
| `AgentDeleted` | DELETE /api/agents/:id | agent_id |
| `AgentStatusChanged` | Process start/stop/error | agent_id, old_status, new_status |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| `UsageAccumulated` | Event Streaming | Update agent's TokenUsage |
| `SessionIdDiscovered` | Event Streaming | Set agent's session_id for --resume |

**Public Interface:**

```rust
// What other contexts can call
pub fn register_agent(state: &AppState, config: AgentConfig) -> Uuid;
pub fn get_agent(state: &AppState, id: Uuid) -> Option<Ref<AgentHandle>>;
pub fn update_agent(state: &AppState, id: Uuid, req: UpdateAgentRequest) -> Result<()>;
pub fn delete_agent(state: &AppState, id: Uuid) -> Result<()>;
pub fn list_agents(state: &AppState) -> Vec<AgentResponse>;
```

**Database Tables Owned:**

| Table | Purpose |
|-------|---------|
| `agents` | Agent records (id, name, config JSON, session_id, status, usage JSON, timestamps) |

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/agents` | Create agent |
| GET | `/api/agents` | List all agents |
| GET | `/api/agents/{id}` | Get agent detail (with recent events and config) |
| PATCH | `/api/agents/{id}` | Update agent config |
| DELETE | `/api/agents/{id}` | Delete agent and its events |
| GET | `/api/agents/{id}/export` | Export agent data (JSON or Markdown) |
| GET | `/api/agents/{id}/claude-md` | Read CLAUDE.md from agent working directory |
| PUT | `/api/agents/{id}/claude-md` | Write CLAUDE.md to agent working directory |

**MCP Tools Owned:**

| Tool | Description |
|------|-------------|
| `forge_create_agent` | Create a new agent with configuration |
| `forge_list_agents` | List all agents with status |
| `forge_get_agent` | Get agent details |
| `forge_update_agent` | Update agent configuration |
| `forge_delete_agent` | Delete an agent |

---

### 2. Process Execution

**Purpose:** Spawns and manages Claude Code CLI child processes. Translates AgentConfig into CLI arguments. Owns the process lifecycle.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| ChildProcess | (tokio::process::Child) | CliArgs, McpConfigFile, HooksConfigFile |

Note: This context does NOT own persistent state. Processes are ephemeral. The Agent Management context tracks whether an agent is running.

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `ProcessSpawned` | send_prompt() succeeds | agent_id, pid, cli_args |
| `ProcessExited` | child.wait() completes | agent_id, exit_status |
| `ProcessFailed` | spawn error | agent_id, error_message |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| (none -- this context is triggered by direct function calls, not events) | | |

**Public Interface:**

```rust
pub fn register_agent(state: &AppState, config: AgentConfig) -> Uuid;
pub async fn send_prompt(state: &AppState, agent_id: Uuid, prompt: String) -> Result<()>;
pub async fn kill_agent(state: &AppState, agent_id: Uuid) -> Result<()>;
```

**Database Tables Owned:** None (ephemeral process state only).

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/agents/{id}/prompt` | Send prompt to agent (spawns process) |

**MCP Tools Owned:**

| Tool | Description |
|------|-------------|
| `forge_send_prompt` | Send a prompt to a specific agent |
| `forge_stop_agent` | Stop a running agent's process |

---

### 3. Event Streaming

**Purpose:** The event bus. Receives raw events from process stdout, tags them, broadcasts to subscribers (WebSocket clients, event accumulator, safety engine). This is the shared kernel that most other contexts depend on.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| TaggedEvent | TaggedEvent | event_type, raw JSON, timestamp |
| UsageDelta | (value object) | input_tokens, output_tokens, cache tokens |

**Domain Events Published:**

Every event from Claude Code's stream-json output becomes a TaggedEvent on the broadcast channel. Event types include:

| Event Type | Source | Meaning |
|-----------|--------|---------|
| `system` | Claude init | Session initialization, contains session_id |
| `assistant` | Claude response | LLM response with content blocks and usage |
| `user` | Claude echo | User message echo |
| `tool_use` | Claude action | Tool invocation (Read, Write, Bash, etc.) |
| `tool_result` | Claude action | Tool execution result |
| `result` | Claude done | Final result of the prompt |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| Raw stdout lines | Process Execution (Stream Reader) | Parse JSON, tag with agent_id and timestamp, broadcast |

**Public Interface:**

```rust
pub fn spawn_stream_reader(
    agent_id: Uuid,
    stdout: ChildStdout,
    tx: broadcast::Sender<TaggedEvent>,
) -> JoinHandle<()>;

pub fn extract_usage(event: &Value) -> Option<UsageDelta>;
pub fn extract_session_id(event: &Value) -> Option<String>;

// Subscription (via AppState)
pub fn subscribe_events(&self) -> broadcast::Receiver<TaggedEvent>;
```

**Database Tables Owned:** None (events are written by Persistence context).

**API Endpoints Owned:** None (events flow through WebSocket, owned by Presentation context).

**MCP Resources Owned:**

| Resource URI | Description |
|-------------|-------------|
| `forge://events/stream` | Live event stream subscription |
| `forge://agents/{id}/events` | Historical events for an agent |

---

### 4. Persistence

**Purpose:** Owns SQLite database operations. Schema migrations, batch writes, queries. No business logic -- pure data access.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| StoredAgent | StoredAgent | (deserialized from DB row) |
| StoredEvent | (DB row) | agent_id, event_type, event JSON, timestamp |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `BatchFlushed` | 50 events or 2s timer | event_count, agent_ids |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| All TaggedEvents | Event Streaming | Batch into pending buffer, flush to DB |
| AgentCreated/Updated/Deleted | Agent Management | Immediate write to agents table |

**Public Interface:**

```rust
pub struct Db {
    pub fn open() -> Result<Self>;
    pub fn save_agent(...) -> Result<()>;
    pub fn load_agents() -> Result<Vec<StoredAgent>>;
    pub fn delete_agent(id: Uuid) -> Result<()>;
    pub fn save_events(events: &[TaggedEvent]) -> Result<()>;
    pub fn load_events(agent_id: Uuid, limit: usize, offset: usize) -> Result<Vec<TaggedEvent>>;
    pub fn update_agent_state(id: Uuid, session_id: Option<&str>, usage: &TokenUsage) -> Result<()>;
}
```

**Database Tables Owned:**

| Table | Purpose |
|-------|---------|
| `schema_version` | Migration tracking |
| `agents` | Agent records (shared write with Agent Management) |
| `events` | Event log (append-only, with agent_id FK) |
| `workflows` | (planned) Workflow definitions and state |
| `workflow_steps` | (planned) Individual step records |
| `skills` | (planned) Skill catalog entries |
| `fts_events` | (planned) FTS5 virtual table for event search |

**API Endpoints Owned:** None (data access is internal).

---

### 5. Session History

**Purpose:** Scans Claude Code's native session storage (`~/.claude/projects/`) to provide a browsable history of past sessions, including those NOT created by Forge.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| ProjectDir | ProjectDir | name, original_path, dir_path |
| SessionInfo | SessionInfo | session_id, file_path, timestamps, message_count |

**Domain Events Published:** None (read-only context).

**Domain Events Consumed:** None.

**Public Interface:**

```rust
pub fn list_project_dirs() -> Vec<ProjectDir>;
pub fn list_sessions(working_dir: &str) -> Vec<SessionInfo>;
pub fn list_all_sessions() -> Vec<SessionInfo>;
```

**Database Tables Owned:** None (reads from filesystem only).

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/sessions` | List sessions, optionally filtered by working directory path |
| GET | `/api/sessions/projects` | List all project directories |

**MCP Resources Owned:**

| Resource URI | Description |
|-------------|-------------|
| `forge://sessions` | List all sessions across projects |
| `forge://sessions/{project}` | Sessions for a specific project |

---

### 6. Preset Library

**Purpose:** Provides built-in agent presets (Planner, Reviewer, Bug Hunter, etc.) with curated system prompts, model choices, and configurations.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| AgentPreset | AgentPreset | id, name, description, icon, model, system_prompt, permission_mode |

**Domain Events Published:** None (static data).

**Domain Events Consumed:** None.

**Public Interface:**

```rust
pub fn all_presets() -> Vec<AgentPreset>;
pub fn gitnexus_mcp() -> (String, McpServerConfig);
// Planned:
pub fn get_preset(id: &str) -> Option<AgentPreset>;
pub fn custom_presets(db: &Db) -> Vec<AgentPreset>;
```

**Database Tables Owned:**

| Table | Purpose |
|-------|---------|
| `custom_presets` | (planned) User-defined preset storage |

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/presets` | List all available presets |
| POST | `/api/presets` | (planned) Create custom preset |

**MCP Tools Owned:**

| Tool | Description |
|------|-------------|
| `forge_list_presets` | List available agent presets |
| `forge_create_from_preset` | Create an agent from a preset ID |

---

### 7. Safety and Limits

**Purpose:** Enforces guardrails on agent operations: budget limits, tool restrictions, permission modes, rate limiting. Prevents runaway agents and accidental cost overruns.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| BudgetLedger | (per-agent budget tracking) | spent_usd, limit_usd, remaining_usd |
| RateWindow | (sliding window counter) | request_count, window_start, window_size |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `BudgetWarning` | Agent at 80% of limit | agent_id, spent, limit |
| `BudgetExceeded` | Agent over limit | agent_id, spent, limit |
| `RateLimited` | Too many requests | agent_id, retry_after_ms |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| `assistant` events | Event Streaming | Extract usage, update budget ledger |
| `AgentCreated` | Agent Management | Initialize budget tracking |
| `AgentDeleted` | Agent Management | Clean up budget state |

**Public Interface:**

```rust
// Guards called before process execution
pub fn check_budget(agent: &AgentHandle, global: &ForgeConfig) -> Result<(), ForgeError>;
pub fn check_rate_limit(agent_id: Uuid) -> Result<(), ForgeError>;
pub fn validate_tools(config: &AgentConfig) -> Result<(), ForgeError>;
```

**Database Tables Owned:** None (derived from event stream and agent config).

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/cost` | Get cost summary across all agents |
| GET | `/api/agents/{id}/budget` | (planned) Get budget status for an agent |
| PUT | `/api/config/limits` | (planned) Set global safety limits |

---

### 8. Workflow Orchestration

**Purpose:** Enables multi-agent coordination through structured workflows. DAG-based execution with sequential, parallel, fan-out, and fan-in patterns.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| Workflow | WorkflowDefinition | WorkflowStep, StepDependency, StepConfig |
| WorkflowRun | WorkflowExecution | StepResult, RunStatus, RunTimeline |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `WorkflowStarted` | Workflow begins execution | workflow_id, step_count |
| `WorkflowStepStarted` | Step begins | workflow_id, step_id, agent_id |
| `WorkflowStepCompleted` | Step finishes | workflow_id, step_id, result_summary |
| `WorkflowStepFailed` | Step errors | workflow_id, step_id, error |
| `WorkflowCompleted` | All steps done | workflow_id, duration, total_cost |
| `WorkflowFailed` | Unrecoverable error | workflow_id, failed_step, error |
| `AgentHandoff` | Context transfer | from_agent_id, to_agent_id, context |
| `AgentBroadcast` | Message to all | from_agent_id, message |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| `result` events | Event Streaming | Detect step completion, trigger next steps |
| `ProcessExited` | Process Execution | Handle step process exit |
| `BudgetExceeded` | Safety & Limits | Pause or abort workflow |

**Public Interface:**

```rust
pub async fn create_workflow(definition: WorkflowDefinition) -> Result<WorkflowId>;
pub async fn start_workflow(state: &AppState, id: WorkflowId) -> Result<()>;
pub async fn pause_workflow(id: WorkflowId) -> Result<()>;
pub async fn resume_workflow(state: &AppState, id: WorkflowId) -> Result<()>;
pub async fn cancel_workflow(id: WorkflowId) -> Result<()>;
pub fn get_workflow_status(id: WorkflowId) -> Result<WorkflowStatus>;
```

**Database Tables Owned:**

| Table | Purpose |
|-------|---------|
| `workflows` | Workflow definitions (name, DAG JSON, created_at) |
| `workflow_runs` | Execution instances (workflow_id, status, started_at, completed_at) |
| `workflow_steps` | Step state (run_id, step_id, agent_id, status, result) |

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/workflows` | Create workflow definition |
| GET | `/api/workflows` | List workflows |
| GET | `/api/workflows/{id}` | Get workflow details |
| POST | `/api/workflows/{id}/start` | Start workflow execution |
| POST | `/api/workflows/{id}/pause` | Pause running workflow |
| POST | `/api/workflows/{id}/resume` | Resume paused workflow |
| DELETE | `/api/workflows/{id}` | Delete workflow |
| GET | `/api/workflows/{id}/runs` | List execution history |
| GET | `/api/workflows/{id}/runs/{run_id}` | Get run details with step status |

**MCP Tools Owned:**

| Tool | Description |
|------|-------------|
| `forge_create_workflow` | Define a multi-step workflow |
| `forge_run_workflow` | Execute a workflow |
| `forge_workflow_status` | Check workflow execution status |
| `forge_handoff` | Transfer context from one agent to another |
| `forge_broadcast` | Send message to all agents in a group |

---

### 9. MCP Interface

**Purpose:** Exposes Forge as an MCP server so that external AI agents (Claude Code, VS Code Copilot, etc.) can orchestrate Forge programmatically. Translates between JSON-RPC and internal Forge operations.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| McpSession | (connection state) | transport, capabilities, subscriptions |
| ToolDefinition | (static schema) | name, description, input_schema |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `McpToolInvoked` | External tool call | tool_name, arguments |
| `McpResourceRead` | External resource read | resource_uri |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| All TaggedEvents | Event Streaming | Forward to MCP resource subscriptions |

**Public Interface:**

```rust
// MCP server entry point
pub async fn run_mcp_server(state: AppState, transport: StdioTransport) -> Result<()>;

// Tool registry
pub fn register_tools() -> Vec<ToolDefinition>;
pub fn register_resources() -> Vec<ResourceDefinition>;
pub fn register_prompts() -> Vec<PromptDefinition>;
```

**Database Tables Owned:** None.

**API Endpoints Owned:** None (MCP uses stdio transport, not HTTP).

**MCP Tools Owned:** (All tools -- this context IS the MCP interface. See [MCP_INTERFACE.md](MCP_INTERFACE.md) for the complete catalog.)

---

### 10. Skill Catalog

**Purpose:** Manages a catalog of reusable agent skills (slash commands, recipes, prompt templates). Skills can be built-in, loaded from presets, or imported from the reference repos.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| Skill | SkillEntry | SkillId, name, description, category, prompt_template |
| SkillCategory | (enum) | coding, review, testing, docs, security, workflow |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `SkillInstalled` | Skill added to catalog | skill_id, name, source |
| `SkillInvoked` | Slash command used | skill_id, agent_id, arguments |

**Domain Events Consumed:** None.

**Public Interface:**

```rust
pub fn list_skills(category: Option<SkillCategory>) -> Vec<SkillEntry>;
pub fn get_skill(id: &str) -> Option<SkillEntry>;
pub fn invoke_skill(id: &str, agent_id: Uuid, args: &str) -> Result<String>;
pub fn register_skill(entry: SkillEntry) -> Result<()>;
```

**Database Tables Owned:**

| Table | Purpose |
|-------|---------|
| `skills` | Skill catalog (id, name, description, category, prompt_template, source) |

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/skills` | List skills, optionally filtered by category |
| GET | `/api/skills/{id}` | Get skill details |
| POST | `/api/skills` | Install a custom skill |
| POST | `/api/skills/{id}/invoke` | Invoke a skill for an agent |

**MCP Tools Owned:**

| Tool | Description |
|------|-------------|
| `forge_list_skills` | Browse available skills |
| `forge_invoke_skill` | Run a skill with arguments |

---

### 11. Git Integration

**Purpose:** Provides git-aware features: repository status, diffs, worktree management, branch operations. Enables Forge to understand the code context agents operate in.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| Repository | RepoInfo | path, branch, remote, status |
| Worktree | WorktreeInfo | name, path, branch, is_main |
| Diff | DiffResult | files_changed, insertions, deletions, hunks |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `WorktreeCreated` | Agent gets a worktree | agent_id, worktree_path, branch |
| `WorktreeRemoved` | Worktree cleanup | worktree_path |

**Domain Events Consumed:** None.

**Public Interface:**

```rust
pub fn git_status(working_dir: &Path) -> Result<RepoStatus>;
pub fn git_diff(working_dir: &Path, base: Option<&str>) -> Result<DiffResult>;
pub fn list_worktrees(working_dir: &Path) -> Result<Vec<WorktreeInfo>>;
pub fn create_worktree(working_dir: &Path, name: &str, branch: &str) -> Result<PathBuf>;
pub fn remove_worktree(path: &Path) -> Result<()>;
pub fn git_log(working_dir: &Path, limit: usize) -> Result<Vec<CommitInfo>>;
```

**Database Tables Owned:** None (reads from git directly).

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/agents/{id}/git/status` | Git status for agent's working directory |
| GET | `/api/agents/{id}/git/diff` | Git diff for agent's working directory |
| GET | `/api/agents/{id}/git/log` | Recent commit log |
| GET | `/api/agents/{id}/git/worktrees` | List worktrees |
| POST | `/api/agents/{id}/git/worktrees` | Create a new worktree |
| DELETE | `/api/agents/{id}/git/worktrees/{name}` | Remove a worktree |

**MCP Tools Owned:**

| Tool | Description |
|------|-------------|
| `forge_git_status` | Get git status for a directory |
| `forge_git_diff` | Get git diff |
| `forge_create_worktree` | Create a git worktree for an agent |

---

### 12. Presentation

**Purpose:** The user-facing layer. Owns the embedded SvelteKit frontend, the WebSocket protocol for real-time updates, and the filesystem browse endpoint. Translates between user intent and domain operations.

**Aggregates and Entities:**

| Aggregate | Root Entity | Value Objects |
|-----------|------------|---------------|
| WebSocketSession | (connection) | filter, subscription state |
| ClientMessage | (enum) | Subscribe, Prompt, Ping |
| ServerMessage | (enum) | Event, AgentStatus, Error, Pong |

**Domain Events Published:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `ClientConnected` | WebSocket opened | client_id |
| `ClientDisconnected` | WebSocket closed | client_id |

**Domain Events Consumed:**

| Event | Source | Action |
|-------|--------|--------|
| All TaggedEvents | Event Streaming | Forward matching events to subscribed WebSocket clients |

**Public Interface:**

```rust
// WebSocket handler
pub async fn handle_ws(socket: WebSocket, state: AppState);
pub async fn serve_spa(uri: Uri) -> Response;
pub async fn browse_directory(query: BrowseQuery) -> Result<BrowseResponse>;
```

**Database Tables Owned:** None.

**API Endpoints Owned:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/ws` | WebSocket upgrade endpoint |
| GET | `/api/fs/browse` | Browse filesystem directories |
| GET | `/*` | SPA fallback (serves index.html for client-side routing) |

**WebSocket Protocol:**

Client to Server:
```json
{ "type": "subscribe", "filter": "all" }
{ "type": "subscribe", "filter": { "agent_id": "uuid" } }
{ "type": "prompt", "agent_id": "uuid", "text": "..." }
{ "type": "ping" }
```

Server to Client:
```json
{ "type": "event", "agent_id": "uuid", "event": {...}, "event_type": "assistant" }
{ "type": "agent_status", "agent_id": "uuid", "status": "running" }
{ "type": "error", "message": "..." }
{ "type": "pong" }
```

---

## Anti-Corruption Layers

Anti-corruption layers (ACLs) protect bounded contexts from external or mismatched models.

### ACL 1: Claude Code CLI Output -> Event Streaming

The Claude Code CLI produces stream-json output with its own schema. The Event Streaming context wraps this in `TaggedEvent` to add `agent_id` and `timestamp`, isolating the rest of Forge from Claude's output format.

```
Claude stdout line (raw JSON)
         |
    +----v----+
    | parse   | serde_json::from_str
    +----+----+
         |
    +----v----+
    | tag     | Add agent_id, event_type, timestamp
    +----+----+
         |
    +----v----+
    | broadcast | broadcast::Sender<TaggedEvent>
    +---------+
```

If Claude changes its output format, only `stream.rs` needs to change.

### ACL 2: MCP JSON-RPC -> Internal Operations

The MCP Interface receives JSON-RPC calls with MCP-specific schemas. It translates these to internal Forge operations, never exposing Forge internals to MCP clients.

```
MCP tool call: { method: "tools/call", params: { name: "forge_create_agent", ... } }
         |
    +----v----+
    | validate | Check JSON schema, extract params
    +----+----+
         |
    +----v----+
    | translate | Convert MCP params to CreateAgentRequest
    +----+----+
         |
    +----v----+
    | delegate | Call Agent Management context
    +----+----+
         |
    +----v----+
    | respond | Convert AgentResponse to MCP result JSON
    +---------+
```

### ACL 3: Session File Format -> Session History

Claude Code stores sessions as JSONL files in `~/.claude/projects/`. The Session History context parses these files and produces its own `SessionInfo` model, insulating Forge from Claude's internal file layout.

```
~/.claude/projects/-Users-bm-project/session-id.jsonl
         |
    +----v----+
    | scan    | Read directory, find .jsonl files
    +----+----+
         |
    +----v----+
    | parse   | Read line-by-line, extract metadata
    +----+----+
         |
    +----v----+
    | model   | Produce SessionInfo with normalized fields
    +---------+
```

### ACL 4: Frontend API Types -> Domain Types

The API layer converts between domain types (AgentHandle, AgentConfig) and API DTOs (AgentResponse, CreateAgentRequest). The frontend never sees internal domain structures.

```
AgentHandle (internal, has events VecDeque, temp files, etc.)
         |
    +----v----+
    | project | Select only API-safe fields
    +----+----+
         |
    +----v----+
    | AgentResponse | id, name, status, model, session_id, usage, created_at
    +---------+
```

---

## Ubiquitous Language

Terms used consistently across all contexts.

| Term | Definition |
|------|-----------|
| Agent | A configured Claude Code instance with identity, model, system prompt, and lifecycle |
| Prompt | A user message sent to an agent, triggering a Claude Code process |
| Event | A single line of stream-json output from a Claude Code process, tagged with metadata |
| Session | A Claude Code conversation identified by session_id, enabling --resume continuity |
| Preset | A curated agent configuration template (model, prompt, permissions, tools) |
| Workflow | A multi-step agent coordination plan expressed as a DAG |
| Step | A single node in a workflow DAG, executed by one agent |
| Handoff | Transfer of context from one agent to another within a workflow |
| Broadcast | A message sent from one agent to all others in a group |
| Skill | A reusable prompt template or recipe invocable via slash command |
| Working Directory | The filesystem path where an agent operates (current_dir for Claude CLI) |
| Budget | A spending limit in USD for an agent or globally |
| MCP Server | An external tool server that agents can use (GitNexus, filesystem, etc.) |
| MCP Tool | A callable operation exposed by an MCP server |
| MCP Resource | A readable data source exposed by an MCP server |
| Worktree | A git worktree providing isolated branch context for an agent |
| Tag | Metadata added to a raw event: agent_id, event_type, timestamp |

---

*Next: [DATA_MODEL.md](DATA_MODEL.md) for the complete SQLite schema and data lifecycle.*
