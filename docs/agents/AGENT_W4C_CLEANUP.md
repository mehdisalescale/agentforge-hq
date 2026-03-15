# Agent W4-C: Middleware Cleanup + MCP Tool Expansion

> You are Agent W4-C. Your job: (1) add a forge_classify_task MCP tool so task classification is available on-demand instead of auto-injected, and (2) update NORTH_STAR.md and CLAUDE.md to reflect the new architecture.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules (will be updated)
NORTH_STAR.md                                      — current state (will be updated)
crates/forge-mcp-bin/src/main.rs                   — FULL FILE (current MCP tools)
crates/forge-process/src/task_type.rs              — TaskTypeDetector
crates/forge-process/src/skill_router.rs           — SkillRouter
stat-qou-plan/REVISED_PLAN.md                      — what we're doing and why
stat-qou-plan/ARCHITECTURE_RETHINK.md              — the vision
```

## Step 2: Add MCP Tool — forge_classify_task

In `crates/forge-mcp-bin/src/main.rs`, add a new tool that exposes task classification as an on-demand MCP tool (replacing the auto-injecting TaskTypeDetectionMiddleware):

### Parameter type:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClassifyTaskParam {
    #[schemars(description = "The prompt or task description to classify")]
    prompt: String,
}
```

### Tool implementation:

```rust
#[tool(
    name = "forge_classify_task",
    description = "Classify a task/prompt into a type (BugFix, Feature, Refactor, Test, Review, Documentation, Architecture, Security, Performance, Deploy, Research, General) and get recommended skills"
)]
async fn classify_task(&self, #[tool(params)] p: ClassifyTaskParam) -> Result<CallToolResult, ErrorData> {
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
```

### Add to tool_router! macro:

Find the existing `tool_router!` call and add `classify_task` to the list.

## Step 3: Add MCP Tool — forge_list_personas

Add a tool to list personas from the catalog (this is the beginning of the workforce MCP surface):

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPersonasParam {
    #[schemars(description = "Optional division filter (e.g. 'engineering', 'security', 'product')")]
    division: Option<String>,
    #[schemars(description = "Optional search term to filter by name or description")]
    search: Option<String>,
}
```

For this tool, you'll need access to `PersonaRepo`. Check if the MCP binary already has it. If not:
1. Add `forge-persona` as a dependency in `crates/forge-mcp-bin/Cargo.toml`
2. Add `persona_repo: Arc<PersonaRepo>` to the `ForgeMcp` struct
3. Initialize it in `main()` the same way agent_repo is initialized

```rust
#[tool(
    name = "forge_list_personas",
    description = "List available AI personas from the catalog. Filter by division or search term."
)]
async fn list_personas(&self, #[tool(params)] p: ListPersonasParam) -> Result<CallToolResult, ErrorData> {
    let personas = self.persona_repo.list()
        .map_err(|e| ErrorData::internal_error(format!("Failed to list personas: {}", e), None))?;

    let filtered: Vec<_> = personas.iter()
        .filter(|p_item| {
            if let Some(ref div) = p.division {
                if !p_item.division.to_lowercase().contains(&div.to_lowercase()) {
                    return false;
                }
            }
            if let Some(ref search) = p.search {
                let s = search.to_lowercase();
                if !p_item.name.to_lowercase().contains(&s)
                    && !p_item.title.to_lowercase().contains(&s) {
                    return false;
                }
            }
            true
        })
        .map(|p_item| serde_json::json!({
            "id": p_item.id,
            "name": p_item.name,
            "title": p_item.title,
            "division": p_item.division,
        }))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&filtered).unwrap_or_default(),
    )]))
}
```

## Step 4: Add MCP Tool — forge_get_budget

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBudgetParam {
    #[schemars(description = "UUID of the company")]
    company_id: String,
}

#[tool(
    name = "forge_get_budget",
    description = "Get budget status for a company — remaining, used, and limit"
)]
async fn get_budget(&self, #[tool(params)] p: GetBudgetParam) -> Result<CallToolResult, ErrorData> {
    let company = self.company_repo.get(&p.company_id)
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
```

For this you'll need `CompanyRepo`. Add `forge-db` dependency if not already there, add `company_repo: Arc<CompanyRepo>` to `ForgeMcp` struct.

## Step 5: Update NORTH_STAR.md

Update the "Current State" section to reflect:
- Wave 3 completed (sidebar cleanup, governance wiring, session output, page verification)
- Wave 4 in progress (AgentConfigurator, HookReceiver, MCP expansion)
- Architecture direction: configure→execute→observe loop
- Middleware chain reduced from 9 to 6-7
- MCP tools expanding from 10 to 13+

Update the version history to add the Wave 3/4 entries.

## Step 6: Update CLAUDE.md

Update:
- Add the three new MCP tools to the API list
- Note the new HookReceiver endpoints (POST /api/v1/hooks/pre-tool, post-tool, stop)
- Note the AgentConfigurator concept
- Update middleware chain description

## Step 7: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test 2>&1 | grep "FAILED"     # no failures
```

## Rules

- Modify: `crates/forge-mcp-bin/src/main.rs` — add 3 new MCP tools
- Modify: `crates/forge-mcp-bin/Cargo.toml` — add dependencies if needed
- Modify: `NORTH_STAR.md` — update current state
- Modify: `CLAUDE.md` — update project context
- Do NOT modify middleware.rs — Agent W4-A handles that
- Do NOT modify hooks.rs — Agent W4-B handles that
- Do NOT modify run.rs — Agent W4-A handles that
- Do NOT touch frontend files
- Do NOT touch main.rs (in forge-app)
- Do NOT modify existing tests — only add new ones
- Commit with: `feat(mcp): add classify_task, list_personas, get_budget MCP tools`

## Report

When done, append your report here:

```
STATUS: complete
FILES_MODIFIED: [
  "crates/forge-mcp-bin/src/main.rs",
  "crates/forge-mcp-bin/Cargo.toml",
  "NORTH_STAR.md",
  "CLAUDE.md"
]
MCP_TOOLS_ADDED: [
  "forge_classify_task",
  "forge_list_personas",
  "forge_get_budget"
]
MCP_TOOLS_TOTAL: 13
DOCS_UPDATED: [
  "NORTH_STAR.md — updated current state to reflect Wave 3 complete, Wave 4 in progress; version history; MCP tool count 10→13",
  "CLAUDE.md — added MCP Tools section (13 tools), Wave 4 Architecture section (AgentConfigurator, HookReceiver, middleware simplification)"
]
ISSUES: []
NOTES:
  - Adapted brief's code to match actual model fields (short_description instead of title, division_slug instead of division)
  - PersonaRepo.list() already supports division_slug and search params natively — used them directly
  - Added forge-process and forge-persona as dependencies to forge-mcp-bin
  - Added persona_repo and company_repo to ForgeMcp struct and constructor
  - cargo check: 0 warnings
  - cargo test: 0 failures
```
