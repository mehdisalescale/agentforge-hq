//! API route handlers.

mod agents;
mod analytics;
mod backends;
mod health;
mod hooks;
mod memory;
mod org;
mod personas;
mod governance;
mod run;
mod schedules;
mod sessions;
mod settings;
mod skills;
mod workflows;
mod ws;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .merge(agents::routes())
        .merge(run::routes())
        .merge(sessions::routes())
        .merge(skills::routes())
        .merge(workflows::routes())
        .merge(ws::routes())
        .nest("/memory", memory::routes())
        .merge(hooks::routes())
        .merge(schedules::routes())
        .merge(analytics::routes())
        .merge(settings::routes())
        .merge(org::routes())
        .merge(personas::routes())
        .merge(governance::routes())
        .merge(backends::routes())
}
