//! Application state shared across handlers.

use forge_core::EventBus;
use forge_db::{AgentRepo, EventRepo, SessionRepo, SkillRepo, WorkflowRepo};
use forge_safety::{CircuitBreaker, RateLimiter};
use std::sync::Arc;

/// Circuit breaker and rate limiter for run handler.
#[derive(Clone)]
pub struct SafetyState {
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub rate_limiter: Arc<RateLimiter>,
}

/// Optional budget limits (USD). When cost exceeds warn/limit, emit events.
#[derive(Clone, Default)]
pub struct BudgetConfig {
    pub warn: Option<f64>,
    pub limit: Option<f64>,
}

/// Shared state for the API: repositories, event bus, safety, and optional budget.
#[derive(Clone)]
pub struct AppState {
    pub agent_repo: Arc<AgentRepo>,
    pub session_repo: Arc<SessionRepo>,
    pub event_repo: Arc<EventRepo>,
    pub event_bus: Arc<EventBus>,
    pub skill_repo: Arc<SkillRepo>,
    pub workflow_repo: Arc<WorkflowRepo>,
    pub safety: SafetyState,
    pub budget: BudgetConfig,
}

impl AppState {
    pub fn new(
        agent_repo: Arc<AgentRepo>,
        session_repo: Arc<SessionRepo>,
        event_repo: Arc<EventRepo>,
        event_bus: Arc<EventBus>,
        skill_repo: Arc<SkillRepo>,
        workflow_repo: Arc<WorkflowRepo>,
        safety: SafetyState,
        budget: BudgetConfig,
    ) -> Self {
        Self {
            agent_repo,
            session_repo,
            event_repo,
            event_bus,
            skill_repo,
            workflow_repo,
            safety,
            budget,
        }
    }
}
