//! API route handlers.

mod agents;
mod health;
mod run;
mod sessions;
mod skills;
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
        .merge(ws::routes())
}
