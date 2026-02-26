# Claude Forge -- System Architecture

> C4 model architecture for a single-binary multi-agent Claude Code orchestrator.
> Version 0.2 | Target: ~33K LOC across ~12-15 Rust crates

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Level 1: System Context](#level-1-system-context)
3. [Level 2: Container Diagram](#level-2-container-diagram)
4. [Level 3: Component Diagram](#level-3-component-diagram)
5. [Level 4: Key Code-Level Designs](#level-4-key-code-level-designs)
6. [Cross-Cutting Concerns](#cross-cutting-concerns)
7. [Deployment Model](#deployment-model)
8. [Crate Map](#crate-map)

---

## Design Principles

| Principle | Rationale |
|-----------|-----------|
| Single binary, zero runtime dependencies | Users run `claude-forge` with no Docker, no database server, no language runtime. |
| SQLite WAL for all persistence | One file, crash-safe, concurrent reads, FTS5 for search. |
| Process-per-agent isolation | Each agent is a `claude -p` child process. Forge never interprets LLM output -- it streams, stores, and presents it. |
| Event-sourced core loop | Every agent interaction is an immutable event. State is derived from the event log. |
| Embedded frontend | SvelteKit static build embedded via `rust-embed`. No separate web server. |
| MCP dual-role | Forge consumes external MCP servers (GitNexus, filesystem, etc.) AND exposes itself as an MCP server so other tools can orchestrate it. |

---

## Level 1: System Context

Who interacts with Forge and what are the external systems?

```
                              +---------------------+
                              |     Human User      |
                              | (developer at desk) |
                              +----------+----------+
                                         |
                          Browser (http://127.0.0.1:4173)
                          or CLI (claude-forge --headless)
                                         |
                              +----------v----------+
                              |                     |
                              |    Claude Forge     |
                              |  (single binary)    |
                              |                     |
                              +--+-----+-----+--+--+
                                 |     |     |  |
                    +------------+  +--+--+  |  +------------+
                    |               |     |  |               |
           +--------v------+  +----v--+  |  |  +------------v------+
           | Claude Code   |  | MCP   |  |  |  | ~/.claude/        |
           | CLI processes |  |Servers|  |  |  | projects/         |
           | (child procs) |  |(ext.) |  |  |  | (session history) |
           +---------------+  +-------+  |  |  +-------------------+
                                         |  |
                              +----------v--v-----------+
                              | ~/.claude-forge/        |
                              | forge.db (SQLite WAL)   |
                              +-------------------------+
```

### External Actors

| Actor | Type | Interaction |
|-------|------|-------------|
| Human User | Person | Creates agents, sends prompts, views results via browser or API |
| Claude Code CLI | Software System | Spawned as child process per agent prompt. Produces stream-json events on stdout |
| External MCP Servers | Software System | GitNexus, filesystem, custom servers -- injected into agent config via `--mcp-config` |
| Anthropic API | External Service | Accessed by Claude Code CLI (not by Forge directly). API key or Max subscription auth |
| Local Filesystem | Infrastructure | Working directories, CLAUDE.md files, git repositories that agents operate on |
| Claude Session Store | File System | `~/.claude/projects/` JSONL files -- Forge reads these for the Session Browser |

### Key System Boundaries

- Forge NEVER calls the Anthropic API directly. It always delegates to `claude` CLI processes.
- Forge NEVER interprets or executes LLM-generated code. It streams, stores, and renders.
- Forge owns `~/.claude-forge/`. It reads (but does not write) `~/.claude/projects/`.

---

## Level 2: Container Diagram

The single binary contains four logical containers.

```
+===========================================================================+
|                        claude-forge (single binary)                        |
|                                                                            |
|  +---------------------------+    +----------------------------+           |
|  |     Axum HTTP Server      |    |    Embedded SPA Frontend   |           |
|  |  (REST API + WebSocket)   |    |   (SvelteKit static build  |           |
|  |                           |    |    served via rust-embed)   |           |
|  |  Port 4173 (default)      |    |                            |           |
|  |  /api/* -> handlers       |    |  Routes:                   |           |
|  |  /ws   -> websocket       |    |    / (dashboard)           |           |
|  |  /*    -> SPA fallback    |    |    /agents/:id             |           |
|  +--+------+------+------+--+    |    /agents/:id/edit        |           |
|     |      |      |      |       |    /sessions               |           |
|     |      |      |      |       +----------------------------+           |
|     |      |      |      |                                                 |
|  +--v--+ +-v----+ |  +---v------+                                          |
|  |Agent| |Event | |  |WebSocket |                                          |
|  |Mgr  | |Bus   | |  |Handler   |                                          |
|  +--+--+ +--+---+ |  +----+-----+                                          |
|     |       |      |       |                                                |
|  +--v-------v------v-------v--+    +----------------------------+           |
|  |       Application Core     |    |      MCP Server            |           |
|  |                             |    |   (stdio transport)        |           |
|  |  - Agent lifecycle          |    |                            |           |
|  |  - Process spawning         |    |   Tools: 30+ operations   |           |
|  |  - Event accumulation       |    |   Resources: 10+ URIs     |           |
|  |  - Session scanning         |    |   Prompts: 7+ templates   |           |
|  |  - Cost tracking            |    |                            |           |
|  +-------------+---------------+    +----------------------------+           |
|                |                                                             |
|  +-------------v---------------+                                             |
|  |       SQLite (WAL)          |                                             |
|  |   ~/.claude-forge/forge.db  |                                             |
|  |                             |                                             |
|  |  Tables:                    |                                             |
|  |    schema_version           |                                             |
|  |    agents                   |                                             |
|  |    events                   |                                             |
|  |    workflows (planned)      |                                             |
|  |    skills (planned)         |                                             |
|  |    fts_events (planned)     |                                             |
|  +-----------------------------+                                             |
+===========================================================================+
```

### Container Responsibilities

| Container | Technology | Purpose |
|-----------|-----------|---------|
| Axum HTTP Server | Rust, Axum 0.8, tower-http | REST API for CRUD, WebSocket for real-time streaming, CORS layer |
| Embedded Frontend | Svelte 5, TailwindCSS 4, rust-embed | Single-page application. Built to static files, embedded in binary at compile time |
| Application Core | Rust, DashMap, broadcast channels, tokio | Agent lifecycle, process management, event accumulation, business logic |
| MCP Server | Rust, JSON-RPC over stdio | Exposes Forge capabilities to external AI agents and tools |
| SQLite Storage | rusqlite (bundled), WAL mode | Persistence for agents, events, configuration. FTS5 for search |

---

## Level 3: Component Diagram

Components within the Application Core, grouped by bounded context.

```
+============================================================================+
|                           APPLICATION CORE                                  |
|                                                                             |
|  +------------------+    +-------------------+    +-------------------+     |
|  | Agent Manager    |    | Process Manager   |    | Event System      |     |
|  |                  |    |                   |    |                   |     |
|  | - register_agent |    | - spawn_process   |    | - broadcast tx/rx |     |
|  | - update_agent   |    | - kill_process    |    | - accumulator     |     |
|  | - delete_agent   |    | - build_cli_args  |    | - batch writer    |     |
|  | - restore_from_db|    | - build_mcp_cfg   |    | - usage extractor |     |
|  +--------+---------+    +--------+----------+    +--------+----------+     |
|           |                       |                        |                |
|           v                       v                        v                |
|  +------------------+    +-------------------+    +-------------------+     |
|  | State Store      |    | Stream Reader     |    | WebSocket Hub     |     |
|  |                  |    |                   |    |                   |     |
|  | - DashMap<Uuid,  |    | - stdout line     |    | - client sessions |     |
|  |   AgentHandle>   |    |   reader          |    | - subscribe/      |     |
|  | - ForgeConfig    |    | - JSON parser     |    |   filter          |     |
|  | - Db reference   |    | - event tagging   |    | - event forwarding|     |
|  +--------+---------+    +-------------------+    +-------------------+     |
|           |                                                                 |
|           v                                                                 |
|  +------------------+    +-------------------+    +-------------------+     |
|  | Preset Library   |    | Session Scanner   |    | Safety Engine     |     |
|  |                  |    |                   |    |  (planned)        |     |
|  | - 9 built-in     |    | - project dir     |    |                   |     |
|  |   presets         |    |   listing         |    | - budget limits   |     |
|  | - GitNexus MCP   |    | - JSONL parsing   |    | - tool allowlists |     |
|  |   injection      |    | - session metadata|    | - permission modes|     |
|  +------------------+    +-------------------+    | - rate limiting   |     |
|                                                   +-------------------+     |
|                                                                             |
|  +------------------+    +-------------------+    +-------------------+     |
|  | Workflow Engine  |    | Skill Catalog     |    | Git Integration   |     |
|  |  (planned)       |    |  (planned)        |    |  (planned)        |     |
|  |                  |    |                   |    |                   |     |
|  | - DAG execution  |    | - skill registry  |    | - status / diff   |     |
|  | - step sequencing|    | - slash commands  |    | - worktree mgmt   |     |
|  | - handoff/bcast  |    | - skill browser   |    | - branch ops      |     |
|  +------------------+    +-------------------+    +-------------------+     |
+============================================================================+
```

### Component Interfaces

| Component | Depends On | Depended On By | Interface Type |
|-----------|-----------|---------------|---------------|
| Agent Manager | State Store, Db | API handlers, Process Manager | Function calls |
| Process Manager | Agent Manager, Stream Reader | API handlers (send_prompt) | Async function calls |
| Event System | State Store, Db | WebSocket Hub, Agent Manager | broadcast::channel |
| State Store | Db | Everything | Arc<AppStateInner> shared reference |
| Stream Reader | -- | Process Manager, Event System | tokio task + broadcast sender |
| WebSocket Hub | Event System | Frontend clients | WebSocket protocol |
| Preset Library | -- | Agent Manager, API | Pure data |
| Session Scanner | Filesystem | API handlers | Synchronous file I/O |
| Safety Engine | State Store | Process Manager, API | Middleware / guards |
| Workflow Engine | Agent Manager, Event System | API, MCP Server | Async task graph |

---

## Level 4: Key Code-Level Designs

### 4.1 Agent Manager

The Agent Manager is the core of Forge. It maintains the in-memory agent registry and coordinates with the database.

```
                    CreateAgentRequest
                           |
                           v
                    +------+------+
                    | register_   |
                    | agent()     |
                    +------+------+
                           |
              +------------+------------+
              |                         |
              v                         v
    +---------+--------+     +----------+---------+
    | DashMap.insert()  |     | Db.save_agent()    |
    | (in-memory state) |     | (SQLite persist)   |
    +------------------+     +--------------------+

    State Machine per Agent:

    +-------+   send_prompt()   +---------+   process exits   +-------+
    | Idle  | ----------------> | Running | ----------------> | Idle  |
    +-------+                   +---------+                   +-------+
        |                           |
        | delete_agent()            | error / kill
        v                           v
    +---------+                 +---------+
    | Removed |                 | Error   |
    +---------+                 +---------+
```

**Key data structures (current):**

```rust
// In-memory agent handle (state.rs)
pub struct AgentHandle {
    pub id: Uuid,
    pub name: String,
    pub config: AgentConfig,          // Full agent configuration
    pub status: AgentStatus,          // Idle | Running | Stopped | Error
    pub session_id: Option<String>,   // Claude Code session for --resume
    pub usage: TokenUsage,            // Accumulated token/cost counters
    pub events: VecDeque<TaggedEvent>, // Ring buffer, max 10K events
    pub created_at: DateTime<Utc>,
    pub temp_files: Vec<TempPath>,    // MCP/hooks config files, cleaned on exit
}

// Thread-safe state (shared via Arc)
pub struct AppStateInner {
    pub agents: DashMap<Uuid, AgentHandle>,  // Lock-free concurrent map
    pub event_tx: broadcast::Sender<TaggedEvent>,  // Fanout channel
    pub config: RwLock<ForgeConfig>,
    pub db: Db,
}
```

### 4.2 Safety Engine (Planned)

The Safety Engine wraps every agent operation with configurable guardrails.

```
    User Request
         |
         v
    +----+----+
    | Budget  |  Is agent over its max_budget_usd?
    | Check   |  Is global budget exceeded?
    +----+----+
         |
         v
    +----+----+
    | Tool    |  Are requested tools in the allowlist?
    | Filter  |  Are any in the denylist?
    +----+----+
         |
         v
    +----+----+
    | Perm.   |  Does permission_mode allow this action?
    | Mode    |  (plan / acceptEdits / dontAsk / bypassPermissions)
    +----+----+
         |
         v
    +----+----+
    | Rate    |  Has this agent exceeded turns/minute?
    | Limit   |  Has this user exceeded requests/hour?
    +----+----+
         |
         v
    Process Manager (spawn or reject)
```

**Safety rules hierarchy:**
1. Global limits (set in ForgeConfig) -- cannot be overridden by agents
2. Agent-level limits (set in AgentConfig) -- apply to one agent
3. Session-level limits (max_turns, max_budget_usd) -- apply per prompt session

### 4.3 Workflow Engine (Planned)

The Workflow Engine enables multi-agent coordination through directed acyclic graphs.

```
    Workflow Definition (JSON/YAML)
         |
         v
    +----+----+
    | Parser  |  Validate DAG, check for cycles
    +----+----+
         |
         v
    +----+----+-----+-----+-----+
    | Step 1  | Step 2  | Step 3  |    (parallel where no dependencies)
    | Agent A | Agent B | Agent C |
    +----+----+----+----+----+----+
         |         |         |
         v         v         v
    +----+---------+---------+----+
    |       Result Aggregator      |
    +----+------------------------+
         |
         v
    +----+----+
    | Step 4  |   (depends on steps 1-3)
    | Agent D |
    +----+----+
         |
         v
    Workflow Complete (events emitted for each step)
```

**Coordination patterns:**

| Pattern | Description | Use Case |
|---------|------------|----------|
| Sequential | A then B then C | Plan -> Implement -> Review |
| Parallel fan-out | A spawns B,C,D concurrently | Multi-file refactoring |
| Fan-in | Wait for B,C,D then run E | Aggregate reviews |
| Supervisor | A delegates, monitors, retries | Complex multi-step tasks |
| Handoff | A passes context to B | Planner hands off to coder |
| Broadcast | A sends message to all others | "Stop, plan changed" |

---

## Cross-Cutting Concerns

### Logging

```
Technology: tracing + tracing-subscriber with EnvFilter
Default level: info
Override: RUST_LOG=debug (or RUST_LOG=claude_forge::process=trace)

Structured fields on all log lines:
  - agent_id (when applicable)
  - event_type (for event processing)
  - endpoint (for API requests)

Log destinations:
  - stderr (default, human-readable format)
  - JSON format available via FORGE_LOG_FORMAT=json (planned)
```

### Error Handling

```
Error type: ForgeError (src/error.rs)
Variants:
  - AgentNotFound(Uuid)       -> 404
  - AgentNotRunning(Uuid)     -> 400
  - ProcessSpawnFailed(String) -> 500
  - ConfigError(String)       -> 400
  - Internal(String)          -> 500

Pattern: ForgeError implements IntoResponse for Axum.
All errors return JSON: { "error": "human-readable message" }

Future additions:
  - WorkflowError(WorkflowId, String)
  - BudgetExceeded(Uuid, f64)
  - RateLimited(Uuid)
  - McpError(String)
```

### Configuration

```
Configuration layers (lowest to highest priority):
  1. Compiled defaults (ForgeConfig::default())
  2. Config file: ~/.claude-forge/config.toml (planned)
  3. Environment variables: FORGE_PORT, FORGE_BIND, etc. (planned)
  4. CLI arguments: --port, --bind, --no-open

Agent configuration:
  - Stored as JSON blob in agents.config column
  - Full AgentConfig struct with 20+ fields
  - Option<Option<T>> pattern for nullable PATCH fields in UpdateAgentRequest
```

### Persistence

```
Database: SQLite with WAL journal mode
Location: ~/.claude-forge/forge.db
Connection: Arc<Mutex<Connection>> (single writer, safe for WAL reads)

Write strategy:
  - Agent CRUD: immediate write on each operation
  - Events: batched (50 events or 2-second timer, whichever comes first)
  - Agent state (session_id, usage): written alongside event batch flush

Read strategy:
  - On startup: load all agents + last 10K events per agent into memory
  - Hot path: all reads served from DashMap (in-memory)
  - Cold path: historical event queries go to SQLite

Migration:
  - schema_version table tracks current version
  - Sequential migration functions (v0->v1, v1->v2, etc.)
  - Runs automatically on startup
```

---

## Deployment Model

### Single Binary Architecture

```
Build pipeline:

  frontend/               cargo build
  (SvelteKit) ----+         |
       |          |         |
  pnpm build      |    +---------+
       |          +--->| rustc   |----> claude-forge (single binary)
       v          |    | + embed |      (~15-25 MB)
  frontend/build/ |    +---------+
  (static files)--+         |
                        rust-embed
                     compiles assets
                     into the binary
```

### Runtime Requirements

| Requirement | Notes |
|-------------|-------|
| OS | macOS (primary), Linux (supported), Windows (untested) |
| `claude` CLI | Must be on PATH. This is the only external dependency |
| Filesystem | Write access to `~/.claude-forge/` |
| Network | Loopback (127.0.0.1) by default; no outbound connections from Forge itself |
| Memory | ~50MB base + ~10MB per active agent (event buffer) |
| Disk | SQLite DB grows ~1KB per event; typical session: 1-5 MB |

### Process Model

```
    claude-forge (PID 1000)
         |
         |-- tokio runtime (multi-threaded)
         |     |-- HTTP listener task
         |     |-- WebSocket handler tasks (1 per client)
         |     |-- Event accumulator task (1 global)
         |     |-- Process watcher tasks (1 per running agent)
         |
         |-- child: claude -p "..." --output-format stream-json (PID 1001)
         |-- child: claude -p "..." --output-format stream-json (PID 1002)
         |-- child: claude -p "..." --output-format stream-json (PID 1003)
         ...
```

### Startup Sequence

```
1. Parse CLI arguments (clap)
2. Initialize tracing subscriber
3. Open SQLite database (~/.claude-forge/forge.db)
4. Run schema migrations
5. Create AppState (DashMap, broadcast channel, config)
6. Restore agents from database into memory
7. Spawn event accumulator background task
8. Build Axum router (API routes + WebSocket + SPA fallback)
9. Bind TCP listener
10. Open browser (unless --no-open)
11. Serve requests until Ctrl+C
12. Graceful shutdown (drain in-flight requests)
```

---

## Crate Map

Target workspace structure for ~33K LOC across ~12-15 crates. Follows the principle: if a crate has less than 500 LOC and only one dependent, keep it merged.

```
claude-forge/                    (workspace root)
|
+-- Cargo.toml                   (workspace definition, resolver = "2")
|
+-- crates/
|   +-- forge-core/              (~4K LOC) Agent model, config, error types, shared types
|   +-- forge-db/                (~3K LOC) SQLite schema, migrations, queries, FTS5
|   +-- forge-events/            (~2K LOC) Event types, bus, accumulator, replay
|   +-- forge-process/           (~2K LOC) Claude CLI spawning, stream parsing, arg building
|   +-- forge-api/               (~3K LOC) Axum handlers, REST endpoints, request/response DTOs
|   +-- forge-ws/                (~1.5K LOC) WebSocket handler, subscription filtering
|   +-- forge-mcp/               (~3K LOC) MCP server implementation (tools, resources, prompts)
|   +-- forge-workflow/          (~3K LOC) DAG engine, step execution, coordination patterns
|   +-- forge-safety/            (~1.5K LOC) Budget checks, rate limiting, permission guards
|   +-- forge-git/               (~2K LOC) Git status, diff, worktree management
|   +-- forge-sessions/          (~1K LOC) Session scanner, JSONL parser
|   +-- forge-skills/            (~1.5K LOC) Skill catalog, slash command registry
|   +-- forge-presets/           (~1K LOC) Built-in agent presets, system prompts
|   +-- forge-assets/            (~0.5K LOC) rust-embed SPA serving
|
+-- forge-bin/                   (~1K LOC) Main binary: CLI parsing, wiring, startup
|
+-- frontend/                    (SvelteKit project, not a Rust crate)
    +-- src/
    +-- static/
    +-- build/                   (generated, embedded by forge-assets)
```

### Dependency Graph (no circular dependencies)

```
forge-bin
  +-- forge-api
  |     +-- forge-core
  |     +-- forge-process
  |     +-- forge-events
  |     +-- forge-safety
  |     +-- forge-sessions
  |     +-- forge-presets
  +-- forge-ws
  |     +-- forge-core
  |     +-- forge-events
  +-- forge-mcp
  |     +-- forge-core
  |     +-- forge-events
  |     +-- forge-workflow
  |     +-- forge-process
  +-- forge-workflow
  |     +-- forge-core
  |     +-- forge-events
  |     +-- forge-process
  +-- forge-db
  |     +-- forge-core
  |     +-- forge-events
  +-- forge-git
  |     +-- forge-core
  +-- forge-skills
  |     +-- forge-core
  +-- forge-assets
  +-- forge-safety
        +-- forge-core
```

### Shared Kernel: forge-core

The `forge-core` crate is the shared kernel. It contains:
- `AgentConfig`, `AgentStatus`, `TokenUsage` -- core domain types
- `ForgeError` -- error enum used everywhere
- `McpServerConfig`, `HooksConfig` -- configuration types
- `PermissionMode` -- permission enum
- No dependencies on any other forge-* crate (leaf node in the graph)

This prevents circular dependencies because every crate can depend on `forge-core` without creating cycles.

---

*Next: [BOUNDED_CONTEXTS.md](BOUNDED_CONTEXTS.md) for detailed domain-driven design.*
