//! Hook CRUD + HookReceiver endpoints for Claude Code event capture.
//!
//! Legacy: GET/POST /api/v1/hooks, GET/PUT/DELETE /api/v1/hooks/:id
//! New:    POST /api/v1/hooks/pre-tool, /hooks/post-tool, /hooks/stop

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::api_error;
use crate::state::AppState;

use forge_core::events::ForgeEvent;
use forge_core::ids::SessionId;
use forge_db::repos::hooks::{Hook, NewHook, UpdateHook};
use forge_safety::scanner::SecurityScanner;

// ---------------------------------------------------------------------------
// Route registration
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        // HookReceiver endpoints (specific paths first to avoid :id capture)
        .route("/hooks/pre-tool", post(pre_tool_hook))
        .route("/hooks/post-tool", post(post_tool_hook))
        .route("/hooks/stop", post(stop_hook))
        // Legacy CRUD
        .route("/hooks", get(list_hooks).post(create_hook))
        .route(
            "/hooks/:id",
            get(get_hook).put(update_hook).delete(delete_hook),
        )
}

// ---------------------------------------------------------------------------
// HookReceiver: pre-tool
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PreToolPayload {
    pub session_id: String,
    pub tool_name: String,
}

#[derive(Debug, Serialize)]
pub struct PreToolResponse {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

async fn pre_tool_hook(
    State(state): State<AppState>,
    Json(payload): Json<PreToolPayload>,
) -> Json<PreToolResponse> {
    // Emit observability event; always allow for now.
    let _ = state.event_bus.emit(ForgeEvent::ToolUseRequested {
        session_id: SessionId(parse_uuid_silent(&payload.session_id)),
        tool_name: payload.tool_name.clone(),
        timestamp: chrono::Utc::now(),
    }).await;

    Json(PreToolResponse {
        allowed: true,
        reason: None,
    })
}

// ---------------------------------------------------------------------------
// HookReceiver: post-tool
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PostToolPayload {
    pub session_id: String,
    pub tool_name: String,
    pub tool_output: Option<String>,
}

async fn post_tool_hook(
    State(state): State<AppState>,
    Json(payload): Json<PostToolPayload>,
) -> StatusCode {
    let session_id = SessionId(parse_uuid_silent(&payload.session_id));

    // Emit tool-completed event
    let _ = state.event_bus.emit(ForgeEvent::ToolUseCompleted {
        session_id: session_id.clone(),
        tool_name: payload.tool_name.clone(),
        timestamp: chrono::Utc::now(),
    }).await;

    // Security scan on tool output (migrated from SecurityScanMiddleware)
    if let Some(ref output) = payload.tool_output {
        let scanner = SecurityScanner::new();
        let code_blocks = extract_code_blocks(output);
        for block in &code_blocks {
            let findings = scanner.scan(block);
            if !findings.is_empty() {
                let finding_strs: Vec<String> = findings
                    .iter()
                    .map(|f| {
                        format!(
                            "[{:?}] {} (line {}): {}",
                            f.severity, f.pattern, f.line, f.description
                        )
                    })
                    .collect();
                let _ = state.event_bus.emit(ForgeEvent::SecurityScanFailed {
                    session_id: session_id.clone(),
                    findings: finding_strs,
                    timestamp: chrono::Utc::now(),
                }).await;
            }
        }
    }

    StatusCode::OK
}

// ---------------------------------------------------------------------------
// HookReceiver: stop
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct StopPayload {
    pub session_id: String,
}

async fn stop_hook(
    State(state): State<AppState>,
    Json(payload): Json<StopPayload>,
) -> StatusCode {
    if let Ok(sid) = uuid::Uuid::parse_str(&payload.session_id) {
        let session_id = SessionId(sid);

        // Update session status
        let _ = state.uow.session_repo.update_status(&session_id, "completed");

        let _ = state.event_bus.emit(ForgeEvent::SessionCompleted {
            session_id,
            exit_code: 0,
            timestamp: chrono::Utc::now(),
        }).await;
    }

    StatusCode::OK
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_uuid_silent(s: &str) -> uuid::Uuid {
    uuid::Uuid::parse_str(s).unwrap_or_else(|_| uuid::Uuid::nil())
}

/// Extract fenced code blocks from markdown output.
fn extract_code_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current = Vec::new();

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_block {
                blocks.push(current.join("\n"));
                current.clear();
                in_block = false;
            } else {
                in_block = true;
            }
        } else if in_block {
            current.push(line.to_string());
        }
    }

    blocks
}

// ---------------------------------------------------------------------------
// Legacy CRUD handlers (unchanged)
// ---------------------------------------------------------------------------

async fn list_hooks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Hook>>, axum::response::Response> {
    let hooks = state.uow.hook_repo.list().map_err(api_error)?;
    Ok(Json(hooks))
}

async fn get_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Hook>, axum::response::Response> {
    let hook = state
        .uow.hook_repo
        .get(&id)
        .map_err(api_error)?
        .ok_or_else(|| {
            api_error(forge_core::error::ForgeError::Internal(format!(
                "hook not found: {}",
                id
            )))
        })?;
    Ok(Json(hook))
}

async fn create_hook(
    State(state): State<AppState>,
    Json(input): Json<NewHook>,
) -> Result<Json<Hook>, axum::response::Response> {
    let hook = state.uow.hook_repo.create(&input).map_err(api_error)?;
    Ok(Json(hook))
}

async fn update_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateHook>,
) -> Result<Json<Hook>, axum::response::Response> {
    let hook = state.uow.hook_repo.update(&id, &input).map_err(api_error)?;
    Ok(Json(hook))
}

async fn delete_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    state.uow.hook_repo.delete(&id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_tool_payload_deserializes() {
        let json = r#"{"session_id":"abc-123","tool_name":"Read"}"#;
        let payload: PreToolPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.tool_name, "Read");
        assert_eq!(payload.session_id, "abc-123");
    }

    #[test]
    fn post_tool_payload_deserializes() {
        let json = r#"{"session_id":"abc-123","tool_name":"Write","tool_output":"hello"}"#;
        let payload: PostToolPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.tool_name, "Write");
        assert_eq!(payload.tool_output, Some("hello".to_string()));
    }

    #[test]
    fn post_tool_payload_optional_output() {
        let json = r#"{"session_id":"abc-123","tool_name":"Bash"}"#;
        let payload: PostToolPayload = serde_json::from_str(json).unwrap();
        assert!(payload.tool_output.is_none());
    }

    #[test]
    fn post_tool_with_code_block_extracts() {
        let output = "Here is the fix:\n```rust\nfn main() {}\n```\nDone.";
        let blocks = extract_code_blocks(output);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("fn main"));
    }

    #[test]
    fn extract_code_blocks_multiple() {
        let output = "text\n```py\neval(x)\n```\nmore\n```js\nalert(1)\n```";
        let blocks = extract_code_blocks(output);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].contains("eval"));
        assert!(blocks[1].contains("alert"));
    }

    #[test]
    fn extract_code_blocks_empty_input() {
        let blocks = extract_code_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn stop_payload_deserializes() {
        let json = r#"{"session_id":"abc-123"}"#;
        let payload: StopPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.session_id, "abc-123");
    }

    #[test]
    fn parse_uuid_silent_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid_silent(uuid_str);
        assert_eq!(result.to_string(), uuid_str);
    }

    #[test]
    fn parse_uuid_silent_invalid_returns_nil() {
        let result = parse_uuid_silent("not-a-uuid");
        assert!(result.is_nil());
    }

    #[test]
    fn pre_tool_response_serializes() {
        let resp = PreToolResponse {
            allowed: true,
            reason: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"allowed\":true"));
        assert!(!json.contains("reason")); // skip_serializing_if
    }

    #[test]
    fn pre_tool_response_with_reason() {
        let resp = PreToolResponse {
            allowed: false,
            reason: Some("budget exceeded".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"allowed\":false"));
        assert!(json.contains("budget exceeded"));
    }
}
