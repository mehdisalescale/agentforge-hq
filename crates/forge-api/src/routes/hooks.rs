//! Hook CRUD: GET/POST /api/v1/hooks, GET/PUT/DELETE /api/v1/hooks/:id

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use crate::error::api_error;
use crate::state::AppState;

// Re-use types from forge-db.
use forge_db::repos::hooks::{Hook, NewHook, UpdateHook};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/hooks", get(list_hooks).post(create_hook))
        .route(
            "/hooks/:id",
            get(get_hook).put(update_hook).delete(delete_hook),
        )
}

async fn list_hooks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Hook>>, axum::response::Response> {
    let hooks = state.hook_repo.list().map_err(api_error)?;
    Ok(Json(hooks))
}

async fn get_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Hook>, axum::response::Response> {
    let hook = state
        .hook_repo
        .get(&id)
        .map_err(api_error)?
        .ok_or_else(|| {
            api_error(forge_core::error::ForgeError::Internal(format!(
                "hook not found: {}",
                id
            )))
        })?;
    Ok(Json(hook))
}

async fn create_hook(
    State(state): State<AppState>,
    Json(input): Json<NewHook>,
) -> Result<Json<Hook>, axum::response::Response> {
    let hook = state.hook_repo.create(&input).map_err(api_error)?;
    Ok(Json(hook))
}

async fn update_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateHook>,
) -> Result<Json<Hook>, axum::response::Response> {
    let hook = state.hook_repo.update(&id, &input).map_err(api_error)?;
    Ok(Json(hook))
}

async fn delete_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    state.hook_repo.delete(&id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
