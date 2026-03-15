//! Safety controls: circuit breaker, rate limiter, budget enforcement.
//! Circuit breaker: 3-state FSM to stop calls when failures exceed threshold.

pub mod scanner;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Serializable snapshot of circuit breaker state for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub state: String,
    pub failure_count: u32,
    pub last_failure_epoch_ms: Option<u64>,
}

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

    /// Export current state for persistence.
    pub fn export_state(&self) -> CircuitBreakerState {
        let state_name = match self.state() {
            CircuitState::Closed => "Closed",
            CircuitState::Open => "Open",
            CircuitState::HalfOpen => "HalfOpen",
        };
        let last_failure_ms = self
            .last_failure
            .lock()
            .unwrap()
            .map(|i| i.elapsed().as_millis() as u64);
        CircuitBreakerState {
            state: state_name.to_string(),
            failure_count: self.failure_count.load(Ordering::SeqCst),
            last_failure_epoch_ms: last_failure_ms,
        }
    }

    /// Restore state from persistence. Call on startup.
    pub fn restore_state(&self, saved: &CircuitBreakerState) {
        let state_val = match saved.state.as_str() {
            "Open" => CircuitState::Open as u8,
            "HalfOpen" => CircuitState::HalfOpen as u8,
            _ => CircuitState::Closed as u8,
        };
        self.state.store(state_val, Ordering::SeqCst);
        self.failure_count
            .store(saved.failure_count, Ordering::SeqCst);

        if state_val == CircuitState::Open as u8 {
            if let Some(ms_ago) = saved.last_failure_epoch_ms {
                let elapsed = Duration::from_millis(ms_ago);
                if elapsed < self.timeout {
                    // Still within timeout — keep Open
                    *self.last_failure.lock().unwrap() =
                        Some(Instant::now() - (self.timeout - elapsed));
                } else {
                    // Timeout has passed — transition to HalfOpen
                    self.state
                        .store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
        }
    }

    pub fn reset(&self) {
        self.state.store(CircuitState::Closed as u8, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.last_failure.lock().unwrap() = None;
    }
}

/// Token-bucket rate limiter: refill at refill_interval, up to max_tokens.
pub struct RateLimiter {
    tokens: AtomicU32,
    max_tokens: u32,
    refill_interval: Duration,
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    /// Create a rate limiter. Refills one token every refill_interval.
    pub fn new(max_tokens: u32, refill_interval: Duration) -> Self {
        Self {
            tokens: AtomicU32::new(max_tokens),
            max_tokens,
            refill_interval,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    /// Refill tokens based on elapsed time, then try to take one token. Returns true if acquired.
    pub fn try_acquire(&self) -> bool {
        let mut last = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = last.elapsed();
        if elapsed >= self.refill_interval && self.refill_interval.as_nanos() > 0 {
            let refills = (elapsed.as_nanos() / self.refill_interval.as_nanos()) as u32;
            *last = now;
            drop(last);
            let current = self.tokens.load(Ordering::SeqCst);
            let added = refills.min(self.max_tokens.saturating_sub(current));
            self.tokens.fetch_add(added, Ordering::SeqCst);
        }
        self.tokens
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |t| {
                if t > 0 {
                    Some(t - 1)
                } else {
                    None
                }
            })
            .is_ok()
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("max_tokens", &self.max_tokens)
            .field("refill_interval", &self.refill_interval)
            .finish_non_exhaustive()
    }
}

/// Result of a budget check against current cost.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetStatus {
    /// Cost is within acceptable range.
    Ok,
    /// Cost has crossed the warning threshold.
    Warning { current_cost: f64, threshold: f64 },
    /// Cost has reached or exceeded the hard limit.
    Exceeded { current_cost: f64, limit: f64 },
}

/// Tracks cost/budget for agent usage. Holds warning and limit thresholds,
/// returns a `BudgetStatus` when checked against a cost value.
#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    warn: Option<f64>,
    limit: Option<f64>,
}

impl CostTracker {
    pub fn new(warn: Option<f64>, limit: Option<f64>) -> Self {
        Self { warn, limit }
    }

    /// Check cost against thresholds. Exceeded takes priority over Warning.
    pub fn check(&self, cost: f64) -> BudgetStatus {
        if let Some(limit) = self.limit {
            if cost >= limit {
                return BudgetStatus::Exceeded { current_cost: cost, limit };
            }
        }
        if let Some(warn) = self.warn {
            if cost >= warn {
                return BudgetStatus::Warning { current_cost: cost, threshold: warn };
            }
        }
        BudgetStatus::Ok
    }

    pub fn warn_threshold(&self) -> Option<f64> {
        self.warn
    }

    pub fn limit_threshold(&self) -> Option<f64> {
        self.limit
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

    #[test]
    fn rate_limiter_allows_up_to_max_tokens() {
        let rl = RateLimiter::new(3, Duration::from_secs(60));
        assert!(rl.try_acquire());
        assert!(rl.try_acquire());
        assert!(rl.try_acquire());
        assert!(!rl.try_acquire());
    }

    #[test]
    fn rate_limiter_rejects_after_exhaustion() {
        let rl = RateLimiter::new(1, Duration::from_secs(60));
        assert!(rl.try_acquire());
        assert!(!rl.try_acquire());
        assert!(!rl.try_acquire());
    }

    #[test]
    fn rate_limiter_refills_over_time() {
        let rl = RateLimiter::new(1, Duration::from_millis(50));
        assert!(rl.try_acquire());
        assert!(!rl.try_acquire());
        thread::sleep(Duration::from_millis(60));
        assert!(rl.try_acquire());
    }

    #[test]
    fn cost_tracker_ok_when_no_thresholds() {
        let ct = CostTracker::default();
        assert_eq!(ct.check(100.0), BudgetStatus::Ok);
    }

    #[test]
    fn cost_tracker_warning_when_above_warn() {
        let ct = CostTracker::new(Some(5.0), None);
        assert_eq!(ct.check(3.0), BudgetStatus::Ok);
        assert_eq!(ct.check(5.0), BudgetStatus::Warning { current_cost: 5.0, threshold: 5.0 });
        assert_eq!(ct.check(8.0), BudgetStatus::Warning { current_cost: 8.0, threshold: 5.0 });
    }

    #[test]
    fn cost_tracker_exceeded_takes_priority() {
        let ct = CostTracker::new(Some(5.0), Some(10.0));
        assert_eq!(ct.check(3.0), BudgetStatus::Ok);
        assert_eq!(ct.check(7.0), BudgetStatus::Warning { current_cost: 7.0, threshold: 5.0 });
        assert_eq!(ct.check(10.0), BudgetStatus::Exceeded { current_cost: 10.0, limit: 10.0 });
        assert_eq!(ct.check(15.0), BudgetStatus::Exceeded { current_cost: 15.0, limit: 10.0 });
    }

    #[test]
    fn cost_tracker_limit_only_no_warn() {
        let ct = CostTracker::new(None, Some(10.0));
        assert_eq!(ct.check(9.0), BudgetStatus::Ok);
        assert_eq!(ct.check(10.0), BudgetStatus::Exceeded { current_cost: 10.0, limit: 10.0 });
    }
}
