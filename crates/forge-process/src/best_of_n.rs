//! Best-of-N runner: executes the same prompt with multiple strategies
//! and selects the best result using heuristic scoring.
//!
//! Wraps [`ConcurrentRunner`] to run N variants in parallel, then uses
//! [`select_best`] (a pure, synchronous function) to pick the winner.

use std::sync::Arc;

use forge_core::event_bus::EventBus;
use forge_core::ids::{AgentId, SessionId};

use crate::concurrent::{ConcurrentRunner, SubTask, SubTaskResult};

/// The result of selecting the best output from N candidates.
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Index into the results vector for the chosen candidate.
    pub chosen_index: usize,
    /// Human-readable reason for the selection.
    pub reason: String,
    /// Suggested improvements or notes about the chosen result.
    pub improvements: Vec<String>,
}

/// Runs the same prompt with N different strategy suffixes and picks the best.
///
/// Delegates parallel execution to [`ConcurrentRunner`] and selection to
/// the pure [`select_best`] function.
pub struct BestOfNRunner {
    runner: ConcurrentRunner,
}

impl BestOfNRunner {
    /// Create a new runner backed by a [`ConcurrentRunner`] with the given
    /// event bus and concurrency limit.
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self {
        Self {
            runner: ConcurrentRunner::new(event_bus, max_concurrent),
        }
    }

    /// Runs the same prompt with N strategies, returns all results plus the
    /// selection decision.
    ///
    /// For each strategy in `strategies`, the `base_task` prompt is cloned
    /// and the strategy's `system_prompt_suffix` is appended. All N variants
    /// are then executed concurrently via [`ConcurrentRunner::run_all`].
    pub async fn run_best_of_n(
        &self,
        parent_session_id: &SessionId,
        base_task: SubTask,
        strategies: &forge_agent::strategy::StrategySet,
    ) -> (Vec<SubTaskResult>, SelectionResult) {
        let tasks: Vec<SubTask> = strategies
            .strategies
            .iter()
            .map(|strategy| SubTask {
                agent_id: AgentId::new(),
                prompt: format!(
                    "{}\n\n[Strategy: {}] {}",
                    base_task.prompt, strategy.name, strategy.system_prompt_suffix
                ),
                working_dir: base_task.working_dir.clone(),
            })
            .collect();

        let results = self.runner.run_all(parent_session_id, tasks).await;
        let selection = select_best(&results);

        (results, selection)
    }
}

/// Compares results and picks the best one using heuristic scoring.
///
/// Scoring rules (applied per result):
/// - **+10** if `success` is true (exit code 0)
/// - **+1** per 100 characters of output
/// - **-5** per occurrence of "error" or "Error" in output
/// - **-3** per occurrence of "fail" or "Fail" in output
///
/// This is a pure function with no I/O -- easy to test deterministically.
pub fn select_best(results: &[SubTaskResult]) -> SelectionResult {
    if results.is_empty() {
        return SelectionResult {
            chosen_index: 0,
            reason: "No results to compare".to_string(),
            improvements: vec!["Provide at least one strategy to evaluate".to_string()],
        };
    }

    let scores: Vec<i64> = results.iter().map(score_result).collect();

    let (chosen_index, &best_score) = scores
        .iter()
        .enumerate()
        .max_by_key(|(_, score)| *score)
        .unwrap(); // safe: results is non-empty

    let mut improvements = Vec::new();

    if best_score <= 0 {
        improvements.push("All candidates scored poorly; consider revising the prompt".to_string());
    }

    if !results[chosen_index].success {
        improvements.push("The chosen result did not exit successfully".to_string());
    }

    let reason = format!(
        "Candidate {} selected with score {} (success={}, output_len={})",
        chosen_index,
        best_score,
        results[chosen_index].success,
        results[chosen_index].output.len(),
    );

    SelectionResult {
        chosen_index,
        reason,
        improvements,
    }
}

/// Compute a heuristic score for a single result.
fn score_result(result: &SubTaskResult) -> i64 {
    let mut score: i64 = 0;

    // +10 for success
    if result.success {
        score += 10;
    }

    // +1 per 100 chars of output
    score += (result.output.len() / 100) as i64;

    // -5 per "error" or "Error" occurrence
    let error_count = result.output.matches("error").count()
        + result.output.matches("Error").count();
    score -= (error_count as i64) * 5;

    // -3 per "fail" or "Fail" occurrence
    let fail_count = result.output.matches("fail").count()
        + result.output.matches("Fail").count();
    score -= (fail_count as i64) * 3;

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::ids::{AgentId, SessionId};

    fn make_result(success: bool, exit_code: i32, output: &str) -> SubTaskResult {
        SubTaskResult {
            agent_id: AgentId::new(),
            session_id: SessionId::new(),
            output: output.to_string(),
            exit_code,
            success,
        }
    }

    #[test]
    fn select_best_prefers_success() {
        let results = vec![
            // Failed but longer output (500 chars = +5 length, +0 success = 5)
            make_result(false, 1, &"a".repeat(500)),
            // Succeeded with short output (+0 length, +10 success = 10)
            make_result(true, 0, "done"),
        ];

        let selection = select_best(&results);
        assert_eq!(
            selection.chosen_index, 1,
            "should prefer successful result even if shorter"
        );
    }

    #[test]
    fn select_best_prefers_longer_on_tie() {
        let results = vec![
            make_result(true, 0, "short"),
            make_result(true, 0, &"x".repeat(1000)),
        ];

        let selection = select_best(&results);
        assert_eq!(
            selection.chosen_index, 1,
            "on equal success, longer output should win"
        );
    }

    #[test]
    fn select_best_penalizes_errors() {
        let results = vec![
            // Long output but contains many "error" / "Error" strings
            make_result(
                true,
                0,
                &format!(
                    "{}error Error error Error error",
                    "x".repeat(1000)
                ),
            ),
            // Shorter, clean output
            make_result(true, 0, &"y".repeat(500)),
        ];

        let selection = select_best(&results);
        assert_eq!(
            selection.chosen_index, 1,
            "result with error keywords should score lower"
        );
    }

    #[test]
    fn select_best_empty_results() {
        let selection = select_best(&[]);
        assert_eq!(selection.chosen_index, 0);
        assert!(!selection.improvements.is_empty());
    }
}
