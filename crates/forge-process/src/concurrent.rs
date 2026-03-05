//! Concurrent sub-agent runner with semaphore-limited parallel spawning.

use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::spawn::{spawn, ProcessHandle, SpawnConfig};
use forge_core::event_bus::EventBus;
use forge_core::events::ForgeEvent;
use forge_core::ids::{AgentId, SessionId};

/// A sub-task to be executed concurrently.
pub struct SubTask {
    pub agent_id: AgentId,
    pub prompt: String,
    pub working_dir: String,
}

/// Result from a completed sub-task.
#[derive(Debug)]
pub struct SubTaskResult {
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub output: String,
    pub exit_code: i32,
    pub success: bool,
}

/// Runs multiple sub-agent processes concurrently with a configurable
/// concurrency limit using a tokio semaphore.
pub struct ConcurrentRunner {
    event_bus: Arc<EventBus>,
    max_concurrent: usize,
    spawn_config: SpawnConfig,
}

impl ConcurrentRunner {
    /// Create a runner using `SpawnConfig::from_env()` as the base config.
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self {
        Self {
            event_bus,
            max_concurrent,
            spawn_config: SpawnConfig::from_env(),
        }
    }

    /// Create a runner with a custom base `SpawnConfig`.
    pub fn with_spawn_config(
        event_bus: Arc<EventBus>,
        max_concurrent: usize,
        config: SpawnConfig,
    ) -> Self {
        Self {
            event_bus,
            max_concurrent,
            spawn_config: config,
        }
    }

    /// Run all sub-tasks concurrently (up to `max_concurrent` at a time).
    /// Emits `SubAgent*` events at each lifecycle point.
    /// Returns results for all tasks (success or failure).
    pub async fn run_all(
        &self,
        parent_session_id: &SessionId,
        tasks: Vec<SubTask>,
    ) -> Vec<SubTaskResult> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut handles = Vec::new();

        for task in tasks {
            let sem = semaphore.clone();
            let event_bus = Arc::clone(&self.event_bus);
            let parent_sid = parent_session_id.clone();
            let config = if task.working_dir.is_empty() {
                self.spawn_config.clone()
            } else {
                self.spawn_config.clone().with_working_dir(&task.working_dir)
            };

            // Emit request before scheduling
            let _ = event_bus.emit(ForgeEvent::SubAgentRequested {
                parent_session_id: parent_sid.clone(),
                sub_agent_id: task.agent_id.clone(),
                prompt: task.prompt.clone(),
                timestamp: chrono::Utc::now(),
            });

            let handle = tokio::spawn(async move {
                // Acquire semaphore permit (limits concurrency)
                let _permit = sem.acquire_owned().await.unwrap();

                let session_id = SessionId::new();

                let _ = event_bus.emit(ForgeEvent::SubAgentStarted {
                    parent_session_id: parent_sid.clone(),
                    sub_agent_id: task.agent_id.clone(),
                    session_id: session_id.clone(),
                    timestamp: chrono::Utc::now(),
                });

                match spawn(&config, &task.prompt, None).await {
                    Ok(mut proc_handle) => {
                        let output = collect_output(&mut proc_handle).await;
                        let status = proc_handle.wait().await;
                        let exit_code = status
                            .map(|s| s.code().unwrap_or(-1))
                            .unwrap_or(-1);
                        let success = exit_code == 0;

                        if success {
                            let _ = event_bus.emit(ForgeEvent::SubAgentCompleted {
                                parent_session_id: parent_sid,
                                sub_agent_id: task.agent_id.clone(),
                                session_id: session_id.clone(),
                                timestamp: chrono::Utc::now(),
                            });
                        } else {
                            let _ = event_bus.emit(ForgeEvent::SubAgentFailed {
                                parent_session_id: parent_sid,
                                sub_agent_id: task.agent_id.clone(),
                                error: format!("exit code {}", exit_code),
                                timestamp: chrono::Utc::now(),
                            });
                        }

                        SubTaskResult {
                            agent_id: task.agent_id,
                            session_id,
                            output,
                            exit_code,
                            success,
                        }
                    }
                    Err(e) => {
                        let _ = event_bus.emit(ForgeEvent::SubAgentFailed {
                            parent_session_id: parent_sid,
                            sub_agent_id: task.agent_id.clone(),
                            error: e.to_string(),
                            timestamp: chrono::Utc::now(),
                        });

                        SubTaskResult {
                            agent_id: task.agent_id,
                            session_id,
                            output: String::new(),
                            exit_code: -1,
                            success: false,
                        }
                    }
                }
                // _permit is dropped here, releasing semaphore
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        results
    }
}

/// Collect all stdout from a process handle into a single string.
async fn collect_output(handle: &mut ProcessHandle) -> String {
    use tokio::io::AsyncReadExt;
    let mut output = String::new();
    if let Some(mut stdout) = handle.take_stdout() {
        let _ = stdout.read_to_string(&mut output).await;
    }
    output
}

/// Aggregate sub-task results into a summary.
pub fn aggregate_results(results: &[SubTaskResult]) -> String {
    let total = results.len();
    let succeeded = results.iter().filter(|r| r.success).count();

    let mut summary = format!(
        "## Sub-agent Results: {}/{} succeeded\n\n",
        succeeded, total
    );

    for (i, result) in results.iter().enumerate() {
        let status = if result.success { "OK" } else { "FAILED" };
        summary.push_str(&format!(
            "### Sub-task {} [{}]\n{}\n\n",
            i + 1,
            status,
            if result.output.is_empty() {
                "(no output)".to_string()
            } else {
                result.output.chars().take(2000).collect::<String>()
            }
        ));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_config() -> SpawnConfig {
        SpawnConfig {
            command: "echo".to_string(),
            args_before_prompt: vec![],
            working_dir: None,
            env_remove: vec![],
            env_set: vec![],
            timeout: Some(Duration::from_secs(5)),
        }
    }

    #[tokio::test]
    async fn concurrent_runner_respects_semaphore() {
        let bus = Arc::new(EventBus::new(64));
        let runner = ConcurrentRunner::with_spawn_config(
            Arc::clone(&bus),
            1, // max_concurrent = 1 forces sequential execution
            test_config(),
        );

        let tasks = vec![
            SubTask {
                agent_id: AgentId::new(),
                prompt: "task1".into(),
                working_dir: String::new(),
            },
            SubTask {
                agent_id: AgentId::new(),
                prompt: "task2".into(),
                working_dir: String::new(),
            },
            SubTask {
                agent_id: AgentId::new(),
                prompt: "task3".into(),
                working_dir: String::new(),
            },
        ];

        let parent = SessionId::new();
        let results = runner.run_all(&parent, tasks).await;

        assert_eq!(results.len(), 3);
        for r in &results {
            assert!(r.success, "task should succeed");
            assert_eq!(r.exit_code, 0);
            assert!(!r.output.is_empty(), "should have output");
        }
    }

    #[test]
    fn aggregate_results_formats_summary() {
        let results = vec![
            SubTaskResult {
                agent_id: AgentId::new(),
                session_id: SessionId::new(),
                output: "done".into(),
                exit_code: 0,
                success: true,
            },
            SubTaskResult {
                agent_id: AgentId::new(),
                session_id: SessionId::new(),
                output: String::new(),
                exit_code: 1,
                success: false,
            },
        ];

        let summary = aggregate_results(&results);
        assert!(summary.contains("1/2 succeeded"));
        assert!(summary.contains("[OK]"));
        assert!(summary.contains("[FAILED]"));
        assert!(summary.contains("done"));
        assert!(summary.contains("(no output)"));
    }

    #[test]
    fn sub_task_result_tracks_success() {
        let ok = SubTaskResult {
            agent_id: AgentId::new(),
            session_id: SessionId::new(),
            output: "hello".into(),
            exit_code: 0,
            success: true,
        };
        assert!(ok.success);
        assert_eq!(ok.exit_code, 0);

        let fail = SubTaskResult {
            agent_id: AgentId::new(),
            session_id: SessionId::new(),
            output: String::new(),
            exit_code: 1,
            success: false,
        };
        assert!(!fail.success);
        assert_eq!(fail.exit_code, 1);
    }
}
