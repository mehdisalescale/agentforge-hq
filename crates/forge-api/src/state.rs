//! Application state shared across handlers.

use forge_core::EventBus;
use forge_db::{AgentRepo, EventRepo, SessionRepo, SkillRepo, WorkflowRepo};
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
    pub safety: SafetyState,
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
    ) -> Self {
        Self {
            agent_repo,
            session_repo,
            event_repo,
            event_bus,
            skill_repo,
            workflow_repo,
            safety,
        }
    }
}
