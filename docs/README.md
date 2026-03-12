# Claude Forge — Technical Documentation

> Implementation-specific docs for the Rust + Svelte single-binary orchestrator.  
> For project-wide AgentForge docs (vision, epics, sprints, research), start at [`/docs/INDEX.md`](../../docs/INDEX.md).

## Architecture

- [MCP Design](MCP_DESIGN.md) — MCP server architecture (rmcp, stdio transport, 10 tools)
- [E2E Smoke Test](E2E_SMOKE_TEST.md) — End-to-end test documentation

## Crate Reference

| Crate | Purpose |
|-------|---------|
| `forge-app` | Binary entry point, DB setup, API server, cron scheduler |
| `forge-api` | Axum HTTP + WebSocket, routes, CORS, rust-embed SPA |
| `forge-process` | Spawn Claude CLI, stream-json parsing, ConcurrentRunner, LoopDetector |
| `forge-agent` | Agent model, 10 presets, validation |
| `forge-db` | SQLite WAL, 8 migrations, 12 repos, BatchWriter |
| `forge-core` | ForgeEvent (35 variants), EventBus, shared types, error hierarchy |
| `forge-safety` | CircuitBreaker, RateLimiter, CostTracker |
| `forge-git` | Git worktree create/remove/list for multi-agent isolation |
| `forge-mcp-bin` | MCP stdio server (rmcp, 10 tools, 5 resources) |

## Epic 1 — Implemented Surfaces (Org + Personas + Governance)

These are the concrete slices implemented in this codebase for Epic 1:

- **Org & Company**
  - DB: `forge-db::companies`, `departments`, `org_positions`, `goals`, `approvals`
  - Domain: `forge-org` (`model`, `service::build_org_chart`)
  - API: `forge-api::routes::org` (`/companies`, `/departments`, `/org-positions`, `/org-chart`)
  - Frontend: `frontend/src/routes/companies/+page.svelte`, `frontend/src/routes/org-chart/+page.svelte`

- **Persona Catalog**
  - DB: `forge-db::repos::personas` + migration `0009_personas.sql`
  - Domain: `forge-persona` (`model`, `parser`, `catalog`)
  - API: `forge-api::routes::personas` (`GET /personas`, `GET /personas/:id`, `POST /personas/:id` for hire)
  - Frontend: `frontend/src/routes/personas/+page.svelte`

- **Governance (Goals & Approvals)**
  - DB: `goals`, `approvals` tables (migration `0011_org_charts.sql`)
  - Repos: `forge-db::repos::goals`, `forge-db::repos::approvals`
  - API: `forge-api::routes::governance` (`/goals`, `/approvals`)
  - Frontend: `frontend/src/routes/goals/+page.svelte`, `frontend/src/routes/approvals/+page.svelte`

For higher-level intent and story breakdown for Epic 1, see `/docs/engineering/EPIC1_FOUNDATION_TASKS.md` in the workspace root; this section exists only to show how that plan maps onto concrete code and endpoints here.

## Where to Find High-Level Plans

The **authoritative source** for AgentForge strategy and planning lives one level up:

- Product vision & strategy → [`/docs/strategy/`](../../docs/strategy/)  
- Epics & requirements → [`/docs/product/`](../../docs/product/)  
- Expansion plan (absorbing 8 repos) → [`/docs/engineering/EXPANSION_PLAN.md`](../../docs/engineering/EXPANSION_PLAN.md)  
- Multi-agent dev system → [`/docs/engineering/MULTI_AGENT_DEVELOPMENT_SYSTEM.md`](../../docs/engineering/MULTI_AGENT_DEVELOPMENT_SYSTEM.md)  
- Current sprint plan → [`/docs/sprints/SPRINT_PLAN.md`](../../docs/sprints/SPRINT_PLAN.md)  

This `forge-project/docs` folder should only contain **implementation-level** docs: how we apply those plans to this Rust + Svelte codebase.
