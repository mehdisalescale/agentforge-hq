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
