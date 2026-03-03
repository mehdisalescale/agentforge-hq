//! API route handlers.

mod agents;
mod health;
mod hooks;
mod memory;
mod run;
mod sessions;
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
}
