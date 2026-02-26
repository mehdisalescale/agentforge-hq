//! Application state shared across handlers.

use forge_core::EventBus;
use forge_db::{AgentRepo, EventRepo, SessionRepo, SkillRepo};
use std::sync::Arc;

/// Shared state for the API: repositories and event bus.
#[derive(Clone)]
pub struct AppState {
    pub agent_repo: Arc<AgentRepo>,
    pub session_repo: Arc<SessionRepo>,
    pub event_repo: Arc<EventRepo>,
    pub event_bus: Arc<EventBus>,
    pub skill_repo: Arc<SkillRepo>,
}

impl AppState {
    pub fn new(
        agent_repo: Arc<AgentRepo>,
        session_repo: Arc<SessionRepo>,
        event_repo: Arc<EventRepo>,
        event_bus: Arc<EventBus>,
        skill_repo: Arc<SkillRepo>,
    ) -> Self {
        Self {
            agent_repo,
            session_repo,
            event_repo,
            event_bus,
            skill_repo,
        }
    }
}
