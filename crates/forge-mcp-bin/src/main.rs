//! Forge MCP server: stdio JSON-RPC loop, agent and session tools.
//! Usage: FORGE_DB_PATH=~/.claude-forge/forge.db forge-mcp (or from IDE MCP config).

use forge_core::ids::{AgentId, SessionId};
use forge_db::{AgentRepo, EventRepo, Migrator, NewSession, SessionRepo, StoredEvent};
use forge_mcp::{McpError, McpRequest, McpResponse};
use forge_agent::model::{NewAgent, UpdateAgent};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;

fn default_db_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/.claude-forge/forge.db", home)
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_path = env::var("FORGE_DB_PATH").unwrap_or_else(|_| default_db_path());
    let path = Path::new(&db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db = forge_db::DbPool::new(path)?;
    {
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending()?;
    }
    let conn = db.conn_arc();
    let agent_repo = Arc::new(AgentRepo::new(Arc::clone(&conn)));
    let session_repo = Arc::new(SessionRepo::new(Arc::clone(&conn)));
    let event_repo = Arc::new(EventRepo::new(Arc::clone(&conn)));

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<McpRequest>(line) {
            Ok(req) => dispatch(&req, &agent_repo, &session_repo, &event_repo),
            Err(e) => McpResponse {
                jsonrpc: "2.0".into(),
                id: None,
                result: None,
                error: Some(McpError {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                }),
            },
        };
        if let Err(e) = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap_or_default()) {
            eprintln!("write error: {}", e);
            break;
        }
        stdout.flush()?;
    }
    Ok(())
}

fn dispatch(
    req: &McpRequest,
    agent_repo: &AgentRepo,
    session_repo: &SessionRepo,
    event_repo: &EventRepo,
) -> McpResponse {
    let id = req.id.clone();
    let method = req.method.as_str();
    let params = req.params.clone().unwrap_or(serde_json::Value::Null);

    let result = match method {
        "agent_list" => agent_list(agent_repo),
        "agent_get" => agent_get(agent_repo, &params),
        "agent_create" => agent_create(agent_repo, &params),
        "agent_update" => agent_update(agent_repo, &params),
        "agent_delete" => agent_delete(agent_repo, &params),
        "session_list" => session_list(session_repo),
        "session_get" => session_get(session_repo, &params),
        "session_create" => session_create(agent_repo, session_repo, &params),
        "session_delete" => session_delete(session_repo, &params),
        "session_export" => session_export(session_repo, event_repo, &params),
        _ => Err(forge_core::ForgeError::Validation(format!(
            "unknown method: {}",
            method
        ))),
    };

    match result {
        Ok(value) => McpResponse {
            jsonrpc: "2.0".into(),
            id: id.clone(),
            result: Some(value),
            error: None,
        },
        Err(e) => {
            let (code, message) = forge_error_to_rpc(&e);
            McpResponse {
                jsonrpc: "2.0".into(),
                id: id.clone(),
                result: None,
                error: Some(McpError { code, message }),
            }
        }
    }
}

fn forge_error_to_rpc(e: &forge_core::ForgeError) -> (i32, String) {
    use forge_core::ForgeError;
    let message = e.to_string();
    let code = match e {
        ForgeError::AgentNotFound(_) | ForgeError::SessionNotFound(_) => -32001,
        ForgeError::Validation(_) => -32602,
        ForgeError::Database(_) | ForgeError::Internal(_) => -32000,
        _ => -32603,
    };
    (code, message)
}

fn agent_list(repo: &AgentRepo) -> Result<serde_json::Value, forge_core::ForgeError> {
    let agents = repo.list()?;
    Ok(serde_json::to_value(agents).map_err(forge_core::ForgeError::Serialization)?)
}

fn agent_get(repo: &AgentRepo, params: &serde_json::Value) -> Result<serde_json::Value, forge_core::ForgeError> {
    let id = params
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| forge_core::ForgeError::Validation("missing id".into()))?;
    let agent_id = AgentId(uuid::Uuid::parse_str(id).map_err(|_| forge_core::ForgeError::Validation("invalid id".into()))?);
    let agent = repo.get(&agent_id)?;
    Ok(serde_json::to_value(agent).map_err(forge_core::ForgeError::Serialization)?)
}

fn agent_create(repo: &AgentRepo, params: &serde_json::Value) -> Result<serde_json::Value, forge_core::ForgeError> {
    let input: NewAgent = serde_json::from_value(params.clone()).map_err(forge_core::ForgeError::Serialization)?;
    let agent = repo.create(&input)?;
    Ok(serde_json::to_value(agent).map_err(forge_core::ForgeError::Serialization)?)
}

fn agent_update(repo: &AgentRepo, params: &serde_json::Value) -> Result<serde_json::Value, forge_core::ForgeError> {
    let id = params
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| forge_core::ForgeError::Validation("missing id".into()))?;
    let agent_id = AgentId(uuid::Uuid::parse_str(id).map_err(|_| forge_core::ForgeError::Validation("invalid id".into()))?);
    let input: UpdateAgent = serde_json::from_value(params.clone()).map_err(forge_core::ForgeError::Serialization)?;
    let agent = repo.update(&agent_id, &input)?;
    Ok(serde_json::to_value(agent).map_err(forge_core::ForgeError::Serialization)?)
}

fn agent_delete(repo: &AgentRepo, params: &serde_json::Value) -> Result<serde_json::Value, forge_core::ForgeError> {
    let id = params
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| forge_core::ForgeError::Validation("missing id".into()))?;
    let agent_id = AgentId(uuid::Uuid::parse_str(id).map_err(|_| forge_core::ForgeError::Validation("invalid id".into()))?);
    repo.delete(&agent_id)?;
    Ok(serde_json::json!({ "ok": true }))
}

fn session_list(repo: &SessionRepo) -> Result<serde_json::Value, forge_core::ForgeError> {
    let sessions = repo.list()?;
    Ok(serde_json::to_value(sessions).map_err(forge_core::ForgeError::Serialization)?)
}

fn session_get(repo: &SessionRepo, params: &serde_json::Value) -> Result<serde_json::Value, forge_core::ForgeError> {
    let id = params
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| forge_core::ForgeError::Validation("missing id".into()))?;
    let session_id = SessionId(uuid::Uuid::parse_str(id).map_err(|_| forge_core::ForgeError::Validation("invalid id".into()))?);
    let session = repo.get(&session_id)?;
    Ok(serde_json::to_value(session).map_err(forge_core::ForgeError::Serialization)?)
}

fn session_create(
    agent_repo: &AgentRepo,
    session_repo: &SessionRepo,
    params: &serde_json::Value,
) -> Result<serde_json::Value, forge_core::ForgeError> {
    let agent_id_str = params
        .get("agent_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| forge_core::ForgeError::Validation("missing agent_id".into()))?;
    let agent_id = AgentId(uuid::Uuid::parse_str(agent_id_str).map_err(|_| forge_core::ForgeError::Validation("invalid agent_id".into()))?);
    agent_repo.get(&agent_id)?;
    let directory = params
        .get("directory")
        .and_then(|v| v.as_str())
        .unwrap_or(".")
        .to_string();
    let claude_session_id = params.get("claude_session_id").and_then(|v| v.as_str()).map(String::from);
    let input = NewSession {
        agent_id,
        directory,
        claude_session_id,
    };
    let session = session_repo.create(&input)?;
    Ok(serde_json::to_value(session).map_err(forge_core::ForgeError::Serialization)?)
}

fn session_delete(repo: &SessionRepo, params: &serde_json::Value) -> Result<serde_json::Value, forge_core::ForgeError> {
    let id = params
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| forge_core::ForgeError::Validation("missing id".into()))?;
    let session_id = SessionId(uuid::Uuid::parse_str(id).map_err(|_| forge_core::ForgeError::Validation("invalid id".into()))?);
    repo.delete(&session_id)?;
    Ok(serde_json::json!({ "ok": true }))
}

fn session_export(
    session_repo: &SessionRepo,
    event_repo: &EventRepo,
    params: &serde_json::Value,
) -> Result<serde_json::Value, forge_core::ForgeError> {
    let id = params
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| forge_core::ForgeError::Validation("missing id".into()))?;
    let session_id = SessionId(uuid::Uuid::parse_str(id).map_err(|_| forge_core::ForgeError::Validation("invalid id".into()))?);
    let session = session_repo.get(&session_id)?;
    let events = event_repo.query_by_session(&session_id)?;
    let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("json");
    if format == "markdown" {
        let md = session_to_markdown(&session, &events);
        Ok(serde_json::Value::String(md))
    } else {
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
            session: forge_db::Session,
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
        Ok(serde_json::to_value(ExportJson {
            session,
            events: events_export,
        })
        .map_err(forge_core::ForgeError::Serialization)?)
    }
}

fn session_to_markdown(session: &forge_db::Session, events: &[StoredEvent]) -> String {
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
