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

## ForgeEvent (43 variants)

### System lifecycle
- `SystemStarted` — server started with version
- `SystemStopped` — server shutting down
- `Heartbeat` — periodic liveness signal

### Agent lifecycle
- `AgentCreated` — new agent registered
- `AgentUpdated` — agent configuration changed
- `AgentDeleted` — agent removed

### Process execution
- `ProcessStarted` — CLI process spawned for a session
- `ProcessOutput` — stdout/stderr chunk (with `OutputKind`: Assistant, ToolUse, ToolResult, Thinking, Result)
- `ProcessCompleted` — CLI process exited with code
- `ProcessFailed` — CLI process error

### Session lifecycle
- `SessionCreated` — new session opened
- `SessionResumed` — existing session resumed
- `SessionCompleted` — session finished (from HookReceiver stop event)

### Workflow lifecycle
- `WorkflowStarted` — workflow execution began
- `WorkflowStepCompleted` — a workflow step finished
- `WorkflowCompleted` — all workflow steps done
- `WorkflowFailed` — workflow execution error

### Safety
- `CircuitBreakerTripped` — circuit breaker opened for an agent
- `BudgetWarning` — cost approaching limit
- `BudgetExceeded` — cost exceeded limit

### Hook lifecycle
- `HookStarted` — a Claude Code hook began executing
- `HookCompleted` — hook finished with duration
- `HookFailed` — hook execution error

### Sub-agent lifecycle
- `SubAgentRequested` — parent session requested a sub-agent
- `SubAgentStarted` — sub-agent session started
- `SubAgentCompleted` — sub-agent session finished
- `SubAgentFailed` — sub-agent error

### Schedule lifecycle
- `ScheduleCreated` — new cron schedule registered
- `ScheduleTriggered` — schedule fired, session started
- `ScheduleDeleted` — schedule removed

### Exit gate
- `ExitGateTriggered` — session stopped by exit gate rule

### Quality checks
- `QualityCheckStarted` — quality gate evaluation began
- `QualityCheckPassed` — quality score above threshold
- `QualityCheckFailed` — quality score below threshold

### Pipeline lifecycle
- `PipelineStarted` — multi-step pipeline began
- `PipelineStepCompleted` — a pipeline step finished
- `PipelineCompleted` — all pipeline steps done

### Compaction
- `CompactionCompleted` — context compaction with token counts

### Tool use (HookReceiver)
- `ToolUseRequested` — Claude Code is about to use a tool
- `ToolUseCompleted` — tool use finished

### Security
- `SecurityScanPassed` — security scan found no issues
- `SecurityScanFailed` — security scan found findings

### Generic
- `Error` — catch-all error with message and optional context

## Event Normalization (Multi-Backend)

When multiple backends (Claude, Hermes, OpenClaw) produce output, their raw stream-json events are normalized to `ForgeEvent` via `normalize_to_forge_event()` in `forge-process/src/stream_event.rs`.

This ensures the UI, BatchWriter, and analytics work identically regardless of which backend produced the output. The `SpawnMiddleware` calls this function in its event processing loop rather than constructing events inline.

```rust
let forge_events = normalize_to_forge_event(backend_name, &raw_event, &session_id, &agent_id);
for event in forge_events {
    event_bus.emit_sync(event)?;
}
```

Each backend must produce `ProcessHandle` instances whose stdout emits newline-delimited JSON parseable by `parse_line()`. The normalization layer then maps these to the appropriate `ForgeEvent` variants.

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
