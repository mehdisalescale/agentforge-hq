//! Agent CRUD: GET/POST /api/v1/agents, GET/PUT/DELETE /api/v1/agents/:id

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use forge_agent::model::{Agent, NewAgent, UpdateAgent};
use forge_core::events::ForgeEvent;
use forge_core::ids::AgentId;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::{api_error, parse_uuid};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/stats", get(all_agent_stats))
        .route("/agents/:id", get(get_agent).put(update_agent).delete(delete_agent))
        .route("/agents/:id/stats", get(agent_stats))
}

async fn list_agents(State(state): State<AppState>) -> Result<Json<Vec<Agent>>, axum::response::Response> {
    let agents = state.agent_repo.list().map_err(api_error)?;
    Ok(Json(agents))
}

async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Agent>, axum::response::Response> {
    let agent_id = AgentId(parse_uuid(&id)?);
    let agent = state.agent_repo.get(&agent_id).map_err(api_error)?;
    Ok(Json(agent))
}

async fn create_agent(
    State(state): State<AppState>,
    Json(input): Json<NewAgent>,
) -> Result<Json<Agent>, axum::response::Response> {
    let agent = state.agent_repo.create(&input).map_err(api_error)?;
    if let Err(e) = state.event_bus.emit(ForgeEvent::AgentCreated {
        agent_id: agent.id.clone(),
        name: agent.name.clone(),
        timestamp: Utc::now(),
    }) {
        tracing::warn!(error = %e, "failed to emit AgentCreated event");
    }
    Ok(Json(agent))
}

async fn update_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateAgent>,
) -> Result<Json<Agent>, axum::response::Response> {
    let agent_id = AgentId(parse_uuid(&id)?);
    let agent = state.agent_repo.update(&agent_id, &input).map_err(api_error)?;
    if let Err(e) = state.event_bus.emit(ForgeEvent::AgentUpdated {
        agent_id: agent.id.clone(),
        name: agent.name.clone(),
        timestamp: Utc::now(),
    }) {
        tracing::warn!(error = %e, "failed to emit AgentUpdated event");
    }
    Ok(Json(agent))
}

async fn delete_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    let agent_id = AgentId(parse_uuid(&id)?);
    state.agent_repo.delete(&agent_id).map_err(api_error)?;
    if let Err(e) = state.event_bus.emit(ForgeEvent::AgentDeleted {
        agent_id,
        timestamp: Utc::now(),
    }) {
        tracing::warn!(error = %e, "failed to emit AgentDeleted event");
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// --- Agent stats ---

#[derive(Debug, Serialize, Clone)]
pub struct AgentStats {
    pub run_count: i64,
    pub last_run: Option<String>,
    pub total_cost: f64,
    pub success_rate: f64,
}

async fn agent_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentStats>, axum::response::Response> {
    let agent_id = AgentId(parse_uuid(&id)?);
    // Verify agent exists
    state.agent_repo.get(&agent_id).map_err(api_error)?;

    let sessions = state.session_repo.list().map_err(api_error)?;
    let agent_sessions: Vec<_> = sessions
        .iter()
        .filter(|s| s.agent_id == agent_id)
        .collect();

    let run_count = agent_sessions.len() as i64;
    let last_run = agent_sessions
        .iter()
        .map(|s| s.created_at.to_rfc3339())
        .max();
    let total_cost: f64 = agent_sessions.iter().map(|s| s.cost_usd).sum();
    let completed = agent_sessions
        .iter()
        .filter(|s| s.status == "completed")
        .count();
    let success_rate = if run_count > 0 {
        completed as f64 / run_count as f64 * 100.0
    } else {
        0.0
    };

    Ok(Json(AgentStats {
        run_count,
        last_run,
        total_cost,
        success_rate,
    }))
}

async fn all_agent_stats(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, AgentStats>>, axum::response::Response> {
    let sessions = state.session_repo.list().map_err(api_error)?;
    let mut stats_map: HashMap<String, (i64, Option<String>, f64, i64)> = HashMap::new();

    for s in &sessions {
        let key = s.agent_id.0.to_string();
        let entry = stats_map.entry(key).or_insert((0, None, 0.0, 0));
        entry.0 += 1;
        let ts = s.created_at.to_rfc3339();
        if entry.1.as_ref().map_or(true, |prev| ts > *prev) {
            entry.1 = Some(ts);
        }
        entry.2 += s.cost_usd;
        if s.status == "completed" {
            entry.3 += 1;
        }
    }

    let result: HashMap<String, AgentStats> = stats_map
        .into_iter()
        .map(|(id, (run_count, last_run, total_cost, completed))| {
            let success_rate = if run_count > 0 {
                completed as f64 / run_count as f64 * 100.0
            } else {
                0.0
            };
            (
                id,
                AgentStats {
                    run_count,
                    last_run,
                    total_cost,
                    success_rate,
                },
            )
        })
        .collect();

    Ok(Json(result))
}
