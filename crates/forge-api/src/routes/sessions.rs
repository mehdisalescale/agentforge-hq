//! Session CRUD and export: GET/POST /api/v1/sessions, GET/DELETE /api/v1/sessions/:id, GET /api/v1/sessions/:id/export

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use forge_core::ids::SessionId;
use forge_db::{NewSession, Session};
use serde::Deserialize;

use crate::error::{api_error, parse_uuid};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateSessionBody {
    pub agent_id: String,
    pub directory: String,
    #[serde(default)]
    pub claude_session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "json".to_string()
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/:id", get(get_session).delete(delete_session))
        .route("/sessions/:id/export", get(export_session))
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Session>>, axum::response::Response> {
    let sessions = state.session_repo.list().map_err(api_error)?;
    Ok(Json(sessions))
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> Result<Json<Session>, axum::response::Response> {
    let agent_id = forge_core::ids::AgentId(parse_uuid(&body.agent_id)?);
    // Verify agent exists
    state.agent_repo.get(&agent_id).map_err(api_error)?;

    let input = NewSession {
        agent_id,
        directory: body.directory,
        claude_session_id: body.claude_session_id,
    };
    let session = state.session_repo.create(&input).map_err(api_error)?;
    Ok(Json(session))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    let session = state.session_repo.get(&session_id).map_err(api_error)?;
    Ok(Json(session))
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    state.session_repo.delete(&session_id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn export_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    let session = state.session_repo.get(&session_id).map_err(api_error)?;
    let events = state.event_repo.query_by_session(&session_id).map_err(api_error)?;

    let format = query.format.to_lowercase();
    if format == "markdown" {
        let md = session_to_markdown(&session, &events);
        return Ok(
            ([("content-type", "text/markdown; charset=utf-8")], md).into_response(),
        );
    }
    // default: json
    #[derive(serde::Serialize)]
    struct ExportEvent {
        id: String,
        session_id: Option<String>,
        agent_id: Option<String>,
        event_type: String,
        data_json: String,
        timestamp: String,
    }
    #[derive(serde::Serialize)]
    struct ExportJson {
        session: Session,
        events: Vec<ExportEvent>,
    }
    let events_export: Vec<ExportEvent> = events
        .iter()
        .map(|e| ExportEvent {
            id: e.id.clone(),
            session_id: e.session_id.clone(),
            agent_id: e.agent_id.clone(),
            event_type: e.event_type.clone(),
            data_json: e.data_json.clone(),
            timestamp: e.timestamp.clone(),
        })
        .collect();
    let body = ExportJson {
        session,
        events: events_export,
    };
    Ok(Json(body).into_response())
}

fn session_to_markdown(session: &Session, events: &[forge_db::StoredEvent]) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Session {}\n\n", session.id));
    md.push_str(&format!("- **Agent ID:** {}\n", session.agent_id));
    md.push_str(&format!("- **Directory:** {}\n", session.directory));
    md.push_str(&format!("- **Status:** {}\n", session.status));
    md.push_str(&format!(
        "- **Created:** {}\n",
        session.created_at.to_rfc3339()
    ));
    if let Some(ref c) = session.claude_session_id {
        md.push_str(&format!("- **Claude session:** {}\n", c));
    }
    md.push_str("\n## Events\n\n");
    for ev in events {
        md.push_str(&format!("### {} ({})\n\n", ev.event_type, ev.timestamp));
        md.push_str("```json\n");
        md.push_str(&ev.data_json);
        md.push_str("\n```\n\n");
    }
    md
}
