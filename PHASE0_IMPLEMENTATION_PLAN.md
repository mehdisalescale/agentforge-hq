# Claude Forge -- Phase 0 Implementation Plan

> Foundation Build. From empty workspace to a running binary with working agent CRUD.
> 4 weeks. 8 crates. 2 approval gates. Red-green-refactor throughout.

---

## Guiding Principles

1. **Study before building.** Every step lists reference repos to read first.
2. **Full types now, stub logic later.** Define ALL event variants, error types, and schema upfront. Implement only what Phase 0 needs.
3. **Prove the full stack works.** By end of Phase 0: create an agent via API -> stored in SQLite -> visible in Svelte UI -> events flow through WebSocket. Not a skeleton. A working vertical slice.
4. **2 approval gates, not 7.** Get sign-off on crate boundaries and schema DDL. Everything else: just build it.
5. **TDD where it matters.** Test event bus behavior, batch writer timing, migration ordering, repository CRUD. Don't test Rust's standard library.

---

## Reference Material (Read Before Starting)

Before writing any code, study these:

| What | Where | Why |
|------|-------|-----|
| Existing Forge process spawning | Old Forge `src/main.rs` | WebSocket streaming, process management patterns |
| Event batching patterns | `refrence-repo/claude-code-tools` | Session management, Tantivy FTS, batch write patterns |
| System architecture spec | `forge-project/03-architecture/SYSTEM_ARCHITECTURE.md` | Crate map, event types, state management |
| Data model spec | `forge-project/03-architecture/DATA_MODEL.md` | Full SQLite schema reference |
| API design spec | `forge-project/03-architecture/API_DESIGN.md` | Route structure, request/response contracts |
| Tech stack spec | `forge-project/05-engineering/TECH_STACK.md` | Exact versions, dependency choices |
| CI/CD spec | `forge-project/05-engineering/CI_CD.md` | Build pipeline, lint rules, binary targets |
| Coding standards | `forge-project/05-engineering/CODING_STANDARDS.md` | Naming, error handling, testing conventions |

---

## Crate Layout

```
claude-forge/                   # New repo root
  Cargo.toml                    # Workspace manifest
  Makefile                      # Build orchestration
  .github/workflows/ci.yml     # Basic CI (test + clippy + fmt)
  crates/
    forge-core/                 # Types, events, errors, traits
    forge-db/                   # SQLite, migrations, repositories
    forge-api/                  # Axum server, routes, middleware, WebSocket
    forge-agent/                # Agent types, presets, validation, CRUD
    forge-process/              # Process spawning types (stubs for Phase 1)
    forge-safety/               # Safety types (stubs for Phase 4)
    forge-mcp/                  # MCP protocol types (stubs for Phase 4)
    forge-app/                  # Binary entry point, wiring, CLI, static serving
  frontend/                     # SvelteKit app
    src/
      routes/
        +layout.svelte          # Sidebar nav, status bar
        +page.svelte            # Dashboard
        agents/+page.svelte     # Agent list + create
        sessions/+page.svelte   # Sessions (empty shell)
        workflows/+page.svelte  # Workflows (empty shell)
        skills/+page.svelte     # Skills (empty shell)
        settings/+page.svelte   # Settings (empty shell)
      lib/
        stores/
          websocket.ts          # WebSocket connection store
          agents.ts             # Agent CRUD store
        types/
          index.ts              # Shared TypeScript types matching Rust
        components/
          Sidebar.svelte
          StatusBar.svelte
          AgentCard.svelte
    svelte.config.js
    tailwind.config.js
    package.json
  migrations/
    0001_init.sql               # Full schema for all phases
```

### Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/zixelfreelance/claude-forge"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Web framework
axum = { version = "0.8", features = ["ws", "macros"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "fs"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Database
rusqlite = { version = "0.32", features = ["bundled", "vtab", "fts5"] }

# Types and utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Embedding
rust-embed = { version = "8", features = ["mime-guess"] }

# Cross-thread
crossbeam-channel = "0.5"
```

> **APPROVAL GATE 1**: Review crate layout, workspace dependencies, and directory structure before proceeding.

---

## Step 1: Core Types, Events, and Error Handling (forge-core)

### Study First
- `refrence-repo/claude-code-tools/src/` -- how they structure types
- `forge-project/03-architecture/SYSTEM_ARCHITECTURE.md` -- ForgeEvent definitions
- `forge-project/03-architecture/EVENT_SYSTEM.md` -- event flow design

### Error Types

Establish the error hierarchy used across all crates:

```rust
// crates/forge-core/src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),

    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Event bus error: {0}")]
    EventBus(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Internal(String),
}

pub type ForgeResult<T> = Result<T, ForgeError>;
```

### ID Types

Newtype wrappers for type safety:

```rust
// crates/forge-core/src/ids.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId(pub Uuid);
```

### ForgeEvent Enum (Full, Not Toy)

Define ALL variants now. Phase 0 only emits a few, but the enum is complete:

```rust
// crates/forge-core/src/events.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ForgeEvent {
    // System lifecycle
    SystemStarted { version: String, timestamp: DateTime<Utc> },
    SystemStopped { timestamp: DateTime<Utc> },
    Heartbeat { timestamp: DateTime<Utc> },

    // Agent lifecycle (used in Phase 0 for CRUD)
    AgentCreated { agent_id: AgentId, name: String, timestamp: DateTime<Utc> },
    AgentUpdated { agent_id: AgentId, name: String, timestamp: DateTime<Utc> },
    AgentDeleted { agent_id: AgentId, timestamp: DateTime<Utc> },

    // Process lifecycle (used in Phase 1)
    ProcessStarted { session_id: SessionId, agent_id: AgentId, timestamp: DateTime<Utc> },
    ProcessOutput { session_id: SessionId, kind: OutputKind, content: String, timestamp: DateTime<Utc> },
    ProcessCompleted { session_id: SessionId, exit_code: i32, timestamp: DateTime<Utc> },
    ProcessFailed { session_id: SessionId, error: String, timestamp: DateTime<Utc> },

    // Session lifecycle (used in Phase 1)
    SessionCreated { session_id: SessionId, agent_id: AgentId, directory: String, timestamp: DateTime<Utc> },
    SessionResumed { session_id: SessionId, timestamp: DateTime<Utc> },

    // Workflow lifecycle (used in Phase 2)
    WorkflowStarted { workflow_id: WorkflowId, timestamp: DateTime<Utc> },
    WorkflowStepCompleted { workflow_id: WorkflowId, step: u32, timestamp: DateTime<Utc> },
    WorkflowCompleted { workflow_id: WorkflowId, timestamp: DateTime<Utc> },
    WorkflowFailed { workflow_id: WorkflowId, error: String, timestamp: DateTime<Utc> },

    // Safety events (used in Phase 4)
    CircuitBreakerTripped { agent_id: AgentId, reason: String, timestamp: DateTime<Utc> },
    BudgetWarning { current_cost: f64, limit: f64, timestamp: DateTime<Utc> },
    BudgetExceeded { current_cost: f64, limit: f64, timestamp: DateTime<Utc> },

    // Generic error
    Error { message: String, context: Option<String>, timestamp: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputKind {
    Assistant,
    ToolUse,
    ToolResult,
    Thinking,
    Result,
}
```

### EventBus

```rust
// crates/forge-core/src/event_bus.rs

pub struct EventBus {
    sender: broadcast::Sender<ForgeEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self;
    pub fn emit(&self, event: ForgeEvent) -> ForgeResult<()>;
    pub fn subscribe(&self) -> broadcast::Receiver<ForgeEvent>;
}

/// Trait for anything that can receive events (DB writer, WebSocket, logger)
pub trait EventSink: Send + Sync {
    fn handle(&self, event: &ForgeEvent);
}
```

### TDD Targets (Red-Green)

| Test | What It Proves |
|------|---------------|
| `emit_event_received_by_subscriber` | Basic bus works |
| `multiple_subscribers_all_receive` | Broadcast, not unicast |
| `slow_subscriber_gets_lagged_error` | Bus doesn't block on slow consumers |
| `event_serializes_to_json_roundtrip` | Serde tag+content format works |
| `all_event_variants_serialize` | No variant is accidentally missing serde |

---

## Step 2: Database Schema and Batch Writer (forge-db)

### Study First
- `forge-project/03-architecture/DATA_MODEL.md` -- full schema
- `refrence-repo/claude-code-tools` -- how they handle session storage
- Existing Forge `src/db.rs` -- batch write pattern (50 events / 2 seconds)

### Schema (0001_init.sql)

```sql
-- Claude Forge Schema v1
-- Designed for ALL phases. Phase 0 uses agents, sessions, events.

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Agents
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    model TEXT NOT NULL DEFAULT 'claude-sonnet-4-20250514',
    system_prompt TEXT,
    allowed_tools TEXT,          -- JSON array
    max_turns INTEGER,
    use_max BOOLEAN NOT NULL DEFAULT 0,
    preset TEXT,
    config_json TEXT,            -- JSON object for extensible config
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sessions
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    claude_session_id TEXT,      -- Claude Code's internal session ID for --resume
    directory TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'running', 'completed', 'failed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Events (the core event log, append-only)
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    data_json TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);

-- Workflows (Phase 2)
CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    definition_json TEXT NOT NULL,   -- workflow DSL (steps, conditions, etc.)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Workflow runs (Phase 2)
CREATE TABLE IF NOT EXISTS workflow_runs (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    current_step INTEGER DEFAULT 0,
    result_json TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Skills (Phase 2)
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT,
    subcategory TEXT,
    content TEXT NOT NULL,           -- the skill template/prompt
    source_repo TEXT,                -- which reference repo it came from
    parameters_json TEXT,            -- JSON schema for skill parameters
    examples_json TEXT,              -- usage examples
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Skills FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS skills_fts USING fts5(
    name, description, category, content,
    content=skills,
    content_rowid=rowid
);

-- Sessions FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(
    directory, status,
    content=sessions,
    content_rowid=rowid
);

-- Events FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(
    event_type, data_json,
    content=events,
    content_rowid=rowid
);

-- Schedules (Phase 4)
CREATE TABLE IF NOT EXISTS schedules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cron_expr TEXT NOT NULL,
    job_type TEXT NOT NULL CHECK (job_type IN ('agent', 'workflow', 'report')),
    job_config_json TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    run_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Audit log (Phase 5)
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    actor TEXT NOT NULL DEFAULT 'system',
    action TEXT NOT NULL,
    target_type TEXT,
    target_id TEXT,
    details_json TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);

-- Config (hierarchical: default < global < project < agent)
CREATE TABLE IF NOT EXISTS config (
    scope TEXT NOT NULL,             -- 'default', 'global', 'project:<path>', 'agent:<id>'
    key TEXT NOT NULL,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (scope, key)
);

INSERT INTO schema_version (version) VALUES (1);
```

> **APPROVAL GATE 2**: Review this schema DDL before creating the migration file. This schema serves ALL phases -- changes later are costly.

### Migration Runner

```rust
// crates/forge-db/src/migrations.rs

pub struct Migrator { /* ... */ }

impl Migrator {
    pub fn new(conn: &Connection) -> Self;
    pub fn current_version(&self) -> ForgeResult<u32>;
    pub fn apply_pending(&self) -> ForgeResult<u32>;  // returns number applied
}
```

### Batch Writer

This is the critical piece Cursor's plan missed. Events accumulate in a channel and flush to SQLite in batches:

```rust
// crates/forge-db/src/batch_writer.rs

pub struct BatchWriter {
    sender: crossbeam_channel::Sender<ForgeEvent>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl BatchWriter {
    /// Spawn a dedicated writer thread.
    /// Flushes when: 50 events accumulated OR 2 seconds elapsed, whichever comes first.
    pub fn spawn(conn: Connection) -> Self;

    /// Queue an event for batch writing. Non-blocking.
    pub fn write(&self, event: ForgeEvent) -> ForgeResult<()>;

    /// Flush remaining events and shut down the writer thread.
    pub fn shutdown(self) -> ForgeResult<()>;
}
```

Writer thread pseudocode:
```
loop {
    select! {
        recv(channel) -> event => {
            buffer.push(event);
            if buffer.len() >= 50 {
                flush_to_db(&buffer);
                buffer.clear();
            }
        }
        recv(tick_2s) -> _ => {
            if !buffer.is_empty() {
                flush_to_db(&buffer);
                buffer.clear();
            }
        }
    }
}
```

### Repositories

```rust
// crates/forge-db/src/repos/agents.rs

pub struct AgentRepo { /* conn reference */ }

impl AgentRepo {
    pub fn create(&self, agent: &NewAgent) -> ForgeResult<Agent>;
    pub fn get(&self, id: &AgentId) -> ForgeResult<Agent>;
    pub fn list(&self) -> ForgeResult<Vec<Agent>>;
    pub fn update(&self, id: &AgentId, update: &UpdateAgent) -> ForgeResult<Agent>;
    pub fn delete(&self, id: &AgentId) -> ForgeResult<()>;
}
```

### TDD Targets

| Test | What It Proves |
|------|---------------|
| `migration_applies_cleanly` | Schema SQL is valid |
| `migration_is_idempotent` | Running twice doesn't fail |
| `migration_version_tracked` | schema_version table updated |
| `agent_crud_roundtrip` | Insert -> read -> fields match |
| `agent_name_unique` | Duplicate name fails with Validation error |
| `fts5_tables_exist` | FTS5 virtual tables created |
| `fts5_skill_search` | Insert skill, search by keyword, found |
| `batch_writer_flushes_at_50` | Buffer limit triggers flush |
| `batch_writer_flushes_at_2s` | Timer triggers flush for small batches |
| `batch_writer_shutdown_flushes` | Remaining events written on shutdown |
| `event_persisted_with_correct_type` | ForgeEvent -> events table -> correct event_type column |

---

## Step 3: Agent Types and Presets (forge-agent)

### Study First
- `refrence-repo/awesome-claude-code-subagents` -- 127+ agent definitions, model routing
- `refrence-repo/claude-code-templates` -- 100+ agent templates
- Existing Forge agent presets

### Agent Model

```rust
// crates/forge-agent/src/lib.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub use_max: bool,
    pub preset: Option<AgentPreset>,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentPreset {
    CodeWriter,
    Reviewer,
    Tester,
    Debugger,
    Architect,
    Documenter,
    SecurityAuditor,
    Refactorer,
    Explorer,
}

impl AgentPreset {
    /// Returns the default system prompt and tool configuration for this preset.
    pub fn defaults(&self) -> PresetDefaults;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAgent {
    pub name: String,
    pub model: Option<String>,          // defaults to claude-sonnet-4-20250514
    pub system_prompt: Option<String>,
    pub preset: Option<AgentPreset>,
    // ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgent {
    pub name: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<Option<String>>,  // Option<Option<T>> for nullable PATCH
    // ...
}
```

### Validation

```rust
// crates/forge-agent/src/validation.rs

pub fn validate_new_agent(agent: &NewAgent) -> ForgeResult<()>;
    // - name: 1-100 chars, alphanumeric + hyphens + underscores
    // - model: must be in allowed model list
    // - system_prompt: max 100KB
```

### TDD Targets

| Test | What It Proves |
|------|---------------|
| `preset_defaults_all_have_system_prompt` | Every preset returns a non-empty prompt |
| `validate_rejects_empty_name` | Validation catches empty string |
| `validate_rejects_long_name` | Validation catches > 100 chars |
| `new_agent_uses_default_model` | Omitting model gets sonnet default |

---

## Step 4: API Server and WebSocket (forge-api)

### Study First
- `forge-project/03-architecture/API_DESIGN.md` -- full route spec
- Existing Forge Axum handlers -- WebSocket upgrade pattern
- `refrence-repo/claude-code-hub` -- API proxy patterns

### Routes

```
GET  /api/v1/health              -> { status, version, uptime, db_ok }
GET  /api/v1/agents              -> Agent[]
POST /api/v1/agents              -> Agent (created)
GET  /api/v1/agents/:id          -> Agent
PUT  /api/v1/agents/:id          -> Agent (updated)
DELETE /api/v1/agents/:id        -> 204
GET  /api/v1/ws                  -> WebSocket upgrade
```

### WebSocket Behavior

On connect:
1. Subscribe to EventBus
2. Send `SystemStarted` event as first message
3. Stream all ForgeEvents as JSON to client
4. Heartbeat every 30 seconds
5. On disconnect: drop subscription (automatic via broadcast::Receiver drop)

### Middleware Stack

```rust
Router::new()
    .nest("/api/v1", api_routes)
    .fallback(static_files)         // SPA: serve index.html for non-API routes
    .layer(CorsLayer::restrictive())
    .layer(TraceLayer::new_for_http())
    .layer(RequestIdLayer::new())
```

### Shared Application State

```rust
// crates/forge-api/src/state.rs

pub struct AppState {
    pub event_bus: Arc<EventBus>,
    pub db: Arc<DbPool>,           // Connection pool or mutex-wrapped connection
    pub agent_repo: Arc<AgentRepo>,
    pub batch_writer: Arc<BatchWriter>,
}
```

### TDD Targets

| Test | What It Proves |
|------|---------------|
| `health_returns_200` | Server starts, health check works |
| `health_includes_version` | Response has version field |
| `create_agent_returns_201` | POST with valid body succeeds |
| `create_agent_invalid_returns_422` | POST with empty name fails with validation error |
| `get_agent_not_found_returns_404` | Unknown ID returns 404 |
| `list_agents_empty` | GET /agents returns [] when no agents |
| `crud_roundtrip` | Create -> get -> update -> list -> delete |
| `unknown_route_returns_404` | /api/v1/nope returns 404 |

---

## Step 5: Frontend Shell (SvelteKit + Svelte 5 + Tailwind 4)

### Study First
- Existing Forge frontend -- layout, component patterns, Svelte 5 runes
- `refrence-repo/1code` -- multi-agent desktop UI patterns
- `refrence-repo/claude-code-webui` -- web UI for CLI with streaming

### Tech

- SvelteKit with `adapter-static`
- Svelte 5 runes: `$state`, `$derived`, `$effect`
- TailwindCSS 4
- TypeScript
- pnpm (path: `~/.local/share/mise/installs/node/22/bin/npx pnpm`)

### Layout

```
+------+----------------------------------+
|      |         Top Status Bar           |
| Side |----------------------------------|
| bar  |                                  |
|      |         Main Content             |
| Nav  |                                  |
|      |                                  |
|      |                                  |
+------+----------------------------------+
```

Sidebar links: Dashboard, Agents, Sessions, Workflows, Skills, Settings.
Status bar: WebSocket connection indicator (green/red dot), Forge version, uptime.

### WebSocket Store

```typescript
// frontend/src/lib/stores/websocket.ts

interface WebSocketStore {
    connected: boolean;
    lastEvent: ForgeEvent | null;
    events: ForgeEvent[];       // rolling buffer, last 100
}
```

Connects to `/api/v1/ws` on mount. Reconnects with exponential backoff on disconnect.

### Agent Store

```typescript
// frontend/src/lib/stores/agents.ts

interface AgentStore {
    agents: Agent[];
    loading: boolean;
    error: string | null;
}

// Functions
async function loadAgents(): Promise<void>;
async function createAgent(agent: NewAgent): Promise<Agent>;
async function updateAgent(id: string, update: UpdateAgent): Promise<Agent>;
async function deleteAgent(id: string): Promise<void>;
```

### Pages

| Page | Content (Phase 0) |
|------|-------------------|
| `/` (Dashboard) | "Claude Forge" heading, version, WebSocket status, event count |
| `/agents` | Agent list (cards), create button, create/edit form |
| `/sessions` | Empty shell: "Sessions -- coming in Phase 1" |
| `/workflows` | Empty shell: "Workflows -- coming in Phase 2" |
| `/skills` | Empty shell: "Skills -- coming in Phase 2" |
| `/settings` | Empty shell: "Settings -- coming in Phase 5" |

### What's NOT Empty

The `/agents` page is **fully functional** in Phase 0:
- List all agents as cards
- Create new agent (form with name, model, preset selector, system prompt)
- Edit existing agent
- Delete with confirmation
- Real-time updates via WebSocket (agent created elsewhere shows up immediately)

### Checks

- `svelte-check` passes
- `pnpm build` produces static output in `frontend/build/`
- Vitest test for WebSocket store: mock WebSocket, verify reconnection

---

## Step 6: Binary Assembly (forge-app)

### Study First
- Existing Forge `src/main.rs` -- rust-embed serving pattern
- `forge-project/05-engineering/CI_CD.md` -- build.rs strategy

### CLI

```rust
// crates/forge-app/src/main.rs

#[derive(Parser)]
#[command(name = "forge", about = "Claude Forge - Multi-agent orchestrator")]
struct Cli {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value = "4173")]
    port: u16,

    /// Data directory for SQLite database
    #[arg(long, default_value = "~/.claude-forge")]
    data_dir: PathBuf,
}
```

### Static File Serving

```rust
// rust-embed for frontend
#[derive(RustEmbed)]
#[folder = "frontend/build"]
struct FrontendAssets;

// Fallback handler: serve static files or index.html for SPA routes
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    // Try exact file match first (JS, CSS, images)
    // Fall back to index.html for SPA routes
}
```

### Build Pipeline (Makefile, NOT build.rs)

```makefile
# Added to existing Makefile

.PHONY: dev build test check release

dev: ## Run in development mode (cargo watch + frontend dev)
	cd frontend && pnpm dev &
	cargo watch -x 'run -- --port 4173'

build-frontend: ## Build frontend static files
	cd frontend && pnpm install && pnpm build

build: build-frontend ## Build release binary with embedded frontend
	cargo build --release

test: ## Run all tests
	cargo test --workspace

check: ## Run all checks (fmt, clippy, svelte-check)
	cargo fmt --check
	cargo clippy --workspace -- -D warnings
	cd frontend && pnpm check

release: build ## Full release build
	@echo "Binary: target/release/forge"
	@ls -lh target/release/forge
```

### Startup Sequence

```
main() {
    1. Parse CLI args
    2. Create data directory (~/.claude-forge/) if needed
    3. Open SQLite connection (WAL mode)
    4. Run migrations
    5. Create EventBus (capacity: 1024)
    6. Spawn BatchWriter (connects to EventBus)
    7. Create repositories (AgentRepo, etc.)
    8. Build AppState
    9. Build Axum router (API routes + static files + WebSocket)
    10. Emit SystemStarted event
    11. Bind to host:port
    12. Log: "Forge running at http://host:port"
    13. Serve until SIGINT/SIGTERM
    14. On shutdown: emit SystemStopped, flush BatchWriter
}
```

### End-to-End Smoke Test

```rust
#[tokio::test]
async fn test_full_startup_and_health() {
    // Start server on random port
    // GET /api/v1/health -> 200
    // GET / -> 200 (index.html)
    // Connect WebSocket -> receive SystemStarted event
    // POST /api/v1/agents -> 201
    // GET /api/v1/agents -> [agent]
    // Shutdown cleanly
}
```

---

## Step 7: Git Workflow

### Branch

```bash
git checkout -b feat/phase0-foundation
```

### Commit Sequence

| Commit | Content |
|--------|---------|
| 1 | `feat: scaffold workspace with 8 crates` -- Cargo.toml, crate stubs, Makefile |
| 2 | `feat(core): implement ForgeEvent enum and EventBus` -- types, events, bus, tests |
| 3 | `feat(db): implement schema, migrations, and batch writer` -- SQL, migrator, batch writer, tests |
| 4 | `feat(agent): implement Agent types, presets, and validation` -- model, presets, validation, tests |
| 5 | `feat(db): implement AgentRepo with CRUD` -- repository, tests |
| 6 | `feat(api): implement Axum server with agent CRUD and WebSocket` -- routes, handlers, WS, tests |
| 7 | `feat(frontend): implement SvelteKit shell with agent CRUD UI` -- layout, pages, stores |
| 8 | `feat(app): wire everything into single binary` -- main.rs, CLI, rust-embed, smoke test |

### Rules

- Each commit compiles and passes tests
- `cargo clippy -- -D warnings` clean
- `cargo fmt` applied
- No unrelated changes mixed
- Commit messages: imperative mood, explain WHY not just WHAT

---

## Phase 0 Success Criteria

When Phase 0 is done, this works:

```bash
$ make build
$ ./target/release/forge
Forge running at http://127.0.0.1:4173

# In browser:
# - See dashboard with version and green WebSocket indicator
# - Navigate to Agents page
# - Create "My Coder" agent with CodeWriter preset
# - See it appear in the list
# - Edit its system prompt
# - Delete it
# - See events flowing in status bar
```

### Measurable Criteria

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --workspace` -- all pass
- [ ] `cargo clippy --workspace -- -D warnings` -- clean
- [ ] Binary starts, serves UI at configured port
- [ ] WebSocket connects, receives heartbeat events
- [ ] Agent CRUD works end-to-end (API + UI + DB)
- [ ] Batch writer flushes events to SQLite
- [ ] FTS5 tables exist and are queryable
- [ ] Schema supports all planned phases (no migration needed for Phase 1-5)
- [ ] Binary size < 15MB (without frontend) or < 25MB (with frontend)

---

## What Phase 0 Does NOT Include

Explicitly out of scope (Phase 1+):
- Process spawning (`claude -p`)
- Stream-json parsing
- Session resume (`--resume`)
- Any workflow execution
- Any skill importing
- Any safety enforcement (stubs only)
- Any MCP protocol (stubs only)
- Notifications, scheduler, plugins
- Git integration
