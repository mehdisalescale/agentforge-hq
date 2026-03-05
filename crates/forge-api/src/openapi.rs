//! OpenAPI specification and Scalar documentation UI.
//!
//! Provides mirror/schema types for OpenAPI documentation without modifying
//! existing route handlers. Serves `/api/openapi.json` and `/docs` (Scalar UI).

use axum::{routing::get, Json, Router};
use utoipa::openapi::{self};
use utoipa::{OpenApi, ToSchema};
use utoipa_scalar::{Scalar, Servable};

// ---------------------------------------------------------------------------
// Mirror schema types — used only for OpenAPI doc generation
// ---------------------------------------------------------------------------

/// Request body for POST /api/v1/run.
#[derive(ToSchema, serde::Serialize)]
pub struct RunRequestSchema {
    /// UUID of the agent to run.
    pub agent_id: String,
    /// The prompt / task to execute.
    pub prompt: String,
    /// Optional existing session to resume.
    pub session_id: Option<String>,
    /// Working directory for the run.
    pub directory: Option<String>,
}

/// Response from POST /api/v1/run (202 Accepted).
#[derive(ToSchema, serde::Serialize)]
pub struct RunResponseSchema {
    /// UUID of the (new or existing) session.
    pub session_id: String,
    /// Human-readable status message.
    pub message: Option<String>,
}

/// An AI agent configuration.
#[derive(ToSchema, serde::Serialize)]
pub struct AgentSchema {
    /// UUID.
    pub id: String,
    pub name: String,
    /// Model identifier (e.g. `claude-sonnet-4-20250514`).
    pub model: String,
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub use_max: bool,
    pub preset: Option<String>,
    pub config: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for POST /api/v1/agents.
#[derive(ToSchema, serde::Serialize)]
pub struct NewAgentSchema {
    pub name: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub use_max: Option<bool>,
    pub preset: Option<String>,
    pub config: Option<serde_json::Value>,
}

/// Request body for PUT /api/v1/agents/:id.
#[derive(ToSchema, serde::Serialize)]
pub struct UpdateAgentSchema {
    pub name: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub use_max: Option<bool>,
    pub preset: Option<String>,
    pub config: Option<serde_json::Value>,
}

/// A running or completed session.
#[derive(ToSchema, serde::Serialize)]
pub struct SessionSchema {
    /// UUID.
    pub id: String,
    /// UUID of the owning agent.
    pub agent_id: String,
    pub claude_session_id: Option<String>,
    pub directory: String,
    /// `created` | `running` | `completed` | `failed`.
    pub status: String,
    pub cost_usd: f64,
    pub created_at: String,
    pub updated_at: String,
}

/// A reusable skill (prompt template).
#[derive(ToSchema, serde::Serialize)]
pub struct SkillSchema {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub content: String,
    pub source_repo: Option<String>,
    pub parameters_json: Option<String>,
    pub examples_json: Option<String>,
    pub usage_count: i32,
    pub created_at: String,
}

/// A multi-step workflow definition.
#[derive(ToSchema, serde::Serialize)]
pub struct WorkflowSchema {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A cross-session memory fact.
#[derive(ToSchema, serde::Serialize)]
pub struct MemorySchema {
    pub id: String,
    pub category: String,
    pub content: String,
    pub confidence: f64,
    pub source_session_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for POST /api/v1/memory.
#[derive(ToSchema, serde::Serialize)]
pub struct NewMemorySchema {
    pub category: Option<String>,
    pub content: String,
    pub confidence: Option<f64>,
    pub source_session_id: Option<String>,
}

/// Request body for PUT /api/v1/memory/:id.
#[derive(ToSchema, serde::Serialize)]
pub struct UpdateMemorySchema {
    pub content: Option<String>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
}

/// A lifecycle hook (shell command triggered on events).
#[derive(ToSchema, serde::Serialize)]
pub struct HookSchema {
    pub id: String,
    pub name: String,
    /// Event type the hook fires on (e.g. `session.complete`).
    pub event_type: String,
    /// `pre` or `post`.
    pub timing: String,
    pub command: String,
    pub enabled: bool,
    pub created_at: String,
}

/// Request body for POST /api/v1/hooks.
#[derive(ToSchema, serde::Serialize)]
pub struct NewHookSchema {
    pub name: String,
    pub event_type: String,
    /// Must be `pre` or `post`.
    pub timing: String,
    pub command: String,
}

/// Request body for PUT /api/v1/hooks/:id.
#[derive(ToSchema, serde::Serialize)]
pub struct UpdateHookSchema {
    pub name: Option<String>,
    pub command: Option<String>,
    pub enabled: Option<bool>,
}

/// A cron-scheduled job.
#[derive(ToSchema, serde::Serialize)]
pub struct ScheduleSchema {
    pub id: String,
    pub name: String,
    /// Cron expression (7 fields, second granularity).
    pub cron_expr: String,
    pub agent_id: String,
    pub prompt: String,
    pub directory: String,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub run_count: i64,
    pub created_at: String,
}

/// Request body for POST /api/v1/schedules.
#[derive(ToSchema, serde::Serialize)]
pub struct NewScheduleSchema {
    pub name: String,
    pub cron_expr: String,
    pub agent_id: String,
    pub prompt: String,
    #[serde(default)]
    pub directory: Option<String>,
}

/// Request body for PUT /api/v1/schedules/:id.
#[derive(ToSchema, serde::Serialize)]
pub struct UpdateScheduleSchema {
    pub name: Option<String>,
    pub cron_expr: Option<String>,
    pub prompt: Option<String>,
    pub directory: Option<String>,
    pub enabled: Option<bool>,
}

/// Health check response.
#[derive(ToSchema, serde::Serialize)]
pub struct HealthSchema {
    /// Always `"ok"`.
    pub status: String,
}

/// Analytics usage report.
#[derive(ToSchema, serde::Serialize)]
pub struct UsageReportSchema {
    pub total_cost: f64,
    pub daily_costs: Vec<DailyCostSchema>,
    pub agent_breakdown: Vec<AgentCostBreakdownSchema>,
    pub stats: SessionStatsSchema,
    pub projected_monthly_cost: f64,
}

#[derive(ToSchema, serde::Serialize)]
pub struct DailyCostSchema {
    pub date: String,
    pub cost: f64,
}

#[derive(ToSchema, serde::Serialize)]
pub struct AgentCostBreakdownSchema {
    pub agent_id: String,
    pub total_cost: f64,
    pub session_count: i64,
}

#[derive(ToSchema, serde::Serialize)]
pub struct SessionStatsSchema {
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
    pub avg_cost: f64,
    pub p90_cost: f64,
}

/// Generic error response.
#[derive(ToSchema, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ---------------------------------------------------------------------------
// OpenAPI document
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Claude Forge API",
        version = "0.6.0",
        description = "Multi-agent Claude Code orchestrator — REST API.\n\nForge manages AI agents, sessions, skills, workflows, memory, hooks, and schedules.\nReal-time events are available via WebSocket at `/ws`.",
        license(name = "MIT")
    ),
    servers(
        (url = "http://127.0.0.1:4173", description = "Local development")
    ),
    components(schemas(
        RunRequestSchema,
        RunResponseSchema,
        AgentSchema,
        NewAgentSchema,
        UpdateAgentSchema,
        SessionSchema,
        SkillSchema,
        WorkflowSchema,
        MemorySchema,
        NewMemorySchema,
        UpdateMemorySchema,
        HookSchema,
        NewHookSchema,
        UpdateHookSchema,
        ScheduleSchema,
        NewScheduleSchema,
        UpdateScheduleSchema,
        HealthSchema,
        UsageReportSchema,
        DailyCostSchema,
        AgentCostBreakdownSchema,
        SessionStatsSchema,
        ErrorResponse,
    )),
    tags(
        (name = "health", description = "Health check"),
        (name = "agents", description = "Agent CRUD"),
        (name = "run", description = "Start agent execution"),
        (name = "sessions", description = "Session management"),
        (name = "skills", description = "Reusable prompt templates"),
        (name = "workflows", description = "Multi-step workflow definitions"),
        (name = "memory", description = "Cross-session memory facts"),
        (name = "hooks", description = "Lifecycle hooks (shell commands)"),
        (name = "schedules", description = "Cron-scheduled jobs"),
        (name = "analytics", description = "Usage analytics and cost tracking"),
        (name = "websocket", description = "Real-time event stream"),
    )
)]
pub struct ApiDoc;

/// Manually add path items to the generated OpenAPI spec since we cannot annotate
/// existing route handlers with `#[utoipa::path]` without modifying them.
fn enrich_spec(mut spec: openapi::OpenApi) -> openapi::OpenApi {
    use openapi::path::{OperationBuilder, ParameterBuilder, ParameterIn, PathItemBuilder};
    use openapi::request_body::RequestBodyBuilder;
    use openapi::response::ResponseBuilder;
    use openapi::path::HttpMethod;
    use openapi::{ContentBuilder, PathsBuilder, RefOr};

    let json_ct = "application/json";

    // Helper: build a JSON content ref
    let json_ref = |schema_name: &str| -> openapi::Content {
        ContentBuilder::new()
            .schema(Some(RefOr::Ref(openapi::Ref::new(format!(
                "#/components/schemas/{}",
                schema_name
            )))))
            .build()
    };

    // Helper: a path parameter
    let path_param = |name: &str, desc: &str| -> openapi::path::Parameter {
        ParameterBuilder::new()
            .name(name)
            .parameter_in(ParameterIn::Path)
            .required(openapi::Required::True)
            .description(Some(desc))
            .build()
    };

    // Helper: 200 response with schema
    let ok_json = |schema_name: &str, desc: &str| -> openapi::response::Response {
        ResponseBuilder::new()
            .description(desc)
            .content(json_ct, json_ref(schema_name))
            .build()
    };

    // Helper: 200 response with array of schema
    let ok_array = |schema_name: &str, desc: &str| -> openapi::response::Response {
        ResponseBuilder::new()
            .description(desc)
            .content(
                json_ct,
                ContentBuilder::new()
                    .schema(Some(openapi::schema::ArrayBuilder::new().items(
                        RefOr::Ref(openapi::Ref::new(format!(
                            "#/components/schemas/{}",
                            schema_name
                        ))),
                    )))
                    .build(),
            )
            .build()
    };

    let no_content = || -> openapi::response::Response {
        ResponseBuilder::new()
            .description("No content")
            .build()
    };

    let accepted_json = |schema_name: &str, desc: &str| -> openapi::response::Response {
        ResponseBuilder::new()
            .description(desc)
            .content(json_ct, json_ref(schema_name))
            .build()
    };

    let json_body = |schema_name: &str| -> openapi::request_body::RequestBody {
        RequestBodyBuilder::new()
            .content(json_ct, json_ref(schema_name))
            .required(Some(openapi::Required::True))
            .build()
    };

    let mut paths = PathsBuilder::new();

    // --- Health ---
    paths = paths.path(
        "/api/v1/health",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("health")
                    .summary(Some("Health check"))
                    .operation_id(Some("health_check"))
                    .response("200", ok_json("HealthSchema", "Service is healthy"))
                    .build(),
            )
            .build(),
    );

    // --- Agents ---
    paths = paths.path(
        "/api/v1/agents",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("agents")
                    .summary(Some("List all agents"))
                    .operation_id(Some("list_agents"))
                    .response("200", ok_array("AgentSchema", "Array of agents"))
                    .build(),
            )
            .operation(
                HttpMethod::Post,
                OperationBuilder::new()
                    .tag("agents")
                    .summary(Some("Create a new agent"))
                    .operation_id(Some("create_agent"))
                    .request_body(Some(json_body("NewAgentSchema")))
                    .response("200", ok_json("AgentSchema", "Created agent"))
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/agents/{id}",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("agents")
                    .summary(Some("Get agent by ID"))
                    .operation_id(Some("get_agent"))
                    .parameter(path_param("id", "Agent UUID"))
                    .response("200", ok_json("AgentSchema", "Agent details"))
                    .response("404", ok_json("ErrorResponse", "Agent not found"))
                    .build(),
            )
            .operation(
                HttpMethod::Put,
                OperationBuilder::new()
                    .tag("agents")
                    .summary(Some("Update an agent"))
                    .operation_id(Some("update_agent"))
                    .parameter(path_param("id", "Agent UUID"))
                    .request_body(Some(json_body("UpdateAgentSchema")))
                    .response("200", ok_json("AgentSchema", "Updated agent"))
                    .build(),
            )
            .operation(
                HttpMethod::Delete,
                OperationBuilder::new()
                    .tag("agents")
                    .summary(Some("Delete an agent"))
                    .operation_id(Some("delete_agent"))
                    .parameter(path_param("id", "Agent UUID"))
                    .response("204", no_content())
                    .build(),
            )
            .build(),
    );

    // --- Run ---
    paths = paths.path(
        "/api/v1/run",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Post,
                OperationBuilder::new()
                    .tag("run")
                    .summary(Some("Start an agent run"))
                    .description(Some(
                        "Starts a new process for the given agent and prompt. \
                         Returns 202 Accepted with the session ID. \
                         Subscribe to `/ws` for real-time progress events.",
                    ))
                    .operation_id(Some("run_agent"))
                    .request_body(Some(json_body("RunRequestSchema")))
                    .response(
                        "202",
                        accepted_json("RunResponseSchema", "Run started"),
                    )
                    .response("429", ok_json("ErrorResponse", "Rate limited"))
                    .build(),
            )
            .build(),
    );

    // --- Sessions ---
    paths = paths.path(
        "/api/v1/sessions",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("sessions")
                    .summary(Some("List all sessions"))
                    .operation_id(Some("list_sessions"))
                    .response("200", ok_array("SessionSchema", "Array of sessions"))
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/sessions/{id}",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("sessions")
                    .summary(Some("Get session by ID"))
                    .operation_id(Some("get_session"))
                    .parameter(path_param("id", "Session UUID"))
                    .response("200", ok_json("SessionSchema", "Session details"))
                    .build(),
            )
            .operation(
                HttpMethod::Delete,
                OperationBuilder::new()
                    .tag("sessions")
                    .summary(Some("Delete a session"))
                    .operation_id(Some("delete_session"))
                    .parameter(path_param("id", "Session UUID"))
                    .response("204", no_content())
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/sessions/{id}/events",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("sessions")
                    .summary(Some("Get session events"))
                    .operation_id(Some("get_session_events"))
                    .parameter(path_param("id", "Session UUID"))
                    .response(
                        "200",
                        ResponseBuilder::new()
                            .description("Array of session events (JSON)")
                            .build(),
                    )
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/sessions/{id}/export",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("sessions")
                    .summary(Some("Export session data"))
                    .description(Some(
                        "Export session and events. Query param `format=json` or `format=markdown`.",
                    ))
                    .operation_id(Some("export_session"))
                    .parameter(path_param("id", "Session UUID"))
                    .response(
                        "200",
                        ResponseBuilder::new()
                            .description("Exported session (JSON or Markdown)")
                            .build(),
                    )
                    .build(),
            )
            .build(),
    );

    // --- Skills ---
    paths = paths.path(
        "/api/v1/skills",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("skills")
                    .summary(Some("List all skills"))
                    .operation_id(Some("list_skills"))
                    .response("200", ok_array("SkillSchema", "Array of skills"))
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/skills/{id}",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("skills")
                    .summary(Some("Get skill by ID"))
                    .operation_id(Some("get_skill"))
                    .parameter(path_param("id", "Skill ID"))
                    .response("200", ok_json("SkillSchema", "Skill details"))
                    .build(),
            )
            .build(),
    );

    // --- Workflows ---
    paths = paths.path(
        "/api/v1/workflows",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("workflows")
                    .summary(Some("List all workflows"))
                    .operation_id(Some("list_workflows"))
                    .response("200", ok_array("WorkflowSchema", "Array of workflows"))
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/workflows/{id}",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("workflows")
                    .summary(Some("Get workflow by ID"))
                    .operation_id(Some("get_workflow"))
                    .parameter(path_param("id", "Workflow ID"))
                    .response("200", ok_json("WorkflowSchema", "Workflow details"))
                    .build(),
            )
            .build(),
    );

    // --- Memory ---
    paths = paths.path(
        "/api/v1/memory",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("memory")
                    .summary(Some("List memory entries"))
                    .operation_id(Some("list_memory"))
                    .response("200", ok_array("MemorySchema", "Array of memory entries"))
                    .build(),
            )
            .operation(
                HttpMethod::Post,
                OperationBuilder::new()
                    .tag("memory")
                    .summary(Some("Create a memory entry"))
                    .operation_id(Some("create_memory"))
                    .request_body(Some(json_body("NewMemorySchema")))
                    .response("200", ok_json("MemorySchema", "Created memory"))
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/memory/{id}",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("memory")
                    .summary(Some("Get memory entry by ID"))
                    .operation_id(Some("get_memory"))
                    .parameter(path_param("id", "Memory UUID"))
                    .response("200", ok_json("MemorySchema", "Memory details"))
                    .build(),
            )
            .operation(
                HttpMethod::Put,
                OperationBuilder::new()
                    .tag("memory")
                    .summary(Some("Update a memory entry"))
                    .operation_id(Some("update_memory"))
                    .parameter(path_param("id", "Memory UUID"))
                    .request_body(Some(json_body("UpdateMemorySchema")))
                    .response("200", ok_json("MemorySchema", "Updated memory"))
                    .build(),
            )
            .operation(
                HttpMethod::Delete,
                OperationBuilder::new()
                    .tag("memory")
                    .summary(Some("Delete a memory entry"))
                    .operation_id(Some("delete_memory"))
                    .parameter(path_param("id", "Memory UUID"))
                    .response("204", no_content())
                    .build(),
            )
            .build(),
    );

    // --- Hooks ---
    paths = paths.path(
        "/api/v1/hooks",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("hooks")
                    .summary(Some("List all hooks"))
                    .operation_id(Some("list_hooks"))
                    .response("200", ok_array("HookSchema", "Array of hooks"))
                    .build(),
            )
            .operation(
                HttpMethod::Post,
                OperationBuilder::new()
                    .tag("hooks")
                    .summary(Some("Create a hook"))
                    .operation_id(Some("create_hook"))
                    .request_body(Some(json_body("NewHookSchema")))
                    .response("200", ok_json("HookSchema", "Created hook"))
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/hooks/{id}",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("hooks")
                    .summary(Some("Get hook by ID"))
                    .operation_id(Some("get_hook"))
                    .parameter(path_param("id", "Hook UUID"))
                    .response("200", ok_json("HookSchema", "Hook details"))
                    .build(),
            )
            .operation(
                HttpMethod::Put,
                OperationBuilder::new()
                    .tag("hooks")
                    .summary(Some("Update a hook"))
                    .operation_id(Some("update_hook"))
                    .parameter(path_param("id", "Hook UUID"))
                    .request_body(Some(json_body("UpdateHookSchema")))
                    .response("200", ok_json("HookSchema", "Updated hook"))
                    .build(),
            )
            .operation(
                HttpMethod::Delete,
                OperationBuilder::new()
                    .tag("hooks")
                    .summary(Some("Delete a hook"))
                    .operation_id(Some("delete_hook"))
                    .parameter(path_param("id", "Hook UUID"))
                    .response("204", no_content())
                    .build(),
            )
            .build(),
    );

    // --- Schedules ---
    paths = paths.path(
        "/api/v1/schedules",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("schedules")
                    .summary(Some("List all schedules"))
                    .operation_id(Some("list_schedules"))
                    .response("200", ok_array("ScheduleSchema", "Array of schedules"))
                    .build(),
            )
            .operation(
                HttpMethod::Post,
                OperationBuilder::new()
                    .tag("schedules")
                    .summary(Some("Create a schedule"))
                    .operation_id(Some("create_schedule"))
                    .request_body(Some(json_body("NewScheduleSchema")))
                    .response("200", ok_json("ScheduleSchema", "Created schedule"))
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/schedules/{id}",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("schedules")
                    .summary(Some("Get schedule by ID"))
                    .operation_id(Some("get_schedule"))
                    .parameter(path_param("id", "Schedule UUID"))
                    .response("200", ok_json("ScheduleSchema", "Schedule details"))
                    .build(),
            )
            .operation(
                HttpMethod::Put,
                OperationBuilder::new()
                    .tag("schedules")
                    .summary(Some("Update a schedule"))
                    .operation_id(Some("update_schedule"))
                    .parameter(path_param("id", "Schedule UUID"))
                    .request_body(Some(json_body("UpdateScheduleSchema")))
                    .response("200", ok_json("ScheduleSchema", "Updated schedule"))
                    .build(),
            )
            .operation(
                HttpMethod::Delete,
                OperationBuilder::new()
                    .tag("schedules")
                    .summary(Some("Delete a schedule"))
                    .operation_id(Some("delete_schedule"))
                    .parameter(path_param("id", "Schedule UUID"))
                    .response("204", no_content())
                    .build(),
            )
            .build(),
    );

    paths = paths.path(
        "/api/v1/schedules/{id}/trigger",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Post,
                OperationBuilder::new()
                    .tag("schedules")
                    .summary(Some("Trigger a schedule manually"))
                    .operation_id(Some("trigger_schedule"))
                    .parameter(path_param("id", "Schedule UUID"))
                    .response(
                        "200",
                        ResponseBuilder::new()
                            .description("Schedule triggered")
                            .build(),
                    )
                    .build(),
            )
            .build(),
    );

    // --- Analytics ---
    paths = paths.path(
        "/api/v1/analytics/usage",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("analytics")
                    .summary(Some("Get usage analytics"))
                    .description(Some(
                        "Returns cost breakdown, daily costs, session stats, and projected monthly cost.",
                    ))
                    .operation_id(Some("get_usage_analytics"))
                    .response("200", ok_json("UsageReportSchema", "Usage report"))
                    .build(),
            )
            .build(),
    );

    // --- WebSocket ---
    paths = paths.path(
        "/ws",
        PathItemBuilder::new()
            .operation(
                HttpMethod::Get,
                OperationBuilder::new()
                    .tag("websocket")
                    .summary(Some("WebSocket event stream"))
                    .description(Some(
                        "Upgrade to WebSocket for real-time ForgeEvent broadcast. \
                         Events are JSON-encoded ForgeEvent variants.",
                    ))
                    .operation_id(Some("ws_events"))
                    .response(
                        "101",
                        ResponseBuilder::new()
                            .description("Switching Protocols (WebSocket)")
                            .build(),
                    )
                    .build(),
            )
            .build(),
    );

    spec.paths = paths.build();
    spec
}

/// Handler: GET /api/openapi.json
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    let spec = enrich_spec(ApiDoc::openapi());
    Json(spec)
}

/// Build routes for OpenAPI spec and Scalar documentation UI.
///
/// - `GET /api/openapi.json` — raw OpenAPI 3.1 JSON
/// - `GET /docs` — Scalar interactive API reference
pub fn openapi_routes() -> Router<crate::state::AppState> {
    let spec = enrich_spec(ApiDoc::openapi());
    Router::new()
        .route("/api/openapi.json", get(openapi_json))
        .merge(Scalar::with_url("/docs", spec))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_spec_deserializes() {
        let spec = enrich_spec(ApiDoc::openapi());
        let json = serde_json::to_string_pretty(&spec).unwrap();
        assert!(json.contains("Claude Forge API"));
        assert!(json.contains("/api/v1/agents"));
        assert!(json.contains("/api/v1/run"));
        assert!(json.contains("/api/v1/sessions"));
        assert!(json.contains("/api/v1/skills"));
        assert!(json.contains("/api/v1/workflows"));
        assert!(json.contains("/api/v1/memory"));
        assert!(json.contains("/api/v1/hooks"));
        assert!(json.contains("/api/v1/schedules"));
        assert!(json.contains("/api/v1/analytics/usage"));
        assert!(json.contains("/ws"));
    }

    #[test]
    fn openapi_spec_has_all_schemas() {
        let spec = enrich_spec(ApiDoc::openapi());
        let json = serde_json::to_string(&spec).unwrap();
        let expected_schemas = [
            "RunRequestSchema",
            "RunResponseSchema",
            "AgentSchema",
            "NewAgentSchema",
            "SessionSchema",
            "SkillSchema",
            "WorkflowSchema",
            "MemorySchema",
            "HookSchema",
            "ScheduleSchema",
            "HealthSchema",
            "UsageReportSchema",
            "ErrorResponse",
        ];
        for schema in expected_schemas {
            assert!(
                json.contains(schema),
                "Missing schema: {}",
                schema
            );
        }
    }

    #[test]
    fn openapi_spec_version_is_correct() {
        let spec = enrich_spec(ApiDoc::openapi());
        assert_eq!(spec.info.version, "0.6.0");
        assert_eq!(spec.info.title, "Claude Forge API");
    }

    #[test]
    fn openapi_spec_has_tags() {
        let spec = enrich_spec(ApiDoc::openapi());
        let tag_names: Vec<&str> = spec
            .tags
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.name.as_str())
            .collect();
        assert!(tag_names.contains(&"agents"));
        assert!(tag_names.contains(&"sessions"));
        assert!(tag_names.contains(&"run"));
        assert!(tag_names.contains(&"health"));
        assert!(tag_names.contains(&"memory"));
        assert!(tag_names.contains(&"hooks"));
        assert!(tag_names.contains(&"schedules"));
        assert!(tag_names.contains(&"analytics"));
        assert!(tag_names.contains(&"websocket"));
    }

    #[test]
    fn openapi_routes_builds_without_panic() {
        // Just verify the router construction doesn't panic
        let _router = openapi_routes();
    }
}
