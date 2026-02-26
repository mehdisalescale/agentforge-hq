//! ForgeEvent enum and OutputKind for system-wide event streaming.
//!
//! Serde format: `#[serde(tag = "type", content = "data")]` so JSON is
//! `{"type": "VariantName", "data": {...}}`.

use crate::ids::{AgentId, SessionId, WorkflowId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ForgeEvent {
    // System lifecycle
    SystemStarted {
        version: String,
        timestamp: DateTime<Utc>,
    },
    SystemStopped {
        timestamp: DateTime<Utc>,
    },
    Heartbeat {
        timestamp: DateTime<Utc>,
    },

    // Agent lifecycle (Phase 0)
    AgentCreated {
        agent_id: AgentId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    AgentUpdated {
        agent_id: AgentId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    AgentDeleted {
        agent_id: AgentId,
        timestamp: DateTime<Utc>,
    },

    // Process lifecycle (Phase 1)
    ProcessStarted {
        session_id: SessionId,
        agent_id: AgentId,
        timestamp: DateTime<Utc>,
    },
    ProcessOutput {
        session_id: SessionId,
        kind: OutputKind,
        content: String,
        timestamp: DateTime<Utc>,
    },
    ProcessCompleted {
        session_id: SessionId,
        exit_code: i32,
        timestamp: DateTime<Utc>,
    },
    ProcessFailed {
        session_id: SessionId,
        error: String,
        timestamp: DateTime<Utc>,
    },

    // Session lifecycle (Phase 1)
    SessionCreated {
        session_id: SessionId,
        agent_id: AgentId,
        directory: String,
        timestamp: DateTime<Utc>,
    },
    SessionResumed {
        session_id: SessionId,
        timestamp: DateTime<Utc>,
    },

    // Workflow lifecycle (Phase 2)
    WorkflowStarted {
        workflow_id: WorkflowId,
        timestamp: DateTime<Utc>,
    },
    WorkflowStepCompleted {
        workflow_id: WorkflowId,
        step: u32,
        timestamp: DateTime<Utc>,
    },
    WorkflowCompleted {
        workflow_id: WorkflowId,
        timestamp: DateTime<Utc>,
    },
    WorkflowFailed {
        workflow_id: WorkflowId,
        error: String,
        timestamp: DateTime<Utc>,
    },

    // Safety events (Phase 4)
    CircuitBreakerTripped {
        agent_id: AgentId,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    BudgetWarning {
        current_cost: f64,
        limit: f64,
        timestamp: DateTime<Utc>,
    },
    BudgetExceeded {
        current_cost: f64,
        limit: f64,
        timestamp: DateTime<Utc>,
    },

    // Generic error
    Error {
        message: String,
        context: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputKind {
    Assistant,
    ToolUse,
    ToolResult,
    Thinking,
    Result,
}
