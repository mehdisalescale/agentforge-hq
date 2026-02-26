//! Forge binary: DB, migrations, EventBus, AgentRepo, API server on 127.0.0.1:4173.
//! No frontend yet — API only.

use forge_api::{serve, AppState};
use forge_core::EventBus;
use forge_db::{AgentRepo, DbPool, EventRepo, Migrator, SessionRepo};
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::info;

fn default_db_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/.claude-forge/forge.db", home)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let db_path = env::var("FORGE_DB_PATH").unwrap_or_else(|_| default_db_path());
    let path = Path::new(&db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    info!(path = %db_path, "opening database");
    let db = DbPool::new(path)?;
    {
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        let applied = migrator.apply_pending()?;
        if applied > 0 {
            info!(count = applied, "migrations applied");
        }
    }

    let conn_arc = db.conn_arc();
    let agent_repo = AgentRepo::new(Arc::clone(&conn_arc));
    let session_repo = SessionRepo::new(Arc::clone(&conn_arc));
    let event_repo = EventRepo::new(Arc::clone(&conn_arc));
    let event_bus = EventBus::new(16);
    let state = AppState::new(
        Arc::new(agent_repo),
        Arc::new(session_repo),
        Arc::new(event_repo),
        Arc::new(event_bus),
    );

    let addr: SocketAddr = "127.0.0.1:4173".parse()?;
    info!(%addr, "starting API server (no frontend)");
    serve(addr, state).await?;
    Ok(())
}
