//! Run endpoint: POST /api/v1/run — start a process for an agent + prompt; optional session_id for resume.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use forge_core::error::ForgeError;
use forge_core::events::ForgeEvent;
use forge_core::ids::{AgentId, SessionId};
use forge_db::NewSession;
use forge_process::{parse_line, ProcessRunner, spawn, SpawnConfig, SpawnError};
use serde::Deserialize;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;

use crate::error::{api_error, parse_uuid};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub agent_id: String,
    pub prompt: String,
    pub session_id: Option<String>,
    /// Optional working directory for the run (used when creating a new session and as spawn cwd).
    pub directory: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct RunResponse {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/run", post(run_handler))
}

async fn run_handler(
    State(state): State<AppState>,
    Json(body): Json<RunRequest>,
) -> Result<impl IntoResponse, axum::response::Response> {
    let agent_id = AgentId(parse_uuid(&body.agent_id)?);
    state.agent_repo.get(&agent_id).map_err(api_error)?;

    let session = if let Some(ref sid) = body.session_id {
        let id = SessionId(parse_uuid(sid)?);
        state.session_repo.get(&id).map_err(api_error)?
    } else {
        let directory = body
            .directory
            .as_deref()
            .unwrap_or(".")
            .to_string();
        let input = NewSession {
            agent_id: agent_id.clone(),
            directory: directory.clone(),
            claude_session_id: None,
        };
        state.session_repo.create(&input).map_err(api_error)?
    };
    let session_id = session.id.clone();

    state
        .circuit_breaker
        .check()
        .map_err(|_| api_error(ForgeError::Internal("circuit breaker open".into())))?;

    let resume_arg = body.session_id.as_deref();
    let config = SpawnConfig::from_env().with_working_dir(&session.directory);
    let mut handle = spawn(&config, &body.prompt, resume_arg)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "run: spawn failed");
            use forge_core::error::ForgeError;
            crate::error::api_error(match e {
                SpawnError::Io(io) => ForgeError::Io(io),
                SpawnError::CommandMissing => ForgeError::Internal("command missing".into()),
            })
        })?;

    let event_bus = Arc::clone(&state.event_bus);
    let session_repo = Arc::clone(&state.session_repo);
    let circuit_breaker = Arc::clone(&state.circuit_breaker);
    let sid = session_id.clone();
    let aid = agent_id.clone();
    tokio::spawn(async move {
        let runner = ProcessRunner::new(event_bus);
        if runner
            .emit(ForgeEvent::ProcessStarted {
                session_id: sid.clone(),
                agent_id: aid.clone(),
                timestamp: chrono::Utc::now(),
            })
            .is_err()
        {
            tracing::warn!("run task: failed to emit ProcessStarted");
        }
        if session_repo.update_status(&sid, "running").is_err() {
            tracing::warn!(session_id = %sid, "run task: failed to update session status to running");
        }
        let mut stdout = match handle.take_stdout() {
            Some(s) => s,
            None => {
                let _ = handle.wait().await;
                return;
            }
        };
        let mut reader = tokio::io::BufReader::new(&mut stdout);
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader.read_line(&mut buf).await {
                Ok(0) => break,
                Err(e) => {
                    tracing::warn!(error = %e, "run task: read_line error");
                    break;
                }
                _ => {}
            }
            if let Ok(Some(ev)) = parse_line(buf.trim()) {
                if runner.emit_parsed_event(&sid, &aid, &ev).is_err() {
                    tracing::warn!("run task: emit_parsed_event failed");
                }
            }
        }
        match handle.wait().await {
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                circuit_breaker.record_success();
                if session_repo.update_status(&sid, "completed").is_err() {
                    tracing::warn!(session_id = %sid, "run task: failed to update session status to completed");
                }
                if runner
                    .emit(ForgeEvent::ProcessCompleted {
                        session_id: sid,
                        exit_code: code,
                        timestamp: chrono::Utc::now(),
                    })
                    .is_err()
                {
                    tracing::warn!("run task: failed to emit ProcessCompleted");
                }
            }
            Err(e) => {
                circuit_breaker.record_failure();
                tracing::warn!(error = %e, "run task: wait failed");
                if session_repo.update_status(&sid, "failed").is_err() {
                    tracing::warn!(session_id = %sid, "run task: failed to update session status to failed");
                }
                if runner
                    .emit(ForgeEvent::ProcessFailed {
                        session_id: sid,
                        error: e.to_string(),
                        timestamp: chrono::Utc::now(),
                    })
                    .is_err()
                {
                    tracing::warn!("run task: failed to emit ProcessFailed");
                }
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(RunResponse {
            session_id: session_id.0.to_string(),
            message: Some("Started.".into()),
        }),
    ))
}
