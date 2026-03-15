# Event System

AgentForge uses an event-driven architecture. All state changes emit `ForgeEvent` variants through an `EventBus` broadcast channel.

## EventBus

```rust
pub struct EventBus {
    tx: broadcast::Sender<ForgeEvent>,
}
```

- Broadcast channel — multiple subscribers receive every event
- Used by: BatchWriter (persistence), WebSocket (real-time UI), analytics

## ForgeEvent (38 variants)

### Session lifecycle
- `SessionCreated`, `SessionStarted`, `SessionCompleted`, `SessionFailed`, `SessionResumed`

### Process execution
- `ProcessSpawned`, `ProcessOutput`, `ProcessCompleted`, `ProcessFailed`

### Tool use (HookReceiver)
- `ToolUseRequested` — Claude Code is about to use a tool
- `ToolUseCompleted` — tool use finished

### Security
- `SecurityScanPassed`, `SecurityScanFailed`

### Safety
- `RateLimitHit`, `CircuitBreakerTripped`, `BudgetWarning`, `BudgetExceeded`

### Multi-agent
- `SubAgentSpawned`, `SubAgentCompleted`, `SubAgentFailed`

### Pipeline
- `PipelineStarted`, `PipelineStepCompleted`, `PipelineCompleted`

### System
- `CompactionCompleted`, `Error`

## BatchWriter

Events are persisted to SQLite via BatchWriter:

- Batches up to **50 events** or flushes every **2 seconds**
- Writes in a single transaction for performance
- Extracts `session_id`, `agent_id`, and `event_type` for indexing

## WebSocket

The `/ws` endpoint streams events to the frontend in real time. The dashboard uses this for live agent output during runs.

## Subscribing to Events

```rust
let mut rx = event_bus.subscribe();
while let Ok(event) = rx.recv().await {
    match event {
        ForgeEvent::ProcessOutput { content, .. } => { /* handle */ }
        _ => {}
    }
}
```
