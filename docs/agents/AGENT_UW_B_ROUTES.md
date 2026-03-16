# Agent UW-B: Update All Route Handlers for UnitOfWork

## Goal

Update all 16 route files in `crates/forge-api/src/routes/` to access repos via `state.uow.xxx_repo` instead of `state.xxx_repo`, and event bus via `state.event_bus` (unchanged name).

## Context

After Agent UW-A runs, `AppState` changes from:
```rust
pub struct AppState {
    pub agent_repo: Arc<AgentRepo>,      // was Arc, now in uow
    pub session_repo: Arc<SessionRepo>,  // was Arc, now in uow
    // ... 15 more ...
    pub event_bus: Arc<EventBus>,        // stays at top level
    pub safety: SafetyState,             // stays at top level
}
```
To:
```rust
pub struct AppState {
    pub uow: Arc<UnitOfWork>,
    pub event_bus: Arc<EventBus>,
    pub safety: SafetyState,
}
```

## Mechanical Transformation

For every route handler, the change is a simple prefix addition:

| Before | After |
|--------|-------|
| `state.agent_repo.list()` | `state.uow.agent_repo.list()` |
| `state.session_repo.get(...)` | `state.uow.session_repo.get(...)` |
| `state.event_bus.emit(...)` | `state.event_bus.emit(...)` ← **unchanged** |
| `state.safety.circuit_breaker` | `state.safety.circuit_breaker` ← **unchanged** |

## Files to Modify (all in `crates/forge-api/src/routes/`)

| File | Repos to prefix with `uow.` |
|------|------------------------------|
| `agents.rs` | `agent_repo`, `session_repo` |
| `sessions.rs` | `session_repo`, `agent_repo`, `event_repo` |
| `org.rs` | `company_repo`, `department_repo`, `org_position_repo` |
| `governance.rs` | `goal_repo`, `approval_repo` |
| `personas.rs` | `persona_repo`, `agent_repo`, `org_position_repo` |
| `skills.rs` | `skill_repo` |
| `workflows.rs` | `workflow_repo` |
| `memory.rs` | `memory_repo` |
| `analytics.rs` | `analytics_repo`, `org_position_repo` |
| `hooks.rs` | `hook_repo` |
| `schedules.rs` | `schedule_repo` |
| `run.rs` | `agent_repo`, `session_repo` + middleware construction uses repos |
| `ws.rs` | (none — uses `event_bus` which stays top-level) |
| `health.rs` | (none) |
| `settings.rs` | (none) |

### Special case: `run.rs` middleware construction

The middleware chain in `run.rs` constructs middleware instances with repo references. Update these:
```rust
// Before
GovernanceMiddleware::new(state.company_repo.clone(), ...)
// After
GovernanceMiddleware::new(Arc::new(state.uow.company_repo.clone()), ...)
```

Wait — check if middleware constructors take `Arc<XyzRepo>` or `&XyzRepo`. If they take `Arc`, you may need to wrap. If repo fields in UnitOfWork are not `Arc`, you'll need to adjust. Look at middleware.rs constructors to determine.

**Likely approach:** Since repos in UnitOfWork are plain structs (not `Arc`), and middleware wants `Arc<XyzRepo>` or references, you may need to either:
1. Change middleware to accept `&XyzRepo` (borrow from uow)
2. Or clone the repo and wrap in Arc

Option 1 is cleaner but may require lifetime changes. Option 2 is mechanical. Choose based on what compiles.

### Also modify: `crates/forge-api/src/middleware.rs`

If middleware structs store `Arc<XyzRepo>`, update their constructors to accept references or clones from UnitOfWork. The SpawnMiddleware, GovernanceMiddleware, PersistMiddleware, AgentConfigurator, and SecurityScanMiddleware are the ones that hold repo references.

## Depends On
- **Agent UW-A must complete first** — AppState won't have `uow` field until then

## Verification
```bash
cargo check -p forge-api   # all routes compile
cargo test -p forge-api    # existing tests pass
```

## Zero Warnings Policy
All modified files must produce zero warnings under `cargo check`.
