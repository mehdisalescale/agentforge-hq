//! Map ForgeError to HTTP response and shared error helpers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use forge_core::error::ForgeError;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    code: String,
}

/// Convert ForgeError to an HTTP response. Use in handlers with .map_err(api_error).
pub fn api_error(e: ForgeError) -> Response {
    let (status, code) = match &e {
        ForgeError::AgentNotFound(_) => (StatusCode::NOT_FOUND, "agent_not_found"),
        ForgeError::Validation(_) => (StatusCode::BAD_REQUEST, "validation"),
        ForgeError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database"),
        ForgeError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, "serialization"),
        ForgeError::EventBus(_) => (StatusCode::INTERNAL_SERVER_ERROR, "event_bus"),
        ForgeError::SessionNotFound(_) => (StatusCode::NOT_FOUND, "session_not_found"),
        ForgeError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "io"),
        ForgeError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
    };
    if status.is_server_error() {
        tracing::error!(http_status = %status, error_code = code, error = %e, "API server error");
    }
    (
        status,
        Json(ErrorBody {
            error: e.to_string(),
            code: code.to_string(),
        }),
    )
        .into_response()
}

/// Parse a UUID string, returning a 400 BAD_REQUEST response on failure.
pub fn parse_uuid(s: &str) -> Result<uuid::Uuid, Response> {
    uuid::Uuid::parse_str(s).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: "invalid uuid".to_string(),
                code: "invalid_id".to_string(),
            }),
        )
            .into_response()
    })
}
