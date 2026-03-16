# Agent UW-A: DbPool → UnitOfWork Foundation

## Goal

Create a `UnitOfWork` struct in `forge-db` that wraps all 17 repos behind a single `Arc<DbPool>`, then slim `AppState` from 17 individual `Arc<XyzRepo>` fields down to `Arc<UnitOfWork>` + `Arc<EventBus>` + `SafetyState`.

## Context

Currently every repo is constructed independently from `conn_arc()` in main.rs, wrapped in `Arc`, and passed as a separate field to `AppState`. This creates:
- 17 `Arc<XyzRepo>` fields in `AppState`
- 18-parameter constructor (`#[allow(clippy::too_many_arguments)]`)
- Boilerplate in main.rs wiring

The `DbPool` (r2d2 read/write pools) already exists in `crates/forge-db/src/pool.rs`.

## Files to Create/Modify

### Create: `crates/forge-db/src/unit_of_work.rs`

```rust
use crate::{
    AgentRepo, SessionRepo, EventRepo, SkillRepo, WorkflowRepo,
    MemoryRepo, HookRepo, ScheduleRepo, AnalyticsRepo, CompactionRepo,
    CompanyRepo, DepartmentRepo, OrgPositionRepo, GoalRepo, ApprovalRepo,
    PersonaRepo, SafetyRepo, DbPool,
};
use std::sync::Arc;

/// Single entry point for all database access.
/// Holds the pool and lazily (or eagerly) provides repo views.
pub struct UnitOfWork {
    pool: Arc<DbPool>,
    // Repos — cheap to construct, each just holds Arc<Mutex<Connection>>
    pub agent_repo: AgentRepo,
    pub session_repo: SessionRepo,
    pub event_repo: EventRepo,
    pub skill_repo: SkillRepo,
    pub workflow_repo: WorkflowRepo,
    pub memory_repo: MemoryRepo,
    pub hook_repo: HookRepo,
    pub schedule_repo: ScheduleRepo,
    pub analytics_repo: AnalyticsRepo,
    pub compaction_repo: CompactionRepo,
    pub company_repo: CompanyRepo,
    pub department_repo: DepartmentRepo,
    pub org_position_repo: OrgPositionRepo,
    pub goal_repo: GoalRepo,
    pub approval_repo: ApprovalRepo,
    pub persona_repo: PersonaRepo,
    pub safety_repo: SafetyRepo,
}

impl UnitOfWork {
    pub fn new(pool: Arc<DbPool>) -> Self {
        let conn = pool.conn_arc();
        Self {
            agent_repo: AgentRepo::new(Arc::clone(&conn)),
            session_repo: SessionRepo::new(Arc::clone(&conn)),
            event_repo: EventRepo::new(Arc::clone(&conn)),
            skill_repo: SkillRepo::new(Arc::clone(&conn)),
            workflow_repo: WorkflowRepo::new(Arc::clone(&conn)),
            memory_repo: MemoryRepo::new(Arc::clone(&conn)),
            hook_repo: HookRepo::new(Arc::clone(&conn)),
            schedule_repo: ScheduleRepo::new(Arc::clone(&conn)),
            analytics_repo: AnalyticsRepo::new(Arc::clone(&conn)),
            compaction_repo: CompactionRepo::new(Arc::clone(&conn)),
            company_repo: CompanyRepo::new(Arc::clone(&conn)),
            department_repo: DepartmentRepo::new(Arc::clone(&conn)),
            org_position_repo: OrgPositionRepo::new(Arc::clone(&conn)),
            goal_repo: GoalRepo::new(Arc::clone(&conn)),
            approval_repo: ApprovalRepo::new(Arc::clone(&conn)),
            persona_repo: PersonaRepo::new(Arc::clone(&conn)),
            safety_repo: SafetyRepo::new(Arc::clone(&conn)),
            pool,
        }
    }

    /// Access the underlying pool (for BatchWriter, migrations, etc.)
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}
```

### Modify: `crates/forge-db/src/lib.rs`
- Add `pub mod unit_of_work;`
- Add `pub use unit_of_work::UnitOfWork;`

### Modify: `crates/forge-api/src/state.rs`

**Before:**
```rust
pub struct AppState {
    pub agent_repo: Arc<AgentRepo>,
    pub session_repo: Arc<SessionRepo>,
    // ... 15 more repos ...
    pub safety: SafetyState,
}
```

**After:**
```rust
pub struct AppState {
    pub uow: Arc<UnitOfWork>,
    pub event_bus: Arc<EventBus>,
    pub safety: SafetyState,
}

impl AppState {
    pub fn new(uow: Arc<UnitOfWork>, event_bus: Arc<EventBus>, safety: SafetyState) -> Self {
        Self { uow, event_bus, safety }
    }
}
```

### Modify: `crates/forge-app/src/main.rs`

Replace the 17-repo construction block with:
```rust
let db = Arc::new(db);
let uow = Arc::new(UnitOfWork::new(Arc::clone(&db)));

// Seed data uses uow.persona_repo, uow.company_repo, etc.

let state = AppState::new(Arc::clone(&uow), Arc::new(event_bus), safety);
```

BatchWriter still gets `db.conn_arc()` separately — it doesn't go through UnitOfWork.

## Do NOT modify
- Any route handler files (`crates/forge-api/src/routes/*.rs`) — Agent UW-B handles those
- `crates/forge-mcp-bin/` — Agent UW-C handles that
- Any test files — Agent UW-D handles those

## Verification
```bash
cargo check -p forge-db        # UnitOfWork compiles
cargo check -p forge-api       # AppState compiles (routes will break — expected)
```

Routes WILL have compile errors after this change (they reference `state.agent_repo` etc.). That's expected — UW-B fixes them.

## Zero Warnings Policy
`cargo check` on forge-db and forge-api crates individually must produce zero warnings for the files you touch.
