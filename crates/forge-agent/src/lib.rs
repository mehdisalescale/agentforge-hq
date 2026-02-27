//! Agent model, presets, and validation for Claude Forge.

pub mod model;
pub mod preset;
pub mod validation;

pub use model::{Agent, NewAgent, UpdateAgent, DEFAULT_MODEL};
pub use preset::{AgentPreset, PresetDefaults};
pub use validation::{validate_new_agent, validate_update_agent};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NewAgent;

    fn default_new_agent() -> NewAgent {
        NewAgent {
            name: "Default".into(),
            model: None,
            system_prompt: None,
            allowed_tools: None,
            max_turns: None,
            use_max: None,
            preset: None,
            config: None,
        }
    }

    #[test]
    fn all_presets_have_non_empty_system_prompt() {
        for preset in AgentPreset::all() {
            let defaults = preset.defaults();
            assert!(
                !defaults.system_prompt.is_empty(),
                "{:?} has empty prompt",
                preset
            );
        }
    }

    #[test]
    fn validate_rejects_empty_name() {
        let agent = NewAgent {
            name: "".into(),
            ..default_new_agent()
        };
        assert!(matches!(
            validate_new_agent(&agent),
            Err(forge_core::ForgeError::Validation(_))
        ));
    }

    #[test]
    fn validate_rejects_long_name() {
        let agent = NewAgent {
            name: "x".repeat(101),
            ..default_new_agent()
        };
        assert!(matches!(
            validate_new_agent(&agent),
            Err(forge_core::ForgeError::Validation(_))
        ));
    }

    #[test]
    fn validate_accepts_valid_agent() {
        let agent = NewAgent {
            name: "MyAgent".into(),
            ..default_new_agent()
        };
        assert!(validate_new_agent(&agent).is_ok());
    }

    #[test]
    fn validate_rejects_name_with_spaces() {
        let agent = NewAgent {
            name: "My Agent".into(),
            ..default_new_agent()
        };
        assert!(matches!(
            validate_new_agent(&agent),
            Err(forge_core::ForgeError::Validation(_))
        ));
    }

    #[test]
    fn new_agent_uses_default_model() {
        assert_eq!(DEFAULT_MODEL, "claude-sonnet-4-20250514");
        let defaults = AgentPreset::CodeWriter.defaults();
        assert_eq!(defaults.model, DEFAULT_MODEL);
    }

    #[test]
    fn preset_code_writer_allows_all_tools() {
        let defaults = AgentPreset::CodeWriter.defaults();
        assert!(defaults.allowed_tools.is_none());
    }

    #[test]
    fn preset_reviewer_has_read_only_tools() {
        let defaults = AgentPreset::Reviewer.defaults();
        assert!(defaults.allowed_tools.is_some());
        let tools = defaults.allowed_tools.unwrap();
        assert!(tools.contains(&"Read".to_string()));
        assert!(!tools.contains(&"Write".to_string()));
    }
}
