use crate::model::Persona;
use forge_agent::model::NewAgent;
use forge_agent::model::DEFAULT_MODEL;
use serde_json::json;

pub struct HireConfig {
    pub default_model: String,
    pub max_turns: Option<u32>,
}

impl Default for HireConfig {
    fn default() -> Self {
        Self {
            default_model: DEFAULT_MODEL.to_string(),
            max_turns: Some(64),
        }
    }
}

pub struct PersonaMapper {
    config: HireConfig,
}

impl PersonaMapper {
    pub fn new(config: HireConfig) -> Self {
        Self { config }
    }

    pub fn to_new_agent(&self, persona: &Persona) -> NewAgent {
        let mut sections = Vec::new();
        sections.push(format!(
            "# Persona: {} ({})",
            persona.name, persona.division_slug
        ));
        sections.push(String::new());
        sections.push(format!("Summary: {}", persona.short_description));

        if let Some(ref p) = persona.personality {
            sections.push(String::new());
            sections.push("## Personality".into());
            sections.push(p.clone());
        }
        if let Some(ref d) = persona.deliverables {
            sections.push(String::new());
            sections.push("## Deliverables".into());
            sections.push(d.clone());
        }
        if let Some(ref m) = persona.success_metrics {
            sections.push(String::new());
            sections.push("## Success Metrics".into());
            sections.push(m.clone());
        }
        if let Some(ref w) = persona.workflow {
            sections.push(String::new());
            sections.push("## Workflow".into());
            sections.push(w.clone());
        }
        if !persona.tags.is_empty() {
            sections.push(String::new());
            sections.push("## Tags".into());
            sections.push(persona.tags.join(", "));
        }

        let system_prompt = sections.join("\n");

        let persona_meta = json!({
            "id": persona.id.0.to_string(),
            "division": persona.division_slug,
            "slug": persona.slug,
            "source_file": persona.source_file,
            "tags": persona.tags,
        });

        let config = json!({
            "persona": persona_meta,
        });

        NewAgent {
            name: persona.name.clone(),
            model: Some(self.config.default_model.clone()),
            system_prompt: Some(system_prompt),
            allowed_tools: None,
            max_turns: self.config.max_turns,
            use_max: self.config.max_turns.map(|_| true),
            preset: None,
            config: Some(config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Persona, PersonaId};
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_persona() -> Persona {
        let now = Utc::now();
        Persona {
            id: PersonaId(Uuid::new_v4()),
            division_slug: "marketing".into(),
            slug: "marketing-growth-hacker".into(),
            name: "Marketing Growth Hacker".into(),
            short_description: "Drives rapid experimentation".into(),
            personality: Some("Curious and data-driven.".into()),
            deliverables: None,
            success_metrics: None,
            workflow: None,
            tags: vec!["marketing".into(), "growth".into()],
            source_file: "marketing/marketing-growth-hacker.md".into(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn maps_persona_to_new_agent() {
        let mapper = PersonaMapper::new(HireConfig::default());
        let persona = sample_persona();
        let agent = mapper.to_new_agent(&persona);
        assert_eq!(agent.name, persona.name);
        assert!(agent.system_prompt.as_ref().unwrap().contains("Marketing Growth Hacker"));
        assert!(agent.config.is_some());
    }
}

