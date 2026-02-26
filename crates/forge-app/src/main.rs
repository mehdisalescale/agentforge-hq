//! Forge binary: DB, migrations, EventBus, BatchWriter, AgentRepo, API server (FORGE_HOST:FORGE_PORT).
//! Graceful shutdown on Ctrl+C: server stops accepting, then BatchWriter flushes and exits.

use forge_api::{serve_until_signal, AppState};
use forge_core::EventBus;
use forge_db::{AgentRepo, BatchWriter, DbPool, EventRepo, Migrator, SessionRepo, SkillRepo, WorkflowRepo};
use forge_safety::{CircuitBreaker, RateLimiter};
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("shutdown signal received");
}

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

    let rate_limit_max: u32 = env::var("FORGE_RATE_LIMIT_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let rate_limit_refill_ms: u64 = env::var("FORGE_RATE_LIMIT_REFILL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    let rate_limiter = Arc::new(RateLimiter::new(
        rate_limit_max,
        std::time::Duration::from_millis(rate_limit_refill_ms),
    ));

    let state = AppState::new(
        Arc::new(agent_repo),
        Arc::new(session_repo),
        Arc::new(event_repo),
        Arc::new(event_bus),
        Arc::new(skill_repo),
        Arc::new(workflow_repo),
        Arc::new(CircuitBreaker::default()),
        rate_limiter,
    );

    let host = env::var("FORGE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("FORGE_PORT").unwrap_or_else(|_| "4173".into());
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!(%addr, "starting forge server");
    serve_until_signal(addr, state, shutdown_signal()).await?;

    // Shut down BatchWriter so it flushes remaining events. If another ref exists, drop and let thread exit on channel close.
    match Arc::try_unwrap(batch_writer) {
        Ok(bw) => {
            if let Err(e) = bw.shutdown() {
                tracing::warn!(error = %e, "batch writer shutdown error");
            }
        }
        Err(arc) => {
            tracing::warn!(
                "batch writer: extra refs held, dropping (thread will flush on channel close)"
            );
            drop(arc);
        }
    }
    Ok(())
}
