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
        .route("/sessions/:id/events", get(get_session_events))
        .route("/sessions/:id/export", get(export_session))
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Session>>, axum::response::Response> {
    let sessions = state.uow.session_repo.list().map_err(api_error)?;
    Ok(Json(sessions))
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> Result<Json<Session>, axum::response::Response> {
    let agent_id = forge_core::ids::AgentId(parse_uuid(&body.agent_id)?);
    // Verify agent exists
    state.uow.agent_repo.get(&agent_id).map_err(api_error)?;

    let input = NewSession {
        agent_id,
        directory: body.directory,
        claude_session_id: body.claude_session_id,
    };
    let session = state.uow.session_repo.create(&input).map_err(api_error)?;
    Ok(Json(session))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    let session = state.uow.session_repo.get(&session_id).map_err(api_error)?;
    Ok(Json(session))
}

/// GET /api/v1/sessions/:id/events — fetch stored events for a session
async fn get_session_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<forge_db::StoredEvent>>, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    // Verify session exists
    state.uow.session_repo.get(&session_id).map_err(api_error)?;
    let events = state.uow.event_repo.query_by_session(&session_id).map_err(api_error)?;
    Ok(Json(events))
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    state.uow.session_repo.delete(&session_id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn export_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    let session = state.uow.session_repo.get(&session_id).map_err(api_error)?;
    let events = state.uow.event_repo.query_by_session(&session_id).map_err(api_error)?;

    let format = query.format.to_lowercase();
    if format == "html" {
        let html = session_to_html(&session, &events);
        return Ok(
            ([("content-type", "text/html; charset=utf-8")], html).into_response(),
        );
    }
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn session_to_html(session: &Session, events: &[forge_db::StoredEvent]) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str(&format!("<title>Session {}</title>\n", html_escape(&session.id.to_string())));
    html.push_str("<style>\n");
    html.push_str("body{font-family:system-ui,-apple-system,sans-serif;background:#1a1a2e;color:#e0e0e0;margin:0;padding:2rem;max-width:900px;margin:0 auto}\n");
    html.push_str("h1{color:#7c3aed}h2{color:#a78bfa;border-bottom:1px solid #333;padding-bottom:.5rem}\n");
    html.push_str(".meta{background:#16213e;padding:1rem;border-radius:8px;margin-bottom:2rem}\n");
    html.push_str(".meta dt{font-weight:bold;color:#a78bfa}.meta dd{margin:0 0 .5rem 0}\n");
    html.push_str(".event{background:#0f3460;padding:1rem;border-radius:8px;margin-bottom:1rem;border-left:4px solid #7c3aed}\n");
    html.push_str(".event-type{display:inline-block;background:#7c3aed;color:white;padding:2px 8px;border-radius:4px;font-size:.85rem;font-weight:bold}\n");
    html.push_str(".timestamp{color:#888;font-size:.85rem;margin-left:.5rem}\n");
    html.push_str("pre{background:#1a1a2e;padding:1rem;border-radius:4px;overflow-x:auto;font-size:.85rem}\n");
    html.push_str("code{color:#e2e8f0}\n");
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str(&format!("<h1>Session {}</h1>\n", html_escape(&session.id.to_string())));
    html.push_str("<div class=\"meta\"><dl>\n");
    html.push_str(&format!("<dt>Agent ID</dt><dd>{}</dd>\n", html_escape(&session.agent_id.to_string())));
    html.push_str(&format!("<dt>Directory</dt><dd>{}</dd>\n", html_escape(&session.directory)));
    html.push_str(&format!("<dt>Status</dt><dd>{}</dd>\n", html_escape(&session.status)));
    html.push_str(&format!("<dt>Created</dt><dd>{}</dd>\n", session.created_at.to_rfc3339()));
    if let Some(ref c) = session.claude_session_id {
        html.push_str(&format!("<dt>Claude Session</dt><dd>{}</dd>\n", html_escape(c)));
    }
    html.push_str("</dl></div>\n");
    html.push_str("<h2>Events</h2>\n");
    for ev in events {
        html.push_str("<div class=\"event\">\n");
        html.push_str(&format!("<span class=\"event-type\">{}</span>", html_escape(&ev.event_type)));
        html.push_str(&format!("<span class=\"timestamp\">{}</span>\n", html_escape(&ev.timestamp)));
        html.push_str(&format!("<pre><code>{}</code></pre>\n", html_escape(&ev.data_json)));
        html.push_str("</div>\n");
    }
    html.push_str("</body>\n</html>");
    html
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
