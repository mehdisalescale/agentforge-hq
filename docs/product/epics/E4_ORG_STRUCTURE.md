# Epic E4: Org Structure & Governance

> **Multi-company isolation, org charts, budgets, goals, and approval gates.**
>
> Source repo: Paperclip (orchestration concepts)

---

## Business Value

Users running multiple client engagements or projects need isolation. This epic adds companies (tenants), departments, org chart hierarchies, per-company budgets with enforcement, goal lineage (every task traces to mission), and approval gates for high-impact decisions. This is what makes AgentForge an "AI company builder," not just an "agent spawner."

## Acceptance Gate

**The epic is DONE when:**
1. Users can create multiple companies with isolated data
2. Agents are assigned to companies with department and reporting chain
3. Company budgets are enforced (agents pause when budget exhausted)
4. Goals cascade from mission → sub-goals → tasks
5. Approval gates block sensitive actions until approved
6. Existing single-user mode still works (default company auto-created)
7. 40+ tests covering CRUD, budget enforcement, goal lineage, approvals

---

## User Stories

### E4-S1: Company Entity & Multi-Tenancy

**As a** user managing multiple projects,
**I want** to create separate companies with isolated agent teams,
**So that** client work doesn't mix.

```gherkin
GIVEN I POST /api/v1/companies with { name, mission, budget_limit }
WHEN the company is created
THEN it has a unique ID, isolated namespace, and budget tracking

GIVEN two companies exist
WHEN I list agents for company A
THEN only company A's agents are returned (company B's are invisible)

GIVEN no companies exist (fresh install)
WHEN the server starts
THEN a "Default" company is auto-created
AND all existing agents are assigned to it (backward compat)
```

**Test Plan:**
- `test_create_company`
- `test_list_agents_isolated_by_company`
- `test_default_company_auto_created`
- `test_existing_agents_assigned_to_default`

---

### E4-S2: Department & Org Position Model

**As a** company owner,
**I want** to organize agents into departments with reporting chains,
**So that** I have a clear organizational structure.

```gherkin
GIVEN a company exists
WHEN I create departments (Engineering, Design, Marketing)
THEN each department has a name, description, and belongs to the company

GIVEN departments exist
WHEN I assign an agent to Engineering with reports_to: CEO agent
THEN an OrgPosition is created linking agent → department → manager

GIVEN an org chart
WHEN I GET /api/v1/companies/:id/org-chart
THEN I receive a tree: CEO → managers → ICs with agent details at each node
```

**Test Plan:**
- `test_create_department`
- `test_assign_agent_to_department`
- `test_org_chart_returns_tree`
- `test_reports_to_chain`

---

### E4-S3: Budget Enforcement

**As a** company owner,
**I want** per-company budgets enforced automatically,
**So that** agents can't overspend without my knowledge.

```gherkin
GIVEN a company with budget_limit = $100
WHEN an agent run would push total_cost to $95
THEN a BudgetWarning event is emitted (at 90% threshold)

GIVEN a company with budget_limit = $100 and total_cost = $100
WHEN an agent run is initiated
THEN the CompanyBudgetCheck middleware REJECTS the run
AND a BudgetExhausted event is emitted
AND the HTTP response is 402 with { error: "Company budget exhausted" }

GIVEN a company with no budget_limit set
WHEN agents run
THEN no budget enforcement occurs (unlimited)

GIVEN a company's budget is exhausted
WHEN the owner increases budget_limit
THEN agents can run again immediately
```

**Test Plan:**
- `test_budget_warning_at_90_percent`
- `test_budget_exhausted_rejects_run`
- `test_no_limit_allows_unlimited`
- `test_increase_budget_unblocks_agents`
- `test_budget_check_is_atomic` (no race condition)

---

### E4-S4: Goal Hierarchy

**As a** company owner,
**I want** to define goals that cascade from company mission to actionable tasks,
**So that** every agent action traces back to business objectives.

```gherkin
GIVEN a company with mission "Build an MVP SaaS product"
WHEN I create a goal "Implement user authentication"
THEN the goal is linked to the company with parent_goal: null (top-level)

GIVEN a top-level goal exists
WHEN I create a sub-goal "Write auth API endpoints" under it
THEN parent_goal_id points to the parent goal

GIVEN a goal hierarchy 3 levels deep
WHEN I GET /api/v1/companies/:id/goals?tree=true
THEN I receive the full tree with children nested

GIVEN a sub-goal is marked completed
WHEN all sibling sub-goals are also completed
THEN the parent goal status can be updated to completed
```

**Test Plan:**
- `test_create_top_level_goal`
- `test_create_sub_goal`
- `test_goal_tree_returns_nested`
- `test_goal_status_transitions`

---

### E4-S5: Approval Gates

**As a** company owner,
**I want** certain actions to require my approval,
**So that** I maintain control over high-impact decisions.

```gherkin
GIVEN approval_required is configured for "agent_hire"
WHEN an agent tries to hire a new sub-agent
THEN an Approval is created with status "pending"
AND an ApprovalRequested event is emitted
AND the action is BLOCKED until approved

GIVEN a pending approval
WHEN I POST /api/v1/approvals/:id/approve
THEN the approval status becomes "approved"
AND the blocked action proceeds
AND an ApprovalGranted event is emitted

GIVEN a pending approval
WHEN I POST /api/v1/approvals/:id/deny with { reason }
THEN the approval status becomes "denied"
AND the action does NOT proceed
AND an ApprovalDenied event is emitted

Approval types:
- agent_hire: Adding new agents to the company
- budget_increase: Requesting budget increase
- strategic_decision: Major direction changes
- external_action: Actions visible to others (git push, PR creation)
```

**Test Plan:**
- `test_approval_created_on_restricted_action`
- `test_approve_unblocks_action`
- `test_deny_blocks_action`
- `test_approval_event_emitted`
- `test_all_4_approval_types`

---

### E4-S6: Company Budget Middleware

**As a** system,
**I want** company budget checks as a middleware in the run pipeline,
**So that** enforcement is automatic and consistent.

```gherkin
GIVEN the CompanyBudgetCheck middleware is in the chain
WHEN a run request arrives for an agent in company X
THEN the middleware checks company X's remaining budget
AND allows or rejects based on budget_limit - budget_used

GIVEN the middleware runs
WHEN it allows the request
THEN it continues to the next middleware

GIVEN the middleware runs
WHEN it rejects the request
THEN it returns MiddlewareError::BudgetExceeded { company_id, cost, limit }
```

**Technical Notes:**
- New middleware: `CompanyBudgetCheckMiddleware`
- Position: after CircuitBreaker, before CostCheck (slot 3)
- Needs company_id lookup from agent_id

**Test Plan:**
- `test_allows_when_under_budget`
- `test_rejects_when_over_budget`
- `test_skips_when_no_company`

---

### E4-S7: Company API Endpoints

**As a** frontend or MCP client,
**I want** full CRUD for companies, departments, goals, and approvals.

```gherkin
Companies:
  POST   /api/v1/companies
  GET    /api/v1/companies
  GET    /api/v1/companies/:id
  PUT    /api/v1/companies/:id
  DELETE /api/v1/companies/:id

Departments:
  POST   /api/v1/companies/:id/departments
  GET    /api/v1/companies/:id/departments
  DELETE /api/v1/companies/:cid/departments/:did

Org Chart:
  GET    /api/v1/companies/:id/org-chart
  POST   /api/v1/companies/:id/org-chart/positions

Goals:
  POST   /api/v1/companies/:id/goals
  GET    /api/v1/companies/:id/goals
  PUT    /api/v1/goals/:id
  DELETE /api/v1/goals/:id

Approvals:
  GET    /api/v1/approvals?status=pending
  POST   /api/v1/approvals/:id/approve
  POST   /api/v1/approvals/:id/deny
```

**Test Plan:**
- Integration tests for each endpoint (happy path + validation errors)

---

### E4-S8–S12: Frontend Pages

*(Companies page, Org Chart visualization, Goals hierarchy, Approvals queue, Budget dashboard)*

**Acceptance criteria for each:**
- Page loads with correct data
- CRUD operations work from UI
- Empty states shown with helpful messages
- Loading states with skeletons
- Responsive layout

---

## DB Schema

```sql
-- 0011_org_charts.sql

CREATE TABLE companies (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  mission TEXT,
  budget_limit REAL,
  budget_used REAL NOT NULL DEFAULT 0.0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE departments (
  id TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX idx_dept_company_name ON departments(company_id, name);

CREATE TABLE org_positions (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
  company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  department_id TEXT REFERENCES departments(id) ON DELETE SET NULL,
  reports_to TEXT REFERENCES org_positions(id) ON DELETE SET NULL,
  role TEXT NOT NULL DEFAULT 'ic' CHECK(role IN ('ceo','manager','ic')),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX idx_org_agent ON org_positions(agent_id);

CREATE TABLE goals (
  id TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  parent_id TEXT REFERENCES goals(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','completed','blocked','cancelled')),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE approvals (
  id TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  approval_type TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','approved','denied')),
  requester_agent_id TEXT REFERENCES agents(id),
  data_json TEXT,
  reason TEXT,
  decided_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Add company_id to agents (nullable for backward compat)
ALTER TABLE agents ADD COLUMN company_id TEXT REFERENCES companies(id);
```

---

## Story Point Estimates

| Story | Points | Sprint |
|-------|--------|--------|
| E4-S1 Company & Tenancy | 5 | S3 |
| E4-S2 Departments & Org | 5 | S3 |
| E4-S3 Budget Enforcement | 5 | S3 |
| E4-S4 Goal Hierarchy | 3 | S3 |
| E4-S5 Approval Gates | 5 | S4 |
| E4-S6 Budget Middleware | 3 | S4 |
| E4-S7 API Endpoints | 5 | S4 |
| E4-S8 Companies Page | 3 | S4 |
| E4-S9 Org Chart Page | 5 | S4 |
| E4-S10 Goals Page | 3 | S4 |
| E4-S11 Approvals Page | 3 | S4 |
| E4-S12 Budget Dashboard | 3 | S4 |
| **Total** | **48** | |
