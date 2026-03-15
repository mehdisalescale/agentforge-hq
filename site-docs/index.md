# AgentForge HQ

**Rust-native AI workforce and governance layer on top of Claude Code and MCP.**

Not another agent runtime — the management layer that makes AI teams accountable, observable, and composable from any tool.

---

## What It Does

- Browse **100+ pre-built AI personas** across 11 divisions
- Arrange agents in **org charts** with companies, departments, and budgets
- **Hire personas** into companies — creates agents + org positions with one click
- Spawn Claude Code agents with governance controls (budgets, approvals, goals)
- **19 MCP tools** — any MCP client (Claude Code, Cursor, ADK) can use AgentForge as infrastructure
- Real-time analytics, cost tracking, security scanning, audit trail

## Architecture at a Glance

```
┌─────────────────────────────────────────┐
│           UPSTREAM CLIENTS              │
│  Claude Code · Cursor · ADK · Scripts   │
└──────────────┬──────────────────────────┘
               │ MCP tools (19)
┌──────────────▼──────────────────────────┐
│  AGENTFORGE (Orchestrator + Governor)   │
│                                         │
│  Workforce    Governance   Observation  │
│  112 personas Budgets      Event capture│
│  Org charts   Approvals    Cost tracking│
│  Hire flow    Goal inject  Audit log    │
│                                         │
│  AgentConfigurator → CLAUDE.md per agent│
│  HookReceiver ← Claude Code events     │
└──────────────┬──────────────────────────┘
               │ spawns + configures
┌──────────────▼──────────────────────────┐
│  CLAUDE CODE INSTANCES (workers)        │
│  Each with: persona CLAUDE.md, hooks,   │
│  scoped tools, git worktree isolation   │
└─────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum, SQLite (WAL mode), tokio |
| Frontend | SvelteKit 5, TailwindCSS 4, embedded via rust-embed |
| MCP | rmcp v0.17 (official Rust SDK), stdio transport |
| Safety | Circuit breaker, rate limiter, CostTracker, SecurityScanner |
| Delivery | Single binary, zero runtime dependencies |

## Quick Links

- [Quick Start](getting-started/quickstart.md) — get running in 60 seconds
- [Architecture Overview](architecture/overview.md) — 13 crates, how they fit
- [MCP Tools](reference/mcp-tools.md) — 19 tools for AI clients
- [API Reference](reference/api.md) — full HTTP API
- [Executive Summary](strategy/executive-summary.md) — where we are and where we're going
