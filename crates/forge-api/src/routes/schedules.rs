//! Schedule CRUD: GET/POST /api/v1/schedules, GET/PUT/DELETE /api/v1/schedules/:id

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use crate::error::api_error;
use crate::state::AppState;

use forge_db::repos::schedules::{NewSchedule, Schedule, UpdateSchedule};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/schedules", get(list_schedules).post(create_schedule))
        .route(
            "/schedules/:id",
            get(get_schedule).put(update_schedule).delete(delete_schedule),
        )
}

async fn list_schedules(
    State(state): State<AppState>,
) -> Result<Json<Vec<Schedule>>, axum::response::Response> {
    let schedules = state.uow.schedule_repo.list().map_err(api_error)?;
    Ok(Json(schedules))
}

async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Schedule>, axum::response::Response> {
    let schedule = state
        .uow.schedule_repo
        .get(&id)
        .map_err(api_error)?
        .ok_or_else(|| {
            api_error(forge_core::error::ForgeError::Internal(format!(
                "schedule not found: {}",
                id
            )))
        })?;
    Ok(Json(schedule))
}

async fn create_schedule(
    State(state): State<AppState>,
    Json(input): Json<NewSchedule>,
) -> Result<Json<Schedule>, axum::response::Response> {
    let schedule = state.uow.schedule_repo.create(&input).map_err(api_error)?;
    Ok(Json(schedule))
}

async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateSchedule>,
) -> Result<Json<Schedule>, axum::response::Response> {
    let schedule = state.uow.schedule_repo.update(&id, &input).map_err(api_error)?;
    Ok(Json(schedule))
}

async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    state.uow.schedule_repo.delete(&id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
