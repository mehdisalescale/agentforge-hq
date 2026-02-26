//! Batch writer: events accumulate in a channel and flush to SQLite in batches.

use crossbeam_channel::{bounded, select, tick, Receiver, Sender};
use forge_core::error::{ForgeError, ForgeResult};
use forge_core::events::ForgeEvent;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

const BATCH_SIZE: usize = 50;
const FLUSH_INTERVAL: Duration = Duration::from_secs(2);

pub struct BatchWriter {
    sender: Sender<ForgeEvent>,
    handle: Option<thread::JoinHandle<()>>,
}

impl BatchWriter {
    /// Spawn a dedicated writer thread.
    /// Flushes when: BATCH_SIZE events accumulated OR FLUSH_INTERVAL elapsed.
    /// Pass Arc<Mutex<Connection>> so the same connection can be used for queries (e.g. in tests).
    pub fn spawn(conn: Arc<Mutex<Connection>>) -> Self {
        let (sender, receiver) = bounded::<ForgeEvent>(1024);

        let handle = thread::spawn(move || {
            writer_loop(conn, receiver);
        });

        Self {
            sender,
            handle: Some(handle),
        }
    }

    /// Queue an event for batch writing. Non-blocking.
    pub fn write(&self, event: ForgeEvent) -> ForgeResult<()> {
        self.sender.send(event).map_err(|e| {
            ForgeError::Internal(format!("batch writer channel closed: {}", e))
        })
    }

    /// Flush remaining events and shut down the writer thread.
    pub fn shutdown(mut self) -> ForgeResult<()> {
        drop(self.sender);
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| {
                ForgeError::Internal("batch writer thread panicked".into())
            })?;
        }
        Ok(())
    }
}

fn writer_loop(conn: Arc<Mutex<Connection>>, receiver: Receiver<ForgeEvent>) {
    let mut buffer: Vec<ForgeEvent> = Vec::with_capacity(BATCH_SIZE);
    let ticker = tick(FLUSH_INTERVAL);

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(event) => {
                        buffer.push(event);
                        if buffer.len() >= BATCH_SIZE {
                            flush_to_db(&conn, &mut buffer);
                        }
                    }
                    Err(_) => {
                        if !buffer.is_empty() {
                            flush_to_db(&conn, &mut buffer);
                        }
                        info!("batch writer shutting down");
                        return;
                    }
                }
            }
            recv(ticker) -> _ => {
                if !buffer.is_empty() {
                    flush_to_db(&conn, &mut buffer);
                }
            }
        }
    }
}

fn flush_to_db(conn: &Arc<Mutex<Connection>>, buffer: &mut Vec<ForgeEvent>) {
    let conn = conn.lock().expect("db mutex poisoned");
    let count = buffer.len();

    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(e) => {
            error!(error = %e, "failed to begin transaction");
            return;
        }
    };

    for event in buffer.iter() {
        let id = Uuid::new_v4().to_string();
        let event_type = event_type_name(event);
        let data_json = serde_json::to_string(event).unwrap_or_default();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let (agent_id, session_id) = extract_ids(event);

        if let Err(e) = tx.execute(
            "INSERT INTO events (id, session_id, agent_id, event_type, data_json, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, session_id, agent_id, event_type, data_json, timestamp],
        ) {
            error!(error = %e, event_type = event_type, "failed to insert event");
        }
    }

    if let Err(e) = tx.commit() {
        error!(error = %e, "failed to commit batch");
    } else {
        debug!(count = count, "flushed events to db");
    }

    buffer.clear();
}

fn event_type_name(event: &ForgeEvent) -> &'static str {
    match event {
        ForgeEvent::SystemStarted { .. } => "SystemStarted",
        ForgeEvent::SystemStopped { .. } => "SystemStopped",
        ForgeEvent::Heartbeat { .. } => "Heartbeat",
        ForgeEvent::AgentCreated { .. } => "AgentCreated",
        ForgeEvent::AgentUpdated { .. } => "AgentUpdated",
        ForgeEvent::AgentDeleted { .. } => "AgentDeleted",
        ForgeEvent::ProcessStarted { .. } => "ProcessStarted",
        ForgeEvent::ProcessOutput { .. } => "ProcessOutput",
        ForgeEvent::ProcessCompleted { .. } => "ProcessCompleted",
        ForgeEvent::ProcessFailed { .. } => "ProcessFailed",
        ForgeEvent::SessionCreated { .. } => "SessionCreated",
        ForgeEvent::SessionResumed { .. } => "SessionResumed",
        ForgeEvent::WorkflowStarted { .. } => "WorkflowStarted",
        ForgeEvent::WorkflowStepCompleted { .. } => "WorkflowStepCompleted",
        ForgeEvent::WorkflowCompleted { .. } => "WorkflowCompleted",
        ForgeEvent::WorkflowFailed { .. } => "WorkflowFailed",
        ForgeEvent::CircuitBreakerTripped { .. } => "CircuitBreakerTripped",
        ForgeEvent::BudgetWarning { .. } => "BudgetWarning",
        ForgeEvent::BudgetExceeded { .. } => "BudgetExceeded",
        ForgeEvent::Error { .. } => "Error",
    }
}

fn extract_ids(event: &ForgeEvent) -> (Option<String>, Option<String>) {
    use forge_core::events::ForgeEvent;
    match event {
        ForgeEvent::AgentCreated { agent_id, .. }
        | ForgeEvent::AgentUpdated { agent_id, .. }
        | ForgeEvent::AgentDeleted { agent_id, .. }
        | ForgeEvent::CircuitBreakerTripped { agent_id, .. } => {
            (Some(agent_id.0.to_string()), None)
        }
        ForgeEvent::ProcessStarted {
            session_id,
            agent_id,
            ..
        }
        | ForgeEvent::SessionCreated {
            session_id,
            agent_id,
            ..
        } => (
            Some(agent_id.0.to_string()),
            Some(session_id.0.to_string()),
        ),
        ForgeEvent::ProcessOutput { session_id, .. }
        | ForgeEvent::ProcessCompleted { session_id, .. }
        | ForgeEvent::ProcessFailed { session_id, .. }
        | ForgeEvent::SessionResumed { session_id, .. } => {
            (None, Some(session_id.0.to_string()))
        }
        _ => (None, None),
    }
}
