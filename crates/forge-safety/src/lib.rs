//! Safety controls: circuit breaker, rate limiter, budget enforcement.
//! Circuit breaker: 3-state FSM to stop calls when failures exceed threshold.

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Circuit breaker error when the circuit is open (requests rejected).
#[derive(Debug, Clone)]
pub struct CircuitBreakerError;

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "circuit breaker open")
    }
}

impl std::error::Error for CircuitBreakerError {}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum CircuitState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

/// 3-state circuit breaker: Closed → (N failures) → Open → (timeout) → HalfOpen → (success) → Closed.
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 2, Duration::from_secs(60))
    }
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u32,
        success_threshold: u32,
        timeout: Duration,
    ) -> Self {
        Self {
            state: AtomicU8::new(CircuitState::Closed as u8),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure: Mutex::new(None),
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    /// Returns Ok(()) if request is allowed, Err if circuit is open.
    pub fn check(&self) -> Result<(), CircuitBreakerError> {
        match self.state() {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                let mut last = self.last_failure.lock().unwrap();
                if let Some(instant) = *last {
                    if instant.elapsed() >= self.timeout {
                        *last = None;
                        self.state.store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
                        self.success_count.store(0, Ordering::SeqCst);
                        return Ok(());
                    }
                }
                Err(CircuitBreakerError)
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    pub fn record_success(&self) {
        match self.state() {
            CircuitState::Closed => {}
            CircuitState::Open => {}
            CircuitState::HalfOpen => {
                let prev = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if prev >= self.success_threshold {
                    self.state.store(CircuitState::Closed as u8, Ordering::SeqCst);
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
        }
    }

    pub fn record_failure(&self) {
        match self.state() {
            CircuitState::Closed => {
                let prev = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if prev >= self.failure_threshold {
                    self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
                    *self.last_failure.lock().unwrap() = Some(Instant::now());
                }
            }
            CircuitState::Open => {}
            CircuitState::HalfOpen => {
                self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
                *self.last_failure.lock().unwrap() = Some(Instant::now());
            }
        }
    }

    pub fn state(&self) -> CircuitState {
        match self.state.load(Ordering::SeqCst) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            _ => CircuitState::HalfOpen,
        }
    }

    pub fn reset(&self) {
        self.state.store(CircuitState::Closed as u8, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.last_failure.lock().unwrap() = None;
    }
}

/// Rate limiter for API / agent calls. Stub for Phase 4.
#[derive(Debug, Clone, Default)]
pub struct RateLimiter;

/// Tracks cost/budget for agent usage. Stub for Phase 4.
#[derive(Debug, Clone, Default)]
pub struct CostTracker;

impl RateLimiter {
    pub fn new() -> Self {
        Self
    }
}

impl CostTracker {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn closed_allows_requests() {
        let cb = CircuitBreaker::new(2, 1, Duration::from_secs(60));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.check().is_ok());
    }

    #[test]
    fn opens_after_n_failures() {
        let cb = CircuitBreaker::new(3, 2, Duration::from_secs(60));
        assert!(cb.check().is_ok());
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(cb.check().is_err());
    }

    #[test]
    fn open_rejects_immediately() {
        let cb = CircuitBreaker::new(1, 1, Duration::from_secs(10));
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(cb.check().is_err());
        assert!(cb.check().is_err());
    }

    #[test]
    fn transitions_to_half_open_after_timeout() {
        let cb = CircuitBreaker::new(1, 2, Duration::from_millis(50));
        cb.record_failure();
        assert!(cb.check().is_err());
        thread::sleep(Duration::from_millis(60));
        assert!(cb.check().is_ok());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn closes_after_success_in_half_open() {
        let cb = CircuitBreaker::new(1, 2, Duration::from_millis(50));
        cb.record_failure();
        thread::sleep(Duration::from_millis(60));
        let _ = cb.check();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.check().is_ok());
    }

    #[test]
    fn half_open_failure_goes_back_to_open() {
        let cb = CircuitBreaker::new(1, 2, Duration::from_millis(50));
        cb.record_failure();
        thread::sleep(Duration::from_millis(60));
        let _ = cb.check();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(cb.check().is_err());
    }

    #[test]
    fn reset_returns_to_closed() {
        let cb = CircuitBreaker::new(1, 1, Duration::from_secs(60));
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.check().is_ok());
    }
}
