# Agent W5-C: Agent Cards Enrichment + Health Check

> You are Agent W5-C. Your job: (1) enrich agent cards on the Agents page with run count, last run time, total cost, and persona details, and (2) add CLI health detection to the health endpoint + a banner in the UI.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
frontend/src/routes/agents/+page.svelte            — FULL FILE (agents page)
frontend/src/routes/+layout.svelte                 — FULL FILE (layout with sidebar)
frontend/src/lib/api.ts                            — FULL FILE (API client)
crates/forge-api/src/routes/agents.rs              — agents API
crates/forge-api/src/routes/health.rs              — FULL FILE (health endpoint)
crates/forge-db/src/repos/agents.rs                — AgentRepo, Agent struct
crates/forge-db/src/repos/sessions.rs              — SessionRepo
```

## Step 2: Enrich Agent Cards

### 2a. Backend: Add agent stats endpoint

In `crates/forge-api/src/routes/agents.rs`, add an endpoint that returns per-agent statistics:

```rust
/// GET /api/v1/agents/:id/stats
#[derive(Debug, Serialize)]
pub struct AgentStats {
    pub run_count: i64,
    pub last_run: Option<String>,
    pub total_cost: f64,
    pub success_rate: f64,
}

async fn agent_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentStats>, axum::response::Response> {
    let agent_id = AgentId(parse_uuid(&id)?);
    // Query sessions table for this agent
    let sessions = state.session_repo.list_by_agent(&agent_id).map_err(api_error)?;
    let run_count = sessions.len() as i64;
    let last_run = sessions.iter().map(|s| &s.created_at).max().cloned();
    let total_cost: f64 = sessions.iter().map(|s| s.cost_usd).sum();
    let completed = sessions.iter().filter(|s| s.status == "completed").count();
    let success_rate = if run_count > 0 { completed as f64 / run_count as f64 * 100.0 } else { 0.0 };

    Ok(Json(AgentStats { run_count, last_run, total_cost, success_rate }))
}
```

Check if `SessionRepo` has `list_by_agent`. If not, use a raw query or filter from `list()`. If the agent has too many sessions, this is fine for now.

Register: `.route("/agents/:id/stats", get(agent_stats))`

**Alternative (simpler)**: Add a bulk stats endpoint that returns stats for all agents at once:
```rust
/// GET /api/v1/agents/stats — returns stats for all agents
```
This avoids N+1 API calls from the frontend. Use whichever approach is cleaner.

### 2b. Frontend: Show stats on agent cards

In `frontend/src/routes/agents/+page.svelte`, after loading agents, fetch stats:

```typescript
// After loadAgents():
let agentStats = $state<Record<string, AgentStats>>({});

async function loadStats() {
  for (const agent of agents) {
    try {
      const res = await fetch(`/api/v1/agents/${agent.id}/stats`);
      if (res.ok) {
        agentStats[agent.id] = await res.json();
      }
    } catch { /* ignore */ }
  }
}
```

Or with bulk endpoint:
```typescript
const statsRes = await fetch('/api/v1/agents/stats');
if (statsRes.ok) agentStats = await statsRes.json();
```

On each agent card, add a stats row below the existing content:

```svelte
{#if agentStats[agent.id]}
  {@const stats = agentStats[agent.id]}
  <div class="agent-stats">
    <span class="stat"><strong>{stats.run_count}</strong> runs</span>
    <span class="stat"><strong>${stats.total_cost.toFixed(4)}</strong> cost</span>
    {#if stats.last_run}
      <span class="stat">Last: {new Date(stats.last_run).toLocaleDateString()}</span>
    {/if}
  </div>
{/if}
```

Style `.agent-stats` as a row of small chips/badges below the card content.

## Step 3: Health Check — CLI Detection

### 3a. Backend: Enhance health endpoint

In `crates/forge-api/src/routes/health.rs`, add CLI detection:

```rust
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub cli_available: bool,
    pub cli_command: String,
}

pub async fn health() -> Json<HealthResponse> {
    let cli_command = std::env::var("FORGE_CLI_COMMAND").unwrap_or_else(|_| "claude".into());
    let cli_available = std::process::Command::new(&cli_command)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    Json(HealthResponse {
        status: if cli_available { "ok".into() } else { "degraded".into() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime_secs(),
        cli_available,
        cli_command,
    })
}
```

### 3b. Frontend: Health banner

In `frontend/src/routes/+layout.svelte`, add a health check on mount:

```typescript
let healthWarning = $state<string | null>(null);

onMount(async () => {
  try {
    const res = await fetch('/api/v1/health');
    if (res.ok) {
      const health = await res.json();
      if (!health.cli_available) {
        healthWarning = `CLI "${health.cli_command}" not found. Agent runs will fail. Install it or set FORGE_CLI_COMMAND.`;
      }
    }
  } catch { /* server not reachable — ignore, SPA handles this */ }
});
```

Render as a banner at the top of the page:

```svelte
{#if healthWarning}
  <div class="health-banner">
    <span>⚠ {healthWarning}</span>
    <button onclick={() => healthWarning = null}>Dismiss</button>
  </div>
{/if}
```

Style:
```css
.health-banner {
  background: #7c2d12;
  color: #fed7aa;
  padding: 0.5rem 1rem;
  font-size: 0.8rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
```

## Step 4: Add API Client Types

In `frontend/src/lib/api.ts`, add:

```typescript
export interface AgentStats {
  run_count: number;
  last_run: string | null;
  total_cost: number;
  success_rate: number;
}

export async function getAgentStats(id: string): Promise<AgentStats> {
  const res = await fetch(`${BASE}/agents/${id}/stats`);
  if (!res.ok) throw new Error(`Failed: ${res.status}`);
  return res.json();
}
```

## Step 5: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-api 2>&1         # all tests pass
cd frontend && pnpm build 2>&1       # must build cleanly
```

## Rules

- Modify: `crates/forge-api/src/routes/agents.rs` — add stats endpoint
- Modify: `crates/forge-api/src/routes/health.rs` — add CLI detection
- Modify: `frontend/src/routes/agents/+page.svelte` — show stats on cards
- Modify: `frontend/src/routes/+layout.svelte` — health warning banner
- Modify: `frontend/src/lib/api.ts` — add AgentStats type and function
- Do NOT modify middleware.rs, run.rs, hooks.rs
- Do NOT modify analytics page — Agent W5-B handles that
- Do NOT modify forge-mcp-bin — Agent W5-A handles that
- Do NOT modify forge-db repos — use what exists
- Do NOT modify existing tests — only add new ones
- Commit with: `feat: enrich agent cards with stats and add CLI health check banner`

## Report

When done, append your report here:

```
STATUS: complete
FILES_MODIFIED: [
  crates/forge-api/src/routes/agents.rs,
  crates/forge-api/src/routes/health.rs,
  frontend/src/lib/api.ts,
  frontend/src/routes/agents/+page.svelte,
  frontend/src/routes/+layout.svelte
]
AGENT_STATS_ENDPOINT: both (bulk GET /agents/stats + per-agent GET /agents/:id/stats)
HEALTH_CLI_CHECK: yes
HEALTH_BANNER: yes
ISSUES: []
```
