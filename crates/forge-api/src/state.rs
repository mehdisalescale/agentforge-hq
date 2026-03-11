//! Application state shared across handlers.

use forge_core::EventBus;
use forge_db::{
    AgentRepo, AnalyticsRepo, ApprovalRepo, CompanyRepo, CompactionRepo, DepartmentRepo, EventRepo,
    GoalRepo, HookRepo, MemoryRepo, OrgPositionRepo, ScheduleRepo, SessionRepo, SkillRepo,
    WorkflowRepo,
};
use forge_safety::{CircuitBreaker, CostTracker, RateLimiter};
use std::sync::Arc;

/// Circuit breaker and rate limiter for run handler.
#[derive(Clone)]
pub struct SafetyState {
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub rate_limiter: Arc<RateLimiter>,
    pub cost_tracker: Arc<CostTracker>,
}

/// Shared state for the API: repositories, event bus, safety.
#[derive(Clone)]
pub struct AppState {
    pub agent_repo: Arc<AgentRepo>,
    pub session_repo: Arc<SessionRepo>,
    pub event_repo: Arc<EventRepo>,
    pub event_bus: Arc<EventBus>,
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
    pub safety: SafetyState,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agent_repo: Arc<AgentRepo>,
        session_repo: Arc<SessionRepo>,
        event_repo: Arc<EventRepo>,
        event_bus: Arc<EventBus>,
        skill_repo: Arc<SkillRepo>,
        workflow_repo: Arc<WorkflowRepo>,
        memory_repo: Arc<MemoryRepo>,
        hook_repo: Arc<HookRepo>,
        schedule_repo: Arc<ScheduleRepo>,
        analytics_repo: Arc<AnalyticsRepo>,
        compaction_repo: Arc<CompactionRepo>,
        company_repo: Arc<CompanyRepo>,
        department_repo: Arc<DepartmentRepo>,
        org_position_repo: Arc<OrgPositionRepo>,
        goal_repo: Arc<GoalRepo>,
        approval_repo: Arc<ApprovalRepo>,
        safety: SafetyState,
    ) -> Self {
        Self {
            agent_repo,
            session_repo,
            event_repo,
            event_bus,
            skill_repo,
            workflow_repo,
            memory_repo,
            hook_repo,
            schedule_repo,
            analytics_repo,
            compaction_repo,
            company_repo,
            department_repo,
            org_position_repo,
            goal_repo,
            approval_repo,
            safety,
        }
    }
}
