# Agent W5-B: Analytics Dashboard Enrichment + Company-Scoped Analytics

> You are Agent W5-B. Your job: make the analytics page show real, useful data — add company-scoped filtering, per-agent cost breakdown with agent names (not UUIDs), success rate visualization, and cost trend chart. Also add a company-scoped analytics API endpoint.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
frontend/src/routes/analytics/+page.svelte         — FULL FILE (current analytics page)
frontend/src/lib/api.ts                            — FULL FILE (API client)
crates/forge-api/src/routes/analytics.rs           — FULL FILE (analytics API)
crates/forge-db/src/repos/analytics.rs             — FULL FILE (AnalyticsRepo + tests)
crates/forge-db/src/repos/companies.rs             — CompanyRepo
crates/forge-db/src/repos/agents.rs                — AgentRepo
```

## Step 2: Add Company-Scoped Analytics Endpoint

In `crates/forge-api/src/routes/analytics.rs`, add:

```rust
#[derive(Debug, Deserialize)]
pub struct CompanyUsageQuery {
    pub company_id: String,
    pub start: Option<String>,
    pub end: Option<String>,
}

/// GET /api/v1/analytics/company?company_id=...&start=...&end=...
async fn company_usage_report(
    State(state): State<AppState>,
    Query(query): Query<CompanyUsageQuery>,
) -> Result<Json<CompanyUsageReport>, axum::response::Response> {
    // 1. Get all agents in this company via org_positions
    // 2. Filter analytics to only those agents
    // 3. Include company budget info
}
```

Add `CompanyUsageReport` type that includes:
- Everything from `UsageReport`
- `company_name: String`
- `budget_limit: Option<f64>`
- `budget_used: f64`
- `budget_remaining: Option<f64>`

Register: `.route("/analytics/company", get(company_usage_report))`

If this is too complex to wire through the repos, a simpler approach:
- Just add `company_id` as optional filter to the existing `UsageQuery`
- Filter agent_breakdown results by agents belonging to that company

## Step 3: Enrich Analytics Frontend

In `frontend/src/routes/analytics/+page.svelte`, enhance:

### 3a. Company filter dropdown
```svelte
<select bind:value={selectedCompany} onchange={loadReport}>
  <option value="">All Companies</option>
  {#each companies as company}
    <option value={company.id}>{company.name}</option>
  {/each}
</select>
```

Load companies list on mount: `const companies = await listCompanies();`

### 3b. Summary cards row
Show 4 cards at the top:
- **Total Cost** — `report.total_cost` formatted as USD
- **Sessions** — `report.stats.total` with completed/failed breakdown
- **Success Rate** — `(report.stats.completed / report.stats.total * 100)%`
- **Projected Monthly** — `report.projected_monthly_cost`

```svelte
<div class="summary-cards">
  <div class="card">
    <span class="card-label">Total Cost</span>
    <span class="card-value">{formatCost(report.total_cost)}</span>
  </div>
  <!-- ... -->
</div>
```

### 3c. Agent breakdown with names
Currently `agent_breakdown` shows agent UUIDs. Resolve to names:

```svelte
{#each report.agent_breakdown as agent}
  <div class="agent-row">
    <span class="agent-name">{agentNames[agent.agent_id] ?? agent.agent_id.slice(0,8)}</span>
    <span class="agent-cost">{formatCost(agent.total_cost)}</span>
    <div class="bar" style="width: {barWidth(agent.total_cost, maxAgentCost)}"></div>
  </div>
{/each}
```

Load agent names on mount:
```typescript
const agentList = await listAgents();
const agentNames: Record<string, string> = {};
for (const a of agentList) {
  agentNames[a.id] = a.name;
}
```

### 3d. Cost trend chart (CSS bars)
The daily_costs data already exists. Make sure the bar chart renders visually with date labels:

```svelte
<div class="daily-chart">
  {#each report.daily_costs as day}
    <div class="day-bar">
      <div class="bar-fill" style="height: {barWidth(day.cost, maxDailyCost)}"></div>
      <span class="day-label">{day.date.slice(5)}</span>
    </div>
  {/each}
</div>
```

### 3e. Empty state
If no sessions exist, show a helpful message instead of empty charts:

```svelte
{#if report && report.stats.total === 0}
  <div class="empty-state">
    <p>No sessions recorded yet. Run an agent to see analytics here.</p>
  </div>
{/if}
```

## Step 4: Add API Client Function

In `frontend/src/lib/api.ts`, check if `listCompanies` exists. If not, add it. Also check `getUsageAnalytics` signature matches the backend.

## Step 5: Style the Dashboard

Use existing CSS variable patterns from other pages. Key classes:
- `.summary-cards` — flex row, 4 cards
- `.card` — `var(--surface)` background, `var(--border)` border
- `.agent-row` — flex row with name, cost, bar
- `.daily-chart` — flex row, bottom-aligned bars

Keep it minimal — CSS only, no chart libraries.

## Step 6: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cd frontend && pnpm build 2>&1       # must build cleanly
```

## Rules

- Modify: `frontend/src/routes/analytics/+page.svelte` — enrich dashboard
- Modify: `crates/forge-api/src/routes/analytics.rs` — add company-scoped endpoint (optional)
- You may modify: `frontend/src/lib/api.ts` — add missing API functions
- Do NOT modify middleware.rs, run.rs, hooks.rs — other agents handle those
- Do NOT modify forge-db repos — use what exists
- Do NOT modify agents page or sessions page
- Do NOT modify existing tests — only add new ones
- Commit with: `feat(analytics): enrich dashboard with summary cards, agent names, and company filter`

## Report

When done, append your report here:

```
STATUS: done
FILES_MODIFIED: [
  crates/forge-api/src/routes/analytics.rs,
  frontend/src/lib/api.ts,
  frontend/src/routes/analytics/+page.svelte
]
SUMMARY_CARDS: [Total Cost, Sessions (with completed/failed breakdown), Success Rate, Projected Monthly]
COMPANY_FILTER: yes
AGENT_NAMES_RESOLVED: yes
ISSUES: [forge-mcp-bin has pre-existing compilation error unrelated to this work]
```
