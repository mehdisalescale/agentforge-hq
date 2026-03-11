# State of the Project — 2026-03-11

> Comprehensive distillation of all documentation, codebase state, and gaps.
> Generated from full audit of 40+ docs and 14 crates.

---

## 1. Two Competing Visions

This project has evolved through two overlapping identities:

### Vision A — "Claude Forge" (original, v0.1–v0.6)

Local multi-agent Claude Code orchestrator. Single Rust binary with embedded Svelte 5 frontend. Spawn agents, stream output, keep them safe. Ship small, ship often.

- Born 2026-02-26, split from `claude-parent` on 2026-03-02
- 5 releases (v0.1→v0.5) shipped in ~1 week via wave-based parallel agent execution
- v0.6.0 features committed (best-of-N, pipelines, OpenAPI, typed memory, kanban)
- Scope: ~20 focused features, single execution backend (Claude CLI)

### Vision B — "AgentForge" (new, docs/plan0 + docs/product)

AI workforce management platform absorbing 8 open-source repos into a unified system:

- 100+ specialized agent personas organized in org charts with budgets
- 3 execution backends (Claude, Hermes, OpenClaw)
- 16+ messaging platforms via AstrBot sidecar
- Knowledge base with FTS5 search
- Formal product management: epics, stories, ADRs, definition of done
- Target: v1.0.0 with 16 crates, 22+ tables, 80+ routes, 400+ tests

**The transition is underway.** Three new crates (forge-org, forge-governance, forge-persona) represent the first code for Vision B. Docs have jumped well ahead of code.

---

## 2. Documentation Map

### Tier 1 — Source of Truth

| File | Role | Current? |
|------|------|----------|
| `NORTH_STAR.md` | Vision, verified state, sprint plan, decisions | **Stale** — says 9 crates, actual is 12 |
| `MASTER_TASK_LIST.md` | Sprint tasks (What/Where/How/Verify) | **Stale** — stops at v0.6.0 planned |
| `CLAUDE.md` | AI session context | **Stale** — says 9 crates |
| `README.md` | GitHub landing page | **Stale** — says 9 crates |
| `docs/DOC_INDEX.md` | What to read vs archived | **Stale** — doesn't list new docs |

### Tier 2 — Sprint Plans (historical, accurate for their era)

| File | Covers | Status |
|------|--------|--------|
| `docs/V050_SPRINT_PLAN.md` | v0.5.0 (scheduler, analytics, loop detection) | Done |
| `docs/V060_SPRINT_PLAN.md` | v0.6.0 (best-of-N, pipelines, swim lanes) | Code committed |
| `docs/agents/V060_WAVE_PROMPTS.md` | Copy-paste prompts for v0.6.0 agents | Used |

### Tier 3 — AgentForge Expansion (new, partially implemented)

| File | Says | Code exists? |
|------|------|-------------|
| `docs/EXPANSION_PLAN.md` | 8 waves absorbing 8 repos → v1.0.0 | Waves 1-3 started |
| `docs/AGENTFORGE-BUILD-PLAN.md` | Alternative 4-repo integration plan | Planning only |
| `docs/AGENTFORGE-EXECUTIVE-SUMMARY.md` | Product pitch | Planning only |
| `docs/AGENTFORGE_AGENT_ROLES.md` | 4 parallel dev personas for expansion | Planning only |
| `docs/EPIC1_FOUNDATION_TASKS.md` | Story-level breakdown for v0.7.0 | Partial code |
| `docs/FORGE_KNOWLEDGE_MESSAGING_SCHEMA.md` | KB + messaging DB schema design | No code |

### Tier 4 — Product Management (new, formal process)

| File | Purpose |
|------|---------|
| `docs/product/PRODUCT_VISION.md` | Full vision: 6 bounded contexts, 10 ADRs, success metrics |
| `docs/product/DEFINITION_OF_DONE.md` | Quality gates for stories, epics, releases |
| `docs/product/README.md` | Product index with metrics snapshot |
| `docs/product/EPIC_INDEX.md` | 9 epics (E1–E9) |
| `docs/product/epics/E1–E9` | Individual epic specs (persona, methodology, backends, org, KB, messaging, desktop, hardening) |
| `docs/product/adrs/ADR-001, ADR-005` | Hexagonal architecture, company tenancy |
| `docs/product/sprints/SPRINT_PLAN.md` | Sprint planning |
| `docs/product/testing/TEST_STRATEGY.md` | Test strategy |

### Tier 5 — External Integration Specs

| File | Covers |
|------|--------|
| `docs/EXTERNAL_REPOS/ASTRBOT.md` | Messaging sidecar adapter spec |
| `docs/EXTERNAL_REPOS/HERMES_AGENT.md` | Primary execution backend adapter spec |
| `docs/EXTERNAL_REPOS/OPENCLAW.md` | Secondary execution backend adapter spec |

### Tier 6 — AgentForge Proposals (vision docs)

| File | Content |
|------|---------|
| `docs/plan0/PROPOSAL-AgentForge.md` | Full 8-repo proposal (~43K) |
| `docs/plan0/PROPOSAL-AgentForge_1.md` | 4-repo variant (~21K) |
| `docs/plan0/PROPOSAL-AgentForge_2.md` | Duplicate of full proposal |
| `docs/plan0/PROPOSAL-AgentForge_3.md` | Duplicate of full proposal |

### Tier 7 — Research & Reference (stable, read-only)

| File | Value |
|------|-------|
| `docs/RESEARCH_FINDINGS_2026_03_05.md` | 67 repos analyzed, 28 adoptable patterns in 3 tiers |
| `docs/BORROWED_IDEAS.md` | DeerFlow (~10K real LOC) + Claude-Flow (~60% real) analysis |
| `docs/FORGE_AUDIT_2026_03_02.md` | Per-crate grades, gap analysis |
| `docs/HARVESTER_INTEGRATION.md` | Harvester integration assessment (deferred) |
| `docs/MCP_DESIGN.md` | MCP server design notes |
| `docs/E2E_SMOKE_TEST.md` | E2E test documentation |
| `docs/SESSION_LOG.md` | 16 session records |

### Tier 8 — Agent Coordination (historical)

| File | Content |
|------|---------|
| `docs/agents/AGENT_PROTOCOL.md` | Read/write protocol for parallel agents |
| `docs/agents/WAVE*_PROMPTS.md` | Copy-paste prompts for wave execution |
| `docs/agents/WAVE1_STATUS.md` | Wave 1 tracker |
| `docs/agents/HANDOFF_*.md` | Handoff specs between waves |
| `docs/agents/TASK_01–TASK_20.md` | Individual agent task records |
| `docs/agents/README.md` | Agent task system overview |

### Tier 9 — Archive

`archive/` — 35 frozen planning docs (00-vision through 08-reference), 14 planning files, 9 superseded docs. All preserved in git history.

---

## 3. Codebase State

### 3.1 Crate Inventory (12 workspace members, 14,752 lines Rust)

| Crate | LOC | Tests | Status | Purpose |
|-------|-----|-------|--------|---------|
| forge-core | 742 | 6/6 | Stable | ForgeEvent (35 variants), EventBus, ForgeError, typed IDs |
| forge-agent | 407 | 9/9 | Stable | Agent model, 10 presets (incl. Coordinator), validation |
| forge-db | 5,576 | 81/81 | Stable | SQLite WAL, 21 repos, BatchWriter, Migrator, 10 migrations |
| forge-api | 3,874 | 25/25+1ignored | Stable | Axum HTTP/WS, 18 handlers, 8-middleware chain, OpenAPI, rust-embed |
| forge-process | 2,023 | ~10 | Stable | Claude CLI spawn, stream-json, ConcurrentRunner, LoopDetector |
| forge-safety | 364 | ~5 | Stable | CircuitBreaker (3-state FSM), RateLimiter, CostTracker |
| forge-app | 362 | — | Stable | Binary: DB init, migrations, server, graceful shutdown, cron |
| forge-mcp-bin | 342 | ~5 | Stable | MCP stdio server (rmcp v0.17), 10 tools |
| forge-git | 215 | 7/7 | Stable | Worktree create/remove/list |
| **forge-persona** | **558** | **3/3** | **Complete** | Markdown parser, catalog, mapper (persona → agent) |
| **forge-org** | **175** | **0/1 FAIL** | **Broken** | Company, Department, OrgChart — tree build bug |
| **forge-governance** | **65** | **2/2** | **Stub** | Goal + Approval models only, no logic |

### 3.2 Build Status

```
cargo check    → PASS (zero warnings)
cargo test     → 149/150 pass, 1 FAILURE (forge-org::build_org_chart_creates_tree)
cargo clippy   → clean
```

### 3.3 Database (10 migrations)

| Migration | Tables |
|-----------|--------|
| 0001_init | agents, sessions, events, workflows, skills, skills_fts |
| 0002_add_cost | +cost column on sessions |
| 0003_add_memory | memory, memory_types |
| 0004_add_hooks | hooks |
| 0005_scheduler_analytics | schedules, analytics |
| 0006_compaction | compactions |
| 0007_workflow_columns | +columns on workflows |
| 0008_memory_types | memory types, skill_rules |
| 0009_personas | personas |
| 0011_org_governance | companies, departments, org_positions, goals, approvals |

**Note:** Migration 0010 is missing (jumps 0009 → 0011).

### 3.4 Frontend (12 pages, ~630 lines api.ts)

| Page | Status |
|------|--------|
| Dashboard | Done — agent selector, prompt, WebSocket streaming, sub-agent progress |
| Agents | Done — CRUD, 10 presets, domain badges |
| Sessions | Done — two-pane, worktree badges, export (JSON/MD/HTML) |
| Analytics | Done — daily costs, agent breakdown, P90, projected monthly |
| Schedules | Done — cron CRUD, enable/disable, run count |
| Skills | Done — tags, category filter, expandable content |
| Workflows | Done — visual placeholder, card layout |
| Memory | Done — CRUD, search, confidence bars |
| Hooks | Done — CRUD, event type select, enable/disable |
| Settings | Done — config dashboard, health endpoint |
| **Companies** | **New** — list/create companies |
| **Org Chart** | **New** — hierarchical view |

### 3.5 API Routes (18+ handlers)

Core: health, agents CRUD, run (spawn), sessions (list/get/delete/export), WebSocket streaming
Features: skills, workflows CRUD, memory CRUD+search, hooks CRUD, schedules CRUD, analytics
New: companies (list/create), departments (create), org-positions (create), org-chart (get)

### 3.6 Git Status

- **Branch:** main
- **Working tree:** clean
- **Unpushed:** 4 commits (org schema, persona crate, org routes, AstrBot docs)
- **Untracked:** several new docs (AGENTFORGE*, EXPANSION_PLAN, EXTERNAL_REPOS/, plan0/, product/)

---

## 4. Feature Completeness Matrix

### Fully Implemented (in code, tested, working)

- Agent CRUD with 10 presets
- Process spawn with stream-json parsing
- Session management with cost tracking
- WebSocket real-time streaming
- 8-middleware pipeline (rate limit → circuit breaker → cost → skill inject → persist → spawn → exit gate → quality gate)
- Memory extraction, injection, and CRUD
- Hook system (35 event types, pre/post timing)
- Skill system (10 seed files, tag matching, injection)
- Git worktree isolation for multi-agent safety
- Cron scheduler with background tick
- Usage analytics (daily costs, P90, projections)
- Loop detection (sliding-window hash dedup)
- Sub-agent parallel execution (ConcurrentRunner + Coordinator)
- Best-of-N selection mode
- Context pruner + memory compaction
- Pipeline engine + WorkflowRepo CRUD
- OpenAPI auto-docs (utoipa + Scalar UI)
- Typed memory (3 types) + skill auto-activation rules
- MCP stdio server (10 tools, rmcp)
- Embedded SPA (rust-embed)
- Graceful shutdown
- CI/CD (GitHub Actions)

### Partially Implemented (code exists, incomplete)

| Feature | What exists | What's missing |
|---------|------------|----------------|
| Org structure | DB schema, repos, API routes, 2 frontend pages | `build_org_chart` service is broken (test fails) |
| Personas | Parser, catalog, mapper — all working | Not wired to DB (PersonaRepo exists but not connected) |
| Governance | Goal + Approval models with serde | No service logic, no API, no frontend |

### Designed but Not Started

| Feature | Design Doc | Target |
|---------|-----------|--------|
| Hermes execution backend | `EXTERNAL_REPOS/HERMES_AGENT.md` | v0.8.0 |
| OpenClaw execution backend | `EXTERNAL_REPOS/OPENCLAW.md` | v0.8.0 |
| Knowledge base (FTS5) | `FORGE_KNOWLEDGE_MESSAGING_SCHEMA.md`, `E6_KNOWLEDGE_BASE.md` | v0.9.0 |
| Messaging (AstrBot sidecar) | `FORGE_KNOWLEDGE_MESSAGING_SCHEMA.md`, `E7_MESSAGING.md` | v0.9.0 |
| Desktop client | `E8_DESKTOP_CLIENT.md` | v1.0.0 |
| Production hardening | `E9_PROD_HARDENING.md` | v1.0.0 |
| Dev methodology routing | `E2_DEV_METHODOLOGY.md` | v0.7.0 |
| Multi-backend abstraction | `E3_HEXAGONAL_BACKENDS.md`, `E5_MULTI_BACKEND.md` | v0.8.0 |

### Explicitly Cut

| Feature | Reason |
|---------|--------|
| WASM plugin runtime | MCP is the extension mechanism |
| Multi-LLM routing | Claude-first by design |
| Consensus protocols | Agents are independent |
| RL/learning layer | No usage data yet |
| Plugin marketplace | Need users first |
| Authentication | Post-v1.0 (local-only tool) |

---

## 5. Known Issues

| # | Severity | Issue | Location |
|---|----------|-------|----------|
| 1 | **HIGH** | `build_org_chart` test fails — tree parent-child logic broken | `crates/forge-org/src/service.rs` |
| 2 | **MEDIUM** | forge-governance is a stub — models only, no service/API | `crates/forge-governance/` |
| 3 | **LOW** | Migration 0010 missing — sequence jumps 0009→0011 | `migrations/` |
| 4 | **LOW** | forge-persona not wired to DB persistence | `crates/forge-persona/` ↔ `forge-db` |
| 5 | **INFO** | NORTH_STAR, CLAUDE.md, README, DOC_INDEX all say "9 crates" — actual is 12 | Root docs |
| 6 | **INFO** | 4 commits unpushed to remote | `main` branch |
| 7 | **INFO** | Several new doc files untracked by git | `docs/` |
| 8 | **INFO** | `docs/plan0/` has 2 duplicate proposal files (2 & 3 identical to base) | `docs/plan0/` |

---

## 6. Architecture Patterns

These patterns are established and consistent across the codebase:

| Pattern | Implementation |
|---------|---------------|
| **Hexagonal architecture** | ProcessBackend trait for pluggable runtimes |
| **Event sourcing** | All state changes → ForgeEvent (35 variants) → EventBus broadcast |
| **Repository pattern** | All persistence through typed repos, no raw SQL in handlers |
| **Middleware chain** | Composable cross-cutting concerns (8 middlewares) |
| **Newtype IDs** | AgentId, SessionId, ScheduleId etc. wrapping uuid::Uuid |
| **Persona-as-code** | Markdown files parsed into structured agent configs |
| **Wave execution** | Parallel agents per wave, sequential gates between waves |
| **Single binary** | rust-embed for SPA, SQLite bundled, zero runtime deps |
| **Company tenancy** | company_id scoping (designed), backward compat via default |

---

## 7. Key Decisions Log

| Decision | Rationale | Date |
|----------|-----------|------|
| Rust + Svelte 5 single binary | Performance, no deps, unique in space | Pre-project |
| SQLite WAL mode | Single-file, concurrent reads, no server | Pre-project |
| Ship prototype over rewriting | 3K working LOC > 44K planning | 2026-02-26 |
| rmcp for MCP | Official Rust SDK, maintained | 2026-03-02 |
| Middleware chain from DeerFlow | Verified: 8 real middlewares, 1,089 LOC | 2026-03-02 |
| Git worktree isolation | Industry standard, official support Feb 2026 | 2026-03-02 |
| Expand to AgentForge | Absorb 8 repos for full workforce platform | 2026-03-11 |
| Hexagonal backends | ADR-001: pluggable execution via trait | 2026-03-11 |
| Company tenancy | ADR-005: multi-org via company_id scoping | 2026-03-11 |

---

## 8. Metrics Snapshot

| Metric | v0.5.0 (shipped) | Current (code) | v1.0.0 (target) |
|--------|-----------------|----------------|-----------------|
| Rust crates | 9 | 12 | 16 |
| Lines of Rust | ~12K | ~14.7K | ~25K+ |
| DB tables | 11 | 16 | 22+ |
| DB migrations | 5 | 10 | 15+ |
| API routes | 40+ | 50+ | 80+ |
| Frontend pages | 12 | 14 | 20+ |
| Tests | 150 | 150 (1 failing) | 400+ |
| Agent presets | 10 | 10 | 110+ (personas) |
| Skills | 10 | 10 | 30+ |
| Event types | 35 | 35 | 55+ |
| Middlewares | 8 | 8 | 14 |
| Execution backends | 1 | 1 | 3 |
| Messaging platforms | 0 | 0 | 16+ |

---

## 9. What Needs Attention

### Immediate (before any new features)

1. **Fix forge-org test** — tree build logic in `service.rs`
2. **Wire forge-persona to DB** — PersonaRepo exists but isn't connected
3. **Add migration 0010** or renumber 0011
4. **Update NORTH_STAR, CLAUDE.md, README** — reflect 12 crates and AgentForge direction

### Docs Reconciliation

The project has two doc systems that need merging:

- **Original:** NORTH_STAR → MASTER_TASK_LIST → sprint plans (accurate through v0.6.0)
- **New:** product/PRODUCT_VISION → product/epics → product/sprints (covers v0.7.0→v1.0.0)

DOC_INDEX should be updated to reference all new files. Duplicate proposals in `plan0/` should be cleaned up.

### Strategic

The project needs to decide:
- Is NORTH_STAR still the "read this first" doc, or is `product/PRODUCT_VISION.md`?
- Does MASTER_TASK_LIST continue, or do epics/stories in `product/` take over?
- Are the 8-repo and 4-repo proposals both active, or has one been chosen?

---

*Generated 2026-03-11 from full audit of forge-project repository.*
