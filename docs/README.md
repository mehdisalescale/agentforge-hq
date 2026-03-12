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

## Where to Find High-Level Plans

The **authoritative source** for AgentForge strategy and planning lives one level up:

- Product vision & strategy → [`/docs/strategy/`](../../docs/strategy/)  
- Epics & requirements → [`/docs/product/`](../../docs/product/)  
- Expansion plan (absorbing 8 repos) → [`/docs/engineering/EXPANSION_PLAN.md`](../../docs/engineering/EXPANSION_PLAN.md)  
- Multi-agent dev system → [`/docs/engineering/MULTI_AGENT_DEVELOPMENT_SYSTEM.md`](../../docs/engineering/MULTI_AGENT_DEVELOPMENT_SYSTEM.md)  
- Current sprint plan → [`/docs/sprints/SPRINT_PLAN.md`](../../docs/sprints/SPRINT_PLAN.md)  

This `forge-project/docs` folder should only contain **implementation-level** docs: how we apply those plans to this Rust + Svelte codebase.
