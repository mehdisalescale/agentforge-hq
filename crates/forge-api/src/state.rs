//! Application state shared across handlers.

use forge_core::EventBus;
use forge_db::UnitOfWork;
use forge_process::BackendRegistry;
use forge_safety::{CircuitBreaker, CostTracker, RateLimiter};
use std::sync::Arc;

/// Circuit breaker and rate limiter for run handler.
#[derive(Clone)]
pub struct SafetyState {
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub rate_limiter: Arc<RateLimiter>,
    pub cost_tracker: Arc<CostTracker>,
}

/// Shared state for the API: unit of work, event bus, safety, backend registry.
#[derive(Clone)]
pub struct AppState {
    pub uow: Arc<UnitOfWork>,
    pub event_bus: Arc<EventBus>,
    pub safety: SafetyState,
    pub backend_registry: Arc<BackendRegistry>,
}

impl AppState {
    pub fn new(
        uow: Arc<UnitOfWork>,
        event_bus: Arc<EventBus>,
        safety: SafetyState,
        backend_registry: Arc<BackendRegistry>,
    ) -> Self {
        Self {
            uow,
            event_bus,
            safety,
            backend_registry,
        }
    }
}
