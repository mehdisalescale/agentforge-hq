# Absorption Pipeline

> The systematic process for absorbing each of the 62 reference repositories' features into Claude Forge.

---

## Overview

Claude Forge's unique challenge is absorbing ~200K+ lines of code worth of patterns from 62 repositories written in TypeScript, Python, Bash, Swift, Lua, Java, and Emacs Lisp into a single Rust + Svelte codebase. This is not a port. We do not translate line by line. We absorb the **interface** and **behavior**, then implement idiomatically in Rust.

The Absorption Pipeline is a 5-phase process that transforms a reference repository's patterns into production Forge features.

```
 ANALYZE       EXTRACT       DESIGN        IMPLEMENT     VALIDATE
 ─────────    ─────────    ─────────     ──────────    ──────────
 Read the     Extract      Map to Forge   Write Rust    Verify
 reference    interfaces,  contexts,      + Svelte +    behavior,
 map entry    behaviors,   traits, DB,    MCP + tests   interfaces,
              data flows   API, MCP                     docs
```

---

## Phase 1: ANALYZE

**Goal**: Understand what the reference repo does, what is worth absorbing, and how it classifies.

### Steps

1. **Read the reference-map entry** (`reference-map/<category>/<repo>.md`)
   - Purpose and core value
   - Key features
   - Tech stack
   - Adoptable patterns table

2. **Identify adoptable patterns and features**
   - Which patterns solve problems Forge has?
   - Which patterns serve > 5% of users? (below that threshold, consider plugin)
   - Which patterns are unique to this repo vs. duplicated across others?

3. **Classify each pattern**

   | Classification | Description | Examples |
   |---------------|-------------|----------|
   | **Data** | Presets, skills, configs, templates -- no logic, just content | Agent definitions, skill catalogs, prompt presets, config templates |
   | **Logic** | Algorithms, state machines, pipelines -- behavioral patterns | Circuit breaker FSM, DAG executor, rate limiter, FTS indexer |
   | **UI** | Component designs, layouts, visualizations | Swim-lane view, Kanban board, diff viewer, terminal panel |

4. **Assess effort**

   | Size | Effort | Description |
   |------|--------|-------------|
   | **S** | 1-2 days | Data import, simple config, single component |
   | **M** | 3-5 days | New API endpoint + UI + tests |
   | **L** | 1-2 weeks | New bounded context, trait hierarchy, multi-component UI |
   | **XL** | 2-4 weeks | Major subsystem (workflow engine, observability dashboard) |

5. **Check for dependencies**: Does this pattern require another Forge module that does not exist yet?

### Output

An analysis document (can be informal -- a GitHub Issue or a section in a tracking doc) that answers:
- What are we absorbing? (specific patterns, not "everything")
- How does it classify? (Data / Logic / UI)
- What size is it? (S / M / L / XL)
- What does it depend on?
- What bounded context does it belong to?

---

## Phase 2: EXTRACT

**Goal**: Extract the INTERFACE of each pattern -- inputs, outputs, behaviors -- without copying the implementation.

### Steps

1. **Extract the interface contract**
   - What inputs does this pattern accept?
   - What outputs does it produce?
   - What are the edge cases and error conditions?
   - What are the performance characteristics? (latency, throughput)

2. **Document data structures**
   - What are the core types?
   - What are the relationships between types?
   - What is stored persistently vs. ephemerally?

3. **Document event flows**
   - What events does this pattern emit?
   - What events does it consume?
   - What is the lifecycle? (create -> running -> paused -> completed -> archived)

4. **Identify integration points**
   - Which other Forge modules need to know about this?
   - What API surface does it expose? (HTTP, MCP, internal)
   - What database tables does it read/write?

### Extraction Rules

- **DO** extract: interfaces, behaviors, data shapes, event names, error conditions
- **DO NOT** extract: implementation details specific to the source language
- **DO NOT** copy: code, comments, variable names, or architecture decisions that are artifacts of the source stack (e.g., JavaScript Promises, Python asyncio patterns, React component trees)
- **ALWAYS** note: where the reference implementation has bugs, limitations, or design mistakes we should avoid

### Output

An extraction document listing:
- Interface contracts (function signatures in pseudocode)
- Data structure definitions
- Event flow diagrams (text-based is fine)
- Integration point list

---

## Phase 3: DESIGN

**Goal**: Map the extracted interface to Forge's architecture.

### Steps

1. **Map to bounded context**
   - Which of Forge's bounded contexts owns this feature?
   - Agent Management, Session Lifecycle, Workflow Engine, Observability, Skill Marketplace, Git Operations, Safety and Governance, Configuration, Remote Control

2. **Define Rust traits and types**

   ```rust
   // Example: Circuit breaker from ralph-claude-code

   #[derive(Debug, Clone, Copy, PartialEq)]
   pub enum CircuitState {
       Closed,    // Normal operation
       Open,      // Tripped, rejecting requests
       HalfOpen,  // Testing recovery
   }

   pub trait CircuitBreaker: Send + Sync {
       fn check(&self) -> Result<(), CircuitBreakerError>;
       fn record_success(&self);
       fn record_failure(&self);
       fn state(&self) -> CircuitState;
       fn reset(&self);
   }
   ```

3. **Define database schema additions**

   ```sql
   -- Example: Circuit breaker state persistence
   CREATE TABLE IF NOT EXISTS circuit_breaker_state (
       agent_id TEXT PRIMARY KEY,
       state TEXT NOT NULL DEFAULT 'closed',
       failure_count INTEGER NOT NULL DEFAULT 0,
       last_failure_at TEXT,
       last_success_at TEXT,
       opened_at TEXT
   );
   ```

4. **Define API endpoints**

   ```
   GET  /api/agents/{id}/circuit-breaker     -> CircuitBreakerState
   POST /api/agents/{id}/circuit-breaker/reset -> ()
   ```

5. **Define MCP tools/resources**

   ```json
   {
     "name": "forge_circuit_breaker_status",
     "description": "Get the circuit breaker state for an agent",
     "inputSchema": {
       "type": "object",
       "properties": {
         "agent_id": { "type": "string" }
       },
       "required": ["agent_id"]
     }
   }
   ```

6. **Write ADR if needed**
   - Is this a new pattern Forge has not used before?
   - Does it change an existing interface?
   - Are there multiple viable approaches?
   - If yes to any: write an ADR

### Output

A design document (or PR description) with:
- Bounded context assignment
- Rust trait/type definitions
- SQL schema additions
- API endpoint definitions
- MCP tool/resource definitions
- ADR reference (if applicable)

---

## Phase 4: IMPLEMENT

**Goal**: Write production code.

### Steps

1. **Write Rust code in the appropriate module**
   - Follow existing patterns in the codebase
   - Use `DashMap` for concurrent state (established pattern)
   - Use `rusqlite` with WAL mode for persistence (established pattern)
   - Use `tokio::sync::broadcast` for event streaming (established pattern)

2. **Add API routes**
   - Register in the Axum router (in `main.rs` or the appropriate module's route function)
   - Use extractors for path params, query params, JSON body
   - Return appropriate HTTP status codes
   - Add error handling with `thiserror` types

3. **Add MCP tool/resource**
   - Implement the tool handler
   - Register in the MCP server
   - Include JSON Schema for input validation

4. **Add Svelte UI component**
   - Create in the appropriate route or component directory
   - Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
   - Support dark/light theme
   - Add keyboard shortcuts where applicable
   - Use TailwindCSS 4 for styling

5. **Write tests**
   - Unit tests for Rust logic (`#[cfg(test)]` modules)
   - Integration tests for API endpoints (in `tests/` directory)
   - Type-check Svelte components

### Implementation Order

For a given feature, implement in this order:
1. Database schema (if any)
2. Rust types and traits
3. Core logic (with unit tests)
4. API endpoints (with integration tests)
5. MCP tools (with tests)
6. Svelte UI components

This order ensures each layer has a stable foundation beneath it.

### Output

A PR containing:
- Implementation code
- Tests (unit + integration)
- Database migration (if schema changed)
- Updated type definitions in frontend

---

## Phase 5: VALIDATE

**Goal**: Confirm the implementation matches the reference behavior and meets quality standards.

### Validation Checklist

- [ ] **Behavioral match**: Does the Forge implementation match the reference behavior? (Test with the same inputs, compare outputs)
- [ ] **HTTP interface works**: API endpoint returns correct responses for happy path and error cases
- [ ] **MCP interface works**: MCP tool can be invoked by an MCP client and returns correct results
- [ ] **CLI integration works**: Feature is accessible from the command line (if applicable)
- [ ] **Tests pass**: All new tests pass, no existing tests broken
- [ ] **Documentation updated**: API docs, MCP docs, user docs reflect the new feature
- [ ] **Feature source map updated**: `FEATURE_SOURCE_MAP.md` entry added/updated
- [ ] **Performance acceptable**: No regression beyond 10% on hot paths

### Deviation Documentation

If the Forge implementation intentionally deviates from the reference behavior, document:
- What is different
- Why it is different
- Why the Forge approach is better (or at least not worse)

---

## Checklist Template

Copy this checklist for each absorption task:

```markdown
## Absorption: [Feature Name] from [Repo Name]

### Phase 1: ANALYZE
- [ ] Read reference-map entry
- [ ] Identify adoptable patterns: ___
- [ ] Classify: Data / Logic / UI
- [ ] Estimate size: S / M / L / XL
- [ ] Identify dependencies: ___
- [ ] Target bounded context: ___

### Phase 2: EXTRACT
- [ ] Interface contract documented
- [ ] Data structures documented
- [ ] Event flows documented
- [ ] Integration points listed

### Phase 3: DESIGN
- [ ] Rust traits and types defined
- [ ] Database schema additions defined
- [ ] API endpoints defined
- [ ] MCP tools/resources defined
- [ ] ADR written (if needed): ADR-___

### Phase 4: IMPLEMENT
- [ ] Rust code written
- [ ] API routes added
- [ ] MCP tools added
- [ ] Svelte UI added
- [ ] Unit tests written
- [ ] Integration tests written

### Phase 5: VALIDATE
- [ ] Behavioral match confirmed
- [ ] HTTP interface tested
- [ ] MCP interface tested
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Feature source map updated
- [ ] Performance verified
```

---

## Example Walkthrough: Absorbing Circuit Breaker from ralph-claude-code

### Phase 1: ANALYZE

**Source**: `reference-map/02-orchestration-workflows/ralph-claude-code.md`

**Adoptable pattern**: 3-state circuit breaker (CLOSED / OPEN / HALF_OPEN) with auto-recovery

**Classification**: Logic (state machine algorithm)

**Size**: M (3-5 days) -- well-defined state machine, clear interface, moderate UI component

**Dependencies**: Agent Management context (needs agent ID), Event streaming (needs to emit state change events)

**Target bounded context**: Safety and Governance

### Phase 2: EXTRACT

**Interface contract**:
```
check(agent_id) -> Ok | Err(CircuitOpen { retry_after })
record_success(agent_id) -> ()
record_failure(agent_id) -> ()
state(agent_id) -> { state: Closed|Open|HalfOpen, failure_count, last_failure }
reset(agent_id) -> ()
```

**Data structures**:
```
CircuitState: enum { Closed, Open, HalfOpen }
CircuitConfig: { failure_threshold: u32, recovery_timeout: Duration, half_open_max_calls: u32 }
CircuitSnapshot: { state, failure_count, last_failure_at, opened_at, config }
```

**Event flow**:
```
Agent runs -> check() -> Closed: allow
                      -> Open: reject, emit CircuitOpen event
                      -> HalfOpen: allow limited

Agent call succeeds -> record_success() -> if HalfOpen: transition to Closed, emit CircuitClosed event
Agent call fails -> record_failure() -> if failures >= threshold: transition to Open, emit CircuitTripped event

Timer expires -> transition Open to HalfOpen, emit CircuitHalfOpen event
```

**Integration points**:
- Agent runner (calls `check()` before each agent invocation)
- Event stream (emits circuit state changes)
- Database (persists state across restarts)
- Dashboard (displays circuit status per agent)

### Phase 3: DESIGN

**Rust types**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CircuitState { Closed, Open, HalfOpen }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitConfig {
    pub failure_threshold: u32,       // default: 5
    pub recovery_timeout_secs: u64,   // default: 60
    pub half_open_max_calls: u32,     // default: 1
}

pub struct CircuitBreaker {
    states: DashMap<String, CircuitSnapshot>,
    config: CircuitConfig,
    event_tx: broadcast::Sender<ForgeEvent>,
}
```

**Database schema**:
```sql
ALTER TABLE agents ADD COLUMN circuit_state TEXT NOT NULL DEFAULT 'closed';
ALTER TABLE agents ADD COLUMN circuit_failure_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE agents ADD COLUMN circuit_last_failure TEXT;
ALTER TABLE agents ADD COLUMN circuit_opened_at TEXT;
```

**API endpoints**:
```
GET  /api/agents/{id}/circuit-breaker     -> { state, failure_count, last_failure, config }
POST /api/agents/{id}/circuit-breaker/reset -> 204 No Content
PUT  /api/agents/{id}/circuit-breaker/config -> { config }
```

**MCP tools**:
```json
{
  "name": "forge_get_circuit_breaker",
  "description": "Get circuit breaker state for an agent",
  "inputSchema": { "properties": { "agent_id": { "type": "string" } }, "required": ["agent_id"] }
}
{
  "name": "forge_reset_circuit_breaker",
  "description": "Reset circuit breaker to closed state",
  "inputSchema": { "properties": { "agent_id": { "type": "string" } }, "required": ["agent_id"] }
}
```

**ADR**: ADR-0012: Circuit Breaker for Agent Execution Safety

### Phase 4: IMPLEMENT

**Day 1**: Define types, implement `CircuitBreaker` struct with unit tests for all state transitions

**Day 2**: Add API routes, integration test the endpoints, add DB persistence

**Day 3**: Add MCP tools, implement UI status indicator on agent cards

**Day 4**: Integration testing, edge cases (restart recovery, concurrent access), documentation

### Phase 5: VALIDATE

- Behavioral match: Verify same state transitions as ralph-claude-code for identical failure sequences
- HTTP: `curl` test all 3 endpoints
- MCP: Invoke tools via MCP client, verify responses
- Tests: 15+ unit tests, 6+ integration tests
- Docs: API reference updated, MCP tool reference updated, user guide section added
- Feature source map: Entry added linking circuit breaker to ralph-claude-code

---

## Timeline Estimates by Feature Size

| Size | Analysis | Extraction | Design | Implementation | Validation | Total |
|------|----------|-----------|--------|----------------|------------|-------|
| **S** (Data import) | 1 hour | 1 hour | 1 hour | 4-8 hours | 2 hours | 1-2 days |
| **M** (Single feature) | 2 hours | 3 hours | 4 hours | 2-3 days | 4 hours | 3-5 days |
| **L** (New context) | 4 hours | 1 day | 1 day | 5-8 days | 1 day | 1.5-2 weeks |
| **XL** (Subsystem) | 1 day | 2 days | 2 days | 2-3 weeks | 2 days | 3-4 weeks |

### Size Examples from Reference Repos

| Size | Example Absorption | Source Repo |
|------|--------------------|-------------|
| **S** | Import 100+ agent presets as JSON | claude-code-subagents |
| **S** | Import 69 prompt presets | claude-code-skills |
| **S** | Import SKILL.md format parser | claude-code-plugins-plus-skills |
| **M** | Circuit breaker state machine | ralph-claude-code |
| **M** | Cron scheduler for prompts | claude-code-viewer |
| **M** | Todo extraction from sessions | claude-code-viewer |
| **M** | Session-to-HTML export | claude-code-transcripts |
| **L** | Swim-lane observability dashboard | hooks-observability |
| **L** | FTS5 fuzzy session search | claude-code-viewer |
| **L** | Git integration panel | 1code |
| **XL** | DAG workflow engine | Claude-Code-Workflow |
| **XL** | Skill marketplace with auto-activation | claude-code-plugins-plus-skills |
| **XL** | Multi-provider proxy pipeline | claude-code-hub |

---

## Absorption Priority Framework

Not all 62 repos are equally valuable. Prioritize by:

1. **Impact** (1-5): How many users benefit from this pattern?
2. **Uniqueness** (1-5): Is this the only repo with this pattern, or is it common?
3. **Effort** (1-5, inverted): How much work to absorb? (5 = easy, 1 = very hard)
4. **Dependencies** (0-3): How many unimplemented Forge modules does this require? (0 = none)

**Priority Score** = Impact + Uniqueness + Effort - Dependencies

Absorb in descending priority score order within each sprint.

### Priority Tiers

| Tier | Score | Action |
|------|-------|--------|
| **P0: Absorb Now** | 12-15 | Schedule in current or next sprint |
| **P1: Absorb Soon** | 8-11 | Schedule within next 3 sprints |
| **P2: Absorb Later** | 5-7 | Backlog, schedule when dependencies are met |
| **P3: Consider for Plugin** | 1-4 | May be better as a community plugin than core |

---

## Batch Absorption Strategy

Many repos contribute overlapping features. Instead of absorbing one repo at a time, batch by bounded context:

| Batch | Bounded Context | Repos Involved | Estimated Duration |
|-------|----------------|----------------|-------------------|
| 1 | Agent Management | ralph-claude-code, claude-code-subagents, claude-code-sub-agents, claude-code-agents | 2 sprints |
| 2 | Session Lifecycle | claude-code-viewer, claude-code-transcripts, claude-code-tools | 2 sprints |
| 3 | Workflow Engine | Claude-Code-Workflow, claude-code-spec-workflow, claude-code-workflows | 3 sprints |
| 4 | Observability | hooks-observability, hooks-mastery, usage-monitor | 2 sprints |
| 5 | Skill Marketplace | plugins-plus-skills, claude-code-skills, claude-code-templates, skill-factory | 2 sprints |
| 6 | Safety and Governance | ralph-claude-code, security-review, claude-code-config | 2 sprints |
| 7 | Git Operations | 1code, codemcp, claude-code-viewer | 1 sprint |
| 8 | Configuration | claude-code-settings, claude-code-config, claude-code-config2, everything-claude-code | 1 sprint |
| 9 | Remote Control | claude-code-hub, claude-code-remote, claude-code-telegram, claude-code-proxy | 2 sprints |
| 10 | MCP Foundation | claude-code-mcp, claude-code-tools, claude-code-ide.el | 1 sprint |

Total: ~18 sprints (~9 months) for comprehensive absorption, with the most impactful features landing in the first 6 sprints (~3 months).
