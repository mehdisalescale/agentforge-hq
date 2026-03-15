//! AgentConfigurator: generates per-agent CLAUDE.md and hooks.json files.
//!
//! Replaces runtime SkillInjection and TaskTypeDetection middlewares with
//! file-based configuration written into the agent's working directory
//! before the CLI is spawned.

use std::sync::Arc;

use forge_db::{AgentRepo, CompanyRepo, GoalRepo, OrgPositionRepo, PersonaRepo, SkillRepo};

/// Generates per-agent configuration files (CLAUDE.md + hooks.json) before spawn.
pub struct AgentConfigurator {
    pub skill_repo: Arc<SkillRepo>,
    pub company_repo: Arc<CompanyRepo>,
    pub org_position_repo: Arc<OrgPositionRepo>,
    pub goal_repo: Arc<GoalRepo>,
    pub persona_repo: Arc<PersonaRepo>,
    pub agent_repo: Arc<AgentRepo>,
}

impl AgentConfigurator {
    /// Generate CLAUDE.md content for an agent about to execute a prompt.
    pub fn generate_claude_md(&self, agent_id: &str, prompt: &str) -> String {
        let mut sections = Vec::new();

        sections.push("# AgentForge Agent Configuration\n".to_string());

        // 1. Persona identity — look up agent, then find persona via org position context
        if let Ok(agent_uuid) = uuid::Uuid::parse_str(agent_id) {
            let aid = forge_core::ids::AgentId(agent_uuid);
            if let Ok(agent) = self.agent_repo.get(&aid) {
                sections.push(format!("## Agent: {}\n", agent.name));
                if let Some(ref sp) = agent.system_prompt {
                    sections.push(format!("{}\n", sp));
                }
            }
        }

        // 2. Company context — find company via org positions
        if let Some(company_id) = self.find_company_id(agent_id) {
            if let Ok(company) = self.company_repo.get(&company_id) {
                sections.push("## Company Context\n".to_string());
                sections.push(format!("- **Company:** {}", company.name));
                if let Some(ref mission) = company.mission {
                    sections.push(format!("- **Mission:** {}", mission));
                }
                if let Some(limit) = company.budget_limit {
                    let remaining = limit - company.budget_used;
                    sections.push(format!(
                        "- **Budget:** ${:.2} remaining (${:.2} of ${:.2} used)",
                        remaining, company.budget_used, limit
                    ));
                }
                sections.push(String::new());

                // 3. Active goals for this company
                if let Ok(goals) = self.goal_repo.list_by_company(&company_id) {
                    let active: Vec<_> = goals
                        .iter()
                        .filter(|g| g.status == "planned" || g.status == "in_progress")
                        .collect();
                    if !active.is_empty() {
                        sections.push("## Active Goals\n".to_string());
                        for goal in &active {
                            let desc = goal
                                .description
                                .as_deref()
                                .unwrap_or("(no description)");
                            sections.push(format!(
                                "- [{}] **{}**: {}",
                                goal.status, goal.title, desc
                            ));
                        }
                        sections.push(String::new());
                    }
                }
            }
        }

        // 4. Relevant skills — keyword matching + task type routing
        let mut skill_sections = Vec::new();

        // 4a. Keyword-based skill matching (same logic as SkillInjectionMiddleware)
        let keywords = Self::extract_keywords(prompt);
        if let Ok(skills) = self.skill_repo.list() {
            for skill in &skills {
                if let Some(ref tags_json) = skill.parameters_json {
                    if let Ok(tags) = serde_json::from_str::<Vec<String>>(tags_json) {
                        let matched = keywords
                            .iter()
                            .any(|kw| tags.iter().any(|tag| tag.to_lowercase().contains(kw)));
                        if matched {
                            skill_sections.push(format!(
                                "### Skill: {}\n\n{}",
                                skill.name, skill.content
                            ));
                        }
                    }
                }
            }

            // 4b. Task type routing (same logic as TaskTypeDetectionMiddleware)
            let detector = forge_process::task_type::TaskTypeDetector::new();
            let task_type = detector.classify(prompt);
            let router = forge_process::skill_router::SkillRouter::new();
            let skill_names = router.skills_for(task_type);

            for skill in &skills {
                if skill_names.iter().any(|n| n == &skill.name) {
                    // Avoid duplicates
                    let header = format!("### Methodology: {}", skill.name);
                    if !skill_sections.iter().any(|s| s.contains(&skill.name)) {
                        skill_sections.push(format!("{}\n\n{}", header, skill.content));
                    }
                }
            }
        }

        if !skill_sections.is_empty() {
            sections.push("## Relevant Skills & Methodologies\n".to_string());
            sections.push(skill_sections.join("\n\n"));
            sections.push(String::new());
        }

        // 5. Rules
        sections.push("## Rules\n".to_string());
        sections.push("- Stay within your role scope and assigned deliverables.".to_string());
        sections.push("- Respect budget constraints — do not exceed allocated limits.".to_string());
        sections
            .push("- Request approval for actions above your authority threshold.".to_string());
        sections.push("- Report progress and blockers clearly.".to_string());

        sections.join("\n")
    }

    /// Generate hooks.json content that reports back to AgentForge.
    pub fn generate_hooks_json(&self, session_id: &str, port: u16) -> String {
        serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "type": "command",
                    "command": format!(
                        "curl -sf http://127.0.0.1:{}/api/v1/hooks/pre-tool -H 'Content-Type: application/json' -d '{{\"session_id\":\"{}\",\"tool_name\":\"$TOOL_NAME\"}}'",
                        port, session_id
                    ),
                    "timeout": 3000
                }],
                "PostToolUse": [{
                    "type": "command",
                    "command": format!(
                        "curl -sf http://127.0.0.1:{}/api/v1/hooks/post-tool -H 'Content-Type: application/json' -d '{{\"session_id\":\"{}\",\"tool_name\":\"$TOOL_NAME\"}}'",
                        port, session_id
                    ),
                    "timeout": 3000
                }],
                "Stop": [{
                    "type": "command",
                    "command": format!(
                        "curl -sf http://127.0.0.1:{}/api/v1/hooks/stop -H 'Content-Type: application/json' -d '{{\"session_id\":\"{}\"}}'",
                        port, session_id
                    ),
                    "timeout": 5000
                }]
            }
        })
        .to_string()
    }

    /// Write both files into a directory (worktree or temp dir).
    pub fn configure_workspace(
        &self,
        agent_id: &str,
        prompt: &str,
        session_id: &str,
        dir: &str,
    ) -> std::io::Result<()> {
        let claude_md = self.generate_claude_md(agent_id, prompt);
        let hooks_json = self.generate_hooks_json(session_id, 4173);

        // Write {dir}/CLAUDE.md
        std::fs::write(format!("{}/CLAUDE.md", dir), &claude_md)?;

        // Write {dir}/.claude/hooks.json
        let claude_dir = format!("{}/.claude", dir);
        std::fs::create_dir_all(&claude_dir)?;
        std::fs::write(format!("{}/hooks.json", claude_dir), &hooks_json)?;

        Ok(())
    }

    /// Find the company_id for a given agent by looking up org_positions.
    fn find_company_id(&self, agent_id: &str) -> Option<String> {
        if let Ok(companies) = self.company_repo.list() {
            for company in &companies {
                if let Ok(positions) = self.org_position_repo.list_by_company(&company.id) {
                    if positions
                        .iter()
                        .any(|p| p.agent_id.as_deref() == Some(agent_id))
                    {
                        return Some(company.id.clone());
                    }
                }
            }
        }
        None
    }

    /// Extract keywords from a prompt: lowercase, split on whitespace, filter short words.
    fn extract_keywords(prompt: &str) -> Vec<String> {
        prompt
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|w| w.len() > 2)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_db::{DbPool, Migrator};
    use std::sync::Arc;

    fn setup_configurator() -> AgentConfigurator {
        let db = DbPool::in_memory().unwrap();
        let conn_arc = db.conn_arc();
        {
            let conn = conn_arc.lock().unwrap();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        AgentConfigurator {
            skill_repo: Arc::new(SkillRepo::new(Arc::clone(&conn_arc))),
            company_repo: Arc::new(CompanyRepo::new(Arc::clone(&conn_arc))),
            org_position_repo: Arc::new(OrgPositionRepo::new(Arc::clone(&conn_arc))),
            goal_repo: Arc::new(GoalRepo::new(Arc::clone(&conn_arc))),
            persona_repo: Arc::new(PersonaRepo::new(Arc::clone(&conn_arc))),
            agent_repo: Arc::new(AgentRepo::new(Arc::clone(&conn_arc))),
        }
    }

    #[test]
    fn generate_claude_md_includes_agent_name() {
        let configurator = setup_configurator();

        // Create an agent
        let agent = configurator
            .agent_repo
            .create(&forge_agent::model::NewAgent {
                name: "TestPersonaAgent".into(),
                model: None,
                system_prompt: Some("You are a marketing specialist.".into()),
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();

        let md = configurator.generate_claude_md(&agent.id.0.to_string(), "write a blog post");
        assert!(md.contains("TestPersonaAgent"));
        assert!(md.contains("marketing specialist"));
    }

    #[test]
    fn generate_claude_md_includes_matched_skills() {
        let configurator = setup_configurator();

        // Insert a skill with tags
        configurator
            .skill_repo
            .upsert(&forge_db::repos::skills::UpsertSkill {
                id: "rust-skill".into(),
                name: "rust-skill".into(),
                description: Some("Rust development".into()),
                category: Some("development".into()),
                subcategory: None,
                content: "Use cargo and clippy.".into(),
                source_repo: Some("builtin".into()),
                parameters_json: Some(r#"["rust","cargo"]"#.into()),
                examples_json: None,
            })
            .unwrap();

        let md = configurator.generate_claude_md("nonexistent-id", "write rust code with cargo");
        assert!(md.contains("rust-skill"));
        assert!(md.contains("Use cargo and clippy."));
    }

    #[test]
    fn generate_hooks_json_is_valid_json() {
        let configurator = setup_configurator();
        let hooks = configurator.generate_hooks_json("sess-123", 4173);

        let parsed: serde_json::Value = serde_json::from_str(&hooks).unwrap();
        let hooks_obj = parsed.get("hooks").unwrap();
        assert!(hooks_obj.get("PreToolUse").is_some());
        assert!(hooks_obj.get("PostToolUse").is_some());
        assert!(hooks_obj.get("Stop").is_some());

        // Verify session_id is embedded
        assert!(hooks.contains("sess-123"));
    }

    #[test]
    fn configure_workspace_writes_files() {
        let configurator = setup_configurator();
        let dir = std::env::temp_dir().join(format!(
            "forge-configurator-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        configurator
            .configure_workspace("agent-1", "test prompt", "sess-1", dir.to_str().unwrap())
            .unwrap();

        // Assert CLAUDE.md exists and has content
        let claude_md_path = dir.join("CLAUDE.md");
        assert!(claude_md_path.exists());
        let content = std::fs::read_to_string(&claude_md_path).unwrap();
        assert!(content.contains("AgentForge Agent Configuration"));

        // Assert .claude/hooks.json exists and is valid JSON
        let hooks_path = dir.join(".claude/hooks.json");
        assert!(hooks_path.exists());
        let hooks_content = std::fs::read_to_string(&hooks_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&hooks_content).unwrap();
        assert!(parsed.get("hooks").is_some());

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
