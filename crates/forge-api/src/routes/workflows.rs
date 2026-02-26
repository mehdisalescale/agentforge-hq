//! Workflows read API: GET /api/v1/workflows, GET /api/v1/workflows/:id

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use forge_db::Workflow;

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/workflows", get(list_workflows))
        .route("/workflows/:id", get(get_workflow))
}

async fn list_workflows(State(state): State<AppState>) -> Result<Json<Vec<Workflow>>, axum::response::Response> {
    let workflows = state.workflow_repo.list().map_err(api_error)?;
    Ok(Json(workflows))
}

async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Workflow>, axum::response::Response> {
    let workflow = state.workflow_repo.get(&id).map_err(api_error)?;
    Ok(Json(workflow))
}
