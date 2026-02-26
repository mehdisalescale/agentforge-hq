//! Forge binary: DB, migrations, EventBus, BatchWriter, AgentRepo, API server on 127.0.0.1:4173.
//! No frontend yet — API only.

use forge_api::{serve, AppState};
use forge_core::EventBus;
use forge_db::{AgentRepo, BatchWriter, DbPool, EventRepo, Migrator, SessionRepo, SkillRepo, WorkflowRepo};
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
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
    let skill_repo = SkillRepo::new(Arc::clone(&conn_arc));
    let workflow_repo = WorkflowRepo::new(Arc::clone(&conn_arc));
    let event_bus = EventBus::new(256);

    // S1: Wire BatchWriter to EventBus — persist all events to SQLite.
    let batch_writer = Arc::new(BatchWriter::spawn(Arc::clone(&conn_arc)));
    let bw = Arc::clone(&batch_writer);
    let mut event_rx = event_bus.subscribe();
    tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    if let Err(e) = bw.write(event) {
                        tracing::warn!(error = %e, "batch writer: failed to queue event");
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(count = n, "batch writer: subscriber lagged, lost events");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    info!("event bus closed, stopping event persistence");
                    break;
                }
            }
        }
    });
    info!("event persistence wired (BatchWriter → EventBus)");

    let state = AppState::new(
        Arc::new(agent_repo),
        Arc::new(session_repo),
        Arc::new(event_repo),
        Arc::new(event_bus),
        Arc::new(skill_repo),
        Arc::new(workflow_repo),
    );

    let addr: SocketAddr = "127.0.0.1:4173".parse()?;
    info!(%addr, "starting API server (no frontend)");
    serve(addr, state).await?;

    // Drop the last Arc ref so the batch writer thread can flush and exit.
    drop(batch_writer);
    Ok(())
}
