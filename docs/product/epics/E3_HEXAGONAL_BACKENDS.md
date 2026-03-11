# Epic E3: Hexagonal Backend Architecture

> **Refactor process spawning into a Ports & Adapters pattern to support multiple execution backends.**
>
> This is the architectural foundation for E5 (Multi-Backend Execution).

---

## Business Value

Currently, forge-process is hardwired to spawn `claude` CLI. This epic introduces a `ProcessBackend` trait (port) that any runtime adapter can implement (adapter). This enables Hermes, OpenClaw, and future backends without touching the orchestration core.

## Acceptance Gate

**The epic is DONE when:**
1. `ProcessBackend` trait is defined with spawn/stream/kill operations
2. Existing Claude spawning reimplemented as `ClaudeBackend` adapter
3. All existing tests pass without modification
4. Backend selection is configurable per agent
5. Health check reports available backends
6. 15+ tests covering trait behavior, backend routing, and fallback

---

## User Stories

### E3-S1: ProcessBackend Trait (Port)

**As a** developer building a new runtime adapter,
**I want** a clear trait interface defining what a backend must implement,
**So that** I can add new backends without modifying core orchestration.

**Acceptance Criteria:**

```gherkin
GIVEN the ProcessBackend trait
WHEN I implement it
THEN I must provide:
  - name() -> &str
  - health_check() -> BackendHealth
  - spawn(config, prompt, session_id) -> ProcessHandle
  - capabilities() -> BackendCapabilities (tools, models, isolation_level)

GIVEN a backend implementation
WHEN I register it with the BackendRegistry
THEN it becomes available for agent runs

GIVEN the BackendRegistry
WHEN I query available_backends()
THEN I get all registered backends with their health status
```

**Technical Notes:**
- Trait in `forge-process/src/backend.rs`
- `BackendCapabilities { supports_streaming: bool, supports_tools: bool, isolation_levels: Vec<IsolationLevel>, supported_models: Vec<String> }`
- `BackendHealth { status: Healthy|Degraded|Unavailable, message: Option<String> }`
- `BackendRegistry`: `HashMap<String, Arc<dyn ProcessBackend>>`

**Test Plan:**
- `test_mock_backend_registers_and_queries`
- `test_backend_health_check_reports_status`
- `test_backend_capabilities_reported`
- `test_registry_returns_all_backends`

---

### E3-S2: Claude Backend Adapter (Extract & Reimpl)

**As a** system,
**I want** existing Claude CLI spawning extracted into a `ClaudeBackend` adapter,
**So that** it conforms to the `ProcessBackend` trait.

**Acceptance Criteria:**

```gherkin
GIVEN the ClaudeBackend adapter
WHEN I call spawn(config, prompt, session_id)
THEN it spawns `claude` CLI with stream-json output (same as current behavior)

GIVEN the ClaudeBackend
WHEN I call health_check()
THEN it verifies `claude --version` returns successfully

GIVEN the ClaudeBackend
WHEN I call capabilities()
THEN it returns { supports_streaming: true, supports_tools: true,
  isolation_levels: [Worktree], supported_models: ["claude-*"] }

GIVEN existing tests for process spawning
WHEN I run the test suite
THEN ALL existing tests pass without modification
```

**Technical Notes:**
- Extract current `spawn()` + `SpawnConfig` logic into `ClaudeBackend`
- `forge-process` keeps the same public API — just delegates to backend internally
- Backward compatible: default backend is Claude

**Test Plan:**
- All existing forge-process tests pass
- `test_claude_backend_health_check`
- `test_claude_backend_capabilities`

---

### E3-S3: Backend Routing in Middleware

**As a** system,
**I want** the middleware chain to route to the correct backend based on agent config,
**So that** each agent can use a different execution engine.

**Acceptance Criteria:**

```gherkin
GIVEN an agent with config_json.backend = "claude"
WHEN a run is initiated
THEN the SpawnMiddleware uses ClaudeBackend

GIVEN an agent with config_json.backend = "hermes"
WHEN a run is initiated and HermesBackend is registered
THEN the SpawnMiddleware uses HermesBackend

GIVEN an agent with config_json.backend = "hermes"
WHEN a run is initiated and HermesBackend is NOT available
THEN the middleware returns an error: "Backend 'hermes' not available"
AND a BackendHealthChanged event is emitted

GIVEN an agent with no backend specified
WHEN a run is initiated
THEN the default backend (claude) is used
```

**Technical Notes:**
- SpawnMiddleware receives `BackendRegistry` from AppState
- Backend lookup: `registry.get(agent.config.backend.unwrap_or("claude"))`
- New event: `BackendSwitched { agent_id, from, to }`

**Test Plan:**
- `test_default_backend_is_claude`
- `test_explicit_backend_selection`
- `test_unavailable_backend_returns_error`
- `test_backend_switched_event_emitted`

---

### E3-S4: Agent Backend Configuration

**As a** user,
**I want** to configure which backend each agent uses,
**So that** I can choose the right execution engine for each role.

**Acceptance Criteria:**

```gherkin
GIVEN I create a new agent
WHEN I specify config_json.backend = "hermes"
THEN the agent is configured to use Hermes backend

GIVEN I update an existing agent
WHEN I change config_json.backend from "claude" to "hermes"
THEN subsequent runs use Hermes backend

GIVEN I GET /api/v1/backends
THEN I receive a list of available backends with health and capabilities
```

**Technical Notes:**
- Migration: `0012_agent_backends.sql` — add `backend_type TEXT DEFAULT 'claude'` to agents
- New route: `GET /api/v1/backends`
- Agent create/update validates backend exists in registry

**Test Plan:**
- `test_create_agent_with_backend`
- `test_update_agent_backend`
- `test_list_backends_endpoint`
- `test_invalid_backend_returns_400`

---

### E3-S5: Backend Health Dashboard

**As a** user,
**I want** to see the health of all backends in the UI,
**So that** I know which execution engines are available.

**Acceptance Criteria:**

```gherkin
GIVEN I'm on the Settings page
WHEN I view the "Backends" section
THEN I see each registered backend with:
  - Name, health status (green/yellow/red)
  - Capabilities (streaming, tools, isolation)
  - Agent count using this backend

GIVEN a backend becomes unavailable
WHEN the health check runs
THEN the status updates to red
AND a BackendHealthChanged event appears in the event stream
```

**Test Plan:**
- E2E: backends section shows on settings
- E2E: health status badges update

---

### E3-S6: Backend Event Stream Normalization

**As a** system,
**I want** all backends to emit normalized ForgeEvents,
**So that** the UI and persistence work identically regardless of backend.

**Acceptance Criteria:**

```gherkin
GIVEN a Claude backend run produces ProcessOutput events
WHEN streamed to WebSocket
THEN the events have consistent format: { type, session_id, agent_id, data }

GIVEN a future Hermes backend run produces output
WHEN streamed to WebSocket
THEN the events have the SAME format as Claude backend events
AND the UI renders them identically

GIVEN the event stream
WHEN events from different backends appear
THEN each event includes metadata.backend_type for origin tracking
```

**Technical Notes:**
- Events already normalized through ForgeEvent enum
- Each backend adapter is responsible for mapping its output to ForgeEvent
- Add `backend_type` to event metadata

**Test Plan:**
- `test_claude_events_have_backend_metadata`
- `test_events_from_different_backends_have_same_shape`

---

## Story Point Estimates

| Story | Points | Sprint |
|-------|--------|--------|
| E3-S1 ProcessBackend Trait | 5 | S2 |
| E3-S2 Claude Backend Extract | 5 | S2 |
| E3-S3 Backend Routing | 3 | S2 |
| E3-S4 Agent Backend Config | 3 | S2 |
| E3-S5 Health Dashboard | 2 | S2 |
| E3-S6 Event Normalization | 3 | S2 |
| **Total** | **21** | |
