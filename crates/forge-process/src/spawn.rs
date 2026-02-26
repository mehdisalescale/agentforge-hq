//! Spawn the Claude CLI (or configurable command) with working directory and env isolation.

use std::path::Path;
use std::process::Stdio;
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
            env_remove: vec!["CLAUDECODE".to_string()],
            env_set: vec![],
        }
    }
}

/// Handle to a spawned process. Caller can take stdout and parse stream-json lines; kill or wait for exit.
pub struct ProcessHandle {
    pub(crate) child: Child,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_line;
    use tokio::io::AsyncBufReadExt;

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
