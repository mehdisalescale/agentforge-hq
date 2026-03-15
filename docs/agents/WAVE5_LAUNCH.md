# Wave 5 Launch: Observable + MCP-Complete

> 3 agents, zero file conflicts. Run after Wave 4 commits are in.

## What This Wave Does

- **W5-A**: Expand MCP to 15+ tools (governance, analytics, workforce) + attempt HTTP SSE transport
- **W5-B**: Enrich analytics dashboard (summary cards, agent names, company filter, cost chart)
- **W5-C**: Agent card stats (run count, cost, last run) + CLI health check with UI banner

## Conflict Matrix

| File | W5-A | W5-B | W5-C |
|------|------|------|------|
| `crates/forge-mcp-bin/src/main.rs` | MODIFY | — | — |
| `crates/forge-mcp-bin/Cargo.toml` | MODIFY | — | — |
| `crates/forge-api/src/routes/analytics.rs` | — | MODIFY | — |
| `frontend/src/routes/analytics/+page.svelte` | — | MODIFY | — |
| `crates/forge-api/src/routes/agents.rs` | — | — | MODIFY |
| `crates/forge-api/src/routes/health.rs` | — | — | MODIFY |
| `frontend/src/routes/agents/+page.svelte` | — | — | MODIFY |
| `frontend/src/routes/+layout.svelte` | — | — | MODIFY |
| `frontend/src/lib/api.ts` | — | MAY MODIFY | MAY MODIFY |

**api.ts** is the only potential overlap — both W5-B and W5-C may add functions. This is low-risk since they add different functions. If git conflict occurs, merge manually.

## How to Launch

### Tab 1 — Agent W5-A
```
You are Agent W5-A. Read docs/agents/AGENT_W5A_MCP_EXPAND.md — it contains your full instructions. Execute every step: add MCP tools for governance, analytics, and workforce. Verify with cargo check. When done, append your report to the file and commit.
```

### Tab 2 — Agent W5-B
```
You are Agent W5-B. Read docs/agents/AGENT_W5B_ANALYTICS.md — it contains your full instructions. Execute every step: enrich the analytics dashboard with summary cards, agent names, company filter. Verify with cargo check and pnpm build. When done, append your report to the file and commit.
```

### Tab 3 — Agent W5-C
```
You are Agent W5-C. Read docs/agents/AGENT_W5C_AGENTS_HEALTH.md — it contains your full instructions. Execute every step: add agent stats endpoint, enrich agent cards, add CLI health check with banner. Verify with cargo check and pnpm build. When done, append your report to the file and commit.
```

## Result After Wave 5

- 15+ MCP tools covering full workforce/governance/observability API
- Analytics dashboard with real summary cards, cost trends, agent name resolution
- Agent cards show run count, total cost, last run date
- Health check detects missing CLI, warns user with banner
- AgentForge is observable, governable, and MCP-complete
