//! Spawn the Claude CLI (or configurable command) with working directory and env isolation.

use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::process::{Child, Command};

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("failed to spawn process: {0}")]
    Io(#[from] std::io::Error),

    #[error("command missing or empty")]
    CommandMissing,
}

/// Configuration for spawning the agent process (e.g. claude CLI).
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// Executable name or path (default: "claude").
    pub command: String,
    /// Extra args before prompt (e.g. --output-format stream-json --verbose).
    pub args_before_prompt: Vec<String>,
    /// Working directory; None = current dir.
    pub working_dir: Option<std::path::PathBuf>,
    /// Env vars to remove (e.g. CLAUDECODE to avoid nested session).
    pub env_remove: Vec<String>,
    /// Env vars to set (key, value).
    pub env_set: Vec<(String, String)>,
    /// Maximum time the process may run before being killed. Default: 5 minutes.
    pub timeout: Option<Duration>,
    /// Maximum concurrent agent processes. Default: 4.
    pub max_concurrent: usize,
    /// Maximum stdout bytes before truncation. Default: 10MB.
    pub max_output_bytes: usize,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            command: "claude".to_string(),
            args_before_prompt: vec![
                "--output-format".into(),
                "stream-json".into(),
                "--verbose".into(),
            ],
            working_dir: None,
            env_remove: vec![
                "CLAUDECODE".to_string(),
                "ANTHROPIC_API_KEY".to_string(),
            ],
            env_set: vec![],
            timeout: Some(Duration::from_secs(300)),
            max_concurrent: 4,
            max_output_bytes: 10 * 1024 * 1024,
        }
    }
}

/// Environment variable names for SpawnConfig overrides.
pub const ENV_CLI_COMMAND: &str = "FORGE_CLI_COMMAND";
pub const ENV_CLI_ARGS: &str = "FORGE_CLI_ARGS";

/// Build SpawnConfig from defaults and override with environment variables.
///
/// - `FORGE_CLI_COMMAND`: executable (e.g. "claude" or "/path/to/claude").
/// - `FORGE_CLI_ARGS`: space-separated args before prompt (e.g. "--output-format stream-json --verbose").
///
/// Backward compatible: if env vars are unset, defaults are used.
impl SpawnConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(cmd) = std::env::var(ENV_CLI_COMMAND) {
            if !cmd.is_empty() {
                config.command = cmd;
            }
        }
        if let Ok(args) = std::env::var(ENV_CLI_ARGS) {
            let parsed: Vec<String> = args
                .split_whitespace()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !parsed.is_empty() {
                config.args_before_prompt = parsed;
            }
        }
        if let Ok(max) = std::env::var("FORGE_MAX_CONCURRENT") {
            if let Ok(n) = max.parse::<usize>() {
                config.max_concurrent = n;
            }
        }
        config
    }

    /// Set working directory (e.g. from session.directory or run request). Chainable.
    pub fn with_working_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.working_dir = Some(dir.as_ref().to_path_buf());
        self
    }
}

/// Handle to a spawned process. Caller can take stdout and parse stream-json lines; kill or wait for exit.
/// On drop, the child process is killed to prevent zombies.
pub struct ProcessHandle {
    pub(crate) child: Child,
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if let Err(e) = self.child.start_kill() {
            tracing::warn!("failed to kill child process on drop: {}", e);
        }
    }
}

impl ProcessHandle {
    /// Take ownership of the child's stdout for reading stream-json lines. Returns None if already taken or not captured.
    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout.take()
    }

    /// Kill the process.
    pub fn kill(&mut self) -> std::io::Result<()> {
        self.child.start_kill()
    }

    /// Wait for the process to exit. Returns exit status.
    pub async fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait().await
    }

    /// Process ID if available.
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }
}

/// Spawn the configured command with prompt and optional session ID for resume.
/// Stdout is captured for stream-json parsing; stderr is inherited (or could be piped).
pub async fn spawn(
    config: &SpawnConfig,
    prompt: &str,
    session_id: Option<&str>,
) -> Result<ProcessHandle, SpawnError> {
    if config.command.is_empty() {
        return Err(SpawnError::CommandMissing);
    }

    let mut cmd = Command::new(&config.command);
    cmd.args(&config.args_before_prompt)
        .arg("-p")
        .arg(prompt)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if let Some(ref dir) = config.working_dir {
        cmd.current_dir(Path::new(dir));
    }
    for key in &config.env_remove {
        cmd.env_remove(key);
    }
    for (k, v) in &config.env_set {
        cmd.env(k, v);
    }
    if let Some(sid) = session_id {
        cmd.arg("--resume").arg(sid);
    }

    let child = cmd.spawn()?;
    Ok(ProcessHandle { child })
}

/// Enforces max concurrent process spawns.
pub struct SpawnLimiter {
    semaphore: Arc<tokio::sync::Semaphore>,
    pub config: SpawnConfig,
}

impl SpawnLimiter {
    pub fn new(config: SpawnConfig, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            config,
        }
    }

    /// Spawn with concurrency control. Waits for a permit if at max.
    pub async fn spawn_limited(
        &self,
        prompt: &str,
        session_id: Option<&str>,
    ) -> Result<(ProcessHandle, tokio::sync::OwnedSemaphorePermit), SpawnError> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| SpawnError::CommandMissing)?;
        let handle = spawn(&self.config, prompt, session_id).await?;
        Ok((handle, permit))
    }

    /// Number of available permits (inverse of active count).
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_line;
    use tokio::io::AsyncBufReadExt;

    /// Spawn with working_dir set; child runs in that directory (verify via pwd/cd).
    #[tokio::test]
    async fn spawn_uses_working_dir_when_set() {
        let temp = std::env::temp_dir().join("forge_process_wd_test");
        let _ = std::fs::create_dir_all(&temp);
        let config = SpawnConfig {
            command: if cfg!(target_os = "windows") {
                "cmd".to_string()
            } else {
                "sh".to_string()
            },
            args_before_prompt: if cfg!(target_os = "windows") {
                vec!["/c".into(), "cd".into()]
            } else {
                vec!["-c".into(), "pwd".into()]
            },
            working_dir: Some(temp.clone()),
            ..Default::default()
        };
        let mut handle = spawn(&config, "", None).await.unwrap();
        let mut stdout = handle.take_stdout().expect("stdout");
        let mut buf = String::new();
        let _ = tokio::io::AsyncReadExt::read_to_string(&mut stdout, &mut buf).await;
        let _ = handle.wait().await;
        let got = buf.trim();
        // Child CWD should be our temp dir (path may use / or \)
        assert!(
            got.contains("forge_process_wd_test"),
            "expected cwd output to contain forge_process_wd_test, got {:?}",
            got
        );
    }

    /// Spawn a command that prints one stream-json line and exits; parse the line.
    #[tokio::test]
    async fn spawn_echo_one_line_and_parse() {
        let line = r#"{"type":"result","result":"ok"}"#;
        let config = SpawnConfig {
            command: if cfg!(target_os = "windows") {
                "cmd".to_string()
            } else {
                "sh".to_string()
            },
            args_before_prompt: if cfg!(target_os = "windows") {
                vec!["/c".into(), format!("echo {}", line)]
            } else {
                vec!["-c".into(), format!("echo '{}'", line)]
            },
            ..Default::default()
        };
        // Spawn with prompt "" (echo doesn't use it)
        let mut handle = spawn(&config, "", None).await.unwrap();
        let mut stdout = handle.take_stdout().expect("stdout captured");
        let mut reader = tokio::io::BufReader::new(&mut stdout);
        let mut buf = String::new();
        let _ = reader.read_line(&mut buf).await;
        let _ = handle.wait().await;

        let parsed = parse_line(buf.trim()).unwrap();
        let ev = parsed.expect("one event");
        match ev {
            crate::stream_event::StreamJsonEvent::Result(p) => {
                assert_eq!(p.result.as_deref(), Some("ok"));
            }
            _ => panic!("expected Result event"),
        }
    }
}
