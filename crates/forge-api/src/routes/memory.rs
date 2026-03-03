//! Memory CRUD: GET/POST /api/v1/memory, GET/PUT/DELETE /api/v1/memory/:id

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use forge_db::repos::memory::{Memory, NewMemory, UpdateMemory};

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_memories).post(create_memory))
        .route(
            "/:id",
            get(get_memory).put(update_memory).delete(delete_memory),
        )
}

#[derive(Debug, Deserialize)]
struct ListParams {
    limit: Option<i64>,
    offset: Option<i64>,
    q: Option<String>,
}

async fn list_memories(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Memory>>, axum::response::Response> {
    let memories = if let Some(query) = &params.q {
        state.memory_repo.search(query).map_err(api_error)?
    } else {
        let limit = params.limit.unwrap_or(50);
        let offset = params.offset.unwrap_or(0);
        state.memory_repo.list(limit, offset).map_err(api_error)?
    };
    Ok(Json(memories))
}

async fn get_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Memory>, axum::response::Response> {
    let memory = state.memory_repo.get(&id).map_err(api_error)?;
    Ok(Json(memory))
}

async fn create_memory(
    State(state): State<AppState>,
    Json(input): Json<NewMemory>,
) -> Result<Json<Memory>, axum::response::Response> {
    let memory = state.memory_repo.create(&input).map_err(api_error)?;
    Ok(Json(memory))
}

async fn update_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateMemory>,
) -> Result<Json<Memory>, axum::response::Response> {
    let memory = state.memory_repo.update(&id, &input).map_err(api_error)?;
    Ok(Json(memory))
}

async fn delete_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    state.memory_repo.delete(&id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
