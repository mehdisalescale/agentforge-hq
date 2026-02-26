//! Process spawning for Claude Code CLI.
//! Phase 0 stub — minimal types for later implementation.

use serde::{Deserialize, Serialize};

/// Handle to a spawned process. Stub for Phase 1 process lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHandle {
    /// Placeholder for process/session identifier.
    pub id: Option<String>,
}

/// Parsed event from stream-json (or similar) output. Stub for Phase 1 streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamJsonEvent {
    /// Raw line or event kind placeholder.
    pub kind: Option<String>,
}

impl ProcessHandle {
    /// Stub constructor.
    pub fn stub() -> Self {
        Self { id: None }
    }
}

impl StreamJsonEvent {
    /// Stub constructor.
    pub fn stub() -> Self {
        Self { kind: None }
    }
}
