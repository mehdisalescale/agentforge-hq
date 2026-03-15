//! Forge MCP server: stdio transport via rmcp, agent and session tools.
//! Usage: FORGE_DB_PATH=~/.claude-forge/forge.db forge-mcp

use forge_agent::model::{NewAgent, UpdateAgent};
use forge_core::ids::{AgentId, SessionId};
use forge_db::{
    AgentRepo, AnalyticsRepo, ApprovalRepo, CompanyRepo, EventRepo, GoalRepo, Migrator,
    NewApproval, NewOrgPosition, NewSession, OrgPositionRepo, PersonaRepo, SessionRepo,
    StoredEvent,
};
use forge_persona::model::PersonaId;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

fn default_db_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/.claude-forge/forge.db", home)
}

#[derive(Clone)]
pub struct ForgeMcp {
    agent_repo: Arc<AgentRepo>,
    session_repo: Arc<SessionRepo>,
    event_repo: Arc<EventRepo>,
    persona_repo: Arc<PersonaRepo>,
    company_repo: Arc<CompanyRepo>,
    approval_repo: Arc<ApprovalRepo>,
    analytics_repo: Arc<AnalyticsRepo>,
    goal_repo: Arc<GoalRepo>,
    org_position_repo: Arc<OrgPositionRepo>,
    tool_router: ToolRouter<Self>,
}

// --- Parameter types ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IdParam {
    #[schemars(description = "UUID of the entity")]
    id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentCreateParam {
    #[schemars(description = "Agent name (alphanumeric, dashes, underscores)")]
    name: String,
    #[schemars(description = "Model identifier (e.g. claude-sonnet-4-20250514)")]
    model: Option<String>,
    #[schemars(description = "System prompt for the agent")]
    system_prompt: Option<String>,
    #[schemars(description = "Preset name: CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer")]
    preset: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentUpdateParam {
    #[schemars(description = "UUID of the agent to update")]
    id: String,
    #[schemars(description = "New name")]
    name: Option<String>,
    #[schemars(description = "New model identifier")]
    model: Option<String>,
    #[schemars(description = "New system prompt")]
    system_prompt: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SessionCreateParam {
    #[schemars(description = "UUID of the agent for this session")]
    agent_id: String,
    #[schemars(description = "Working directory for the session")]
    directory: Option<String>,
    #[schemars(description = "Claude session ID for resume")]
    claude_session_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SessionExportParam {
    #[schemars(description = "UUID of the session to export")]
    id: String,
    #[schemars(description = "Export format: json or markdown (default: json)")]
    format: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClassifyTaskParam {
    #[schemars(description = "The prompt or task description to classify")]
    prompt: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPersonasParam {
    #[schemars(description = "Optional division filter (e.g. 'engineering', 'security', 'product')")]
    division: Option<String>,
    #[schemars(description = "Optional search term to filter by name or description")]
    search: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBudgetParam {
    #[schemars(description = "UUID of the company")]
    company_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RequestApprovalParam {
    #[schemars(description = "UUID of the company")]
    company_id: String,
    #[schemars(description = "Approval type: budget_increase, run_authorization, deployment, other")]
    approval_type: String,
    #[schemars(description = "Description of what needs approval")]
    description: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSessionEventsParam {
    #[schemars(description = "UUID of the session")]
    session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAnalyticsParam {
    #[schemars(description = "UUID of the company (optional — omit for global analytics)")]
    company_id: Option<String>,
    #[schemars(description = "Start date (YYYY-MM-DD, default: 30 days ago)")]
    start: Option<String>,
    #[schemars(description = "End date (YYYY-MM-DD, default: today)")]
    end: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HirePersonaParam {
    #[schemars(description = "UUID of the persona to hire")]
    persona_id: String,
    #[schemars(description = "UUID of the company to hire into")]
    company_id: String,
    #[schemars(description = "UUID of the department (optional)")]
    department_id: Option<String>,
}

// --- Helper ---

fn parse_uuid(s: &str) -> Result<uuid::Uuid, ErrorData> {
    uuid::Uuid::parse_str(s).map_err(|_| ErrorData::invalid_params(format!("invalid UUID: {}", s), None))
}

fn forge_err(e: forge_core::ForgeError) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

fn to_json_content(value: &impl serde::Serialize) -> Result<CallToolResult, ErrorData> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// --- Tool implementations ---

#[tool_router]
impl ForgeMcp {
    fn new(
        agent_repo: Arc<AgentRepo>,
        session_repo: Arc<SessionRepo>,
        event_repo: Arc<EventRepo>,
        persona_repo: Arc<PersonaRepo>,
        company_repo: Arc<CompanyRepo>,
        approval_repo: Arc<ApprovalRepo>,
        analytics_repo: Arc<AnalyticsRepo>,
        goal_repo: Arc<GoalRepo>,
        org_position_repo: Arc<OrgPositionRepo>,
    ) -> Self {
        Self {
            agent_repo,
            session_repo,
            event_repo,
            persona_repo,
            company_repo,
            approval_repo,
            analytics_repo,
            goal_repo,
            org_position_repo,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List all agents")]
    async fn agent_list(&self) -> Result<CallToolResult, ErrorData> {
        let agents = self.agent_repo.list().map_err(forge_err)?;
        to_json_content(&agents)
    }

    #[tool(description = "Get an agent by ID")]
    async fn agent_get(
        &self,
        Parameters(IdParam { id }): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let agent_id = AgentId(parse_uuid(&id)?);
        let agent = self.agent_repo.get(&agent_id).map_err(forge_err)?;
        to_json_content(&agent)
    }

    #[tool(description = "Create a new agent with name, optional model, system_prompt, and preset")]
    async fn agent_create(
        &self,
        Parameters(params): Parameters<AgentCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let preset = params
            .preset
            .as_deref()
            .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok());
        let input = NewAgent {
            name: params.name,
            model: params.model,
            system_prompt: params.system_prompt,
            allowed_tools: None,
            max_turns: None,
            use_max: Some(false),
            preset,
            config: None,
        };
        let agent = self.agent_repo.create(&input).map_err(forge_err)?;
        to_json_content(&agent)
    }

    #[tool(description = "Update an existing agent's name, model, or system_prompt")]
    async fn agent_update(
        &self,
        Parameters(params): Parameters<AgentUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let agent_id = AgentId(parse_uuid(&params.id)?);
        let input = UpdateAgent {
            name: params.name,
            model: params.model,
            system_prompt: params.system_prompt.map(Some),  // Option<Option<String>> for PATCH semantics
            allowed_tools: None,
            max_turns: None,
            use_max: None,
            preset: None,
            config: None,
        };
        let agent = self.agent_repo.update(&agent_id, &input).map_err(forge_err)?;
        to_json_content(&agent)
    }

    #[tool(description = "Delete an agent by ID")]
    async fn agent_delete(
        &self,
        Parameters(IdParam { id }): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let agent_id = AgentId(parse_uuid(&id)?);
        self.agent_repo.delete(&agent_id).map_err(forge_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            r#"{"ok": true}"#,
        )]))
    }

    #[tool(description = "List all sessions")]
    async fn session_list(&self) -> Result<CallToolResult, ErrorData> {
        let sessions = self.session_repo.list().map_err(forge_err)?;
        to_json_content(&sessions)
    }

    #[tool(description = "Get a session by ID")]
    async fn session_get(
        &self,
        Parameters(IdParam { id }): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let session_id = SessionId(parse_uuid(&id)?);
        let session = self.session_repo.get(&session_id).map_err(forge_err)?;
        to_json_content(&session)
    }

    #[tool(description = "Create a new session for an agent")]
    async fn session_create(
        &self,
        Parameters(params): Parameters<SessionCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let agent_id = AgentId(parse_uuid(&params.agent_id)?);
        self.agent_repo.get(&agent_id).map_err(forge_err)?;
        let input = NewSession {
            agent_id,
            directory: params.directory.unwrap_or_else(|| ".".into()),
            claude_session_id: params.claude_session_id,
        };
        let session = self.session_repo.create(&input).map_err(forge_err)?;
        to_json_content(&session)
    }

    #[tool(description = "Delete a session by ID")]
    async fn session_delete(
        &self,
        Parameters(IdParam { id }): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let session_id = SessionId(parse_uuid(&id)?);
        self.session_repo.delete(&session_id).map_err(forge_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            r#"{"ok": true}"#,
        )]))
    }

    #[tool(description = "Export a session with its events as JSON or Markdown")]
    async fn session_export(
        &self,
        Parameters(params): Parameters<SessionExportParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let session_id = SessionId(parse_uuid(&params.id)?);
        let session = self.session_repo.get(&session_id).map_err(forge_err)?;
        let events = self.event_repo.query_by_session(&session_id).map_err(forge_err)?;
        let format = params.format.as_deref().unwrap_or("json");
        if format == "markdown" {
            let md = session_to_markdown(&session, &events);
            Ok(CallToolResult::success(vec![Content::text(md)]))
        } else {
            #[derive(serde::Serialize)]
            struct Export {
                session: forge_db::Session,
                events: Vec<ExportEvent>,
            }
            #[derive(serde::Serialize)]
            struct ExportEvent {
                id: String,
                session_id: Option<String>,
                agent_id: Option<String>,
                event_type: String,
                data_json: String,
                timestamp: String,
            }
            let events_export: Vec<ExportEvent> = events
                .iter()
                .map(|e| ExportEvent {
                    id: e.id.clone(),
                    session_id: e.session_id.clone(),
                    agent_id: e.agent_id.clone(),
                    event_type: e.event_type.clone(),
                    data_json: e.data_json.clone(),
                    timestamp: e.timestamp.clone(),
                })
                .collect();
            to_json_content(&Export { session, events: events_export })
        }
    }

    #[tool(
        name = "forge_classify_task",
        description = "Classify a task/prompt into a type (NewFeature, BugFix, CodeReview, Refactor, Research, General) and get recommended skills"
    )]
    async fn classify_task(
        &self,
        Parameters(p): Parameters<ClassifyTaskParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let detector = forge_process::task_type::TaskTypeDetector::new();
        let task_type = detector.classify(&p.prompt);

        let router = forge_process::skill_router::SkillRouter::new();
        let skills = router.skills_for(task_type);

        let result = serde_json::json!({
            "task_type": format!("{:?}", task_type),
            "recommended_skills": skills,
            "confidence": "keyword-based"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    #[tool(
        name = "forge_list_personas",
        description = "List available AI personas from the catalog. Filter by division or search term."
    )]
    async fn list_personas(
        &self,
        Parameters(p): Parameters<ListPersonasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let personas = self
            .persona_repo
            .list(p.division.as_deref(), p.search.as_deref())
            .map_err(|e| ErrorData::internal_error(format!("Failed to list personas: {}", e), None))?;

        let result: Vec<_> = personas
            .iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id.0.to_string(),
                    "name": item.name,
                    "short_description": item.short_description,
                    "division": item.division_slug,
                })
            })
            .collect();

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    #[tool(
        name = "forge_get_budget",
        description = "Get budget status for a company — remaining, used, and limit"
    )]
    async fn get_budget(
        &self,
        Parameters(p): Parameters<GetBudgetParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let company = self
            .company_repo
            .get(&p.company_id)
            .map_err(|e| ErrorData::internal_error(format!("Company not found: {}", e), None))?;

        let result = serde_json::json!({
            "company": company.name,
            "budget_limit": company.budget_limit,
            "budget_used": company.budget_used,
            "budget_remaining": company.budget_limit.map(|l| l - company.budget_used),
            "status": if company.budget_limit.map(|l| company.budget_used >= l).unwrap_or(false) {
                "exhausted"
            } else if company.budget_limit.map(|l| company.budget_used >= l * 0.9).unwrap_or(false) {
                "warning"
            } else {
                "ok"
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    // --- Governance tools ---

    #[tool(
        name = "forge_request_approval",
        description = "Request an approval from the company governance. Returns the approval ID to check later."
    )]
    async fn request_approval(
        &self,
        Parameters(p): Parameters<RequestApprovalParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let input = NewApproval {
            company_id: p.company_id,
            approval_type: p.approval_type,
            requester: "mcp-client".to_string(),
            data_json: serde_json::json!({ "description": p.description }).to_string(),
        };
        let approval = self.approval_repo.create(&input).map_err(forge_err)?;
        to_json_content(&serde_json::json!({
            "id": approval.id,
            "status": approval.status,
            "created_at": approval.created_at.to_rfc3339(),
        }))
    }

    #[tool(
        name = "forge_check_approval",
        description = "Check the status of an approval request"
    )]
    async fn check_approval(
        &self,
        Parameters(IdParam { id }): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let approval = self.approval_repo.get(&id).map_err(forge_err)?;
        to_json_content(&serde_json::json!({
            "id": approval.id,
            "company_id": approval.company_id,
            "approval_type": approval.approval_type,
            "status": approval.status,
            "requester": approval.requester,
            "approver": approval.approver,
            "data": approval.data_json,
            "created_at": approval.created_at.to_rfc3339(),
            "updated_at": approval.updated_at.to_rfc3339(),
        }))
    }

    // --- Execution tools ---

    #[tool(
        name = "forge_get_session_events",
        description = "Get all events for a session — tool uses, outputs, costs, security findings"
    )]
    async fn get_session_events(
        &self,
        Parameters(p): Parameters<GetSessionEventsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let session_id = SessionId(parse_uuid(&p.session_id)?);
        let events = self.event_repo.query_by_session(&session_id).map_err(forge_err)?;
        to_json_content(&events)
    }

    // --- Observability tools ---

    #[tool(
        name = "forge_get_analytics",
        description = "Get usage analytics — run counts, costs, success rates. Filter by company and date range."
    )]
    async fn get_analytics(
        &self,
        Parameters(p): Parameters<GetAnalyticsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let today = chrono::Utc::now().date_naive();
        let end = p.end.unwrap_or_else(|| today.format("%Y-%m-%d").to_string());
        let start = p.start.unwrap_or_else(|| {
            (today - chrono::Duration::days(30))
                .format("%Y-%m-%d")
                .to_string()
        });
        let report = self.analytics_repo.usage_report(&start, &end).map_err(forge_err)?;
        let mut result = serde_json::json!({
            "total_cost": report.total_cost,
            "projected_monthly_cost": report.projected_monthly_cost,
            "stats": {
                "total_sessions": report.stats.total,
                "completed": report.stats.completed,
                "failed": report.stats.failed,
                "avg_cost": report.stats.avg_cost,
                "p90_cost": report.stats.p90_cost,
            },
            "daily_costs": report.daily_costs.iter().map(|d| serde_json::json!({
                "date": d.date,
                "cost": d.cost,
            })).collect::<Vec<_>>(),
            "agent_breakdown": report.agent_breakdown.iter().map(|a| serde_json::json!({
                "agent_id": a.agent_id,
                "total_cost": a.total_cost,
                "session_count": a.session_count,
            })).collect::<Vec<_>>(),
        });
        if let Some(ref cid) = p.company_id {
            result["company_id"] = serde_json::json!(cid);
        }
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    // --- Workforce tools ---

    #[tool(
        name = "forge_hire_persona",
        description = "Hire a persona into a company — creates an agent and org position"
    )]
    async fn hire_persona(
        &self,
        Parameters(p): Parameters<HirePersonaParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let persona_id = PersonaId(parse_uuid(&p.persona_id)?);
        let persona = self
            .persona_repo
            .get(&persona_id)
            .map_err(|e| ErrorData::internal_error(format!("Persona not found: {}", e), None))?;

        // Create agent from persona (same logic as forge-api hire endpoint)
        let agent_name: String = persona
            .name
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        let new_agent = NewAgent {
            name: agent_name,
            model: None,
            system_prompt: Some(format!(
                "You are persona '{}'. Short summary: {}.\nUse this as your operating persona.",
                persona.name, persona.short_description
            )),
            allowed_tools: None,
            max_turns: None,
            use_max: None,
            preset: None,
            config: None,
        };
        let agent = self.agent_repo.create(&new_agent).map_err(forge_err)?;

        // Link persona_id for traceability
        self.agent_repo
            .set_persona_id(&agent.id, &persona_id.0.to_string())
            .map_err(forge_err)?;

        // Create org position
        let pos_input = NewOrgPosition {
            company_id: p.company_id,
            department_id: p.department_id,
            agent_id: Some(agent.id.0.to_string()),
            reports_to: None,
            role: persona.slug.clone(),
            title: Some(persona.name.clone()),
        };
        let position = self.org_position_repo.create(&pos_input).map_err(forge_err)?;

        to_json_content(&serde_json::json!({
            "agent_id": agent.id.0.to_string(),
            "position_id": position.id,
            "persona_name": persona.name,
        }))
    }

    // --- Goal tools ---

    #[tool(
        name = "forge_list_goals",
        description = "List goals for a company"
    )]
    async fn list_goals(
        &self,
        Parameters(p): Parameters<GetBudgetParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let goals = self.goal_repo.list_by_company(&p.company_id).map_err(forge_err)?;
        to_json_content(&goals)
    }
}

#[tool_handler]
impl ServerHandler for ForgeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Forge MCP server: manage Claude Code agents, sessions, personas, budgets, \
                 governance, and analytics. Tools: agent_list, agent_get, agent_create, \
                 agent_update, agent_delete, session_list, session_get, session_create, \
                 session_delete, session_export, forge_classify_task, forge_list_personas, \
                 forge_get_budget, forge_request_approval, forge_check_approval, \
                 forge_get_session_events, forge_get_analytics, forge_hire_persona, \
                 forge_list_goals."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

fn session_to_markdown(session: &forge_db::Session, events: &[StoredEvent]) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Session {}\n\n", session.id));
    md.push_str(&format!("- **Agent ID:** {}\n", session.agent_id));
    md.push_str(&format!("- **Directory:** {}\n", session.directory));
    md.push_str(&format!("- **Status:** {}\n", session.status));
    md.push_str(&format!("- **Created:** {}\n", session.created_at.to_rfc3339()));
    if let Some(ref c) = session.claude_session_id {
        md.push_str(&format!("- **Claude session:** {}\n", c));
    }
    md.push_str("\n## Events\n\n");
    for ev in events {
        md.push_str(&format!("### {} ({})\n\n", ev.event_type, ev.timestamp));
        md.push_str("```json\n");
        md.push_str(&ev.data_json);
        md.push_str("\n```\n\n");
    }
    md
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Logs go to stderr — stdout is reserved for MCP JSON-RPC transport
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let db_path = env::var("FORGE_DB_PATH").unwrap_or_else(|_| default_db_path());
    let path = Path::new(&db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    tracing::info!(path = %db_path, "opening database");
    let db = forge_db::DbPool::new(path)?;
    {
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending()?;
    }
    let conn = db.conn_arc();
    let agent_repo = Arc::new(AgentRepo::new(Arc::clone(&conn)));
    let session_repo = Arc::new(SessionRepo::new(Arc::clone(&conn)));
    let event_repo = Arc::new(EventRepo::new(Arc::clone(&conn)));
    let persona_repo = Arc::new(PersonaRepo::new(Arc::clone(&conn)));
    let company_repo = Arc::new(CompanyRepo::new(Arc::clone(&conn)));
    let approval_repo = Arc::new(ApprovalRepo::new(Arc::clone(&conn)));
    let analytics_repo = Arc::new(AnalyticsRepo::new(Arc::clone(&conn)));
    let goal_repo = Arc::new(GoalRepo::new(Arc::clone(&conn)));
    let org_position_repo = Arc::new(OrgPositionRepo::new(Arc::clone(&conn)));

    tracing::info!("starting Forge MCP server (stdio)");
    let server = ForgeMcp::new(
        agent_repo,
        session_repo,
        event_repo,
        persona_repo,
        company_repo,
        approval_repo,
        analytics_repo,
        goal_repo,
        org_position_repo,
    );
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    tracing::info!("Forge MCP server stopped");
    Ok(())
}
