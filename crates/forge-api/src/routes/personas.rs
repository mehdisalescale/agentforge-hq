//! Persona catalog read API.
//!
//! This exposes a minimal, read-only surface for the persona catalog:
//! - `GET /api/v1/personas` lists personas with optional `division_slug` and `q` filters.
//! - `GET /api/v1/personas/:id` fetches a single persona by id.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use forge_db::PersonaRepo;
use forge_persona::model::{Persona, PersonaId};
use serde::Deserialize;

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/personas", get(list_personas))
        .route("/personas/:id", get(get_persona))
}

#[derive(Debug, Deserialize)]
struct ListPersonasQuery {
    /// Optional division slug to filter by (e.g. "engineering", "product").
    division_slug: Option<String>,
    /// Optional free-text search across name, short description, and tags.
    q: Option<String>,
}

async fn list_personas(
    State(state): State<AppState>,
    Query(query): Query<ListPersonasQuery>,
) -> Result<Json<Vec<Persona>>, axum::response::Response> {
    // PersonaRepo already implements flexible filtering semantics; we just thread through.
    let division = query.division_slug.as_deref();
    let search = query.q.as_deref();

    let personas = state
        .persona_repo
        .list(division, search)
        .map_err(api_error)?;

    Ok(Json(personas))
}

async fn get_persona(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Persona>, axum::response::Response> {
    // Persona ids are UUIDs; parse from string and surface validation errors clearly.
    let parsed = PersonaId(
        id.parse().map_err(|_| {
            api_error(forge_core::error::ForgeError::Validation(format!(
                "invalid persona id: {id}"
            )))
        })?,
    );

    let persona = state.persona_repo.get(&parsed).map_err(api_error)?;
    Ok(Json(persona))
}

