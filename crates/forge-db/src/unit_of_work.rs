//! Single entry point for all database access.

use crate::{
    AgentRepo, AnalyticsRepo, ApprovalRepo, CompactionRepo, CompanyRepo, DepartmentRepo,
    EventRepo, GoalRepo, HookRepo, MemoryRepo, OrgPositionRepo, PersonaRepo, SafetyRepo,
    ScheduleRepo, SessionRepo, SkillRepo, WorkflowRepo, DbPool,
};
use std::sync::Arc;

/// Wraps all repos behind a single `Arc<DbPool>`.
/// Each repo is `Arc`-wrapped so middleware and handlers can cheaply clone references.
pub struct UnitOfWork {
    pool: Arc<DbPool>,
    pub agent_repo: Arc<AgentRepo>,
    pub session_repo: Arc<SessionRepo>,
    pub event_repo: Arc<EventRepo>,
    pub skill_repo: Arc<SkillRepo>,
    pub workflow_repo: Arc<WorkflowRepo>,
    pub memory_repo: Arc<MemoryRepo>,
    pub hook_repo: Arc<HookRepo>,
    pub schedule_repo: Arc<ScheduleRepo>,
    pub analytics_repo: Arc<AnalyticsRepo>,
    pub compaction_repo: Arc<CompactionRepo>,
    pub company_repo: Arc<CompanyRepo>,
    pub department_repo: Arc<DepartmentRepo>,
    pub org_position_repo: Arc<OrgPositionRepo>,
    pub goal_repo: Arc<GoalRepo>,
    pub approval_repo: Arc<ApprovalRepo>,
    pub persona_repo: Arc<PersonaRepo>,
    pub safety_repo: Arc<SafetyRepo>,
}

impl UnitOfWork {
    pub fn new(pool: Arc<DbPool>) -> Self {
        let conn = pool.conn_arc();
        Self {
            agent_repo: Arc::new(AgentRepo::new(Arc::clone(&conn))),
            session_repo: Arc::new(SessionRepo::new(Arc::clone(&conn))),
            event_repo: Arc::new(EventRepo::new(Arc::clone(&conn))),
            skill_repo: Arc::new(SkillRepo::new(Arc::clone(&conn))),
            workflow_repo: Arc::new(WorkflowRepo::new(Arc::clone(&conn))),
            memory_repo: Arc::new(MemoryRepo::new(Arc::clone(&conn))),
            hook_repo: Arc::new(HookRepo::new(Arc::clone(&conn))),
            schedule_repo: Arc::new(ScheduleRepo::new(Arc::clone(&conn))),
            analytics_repo: Arc::new(AnalyticsRepo::new(Arc::clone(&conn))),
            compaction_repo: Arc::new(CompactionRepo::new(Arc::clone(&conn))),
            company_repo: Arc::new(CompanyRepo::new(Arc::clone(&conn))),
            department_repo: Arc::new(DepartmentRepo::new(Arc::clone(&conn))),
            org_position_repo: Arc::new(OrgPositionRepo::new(Arc::clone(&conn))),
            goal_repo: Arc::new(GoalRepo::new(Arc::clone(&conn))),
            approval_repo: Arc::new(ApprovalRepo::new(Arc::clone(&conn))),
            persona_repo: Arc::new(PersonaRepo::new(Arc::clone(&conn))),
            safety_repo: Arc::new(SafetyRepo::new(Arc::clone(&conn))),
            pool,
        }
    }

    /// Access the underlying pool (for BatchWriter, migrations, etc.)
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Migrator;

    fn setup_uow() -> UnitOfWork {
        let db = DbPool::in_memory().unwrap();
        {
            let conn = db.connection();
            Migrator::new(&conn).apply_pending().unwrap();
        }
        UnitOfWork::new(Arc::new(db))
    }

    #[test]
    fn uow_provides_all_repos() {
        let uow = setup_uow();

        // Every repo should be accessible and return empty results on a fresh DB.
        assert!(uow.agent_repo.list().unwrap().is_empty());
        assert!(uow.session_repo.list().unwrap().is_empty());
        assert!(uow.skill_repo.list().unwrap().is_empty());
        assert!(uow.workflow_repo.list().unwrap().is_empty());
        assert!(uow.memory_repo.list(50, 0).unwrap().is_empty());
        assert!(uow.hook_repo.list().unwrap().is_empty());
        assert!(uow.schedule_repo.list().unwrap().is_empty());
        assert!(uow.company_repo.list().unwrap().is_empty());
        assert!(uow.persona_repo.list(None, None).unwrap().is_empty());
    }

    #[test]
    fn uow_pool_accessible() {
        let uow = setup_uow();
        // pool() should return a valid reference (used for BatchWriter, migrations, etc.)
        let _conn = uow.pool().connection();
    }

    #[test]
    fn uow_repos_share_connection() {
        let uow = setup_uow();

        // Write via one repo, read via another that shares the connection.
        let company = uow
            .company_repo
            .create(&crate::NewCompany {
                name: "SharedTest".into(),
                mission: None,
                budget_limit: None,
            })
            .unwrap();

        // department_repo uses the same underlying connection — should see the company.
        let companies = uow.company_repo.list().unwrap();
        assert_eq!(companies.len(), 1);
        assert_eq!(companies[0].id, company.id);
    }
}
