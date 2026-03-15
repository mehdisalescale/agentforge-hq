//! Settings API: GET /api/v1/settings — return current runtime configuration.

use axum::{routing::get, Json, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/settings", get(get_settings))
}

async fn get_settings() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "host": std::env::var("FORGE_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
        "port": std::env::var("FORGE_PORT").unwrap_or_else(|_| "4173".into()),
        "cli_command": std::env::var("FORGE_CLI_COMMAND").unwrap_or_else(|_| "claude".into()),
        "db_path": std::env::var("FORGE_DB_PATH").unwrap_or_else(|_| "~/.agentforge/forge.db".into()),
        "rate_limit_max": std::env::var("FORGE_RATE_LIMIT_MAX").unwrap_or_else(|_| "10".into()),
        "rate_limit_refill_ms": std::env::var("FORGE_RATE_LIMIT_REFILL_MS").unwrap_or_else(|_| "1000".into()),
        "budget_warn": std::env::var("FORGE_BUDGET_WARN").ok(),
        "budget_limit": std::env::var("FORGE_BUDGET_LIMIT").ok(),
        "cors_origin": std::env::var("FORGE_CORS_ORIGIN").unwrap_or_else(|_| "*".into()),
    }))
}
