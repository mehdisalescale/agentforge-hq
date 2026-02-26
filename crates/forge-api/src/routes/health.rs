//! Health check endpoint.

use axum::{routing::get, Json, Router};
use serde::Serialize;
use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn uptime_secs() -> u64 {
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs()
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime_secs(),
    })
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
