use crate::model::{NewAgent, UpdateAgent};
use forge_core::error::{ForgeError, ForgeResult};

const MAX_NAME_LENGTH: usize = 100;
const MAX_SYSTEM_PROMPT_LENGTH: usize = 102_400; // 100KB

pub fn validate_new_agent(agent: &NewAgent) -> ForgeResult<()> {
    if agent.name.trim().is_empty() {
        return Err(ForgeError::Validation("Agent name cannot be empty".into()));
    }
    if agent.name.len() > MAX_NAME_LENGTH {
        return Err(ForgeError::Validation(format!(
            "Agent name must be {} characters or fewer",
            MAX_NAME_LENGTH
        )));
    }
    if let Some(ref prompt) = agent.system_prompt {
        if prompt.len() > MAX_SYSTEM_PROMPT_LENGTH {
            return Err(ForgeError::Validation(
                "System prompt must be 100KB or fewer".into(),
            ));
        }
    }
    Ok(())
}

/// Validates an UpdateAgent. If name is Some, it must be non-empty and within length.
pub fn validate_update_agent(agent: &UpdateAgent) -> ForgeResult<()> {
    if let Some(ref name) = agent.name {
        if name.trim().is_empty() {
            return Err(ForgeError::Validation(
                "Agent name cannot be empty".into(),
            ));
        }
        if name.len() > MAX_NAME_LENGTH {
            return Err(ForgeError::Validation(format!(
                "Agent name must be {} characters or fewer",
                MAX_NAME_LENGTH
            )));
        }
    }
    if let Some(Some(ref prompt)) = agent.system_prompt {
        if prompt.len() > MAX_SYSTEM_PROMPT_LENGTH {
            return Err(ForgeError::Validation(
                "System prompt must be 100KB or fewer".into(),
            ));
        }
    }
    Ok(())
}
