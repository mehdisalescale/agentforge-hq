# Agent R1-A: Documentation Consistency Sweep

> Fix all site-docs inconsistencies against actual codebase. You touch ONLY files under `site-docs/`. Zero backend changes.

## Step 1: Read Context

Read these files to understand what the code actually has:

- `CLAUDE.md` — project conventions
- `crates/forge-mcp-bin/src/main.rs` — count all `#[tool]` functions (currently 19)
- `crates/forge-core/src/events.rs` — count all `ForgeEvent` enum variants (currently ~43)
- `crates/forge-api/src/routes/run.rs` lines 82–123 — the actual middleware chain order
- `crates/forge-api/src/routes/mod.rs` — all route modules
- `crates/forge-api/src/routes/org.rs` — org routes (companies, departments, org-positions, org-chart)
- `crates/forge-api/src/routes/governance.rs` — goals + approvals routes
- `crates/forge-api/src/routes/personas.rs` — persona routes
- `crates/forge-api/src/routes/health.rs` — health route
- `crates/forge-api/src/routes/agents.rs` — agent routes including stats
- `crates/forge-api/src/routes/analytics.rs` — analytics routes

## Step 2: Fix MCP Tool Count + Reference

File: `site-docs/reference/mcp-tools.md`

1. Count the exact number of `#[tool]` functions in `forge-mcp-bin/src/main.rs`
2. Update the tool count in the doc
3. List all tools grouped by category:
   - **Agent CRUD:** agent_list, agent_get, agent_create, agent_update, agent_delete
   - **Session CRUD:** session_list, session_get, session_create, session_delete, session_export
   - **Intelligence:** forge_classify_task
   - **Workforce:** forge_list_personas, forge_hire_persona
   - **Governance:** forge_get_budget, forge_request_approval, forge_check_approval, forge_list_goals
   - **Analytics:** forge_get_session_events, forge_get_analytics

Also update `site-docs/architecture/mcp.md` — remove any HTTP SSE transport claims. Only stdio is implemented.

## Step 3: Fix Event Count + List

File: `site-docs/architecture/events.md`

1. Count the exact `ForgeEvent` variants from `crates/forge-core/src/events.rs`
2. Replace the doc's variant list with the actual list, grouped by category
3. Update the count (it was 38 in docs, actual is ~43)

## Step 4: Fix Middleware Chain

File: `site-docs/architecture/middleware.md`

The canonical chain from `run.rs` is 7 stages:
1. **RateLimitMiddleware** — token bucket
2. **CircuitBreakerMiddleware** — 3-state FSM
3. **CostCheckMiddleware** — budget warn/limit
4. **GovernanceMiddleware** — org hierarchy + approval checks
5. **SecurityScanMiddleware** — security event emission
6. **PersistMiddleware** — session state + event bus
7. **SpawnMiddleware** — CLI invocation (includes AgentConfigurator internally)

Remove references to standalone SkillInjection, TaskTypeDetection. They are now absorbed into AgentConfigurator inside SpawnMiddleware.

Also fix `site-docs/development/wave-history.md` — update middleware count to 7.

## Step 5: Add Missing API Routes

File: `site-docs/reference/api.md`

Read each route module in `crates/forge-api/src/routes/` and add every registered route. The doc is missing at least:

- `GET /api/v1/companies/:id`
- `PATCH /api/v1/companies/:id`
- `DELETE /api/v1/companies/:id`
- `GET /api/v1/departments/:id`
- `PATCH /api/v1/departments/:id`
- `DELETE /api/v1/departments/:id`
- `GET /api/v1/agents/stats`
- `GET /api/v1/agents/:id/stats`
- `GET /api/v1/personas/divisions`
- `GET /api/v1/health`

Read each route file to find the EXACT routes registered. Document all of them.

## Step 6: Fix Remaining Doc Issues

1. **`site-docs/reference/personas.md`** — clarify that personas are loaded from TOML files → parsed by forge-persona → seeded into DB via migration → served via API. Verify count if possible.

2. **`site-docs/development/wave-history.md`** — fix frontend page count (actual: 16 routes in `frontend/src/routes/`).

3. **`site-docs/strategy/gaps.md`** — update resolution status:
   - Skills page: RESOLVED (has category filter + content)
   - Memory page: RESOLVED (functional UI)
   - Workflows: PARTIAL (has UI)
   - Hooks: STUB
   - Schedules: STUB
   - Settings: RESOLVED (displays config)

4. **`site-docs/development/building.md`** and **`site-docs/getting-started/quickstart.md`** — replace all `npm` with `pnpm`.

5. **`site-docs/getting-started/configuration.md`** and **`site-docs/reference/env-vars.md`** — ensure DB path reference is `~/.agentforge/forge.db` (the canonical brand-aligned path).

## Rules

- Touch ONLY files under `site-docs/`
- Do NOT modify any Rust code, frontend code, or root-level docs
- Every number you write must come from reading actual source code, not from existing docs
- Run `mkdocs build --strict` at the end to verify the site builds cleanly (if mkdocs is available)

## Report

When done, create `docs/agents/REPORT_R1A.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
FILES_MODIFIED: [list]
ISSUES_FOUND: [any discrepancies between plan and reality]
COUNTS_VERIFIED:
  - MCP tools: X
  - ForgeEvent variants: X
  - Middleware stages: 7
  - API routes documented: X
  - Frontend pages: X
```
