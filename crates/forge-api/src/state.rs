//! Application state shared across handlers.

use forge_core::EventBus;
use forge_db::{AgentRepo, EventRepo, SessionRepo, SkillRepo, WorkflowRepo};
use forge_safety::CircuitBreaker;
use std::sync::Arc;

/// Shared state for the API: repositories, event bus, and circuit breaker.
#[derive(Clone)]
pub struct AppState {
    pub agent_repo: Arc<AgentRepo>,
    pub session_repo: Arc<SessionRepo>,
    pub event_repo: Arc<EventRepo>,
    pub event_bus: Arc<EventBus>,
    pub skill_repo: Arc<SkillRepo>,
    pub workflow_repo: Arc<WorkflowRepo>,
    pub circuit_breaker: Arc<CircuitBreaker>,
}

impl AppState {
    pub fn new(
        agent_repo: Arc<AgentRepo>,
        session_repo: Arc<SessionRepo>,
        event_repo: Arc<EventRepo>,
        event_bus: Arc<EventBus>,
        skill_repo: Arc<SkillRepo>,
        workflow_repo: Arc<WorkflowRepo>,
        circuit_breaker: Arc<CircuitBreaker>,
    ) -> Self {
        Self {
            agent_repo,
            session_repo,
            event_repo,
            event_bus,
            skill_repo,
            workflow_repo,
            circuit_breaker,
        }
    }
}
