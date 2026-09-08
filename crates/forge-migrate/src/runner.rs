//! Migration runner: executes tasks by spawning Claude Code agents in git worktrees.

use std::path::{Path, PathBuf};

use forge_core::ForgeError;
use forge_process::{SpawnConfig, spawn};
use tokio::io::AsyncBufReadExt;
use tracing::{error, info, warn};

use crate::task::{MigrationTask, TaskStatus};

/// Result of a single migration task execution.
#[derive(Debug)]
pub struct TaskResult {
    /// The file that was migrated.
    pub file_path: String,
    /// Updated status after execution.
    pub status: TaskStatus,
    /// Worktree path where changes were made (if any).
    pub worktree_path: Option<PathBuf>,
    /// Raw stdout output from the Claude Code agent.
    pub agent_output: String,
}

/// Configuration for the migration runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Path to the iOS repo to modernize.
    pub repo_path: PathBuf,
    /// Whether to use git worktrees for isolation (recommended).
    pub use_worktrees: bool,
    /// SpawnConfig override for the Claude CLI. Uses default if None.
    pub spawn_config: Option<SpawnConfig>,
    /// Maximum output bytes to capture per agent. Default: 2MB.
    pub max_output_bytes: usize,
}

impl RunnerConfig {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            use_worktrees: true,
            spawn_config: None,
            max_output_bytes: 2 * 1024 * 1024,
        }
    }
}

/// Orchestrates migration task execution: worktree creation, agent spawning, result collection.
pub struct MigrationRunner {
    config: RunnerConfig,
}

impl MigrationRunner {
    pub fn new(config: RunnerConfig) -> Self {
        Self { config }
    }

    /// Execute a single migration task. Creates a worktree, spawns a Claude Code agent,
    /// waits for completion, and returns the result.
    pub async fn execute_task(&self, task: &mut MigrationTask) -> Result<TaskResult, ForgeError> {
        // Skip tasks that are already completed, skipped, or info-only.
        if matches!(
            task.status,
            TaskStatus::Completed { .. } | TaskStatus::Skipped { .. }
        ) {
            return Ok(TaskResult {
                file_path: task.file_path.clone(),
                status: task.status.clone(),
                worktree_path: None,
                agent_output: String::new(),
            });
        }

        task.status = TaskStatus::Running;
        let session_id = uuid::Uuid::new_v4().to_string();

        // Determine working directory: worktree or repo root.
        let working_dir = if self.config.use_worktrees {
            match forge_git::create_worktree(&self.config.repo_path, &session_id) {
                Ok(wt_path) => {
                    info!(
                        file = %task.file_path,
                        worktree = %wt_path.display(),
                        "created worktree for migration task"
                    );
                    wt_path
                }
                Err(e) => {
                    warn!(
                        file = %task.file_path,
                        error = %e,
                        "failed to create worktree, falling back to repo root"
                    );
                    self.config.repo_path.clone()
                }
            }
        } else {
            self.config.repo_path.clone()
        };

        // Build spawn config.
        let spawn_config = self
            .config
            .spawn_config
            .clone()
            .unwrap_or_else(SpawnConfig::from_env)
            .with_working_dir(&working_dir);

        // Spawn the Claude Code agent with the task's prompt.
        let result = self
            .run_agent(&spawn_config, &task.prompt, &task.file_path)
            .await;

        // Clean up worktree if we created one (but only on failure; on success we keep it
        // so the user can review changes and merge).
        let worktree_path = if self.config.use_worktrees && working_dir != self.config.repo_path {
            Some(working_dir.clone())
        } else {
            None
        };

        match result {
            Ok(output) => {
                task.status = TaskStatus::Completed {
                    summary: truncate_summary(&output, 500),
                };
                Ok(TaskResult {
                    file_path: task.file_path.clone(),
                    status: task.status.clone(),
                    worktree_path,
                    agent_output: output,
                })
            }
            Err(e) => {
                let error_msg = e.to_string();
                task.status = TaskStatus::Failed {
                    error: error_msg.clone(),
                };
                // Clean up worktree on failure.
                if self.config.use_worktrees {
                    let _ = forge_git::remove_worktree(&self.config.repo_path, &session_id);
                }
                Ok(TaskResult {
                    file_path: task.file_path.clone(),
                    status: task.status.clone(),
                    worktree_path: None,
                    agent_output: error_msg,
                })
            }
        }
    }

    /// Execute all tasks in a migration plan sequentially.
    /// Returns results for each task (including skipped ones).
    pub async fn execute_all(
        &self,
        tasks: &mut [MigrationTask],
    ) -> Vec<TaskResult> {
        let mut results = Vec::with_capacity(tasks.len());

        for (i, task) in tasks.iter_mut().enumerate() {
            info!(
                task = i + 1,
                total = results.capacity(),
                file = %task.file_path,
                priority = %task.priority,
                findings = task.findings.len(),
                "executing migration task"
            );

            match self.execute_task(task).await {
                Ok(result) => {
                    info!(
                        file = %result.file_path,
                        status = %result.status,
                        "task completed"
                    );
                    results.push(result);
                }
                Err(e) => {
                    error!(
                        file = %task.file_path,
                        error = %e,
                        "task execution failed"
                    );
                    task.status = TaskStatus::Failed {
                        error: e.to_string(),
                    };
                    results.push(TaskResult {
                        file_path: task.file_path.clone(),
                        status: task.status.clone(),
                        worktree_path: None,
                        agent_output: e.to_string(),
                    });
                }
            }
        }

        results
    }

    /// Spawn a Claude Code agent and collect its output.
    async fn run_agent(
        &self,
        spawn_config: &SpawnConfig,
        prompt: &str,
        file_path: &str,
    ) -> Result<String, ForgeError> {
        let mut handle = spawn(spawn_config, prompt, None)
            .await
            .map_err(|e| ForgeError::Process(format!("failed to spawn agent for {}: {}", file_path, e)))?;

        let stdout = handle
            .take_stdout()
            .ok_or_else(|| ForgeError::Process("agent stdout not available".into()))?;

        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut output = String::new();
        let max_bytes = self.config.max_output_bytes;

        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| ForgeError::Process(format!("reading agent output: {}", e)))?
        {
            if output.len() + line.len() > max_bytes {
                warn!(
                    file = file_path,
                    "agent output exceeded max bytes, truncating"
                );
                break;
            }
            output.push_str(&line);
            output.push('\n');
        }

        let exit_status = handle
            .wait()
            .await
            .map_err(|e| ForgeError::Process(format!("waiting for agent: {}", e)))?;

        if !exit_status.success() {
            let code = exit_status.code().unwrap_or(-1);
            return Err(ForgeError::Process(format!(
                "agent for {} exited with code {}",
                file_path, code
            )));
        }

        Ok(output)
    }

    /// Clean up all worktrees created during migration.
    pub fn cleanup_worktrees(&self, results: &[TaskResult]) {
        for result in results {
            if let Some(ref wt_path) = result.worktree_path {
                // Extract session ID from worktree path (last component).
                if let Some(session_id) = wt_path.file_name().and_then(|n| n.to_str()) {
                    info!(session_id, "removing migration worktree");
                    let _ = forge_git::remove_worktree(&self.config.repo_path, session_id);
                }
            }
        }
    }
}

/// Truncate a string to at most `max_len` characters, appending "..." if truncated.
fn truncate_summary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated = s[..max_len].to_string();
        truncated.push_str("...");
        truncated
    }
}

/// Validate that a repo path is a git repository before starting migration.
pub fn validate_repo(repo_path: &Path) -> Result<(), ForgeError> {
    if !repo_path.is_dir() {
        return Err(ForgeError::Validation(format!(
            "repo path does not exist or is not a directory: {}",
            repo_path.display()
        )));
    }
    if !forge_git::is_git_repo(repo_path) {
        return Err(ForgeError::Validation(format!(
            "repo path is not a git repository: {}",
            repo_path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate_summary("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(100);
        let result = truncate_summary(&long, 20);
        assert_eq!(result.len(), 23); // 20 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn validate_repo_rejects_missing_dir() {
        let result = validate_repo(Path::new("/nonexistent/path/to/repo"));
        assert!(result.is_err());
    }

    #[test]
    fn runner_config_defaults() {
        let config = RunnerConfig::new("/tmp/test-repo");
        assert!(config.use_worktrees);
        assert!(config.spawn_config.is_none());
        assert_eq!(config.max_output_bytes, 2 * 1024 * 1024);
    }
}
