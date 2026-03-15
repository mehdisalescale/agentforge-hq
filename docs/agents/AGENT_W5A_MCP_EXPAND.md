# Agent W5-A: MCP Tool Expansion + HTTP SSE Transport

> You are Agent W5-A. Your job: expand the MCP tool surface to 15+ tools covering the full workforce/governance API, and add HTTP SSE transport alongside stdio so remote clients can connect.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
crates/forge-mcp-bin/src/main.rs                   — FULL FILE (current MCP server, ~10-13 tools)
crates/forge-mcp-bin/Cargo.toml                    — dependencies
crates/forge-db/src/repos/sessions.rs              — SessionRepo
crates/forge-db/src/repos/events.rs                — EventRepo
crates/forge-db/src/repos/companies.rs             — CompanyRepo
crates/forge-db/src/repos/approvals.rs             — ApprovalRepo
crates/forge-db/src/repos/analytics.rs             — AnalyticsRepo, UsageReport
crates/forge-db/src/repos/goals.rs                 — GoalRepo
crates/forge-db/src/repos/org_positions.rs         — OrgPositionRepo
stat-qou-plan/REVISED_PLAN.md                      — MCP tool surface plan (lines 120-148)
```

## Step 2: Add Missing MCP Tools

Check what tools already exist in main.rs. The target is 15+ tools. Add whichever of these are missing:

### Governance tools:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RequestApprovalParam {
    #[schemars(description = "UUID of the company")]
    company_id: String,
    #[schemars(description = "Approval type: budget_increase, run_authorization, deployment, other")]
    approval_type: String,
    #[schemars(description = "Description of what needs approval")]
    description: String,
}

#[tool(
    name = "forge_request_approval",
    description = "Request an approval from the company governance. Returns the approval ID to check later."
)]
async fn request_approval(&self, #[tool(params)] p: RequestApprovalParam) -> Result<CallToolResult, ErrorData> {
    // Use approval_repo.create()
}

#[tool(
    name = "forge_check_approval",
    description = "Check the status of an approval request"
)]
async fn check_approval(&self, #[tool(params)] p: IdParam) -> Result<CallToolResult, ErrorData> {
    // Use approval_repo.get()
}
```

### Execution tools:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSessionEventsParam {
    #[schemars(description = "UUID of the session")]
    session_id: String,
}

#[tool(
    name = "forge_get_session_events",
    description = "Get all events for a session — tool uses, outputs, costs, security findings"
)]
async fn get_session_events(&self, #[tool(params)] p: GetSessionEventsParam) -> Result<CallToolResult, ErrorData> {
    // Use event_repo.query_by_session()
}
```

### Observability tools:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAnalyticsParam {
    #[schemars(description = "UUID of the company (optional — omit for global analytics)")]
    company_id: Option<String>,
    #[schemars(description = "Start date (YYYY-MM-DD, default: 30 days ago)")]
    start: Option<String>,
    #[schemars(description = "End date (YYYY-MM-DD, default: today)")]
    end: Option<String>,
}

#[tool(
    name = "forge_get_analytics",
    description = "Get usage analytics — run counts, costs, success rates. Filter by company and date range."
)]
async fn get_analytics(&self, #[tool(params)] p: GetAnalyticsParam) -> Result<CallToolResult, ErrorData> {
    // Use analytics_repo.usage_report()
}
```

### Workforce tools:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HirePersonaParam {
    #[schemars(description = "UUID of the persona to hire")]
    persona_id: String,
    #[schemars(description = "UUID of the company to hire into")]
    company_id: String,
    #[schemars(description = "UUID of the department (optional)")]
    department_id: Option<String>,
}

#[tool(
    name = "forge_hire_persona",
    description = "Hire a persona into a company — creates an agent and org position"
)]
async fn hire_persona(&self, #[tool(params)] p: HirePersonaParam) -> Result<CallToolResult, ErrorData> {
    // This is more complex — needs AgentRepo + OrgPositionRepo
    // Follow the same logic as the personas hire endpoint in forge-api
}
```

### Goal tools:

```rust
#[tool(
    name = "forge_list_goals",
    description = "List goals for a company"
)]
async fn list_goals(&self, #[tool(params)] p: GetBudgetParam) -> Result<CallToolResult, ErrorData> {
    // Use goal_repo.list_by_company()
}
```

### Needed repos

Add to `ForgeMcp` struct whichever of these are missing:
- `approval_repo: Arc<ApprovalRepo>`
- `analytics_repo: Arc<AnalyticsRepo>`
- `goal_repo: Arc<GoalRepo>`
- `org_position_repo: Arc<OrgPositionRepo>`

Add corresponding dependencies to `Cargo.toml` if needed.

Update `ForgeMcp::new()` and `main()` to initialize all repos.

## Step 3: HTTP SSE Transport

Add an HTTP SSE endpoint to `forge-mcp-bin` so it can serve MCP over HTTP alongside stdio.

This is a stretch goal. If the rmcp crate supports SSE transport, use it. Check `rmcp` docs or examples. If it's not straightforward, skip this step and note it in the report as "deferred — rmcp SSE transport not yet available."

If you do implement it, the pattern is:
1. Accept `--transport stdio|sse` CLI flag
2. For stdio: current behavior
3. For SSE: start an Axum server with SSE endpoint

## Step 4: Update ServerInfo

Update `get_info()` to list all tools in the instructions string.

## Step 5: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test 2>&1 | grep "FAILED"     # no failures
```

## Rules

- Modify: `crates/forge-mcp-bin/src/main.rs` — add tools, repos, ServerInfo
- Modify: `crates/forge-mcp-bin/Cargo.toml` — add dependencies
- Do NOT modify any forge-api files (middleware, routes, state)
- Do NOT touch frontend files
- Do NOT touch forge-app main.rs
- Do NOT modify existing tests — only add new ones
- Commit with: `feat(mcp): expand to 15+ tools covering governance, analytics, and workforce`

## Report

When done, append your report here:

```
STATUS: complete
FILES_MODIFIED: [crates/forge-mcp-bin/src/main.rs, crates/forge-mcp-bin/Cargo.toml, CLAUDE.md]
MCP_TOOLS_BEFORE: 13
MCP_TOOLS_AFTER: 19
TOOLS_ADDED: [forge_request_approval, forge_check_approval, forge_get_session_events, forge_get_analytics, forge_hire_persona, forge_list_goals]
SSE_TRANSPORT: deferred — rmcp SSE transport not yet available in v0.17
ISSUES: []
```
