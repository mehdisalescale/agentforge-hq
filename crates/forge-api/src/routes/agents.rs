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

use crate::error::{api_error, parse_uuid};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).put(update_agent).delete(delete_agent))
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
