//! Backend discovery and health: GET /api/v1/backends, GET /api/v1/backends/health

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/backends", get(list_backends))
        .route("/backends/health", get(health_check))
}

#[derive(Serialize)]
struct BackendInfo {
    name: String,
    capabilities: forge_process::BackendCapabilities,
}

async fn list_backends(State(state): State<AppState>) -> Json<Vec<BackendInfo>> {
    let names = state.backend_registry.list_backends();
    let result: Vec<BackendInfo> = names
        .iter()
        .filter_map(|name| {
            state.backend_registry.get(name).map(|backend| BackendInfo {
                name: name.clone(),
                capabilities: backend.capabilities(),
            })
        })
        .collect();
    Json(result)
}

#[derive(Serialize)]
struct HealthReport {
    name: String,
    status: String,
    message: Option<String>,
}

async fn health_check(State(state): State<AppState>) -> Json<Vec<HealthReport>> {
    let checks = state.backend_registry.health_check_all().await;
    let result: Vec<HealthReport> = checks
        .into_iter()
        .map(|(name, health)| match health {
            forge_process::BackendHealth::Healthy => HealthReport {
                name,
                status: "healthy".into(),
                message: None,
            },
            forge_process::BackendHealth::Degraded(msg) => HealthReport {
                name,
                status: "degraded".into(),
                message: Some(msg),
            },
            forge_process::BackendHealth::Unavailable(msg) => HealthReport {
                name,
                status: "unavailable".into(),
                message: Some(msg),
            },
        })
        .collect();
    Json(result)
}
