# Cursor Agent Prompts — Claude Forge Phase 0

> 4 self-contained prompts. Each agent gets FULL context.
> Copy-paste one prompt per Cursor agent. No cross-reading needed.
> Claude reviews output. Cursor builds.

---

## How to Use

1. Create a new repo: `git init claude-forge && cd claude-forge`
2. Copy `forge-project/` into the repo (for reference docs)
3. Copy `refrence-repo/` or symlink it (for study material)
4. Open 4 Cursor agent tabs
5. Paste **Agent D's prompt first** — it creates the scaffold
6. Wait for Agent D's scaffold commit
7. Paste Agents A, B, C prompts in parallel

---

## Agent D — Scaffold + API + Binary (RUN FIRST)

<details>
<summary>Click to expand full prompt (paste into Cursor)</summary>

```
You are building Claude Forge — a multi-agent Claude Code orchestrator.
Single Rust binary, embedded SvelteKit frontend, SQLite persistence.
Your role: Agent D — workspace scaffold, API server, binary wiring.

═══════════════════════════════════════════════════════════════
WHAT IS CLAUDE FORGE
═══════════════════════════════════════════════════════════════

Claude Forge is a local-first GUI for orchestrating multiple Claude Code
agents. It wraps the `claude` CLI, adds persistence, real-time streaming,
workflow automation, and a web dashboard. The end result is a single
binary (`./forge`) that opens a browser, shows a dashboard, and lets
users create/run/monitor AI agents.

Phase 0 scope: Empty workspace → running binary with agent CRUD working
end-to-end (API + UI + DB + WebSocket events).

═══════════════════════════════════════════════════════════════
YOUR FILES (YOU OWN THESE — ONLY YOU TOUCH THEM)
═══════════════════════════════════════════════════════════════

Cargo.toml              (workspace root)
Makefile
.github/workflows/ci.yml
crates/forge-api/src/**
crates/forge-app/src/**
crates/forge-process/src/**   (stub only)
crates/forge-safety/src/**    (stub only)
crates/forge-mcp/src/**       (stub only)

═══════════════════════════════════════════════════════════════
DO NOT TOUCH (other agents own these)
═══════════════════════════════════════════════════════════════

crates/forge-core/**     → Agent A owns
crates/forge-agent/**    → Agent A owns
crates/forge-db/**       → Agent B owns
migrations/**            → Agent B owns
frontend/**              → Agent C owns

═══════════════════════════════════════════════════════════════
PHASE 1: SCAFFOLD (DO THIS FIRST, COMMIT, THEN CONTINUE)
═══════════════════════════════════════════════════════════════

Create the workspace so all agents have a compiling baseline.

Directory structure:
```
claude-forge/
  Cargo.toml                    # Workspace manifest
  Makefile                      # Build orchestration
  .github/workflows/ci.yml     # CI (test + clippy + fmt)
  crates/
    forge-core/
      Cargo.toml
      src/lib.rs                # empty: pub mod placeholder;
    forge-db/
      Cargo.toml
      src/lib.rs                # empty
    forge-api/
      Cargo.toml
      src/lib.rs                # empty
    forge-agent/
      Cargo.toml
      src/lib.rs                # empty
    forge-process/
      Cargo.toml
      src/lib.rs                # // Phase 1: process spawning
    forge-safety/
      Cargo.toml
      src/lib.rs                # // Phase 4: circuit breaker, rate limiter
    forge-mcp/
      Cargo.toml
      src/lib.rs                # // Phase 4: MCP protocol
    forge-app/
      Cargo.toml
      src/main.rs               # fn main() { println!("forge scaffold"); }
  migrations/
    .gitkeep
  frontend/
    .gitkeep
```

Workspace Cargo.toml:

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"

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

Each crate's Cargo.toml should use `workspace = true` for shared deps.
Example for forge-api:

```toml
[package]
name = "forge-api"
version.workspace = true
edition.workspace = true

[dependencies]
forge-core = { path = "../forge-core" }
forge-agent = { path = "../forge-agent" }
forge-db = { path = "../forge-db" }
axum.workspace = true
axum-extra.workspace = true
tower.workspace = true
tower-http.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
tracing.workspace = true
```

Makefile (include build targets + existing repo management):

```makefile
.PHONY: dev build-frontend build test check release clean

dev: ## Run in development mode
	cargo run -p forge-app

build-frontend: ## Build frontend static files
	cd frontend && pnpm install && pnpm build

build: build-frontend ## Build release binary with embedded frontend
	cargo build --release

test: ## Run all tests
	cargo test --workspace

check: ## Run all checks (fmt, clippy)
	cargo fmt --check
	cargo clippy --workspace -- -D warnings

release: build ## Full release build
	@echo "Binary: target/release/forge-app"
	@ls -lh target/release/forge-app

clean: ## Clean build artifacts
	cargo clean
```

CI workflow (.github/workflows/ci.yml):

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
```

After creating all this, run `cargo build` to verify it compiles.
Commit: "feat: scaffold workspace with 8 crates"

═══════════════════════════════════════════════════════════════
PHASE 2: STUB CRATES
═══════════════════════════════════════════════════════════════

forge-process/src/lib.rs:
```rust
//! Process spawning for Claude Code CLI.
//! Phase 1 implementation — currently a stub.
```

forge-safety/src/lib.rs:
```rust
//! Safety controls: circuit breaker, rate limiter, budget enforcement.
//! Phase 4 implementation — currently a stub.
```

forge-mcp/src/lib.rs:
```rust
//! MCP (Model Context Protocol) server and client.
//! Phase 4 implementation — currently a stub.
```

═══════════════════════════════════════════════════════════════
PHASE 3: forge-api (Axum server)
═══════════════════════════════════════════════════════════════

Wait until Agents A and B have committed their code. If they haven't
yet, write handler signatures and tests against the contracts below,
using todo!() for missing pieces.

File structure:
```
crates/forge-api/src/
  lib.rs          # re-exports
  state.rs        # AppState
  router.rs       # build_router()
  middleware.rs    # CORS, tracing, request ID
  error.rs        # ForgeError → HTTP response mapping
  handlers/
    mod.rs
    health.rs     # GET /api/v1/health
    agents.rs     # Agent CRUD handlers
    ws.rs         # WebSocket upgrade + event streaming
```

### AppState

```rust
use std::sync::Arc;
use forge_core::EventBus;
use forge_db::{DbPool, BatchWriter};
use forge_db::repos::AgentRepo;

#[derive(Clone)]
pub struct AppState {
    pub event_bus: Arc<EventBus>,
    pub db: Arc<DbPool>,
    pub agent_repo: Arc<AgentRepo>,
    pub batch_writer: Arc<BatchWriter>,
    pub start_time: std::time::Instant,
}
```

### Router

```rust
use axum::{Router, routing::{get, post, put, delete}};

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/agents", get(handlers::agents::list).post(handlers::agents::create))
        .route("/agents/{id}", get(handlers::agents::get_one)
            .put(handlers::agents::update)
            .delete(handlers::agents::delete_one));

    let ws = Router::new()
        .route("/ws", get(handlers::ws::ws_upgrade));

    Router::new()
        .nest("/api/v1", api)
        .nest("/api/v1", ws)
        .with_state(state)
        // Middleware layers added here
}
```

### API Endpoints (Contract — match exactly)

```
GET    /api/v1/health    → { "status": "ok", "version": "0.1.0", "uptime_secs": 42 }
GET    /api/v1/agents    → Agent[]
POST   /api/v1/agents    → Agent              (201 Created)
GET    /api/v1/agents/:id → Agent             (404 if not found)
PUT    /api/v1/agents/:id → Agent             (404 if not found)
DELETE /api/v1/agents/:id → null              (204 No Content)
GET    /api/v1/ws         → WebSocket upgrade

Error format:
{ "error": "Agent not found", "code": "AGENT_NOT_FOUND" }     (404)
{ "error": "Name is required", "code": "VALIDATION_ERROR" }   (422)
{ "error": "Internal server error", "code": "INTERNAL_ERROR" } (500)
```

### Error → HTTP mapping

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

impl IntoResponse for ForgeError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ForgeError::AgentNotFound(_) => (StatusCode::NOT_FOUND, "AGENT_NOT_FOUND", self.to_string()),
            ForgeError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR", self.to_string()),
            ForgeError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Database error".into()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal error".into()),
        };
        (status, Json(serde_json::json!({ "error": message, "code": code }))).into_response()
    }
}
```

### WebSocket handler

```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::extract::State;

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.event_bus.subscribe();

    // Send initial connection event
    let connected = serde_json::json!({
        "type": "SystemStarted",
        "data": { "version": "0.1.0", "timestamp": chrono::Utc::now() }
    });
    let _ = socket.send(Message::Text(connected.to_string())).await;

    // Stream events
    loop {
        tokio::select! {
            Ok(event) = rx.recv() => {
                let json = serde_json::to_string(&event).unwrap_or_default();
                if socket.send(Message::Text(json)).await.is_err() {
                    break; // Client disconnected
                }
            }
            // Heartbeat every 30s
            _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                let hb = serde_json::json!({
                    "type": "Heartbeat",
                    "data": { "timestamp": chrono::Utc::now() }
                });
                if socket.send(Message::Text(hb.to_string())).await.is_err() {
                    break;
                }
            }
        }
    }
}
```

### Agent CRUD handlers

```rust
use axum::extract::{Path, State, Json};
use axum::http::StatusCode;
use forge_agent::{Agent, NewAgent, UpdateAgent};
use forge_core::AgentId;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Agent>>, ForgeError> {
    let agents = state.agent_repo.list()?;
    Ok(Json(agents))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<NewAgent>,
) -> Result<(StatusCode, Json<Agent>), ForgeError> {
    let agent = state.agent_repo.create(&input)?;
    state.event_bus.emit(ForgeEvent::AgentCreated {
        agent_id: agent.id.clone(),
        name: agent.name.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    Ok((StatusCode::CREATED, Json(agent)))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Agent>, ForgeError> {
    let agent = state.agent_repo.get(&AgentId(id))?;
    Ok(Json(agent))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(input): Json<UpdateAgent>,
) -> Result<Json<Agent>, ForgeError> {
    let agent = state.agent_repo.update(&AgentId(id), &input)?;
    state.event_bus.emit(ForgeEvent::AgentUpdated {
        agent_id: agent.id.clone(),
        name: agent.name.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    Ok(Json(agent))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ForgeError> {
    let agent_id = AgentId(id);
    state.agent_repo.delete(&agent_id)?;
    state.event_bus.emit(ForgeEvent::AgentDeleted {
        agent_id,
        timestamp: chrono::Utc::now(),
    })?;
    Ok(StatusCode::NO_CONTENT)
}
```

═══════════════════════════════════════════════════════════════
PHASE 4: forge-app (binary entry point)
═══════════════════════════════════════════════════════════════

File structure:
```
crates/forge-app/
  Cargo.toml
  src/
    main.rs       # CLI + startup sequence
    embed.rs      # rust-embed for frontend
```

### CLI

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "forge", about = "Claude Forge — Multi-agent orchestrator")]
struct Cli {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value_t = 4173)]
    port: u16,

    /// Data directory for SQLite database
    #[arg(long, default_value = "~/.claude-forge")]
    data_dir: PathBuf,
}
```

### Static file embedding (rust-embed)

```rust
use rust_embed::Embed;
use axum::response::{Html, IntoResponse, Response};
use axum::http::{header, StatusCode, Uri};

#[derive(Embed)]
#[folder = "frontend/build"]
struct FrontendAssets;

pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try exact file match (JS, CSS, images, etc.)
    if let Some(file) = FrontendAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            file.data.to_vec(),
        ).into_response();
    }

    // SPA fallback: serve index.html for all other routes
    match FrontendAssets::get("index.html") {
        Some(file) => Html(String::from_utf8_lossy(&file.data).to_string()).into_response(),
        None => (StatusCode::NOT_FOUND, "Frontend not built").into_response(),
    }
}
```

### Startup sequence (main.rs)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Parse CLI
    let cli = Cli::parse();

    // 2. Init tracing
    tracing_subscriber::fmt()
        .with_env_filter("forge=debug,tower_http=debug")
        .init();

    // 3. Create data directory
    let data_dir = expand_tilde(&cli.data_dir);
    std::fs::create_dir_all(&data_dir)?;

    // 4. Open SQLite (WAL mode)
    let db_path = data_dir.join("forge.db");
    let db = forge_db::DbPool::new(&db_path)?;

    // 5. Run migrations
    forge_db::Migrator::new(db.connection()).apply_pending()?;

    // 6. Create EventBus
    let event_bus = Arc::new(forge_core::EventBus::new(1024));

    // 7. Spawn BatchWriter
    let batch_writer = Arc::new(forge_db::BatchWriter::spawn(
        db.write_connection(),
        event_bus.subscribe(),
    ));

    // 8. Create repos
    let agent_repo = Arc::new(forge_db::repos::AgentRepo::new(db.connection()));

    // 9. Build AppState
    let state = forge_api::AppState {
        event_bus: event_bus.clone(),
        db: Arc::new(db),
        agent_repo,
        batch_writer,
        start_time: std::time::Instant::now(),
    };

    // 10. Build router (API + static fallback)
    let app = forge_api::build_router(state)
        .fallback(embed::static_handler);

    // 11. Emit SystemStarted
    event_bus.emit(forge_core::ForgeEvent::SystemStarted {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    })?;

    // 12. Bind and serve
    let addr = format!("{}:{}", cli.host, cli.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Forge running at http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // 13. Shutdown
    tracing::info!("Shutting down...");
    event_bus.emit(forge_core::ForgeEvent::SystemStopped {
        timestamp: chrono::Utc::now(),
    })?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("failed to listen for ctrl+c");
}

fn expand_tilde(path: &std::path::Path) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("~") {
        dirs::home_dir().unwrap_or_default().join(stripped)
    } else {
        path.to_path_buf()
    }
}
```

Add `dirs = "6"` and `anyhow = "1"` and `mime_guess = "2"` to forge-app Cargo.toml.

═══════════════════════════════════════════════════════════════
TESTS (forge-api + forge-app)
═══════════════════════════════════════════════════════════════

Write these as integration tests in crates/forge-api/tests/ or inline:

1. health_returns_200 — GET /api/v1/health → 200 with version field
2. health_includes_version — response has "version": "0.1.0"
3. create_agent_returns_201 — POST valid agent → 201 Created
4. create_agent_invalid_returns_422 — POST empty name → 422
5. get_agent_not_found_returns_404 — GET /agents/random-uuid → 404
6. list_agents_empty — GET /agents → [] when no agents
7. crud_roundtrip — Create → get → update → list → delete
8. unknown_route_returns_404 — GET /api/v1/nope → 404

Use axum::test to create test requests without starting a real server:
```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for oneshot

#[tokio::test]
async fn health_returns_200() {
    let app = build_test_router();
    let resp = app
        .oneshot(Request::builder().uri("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
```

═══════════════════════════════════════════════════════════════
CODING STANDARDS
═══════════════════════════════════════════════════════════════

Naming:
- Crate names: forge-{name} (kebab-case)
- Types: PascalCase (AppState, ForgeError)
- Functions: snake_case (build_router, health_check)
- Constants: SCREAMING_SNAKE (MAX_BATCH_SIZE)

Error handling:
- Library crates: use thiserror, return Result<T, ForgeError>
- forge-app main.rs: use anyhow::Result for top-level
- Never unwrap() in library code
- expect() only for provably infallible cases

Logging (use tracing crate):
- #[instrument(skip(state))] on all handlers
- error! for failures, info! for state changes, debug! for queries
- Structured fields: info!(agent_id = %id, "created agent")
- Never log secrets

Async rules:
- Never hold mutex/lock guards across .await
- All spawned tasks tracked for graceful shutdown
- Explicit channel capacities (never unbounded)
- Use tokio::select! for racing futures

Testing:
- Test names describe behavior: test_name_describes_what_it_proves
- assert!(matches!(...)) for enum variants
- Red-green-refactor: write failing test first

File organization:
- One concept per file, split at ~400 lines
- lib.rs is public API surface + re-exports
- Put handlers in handlers/ subdirectory

═══════════════════════════════════════════════════════════════
COMMIT SEQUENCE
═══════════════════════════════════════════════════════════════

1. "feat: scaffold workspace with 8 crates" — Cargo.toml, stubs, Makefile, CI
2. "feat(api): implement Axum server with agent CRUD and WebSocket" — all forge-api code
3. "feat(app): wire everything into single binary" — main.rs, CLI, embed, smoke test

═══════════════════════════════════════════════════════════════
REFERENCE REPOS TO STUDY BEFORE CODING
═══════════════════════════════════════════════════════════════

If available, read these first:
- refrence-repo/claude-code-hub/ — API proxy patterns, Axum middleware
- forge-project/03-architecture/API_DESIGN.md — full route specification
- forge-project/05-engineering/CODING_STANDARDS.md — naming, errors, testing
```

</details>

---

## Agent A — Core Types + Agent Engine

<details>
<summary>Click to expand full prompt (paste into Cursor)</summary>

```
You are building Claude Forge — a multi-agent Claude Code orchestrator.
Single Rust binary, embedded SvelteKit frontend, SQLite persistence.
Your role: Agent A — core type system and agent model.

═══════════════════════════════════════════════════════════════
WHAT IS CLAUDE FORGE
═══════════════════════════════════════════════════════════════

Claude Forge is a local-first GUI for orchestrating multiple Claude Code
agents. It wraps the `claude` CLI, adds persistence, real-time streaming,
workflow automation, and a web dashboard. The end result is a single
binary (`./forge`) that opens a browser, shows a dashboard, and lets
users create/run/monitor AI agents.

Phase 0 scope: Empty workspace → running binary with agent CRUD working
end-to-end (API + UI + DB + WebSocket events).

You are building the foundational type system that EVERY other crate
depends on. Your types are the contract. Get them right.

═══════════════════════════════════════════════════════════════
YOUR FILES (YOU OWN THESE — ONLY YOU TOUCH THEM)
═══════════════════════════════════════════════════════════════

crates/forge-core/src/**
crates/forge-agent/src/**

═══════════════════════════════════════════════════════════════
DO NOT TOUCH (other agents own these)
═══════════════════════════════════════════════════════════════

Cargo.toml (workspace root)   → Agent D owns
Makefile                       → Agent D owns
crates/forge-db/**             → Agent B owns
crates/forge-api/**            → Agent D owns
crates/forge-app/**            → Agent D owns
crates/forge-process/**        → Agent D owns (stub)
crates/forge-safety/**         → Agent D owns (stub)
crates/forge-mcp/**            → Agent D owns (stub)
frontend/**                    → Agent C owns
migrations/**                  → Agent B owns

═══════════════════════════════════════════════════════════════
TASK 1: forge-core — Type System, Events, EventBus
═══════════════════════════════════════════════════════════════

Create the core types that all other crates import.

File structure:
```
crates/forge-core/
  Cargo.toml
  src/
    lib.rs          # re-exports: pub mod ids, error, events, event_bus;
    ids.rs          # ID newtypes
    error.rs        # ForgeError + ForgeResult
    events.rs       # ForgeEvent enum (all 20 variants) + OutputKind
    event_bus.rs    # EventBus struct + EventSink trait
```

### Cargo.toml

```toml
[package]
name = "forge-core"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
thiserror.workspace = true
tracing.workspace = true
rusqlite.workspace = true
```

### ids.rs — Newtype ID wrappers

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

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

// Implement Display for all IDs (needed for error messages and logging)
impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
// ... same for SessionId, EventId, WorkflowId, SkillId

// Implement new() for convenience
impl AgentId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}
// ... same for all ID types
```

### error.rs — Error hierarchy

```rust
use crate::ids::{AgentId, SessionId};
use thiserror::Error;

#[derive(Debug, Error)]
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

### events.rs — ForgeEvent enum (ALL 20 variants, not 5)

```rust
use crate::ids::{AgentId, SessionId, WorkflowId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ForgeEvent {
    // System lifecycle
    SystemStarted { version: String, timestamp: DateTime<Utc> },
    SystemStopped { timestamp: DateTime<Utc> },
    Heartbeat { timestamp: DateTime<Utc> },

    // Agent lifecycle (used in Phase 0)
    AgentCreated { agent_id: AgentId, name: String, timestamp: DateTime<Utc> },
    AgentUpdated { agent_id: AgentId, name: String, timestamp: DateTime<Utc> },
    AgentDeleted { agent_id: AgentId, timestamp: DateTime<Utc> },

    // Process lifecycle (Phase 1 — define now, implement later)
    ProcessStarted { session_id: SessionId, agent_id: AgentId, timestamp: DateTime<Utc> },
    ProcessOutput { session_id: SessionId, kind: OutputKind, content: String, timestamp: DateTime<Utc> },
    ProcessCompleted { session_id: SessionId, exit_code: i32, timestamp: DateTime<Utc> },
    ProcessFailed { session_id: SessionId, error: String, timestamp: DateTime<Utc> },

    // Session lifecycle (Phase 1)
    SessionCreated { session_id: SessionId, agent_id: AgentId, directory: String, timestamp: DateTime<Utc> },
    SessionResumed { session_id: SessionId, timestamp: DateTime<Utc> },

    // Workflow lifecycle (Phase 2)
    WorkflowStarted { workflow_id: WorkflowId, timestamp: DateTime<Utc> },
    WorkflowStepCompleted { workflow_id: WorkflowId, step: u32, timestamp: DateTime<Utc> },
    WorkflowCompleted { workflow_id: WorkflowId, timestamp: DateTime<Utc> },
    WorkflowFailed { workflow_id: WorkflowId, error: String, timestamp: DateTime<Utc> },

    // Safety events (Phase 4)
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

CRITICAL: The serde format is `#[serde(tag = "type", content = "data")]`.
This means JSON looks like:
```json
{"type": "AgentCreated", "data": {"agent_id": "uuid", "name": "My Agent", "timestamp": "..."}}
```

### event_bus.rs — Broadcast event bus

```rust
use crate::events::ForgeEvent;
use crate::error::ForgeResult;
use tokio::sync::broadcast;
use tracing::{debug, warn};

pub struct EventBus {
    sender: broadcast::Sender<ForgeEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn emit(&self, event: ForgeEvent) -> ForgeResult<()> {
        debug!(event_type = ?std::mem::discriminant(&event), "emitting event");
        self.sender.send(event).map_err(|e| {
            warn!("No active subscribers: {}", e);
            // Not an error — it's fine if no one is listening
            // Return Ok anyway, or log and continue
        });
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ForgeEvent> {
        self.sender.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Trait for anything that consumes events (DB writer, WebSocket, logger).
pub trait EventSink: Send + Sync {
    fn handle(&self, event: &ForgeEvent);
}
```

NOTE: broadcast::Sender::send returns Err when there are no receivers.
This is NOT an error in our case — emit() should succeed even with 0
subscribers. Handle this gracefully (log at debug/warn, don't propagate).

═══════════════════════════════════════════════════════════════
TASK 2: forge-agent — Agent Model, Presets, Validation
═══════════════════════════════════════════════════════════════

File structure:
```
crates/forge-agent/
  Cargo.toml
  src/
    lib.rs          # re-exports
    model.rs        # Agent, NewAgent, UpdateAgent
    preset.rs       # AgentPreset enum + defaults for each
    validation.rs   # validate_new_agent, validate_update_agent
```

### Cargo.toml

```toml
[package]
name = "forge-agent"
version.workspace = true
edition.workspace = true

[dependencies]
forge-core = { path = "../forge-core" }
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
```

### model.rs

```rust
use chrono::{DateTime, Utc};
use forge_core::AgentId;
use serde::{Deserialize, Serialize};

use crate::preset::AgentPreset;

pub const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

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
pub struct NewAgent {
    pub name: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub use_max: Option<bool>,
    pub preset: Option<AgentPreset>,
    pub config: Option<serde_json::Value>,
}

/// Option<Option<T>> pattern:
/// - None → field not included in request (don't change)
/// - Some(None) → explicitly set to null (clear the value)
/// - Some(Some(v)) → set to new value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgent {
    pub name: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<Option<String>>,
    pub allowed_tools: Option<Option<Vec<String>>>,
    pub max_turns: Option<Option<u32>>,
    pub use_max: Option<bool>,
    pub preset: Option<Option<AgentPreset>>,
    pub config: Option<Option<serde_json::Value>>,
}
```

### preset.rs — 9 agent presets with real system prompts

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

pub struct PresetDefaults {
    pub system_prompt: String,
    pub model: String,
    pub allowed_tools: Option<Vec<String>>,
}

impl AgentPreset {
    pub fn defaults(&self) -> PresetDefaults {
        match self {
            Self::CodeWriter => PresetDefaults {
                system_prompt: "You are a senior software engineer. Write clean, well-tested, production-ready code. Follow existing patterns in the codebase. Include error handling. Write tests for new functionality.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None, // all tools
            },
            Self::Reviewer => PresetDefaults {
                system_prompt: "You are a code reviewer. Analyze code for bugs, security issues, performance problems, and style violations. Be specific about line numbers and suggest fixes. Check for edge cases and error handling gaps.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into()]),
            },
            Self::Tester => PresetDefaults {
                system_prompt: "You are a test engineer. Write comprehensive tests: unit tests, integration tests, and edge case tests. Use the project's existing test framework. Aim for meaningful coverage of business logic, not line count.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::Debugger => PresetDefaults {
                system_prompt: "You are a debugging specialist. Systematically identify root causes. Read error messages carefully. Add targeted logging. Form hypotheses, test them, and narrow down. Fix the root cause, not symptoms.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::Architect => PresetDefaults {
                system_prompt: "You are a software architect. Design systems for simplicity, maintainability, and correctness. Identify abstractions, define boundaries, and document trade-offs. Prefer boring technology over clever solutions.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into(), "WebSearch".into()]),
            },
            Self::Documenter => PresetDefaults {
                system_prompt: "You are a technical writer. Write clear, concise documentation. Include examples. Document the why, not just the what. Keep docs close to the code they describe. Use the project's documentation conventions.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::SecurityAuditor => PresetDefaults {
                system_prompt: "You are a security auditor. Check for OWASP Top 10 vulnerabilities: injection, broken auth, sensitive data exposure, XXE, broken access control, misconfig, XSS, insecure deserialization, known vulns, insufficient logging. Report severity and remediation.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into()]),
            },
            Self::Refactorer => PresetDefaults {
                system_prompt: "You are a refactoring specialist. Improve code structure without changing behavior. Apply SOLID principles where they reduce complexity. Extract when duplication is proven, not speculative. Ensure tests pass before and after.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::Explorer => PresetDefaults {
                system_prompt: "You are a codebase explorer. Navigate unfamiliar code quickly. Map dependencies, find entry points, trace execution flows. Summarize architecture and key patterns. Identify tech debt and improvement opportunities.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into()]),
            },
        }
    }

    /// Returns all preset variants.
    pub fn all() -> &'static [AgentPreset] {
        &[
            Self::CodeWriter, Self::Reviewer, Self::Tester, Self::Debugger,
            Self::Architect, Self::Documenter, Self::SecurityAuditor,
            Self::Refactorer, Self::Explorer,
        ]
    }
}
```

### validation.rs

```rust
use crate::model::NewAgent;
use forge_core::error::{ForgeError, ForgeResult};

const MAX_NAME_LENGTH: usize = 100;
const MAX_SYSTEM_PROMPT_LENGTH: usize = 102_400; // 100KB

pub fn validate_new_agent(agent: &NewAgent) -> ForgeResult<()> {
    // Name: required, 1-100 chars
    if agent.name.trim().is_empty() {
        return Err(ForgeError::Validation("Agent name cannot be empty".into()));
    }
    if agent.name.len() > MAX_NAME_LENGTH {
        return Err(ForgeError::Validation(
            format!("Agent name must be {} characters or fewer", MAX_NAME_LENGTH),
        ));
    }

    // System prompt: optional but bounded
    if let Some(ref prompt) = agent.system_prompt {
        if prompt.len() > MAX_SYSTEM_PROMPT_LENGTH {
            return Err(ForgeError::Validation(
                "System prompt must be 100KB or fewer".into(),
            ));
        }
    }

    Ok(())
}
```

═══════════════════════════════════════════════════════════════
TESTS
═══════════════════════════════════════════════════════════════

Write ALL these tests. Red-green-refactor.

### forge-core tests:

```rust
#[test]
fn emit_event_received_by_subscriber() {
    // Create bus, subscribe, emit, assert received
}

#[test]
fn multiple_subscribers_all_receive() {
    // Create bus, 3 subscribers, emit, all 3 receive
}

#[test]
fn event_serializes_to_json_roundtrip() {
    // Create AgentCreated event, serialize to JSON, deserialize back, assert equal
}

#[test]
fn all_event_variants_serialize() {
    // Create one of each variant, serialize each, assert no panic
    // IMPORTANT: if you add a variant to ForgeEvent, add it here too
}

#[test]
fn serde_tag_format_correct() {
    // Serialize AgentCreated, parse as serde_json::Value
    // Assert it has "type": "AgentCreated" and "data": {...}
}

#[test]
fn subscriber_count_tracks_active_subscribers() {
    // subscribe → count is 1, drop receiver → count is 0
}
```

### forge-agent tests:

```rust
#[test]
fn all_presets_have_non_empty_system_prompt() {
    for preset in AgentPreset::all() {
        let defaults = preset.defaults();
        assert!(!defaults.system_prompt.is_empty(), "{:?} has empty prompt", preset);
    }
}

#[test]
fn validate_rejects_empty_name() {
    let agent = NewAgent { name: "".into(), ..default_new_agent() };
    assert!(matches!(validate_new_agent(&agent), Err(ForgeError::Validation(_))));
}

#[test]
fn validate_rejects_long_name() {
    let agent = NewAgent { name: "x".repeat(101), ..default_new_agent() };
    assert!(matches!(validate_new_agent(&agent), Err(ForgeError::Validation(_))));
}

#[test]
fn validate_accepts_valid_agent() {
    let agent = NewAgent { name: "My Agent".into(), ..default_new_agent() };
    assert!(validate_new_agent(&agent).is_ok());
}

#[test]
fn new_agent_uses_default_model() {
    // When model is None, default should be claude-sonnet-4-20250514
}

#[test]
fn preset_code_writer_allows_all_tools() {
    let defaults = AgentPreset::CodeWriter.defaults();
    assert!(defaults.allowed_tools.is_none()); // None = all tools allowed
}

#[test]
fn preset_reviewer_has_read_only_tools() {
    let defaults = AgentPreset::Reviewer.defaults();
    assert!(defaults.allowed_tools.is_some());
    let tools = defaults.allowed_tools.unwrap();
    assert!(tools.contains(&"Read".to_string()));
    assert!(!tools.contains(&"Write".to_string()));
}
```

═══════════════════════════════════════════════════════════════
CODING STANDARDS
═══════════════════════════════════════════════════════════════

Naming:
- Crate names: forge-{name} (kebab-case)
- Types: PascalCase (ForgeEvent, AgentPreset, EventBus)
- Functions: snake_case (validate_new_agent, emit)
- Constants: SCREAMING_SNAKE (MAX_NAME_LENGTH, DEFAULT_MODEL)

Error handling:
- Use thiserror for ForgeError
- Return ForgeResult<T> from all fallible public functions
- Never unwrap() in library code
- expect() only with explanation for provably infallible cases
- #[from] for automatic conversion from dependency errors

Logging (tracing crate):
- debug! for event emission, subscription
- warn! for no-subscriber sends, lagged receivers
- Structured fields: debug!(event_type = ?discriminant, "emitting")

Documentation:
- /// doc comments on all public types and functions
- Include # Errors section listing possible error variants
- No examples needed for Phase 0 (internal API)

File rules:
- One concept per file, split at ~400 lines
- lib.rs is public API surface with re-exports only
- No mod.rs files (use filename.rs pattern)

═══════════════════════════════════════════════════════════════
COMMIT SEQUENCE
═══════════════════════════════════════════════════════════════

1. "feat(core): implement ForgeEvent enum, EventBus, and error types"
2. "feat(agent): implement Agent model, presets, and validation"

Each commit must compile (`cargo build`) and pass tests (`cargo test`).

═══════════════════════════════════════════════════════════════
REFERENCE REPOS TO STUDY BEFORE CODING
═══════════════════════════════════════════════════════════════

If available, read these first:
- refrence-repo/claude-code-tools/src/ — type organization patterns
- refrence-repo/awesome-claude-code-subagents/ — agent preset definitions,
  model routing per agent type, tool permission patterns
- refrence-repo/claude-code-templates/ — agent template structures
- forge-project/03-architecture/SYSTEM_ARCHITECTURE.md — event types
- forge-project/05-engineering/CODING_STANDARDS.md — naming, errors
```

</details>

---

## Agent B — Database, Schema, Batch Writer

<details>
<summary>Click to expand full prompt (paste into Cursor)</summary>

```
You are building Claude Forge — a multi-agent Claude Code orchestrator.
Single Rust binary, embedded SvelteKit frontend, SQLite persistence.
Your role: Agent B — database layer, schema, migrations, batch writer.

═══════════════════════════════════════════════════════════════
WHAT IS CLAUDE FORGE
═══════════════════════════════════════════════════════════════

Claude Forge is a local-first GUI for orchestrating multiple Claude Code
agents. It wraps the `claude` CLI, adds persistence, real-time streaming,
workflow automation, and a web dashboard. The end result is a single
binary (`./forge`) that opens a browser, shows a dashboard, and lets
users create/run/monitor AI agents.

Phase 0 scope: Empty workspace → running binary with agent CRUD working
end-to-end (API + UI + DB + WebSocket events).

You are building the persistence layer. Every event, every agent, every
session flows through your code. The batch writer is the critical piece
that makes high-throughput event logging work without blocking the UI.

═══════════════════════════════════════════════════════════════
YOUR FILES (YOU OWN THESE — ONLY YOU TOUCH THEM)
═══════════════════════════════════════════════════════════════

crates/forge-db/src/**
migrations/**

═══════════════════════════════════════════════════════════════
DO NOT TOUCH (other agents own these)
═══════════════════════════════════════════════════════════════

Cargo.toml (workspace root)   → Agent D owns
crates/forge-core/**           → Agent A owns (you import these)
crates/forge-agent/**          → Agent A owns (you import these)
crates/forge-api/**            → Agent D owns
crates/forge-app/**            → Agent D owns
frontend/**                    → Agent C owns

═══════════════════════════════════════════════════════════════
DEPENDENCY ON AGENT A
═══════════════════════════════════════════════════════════════

You import forge-core (ForgeEvent, ForgeError, AgentId, etc.) and
forge-agent (Agent, NewAgent, UpdateAgent). If Agent A hasn't committed
yet, you can:
1. Start with the SQL migration (no Rust deps needed)
2. Write repository code against the type contracts below
3. Use todo!() for anything that needs Agent A's actual types
4. Write tests that will pass once Agent A delivers

Here are the types you'll use from Agent A (contract):

```rust
// From forge-core:
pub struct AgentId(pub Uuid);     // + Display, Clone, Eq, Hash, Serialize
pub struct SessionId(pub Uuid);
pub struct EventId(pub Uuid);
pub enum ForgeError { Database(..), Validation(..), AgentNotFound(..), .. }
pub type ForgeResult<T> = Result<T, ForgeError>;
pub enum ForgeEvent { AgentCreated{..}, Heartbeat{..}, ... } // 20 variants
// Serde format: #[serde(tag = "type", content = "data")]

// From forge-agent:
pub struct Agent { id, name, model, system_prompt, preset, created_at, updated_at, ... }
pub struct NewAgent { name, model?, system_prompt?, preset?, ... }
pub struct UpdateAgent { name?, model?, system_prompt??, ... }  // Option<Option<T>>
pub const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
```

═══════════════════════════════════════════════════════════════
TASK 1: SQL Schema (migrations/0001_init.sql)
═══════════════════════════════════════════════════════════════

This schema serves ALL phases. Design it upfront. Changes later are costly.

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
    id TEXT PRIMARY KEY,                              -- UUID v4
    name TEXT NOT NULL UNIQUE,
    model TEXT NOT NULL DEFAULT 'claude-sonnet-4-20250514',
    system_prompt TEXT,
    allowed_tools TEXT,                                -- JSON array or NULL
    max_turns INTEGER,
    use_max BOOLEAN NOT NULL DEFAULT 0,
    preset TEXT,                                       -- AgentPreset variant name
    config_json TEXT,                                  -- JSON object
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sessions
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    claude_session_id TEXT,                            -- for --resume
    directory TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'running', 'completed', 'failed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Events (append-only event log)
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,                          -- ForgeEvent variant name
    data_json TEXT NOT NULL,                           -- serialized event data
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
    definition_json TEXT NOT NULL,
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
    content TEXT NOT NULL,
    source_repo TEXT,
    parameters_json TEXT,
    examples_json TEXT,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- FTS5 virtual tables
CREATE VIRTUAL TABLE IF NOT EXISTS skills_fts USING fts5(
    name, description, category, content,
    content=skills, content_rowid=rowid
);

CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(
    directory, status,
    content=sessions, content_rowid=rowid
);

CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(
    event_type, data_json,
    content=events, content_rowid=rowid
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
    scope TEXT NOT NULL,
    key TEXT NOT NULL,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (scope, key)
);

INSERT INTO schema_version (version) VALUES (1);
```

═══════════════════════════════════════════════════════════════
TASK 2: forge-db Crate
═══════════════════════════════════════════════════════════════

File structure:
```
crates/forge-db/
  Cargo.toml
  src/
    lib.rs              # re-exports
    pool.rs             # DbPool (connection init, WAL mode)
    migrations.rs       # Migrator (version check, apply pending)
    batch_writer.rs     # BatchWriter (critical component)
    repos/
      mod.rs
      agents.rs         # AgentRepo CRUD
      events.rs         # EventRepo (insert_batch, queries)
```

### Cargo.toml

```toml
[package]
name = "forge-db"
version.workspace = true
edition.workspace = true

[dependencies]
forge-core = { path = "../forge-core" }
forge-agent = { path = "../forge-agent" }
rusqlite.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
tracing.workspace = true
crossbeam-channel.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

### pool.rs — Database connection management

```rust
use rusqlite::Connection;
use forge_core::error::ForgeResult;
use std::path::Path;
use std::sync::Mutex;

pub struct DbPool {
    conn: Mutex<Connection>,
}

impl DbPool {
    pub fn new(path: &Path) -> ForgeResult<Self> {
        let conn = Connection::open(path)?;

        // WAL mode for concurrent reads
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA cache_size = -8000;
        ")?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    /// In-memory DB for testing
    pub fn in_memory() -> ForgeResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("db mutex poisoned")
    }
}
```

NOTE: For Phase 0, a single Mutex<Connection> is sufficient. We don't
need r2d2 connection pooling yet. Keep it simple. The mutex ensures
serialized writes (which SQLite requires anyway with WAL for writers).

### migrations.rs — Schema migration runner

```rust
use rusqlite::Connection;
use forge_core::error::ForgeResult;
use tracing::info;

const MIGRATION_SQL: &str = include_str!("../../migrations/0001_init.sql");

pub struct Migrator<'a> {
    conn: &'a Connection,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn current_version(&self) -> ForgeResult<u32> {
        // Check if schema_version table exists
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(0);
        }

        let version: u32 = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )?;

        Ok(version)
    }

    pub fn apply_pending(&self) -> ForgeResult<u32> {
        let current = self.current_version()?;

        if current >= 1 {
            info!(version = current, "schema already at latest version");
            return Ok(0);
        }

        info!("applying migration 0001_init.sql");
        self.conn.execute_batch(MIGRATION_SQL)?;
        info!("migration applied, now at version 1");

        Ok(1) // number of migrations applied
    }
}
```

### batch_writer.rs — THE CRITICAL COMPONENT

This is what makes high-throughput event logging work. Events accumulate
in a crossbeam channel and flush to SQLite in batches.

```rust
use crossbeam_channel::{Receiver, Sender, bounded, select, tick};
use forge_core::events::ForgeEvent;
use forge_core::error::ForgeResult;
use rusqlite::Connection;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

const BATCH_SIZE: usize = 50;
const FLUSH_INTERVAL: Duration = Duration::from_secs(2);

pub struct BatchWriter {
    sender: Sender<ForgeEvent>,
    handle: Option<thread::JoinHandle<()>>,
}

impl BatchWriter {
    /// Spawn a dedicated writer thread.
    /// Flushes when: BATCH_SIZE events accumulated OR FLUSH_INTERVAL elapsed.
    pub fn spawn(conn: Mutex<Connection>) -> Self {
        let (sender, receiver) = bounded::<ForgeEvent>(1024);

        let handle = thread::spawn(move || {
            writer_loop(conn, receiver);
        });

        Self {
            sender,
            handle: Some(handle),
        }
    }

    /// Queue an event for batch writing. Non-blocking.
    pub fn write(&self, event: ForgeEvent) -> ForgeResult<()> {
        self.sender.send(event).map_err(|e| {
            forge_core::error::ForgeError::Internal(
                format!("batch writer channel closed: {}", e)
            )
        })
    }

    /// Flush remaining events and shut down the writer thread.
    pub fn shutdown(mut self) -> ForgeResult<()> {
        drop(self.sender); // Close channel, writer will drain and exit
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| {
                forge_core::error::ForgeError::Internal("batch writer thread panicked".into())
            })?;
        }
        Ok(())
    }
}

fn writer_loop(conn: Mutex<Connection>, receiver: Receiver<ForgeEvent>) {
    let mut buffer: Vec<ForgeEvent> = Vec::with_capacity(BATCH_SIZE);
    let ticker = tick(FLUSH_INTERVAL);

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(event) => {
                        buffer.push(event);
                        if buffer.len() >= BATCH_SIZE {
                            flush_to_db(&conn, &mut buffer);
                        }
                    }
                    Err(_) => {
                        // Channel closed — flush remaining and exit
                        if !buffer.is_empty() {
                            flush_to_db(&conn, &mut buffer);
                        }
                        info!("batch writer shutting down");
                        return;
                    }
                }
            }
            recv(ticker) -> _ => {
                if !buffer.is_empty() {
                    flush_to_db(&conn, &mut buffer);
                }
            }
        }
    }
}

fn flush_to_db(conn: &Mutex<Connection>, buffer: &mut Vec<ForgeEvent>) {
    let conn = conn.lock().expect("db mutex poisoned");
    let count = buffer.len();

    // Use a transaction for batch insert
    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(e) => {
            error!(error = %e, "failed to begin transaction");
            return;
        }
    };

    for event in buffer.iter() {
        let id = Uuid::new_v4().to_string();
        let event_type = event_type_name(event);
        let data_json = serde_json::to_string(event).unwrap_or_default();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Extract agent_id and session_id if present
        let (agent_id, session_id) = extract_ids(event);

        if let Err(e) = tx.execute(
            "INSERT INTO events (id, session_id, agent_id, event_type, data_json, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, session_id, agent_id, event_type, data_json, timestamp],
        ) {
            error!(error = %e, event_type = event_type, "failed to insert event");
        }
    }

    if let Err(e) = tx.commit() {
        error!(error = %e, "failed to commit batch");
    } else {
        debug!(count = count, "flushed events to db");
    }

    buffer.clear();
}

/// Extract the variant name as a string for the event_type column.
fn event_type_name(event: &ForgeEvent) -> &'static str {
    match event {
        ForgeEvent::SystemStarted { .. } => "SystemStarted",
        ForgeEvent::SystemStopped { .. } => "SystemStopped",
        ForgeEvent::Heartbeat { .. } => "Heartbeat",
        ForgeEvent::AgentCreated { .. } => "AgentCreated",
        ForgeEvent::AgentUpdated { .. } => "AgentUpdated",
        ForgeEvent::AgentDeleted { .. } => "AgentDeleted",
        ForgeEvent::ProcessStarted { .. } => "ProcessStarted",
        ForgeEvent::ProcessOutput { .. } => "ProcessOutput",
        ForgeEvent::ProcessCompleted { .. } => "ProcessCompleted",
        ForgeEvent::ProcessFailed { .. } => "ProcessFailed",
        ForgeEvent::SessionCreated { .. } => "SessionCreated",
        ForgeEvent::SessionResumed { .. } => "SessionResumed",
        ForgeEvent::WorkflowStarted { .. } => "WorkflowStarted",
        ForgeEvent::WorkflowStepCompleted { .. } => "WorkflowStepCompleted",
        ForgeEvent::WorkflowCompleted { .. } => "WorkflowCompleted",
        ForgeEvent::WorkflowFailed { .. } => "WorkflowFailed",
        ForgeEvent::CircuitBreakerTripped { .. } => "CircuitBreakerTripped",
        ForgeEvent::BudgetWarning { .. } => "BudgetWarning",
        ForgeEvent::BudgetExceeded { .. } => "BudgetExceeded",
        ForgeEvent::Error { .. } => "Error",
    }
}

fn extract_ids(event: &ForgeEvent) -> (Option<String>, Option<String>) {
    match event {
        ForgeEvent::AgentCreated { agent_id, .. } |
        ForgeEvent::AgentUpdated { agent_id, .. } |
        ForgeEvent::AgentDeleted { agent_id, .. } |
        ForgeEvent::CircuitBreakerTripped { agent_id, .. } => {
            (Some(agent_id.0.to_string()), None)
        }
        ForgeEvent::ProcessStarted { session_id, agent_id, .. } |
        ForgeEvent::SessionCreated { session_id, agent_id, .. } => {
            (Some(agent_id.0.to_string()), Some(session_id.0.to_string()))
        }
        ForgeEvent::ProcessOutput { session_id, .. } |
        ForgeEvent::ProcessCompleted { session_id, .. } |
        ForgeEvent::ProcessFailed { session_id, .. } |
        ForgeEvent::SessionResumed { session_id, .. } => {
            (None, Some(session_id.0.to_string()))
        }
        _ => (None, None),
    }
}
```

### repos/agents.rs — Agent CRUD repository

```rust
use forge_core::error::{ForgeError, ForgeResult};
use forge_core::ids::AgentId;
use forge_agent::model::{Agent, NewAgent, UpdateAgent, DEFAULT_MODEL};
use forge_agent::preset::AgentPreset;
use forge_agent::validation::validate_new_agent;
use rusqlite::Connection;
use chrono::Utc;
use uuid::Uuid;
use std::sync::Mutex;

pub struct AgentRepo {
    conn: Mutex<Connection>,  // Or shared reference — depends on pool design
}

impl AgentRepo {
    pub fn new(conn: Mutex<Connection>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewAgent) -> ForgeResult<Agent> {
        validate_new_agent(input)?;

        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = AgentId::new();
        let now = Utc::now();
        let model = input.model.as_deref().unwrap_or(DEFAULT_MODEL);
        let allowed_tools_json = input.allowed_tools.as_ref()
            .map(|t| serde_json::to_string(t).unwrap_or_default());
        let preset_str = input.preset.as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());
        let config_json = input.config.as_ref()
            .map(|c| serde_json::to_string(c).unwrap_or_default());

        conn.execute(
            "INSERT INTO agents (id, name, model, system_prompt, allowed_tools, max_turns, use_max, preset, config_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                id.0.to_string(),
                input.name,
                model,
                input.system_prompt,
                allowed_tools_json,
                input.max_turns,
                input.use_max.unwrap_or(false),
                preset_str,
                config_json,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        self.get(&id)
    }

    pub fn get(&self, id: &AgentId) -> ForgeResult<Agent> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, name, model, system_prompt, allowed_tools, max_turns, use_max, preset, config_json, created_at, updated_at
             FROM agents WHERE id = ?1"
        )?;

        stmt.query_row(rusqlite::params![id.0.to_string()], |row| {
            Ok(row_to_agent(row))
        }).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ForgeError::AgentNotFound(id.clone()),
            other => ForgeError::Database(other),
        })?
    }

    pub fn list(&self) -> ForgeResult<Vec<Agent>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, name, model, system_prompt, allowed_tools, max_turns, use_max, preset, config_json, created_at, updated_at
             FROM agents ORDER BY created_at DESC"
        )?;

        let agents = stmt.query_map([], |row| Ok(row_to_agent(row)))?.collect::<Result<Vec<_>, _>>()?;

        // Flatten inner Results
        agents.into_iter().collect()
    }

    pub fn update(&self, id: &AgentId, input: &UpdateAgent) -> ForgeResult<Agent> {
        // Verify agent exists
        let existing = self.get(id)?;

        let conn = self.conn.lock().expect("db mutex poisoned");
        let now = Utc::now();

        // Build SET clauses dynamically based on which fields are Some
        // ... (implement partial update logic)

        self.get(id)
    }

    pub fn delete(&self, id: &AgentId) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let rows = conn.execute("DELETE FROM agents WHERE id = ?1", rusqlite::params![id.0.to_string()])?;

        if rows == 0 {
            return Err(ForgeError::AgentNotFound(id.clone()));
        }

        Ok(())
    }
}

fn row_to_agent(row: &rusqlite::Row<'_>) -> Result<Agent, rusqlite::Error> {
    // Parse each column, deserialize JSON fields
    // Handle NULL for optional fields
    // Parse preset from string back to AgentPreset enum
    // Parse timestamps from RFC3339 strings
    todo!("implement row_to_agent parsing")
}
```

IMPORTANT: The row_to_agent function is where most bugs will be. Be
careful with:
- Parsing UUID from TEXT column
- Deserializing JSON strings back to Vec<String> / serde_json::Value
- Handling NULL columns → Option<T>
- Parsing AgentPreset from its serialized string form
- Parsing DateTime<Utc> from RFC3339 TEXT

### repos/events.rs — Event repository

```rust
pub struct EventRepo {
    conn: Mutex<Connection>,
}

impl EventRepo {
    pub fn query_by_session(&self, session_id: &SessionId) -> ForgeResult<Vec<StoredEvent>>;
    pub fn query_by_type(&self, event_type: &str, limit: usize) -> ForgeResult<Vec<StoredEvent>>;
    pub fn count(&self) -> ForgeResult<u64>;
}

pub struct StoredEvent {
    pub id: String,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub event_type: String,
    pub data_json: String,
    pub timestamp: String,
}
```

═══════════════════════════════════════════════════════════════
TESTS (ALL REQUIRED)
═══════════════════════════════════════════════════════════════

```rust
// Migration tests
#[test]
fn migration_applies_cleanly() {
    let db = DbPool::in_memory().unwrap();
    let migrator = Migrator::new(&db.connection());
    assert_eq!(migrator.apply_pending().unwrap(), 1);
}

#[test]
fn migration_is_idempotent() {
    let db = DbPool::in_memory().unwrap();
    let migrator = Migrator::new(&db.connection());
    migrator.apply_pending().unwrap();
    assert_eq!(migrator.apply_pending().unwrap(), 0); // no new migrations
}

#[test]
fn migration_version_tracked() {
    let db = DbPool::in_memory().unwrap();
    let migrator = Migrator::new(&db.connection());
    migrator.apply_pending().unwrap();
    assert_eq!(migrator.current_version().unwrap(), 1);
}

// Agent CRUD tests
#[test]
fn agent_crud_roundtrip() {
    let db = setup_test_db();
    let repo = AgentRepo::new(db);
    let input = NewAgent { name: "TestAgent".into(), ..defaults() };
    let created = repo.create(&input).unwrap();
    assert_eq!(created.name, "TestAgent");

    let fetched = repo.get(&created.id).unwrap();
    assert_eq!(fetched.name, "TestAgent");

    let agents = repo.list().unwrap();
    assert_eq!(agents.len(), 1);

    repo.delete(&created.id).unwrap();
    assert!(repo.get(&created.id).is_err());
}

#[test]
fn agent_name_unique() {
    let db = setup_test_db();
    let repo = AgentRepo::new(db);
    let input = NewAgent { name: "Unique".into(), ..defaults() };
    repo.create(&input).unwrap();
    // Second create with same name should fail
    assert!(repo.create(&input).is_err());
}

#[test]
fn fts5_tables_exist() {
    let db = setup_test_db();
    let conn = db.connection();
    // Verify FTS5 virtual tables were created
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name LIKE '%_fts'",
        [], |row| row.get(0),
    ).unwrap();
    assert!(count >= 3); // skills_fts, sessions_fts, events_fts
}

// Batch writer tests (CRITICAL)
#[test]
fn batch_writer_flushes_at_50() {
    let db = DbPool::in_memory().unwrap();
    let migrator = Migrator::new(&db.connection());
    migrator.apply_pending().unwrap();

    let writer = BatchWriter::spawn(/* conn */);

    // Write 50 events
    for i in 0..50 {
        writer.write(ForgeEvent::Heartbeat {
            timestamp: chrono::Utc::now(),
        }).unwrap();
    }

    // Give writer thread time to flush
    std::thread::sleep(Duration::from_millis(100));

    // Check DB has 50 events
    let count: i32 = db.connection().query_row(
        "SELECT COUNT(*) FROM events", [], |row| row.get(0),
    ).unwrap();
    assert_eq!(count, 50);
}

#[test]
fn batch_writer_flushes_at_2s() {
    let db = DbPool::in_memory().unwrap();
    // ... setup ...

    // Write 5 events (below BATCH_SIZE)
    for _ in 0..5 {
        writer.write(heartbeat_event()).unwrap();
    }

    // Wait 2.5 seconds for timer flush
    std::thread::sleep(Duration::from_millis(2500));

    // Check DB has 5 events (flushed by timer, not batch size)
    let count: i32 = /* query */;
    assert_eq!(count, 5);
}

#[test]
fn batch_writer_shutdown_flushes() {
    // Write 10 events, call shutdown(), verify all 10 in DB
}

#[test]
fn event_persisted_with_correct_type() {
    // Write an AgentCreated event, query DB, check event_type = "AgentCreated"
}
```

═══════════════════════════════════════════════════════════════
CODING STANDARDS
═══════════════════════════════════════════════════════════════

Naming:
- Crate names: forge-{name} (kebab-case)
- Types: PascalCase (DbPool, AgentRepo, BatchWriter, Migrator)
- Functions: snake_case (apply_pending, flush_to_db, row_to_agent)
- Constants: SCREAMING_SNAKE (BATCH_SIZE, FLUSH_INTERVAL)

Error handling:
- Use thiserror for ForgeError (from forge-core)
- Return ForgeResult<T> from all fallible public functions
- Never unwrap() in library code (except mutex locks with expect)
- Map rusqlite::Error::QueryReturnedNoRows → ForgeError::AgentNotFound

Logging (tracing crate):
- info! for migrations applied
- debug! for batch flushes with count
- error! for DB write failures
- Structured: debug!(count = 50, "flushed events to db")

Database rules:
- WAL mode always enabled
- Foreign keys always ON
- Transactions for batch writes
- In-memory DB for tests (DbPool::in_memory)
- All timestamps as RFC3339 TEXT (not INTEGER)
- All IDs as UUID TEXT (not INTEGER)

Testing:
- Use DbPool::in_memory() for all tests
- Each test creates its own DB (no shared state)
- Test names describe behavior
- Red-green-refactor

═══════════════════════════════════════════════════════════════
COMMIT SEQUENCE
═══════════════════════════════════════════════════════════════

1. "feat(db): implement schema, migrations, and pool"
2. "feat(db): implement batch writer with 50-event/2s flush"
3. "feat(db): implement AgentRepo CRUD"

═══════════════════════════════════════════════════════════════
REFERENCE REPOS TO STUDY BEFORE CODING
═══════════════════════════════════════════════════════════════

If available, read these first:
- refrence-repo/claude-code-tools/ — session storage, FTS5 patterns
- forge-project/03-architecture/DATA_MODEL.md — full schema reference
- forge-project/05-engineering/CODING_STANDARDS.md — naming, errors
```

</details>

---

## Agent C — Frontend (SvelteKit + Svelte 5 + Tailwind 4)

<details>
<summary>Click to expand full prompt (paste into Cursor)</summary>

```
You are building Claude Forge — a multi-agent Claude Code orchestrator.
Single Rust binary, embedded SvelteKit frontend, SQLite persistence.
Your role: Agent C — the entire frontend application.

═══════════════════════════════════════════════════════════════
WHAT IS CLAUDE FORGE
═══════════════════════════════════════════════════════════════

Claude Forge is a local-first GUI for orchestrating multiple Claude Code
agents. It wraps the `claude` CLI, adds persistence, real-time streaming,
workflow automation, and a web dashboard. The end result is a single
binary (`./forge`) that opens a browser, shows a dashboard, and lets
users create/run/monitor AI agents.

Phase 0 scope: Empty workspace → running binary with agent CRUD working
end-to-end (API + UI + DB + WebSocket events).

You are building the user interface. The Agents page must be FULLY
FUNCTIONAL — list, create, edit, delete. Other pages are empty shells.
WebSocket connects to the backend for real-time event streaming.

═══════════════════════════════════════════════════════════════
YOUR FILES (YOU OWN THESE — ONLY YOU TOUCH THEM)
═══════════════════════════════════════════════════════════════

frontend/**

═══════════════════════════════════════════════════════════════
DO NOT TOUCH (other agents own these)
═══════════════════════════════════════════════════════════════

Cargo.toml        → Agent D owns
crates/**         → Agents A, B, D own
migrations/**     → Agent B owns
Makefile          → Agent D owns

═══════════════════════════════════════════════════════════════
NO RUST DEPENDENCY
═══════════════════════════════════════════════════════════════

You work entirely from TypeScript type contracts. You call
fetch('/api/v1/agents') and expect JSON matching the types below.
During development, just build against the types — the Rust backend
(Agent D) will serve the matching API.

═══════════════════════════════════════════════════════════════
TECH STACK (EXACT VERSIONS)
═══════════════════════════════════════════════════════════════

- SvelteKit (latest) with adapter-static
- Svelte 5 runes: $state, $derived, $effect (NOT Svelte 4 stores)
- TailwindCSS 4 (NOT v3 — use @import "tailwindcss" not @tailwind)
- TypeScript 5.6+
- pnpm as package manager

CRITICAL SVELTE 5 RULES:
- Use $state() for reactive state, NOT writable() stores
- Use $derived() for computed values, NOT $: reactive
- Use $effect() for side effects, NOT onMount with stores
- Use onclick={handler} NOT on:click={handler}
- Use {#snippet name()}...{/snippet} NOT <slot>
- Event modifiers: onclick={(e) => { e.stopPropagation(); handler(); }}
  NOT on:click|stopPropagation={handler}
- Use {#key value}...{/key} to force component re-initialization

═══════════════════════════════════════════════════════════════
TASK 1: Project Setup
═══════════════════════════════════════════════════════════════

Create the SvelteKit project from scratch. File structure:

```
frontend/
  package.json
  svelte.config.js
  vite.config.ts
  tsconfig.json
  tailwind.config.js    (if needed for v4, or use CSS-only config)
  src/
    app.html
    app.css             # @import "tailwindcss";
    routes/
      +layout.svelte    # Sidebar + status bar + main content
      +page.svelte      # Dashboard
      agents/
        +page.svelte    # Agent list + CRUD (FULLY FUNCTIONAL)
      sessions/
        +page.svelte    # Empty shell
      workflows/
        +page.svelte    # Empty shell
      skills/
        +page.svelte    # Empty shell
      settings/
        +page.svelte    # Empty shell
    lib/
      types/
        index.ts        # TypeScript types matching Rust backend
      stores/
        websocket.svelte.ts   # WebSocket connection store (.svelte.ts for runes)
        agents.svelte.ts      # Agent CRUD store
      components/
        Sidebar.svelte
        StatusBar.svelte
        AgentCard.svelte
        AgentForm.svelte
```

### package.json key deps:

```json
{
  "devDependencies": {
    "@sveltejs/adapter-static": "latest",
    "@sveltejs/kit": "latest",
    "svelte": "latest",
    "svelte-check": "latest",
    "tailwindcss": "^4",
    "typescript": "^5.6",
    "vite": "latest"
  }
}
```

### svelte.config.js:

```js
import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',  // SPA mode — Rust serves this for all routes
    }),
  },
};
```

### vite.config.ts:

```ts
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:4173',  // Proxy to Rust backend in dev
      '/ws': { target: 'ws://127.0.0.1:4173', ws: true },
    },
  },
});
```

═══════════════════════════════════════════════════════════════
TASK 2: TypeScript Types (Contract — match exactly)
═══════════════════════════════════════════════════════════════

```typescript
// frontend/src/lib/types/index.ts

export interface Agent {
    id: string;
    name: string;
    model: string;
    system_prompt: string | null;
    allowed_tools: string[] | null;
    max_turns: number | null;
    use_max: boolean;
    preset: AgentPreset | null;
    config: Record<string, unknown> | null;
    created_at: string;   // RFC3339
    updated_at: string;   // RFC3339
}

export type AgentPreset =
    | 'CodeWriter' | 'Reviewer' | 'Tester' | 'Debugger' | 'Architect'
    | 'Documenter' | 'SecurityAuditor' | 'Refactorer' | 'Explorer';

export interface NewAgent {
    name: string;
    model?: string;
    system_prompt?: string;
    allowed_tools?: string[];
    max_turns?: number;
    use_max?: boolean;
    preset?: AgentPreset;
    config?: Record<string, unknown>;
}

export interface UpdateAgent {
    name?: string;
    model?: string;
    system_prompt?: string | null;
    allowed_tools?: string[] | null;
    max_turns?: number | null;
    use_max?: boolean;
    preset?: AgentPreset | null;
    config?: Record<string, unknown> | null;
}

export interface ForgeEvent {
    type: string;
    data: Record<string, unknown>;
}

export interface HealthResponse {
    status: string;
    version: string;
    uptime_secs: number;
}

export interface ApiError {
    error: string;
    code: string;
}

export const AGENT_PRESETS: AgentPreset[] = [
    'CodeWriter', 'Reviewer', 'Tester', 'Debugger', 'Architect',
    'Documenter', 'SecurityAuditor', 'Refactorer', 'Explorer',
];

export const DEFAULT_MODEL = 'claude-sonnet-4-20250514';

export const MODELS = [
    { value: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4' },
    { value: 'claude-haiku-4-5-20251001', label: 'Claude Haiku 4.5' },
    { value: 'claude-opus-4-6', label: 'Claude Opus 4.6' },
];
```

═══════════════════════════════════════════════════════════════
TASK 3: WebSocket Store (Svelte 5 runes)
═══════════════════════════════════════════════════════════════

```typescript
// frontend/src/lib/stores/websocket.svelte.ts

import type { ForgeEvent } from '$lib/types';

const MAX_EVENTS = 100;
const RECONNECT_DELAYS = [1000, 2000, 4000, 8000, 16000]; // exponential backoff

class WebSocketStore {
    connected = $state(false);
    events = $state<ForgeEvent[]>([]);
    lastEvent = $derived(this.events[this.events.length - 1] ?? null);
    eventCount = $derived(this.events.length);

    private ws: WebSocket | null = null;
    private reconnectAttempt = 0;
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const url = `${protocol}//${window.location.host}/api/v1/ws`;

        this.ws = new WebSocket(url);

        this.ws.onopen = () => {
            this.connected = true;
            this.reconnectAttempt = 0;
        };

        this.ws.onmessage = (event) => {
            try {
                const data: ForgeEvent = JSON.parse(event.data);
                this.events = [...this.events.slice(-MAX_EVENTS + 1), data];
            } catch {
                // Ignore malformed messages
            }
        };

        this.ws.onclose = () => {
            this.connected = false;
            this.scheduleReconnect();
        };

        this.ws.onerror = () => {
            this.ws?.close();
        };
    }

    disconnect() {
        if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
        this.ws?.close();
        this.ws = null;
        this.connected = false;
    }

    private scheduleReconnect() {
        const delay = RECONNECT_DELAYS[
            Math.min(this.reconnectAttempt, RECONNECT_DELAYS.length - 1)
        ];
        this.reconnectAttempt++;
        this.reconnectTimer = setTimeout(() => this.connect(), delay);
    }
}

export const wsStore = new WebSocketStore();
```

═══════════════════════════════════════════════════════════════
TASK 4: Agent Store (Svelte 5 runes)
═══════════════════════════════════════════════════════════════

```typescript
// frontend/src/lib/stores/agents.svelte.ts

import type { Agent, NewAgent, UpdateAgent, ApiError } from '$lib/types';

const API_BASE = '/api/v1';

class AgentStore {
    agents = $state<Agent[]>([]);
    loading = $state(false);
    error = $state<string | null>(null);

    async load() {
        this.loading = true;
        this.error = null;
        try {
            const res = await fetch(`${API_BASE}/agents`);
            if (!res.ok) throw await this.parseError(res);
            this.agents = await res.json();
        } catch (e) {
            this.error = e instanceof Error ? e.message : String(e);
        } finally {
            this.loading = false;
        }
    }

    async create(input: NewAgent): Promise<Agent | null> {
        this.error = null;
        try {
            const res = await fetch(`${API_BASE}/agents`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(input),
            });
            if (!res.ok) throw await this.parseError(res);
            const agent: Agent = await res.json();
            this.agents = [...this.agents, agent];
            return agent;
        } catch (e) {
            this.error = e instanceof Error ? e.message : String(e);
            return null;
        }
    }

    async update(id: string, input: UpdateAgent): Promise<Agent | null> {
        this.error = null;
        try {
            const res = await fetch(`${API_BASE}/agents/${id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(input),
            });
            if (!res.ok) throw await this.parseError(res);
            const agent: Agent = await res.json();
            this.agents = this.agents.map(a => a.id === id ? agent : a);
            return agent;
        } catch (e) {
            this.error = e instanceof Error ? e.message : String(e);
            return null;
        }
    }

    async remove(id: string): Promise<boolean> {
        this.error = null;
        try {
            const res = await fetch(`${API_BASE}/agents/${id}`, {
                method: 'DELETE',
            });
            if (!res.ok) throw await this.parseError(res);
            this.agents = this.agents.filter(a => a.id !== id);
            return true;
        } catch (e) {
            this.error = e instanceof Error ? e.message : String(e);
            return false;
        }
    }

    private async parseError(res: Response): Promise<Error> {
        try {
            const body: ApiError = await res.json();
            return new Error(body.error);
        } catch {
            return new Error(`HTTP ${res.status}`);
        }
    }
}

export const agentStore = new AgentStore();
```

═══════════════════════════════════════════════════════════════
TASK 5: Layout
═══════════════════════════════════════════════════════════════

### +layout.svelte

```svelte
<script lang="ts">
    import { wsStore } from '$lib/stores/websocket.svelte';
    import Sidebar from '$lib/components/Sidebar.svelte';
    import StatusBar from '$lib/components/StatusBar.svelte';
    import type { Snippet } from 'svelte';

    let { children }: { children: Snippet } = $props();

    $effect(() => {
        wsStore.connect();
        return () => wsStore.disconnect();
    });
</script>

<div class="flex h-screen bg-gray-950 text-gray-100">
    <Sidebar />
    <div class="flex flex-col flex-1 overflow-hidden">
        <StatusBar />
        <main class="flex-1 overflow-auto p-6">
            {@render children()}
        </main>
    </div>
</div>
```

### Sidebar.svelte

```svelte
<script lang="ts">
    import { page } from '$app/stores';

    const nav = [
        { href: '/', label: 'Dashboard', icon: '◆' },
        { href: '/agents', label: 'Agents', icon: '◉' },
        { href: '/sessions', label: 'Sessions', icon: '▶' },
        { href: '/workflows', label: 'Workflows', icon: '⟳' },
        { href: '/skills', label: 'Skills', icon: '★' },
        { href: '/settings', label: 'Settings', icon: '⚙' },
    ];
</script>

<aside class="w-56 bg-gray-900 border-r border-gray-800 flex flex-col">
    <div class="p-4 border-b border-gray-800">
        <h1 class="text-lg font-bold text-white">Claude Forge</h1>
        <p class="text-xs text-gray-500">v0.1.0</p>
    </div>
    <nav class="flex-1 p-2 space-y-1">
        {#each nav as item}
            <a
                href={item.href}
                class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors
                    {$page.url.pathname === item.href
                        ? 'bg-blue-600/20 text-blue-400'
                        : 'text-gray-400 hover:bg-gray-800 hover:text-gray-200'}"
            >
                <span class="text-base">{item.icon}</span>
                {item.label}
            </a>
        {/each}
    </nav>
</aside>
```

### StatusBar.svelte

```svelte
<script lang="ts">
    import { wsStore } from '$lib/stores/websocket.svelte';
</script>

<header class="h-10 bg-gray-900 border-b border-gray-800 flex items-center px-4 justify-between text-xs">
    <div class="flex items-center gap-2">
        <span class="w-2 h-2 rounded-full {wsStore.connected ? 'bg-green-500' : 'bg-red-500'}"></span>
        <span class="text-gray-400">
            {wsStore.connected ? 'Connected' : 'Disconnected'}
        </span>
    </div>
    <div class="text-gray-500">
        {wsStore.eventCount} events
    </div>
</header>
```

═══════════════════════════════════════════════════════════════
TASK 6: Dashboard Page
═══════════════════════════════════════════════════════════════

```svelte
<!-- src/routes/+page.svelte -->
<script lang="ts">
    import { wsStore } from '$lib/stores/websocket.svelte';
</script>

<div class="max-w-4xl">
    <h1 class="text-2xl font-bold mb-6">Dashboard</h1>

    <div class="grid grid-cols-3 gap-4 mb-8">
        <div class="bg-gray-900 rounded-lg p-4 border border-gray-800">
            <p class="text-xs text-gray-500 uppercase tracking-wide">Status</p>
            <p class="text-lg font-semibold mt-1">
                {wsStore.connected ? 'Online' : 'Offline'}
            </p>
        </div>
        <div class="bg-gray-900 rounded-lg p-4 border border-gray-800">
            <p class="text-xs text-gray-500 uppercase tracking-wide">Version</p>
            <p class="text-lg font-semibold mt-1">0.1.0</p>
        </div>
        <div class="bg-gray-900 rounded-lg p-4 border border-gray-800">
            <p class="text-xs text-gray-500 uppercase tracking-wide">Events</p>
            <p class="text-lg font-semibold mt-1">{wsStore.eventCount}</p>
        </div>
    </div>

    <h2 class="text-lg font-semibold mb-3">Recent Events</h2>
    <div class="space-y-2">
        {#each wsStore.events.slice(-10).reverse() as event}
            <div class="bg-gray-900 rounded px-3 py-2 text-sm font-mono border border-gray-800">
                <span class="text-blue-400">{event.type}</span>
                <span class="text-gray-500 ml-2">{JSON.stringify(event.data).slice(0, 80)}</span>
            </div>
        {:else}
            <p class="text-gray-500 text-sm">No events yet. Waiting for connection...</p>
        {/each}
    </div>
</div>
```

═══════════════════════════════════════════════════════════════
TASK 7: Agents Page (FULLY FUNCTIONAL — not a shell)
═══════════════════════════════════════════════════════════════

This is the main deliverable. Full CRUD.

### agents/+page.svelte

```svelte
<script lang="ts">
    import { agentStore } from '$lib/stores/agents.svelte';
    import AgentCard from '$lib/components/AgentCard.svelte';
    import AgentForm from '$lib/components/AgentForm.svelte';
    import type { Agent } from '$lib/types';

    let showForm = $state(false);
    let editingAgent = $state<Agent | null>(null);
    let deletingAgent = $state<Agent | null>(null);

    $effect(() => {
        agentStore.load();
    });

    function openCreate() {
        editingAgent = null;
        showForm = true;
    }

    function openEdit(agent: Agent) {
        editingAgent = agent;
        showForm = true;
    }

    async function handleSave(data: any) {
        if (editingAgent) {
            await agentStore.update(editingAgent.id, data);
        } else {
            await agentStore.create(data);
        }
        showForm = false;
        editingAgent = null;
    }

    async function confirmDelete() {
        if (deletingAgent) {
            await agentStore.remove(deletingAgent.id);
            deletingAgent = null;
        }
    }
</script>

<div class="max-w-6xl">
    <div class="flex items-center justify-between mb-6">
        <h1 class="text-2xl font-bold">Agents</h1>
        <button
            onclick={openCreate}
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm font-medium transition-colors"
        >
            + New Agent
        </button>
    </div>

    {#if agentStore.error}
        <div class="bg-red-900/30 border border-red-800 rounded-lg p-3 mb-4 text-sm text-red-300">
            {agentStore.error}
        </div>
    {/if}

    {#if agentStore.loading}
        <p class="text-gray-500">Loading agents...</p>
    {:else if agentStore.agents.length === 0}
        <div class="text-center py-12 text-gray-500">
            <p class="text-lg mb-2">No agents yet</p>
            <p class="text-sm">Create your first agent to get started.</p>
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each agentStore.agents as agent (agent.id)}
                <AgentCard
                    {agent}
                    onEdit={() => openEdit(agent)}
                    onDelete={() => deletingAgent = agent}
                />
            {/each}
        </div>
    {/if}
</div>

<!-- Create/Edit Modal -->
{#if showForm}
    <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
         onclick={() => showForm = false}>
        <div class="bg-gray-900 rounded-xl border border-gray-700 p-6 w-full max-w-lg mx-4"
             onclick={(e) => e.stopPropagation()}>
            <h2 class="text-lg font-semibold mb-4">
                {editingAgent ? 'Edit Agent' : 'New Agent'}
            </h2>
            <AgentForm
                agent={editingAgent}
                onSave={handleSave}
                onCancel={() => showForm = false}
            />
        </div>
    </div>
{/if}

<!-- Delete Confirmation -->
{#if deletingAgent}
    <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
        <div class="bg-gray-900 rounded-xl border border-gray-700 p-6 w-full max-w-sm mx-4">
            <h2 class="text-lg font-semibold mb-2">Delete Agent</h2>
            <p class="text-gray-400 text-sm mb-4">
                Are you sure you want to delete "{deletingAgent.name}"? This cannot be undone.
            </p>
            <div class="flex gap-3 justify-end">
                <button onclick={() => deletingAgent = null}
                    class="px-4 py-2 bg-gray-800 hover:bg-gray-700 rounded-lg text-sm">
                    Cancel
                </button>
                <button onclick={confirmDelete}
                    class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg text-sm">
                    Delete
                </button>
            </div>
        </div>
    </div>
{/if}
```

### AgentCard.svelte

```svelte
<script lang="ts">
    import type { Agent } from '$lib/types';

    let { agent, onEdit, onDelete }: {
        agent: Agent;
        onEdit: () => void;
        onDelete: () => void;
    } = $props();

    const presetColors: Record<string, string> = {
        CodeWriter: 'bg-green-900/30 text-green-400',
        Reviewer: 'bg-purple-900/30 text-purple-400',
        Tester: 'bg-yellow-900/30 text-yellow-400',
        Debugger: 'bg-red-900/30 text-red-400',
        Architect: 'bg-blue-900/30 text-blue-400',
        Documenter: 'bg-cyan-900/30 text-cyan-400',
        SecurityAuditor: 'bg-orange-900/30 text-orange-400',
        Refactorer: 'bg-pink-900/30 text-pink-400',
        Explorer: 'bg-indigo-900/30 text-indigo-400',
    };
</script>

<div class="bg-gray-900 rounded-lg border border-gray-800 p-4 hover:border-gray-700 transition-colors">
    <div class="flex items-start justify-between mb-2">
        <h3 class="font-semibold text-white">{agent.name}</h3>
        <div class="flex gap-1">
            <button onclick={onEdit} class="text-gray-500 hover:text-gray-300 text-sm px-1">Edit</button>
            <button onclick={onDelete} class="text-gray-500 hover:text-red-400 text-sm px-1">Delete</button>
        </div>
    </div>

    <p class="text-xs text-gray-500 mb-3">{agent.model}</p>

    {#if agent.preset}
        <span class="inline-block px-2 py-0.5 rounded text-xs {presetColors[agent.preset] ?? 'bg-gray-800 text-gray-400'}">
            {agent.preset}
        </span>
    {/if}

    {#if agent.system_prompt}
        <p class="text-xs text-gray-500 mt-2 line-clamp-2">{agent.system_prompt}</p>
    {/if}
</div>
```

### AgentForm.svelte

```svelte
<script lang="ts">
    import type { Agent, NewAgent, AgentPreset } from '$lib/types';
    import { AGENT_PRESETS, MODELS, DEFAULT_MODEL } from '$lib/types';

    let { agent, onSave, onCancel }: {
        agent: Agent | null;
        onSave: (data: NewAgent) => void;
        onCancel: () => void;
    } = $props();

    let name = $state(agent?.name ?? '');
    let model = $state(agent?.model ?? DEFAULT_MODEL);
    let preset = $state<AgentPreset | ''>(agent?.preset ?? '');
    let systemPrompt = $state(agent?.system_prompt ?? '');

    function handleSubmit(e: Event) {
        e.preventDefault();
        const data: NewAgent = {
            name: name.trim(),
            model,
            preset: preset || undefined,
            system_prompt: systemPrompt || undefined,
        };
        onSave(data);
    }
</script>

<form onsubmit={handleSubmit} class="space-y-4">
    <div>
        <label for="name" class="block text-sm text-gray-400 mb-1">Name *</label>
        <input id="name" bind:value={name} required
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                   focus:outline-none focus:border-blue-500"
            placeholder="My Agent" />
    </div>

    <div>
        <label for="model" class="block text-sm text-gray-400 mb-1">Model</label>
        <select id="model" bind:value={model}
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white">
            {#each MODELS as m}
                <option value={m.value}>{m.label}</option>
            {/each}
        </select>
    </div>

    <div>
        <label for="preset" class="block text-sm text-gray-400 mb-1">Preset</label>
        <select id="preset" bind:value={preset}
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white">
            <option value="">None</option>
            {#each AGENT_PRESETS as p}
                <option value={p}>{p}</option>
            {/each}
        </select>
    </div>

    <div>
        <label for="prompt" class="block text-sm text-gray-400 mb-1">System Prompt</label>
        <textarea id="prompt" bind:value={systemPrompt} rows={4}
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                   focus:outline-none focus:border-blue-500 resize-y"
            placeholder="Optional system instructions..."></textarea>
    </div>

    <div class="flex gap-3 justify-end pt-2">
        <button type="button" onclick={onCancel}
            class="px-4 py-2 bg-gray-800 hover:bg-gray-700 rounded-lg text-sm">
            Cancel
        </button>
        <button type="submit" disabled={!name.trim()}
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 rounded-lg text-sm font-medium">
            {agent ? 'Save Changes' : 'Create Agent'}
        </button>
    </div>
</form>
```

═══════════════════════════════════════════════════════════════
TASK 8: Empty Shell Pages
═══════════════════════════════════════════════════════════════

Each empty page follows the same pattern:

```svelte
<!-- src/routes/sessions/+page.svelte -->
<div class="max-w-4xl">
    <h1 class="text-2xl font-bold mb-4">Sessions</h1>
    <div class="bg-gray-900 rounded-lg border border-gray-800 p-8 text-center">
        <p class="text-gray-500">Coming in Phase 1</p>
        <p class="text-xs text-gray-600 mt-1">Session management with resume and export</p>
    </div>
</div>
```

Create similar pages for:
- /workflows → "Coming in Phase 2"
- /skills → "Coming in Phase 2"
- /settings → "Coming in Phase 5"

═══════════════════════════════════════════════════════════════
TASK 9: Verify
═══════════════════════════════════════════════════════════════

Run these checks before committing:
- `pnpm check` (svelte-check) passes with no errors
- `pnpm build` produces output in frontend/build/
- Build output includes index.html

═══════════════════════════════════════════════════════════════
DESIGN SYSTEM
═══════════════════════════════════════════════════════════════

Colors (dark theme, consistent):
- Background: gray-950 (page), gray-900 (cards/panels), gray-800 (inputs)
- Borders: gray-800 (default), gray-700 (hover/focus)
- Text: white (headings), gray-100 (body), gray-400 (labels), gray-500 (muted)
- Accent: blue-600 (primary buttons), blue-400 (active nav)
- Danger: red-600 (delete buttons), red-400 (error text)
- Status: green-500 (connected), red-500 (disconnected)

Typography:
- Headings: font-bold
- Body: text-sm
- Labels: text-xs text-gray-400 uppercase tracking-wide
- Code/events: font-mono text-sm

Spacing:
- Page padding: p-6
- Card padding: p-4
- Section gaps: mb-6
- Grid gaps: gap-4

Interactions:
- Buttons: hover state with slightly lighter bg, transition-colors
- Cards: hover:border-gray-700
- Links: text color change on hover
- Disabled: opacity-50

═══════════════════════════════════════════════════════════════
COMMIT SEQUENCE
═══════════════════════════════════════════════════════════════

1. "feat(frontend): implement SvelteKit project with layout and stores"
2. "feat(frontend): implement agent CRUD UI with cards and forms"

═══════════════════════════════════════════════════════════════
REFERENCE REPOS TO STUDY BEFORE CODING
═══════════════════════════════════════════════════════════════

If available, read these first:
- refrence-repo/claude-code-webui/ — web UI patterns, streaming
- refrence-repo/1code/ — multi-agent desktop UI, tab management
- forge-project/05-engineering/CODING_STANDARDS.md — naming conventions
```

</details>

---

## Quick Reference: Who Owns What

```
File/Directory                    Owner
─────────────────────────────────────────
Cargo.toml (root)                 Agent D
Makefile                          Agent D
.github/                          Agent D
crates/forge-core/**              Agent A
crates/forge-agent/**             Agent A
crates/forge-db/**                Agent B
migrations/**                     Agent B
crates/forge-api/**               Agent D
crates/forge-app/**               Agent D
crates/forge-process/**           Agent D (stub)
crates/forge-safety/**            Agent D (stub)
crates/forge-mcp/**               Agent D (stub)
frontend/**                       Agent C
forge-project/**                  NONE (read-only reference)
```

## Execution Order

```
Step 1:  Agent D creates scaffold → commit → push
Step 2:  Agents A, B, C start in parallel
Step 3:  Agent D builds forge-api + forge-app once A & B deliver
Step 4:  Integration test: cargo test --workspace && make build && ./forge
```

## Integration Checklist

```
[ ] Agent A: forge-core + forge-agent compile, tests pass
[ ] Agent B: forge-db compiles, migrations apply, batch writer works
[ ] Agent C: frontend builds, svelte-check passes
[ ] Agent D: scaffold + stubs + forge-api + forge-app

Final verification:
[ ] cargo test --workspace — all pass
[ ] cargo clippy --workspace — clean
[ ] make build — single binary produced
[ ] ./forge starts → browser shows UI
[ ] WebSocket connects → heartbeat events flow
[ ] Create agent via UI → stored in DB → visible in list
[ ] Edit agent → changes persist
[ ] Delete agent → removed from list
[ ] Events flowing in status bar
```
