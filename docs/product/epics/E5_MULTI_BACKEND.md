# Epic E5: Multi-Backend Execution

> **Hermes-Agent and OpenClaw as pluggable execution backends.**
>
> Depends on: E3 (Hexagonal Backend Architecture)

---

## Business Value

With the ProcessBackend trait from E3, we can now add real backends. Hermes brings 40+ tools, self-improving memory, and 6 terminal backends. OpenClaw brings webhook-based Docker-sandboxed execution. Users choose the right engine for each agent role.

## Acceptance Gate

1. HermesBackend spawns Hermes process, streams events to Forge UI
2. OpenClawBackend sends/receives webhooks, maps results to ForgeEvents
3. Memory sync between Hermes MEMORY.md and Forge memory table works bidirectionally
4. Agent backend switching works at runtime
5. Backend failures trigger circuit breaker (graceful degradation)
6. 25+ tests

---

## User Stories

### E5-S1: Hermes Backend Adapter
**As a** user, **I want** agents to execute via Hermes with 40+ tools.

```gherkin
GIVEN FORGE_HERMES_COMMAND points to a valid hermes installation
WHEN HermesBackend.health_check() is called
THEN it returns Healthy with hermes version

GIVEN a run is initiated with backend="hermes"
WHEN HermesBackend.spawn() is called
THEN it executes: hermes chat --non-interactive --model <model> --session-id <id>
AND stdout is parsed into ForgeEvents
AND ProcessOutput events stream to WebSocket
```

Tests: `test_hermes_spawn`, `test_hermes_health`, `test_hermes_output_parsing`

### E5-S2: Hermes Memory Sync
**As a** system, **I want** Hermes MEMORY.md ↔ Forge memory bidirectional sync.

```gherkin
GIVEN an agent run starts via Hermes
WHEN MemorySync.export_to_hermes(agent_id) is called
THEN Forge memory entries are written to a temp MEMORY.md for Hermes

GIVEN a Hermes run completes
WHEN MemorySync.import_from_hermes(session_id) is called
THEN new facts in Hermes MEMORY.md are imported to Forge memory table
AND existing facts are not duplicated (content-hash dedup)
```

Tests: `test_export_memories`, `test_import_new_facts`, `test_dedup_on_import`

### E5-S3: Hermes Tool Filtering
**As a** user, **I want** to restrict which Hermes toolsets an agent can use.

```gherkin
GIVEN an agent config has hermes_config.enabled_toolsets = ["web", "files"]
WHEN the Hermes process is spawned
THEN only web and file tools are available (terminal, browser disabled)
```

Tests: `test_toolset_filtering_passed_to_hermes`

### E5-S4: OpenClaw Webhook Adapter
**As a** user, **I want** agents to execute via OpenClaw for Docker-sandboxed workloads.

```gherkin
GIVEN FORGE_OPENCLAW_URL points to an OpenClaw gateway
WHEN OpenClawBackend.spawn() is called
THEN it POSTs a wakeup payload to OpenClaw webhook endpoint
AND registers a callback URL for results

GIVEN OpenClaw completes the task
WHEN it POSTs to the callback URL
THEN the result is parsed into ForgeEvents
AND a ProcessCompleted event is emitted
```

Tests: `test_openclaw_webhook_send`, `test_openclaw_callback_parsing`, `test_openclaw_timeout`

### E5-S5: OpenClaw Callback Endpoint
**As a** system, **I want** a webhook callback route for OpenClaw results.

```gherkin
WHEN OpenClaw POSTs to /api/v1/webhooks/openclaw
THEN the payload is validated (auth token, session_id)
AND the result is mapped to ForgeEvents
AND the session status is updated
```

Tests: `test_callback_valid_payload`, `test_callback_invalid_token_rejected`

### E5-S6: Backend Failover
**As a** system, **I want** backend failures handled gracefully.

```gherkin
GIVEN Hermes process crashes mid-execution
WHEN the crash is detected
THEN a ProcessFailed event is emitted
AND the circuit breaker records a failure
AND the session status is set to "failed"

GIVEN the Hermes circuit breaker is open
WHEN a new run is attempted with backend="hermes"
THEN the middleware rejects with CircuitOpen error
AND the error message suggests using "claude" backend instead
```

Tests: `test_crash_triggers_circuit_breaker`, `test_open_circuit_rejects`, `test_recovery_after_timeout`

### E5-S7–S10: Frontend & Config
- Backend selector on agent create/edit
- Hermes config panel (model, toolsets, terminal backend)
- OpenClaw config panel (gateway URL, auth)
- Session view shows backend badge

---

## Story Point Estimates

| Story | Points | Sprint |
|-------|--------|--------|
| E5-S1 Hermes Adapter | 8 | S4 |
| E5-S2 Memory Sync | 5 | S4 |
| E5-S3 Tool Filtering | 3 | S4 |
| E5-S4 OpenClaw Adapter | 5 | S5 |
| E5-S5 Callback Endpoint | 3 | S5 |
| E5-S6 Backend Failover | 5 | S5 |
| E5-S7-10 Frontend | 8 | S5 |
| **Total** | **37** | |
