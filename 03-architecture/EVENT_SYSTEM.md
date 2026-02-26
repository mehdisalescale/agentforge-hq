# Claude Forge -- Event System

> Event architecture: types, flows, persistence, streaming, replay, and cross-context contracts.
> The event bus is the backbone of Forge -- every agent interaction is an immutable event.

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Event Types](#event-types)
3. [Event Flow Diagrams](#event-flow-diagrams)
4. [Event Persistence](#event-persistence)
5. [Event Streaming](#event-streaming)
6. [Event Replay](#event-replay)
7. [Cross-Context Event Contracts](#cross-context-event-contracts)
8. [Implementation Details](#implementation-details)

---

## Design Philosophy

| Principle | Implementation |
|-----------|---------------|
| Events are immutable | Once created, a TaggedEvent is never modified. Corrections create new events. |
| Events are the source of truth | Agent state (usage, status) is derived from events. The DashMap is a cache. |
| Events flow through a single bus | All events pass through `broadcast::Sender<TaggedEvent>`. No direct coupling between producers and consumers. |
| Consumers are independent | Each consumer (WebSocket, accumulator, safety) subscribes separately. One consumer's failure does not affect others. |
| Batch persistence for throughput | Events are written to SQLite in batches (50 events or 2 seconds) to amortize transaction overhead. |
| Replay from persistence | On startup, events are loaded from SQLite to reconstruct in-memory state. |

---

## Event Types

### Core Event Enum

Events are classified by their `event_type` field, which is extracted from the Claude Code stream-json output's `type` field.

```rust
/// Event types from Claude Code's stream-json output.
/// Extracted from event.get("type") on each stdout line.
pub enum EventType {
    // === From Claude Code stream-json ===
    System,      // Session initialization
    Assistant,   // LLM response (text, content blocks, usage)
    User,        // User message echo
    ToolUse,     // Tool invocation request
    ToolResult,  // Tool execution result
    Result,      // Final prompt result

    // === Forge-internal events (planned) ===
    AgentCreated,
    AgentUpdated,
    AgentDeleted,
    AgentStatusChanged,
    ProcessSpawned,
    ProcessExited,
    WorkflowStarted,
    WorkflowStepStarted,
    WorkflowStepCompleted,
    WorkflowCompleted,
    WorkflowFailed,
    BudgetWarning,
    BudgetExceeded,
    HandoffInitiated,
    BroadcastSent,
    SkillInvoked,
    McpToolInvoked,
    BatchFlushed,
    ClientConnected,
    ClientDisconnected,
}
```

### Event Data Structures

#### TaggedEvent (Current Implementation)

```rust
/// A Claude Code event tagged with Forge metadata.
/// This is the canonical event type on the broadcast channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedEvent {
    pub agent_id: Uuid,                       // Which agent produced this event
    pub event: serde_json::Value,             // Raw Claude stream-json event
    pub event_type: String,                   // Extracted from event["type"]
    pub timestamp: DateTime<Utc>,             // When Forge received the event
}
```

#### ForgeEvent (Planned -- unified internal + external events)

```rust
/// Unified event type that covers both Claude Code events and Forge-internal events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeEvent {
    pub id: Option<i64>,                      // DB row ID (None until persisted)
    pub agent_id: Uuid,                       // Source agent
    pub event_type: EventType,                // Classified event type
    pub payload: EventPayload,                // Type-specific payload
    pub timestamp: DateTime<Utc>,             // When the event occurred
    pub session_id: Option<String>,           // Claude Code session (if applicable)
    pub prompt_id: Option<String>,            // Groups events from one prompt cycle
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    /// Raw Claude Code stream-json event (passed through)
    Claude(serde_json::Value),

    /// Forge-internal structured event
    Internal(InternalEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum InternalEvent {
    AgentCreated { name: String, model: String, preset_id: Option<String> },
    AgentStatusChanged { old_status: AgentStatus, new_status: AgentStatus },
    ProcessSpawned { pid: u32, cli_args: Vec<String> },
    ProcessExited { exit_code: Option<i32>, duration_ms: u64 },
    BudgetWarning { spent: f64, limit: f64, percent: f64 },
    BudgetExceeded { spent: f64, limit: f64 },
    WorkflowStepCompleted { workflow_id: String, step_id: String, cost_usd: f64 },
    HandoffInitiated { from_agent: Uuid, to_agent: Uuid },
    BroadcastSent { sender: Uuid, group_id: Option<String>, message: String },
    BatchFlushed { event_count: usize, agent_ids: Vec<Uuid> },
}
```

### Claude Code Stream-JSON Event Shapes

These are the raw event shapes produced by `claude -p "..." --output-format stream-json --verbose`:

#### system

```json
{
    "type": "system",
    "subtype": "init",
    "session_id": "sess_01J...",
    "tools": [
        { "name": "Read", "description": "Read a file..." },
        { "name": "Write", "description": "Write a file..." },
        { "name": "Bash", "description": "Execute bash..." }
    ],
    "mcp_servers": [
        { "name": "gitnexus", "status": "connected" }
    ],
    "model": "claude-sonnet-4-20250514"
}
```

#### assistant

```json
{
    "type": "assistant",
    "message": {
        "id": "msg_01XY...",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "I'll analyze the authentication module..."
            }
        ],
        "model": "claude-sonnet-4-20250514",
        "usage": {
            "input_tokens": 5140,
            "output_tokens": 1070,
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 4800
        },
        "stop_reason": "end_turn"
    }
}
```

#### user

```json
{
    "type": "user",
    "message": {
        "role": "user",
        "content": [
            {
                "type": "text",
                "text": "Please refactor the database module."
            }
        ]
    }
}
```

#### tool_use (within assistant message)

```json
{
    "type": "assistant",
    "message": {
        "role": "assistant",
        "content": [
            {
                "type": "tool_use",
                "id": "toolu_01AB...",
                "name": "Read",
                "input": {
                    "file_path": "/Users/bm/project/src/db.rs",
                    "limit": 2000
                }
            }
        ]
    }
}
```

#### tool_result

```json
{
    "type": "user",
    "message": {
        "role": "user",
        "content": [
            {
                "type": "tool_result",
                "tool_use_id": "toolu_01AB...",
                "content": "use rusqlite::{params, Connection};\n..."
            }
        ]
    }
}
```

#### result

```json
{
    "type": "result",
    "subtype": "success",
    "result": "I've completed the refactoring of the database module...",
    "cost_usd": 0.0567,
    "duration_ms": 45200,
    "session_id": "sess_01J...",
    "usage": {
        "input_tokens": 25000,
        "output_tokens": 8000,
        "cache_creation_input_tokens": 5000,
        "cache_read_input_tokens": 20000
    }
}
```

---

## Event Flow Diagrams

### Primary Flow: Prompt to Result

```
User clicks "Send Prompt" in UI
         |
         v
    POST /api/agents/{id}/prompt
         |
         v
    process::send_prompt()
         |
    +----v----+
    | Spawn   | claude -p "..." --output-format stream-json --verbose
    | Process |
    +----+----+
         |
    stdout (line by line)
         |
    +----v----+
    | Stream  | Parse JSON, extract event_type,
    | Reader  | create TaggedEvent, add timestamp
    +----+----+
         |
    broadcast::Sender.send(tagged_event)
         |
    +----v---------------------------------------------+
    |                broadcast channel                  |
    |  (4096 capacity, lossy -- lagged receivers skip)  |
    +----+----------+----------+-----------+-----------+
         |          |          |           |
    +----v----+ +---v---+ +---v----+ +----v-----+
    | Event   | |WebSock| |Safety  | |MCP Rsrc  |
    | Accum.  | |Hub    | |Engine  | |Subscript.|
    +----+----+ +---+---+ +---+----+ +----+-----+
         |          |          |           |
    +----v----+     |     budget       resource
    | Batch   |     |     tracking     notifications
    | Buffer  |     |
    +----+----+     |
         |          |
    50 or 2s        |
         |          |
    +----v----+ +---v---+
    | SQLite  | |Client |
    | INSERT  | |Browser|
    +---------+ +-------+
```

### Startup Flow: Event Replay

```
    claude-forge starts
         |
    +----v----+
    | Open DB | Db::open(), WAL mode
    +----+----+
         |
    +----v----+
    | Migrate | Check schema_version, apply pending
    +----+----+
         |
    +----v----+
    | Load    | SELECT * FROM agents
    | Agents  | Deserialize config JSON, usage JSON
    +----+----+
         |
    For each agent:
         |
    +----v----+
    | Load    | SELECT * FROM events WHERE agent_id=?
    | Events  | ORDER BY id ASC LIMIT 10000
    +----+----+
         |
    +----v----+
    | Push to | AgentHandle.events.push_back(event)
    | Ring    | (VecDeque, up to 10K capacity)
    | Buffer  |
    +---------+
         |
    +----v----+
    | Insert  | DashMap.insert(agent_id, handle)
    | to Map  |
    +---------+
         |
    State restored. Ready to serve requests.
```

### WebSocket Subscription Flow

```
    Browser opens WebSocket connection
         |
    +----v----+
    | Upgrade | ws::handle_ws(socket, state)
    +----+----+
         |
    +----v----+
    | Default | SubscriptionFilter::All
    | Filter  |
    +----+----+
         |
    +----v----+                    +----------+
    | Subscribe| state             | Broadcast|
    | to bus   | .subscribe_events()| Channel  |
    +----+----+                    +-----+----+
         |                               |
         +-------------------------------+
         |
    tokio::select! loop:
         |
    +----v----+                    +----------+
    | Forward | broadcast::recv() | Client   |
    | Task    | -> filter match?  | Messages |
    +----+----+   -> send to ws   +----+-----+
         |                              |
         |                         Subscribe/Prompt/Ping
         |                              |
    +----v----+                    +----v-----+
    | To WS   | ServerMessage::Event| Handle   |
    | Client  | ServerMessage::Pong | client   |
    +---------+                    | commands |
                                   +----------+
```

### Event Accumulator Detail

```
    broadcast::Receiver<TaggedEvent>
         |
    +----v------------------------------------------+
    | Event Accumulator (single tokio task)          |
    |                                                 |
    |  State:                                         |
    |    pending_events: Vec<TaggedEvent>              |
    |    last_flush: Instant                           |
    |                                                 |
    |  Loop:                                          |
    |    tokio::select! {                             |
    |      event = rx.recv() => {                     |
    |        // Update agent handle in DashMap        |
    |        if event_type == "assistant" {            |
    |          extract_usage() -> accumulate_usage()   |
    |        }                                        |
    |        if session_id.is_none() {                |
    |          extract_session_id() -> set on handle  |
    |        }                                        |
    |        handle.push_event(tagged_event)          |
    |        pending_events.push(tagged_event)        |
    |      }                                          |
    |      _ = sleep(2s) => { /* timer fired */ }     |
    |    }                                            |
    |                                                 |
    |    if pending_events.len() >= 50                |
    |       || (elapsed >= 2s && !empty) {            |
    |                                                 |
    |      db.save_events(&pending_events)            |
    |      for each unique agent_id in pending:       |
    |        db.update_agent_state(id, session, usage)|
    |      pending_events.clear()                     |
    |      last_flush = Instant::now()                |
    |    }                                            |
    +------------------------------------------------+
```

---

## Event Persistence

### Storage Strategy

| Event Category | Stored | Retention | Rationale |
|---------------|--------|-----------|-----------|
| Claude stream-json events | Yes | Configurable (default 90 days) | Core event log, needed for replay and export |
| Forge-internal events (planned) | Yes | Same retention | Audit trail, workflow state reconstruction |
| Ephemeral status changes | No | In-memory only | Status is derived, not stored as events |
| WebSocket protocol messages | No | Not stored | Transport layer, not domain events |

### Write Path

```
Event created (TaggedEvent)
    |
    v
broadcast::channel (in-memory, 4096 capacity)
    |
    v
Event Accumulator receives event
    |
    +-- Update in-memory state (DashMap)
    +-- Buffer event in pending_events Vec
    |
    v
Flush trigger (50 events OR 2 seconds)
    |
    v
SQLite transaction:
    BEGIN;
    INSERT INTO events (agent_id, event_type, event, timestamp)
        VALUES (?1, ?2, ?3, ?4);  -- repeated for each event in batch
    UPDATE agents SET session_id=?1, usage=?2, updated_at=?3
        WHERE id=?4;              -- for each agent that had events
    COMMIT;
```

### Read Path

```
Hot read (recent events for active agents):
    DashMap.get(&agent_id).events  -- VecDeque ring buffer
    O(1) lookup, no SQLite access

Warm read (event pagination):
    SELECT * FROM events
    WHERE agent_id = ?
    ORDER BY id ASC
    LIMIT ? OFFSET ?
    -- Uses idx_events_agent_time index

Cold read (full-text search):
    SELECT e.* FROM events e
    JOIN fts_events f ON e.id = f.rowid
    WHERE fts_events MATCH ?
    ORDER BY f.rank
    LIMIT ?
    -- Uses FTS5 index
```

### Event Deletion (Planned)

```
Retention cleanup (daily background task):

    DELETE FROM events
    WHERE timestamp < datetime('now', '-90 days')
    AND agent_id IN (
        SELECT id FROM agents WHERE archived_at IS NOT NULL
    );

    -- FTS cleanup happens automatically via trigger events_ad

    PRAGMA incremental_vacuum(1000);
```

---

## Event Streaming

### WebSocket Subscription Model

```
Client connects to ws://host:4173/ws
    |
    v
Default subscription: all events from all agents
    |
    v
Client sends: { "type": "subscribe", "filter": { "agent_id": "uuid" } }
    |
    v
Filter updated: only events for that agent
    |
    v
Events matching filter are forwarded as ServerMessage::Event
```

**Subscription filters:**

| Filter | JSON | Behavior |
|--------|------|----------|
| All agents | `"all"` | Every event from every agent |
| Single agent | `{ "agent_id": "uuid" }` | Events from one agent only |
| Multiple agents (planned) | `{ "agent_ids": ["uuid1", "uuid2"] }` | Events from listed agents |
| Event type (planned) | `{ "event_types": ["assistant", "result"] }` | Only specified event types |
| Combined (planned) | `{ "agent_id": "uuid", "event_types": ["assistant"] }` | Agent + type filter |

### Backpressure and Lag

The broadcast channel has a capacity of 4096 events. When a consumer falls behind:

```
Producer (stream reader) sends at rate R
    |
    v
broadcast::channel (capacity 4096)
    |
    +-- Fast consumer (accumulator): keeps up, no lag
    |
    +-- Slow consumer (WebSocket client on slow network):
         |
         recv() returns Err(Lagged(n))
              |
              v
         Log warning: "ws client lagged by {n} events"
         Continue receiving from current position
         (skipped events are lost for this consumer)
```

**Design decision:** Lossy delivery is acceptable for WebSocket clients because:
- The browser can re-fetch current state via GET /api/agents/{id}
- Missing intermediate events (e.g., token-by-token streaming) are cosmetic
- The SQLite event log has the complete record

### SSE Alternative (Planned)

For simpler clients that cannot use WebSocket:

```
GET /api/agents/{id}/events/stream
Accept: text/event-stream

Response:
Content-Type: text/event-stream
Cache-Control: no-cache

event: system
id: 1
data: {"type":"system","session_id":"sess_01J..."}

event: assistant
id: 2
data: {"type":"assistant","message":{"content":[...]}}

event: result
id: 3
data: {"type":"result","result":"Completed."}
```

SSE advantages:
- Works with curl (`curl -N`)
- Built-in reconnection with `Last-Event-ID`
- Simpler protocol (HTTP, not WebSocket upgrade)

SSE limitations:
- Unidirectional (server to client only)
- Cannot send prompts through the same connection
- One connection per agent (vs. one WebSocket for all agents)

---

## Event Replay

### Purpose

Event replay is used for:
1. **Startup recovery** -- Rebuild in-memory state from SQLite
2. **Session resume** -- Agent continues a previous conversation via `--resume session_id`
3. **Export** -- Reconstruct full conversation for JSON/Markdown export
4. **Observability** -- Replay events for debugging or analysis

### Startup Replay

```rust
// In state.rs - AppState::restore_from_db()
pub fn restore_from_db(&self) {
    match self.inner.db.load_agents() {
        Ok(agents) => {
            for stored in agents {
                let mut handle = AgentHandle::new(stored.id, stored.config);
                handle.name = stored.name;
                handle.session_id = stored.session_id;
                handle.usage = stored.usage;
                handle.created_at = stored.created_at;

                // Replay last 10K events into ring buffer
                if let Ok(events) = self.inner.db.load_events(
                    stored.id,
                    EVENT_BUFFER_SIZE,  // 10,000
                    0
                ) {
                    for event in events {
                        handle.push_event(event);
                    }
                }

                self.inner.agents.insert(stored.id, handle);
            }
        }
        Err(e) => {
            tracing::error!("failed to load agents: {e}");
        }
    }
}
```

**Startup replay characteristics:**
- Loads ALL agents from the database
- For each agent, loads the last 10,000 events
- Events are pushed into the VecDeque ring buffer in chronological order
- Usage counters are loaded from the persisted `agents.usage` JSON (not re-derived from events)
- Session ID is loaded from `agents.session_id` (not re-extracted from events)

### Session Resume Replay

When a user sends a new prompt to an agent that already has a `session_id`:

```
User sends prompt "Continue the refactoring"
    |
    v
Agent has session_id = "sess_01J..."
    |
    v
Process spawning adds: --resume sess_01J...
    |
    v
Claude Code loads its own session history from ~/.claude/projects/
    |
    v
Claude Code continues the conversation (context preserved by Claude, not Forge)
```

**Key insight:** Forge does NOT replay events to Claude Code. Claude Code maintains its own session state. Forge only passes the `session_id` via `--resume` so Claude Code knows which session to continue.

### Export Replay

For the export endpoint, events are replayed to construct the output:

```
GET /api/agents/{id}/export?format=markdown
    |
    v
Read all events from agent.events (in-memory ring buffer)
    |
    v
For each event:
    event_type == "assistant" -> Extract text blocks -> "**Assistant**: {text}"
    event_type == "user"      -> Extract text blocks -> "**User**: {text}"
    event_type == "result"    -> Extract result text  -> "**Result**: {text}"
    other                     -> skip
    |
    v
Assemble Markdown document with agent metadata header
```

### Replay for Workflow State Reconstruction (Planned)

When a workflow run is resumed after a Forge restart:

```
1. Load workflow_runs row (status, context)
2. Load workflow_steps rows for this run
3. For each step with agent_id:
   a. Load agent from DashMap
   b. Check if agent is still running
   c. If step was "running" and agent is now "idle" -> step completed while Forge was down
   d. If step was "pending" and all dependencies complete -> ready to start
4. Resume execution from the current state
```

---

## Cross-Context Event Contracts

### Contract: Process Execution -> Event Streaming

**Producer:** Stream Reader (in process.rs)
**Consumer:** Event Streaming broadcast channel
**Event:** TaggedEvent

```
Guarantee: Every valid JSON line from Claude Code stdout becomes exactly one TaggedEvent.
Ordering: Events are delivered in stdout order per agent.
Timing: Events are broadcast within milliseconds of being read from stdout.
Loss: If no consumers are subscribed, events are dropped silently.
Format: event field contains the raw, unmodified JSON from Claude Code.
```

### Contract: Event Streaming -> Persistence

**Producer:** Event Streaming (broadcast channel)
**Consumer:** Event Accumulator (spawned task)
**Event:** TaggedEvent

```
Guarantee: Events are persisted in batches. At most 50 events or 2 seconds of data may be lost on crash.
Ordering: Events within a batch maintain their broadcast order.
Timing: Events are persisted within 2 seconds of creation.
Deduplication: None. If the same event is somehow broadcast twice, it will be stored twice.
Side effects: Agent usage and session_id are updated as a side effect of event persistence.
```

### Contract: Event Streaming -> WebSocket Hub

**Producer:** Event Streaming (broadcast channel)
**Consumer:** WebSocket forward tasks (one per client)
**Event:** TaggedEvent -> ServerMessage::Event

```
Guarantee: Best-effort delivery. Events may be lost if the client's receive buffer fills up.
Ordering: Events are delivered in broadcast order.
Filtering: Only events matching the client's current SubscriptionFilter are forwarded.
Format: TaggedEvent is transformed into ServerMessage::Event (agent_id, event, event_type).
Backpressure: If the mpsc channel to the WebSocket writer fills (capacity 256), the forward task blocks.
Disconnection: If the WebSocket send fails, the connection is closed and the forward task is aborted.
```

### Contract: Event Streaming -> Safety Engine (Planned)

**Producer:** Event Streaming (broadcast channel)
**Consumer:** Safety monitoring task
**Event:** TaggedEvent (assistant events with usage)

```
Guarantee: Safety checks are best-effort. A budget overrun may not be detected until the next batch.
Trigger: Only "assistant" events with usage data are processed.
Action: When budget threshold (80%) is reached, emit BudgetWarning event.
Action: When budget is exceeded, emit BudgetExceeded event and optionally stop the agent.
Latency: Budget checks happen within the event accumulator loop (same cadence as persistence).
```

### Contract: Agent Management -> Event Streaming (Planned)

**Producer:** Agent Management (API handlers)
**Consumer:** Event Streaming (broadcast channel)
**Event:** ForgeEvent with InternalEvent payload

```
Guarantee: Forge-internal events are published synchronously during the API handler execution.
Format: ForgeEvent with InternalEvent::AgentCreated, AgentUpdated, etc.
Persistence: Internal events are stored in the same events table with a "forge_" prefix on event_type.
Replay: Internal events participate in startup replay like Claude events.
```

### Contract: Workflow Engine -> Event Streaming (Planned)

**Producer:** Workflow Engine
**Consumer:** Event Streaming (broadcast channel)
**Events:** ForgeEvent with WorkflowStarted, WorkflowStepCompleted, etc.

```
Guarantee: Every workflow state transition produces an event.
Ordering: Events are published in execution order.
Idempotency: Step completion events include the step_id. Re-publishing is harmless.
Consumers:
  - WebSocket Hub: forwards workflow events to subscribed clients
  - Persistence: stores for audit trail and state reconstruction
  - Workflow Engine itself: uses events to detect step completion and trigger next steps
```

---

## Implementation Details

### Broadcast Channel Configuration

```rust
const BROADCAST_CHANNEL_SIZE: usize = 4096;

// In AppState::new()
let (event_tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);
```

**Why 4096?**
- A busy agent produces ~10 events/second
- 4096 events = ~400 seconds of buffer for a single agent
- With 10 concurrent agents: ~40 seconds of buffer
- Slow WebSocket clients get Lagged errors after ~40s of unprocessed events
- This is acceptable: clients can re-fetch state via REST

### Ring Buffer Configuration

```rust
const EVENT_BUFFER_SIZE: usize = 10_000;

// In AgentHandle
pub events: VecDeque<TaggedEvent>,  // capacity EVENT_BUFFER_SIZE

pub fn push_event(&mut self, event: TaggedEvent) {
    if self.events.len() >= EVENT_BUFFER_SIZE {
        self.events.pop_front();  // Drop oldest
    }
    self.events.push_back(event);
}
```

**Why 10,000?**
- A typical session has 50-1000 events
- 10K events covers ~10-200 sessions worth of history
- At ~3KB average per event: ~30MB per agent in memory
- For 10 agents: ~300MB total (acceptable for a developer workstation)

### Usage Extraction

```rust
pub fn extract_usage(event: &serde_json::Value) -> Option<UsageDelta> {
    let usage = event.get("message")?.get("usage")?;

    Some(UsageDelta {
        input_tokens: usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        output_tokens: usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        cache_creation_tokens: usage.get("cache_creation_input_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0),
        cache_read_tokens: usage.get("cache_read_input_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0),
    })
}
```

Usage is extracted from `assistant` events only. Other event types do not carry usage data.

### Session ID Extraction

```rust
pub fn extract_session_id(event: &serde_json::Value) -> Option<String> {
    if event.get("type")?.as_str()? == "system" {
        return event.get("session_id").and_then(|v| v.as_str()).map(String::from);
    }
    None
}
```

Session ID is extracted from the first `system` event in a prompt cycle. Once set on the AgentHandle, subsequent prompts use `--resume` with this session ID.

### Cost Estimation

```rust
fn estimate_cost(model: &str, input: u64, output: u64, cache_create: u64, cache_read: u64) -> f64 {
    let (input_price, output_price, cache_create_price, cache_read_price) =
        if model.contains("opus")  { (15.0, 75.0, 18.75, 1.5) }
        else if model.contains("haiku") { (0.25, 1.25, 0.3, 0.03) }
        else { (3.0, 15.0, 3.75, 0.3) };  // sonnet default

    let m = 1_000_000.0;
    (input as f64 / m) * input_price
        + (output as f64 / m) * output_price
        + (cache_create as f64 / m) * cache_create_price
        + (cache_read as f64 / m) * cache_read_price
}
```

Cost is estimated per-model using approximate per-million-token pricing. This runs on every `assistant` event with usage data.

### Error Handling in the Event Pipeline

```
Stream Reader:
  - Invalid JSON line -> skip (continue reading)
  - stdout closed -> log info, task exits
  - broadcast send fails -> ignore (no receivers)

Event Accumulator:
  - Lagged receiver -> log warning, continue
  - Channel closed -> break loop (shutdown)
  - DB write fails -> log error, keep events in buffer for retry
  - DashMap access fails -> agent was deleted, skip

WebSocket Forward:
  - Lagged receiver -> log warning, continue
  - Channel closed -> break, close connection
  - WebSocket send fails -> break, close connection
  - JSON serialization fails -> skip event
```

### Graceful Shutdown

```
Ctrl+C received
    |
    v
shutdown_signal() resolves
    |
    v
Axum graceful shutdown begins
    |
    v
In-flight HTTP requests complete (drain period)
    |
    v
WebSocket connections are closed
    |
    v
broadcast::Sender is dropped
    |
    v
All broadcast::Receivers get RecvError::Closed
    |
    v
Event accumulator loop breaks
    |
    v
Remaining pending_events are flushed to DB (implicit -- the final flush happens because the loop exits)
    |
    v
Child processes are orphaned (they continue running independently)
    |
    v
Process exits
```

**Note:** Child Claude processes are NOT killed on Forge shutdown. They run to completion independently. On next Forge startup, their events are NOT recovered (they were writing to stdout, which is now disconnected). This is acceptable because Claude Code also persists its own sessions to `~/.claude/projects/`.

---

*This completes the architecture documentation suite. See also:*
- *[SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md) -- C4 model overview*
- *[BOUNDED_CONTEXTS.md](BOUNDED_CONTEXTS.md) -- DDD context boundaries*
- *[DATA_MODEL.md](DATA_MODEL.md) -- SQLite schema and migrations*
- *[API_DESIGN.md](API_DESIGN.md) -- REST and WebSocket API*
- *[MCP_INTERFACE.md](MCP_INTERFACE.md) -- MCP server tools, resources, prompts*
