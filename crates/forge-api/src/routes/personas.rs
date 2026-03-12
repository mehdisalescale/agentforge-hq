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
use forge_db::{AgentRepo, OrgPositionRepo, PersonaRepo};
use forge_persona::model::{Persona, PersonaId};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/personas", get(list_personas))
        .route("/personas/:id", get(get_persona).post(hire_persona))
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

#[derive(Debug, Deserialize)]
struct HirePersonaBody {
    company_id: String,
    #[serde(default)]
    department_id: Option<String>,
    #[serde(default)]
    reports_to: Option<String>,
    #[serde(default)]
    title_override: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct HirePersonaResponse {
    agent_id: String,
    position_id: String,
}

async fn hire_persona(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<HirePersonaBody>,
) -> Result<Json<HirePersonaResponse>, axum::response::Response> {
    // Look up the source persona to seed the new agent and position.
    let persona_id = PersonaId(
        id.parse().map_err(|_| {
            api_error(forge_core::error::ForgeError::Validation(format!(
                "invalid persona id: {id}"
            )))
        })?,
    );
    let persona = state.persona_repo.get(&persona_id).map_err(api_error)?;

    // Create an agent that reflects this persona.
    let new_agent = forge_agent::model::NewAgent {
        name: persona.name.clone(),
        model: None,
        system_prompt: Some(format!(
            "You are persona '{}'. Short summary: {}.\nUse this as your operating persona.",
            persona.name, persona.short_description
        )),
        allowed_tools: None,
        max_turns: None,
        use_max: None,
        preset: None,
        config: None,
    };
    let agent = state.agent_repo.create(&new_agent).map_err(api_error)?;

    // Backfill persona_id on the agent row for traceability.
    {
        let conn = state
            .agent_repo
            .conn
            .lock()
            .expect("agent repo db mutex poisoned");
        conn.execute(
            "UPDATE agents SET persona_id = ?1 WHERE id = ?2",
            rusqlite::params![persona_id.0.to_string(), agent.id.0.to_string()],
        )
        .map_err(|e| api_error(forge_core::error::ForgeError::Database(Box::new(e))))?;
    }

    // Create an org position in the chosen company hierarchy.
    let pos_input = forge_db::NewOrgPosition {
        company_id: body.company_id,
        department_id: body.department_id,
        agent_id: Some(agent.id.0.to_string()),
        reports_to: body.reports_to,
        role: persona.slug.clone(),
        title: body
            .title_override
            .or_else(|| Some(persona.name.clone())),
    };
    let position = state
        .org_position_repo
        .create(&pos_input)
        .map_err(api_error)?;

    Ok(Json(HirePersonaResponse {
        agent_id: agent.id.0.to_string(),
        position_id: position.id,
    }))
}


