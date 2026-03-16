# AgentForge HQ — North Star

> **Read this first in every session.** This is the single source of truth for the core app.
> Last updated: 2026-03-16
>
> **Repo:** [mehdisalescale/agentforge-hq](https://github.com/mehdisalescale/agentforge-hq)

---

## What We're Building

A self-improving AI workforce platform: Rust/Axum + Svelte 5, single binary.
Unifies 8 open-source repos into one product. The only Rust-native tool in the space.

**One-liner**: Browse 100+ AI personas, hire them into org charts, let them execute real work with budgets and governance. One binary, zero deps.

---

## Current State (2026-03-15)

### What's Implemented

**12 workspace crates + 1 MCP stubs crate (13 total):**

| Crate | What |
|-------|------|
| forge-core | ForgeEvent (43 variants), EventBus fan-out (mpsc + broadcast), ForgeError, typed IDs |
| forge-agent | 10 presets (incl. Coordinator), Agent/NewAgent/UpdateAgent, validation |
| forge-db | SQLite WAL, 13 migrations (0001–0014), 17 repos, BatchWriter (50/2s), UnitOfWork, r2d2 pool |
| forge-process | Claude CLI spawn, stream-json, ConcurrentRunner, LoopDetector, ProcessBackend trait, BackendRegistry |
| forge-safety | CircuitBreaker (3-state FSM, persistent), RateLimiter (token bucket), CostTracker |
| forge-api | Full HTTP API + WebSocket, CORS, TraceLayer, rust-embed SPA, middleware chain |
| forge-app | Binary wiring, graceful shutdown, env config, skill loading, cron scheduler |
| forge-git | Worktree create/remove/list for multi-agent isolation |
| forge-mcp | MCP protocol stubs |
| forge-mcp-bin | MCP stdio server (rmcp, 21 tools) |
| **forge-org** | Company, Department, OrgPosition models + org chart builder |
| **forge-persona** | Persona catalog (100+ personas, 11 divisions), parser, hire flow |
| **forge-governance** | Goal and Approval models |

**Frontend (16+ pages, all Svelte 5 $state runes):**
- Dashboard, Agents, Sessions, Memory, Hooks, Skills, Workflows, Settings, Schedules, Analytics
- **Companies** — manage companies (name, mission, budget)
- **Org Chart** — visualize org hierarchy per company
- **Personas** — browse catalog, hire into company/org
- **Goals** — define and update per-company goals
- **Approvals** — governance review and resolve

**Epic 1 Baseline (Org + Personas + Governance):**
- Backend APIs: companies, departments, org-positions, org-chart, personas (with hire), goals, approvals
- DB: migrations 0009 (personas), 0011 (org charts), 0012 (agents.persona_id)
- Full frontend pages for the complete flow

**Wave 3 (completed 2026-03-14):**
- Sidebar cleanup (removed non-functional pages: Workflows, Memory, Hooks, Schedules)
- Governance wiring (budget enforcement, goal injection, approval visibility)
- Session output viewing for past sessions
- Page verification (Skills, Analytics, Settings pages verified functional)

**Wave 4 (in progress 2026-03-15):**
- Architecture direction: configure → execute → observe loop
- MCP tools expanded to 19 (agent/session CRUD, classify, personas, budget, approvals, goals, analytics, hire)
- AgentConfigurator concept (generate CLAUDE.md + hooks.json per persona)
- HookReceiver endpoints (Claude Code hooks POST events back)

**Infrastructure:**
- GitHub Actions CI (test + clippy + build)
- GitHub Release workflow (tag → binaries)
- E2E smoke test script

### Version History

- **v0.1.0**: Agent CRUD, process spawn, sessions, streaming, embedded UI
- **v0.2.0**: MCP server rewrite with rmcp, bug fixes
- **v0.4.0**: 6-middleware chain, skill system, memory, git worktree isolation, sub-agent parallelism, 118 tests
- **v0.5.0**: Cron scheduler, usage analytics, loop detection, quality/exit gates, 150 tests
- **Epic 1**: Org structure, persona catalog, governance layer, 4 new crates, 5 new frontend pages
- **Wave 3**: Sidebar cleanup, governance wiring, session output, page verification
- **Wave 4**: MCP expansion (21 tools), AgentConfigurator, HookReceiver, middleware simplification
- **AR-1 + E3**: Unit of Work refactor, Hexagonal Backends (ProcessBackend trait, BackendRegistry, ClaudeBackend), 295 tests

---

## Roadmap (9 Epics → v1.0.0)

Full details: `../docs/product/EPIC_INDEX.md`

| Epic | What | Status |
|------|------|--------|
| E1 | Persona Workforce Catalog | **Baseline implemented** |
| E2 | Dev Methodology Engine | Planning |
| E3 | Hexagonal Backend Architecture | **Foundation implemented** |
| E4 | Org Structure & Governance | Planning |
| E5 | Multi-Backend Execution (Hermes/OpenClaw) | Not started |
| E6 | Knowledge Base | Not started |
| E7 | Multi-Platform Messaging (16+) | Not started |
| E8 | Native Desktop Client | Not started |
| E9 | Production Hardening → v1.0.0 | Not started |

---

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Rust + Svelte 5 single binary | Performance, no runtime deps, unique in space | Pre-project |
| SQLite WAL mode | Single-file, concurrent reads, no server | Pre-project |
| Unify 8 repos into AgentForge | Each repo best-in-class at one piece; together = complete platform | 2026-03-10 |
| Middleware chain pattern | Borrowed from DeerFlow (8 real middlewares, 1,089 LOC) | 2026-03-02 |
| Git worktree isolation | Industry standard for multi-Claude-Code | 2026-03-02 |
| Use rmcp for MCP | Official Rust SDK, `#[tool]` macro | 2026-03-02 |

---

## Session Protocol

### Before Starting Work
1. Read this file
2. Check `../docs/sprints/SPRINT_PLAN.md` for current sprint
3. Pick a task from the active sprint

### During Work
- One session = one focused deliverable
- Commit early, commit often

### When Done
1. Commit all changes
2. Update this file only if priorities changed

---

## File Map

```
agentforge-hq/                   <-- This directory (was forge-project)
  crates/                         <-- 12 workspace crates + forge-mcp stubs
    forge-core/                   ForgeEvent (43 variants), EventBus (fan-out), errors, IDs
    forge-agent/                  Agent model, 10 presets, validation
    forge-db/                     SQLite WAL, 13 migrations, 17 repos, BatchWriter, UnitOfWork
    forge-process/                Claude CLI spawn, stream-json, ConcurrentRunner, LoopDetector
    forge-safety/                 CircuitBreaker, RateLimiter, CostTracker
    forge-api/                    Axum HTTP + WebSocket + middleware + embedded frontend
    forge-app/                    Binary entry point, wiring, shutdown
    forge-git/                    Git worktree create/remove/list
    forge-org/                    Company, Department, OrgPosition, org chart
    forge-persona/                Persona catalog, parser, hire flow
    forge-governance/             Goals, Approvals
    forge-mcp/                    MCP protocol stubs
    forge-mcp-bin/                MCP stdio server (rmcp)
  frontend/                       SvelteKit 5 + TailwindCSS 4
  migrations/                     0001–0014 (org, personas, governance, safety, backends)
  personas/                       112 persona Markdown files (11 divisions, seeded at startup)
  skills/                         10 seed Markdown skill files
  scripts/                        e2e-smoke.sh
  docs/                           Implementation-level docs
  NORTH_STAR.md                   YOU ARE HERE
  CLAUDE.md                       AI agent context
  README.md                       GitHub landing page
```

---

## Documentation

| Scope | Where |
|-------|-------|
| AgentForge strategy, epics, sprints | `../docs/INDEX.md` (workspace level) |
| Implementation details (this app) | `docs/README.md` (this directory) |
| AI agent context | `CLAUDE.md` (this directory) |
