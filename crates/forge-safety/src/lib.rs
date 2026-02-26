//! Safety controls: circuit breaker, rate limiter, budget enforcement.
//! Phase 0 stub — minimal types for later implementation.

/// Circuit breaker to stop calls when failures exceed threshold. Stub for Phase 4.
#[derive(Debug, Clone, Default)]
pub struct CircuitBreaker;

/// Rate limiter for API / agent calls. Stub for Phase 4.
#[derive(Debug, Clone, Default)]
pub struct RateLimiter;

/// Tracks cost/budget for agent usage. Stub for Phase 4.
#[derive(Debug, Clone, Default)]
pub struct CostTracker;

impl CircuitBreaker {
    /// Stub constructor.
    pub fn new() -> Self {
        Self
    }
}

impl RateLimiter {
    /// Stub constructor.
    pub fn new() -> Self {
        Self
    }
}

impl CostTracker {
    /// Stub constructor.
    pub fn new() -> Self {
        Self
    }
}
