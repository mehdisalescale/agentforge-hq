//! Workflows API: CRUD + run.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use forge_db::Workflow;
use forge_process::{Pipeline, PipelineRunner};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/workflows", get(list_workflows).post(create_workflow))
        .route(
            "/workflows/:id",
            get(get_workflow).put(update_workflow).delete(delete_workflow),
        )
        .route("/workflows/:id/run", post(run_workflow))
}

async fn list_workflows(
    State(state): State<AppState>,
) -> Result<Json<Vec<Workflow>>, axum::response::Response> {
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

#[derive(Debug, Deserialize)]
struct CreateWorkflowRequest {
    name: String,
    description: Option<String>,
    definition_json: String,
}

async fn create_workflow(
    State(state): State<AppState>,
    Json(body): Json<CreateWorkflowRequest>,
) -> Result<Json<Workflow>, axum::response::Response> {
    let workflow = state
        .workflow_repo
        .create(&body.name, body.description.as_deref(), &body.definition_json)
        .map_err(api_error)?;
    Ok(Json(workflow))
}

#[derive(Debug, Deserialize)]
struct UpdateWorkflowRequest {
    name: Option<String>,
    description: Option<String>,
    definition_json: Option<String>,
}

async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateWorkflowRequest>,
) -> Result<Json<Workflow>, axum::response::Response> {
    let workflow = state
        .workflow_repo
        .update(
            &id,
            body.name.as_deref(),
            body.description.as_deref(),
            body.definition_json.as_deref(),
        )
        .map_err(api_error)?;
    Ok(Json(workflow))
}

async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    state.workflow_repo.delete(&id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct RunWorkflowRequest {
    /// Initial input text for the pipeline.
    input: String,
    /// Working directory for agent execution.
    #[serde(default = "default_working_dir")]
    working_dir: String,
}

fn default_working_dir() -> String {
    ".".to_string()
}

#[derive(Debug, Serialize)]
struct RunWorkflowResponse {
    workflow_id: String,
    steps_completed: usize,
    success: bool,
}

async fn run_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<RunWorkflowRequest>,
) -> Result<Json<RunWorkflowResponse>, axum::response::Response> {
    let workflow = state.workflow_repo.get(&id).map_err(api_error)?;

    let pipeline: Pipeline = serde_json::from_str(&workflow.definition_json).map_err(|e| {
        api_error(forge_core::error::ForgeError::Validation(format!(
            "invalid pipeline definition: {}",
            e
        )))
    })?;

    let session_id = forge_core::ids::SessionId::new();

    // Emit PipelineStarted event
    let _ = state.event_bus.emit(forge_core::events::ForgeEvent::PipelineStarted {
        session_id: session_id.0.to_string(),
        workflow_id: id.clone(),
        step_count: pipeline.steps.len(),
        timestamp: Utc::now(),
    });

    let runner = PipelineRunner::new(Arc::clone(&state.event_bus), 4);
    let step_results = runner
        .run(&session_id, &pipeline, &body.input, &body.working_dir)
        .await;

    let steps_completed = step_results.len();
    let success = step_results.iter().all(|s| s.success);

    // Emit per-step completed events
    for sr in &step_results {
        let _ = state.event_bus.emit(forge_core::events::ForgeEvent::PipelineStepCompleted {
            session_id: session_id.0.to_string(),
            step_index: sr.step_index,
            success: sr.success,
            timestamp: Utc::now(),
        });
    }

    // Emit PipelineCompleted event
    let _ = state.event_bus.emit(forge_core::events::ForgeEvent::PipelineCompleted {
        session_id: session_id.0.to_string(),
        workflow_id: id.clone(),
        success,
        timestamp: Utc::now(),
    });

    Ok(Json(RunWorkflowResponse {
        workflow_id: id,
        steps_completed,
        success,
    }))
}
