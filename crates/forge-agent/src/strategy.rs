//! Strategy types for best-of-N agent execution.
//!
//! A [`Strategy`] biases an agent's approach by appending a short suffix to
//! the system prompt. [`StrategySet`] holds a collection of strategies to
//! run in parallel, and [`StrategySet::default_three`] provides a sensible
//! default set: minimal changes, modular refactor, and thorough with tests.

/// A single strategy that biases an agent's approach.
///
/// The `system_prompt_suffix` is appended to the agent's base system prompt
/// before execution, steering the agent toward a particular style.
#[derive(Debug, Clone)]
pub struct Strategy {
    /// Human-readable name for this strategy (e.g. "minimal_changes").
    pub name: String,
    /// Short suffix (2-3 sentences) appended to the system prompt.
    pub system_prompt_suffix: String,
}

/// A set of strategies to run in parallel for best-of-N selection.
#[derive(Debug, Clone)]
pub struct StrategySet {
    pub strategies: Vec<Strategy>,
}

impl StrategySet {
    /// Returns the default set of three strategies:
    /// 1. **Minimal changes** -- smallest diff, fewest files touched.
    /// 2. **Modular refactor** -- clean abstractions, extract helpers.
    /// 3. **Thorough with tests** -- comprehensive solution with test coverage.
    pub fn default_three() -> Self {
        Self {
            strategies: vec![
                Strategy {
                    name: "minimal_changes".to_string(),
                    system_prompt_suffix: concat!(
                        "Make the smallest possible change to solve the problem. ",
                        "Touch as few files as possible and prefer the simplest diff ",
                        "that correctly addresses the requirement."
                    )
                    .to_string(),
                },
                Strategy {
                    name: "modular_refactor".to_string(),
                    system_prompt_suffix: concat!(
                        "Favor clean abstractions and modular design. ",
                        "Extract reusable helpers where appropriate and ensure ",
                        "each function has a single clear responsibility."
                    )
                    .to_string(),
                },
                Strategy {
                    name: "thorough_with_tests".to_string(),
                    system_prompt_suffix: concat!(
                        "Provide a comprehensive solution with thorough test coverage. ",
                        "Add unit tests for new logic, handle edge cases explicitly, ",
                        "and document any non-obvious decisions."
                    )
                    .to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_set_has_three() {
        let set = StrategySet::default_three();
        assert_eq!(set.strategies.len(), 3);

        // Each strategy has a non-empty name and suffix
        for s in &set.strategies {
            assert!(!s.name.is_empty(), "strategy name must not be empty");
            assert!(
                !s.system_prompt_suffix.is_empty(),
                "strategy suffix must not be empty"
            );
        }

        // Verify the expected names
        let names: Vec<&str> = set.strategies.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["minimal_changes", "modular_refactor", "thorough_with_tests"]
        );
    }
}
