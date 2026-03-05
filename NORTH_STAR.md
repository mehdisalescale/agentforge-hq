# Claude Forge — North Star

> **Read this first in every session.** This is the single source of truth.
> Last updated: 2026-03-05 (v0.5.0 — scheduler, analytics, loop detection, quality gates)

---

## What We're Building

A multi-agent Claude Code orchestrator: Rust/Axum + Svelte 5, single binary.
The only Rust-native tool in the space — everyone else is TypeScript/Electron or Python.

**One-liner**: Spawn Claude Code agents, see their output in a real-time UI, keep them safe. One binary, zero deps.

---

## Current State (Verified 2026-03-05)

v0.5.0 tagged. All sprints + all 4 waves + v0.5.0 features complete.

### Build Status
- `cargo check` — clean, zero warnings, all 9 crates compile
- `cargo test` — 150 tests, all pass
- `cargo clippy` — clean, zero warnings
- Frontend — built and embedded (SvelteKit 5 + adapter-static)

### What Works (verified in code)

**Backend (9 workspace crates):**
- forge-core: ForgeEvent (35 variants), EventBus broadcast, ForgeError, typed IDs (AgentId, SessionId, ScheduleId)
- forge-agent: 10 presets (incl. Coordinator), Agent/NewAgent/UpdateAgent, name validation
- forge-db: SQLite WAL, 5 migrations, BatchWriter (50/2s), 8 repos (Agent, Session, Event, Skill, Memory, Hook, Schedule, Analytics)
- forge-process: Claude CLI spawn with stream-json, ConcurrentRunner, LoopDetector (sliding-window hash, exit gate)
- forge-safety: CircuitBreaker (3-state FSM), RateLimiter (token bucket), CostTracker (budget warn/limit)
- forge-api: Full HTTP API + WebSocket, CORS, TraceLayer, rust-embed SPA, 8-middleware chain (RateLimit, CircuitBreaker, CostCheck, SkillInjection, Persist, Spawn, ExitGate, QualityGate)
- forge-app: Binary wiring, graceful shutdown, env config, skill loading, cron scheduler background task
- forge-git: Worktree create/remove/list for multi-agent isolation (7 tests)
- forge-mcp-bin: MCP stdio server (rmcp, 10 tools)

**Frontend (12 pages, all $state runes):**
- Dashboard — agent selector, prompt, WebSocket streaming, markdown rendering, sub-agent progress panel
- Agents — full CRUD, 10 presets, domain badges (code/quality/ops/orchestration)
- Sessions — two-pane layout, worktree badges, merge/cleanup buttons, export (JSON/Markdown/HTML), cost display
- Memory — full CRUD, search, confidence bars, category badges
- Hooks — full CRUD, event type select, timing badges, enable/disable toggle
- Skills — tag pills, category filter, expandable content, usage count
- Workflows — visual placeholder diagram, card layout
- Settings — config dashboard, health endpoint, env var table
- **Schedules** — cron CRUD, preset dropdown, enable/disable toggle, run count, last/next run
- **Analytics** — summary cards, CSS bar chart (daily costs), agent breakdown table, P90, projected monthly

**Infrastructure:**
- GitHub Actions CI (test + clippy + build)
- GitHub Release workflow (tag → binaries)
- E2E smoke test script
- Configurable: FORGE_HOST, FORGE_PORT, FORGE_DB_PATH, FORGE_CORS_ORIGIN, FORGE_CLI_COMMAND, FORGE_RATE_LIMIT_*, FORGE_BUDGET_*

### What's New in v0.5.0

- **Cron scheduler**: ScheduleRepo CRUD (468 LOC, 10 tests), background tick loop (60s interval), CancellationToken shutdown, frontend page
- **Usage analytics**: AnalyticsRepo (297 LOC, 7 tests), daily costs, agent breakdown, session stats, P90, projected monthly, frontend dashboard
- **Loop detection**: LoopDetector (201 LOC, 10 tests), sliding-window hash dedup, exit gate config, completion pattern matching
- **Quality/exit gates**: MiddlewareError variants for exit gate + quality gate, 3 quality gate middleware tests
- **Session HTML export**: dark-themed HTML report, `/api/v1/sessions/:id/export?format=html`
- **9 new events**: ScheduleCreated/Triggered/Deleted, ExitGateTriggered, QualityCheckStarted/Passed/Failed, Error

### What's Next — v0.6.0

See `docs/V050_SPRINT_PLAN.md` for remaining planned features. Key items not yet implemented:

| Feature | Priority |
|---------|----------|
| Best-of-N selection (quality multiplier) | HIGH |
| Context pruner + memory compaction | HIGH |
| Sequential + Fanout pipeline engine | HIGH |
| Swim-lane observability dashboard | HIGH |
| Three-type memory + auto-activating skills | MEDIUM |
| OpenAPI auto-docs (utoipa + Scalar UI) | MEDIUM |
| Predictive usage budgeting (P90 forecast) | MEDIUM |

Research: `docs/RESEARCH_FINDINGS_2026_03_05.md` (67 repos analyzed)

### Remaining Gaps

| Gap | Severity | When |
|-----|----------|------|
| Authentication (none anywhere) | Medium | Post-v1.0 |
| E2E automated test suite | Low | v0.5.0 |

---

## Sprint Plan

Single source of truth: `MASTER_TASK_LIST.md`. Agent task cards: `docs/agents/HANDOFF_SPRINT_2_3.md`.

### Sprint 1 → v0.2.0 — MCP + Ship — DONE

- [x] F1-F3: Bug fixes, M1-M5: MCP rewrite, D1-D2: Docs

### Sprints 2-3 → v0.4.0 — Parallel Wave Execution — DONE

13 agents across 4 waves, all complete:

```
Wave 1 (5 parallel) ✓ forge-git, middleware, skills, memory, hooks
Wave 2 (1 sequential) ✓ integration wiring
Wave 3 (3 parallel) ✓ middleware extraction, memory logic, sub-agent runner
Wave 4 (4 parallel) ✓ frontend: memory/hook UI, sub-agent dashboard, polish
```

~1 day parallel execution. See `MASTER_TASK_LIST.md` for details.

---

## What's Cut

These are NOT in scope until users demand them:

| Feature | Why Cut |
|---------|---------|
| WASM plugin runtime | MCP servers are the extension mechanism |
| Multi-LLM routing | Forge is Claude-first; no current user demand |
| Consensus protocols | Agents are independent; don't need agreement |
| RL/learning layer | No usage data to train on yet |
| 305-feature roadmap | Focus on ~20 features that matter |
| Notification system (20 features) | Webhooks (1 feature) covers 90% |
| Plugin marketplace | Need users first |
| ~~Cron scheduler~~ | ~~Manual for now~~ → **Implemented in v0.5.0** |
| Dev environment (code viewer, terminal) | Post-1.0 if ever |

---

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Rust + Svelte 5 single binary | Performance, no runtime deps, unique in space | Pre-project |
| SQLite WAL mode | Single-file, concurrent reads, no server | Pre-project |
| Ship existing prototype (don't rewrite) | 3K lines of working code > 44K lines of planning | 2026-02-26 |
| 4 sprints (not 7 phases) | Ship in weeks, not months | 2026-03-02 |
| Use rmcp for MCP | Official Rust SDK, `#[tool]` macro, maintained by protocol team | 2026-03-02 |
| Middleware chain pattern | Borrowed from DeerFlow (verified: 8 real middlewares, 1,089 LOC) | 2026-03-02 |
| Markdown-based skills | Borrowed from DeerFlow (verified: 15 real SKILL.md files, 208-line loader) | 2026-03-02 |
| Git worktree isolation | Industry standard for multi-Claude-Code (official support Feb 2026) | 2026-03-02 |
| Sub-agent parallelism | Borrowed from DeerFlow (verified: 414-line executor with real threadpool) | 2026-03-02 |
| Cut WASM plugins | No competitor uses them; MCP is the extension mechanism | 2026-02-26 |
| Freeze 00-08 planning dirs | Reference only; stop updating | 2026-02-26 |

---

## Session Protocol

### Before Starting Work
1. Read this file
2. Check `docs/SESSION_LOG.md` for recent sessions
3. Pick a task from the current sprint

### During Work
- One session = one focused deliverable
- Commit early, commit often
- Don't update frozen planning docs — write code

### When Done
1. Commit all changes
2. Log what was done in `docs/SESSION_LOG.md`
3. Update this file only if priorities changed

---

## File Map

```
forge-project/                    <-- Everything lives here
  crates/                         <-- 9 workspace crates
    forge-core/                   ForgeEvent (35 variants), EventBus, errors, IDs
    forge-agent/                  Agent model, 10 presets, validation
    forge-db/                     SQLite WAL, 5 migrations, 8 repos, BatchWriter
    forge-process/                Claude CLI spawn, stream-json, ConcurrentRunner, LoopDetector
    forge-safety/                 CircuitBreaker, RateLimiter, CostTracker
    forge-api/                    Axum HTTP + WebSocket + middleware + embedded frontend
    forge-app/                    Binary entry point, wiring, shutdown
    forge-git/                    Git worktree create/remove/list
    forge-mcp-bin/                MCP stdio server (rmcp)
  frontend/                       SvelteKit 5 + TailwindCSS 4
  migrations/                     0001_init, 0002_add_cost, 0003_add_memory, 0004_add_hooks, 0005_scheduler_analytics
  skills/                         10 seed Markdown skill files
  scripts/                        e2e-smoke.sh
  .github/workflows/              ci.yml, release.yml
  docs/                           Current docs (audit, borrowed ideas, tasks)
    FORGE_AUDIT_2026_03_02.md     Full audit report
    BORROWED_IDEAS.md             DeerFlow + Claude-Flow + industry analysis
    agents/                       Agent task records, wave coordination
  archive/                        Frozen reference + old planning docs
  NORTH_STAR.md                   YOU ARE HERE
  MASTER_TASK_LIST.md             Sprint tasks
  README.md                       GitHub landing page
```

---

## Reference Material

### In This Repo

| Resource | Location | Notes |
|----------|----------|-------|
| Full audit report | `docs/FORGE_AUDIT_2026_03_02.md` | Per-crate grades, gap analysis, proposal |
| Borrowed ideas | `docs/BORROWED_IDEAS.md` | DeerFlow, Claude-Flow, reference repos, industry research |

### Companion Repo: claude-parent

> **GitHub:** [mbaneshi/claude-parent](https://github.com/mbaneshi/claude-parent)
> **Local:** `/Users/bm/claude-parent/` (forge-project was split out of this repo on 2026-03-02)

| Resource | Location | Notes |
|----------|----------|-------|
| 61 reference repos | `refrence-repo/` (local) or [online](https://github.com/mbaneshi/claude-parent) | Top 4: hooks-observability, Workflow, ralph, infrastructure-showcase |
| Reference map | `reference-map/` | 13-category taxonomy of all 61 repos |
| Capability map | `docs/CLAUDE_CODE_CAPABILITY_MAP_AND_FORGE_ROADMAP.md` | Claude Code capabilities mapped to Forge features |
| AI doc landscape | `docs/ai-documentation-landscape.md` | AGENTS.md standard, industry state (2026) |

### External (Local)

| Resource | Location | Notes |
|----------|----------|-------|
| DeerFlow (verified) | `/Users/bm/cod/trend/26-feb/deer-flow` | ~10K real LOC Python, middleware + skills + memory + sub-agents |
| Claude-Flow (mixed) | `/Users/bm/cod/trend/26-feb/claude-flow` | ~60% real, study swarm coordinator + ADR pattern |
