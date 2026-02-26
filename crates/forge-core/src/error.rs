//! Forge error hierarchy and result type.

use crate::ids::{AgentId, SessionId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),

    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Event bus error: {0}")]
    EventBus(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Internal(String),
}

pub type ForgeResult<T> = Result<T, ForgeError>;
