# Agent C2: ProcessBackend Trait + Claude Adapter

> You are Agent C2. Your job: define a `ProcessBackend` trait in forge-process and extract existing Claude CLI spawning into a `ClaudeBackend` adapter.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
crates/forge-process/src/spawn.rs                   — current spawn logic (SpawnConfig, spawn function)
crates/forge-process/src/lib.rs                     — crate exports
crates/forge-process/Cargo.toml                     — current deps
```

## Step 2: Create ProcessBackend Trait

Create `crates/forge-process/src/backend.rs`:

```rust
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Health status of a backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendHealth {
    Healthy,
    Degraded(String),
    Unavailable(String),
}

/// What a backend can do.
#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supported_models: Vec<String>,
}

/// Configuration for spawning a process via a backend.
#[derive(Debug, Clone)]
pub struct BackendSpawnConfig {
    pub prompt: String,
    pub working_dir: String,
    pub model: Option<String>,
    pub max_turns: Option<u32>,
    pub allowed_tools: Option<Vec<String>>,
    pub system_prompt: Option<String>,
    pub resume_session_id: Option<String>,
    pub env: HashMap<String, String>,
}

/// Handle to a running backend process.
/// Wraps the existing ProcessHandle.
pub use crate::spawn::ProcessHandle;

/// Trait that all execution backends must implement.
pub trait ProcessBackend: Send + Sync {
    /// Human-readable name (e.g. "claude", "hermes", "openclaw").
    fn name(&self) -> &str;

    /// Check if the backend is available and healthy.
    fn health_check(&self) -> Pin<Box<dyn Future<Output = BackendHealth> + Send + '_>>;

    /// Report capabilities.
    fn capabilities(&self) -> BackendCapabilities;

    /// Spawn a process. Returns a handle for reading output.
    fn spawn(
        &self,
        config: &BackendSpawnConfig,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessHandle, crate::spawn::SpawnError>> + Send + '_>>;
}

/// Registry of available backends.
pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn ProcessBackend>>,
    default_backend: String,
}

impl BackendRegistry {
    pub fn new(default_backend: &str) -> Self {
        Self {
            backends: HashMap::new(),
            default_backend: default_backend.to_string(),
        }
    }

    pub fn register(&mut self, backend: Box<dyn ProcessBackend>) {
        let name = backend.name().to_string();
        self.backends.insert(name, backend);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ProcessBackend> {
        self.backends.get(name).map(|b| b.as_ref())
    }

    pub fn default(&self) -> Option<&dyn ProcessBackend> {
        self.get(&self.default_backend)
    }

    pub fn available_backends(&self) -> Vec<&str> {
        self.backends.keys().map(|s| s.as_str()).collect()
    }
}
```

## Step 3: Create ClaudeBackend Adapter

Create `crates/forge-process/src/claude_backend.rs`:

```rust
use crate::backend::*;
use crate::spawn::{ProcessHandle, SpawnConfig, SpawnError, spawn};
use std::future::Future;
use std::pin::Pin;

/// Claude CLI backend adapter.
///
/// Delegates to the existing `spawn()` function, adapting
/// BackendSpawnConfig to SpawnConfig.
pub struct ClaudeBackend {
    base_config: SpawnConfig,
}

impl ClaudeBackend {
    pub fn new() -> Self {
        Self {
            base_config: SpawnConfig::from_env(),
        }
    }

    pub fn with_config(config: SpawnConfig) -> Self {
        Self {
            base_config: config,
        }
    }
}

impl Default for ClaudeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessBackend for ClaudeBackend {
    fn name(&self) -> &str {
        "claude"
    }

    fn health_check(&self) -> Pin<Box<dyn Future<Output = BackendHealth> + Send + '_>> {
        Box::pin(async {
            // Check if the claude command exists
            match tokio::process::Command::new(&self.base_config.command)
                .arg("--version")
                .output()
                .await
            {
                Ok(output) if output.status.success() => BackendHealth::Healthy,
                Ok(_) => BackendHealth::Degraded("claude --version returned non-zero".into()),
                Err(e) => BackendHealth::Unavailable(format!("claude not found: {}", e)),
            }
        })
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_streaming: true,
            supports_tools: true,
            supported_models: vec![
                "claude-opus-4-6".into(),
                "claude-sonnet-4-6".into(),
                "claude-haiku-4-5".into(),
            ],
        }
    }

    fn spawn(
        &self,
        config: &BackendSpawnConfig,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessHandle, SpawnError>> + Send + '_>> {
        let spawn_config = self.base_config.clone()
            .with_working_dir(&config.working_dir);
        let prompt = config.prompt.clone();
        let resume = config.resume_session_id.clone();

        Box::pin(async move {
            spawn(&spawn_config, &prompt, resume.as_deref()).await
        })
    }
}
```

## Step 4: Register Modules

Add to `crates/forge-process/src/lib.rs`:
```rust
pub mod backend;
pub mod claude_backend;
```

## Step 5: Write Tests

Add tests in `crates/forge-process/src/backend.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend {
        health: BackendHealth,
    }

    impl ProcessBackend for MockBackend {
        fn name(&self) -> &str { "mock" }
        fn health_check(&self) -> Pin<Box<dyn Future<Output = BackendHealth> + Send + '_>> {
            let h = self.health.clone();
            Box::pin(async move { h })
        }
        fn capabilities(&self) -> BackendCapabilities {
            BackendCapabilities {
                supports_streaming: false,
                supports_tools: false,
                supported_models: vec!["mock-v1".into()],
            }
        }
        fn spawn(&self, _config: &BackendSpawnConfig)
            -> Pin<Box<dyn Future<Output = Result<ProcessHandle, crate::spawn::SpawnError>> + Send + '_>>
        {
            Box::pin(async { Err(crate::spawn::SpawnError::CommandMissing) })
        }
    }

    #[test]
    fn registry_registers_and_queries() {
        let mut reg = BackendRegistry::new("mock");
        reg.register(Box::new(MockBackend { health: BackendHealth::Healthy }));
        assert!(reg.get("mock").is_some());
        assert_eq!(reg.get("mock").unwrap().name(), "mock");
    }

    #[test]
    fn registry_returns_none_for_unknown() {
        let reg = BackendRegistry::new("claude");
        assert!(reg.get("hermes").is_none());
    }

    #[test]
    fn registry_lists_available_backends() {
        let mut reg = BackendRegistry::new("mock");
        reg.register(Box::new(MockBackend { health: BackendHealth::Healthy }));
        let backends = reg.available_backends();
        assert!(backends.contains(&"mock"));
    }

    #[test]
    fn registry_default_returns_correct_backend() {
        let mut reg = BackendRegistry::new("mock");
        reg.register(Box::new(MockBackend { health: BackendHealth::Healthy }));
        assert!(reg.default().is_some());
        assert_eq!(reg.default().unwrap().name(), "mock");
    }

    #[tokio::test]
    async fn mock_backend_health_check() {
        let backend = MockBackend { health: BackendHealth::Healthy };
        assert_eq!(backend.health_check().await, BackendHealth::Healthy);
    }

    #[test]
    fn mock_backend_capabilities() {
        let backend = MockBackend { health: BackendHealth::Healthy };
        let caps = backend.capabilities();
        assert!(!caps.supports_streaming);
        assert_eq!(caps.supported_models, vec!["mock-v1".to_string()]);
    }
}
```

Add tests in `crates/forge-process/src/claude_backend.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_backend_name() {
        let backend = ClaudeBackend::new();
        assert_eq!(backend.name(), "claude");
    }

    #[test]
    fn claude_backend_capabilities() {
        let backend = ClaudeBackend::new();
        let caps = backend.capabilities();
        assert!(caps.supports_streaming);
        assert!(caps.supports_tools);
        assert!(!caps.supported_models.is_empty());
    }
}
```

## Step 6: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-process -- backend 2>&1        # backend tests pass
cargo test -p forge-process -- claude_backend 2>&1 # claude backend tests pass
cargo test -p forge-process 2>&1                   # ALL forge-process tests pass
```

## Rules

- Create `crates/forge-process/src/backend.rs` (new)
- Create `crates/forge-process/src/claude_backend.rs` (new)
- Add 2 `pub mod` lines to `crates/forge-process/src/lib.rs`
- Do NOT modify `spawn.rs` — ClaudeBackend delegates TO it, not replaces it
- Do NOT modify middleware.rs (backend routing in middleware is a future story)
- Do NOT touch forge-api, forge-db, forge-safety, or frontend
- Do NOT modify any existing tests
- Commit with: `feat(process): add ProcessBackend trait and ClaudeBackend adapter`

## Report
```
STATUS: done | blocked
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
TRAIT_METHODS: [list ProcessBackend methods]
ISSUES: [any]
```
