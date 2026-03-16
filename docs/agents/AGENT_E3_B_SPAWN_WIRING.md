# Agent E3-B: Wire BackendRegistry into SpawnMiddleware

## Goal

Replace the hardcoded `spawn()` call in `SpawnMiddleware` with `BackendRegistry` dispatch, so the middleware spawns via the agent's configured backend.

## Context

**Already done:**
- `ProcessBackend` trait in `crates/forge-process/src/backend.rs`
- `ClaudeBackend` in `crates/forge-process/src/claude_backend.rs`
- `BackendRegistry` with register/get/list/health_check_all in `backend.rs`

**Currently:** `SpawnMiddleware` (in `crates/forge-api/src/middleware.rs`, lines ~565-730) calls `spawn(&config, &prompt, session_id)` directly — hardwired to Claude CLI.

**After this agent:** `SpawnMiddleware` holds a `BackendRegistry`, looks up the agent's `backend_type`, and calls `backend.spawn(BackendSpawnConfig { ... })`.

Corresponds to **E3-S3** (Backend Routing in Middleware) from the epic doc.

## Files to Modify

### 1. `crates/forge-api/src/middleware.rs` — SpawnMiddleware

**Current struct:**
```rust
pub struct SpawnMiddleware {
    // possibly just uses SpawnConfig::from_env()
}
```

**New struct:**
```rust
pub struct SpawnMiddleware {
    registry: Arc<BackendRegistry>,
}

impl SpawnMiddleware {
    pub fn new(registry: Arc<BackendRegistry>) -> Self {
        Self { registry }
    }
}
```

**In `process()` method:**

Replace:
```rust
let config = SpawnConfig::from_env();
let handle = spawn(&config, &prompt, resume_arg).await?;
```

With:
```rust
let backend_name = ctx.agent.backend_type.as_deref().unwrap_or("claude");
let backend = self.registry.get(backend_name)
    .ok_or_else(|| MiddlewareError::SpawnFailed(
        format!("unknown backend: {backend_name}")
    ))?;
let handle = backend.spawn(&BackendSpawnConfig {
    prompt: ctx.prompt.clone(),
    working_dir: ctx.working_dir.clone(),
    model: None,
    max_turns: None,
    allowed_tools: vec![],
    system_prompt: ctx.system_prompt.clone(),
    resume_session_id: ctx.resume_session_id.clone(),
    env: HashMap::new(),
}).await.map_err(|e| MiddlewareError::SpawnFailed(e.to_string()))?;
```

The rest of the streaming/parsing/event-emission logic stays the same — `ProcessHandle` is the common return type from all backends.

### 2. `crates/forge-api/src/routes/run.rs` — Middleware chain construction

Where `SpawnMiddleware` is created, pass the registry:
```rust
let registry = Arc::clone(&state.backend_registry);
let spawn_mw = SpawnMiddleware::new(registry);
```

### 3. `crates/forge-api/src/state.rs` — Add registry to AppState

```rust
pub struct AppState {
    // ... existing fields ...
    pub backend_registry: Arc<BackendRegistry>,
}
```

### 4. `crates/forge-app/src/main.rs` — Construct and register backends

```rust
use forge_process::{BackendRegistry, ClaudeBackend, SpawnConfig};

let mut registry = BackendRegistry::new("claude");
registry.register(Box::new(ClaudeBackend::new(SpawnConfig::from_env())));
// Future: registry.register(Box::new(HermesBackend::new(...)));

let state = AppState {
    // ...
    backend_registry: Arc::new(registry),
};
```

### 5. `crates/forge-api/src/middleware.rs` — RunContext

Ensure `RunContext` carries `backend_type` from the agent:
```rust
pub struct RunContext {
    pub agent: Agent,  // Agent already has backend_type after E3-A
    // ... rest unchanged
}
```

### 6. Add `MiddlewareError::BackendUnavailable` variant

```rust
pub enum MiddlewareError {
    // ... existing ...
    BackendUnavailable(String),
}
```

Map to HTTP 503 in the error response.

## Depends On
- **Agent E3-A must complete first** (Agent model needs `backend_type`)

## Do NOT Modify
- `crates/forge-process/src/backend.rs` — already done
- `crates/forge-process/src/claude_backend.rs` — already done
- Frontend — Agent E3-A handles the UI
- Health endpoints — Agent E3-C handles those

## Verification
```bash
cargo check -p forge-api   # middleware compiles with registry
cargo test -p forge-api    # existing run tests still pass
cargo test                 # full workspace green
```

## Zero Warnings Policy
All modified files must produce zero warnings.
