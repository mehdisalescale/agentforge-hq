# Agent R2-A: Event Bus Fan-Out Pipeline

> Replace single broadcast channel with fan-out: mpsc for persistence (guaranteed delivery), broadcast for WebSocket (best-effort). Silent event loss kills observability.

## Step 1: Read Context

- `CLAUDE.md`
- `crates/forge-core/src/event_bus.rs` — current: single `broadcast::Sender<ForgeEvent>` with capacity passed in
- `crates/forge-core/src/lib.rs` — re-exports `EventBus`, `EventSink`; tests use `EventBus::new(16)`
- `crates/forge-app/src/main.rs` — lines 77 (`EventBus::new(256)`), lines 97-118 (BatchWriter wiring via `bus.subscribe()`)
- `crates/forge-api/src/routes/ws.rs` — `state.event_bus.subscribe()` for WebSocket forwarding
- `crates/forge-db/src/batch_writer.rs` — receives events via `crossbeam_channel::bounded(1024)`, flushes in batches

## Step 2: Redesign EventBus

The current design uses a single `tokio::sync::broadcast` channel for ALL consumers. Problem: broadcast drops messages when slow receivers lag (BatchWriter doing I/O). The BatchWriter bridge in main.rs already logs "subscriber lagged, lost events" — this means persistence can silently lose events.

**New design:** Two separate channels:
1. **mpsc** for persistence — guaranteed delivery with backpressure (BatchWriter is the only consumer)
2. **broadcast** for UI — best-effort, WebSocket clients can tolerate drops

Replace `crates/forge-core/src/event_bus.rs` with:

```rust
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
    /// Use this from sync contexts (e.g. middleware). Falls back to best-effort if persist channel is full.
    pub fn emit_sync(&self, event: ForgeEvent) -> ForgeResult<()> {
        debug!(
            event_type = ?std::mem::discriminant(&event),
            "emitting event (sync)"
        );

        match self.persist_tx.try_send(event.clone()) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("persistence channel full, event may be delayed");
                // Don't drop — the broadcast below still goes through for UI
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
    /// These subscribers get best-effort delivery — they may miss events under load.
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
```

**Key changes:**
- `EventBus::new()` now returns `(EventBus, mpsc::Receiver<ForgeEvent>)` — the receiver goes directly to BatchWriter
- `emit()` is now async (sends to mpsc with backpressure)
- Added `emit_sync()` for sync contexts that can't await
- `subscribe()` still exists for WebSocket/UI (broadcast channel)

## Step 3: Update BatchWriter Wiring in main.rs

In `crates/forge-app/src/main.rs`, the EventBus construction and BatchWriter wiring must change.

**Current (lines 77, 97-118):**
```rust
let event_bus = EventBus::new(256);
// ... later ...
let batch_writer = Arc::new(BatchWriter::spawn(Arc::clone(&conn_arc)));
let bw = Arc::clone(&batch_writer);
let mut event_rx = event_bus.subscribe();
tokio::spawn(async move {
    loop {
        match event_rx.recv().await {
            Ok(event) => { bw.write(event)... }
            Err(RecvError::Lagged(n)) => { warn!(...) }
            Err(RecvError::Closed) => { break; }
        }
    }
});
```

**Replace with:**
```rust
let (event_bus, persist_rx) = EventBus::new(1024, 256);

// ... later ...
let batch_writer = Arc::new(BatchWriter::spawn(Arc::clone(&conn_arc)));
let bw = Arc::clone(&batch_writer);
// Persistence channel: guaranteed delivery via mpsc (no Lagged errors possible)
let mut persist_rx = persist_rx;
tokio::spawn(async move {
    while let Some(event) = persist_rx.recv().await {
        if let Err(e) = bw.write(event) {
            tracing::warn!(error = %e, "batch writer: failed to queue event");
        }
    }
    info!("persistence channel closed, stopping event persistence");
});
info!("event persistence wired (BatchWriter ← mpsc guaranteed channel)");
```

**Note:** The mpsc receiver loop is simpler — no `Lagged` error handling needed because mpsc applies backpressure instead of dropping.

## Step 4: Update WebSocket Handler

`crates/forge-api/src/routes/ws.rs` — this still uses `state.event_bus.subscribe()` which returns a broadcast receiver. **No change needed** — the `subscribe()` method still works identically.

However, add a comment explaining the delivery guarantee:

```rust
// Subscribe to the broadcast channel (best-effort delivery for UI).
// Events may be dropped under heavy load — this is acceptable for real-time UI streaming.
let mut bus_rx = state.event_bus.subscribe();
```

## Step 5: Update All emit() Call Sites

The `emit()` method is now `async`. Search for all `event_bus.emit(` calls across the codebase and determine if they're in async or sync contexts:

- If in an async function → change to `event_bus.emit(event).await`
- If in a sync context → change to `event_bus.emit_sync(event)`

Common locations:
- `crates/forge-api/src/middleware.rs` — multiple emit calls in middleware chain (async context)
- `crates/forge-api/src/routes/run.rs` — emit calls (async context)
- `crates/forge-app/src/main.rs` — SystemStarted/Stopped events

Search: `grep -r "event_bus.emit(" crates/` to find all call sites.

## Step 6: Update Tests

In `crates/forge-core/src/lib.rs`, tests use `EventBus::new(16)`. Update to new signature:

```rust
let (bus, _persist_rx) = EventBus::new(16, 16);
```

Tests that check `bus.subscribe()` still work — broadcast channel is unchanged.

For tests that need to verify persistence channel, use the `persist_rx` receiver.

## Step 7: Verify

```bash
cargo check 2>&1 | head -20    # zero warnings
cargo test --workspace 2>&1 | tail -10    # all pass
```

## Rules

- Touch ONLY: `crates/forge-core/src/event_bus.rs`, `crates/forge-core/src/lib.rs` (tests), `crates/forge-app/src/main.rs` (event bus construction + BatchWriter wiring ONLY), `crates/forge-api/src/routes/ws.rs` (comment only), and any files with `event_bus.emit(` calls
- Do NOT touch `crates/forge-db/` (Agent R2-B handles that)
- Do NOT touch `crates/forge-safety/` (Agent R2-C handles that)
- Do NOT touch `crates/forge-process/src/spawn.rs` (Agent R2-C handles that)
- Do NOT touch `site-docs/`, `CLAUDE.md`, `README.md`, `.github/workflows/`
- Run `cargo check` and `cargo test` before reporting done

## Report

When done, create `docs/agents/REPORT_R2A.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
FILES_MODIFIED: [list]
EMIT_CALLSITES_UPDATED: [count] async, [count] sync
TESTS_UPDATED: [count]
CARGO_CHECK: pass/fail
CARGO_TEST: pass/fail
NOTES: [any design decisions made]
```
