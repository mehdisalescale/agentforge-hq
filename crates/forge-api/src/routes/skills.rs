//! Skills read API: GET /api/v1/skills, GET /api/v1/skills/:id

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use forge_db::Skill;

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/skills", get(list_skills))
        .route("/skills/:id", get(get_skill))
}

async fn list_skills(State(state): State<AppState>) -> Result<Json<Vec<Skill>>, axum::response::Response> {
    let skills = state.skill_repo.list().map_err(api_error)?;
    Ok(Json(skills))
}

async fn get_skill(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Skill>, axum::response::Response> {
    let skill = state.skill_repo.get(&id).map_err(api_error)?;
    Ok(Json(skill))
}
