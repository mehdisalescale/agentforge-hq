# Claude Forge -- Module Dependency Graph

> 12 workspace crates + 1 test utilities crate + 1 integration test crate.
> Dependencies flow strictly downward. No cycles.

---

## 1. Crate Map

```
claude-forge (workspace root)
  crates/
    forge-core/           # Types, events, traits, IDs -- depended on by everything
    forge-db/             # SQLite persistence, migrations, repositories
    forge-safety/         # Circuit breaker, rate limiter, cost tracking
    forge-mcp/            # MCP server, tools, resources, protocol
    forge-workflow/        # Workflow engine, steps, conditions, state machine
    forge-skills/         # Skill catalog, search, categories, parameter validation
    forge-git/            # libgit2 wrapper, status, diff, log, worktrees
    forge-notify/         # Notifications: WebSocket, desktop, webhook, email
    forge-scheduler/      # Cron engine, job management, recurring tasks
    forge-plugins/        # WASM host, plugin API, resource limits
    forge-observe/        # Metrics, tracing integration, dashboards
    forge-api/            # Axum HTTP/WS server, routes, middleware, static embedding
    forge-test-utils/     # Test fixtures, builders, mock helpers (dev-dependency only)
    forge-integration-tests/  # Cross-crate integration tests
```

---

## 2. Dependency Graph (ASCII)

```
                         +-----------+
                         | forge-api |  (binary crate -- the final artifact)
                         +-----+-----+
                               |
          +--------------------+--------------------+
          |          |         |         |          |
          v          v         v         v          v
    +---------+ +--------+ +------+ +--------+ +----------+
    |  notify | |  mcp   | | git  | | plugins| | observe  |
    +---------+ +--------+ +------+ +--------+ +----------+
          |          |         |         |          |
          |     +----+----+    |         |          |
          |     |         |    |         |          |
          v     v         v    v         v          v
    +-----------+   +----------+   +----------+
    | workflow  |   | scheduler|   |  safety  |
    +-----------+   +----------+   +----------+
          |              |              |
          v              v              v
    +-----------+   +-----------+
    |  skills   |   |    db     |
    +-----------+   +-----------+
          |              |
          v              v
    +---------------------------+
    |        forge-core         |
    +---------------------------+
          |
          v
    (std + external crates: serde, uuid, chrono, tracing, thiserror)
```

### 2.1 Detailed Dependency Arrows

```
forge-core     -> (none -- leaf crate)
forge-db       -> forge-core
forge-safety   -> forge-core, forge-db
forge-skills   -> forge-core, forge-db
forge-git      -> forge-core
forge-workflow -> forge-core, forge-db, forge-skills, forge-safety
forge-scheduler-> forge-core, forge-db
forge-notify   -> forge-core, forge-db, forge-workflow
forge-mcp      -> forge-core, forge-db, forge-workflow, forge-skills, forge-safety
forge-plugins  -> forge-core, forge-db, forge-safety
forge-observe  -> forge-core, forge-db, forge-safety
forge-api      -> ALL crates above (top-level composition)
```

---

## 3. Build Order

Cargo resolves this automatically, but the logical build order is:

```
Layer 0 (leaf):     forge-core
Layer 1:            forge-db, forge-git
Layer 2:            forge-safety, forge-skills, forge-scheduler
Layer 3:            forge-workflow, forge-plugins, forge-observe
Layer 4:            forge-notify, forge-mcp
Layer 5 (root):     forge-api (binary)
```

| Layer | Crates | Depends On | Approx Build Time (release) |
|-------|--------|------------|---------------------------|
| 0 | forge-core | external only | ~5s |
| 1 | forge-db, forge-git | Layer 0 | ~15s (rusqlite bundled, git2 vendored) |
| 2 | forge-safety, forge-skills, forge-scheduler | Layers 0-1 | ~8s (parallel) |
| 3 | forge-workflow, forge-plugins, forge-observe | Layers 0-2 | ~20s (wasmtime is slow) |
| 4 | forge-notify, forge-mcp | Layers 0-3 | ~5s |
| 5 | forge-api | All | ~10s (rust-embed, linking) |
| **Total** | | | **~60-80s** (incremental: ~5-15s) |

---

## 4. Compilation Time Optimization

### 4.1 Strategies

| Strategy | Impact | Where |
|----------|--------|-------|
| **Workspace crate splitting** | Enables parallel compilation of independent crates | Layer 2+3 crates build in parallel |
| **Feature flags for heavy deps** | Skip wasmtime when `plugins` feature is disabled | forge-plugins behind `plugins` feature |
| **`codegen-units = 256`** (debug) | Faster debug builds at cost of optimization | `[profile.dev]` in workspace Cargo.toml |
| **`codegen-units = 1`** (release) | Better LTO, smaller binary | `[profile.release]` |
| **Thin LTO** (release) | Good size reduction without full LTO compile cost | `lto = "thin"` in release profile |
| **Shared dependency versions** | Prevents duplicate compilation of different versions | `[workspace.dependencies]` section |
| **`cargo-nextest`** | Compiles tests as separate binaries, runs in parallel | CI and local development |
| **`sccache`** | Compilation cache across builds | CI environment |

### 4.2 Cargo Profile Configuration

```toml
# Workspace Cargo.toml

[profile.dev]
opt-level = 0
debug = true
codegen-units = 256        # Fast debug compilation
incremental = true

[profile.dev.package.rusqlite]
opt-level = 2              # SQLite is painfully slow at opt-level 0

[profile.dev.package.git2]
opt-level = 2              # Same for libgit2

[profile.release]
opt-level = 3
debug = false
codegen-units = 1          # Better LTO
lto = "thin"
strip = true
panic = "abort"            # Smaller binary, no unwinding
```

### 4.3 Build Time Targets

| Scenario | Target | Notes |
|----------|--------|-------|
| Full clean build (debug) | < 90s | First build on a fresh clone |
| Full clean build (release) | < 180s | Includes LTO |
| Incremental build (1 crate change) | < 15s | Typical development cycle |
| Frontend build (pnpm build) | < 10s | Vite + Svelte compile |
| Full pipeline (frontend + release) | < 200s | CI build target |

---

## 5. Feature Flags Per Crate

### 5.1 forge-core

```toml
[features]
default = []
test-helpers = []    # Expose test fixture builders (for forge-test-utils)
```

### 5.2 forge-db

```toml
[features]
default = ["fts5"]
fts5 = []            # Full-text search support (requires bundled-full rusqlite)
migrations = []      # Include migration runner (skip in minimal builds)
```

### 5.3 forge-git

```toml
[features]
default = ["vendored"]
vendored = ["git2/vendored-libgit2", "git2/vendored-openssl"]
worktrees = []       # Worktree management APIs
```

### 5.4 forge-plugins

```toml
[features]
default = []
# This entire crate is opt-in. forge-api depends on it behind the "plugins" feature.
```

### 5.5 forge-api (binary crate)

```toml
[features]
default = ["plugins", "fts5"]
plugins = ["dep:forge-plugins"]     # WASM plugin runtime (+12 MB binary)
fts5 = ["forge-db/fts5"]           # Full-text search
dev = []                            # Development mode (CORS *, verbose logging)
```

**Binary size with/without features:**

| Configuration | Approx Size |
|--------------|-------------|
| All features (default) | ~35 MB |
| Without `plugins` | ~23 MB |
| Without `plugins` + `fts5` | ~22 MB |
| Minimal (no optional features) | ~20 MB |

---

## 6. Public API Surface Per Crate

### 6.1 forge-core

```rust
// Types
pub struct Agent { ... }
pub struct Event { ... }
pub struct Session { ... }
pub enum EventKind { ... }

// Traits
pub trait EventSink: Send + Sync { ... }
pub trait Repository<T>: Send + Sync { ... }

// IDs
pub type AgentId = Uuid;
pub type SessionId = Uuid;
pub type EventId = Uuid;

// Config
pub struct ForgeConfig { ... }
pub fn load_config(path: Option<&Path>) -> Result<ForgeConfig> { ... }

// Validation
pub fn validate_agent_name(name: &str) -> Result<(), CoreError> { ... }
pub fn validate_model(model: &str) -> Result<(), CoreError> { ... }
```

### 6.2 forge-db

```rust
// Database
pub struct Database { ... }
pub fn open(path: &Path) -> Result<Database> { ... }
pub fn open_memory() -> Result<Database> { ... }

// Repositories
pub struct AgentRepo { ... }
pub struct EventRepo { ... }
pub struct SessionRepo { ... }
pub struct SkillRepo { ... }
pub struct WorkflowRepo { ... }

// Batch writer
pub struct BatchWriter { ... }
pub struct BatchConfig { ... }

// Migrations
pub fn run_migrations(db: &Database) -> Result<()> { ... }
```

### 6.3 forge-safety

```rust
// Circuit breaker
pub struct CircuitBreaker { ... }
pub enum CircuitState { Closed, Open, HalfOpen }
pub struct CircuitBreakerConfig { ... }

// Rate limiter
pub struct RateLimiter { ... }
pub struct RateLimiterConfig { ... }

// Cost tracking
pub struct CostTracker { ... }
pub struct CostReport { ... }
pub struct CostBudget { ... }
```

### 6.4 forge-mcp

```rust
// Server
pub struct McpServer { ... }
pub fn create_server(config: McpConfig) -> McpServer { ... }

// Tools
pub trait McpTool: Send + Sync { ... }
pub struct ToolRegistry { ... }

// Resources
pub trait McpResource: Send + Sync { ... }

// Protocol
pub struct McpMessage { ... }
pub struct McpResponse { ... }
```

### 6.5 forge-workflow

```rust
// Engine
pub struct WorkflowEngine { ... }
pub async fn execute(engine: &WorkflowEngine, workflow: &Workflow) -> Result<WorkflowRun> { ... }

// Types
pub struct Workflow { ... }
pub struct WorkflowStep { ... }
pub struct WorkflowRun { ... }
pub enum StepKind { Prompt, Parallel, Conditional, Loop, Handoff }
pub enum RunStatus { Pending, Running, Completed, Failed, Cancelled }

// Conditions
pub struct Condition { ... }
pub fn evaluate(condition: &Condition, context: &Context) -> bool { ... }
```

### 6.6 forge-skills

```rust
// Catalog
pub struct SkillCatalog { ... }
pub fn load_catalog(db: &Database) -> Result<SkillCatalog> { ... }

// Types
pub struct Skill { ... }
pub struct SkillCategory { ... }
pub struct SkillParam { ... }

// Search
pub fn search(catalog: &SkillCatalog, query: &str) -> Vec<SkillMatch> { ... }
pub fn get_by_category(catalog: &SkillCatalog, category: &str) -> Vec<&Skill> { ... }
```

### 6.7 forge-git

```rust
// Repository wrapper
pub struct GitRepo { ... }
pub fn open(path: &Path) -> Result<GitRepo> { ... }

// Operations
pub fn status(repo: &GitRepo) -> Result<Vec<StatusEntry>> { ... }
pub fn diff(repo: &GitRepo, options: DiffOptions) -> Result<Diff> { ... }
pub fn log(repo: &GitRepo, options: LogOptions) -> Result<Vec<Commit>> { ... }
pub fn branches(repo: &GitRepo) -> Result<Vec<Branch>> { ... }

// Worktrees (behind feature flag)
pub fn list_worktrees(repo: &GitRepo) -> Result<Vec<Worktree>> { ... }
pub fn create_worktree(repo: &GitRepo, name: &str, branch: &str) -> Result<Worktree> { ... }
pub fn remove_worktree(repo: &GitRepo, name: &str) -> Result<()> { ... }
```

### 6.8 forge-notify

```rust
pub struct NotificationService { ... }
pub struct Notification { ... }
pub enum Channel { WebSocket, Desktop, Webhook, Email }
pub fn send(service: &NotificationService, notification: Notification) -> Result<()> { ... }
```

### 6.9 forge-scheduler

```rust
pub struct Scheduler { ... }
pub struct Job { ... }
pub struct Schedule { ... }  // Wraps cron expression
pub fn start(scheduler: &Scheduler) -> Result<()> { ... }
pub fn add_job(scheduler: &Scheduler, job: Job) -> Result<JobId> { ... }
pub fn remove_job(scheduler: &Scheduler, id: JobId) -> Result<()> { ... }
```

### 6.10 forge-plugins

```rust
pub struct PluginHost { ... }
pub struct Plugin { ... }
pub struct PluginManifest { ... }
pub struct ResourceLimits { ... }
pub fn load_plugin(host: &PluginHost, path: &Path) -> Result<Plugin> { ... }
pub fn execute(host: &PluginHost, plugin: &Plugin, input: Value) -> Result<Value> { ... }
```

### 6.11 forge-observe

```rust
pub struct MetricsCollector { ... }
pub struct Dashboard { ... }
pub struct SpanData { ... }
pub fn record_metric(collector: &MetricsCollector, name: &str, value: f64) { ... }
pub fn get_dashboard(collector: &MetricsCollector) -> Dashboard { ... }
```

### 6.12 forge-api

```rust
// This is the binary crate. No public API -- it composes all other crates.
pub fn build_router(ctx: AppContext) -> Router { ... }  // For testing
pub fn main() -> Result<()> { ... }                     // Entry point
```

---

## 7. Circular Dependency Prevention

### 7.1 Rules

1. **Dependency direction is always downward** in the layer diagram (Section 2). A crate at Layer N may only depend on crates at Layer N-1 or below.

2. **forge-core depends on nothing in the workspace.** It is the leaf node. If you need a type from forge-core in forge-db, that is fine. If you need a type from forge-db in forge-core, you are doing it wrong -- extract it to forge-core.

3. **Traits break upward dependencies.** If `forge-workflow` needs to emit events (which `forge-notify` handles), it does NOT depend on `forge-notify`. Instead:
   - `forge-core` defines `trait EventSink`.
   - `forge-workflow` accepts `impl EventSink`.
   - `forge-notify` implements `EventSink`.
   - `forge-api` wires them together.

4. **No crate depends on forge-api.** It is the root composition crate.

5. **forge-test-utils is a dev-dependency only.** It appears in `[dev-dependencies]`, never in `[dependencies]`.

### 7.2 Enforcement

```toml
# deny.toml (cargo-deny configuration)
[bans]
# Workspace crate restrictions
deny = [
  # forge-core must not depend on any workspace crate
  { name = "forge-core", wrappers = [] },
]

# No duplicate versions of key crates
multiple-versions = "deny"
wildcards = "deny"
```

Additionally, the CI pipeline runs a custom script that parses `cargo metadata --format-version 1` and verifies no cycles exist in workspace crate dependencies.

### 7.3 Common Patterns That Prevent Cycles

**Pattern: Trait in core, impl in higher crate**
```
forge-core:     pub trait ProcessSpawner { ... }
forge-api:      impl ProcessSpawner for TokioSpawner { ... }
forge-workflow: fn execute(spawner: &dyn ProcessSpawner) { ... }
```

**Pattern: Event-driven decoupling**
```
forge-workflow: emits WorkflowCompleted event
forge-notify:   subscribes to WorkflowCompleted, sends notification
// Neither depends on the other. Both depend on forge-core (Event type).
```

**Pattern: Callback/closure injection**
```
forge-scheduler: fn add_job(schedule, callback: Box<dyn Fn()>)
forge-api:       scheduler.add_job(cron, Box::new(|| workflow.run()))
// forge-scheduler does not know about workflows.
```

---

## 8. External Dependency Tree (Major)

```
forge-api
  +-- axum 0.8
  |     +-- tower 0.5
  |     +-- tower-http 0.6
  |     +-- hyper 1.5
  +-- tokio 1.41
  +-- rust-embed 8.5
  +-- clap 4.5
  +-- tracing 0.1
  |     +-- tracing-subscriber 0.3
  +-- serde 1.0
  |     +-- serde_json 1.0
  +-- rusqlite 0.32 (bundled)
  +-- git2 0.19 (vendored)
  +-- wasmtime 27.0 [optional: plugins]
  +-- reqwest 0.12
  |     +-- rustls 0.23
  +-- uuid 1.11
  +-- chrono 0.4
  +-- dashmap 6.1
  +-- cron 0.13
  +-- thiserror 2.0
  +-- anyhow 1.0 (binary only)
  +-- metrics 0.24
  +-- secrecy 0.10
  +-- nix 0.29
```

**Shared dependencies** (used by many crates, compiled once):
- `serde`, `serde_json`: Every crate
- `uuid`: Every crate (ID types)
- `chrono`: Most crates (timestamps)
- `tracing`: Every crate (logging)
- `thiserror`: Every crate (error types)

These are declared in `[workspace.dependencies]` to ensure a single version across the workspace.
