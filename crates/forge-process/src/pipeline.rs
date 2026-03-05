//! Pipeline engine: sequential and fanout step execution over sub-agents.
//!
//! A `Pipeline` is a sequence of `PipelineStep`s. Each step is either:
//! - **Sequential**: runs a single agent
//! - **Fanout**: runs multiple agents concurrently
//!
//! Steps are chained: the concatenated output of one step becomes the `{input}`
//! placeholder value for the next step's prompt template.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::concurrent::{ConcurrentRunner, SubTask, SubTaskResult};
use crate::spawn::SpawnConfig;
use forge_core::event_bus::EventBus;
use forge_core::ids::{AgentId, SessionId};

/// A single step in a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineStep {
    /// Run a single agent sequentially.
    Sequential {
        agent_id: String,
        prompt_template: String,
    },
    /// Run multiple agents concurrently (fan-out).
    Fanout {
        agent_ids: Vec<String>,
        prompt_template: String,
    },
}

/// A pipeline definition: an ordered list of steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
}

/// The result of executing a single pipeline step.
pub struct StepResult {
    pub step_index: usize,
    pub outputs: Vec<SubTaskResult>,
    pub success: bool,
}

impl std::fmt::Debug for StepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StepResult")
            .field("step_index", &self.step_index)
            .field("outputs_count", &self.outputs.len())
            .field("success", &self.success)
            .finish()
    }
}

/// Executes a `Pipeline` by running each step in sequence, threading outputs
/// from one step into the next via `{input}` placeholder substitution.
pub struct PipelineRunner {
    event_bus: Arc<EventBus>,
    max_concurrent: usize,
    spawn_config: SpawnConfig,
}

impl PipelineRunner {
    /// Create a new runner with default `SpawnConfig::from_env()`.
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self {
        Self {
            event_bus,
            max_concurrent,
            spawn_config: SpawnConfig::from_env(),
        }
    }

    /// Create a runner with a custom `SpawnConfig`.
    pub fn with_spawn_config(
        event_bus: Arc<EventBus>,
        max_concurrent: usize,
        spawn_config: SpawnConfig,
    ) -> Self {
        Self {
            event_bus,
            max_concurrent,
            spawn_config,
        }
    }

    /// Execute a pipeline: each step feeds its concatenated output to the next.
    ///
    /// Stops on the first failed step (any `SubTaskResult` with `success == false`).
    /// Returns all `StepResult`s accumulated up to and including the failed step.
    pub async fn run(
        &self,
        parent_session_id: &SessionId,
        pipeline: &Pipeline,
        initial_input: &str,
        working_dir: &str,
    ) -> Vec<StepResult> {
        let mut results = Vec::new();
        let mut current_input = initial_input.to_string();

        for (step_index, step) in pipeline.steps.iter().enumerate() {
            let tasks = self.build_tasks(step, &current_input, working_dir);

            // Use max_concurrent=1 for Sequential, full concurrency for Fanout
            let concurrency = match step {
                PipelineStep::Sequential { .. } => 1,
                PipelineStep::Fanout { .. } => self.max_concurrent,
            };

            let runner = ConcurrentRunner::with_spawn_config(
                Arc::clone(&self.event_bus),
                concurrency,
                self.spawn_config.clone(),
            );

            let sub_results = runner.run_all(parent_session_id, tasks).await;

            let success = sub_results.iter().all(|r| r.success);

            // Concatenate outputs for next step's input
            current_input = sub_results
                .iter()
                .map(|r| r.output.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            results.push(StepResult {
                step_index,
                outputs: sub_results,
                success,
            });

            // Stop on first failure
            if !success {
                break;
            }
        }

        results
    }

    /// Build `SubTask` list from a `PipelineStep`, substituting `{input}` in the
    /// prompt template with the provided input string.
    fn build_tasks(
        &self,
        step: &PipelineStep,
        input: &str,
        working_dir: &str,
    ) -> Vec<SubTask> {
        match step {
            PipelineStep::Sequential {
                agent_id,
                prompt_template,
            } => {
                let prompt = prompt_template.replace("{input}", input);
                vec![SubTask {
                    agent_id: AgentId(uuid::Uuid::parse_str(agent_id).unwrap_or_else(|_| uuid::Uuid::new_v4())),
                    prompt,
                    working_dir: working_dir.to_string(),
                }]
            }
            PipelineStep::Fanout {
                agent_ids,
                prompt_template,
            } => {
                let prompt = prompt_template.replace("{input}", input);
                agent_ids
                    .iter()
                    .map(|aid| SubTask {
                        agent_id: AgentId(uuid::Uuid::parse_str(aid).unwrap_or_else(|_| uuid::Uuid::new_v4())),
                        prompt: prompt.clone(),
                        working_dir: working_dir.to_string(),
                    })
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_step_serializes() {
        let step = PipelineStep::Sequential {
            agent_id: "abc-123".to_string(),
            prompt_template: "Do {input}".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        let back: PipelineStep = serde_json::from_str(&json).unwrap();
        match back {
            PipelineStep::Sequential {
                agent_id,
                prompt_template,
            } => {
                assert_eq!(agent_id, "abc-123");
                assert_eq!(prompt_template, "Do {input}");
            }
            _ => panic!("expected Sequential"),
        }
    }

    #[test]
    fn pipeline_step_fanout_serializes() {
        let step = PipelineStep::Fanout {
            agent_ids: vec!["a1".to_string(), "a2".to_string()],
            prompt_template: "Analyze: {input}".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        let back: PipelineStep = serde_json::from_str(&json).unwrap();
        match back {
            PipelineStep::Fanout {
                agent_ids,
                prompt_template,
            } => {
                assert_eq!(agent_ids, vec!["a1", "a2"]);
                assert_eq!(prompt_template, "Analyze: {input}");
            }
            _ => panic!("expected Fanout"),
        }
    }

    #[test]
    fn pipeline_from_json() {
        let json = r#"{
            "steps": [
                {"type": "Sequential", "agent_id": "agent-1", "prompt_template": "Step 1: {input}"},
                {"type": "Fanout", "agent_ids": ["agent-2", "agent-3"], "prompt_template": "Step 2: {input}"}
            ]
        }"#;
        let pipeline: Pipeline = serde_json::from_str(json).unwrap();
        assert_eq!(pipeline.steps.len(), 2);
        match &pipeline.steps[0] {
            PipelineStep::Sequential { agent_id, .. } => assert_eq!(agent_id, "agent-1"),
            _ => panic!("expected Sequential"),
        }
        match &pipeline.steps[1] {
            PipelineStep::Fanout { agent_ids, .. } => {
                assert_eq!(agent_ids.len(), 2);
                assert_eq!(agent_ids[0], "agent-2");
            }
            _ => panic!("expected Fanout"),
        }
    }

    #[test]
    fn step_result_tracks_index() {
        let result = StepResult {
            step_index: 3,
            outputs: vec![],
            success: true,
        };
        assert_eq!(result.step_index, 3);
        assert!(result.success);
        assert!(result.outputs.is_empty());
    }

    #[test]
    fn step_result_tracks_failure() {
        let result = StepResult {
            step_index: 0,
            outputs: vec![],
            success: false,
        };
        assert_eq!(result.step_index, 0);
        assert!(!result.success);
    }

    #[test]
    fn pipeline_empty_steps() {
        let pipeline = Pipeline { steps: vec![] };
        let json = serde_json::to_string(&pipeline).unwrap();
        let back: Pipeline = serde_json::from_str(&json).unwrap();
        assert!(back.steps.is_empty());
    }
}
