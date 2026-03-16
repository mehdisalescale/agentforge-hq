# Agent E3-C: Health Dashboard Endpoint + MCP Backend Tools

## Goal

Add `GET /api/v1/backends` health endpoint and MCP tools for backend discovery, so operators and AI agents can check which backends are available.

## Context

Corresponds to **E3-S5** (Backend Health Dashboard) from the epic doc.

`BackendRegistry` already has `health_check_all()` and `list_backends()` methods in `crates/forge-process/src/backend.rs`.

## Files to Create/Modify

### 1. New route file: `crates/forge-api/src/routes/backends.rs`

```rust
use axum::{extract::State, routing::get, Json, Router};
use crate::state::AppState;
use serde::Serialize;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/backends", get(list_backends))
        .route("/backends/health", get(health_check))
}

#[derive(Serialize)]
struct BackendInfo {
    name: String,
    capabilities: BackendCapabilities,
}

async fn list_backends(State(state): State<AppState>) -> Json<Vec<BackendInfo>> {
    let names = state.backend_registry.list_backends();
    let result: Vec<BackendInfo> = names.iter().map(|name| {
        let backend = state.backend_registry.get(name).unwrap();
        BackendInfo {
            name: name.clone(),
            capabilities: backend.capabilities(),
        }
    }).collect();
    Json(result)
}

#[derive(Serialize)]
struct HealthReport {
    name: String,
    status: String,  // "healthy", "degraded", "unavailable"
    message: Option<String>,
}

async fn health_check(State(state): State<AppState>) -> Json<Vec<HealthReport>> {
    let checks = state.backend_registry.health_check_all().await;
    let result: Vec<HealthReport> = checks.into_iter().map(|(name, health)| {
        match health {
            BackendHealth::Healthy => HealthReport { name, status: "healthy".into(), message: None },
            BackendHealth::Degraded(msg) => HealthReport { name, status: "degraded".into(), message: Some(msg) },
            BackendHealth::Unavailable(msg) => HealthReport { name, status: "unavailable".into(), message: Some(msg) },
        }
    }).collect();
    Json(result)
}
```

### 2. Register routes: `crates/forge-api/src/routes/mod.rs`

Add `pub mod backends;` and mount under `/api/v1`.

### 3. MCP tools: `crates/forge-mcp-bin/src/main.rs`

Add two new tools:

**`forge_list_backends`** — List available backends with capabilities
```rust
#[tool(description = "List available execution backends and their capabilities")]
async fn forge_list_backends(&self) -> Result<CallToolResult, McpError> {
    let names = self.backend_registry.list_backends();
    // Return JSON with name + capabilities for each
}
```

**`forge_backend_health`** — Check health of all backends
```rust
#[tool(description = "Check health status of all execution backends")]
async fn forge_backend_health(&self) -> Result<CallToolResult, McpError> {
    let health = self.backend_registry.health_check_all().await;
    // Return JSON with name + status for each
}
```

### 4. Update MCP server struct

Add `backend_registry: Arc<BackendRegistry>` to `ForgeMcp` struct and its constructor.

### 5. Frontend: `frontend/src/routes/backends/+page.svelte` (new page)

Simple dashboard showing:
- Table of backends with name, status badge (green/yellow/red), capabilities
- Auto-refresh every 30 seconds
- Add to sidebar navigation in `+layout.svelte`

## Depends On
- **Agent E3-B must complete** (BackendRegistry in AppState)

## Verification
```bash
cargo check                # compiles
cargo test                 # all green
# Manual: curl http://127.0.0.1:4173/api/v1/backends
# Manual: curl http://127.0.0.1:4173/api/v1/backends/health
```

## Zero Warnings Policy
All modified files must produce zero warnings.
