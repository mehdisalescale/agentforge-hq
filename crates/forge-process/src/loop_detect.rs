//! Loop detection for agent output streams.
//!
//! Tracks hashes of recent outputs and detects when an agent is producing
//! repetitive content, indicating a stuck loop.

use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Configuration for the exit gate that validates agent completion.
#[derive(Debug, Clone)]
pub struct ExitGateConfig {
    /// Patterns that indicate successful completion (any match = success).
    pub completion_patterns: Vec<String>,
    /// Maximum retry attempts when exit gate fails.
    pub max_retries: u32,
    /// Number of recent outputs to track for loop detection.
    pub loop_detection_window: usize,
}

impl Default for ExitGateConfig {
    fn default() -> Self {
        Self {
            completion_patterns: vec![],
            max_retries: 2,
            loop_detection_window: 5,
        }
    }
}

/// Detects repetitive output by tracking hashes of recent outputs.
pub struct LoopDetector {
    window: VecDeque<u64>,
    max_window: usize,
}

impl LoopDetector {
    pub fn new(max_window: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_window),
            max_window,
        }
    }

    /// Push a new output and check for loops.
    /// Returns `true` if a loop is detected (hash matches any in the window).
    pub fn push(&mut self, output: &str) -> bool {
        let hash = Self::hash_output(output);
        let is_repeat = self.window.contains(&hash);

        if self.window.len() >= self.max_window {
            self.window.pop_front();
        }
        self.window.push_back(hash);

        is_repeat
    }

    pub fn reset(&mut self) {
        self.window.clear();
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    fn hash_output(output: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        output.hash(&mut hasher);
        hasher.finish()
    }
}

/// Check if output contains any of the completion patterns.
pub fn check_completion_patterns(output: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }
    patterns.iter().any(|pattern| output.contains(pattern))
}

/// Validate exit conditions for an agent run.
/// Returns Ok(()) if valid, Err(reason) if exit gate should trigger.
pub fn validate_exit(
    exit_code: i32,
    output: &str,
    config: &ExitGateConfig,
    detector: &mut LoopDetector,
) -> Result<(), String> {
    if detector.push(output) {
        return Err("loop detected: output matches previous output in window".to_string());
    }
    if exit_code != 0 {
        return Err(format!("non-zero exit code: {}", exit_code));
    }
    if !check_completion_patterns(output, &config.completion_patterns) {
        return Err("output missing required completion pattern".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_repeat_detected() {
        let mut d = LoopDetector::new(5);
        assert!(!d.push("first output"));
        assert!(!d.push("second output"));
        assert!(d.push("first output"));
    }

    #[test]
    fn non_repeat_passes() {
        let mut d = LoopDetector::new(5);
        assert!(!d.push("output 1"));
        assert!(!d.push("output 2"));
        assert!(!d.push("output 3"));
        assert!(!d.push("output 4"));
    }

    #[test]
    fn window_eviction() {
        let mut d = LoopDetector::new(3);
        assert!(!d.push("a"));
        assert!(!d.push("b"));
        assert!(!d.push("c"));
        assert!(!d.push("d"));
        assert!(!d.push("a")); // evicted
    }

    #[test]
    fn reset_clears_history() {
        let mut d = LoopDetector::new(5);
        d.push("test");
        assert_eq!(d.len(), 1);
        d.reset();
        assert!(d.is_empty());
        assert!(!d.push("test"));
    }

    #[test]
    fn completion_pattern_match() {
        let patterns = vec!["DONE".to_string(), "COMPLETE".to_string()];
        assert!(check_completion_patterns("Task DONE successfully", &patterns));
        assert!(check_completion_patterns("COMPLETE: all good", &patterns));
        assert!(!check_completion_patterns("Still working...", &patterns));
    }

    #[test]
    fn empty_patterns_always_passes() {
        assert!(check_completion_patterns("anything", &[]));
    }

    #[test]
    fn validate_exit_success() {
        let config = ExitGateConfig {
            completion_patterns: vec!["DONE".to_string()],
            max_retries: 2,
            loop_detection_window: 5,
        };
        let mut d = LoopDetector::new(5);
        assert!(validate_exit(0, "Task DONE", &config, &mut d).is_ok());
    }

    #[test]
    fn validate_exit_nonzero_code() {
        let config = ExitGateConfig::default();
        let mut d = LoopDetector::new(5);
        let result = validate_exit(1, "error", &config, &mut d);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non-zero exit code"));
    }

    #[test]
    fn validate_exit_missing_pattern() {
        let config = ExitGateConfig {
            completion_patterns: vec!["DONE".to_string()],
            max_retries: 2,
            loop_detection_window: 5,
        };
        let mut d = LoopDetector::new(5);
        let result = validate_exit(0, "Still working", &config, &mut d);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("completion pattern"));
    }

    #[test]
    fn validate_exit_loop_detected() {
        let config = ExitGateConfig::default();
        let mut d = LoopDetector::new(5);
        assert!(validate_exit(0, "output A", &config, &mut d).is_ok());
        let result = validate_exit(0, "output A", &config, &mut d);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("loop detected"));
    }
}
