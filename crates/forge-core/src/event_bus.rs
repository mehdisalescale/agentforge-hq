//! Fan-out event bus: guaranteed delivery for persistence, best-effort for UI subscribers.

use crate::error::ForgeResult;
use crate::events::ForgeEvent;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, warn};

/// Fan-out event bus. Events are sent to:
/// - A guaranteed mpsc channel for persistence (BatchWriter)
/// - A best-effort broadcast channel for UI (WebSocket)
pub struct EventBus {
    /// Guaranteed delivery channel for persistence. Backpressure if full.
    persist_tx: mpsc::Sender<ForgeEvent>,
    /// Best-effort broadcast for UI subscribers. Drops on lag.
    broadcast_tx: broadcast::Sender<ForgeEvent>,
}

impl EventBus {
    /// Create a new fan-out event bus.
    /// - `persist_capacity`: mpsc buffer size for persistence channel
    /// - `broadcast_capacity`: broadcast buffer size for UI subscribers
    pub fn new(persist_capacity: usize, broadcast_capacity: usize) -> (Self, mpsc::Receiver<ForgeEvent>) {
        let (persist_tx, persist_rx) = mpsc::channel(persist_capacity);
        let (broadcast_tx, _) = broadcast::channel(broadcast_capacity);
        (
            Self {
                persist_tx,
                broadcast_tx,
            },
            persist_rx,
        )
    }

    /// Emit an event to all channels.
    /// Persistence channel: awaits if full (backpressure).
    /// Broadcast channel: best-effort (logs if no subscribers).
    pub async fn emit(&self, event: ForgeEvent) -> ForgeResult<()> {
        debug!(
            event_type = ?std::mem::discriminant(&event),
            "emitting event"
        );

        // Guaranteed: send to persistence channel (backpressure if full)
        if let Err(e) = self.persist_tx.send(event.clone()).await {
            warn!("persistence channel closed: {}", e);
        }

        // Best-effort: send to broadcast for UI
        if let Err(e) = self.broadcast_tx.send(event) {
            // This is normal when no WebSocket clients are connected
            debug!("no broadcast subscribers: {}", e);
        }

        Ok(())
    }

    /// Emit synchronously (non-async). Uses try_send for persistence.
    /// Use this from sync contexts (e.g. ProcessRunner). Falls back to best-effort if persist channel is full.
    pub fn emit_sync(&self, event: ForgeEvent) -> ForgeResult<()> {
        debug!(
            event_type = ?std::mem::discriminant(&event),
            "emitting event (sync)"
        );

        match self.persist_tx.try_send(event.clone()) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("persistence channel full, event may be delayed");
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                warn!("persistence channel closed");
            }
        }

        if let Err(e) = self.broadcast_tx.send(event) {
            debug!("no broadcast subscribers: {}", e);
        }

        Ok(())
    }

    /// Subscribe to the broadcast channel (for WebSocket/UI consumers).
    /// These subscribers get best-effort delivery -- they may miss events under load.
    pub fn subscribe(&self) -> broadcast::Receiver<ForgeEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Number of active broadcast subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.broadcast_tx.receiver_count()
    }
}

/// Trait for anything that consumes events (DB writer, WebSocket, logger).
pub trait EventSink: Send + Sync {
    fn handle(&self, event: &ForgeEvent);
}
