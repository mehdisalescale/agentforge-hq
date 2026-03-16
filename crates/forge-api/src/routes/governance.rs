//! Governance routes: goals and approvals for a given company.
//!
//! - `GET /api/v1/goals?company_id=`: list goals for a company.
//! - `POST /api/v1/goals`: create a new goal (initial status `planned`).
//! - `PATCH /api/v1/goals/:id/status`: update a goal's status only.
//! - `GET /api/v1/approvals?company_id=&status=`: list approvals, optionally filtered by status.
//! - `POST /api/v1/approvals`: create a new approval request.
//! - `PATCH /api/v1/approvals/:id`: approve or reject an approval.

use axum::{
    extract::{Path, Query, State},
    routing::{get, patch},
    Json, Router,
};
use forge_db::{Approval, Goal, NewApproval, NewGoal};
use serde::Deserialize;

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/goals", get(list_goals).post(create_goal))
        .route("/goals/:id/status", patch(update_goal_status))
        .route("/approvals", get(list_approvals).post(create_approval))
        .route("/approvals/:id", patch(update_approval_status))
}

// --- Goals ---

#[derive(Debug, Deserialize)]
struct ListGoalsQuery {
    company_id: String,
}

async fn list_goals(
    State(state): State<AppState>,
    Query(query): Query<ListGoalsQuery>,
) -> Result<Json<Vec<Goal>>, axum::response::Response> {
    let goals = state
        .uow.goal_repo
        .list_by_company(&query.company_id)
        .map_err(api_error)?;
    Ok(Json(goals))
}

#[derive(Debug, Deserialize)]
struct NewGoalBody {
    company_id: String,
    #[serde(default)]
    parent_id: Option<String>,
    title: String,
    #[serde(default)]
    description: Option<String>,
}

async fn create_goal(
    State(state): State<AppState>,
    Json(body): Json<NewGoalBody>,
) -> Result<Json<Goal>, axum::response::Response> {
    let input = NewGoal {
        company_id: body.company_id,
        parent_id: body.parent_id,
        title: body.title,
        description: body.description,
    };
    let goal = state.uow.goal_repo.create(&input).map_err(api_error)?;
    Ok(Json(goal))
}

#[derive(Debug, Deserialize)]
struct UpdateGoalStatusBody {
    status: String,
}

async fn update_goal_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateGoalStatusBody>,
) -> Result<Json<Goal>, axum::response::Response> {
    // Minimal validation: enforce the known enum values at the API edge.
    let allowed = ["planned", "in_progress", "completed", "cancelled"];
    if !allowed.contains(&body.status.as_str()) {
        return Err(api_error(forge_core::error::ForgeError::Validation(
            "invalid goal status".into(),
        )));
    }

    let goal = state
        .uow.goal_repo
        .update_status(&id, &body.status)
        .map_err(api_error)?;

    Ok(Json(goal))
}

// --- Approvals ---

#[derive(Debug, Deserialize)]
struct ListApprovalsQuery {
    company_id: String,
    #[serde(default)]
    status: Option<String>,
}

async fn list_approvals(
    State(state): State<AppState>,
    Query(query): Query<ListApprovalsQuery>,
) -> Result<Json<Vec<Approval>>, axum::response::Response> {
    let approvals = state
        .uow.approval_repo
        .list_by_company(&query.company_id)
        .map_err(api_error)?;
    let filtered = if let Some(status) = query.status {
        approvals
            .into_iter()
            .filter(|a| a.status == status)
            .collect()
    } else {
        approvals
    };
    Ok(Json(filtered))
}

#[derive(Debug, Deserialize)]
struct NewApprovalBody {
    company_id: String,
    approval_type: String,
    requester: String,
    data_json: serde_json::Value,
}

async fn create_approval(
    State(state): State<AppState>,
    Json(body): Json<NewApprovalBody>,
) -> Result<Json<Approval>, axum::response::Response> {
    let input = NewApproval {
        company_id: body.company_id,
        approval_type: body.approval_type,
        requester: body.requester,
        data_json: serde_json::to_string(&body.data_json)
            .map_err(|e| api_error(forge_core::error::ForgeError::Validation(e.to_string())))?,
    };
    let approval = state.uow.approval_repo.create(&input).map_err(api_error)?;
    Ok(Json(approval))
}

#[derive(Debug, Deserialize)]
struct UpdateApprovalStatusBody {
    status: String,
    #[serde(default)]
    approver: Option<String>,
}

async fn update_approval_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateApprovalStatusBody>,
) -> Result<Json<Approval>, axum::response::Response> {
    let allowed = ["pending", "approved", "rejected"];
    if !allowed.contains(&body.status.as_str()) {
        return Err(api_error(forge_core::error::ForgeError::Validation(
            "invalid approval status".into(),
        )));
    }

    let approval = state
        .uow.approval_repo
        .update_status(&id, &body.status, body.approver.as_deref())
        .map_err(api_error)?;

    Ok(Json(approval))
}

