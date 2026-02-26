//! Broadcast event bus for ForgeEvent. Emit from any component; subscribers receive via broadcast channel.

use crate::error::ForgeResult;
use crate::events::ForgeEvent;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Broadcast event bus. When there are no subscribers, emit still succeeds (we log and do not propagate).
pub struct EventBus {
    sender: broadcast::Sender<ForgeEvent>,
}

impl EventBus {
    /// Create a new event bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Emit an event to all current subscribers. Returns Ok even when there are no receivers.
    pub fn emit(&self, event: ForgeEvent) -> ForgeResult<()> {
        debug!(
            event_type = ?std::mem::discriminant(&event),
            "emitting event"
        );
        if let Err(e) = self.sender.send(event) {
            warn!("No active subscribers: {}", e);
        }
        Ok(())
    }

    /// Subscribe to events. Returns a receiver that receives all events from this point forward.
    pub fn subscribe(&self) -> broadcast::Receiver<ForgeEvent> {
        self.sender.subscribe()
    }

    /// Number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Trait for anything that consumes events (DB writer, WebSocket, logger).
pub trait EventSink: Send + Sync {
    fn handle(&self, event: &ForgeEvent);
}
