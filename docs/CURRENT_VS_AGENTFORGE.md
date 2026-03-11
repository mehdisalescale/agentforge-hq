# Claude Forge (Current) vs AgentForge (Planning Doc) — Gap Analysis

> Comparing the shipped codebase (v0.5.0 / v0.6.0 planned) against the
> AgentForge Comprehensive Product & Engineering Planning Document v1.0.0.
>
> Date: 2026-03-11

---

## 1. Identity & Scope

| Dimension | Current (Claude Forge) | AgentForge Planning Doc |
|-----------|----------------------|------------------------|
| **Name** | Claude Forge | AgentForge |
| **Tagline** | Multi-agent Claude Code orchestrator | Enterprise AI Workforce Platform |
| **Shipped version** | v0.5.0 (v0.6.0 planned) | Proposal only — no code yet |
| **Target GA** | v0.6.0 next sprint | v1.0.0 after 20 sprints (~20 weeks) |
| **Stack** | Rust + Axum + SvelteKit 5 + SQLite | Same (extends current stack) |
| **Binary philosophy** | Single binary, zero deps | Same — sidecars optional |
| **LLM support** | Claude CLI only | Claude + Hermes + OpenClaw |

---

## 2. Codebase Comparison

### 2.1 Crates

| Crate | Current | AgentForge adds |
|-------|---------|----------------|
| `forge-app` | Binary entrypoint, DB setup, API server | Cron scheduler, sidecar detection |
| `forge-api` | HTTP + WS, routes, CORS, rust-embed SPA | 16+ new route groups (personas, companies, goals, approvals, backends, KB, messaging, webhooks) |
| `forge-process` | Spawn Claude CLI, stream-json, ConcurrentRunner, LoopDetector | BackendRoute middleware dispatching to 3 adapters |
| `forge-agent` | Agent model, 10 presets, validation | `backend_type`, `company_id`, `department_id`, `reports_to`, `persona_id` fields |
| `forge-db` | SQLite WAL, 5 migrations, 8 repos | 7 new migrations (0009–0015), ~6 new repos |
| `forge-core` | ForgeEvent (35 variants), EventBus, shared types | EventBus expanded to 50+ event variants |
| `forge-safety` | CircuitBreaker, RateLimiter, CostTracker | SecurityScanner (9 OWASP patterns) |
| `forge-git` | Worktree create/remove/list | Unchanged |
| `forge-mcp-bin` | MCP stdio server, 10 tools | 10+ new MCP tools |
| `forge-persona` | **NEW** (scaffolded in current) | Full crate: parser, DB projection, hire flow |
| `forge-org` | — | **NEW**: Company, Department, OrgPosition, budget enforcement |
| `forge-governance` | — | **NEW**: Goal hierarchy, approval gates, cycle detection |
| `forge-knowledge` | — | **NEW**: Document parser, FTS5 indexer, chunk storage |
| `forge-adapter-hermes` | — | **NEW**: Hermes CLI adapter, memory sync |
| `forge-adapter-openclaw` | — | **NEW**: HTTP webhook client + callback handler |
| `forge-messaging` | — | **NEW**: AstrBot bridge, intent router, notification prefs |

### 2.2 Numbers at a Glance

| Metric | Current | AgentForge Target |
|--------|---------|------------------|
| Rust crates | 9 | ~16 |
| DB tables | ~11 | 22+ |
| DB migrations | 5 (up to 0008) | 12 (up to 0015) |
| API routes | ~40 | 80+ |
| Frontend pages | 12 | 20+ |
| Tests | 150 | 400+ |
| Agent presets/personas | 10 | 110+ |
| Skills | 10 | 30+ |
| Middlewares | ~8 | 14 (explicit ordered chain) |
| Event types | 35 | 55+ |
| Execution backends | 1 (Claude) | 3 (Claude, Hermes, OpenClaw) |
| Messaging platforms | 0 | 3 native / 16+ via sidecar |
| LOC (Rust) | ~12.7K | ~25K+ |

---

## 3. Feature Gap — What AgentForge Adds

### Wave 1: Persona Workforce Catalog (Epic-01, v0.7.0)
**Status in current codebase:** `forge-persona` crate scaffolded, `personas` table exists. No parser, no hire flow, no UI.

| Feature | Description | Current state |
|---------|-------------|---------------|
| Persona Parser | Parse 100+ agency-agents `.md` files, content-hash dedup | Not implemented |
| Persona Catalog API | `GET /personas` with division filter, search, pagination | Not implemented |
| Persona DB Projection | `personas` + `persona_divisions` tables, upsert on change | Table exists, no projection logic |
| Hire Flow | `POST /personas/:id/hire` → creates Agent with persona config | Not implemented |
| Persona UI | Division sidebar, card grid, detail modal, hire button | Not implemented |
| Persona Import CLI | `POST /personas/import` bulk re-import | Not implemented |
| Persona Analytics | Most-hired heatmap | Not implemented |

### Wave 2: Developer Methodology Engine (Epic-02, v0.7.0)
**Status in current codebase:** Skills exist in DB. No task type detection, no security scanner, no skill routing.

| Feature | Description | Current state |
|---------|-------------|---------------|
| Skill Import (Superpowers) | 14 methodology skills → YAML format | Not imported |
| Skill Import (Plugins) | 6 plugin skills adapted to Forge schema | Not imported |
| TaskTypeDetector | Classify prompt → new_feature/bug_fix/code_review/refactor/research | Not implemented |
| SkillRouter | task_type → load matching skills into system prompt | Not implemented |
| SecurityScanner | 9 OWASP pattern scanner, post-execution hook | Not implemented |
| Methodology UI | Workflow diagram viewer, task→skill mapping | Not implemented |

### Wave 3: Multi-Company Org Governance (Epic-03, v0.8.0)
**Status in current codebase:** `forge-org` crate exists with basic org schema. No budget enforcement, no goals, no approvals.

| Feature | Description | Current state |
|---------|-------------|---------------|
| Company CRUD | Create/read/update/delete with budget limits | Schema exists, basic CRUD |
| Department Management | Auto-create from persona divisions, manual CRUD | Not implemented |
| Org Chart | Reporting chain tree: agent → reports_to → department → company | Not implemented |
| BudgetEnforcer Middleware | Per-company budget check before spawn, auto-pause | Not implemented |
| Goal Hierarchy | Goals with parent chains, cycle detection, mission linkage | Not implemented |
| Approval Gates | Hire/strategy/budget approval flows | Not implemented |
| Org Chart UI | Tree visualization, department grouped cards | Not implemented |

### Wave 4–5: Multi-Backend Execution (Epic-04/05, v0.9.0)
**Status in current codebase:** Only Claude CLI backend. No adapter pattern for multiple backends.

| Feature | Description | Current state |
|---------|-------------|---------------|
| BackendRoute Middleware | Route to claude/hermes/openclaw by `agent.backend_type` | Not implemented |
| HermesAdapter | Spawn hermes CLI, relay tools, bidirectional memory sync | Not implemented |
| OpenClawAdapter | HTTP webhook client + callback handler | Not implemented |
| Backend Health Check | Detect available backends at startup, cache status | Not implemented |
| MemorySync | Hermes MEMORY.md ↔ Forge memory table | Not implemented |
| Backend UI | Backend selector, health indicators | Not implemented |

### Wave 6: Knowledge Base (Epic-06, v0.10.0)
**Status in current codebase:** Not started. Schema documented in `FORGE_KNOWLEDGE_MESSAGING_SCHEMA.md`.

| Feature | Description | Current state |
|---------|-------------|---------------|
| Document Parser | Text/Markdown → chunk splitting, token counting | Not implemented |
| FTS5 Indexer | SQLite FTS5 virtual table, BM25 ranking | Not implemented |
| Knowledge API | Upload, search, list, delete; per-company isolation | Not implemented |
| KnowledgeInjection Middleware | Top-K chunks → prepend to system prompt | Not implemented |
| KB UI | Drag-drop upload, search, chunk preview | Not implemented |

### Wave 7: Messaging (Epic-07, v0.10.0)
**Status in current codebase:** Not started. Schema documented.

| Feature | Description | Current state |
|---------|-------------|---------------|
| AstrBot Bridge | `forge-messaging` HTTP bridge to AstrBot REST API | Not implemented |
| Intent Router | Parse "@agent do X" / "status" / "search KB: X" | Not implemented |
| Notification Prefs | Per-user event filters, platform preferences | Not implemented |
| Webhook Endpoints | `POST /webhooks/:platform` | Not implemented |

### Wave 8: Desktop Application (Epic-08, v1.0.0)
**Status in current codebase:** Not started.

| Feature | Description | Current state |
|---------|-------------|---------------|
| Desktop Fork | Fork Open-Claude-Cowork, rewire to Forge API | Not started |
| Desktop Views | OrgChart, TaskBoard, CostDashboard, PersonaCatalog, KBSearch | Not started |
| OS Notifications | Native notifications for approvals, budget, security | Not started |
| Build Pipeline | Electron builds for mac/win/linux | Not started |

---

## 4. Architecture Differences

### 4.1 Middleware Chain

| # | Current | AgentForge |
|---|---------|-----------|
| 1 | RateLimit | RateLimit |
| 2 | CircuitBreaker | CircuitBreaker |
| 3 | — | **CompanyBudgetCheck** (new) |
| 4 | CostCheck | CostCheck |
| 5 | SkillInjection | SkillInjection |
| 6 | — | **TaskTypeDetection** (new) |
| 7 | — | **KnowledgeInjection** (new) |
| 8 | — | **PersonaInjection** (new) |
| 9 | Persist | Persist |
| 10 | — | **BackendRoute** (new) |
| 11 | Spawn | Spawn |
| 12 | — | **ExitGate** (new) |
| 13 | — | **QualityGate** (new) |
| 14 | — | **SecurityScan** (new) |

### 4.2 Event Types

Current: 35 event variants covering agent lifecycle, sessions, skills, memory, hooks, schedules, analytics.

AgentForge adds 15+ new variants:
- `PersonaHired`, `CompanyBudgetExceeded`, `ApprovalRequired`, `ApprovalDecided`
- `GoalStatusChanged`, `KnowledgeInjected`, `DocumentIndexed`
- `BackendSelected`, `SecurityScanFailed`
- `MessageReceived`, `NotificationSent`, `TaskTypeResolved`

### 4.3 DB Migrations

Current: 5 migrations (up to ~0008).

AgentForge adds 7 migrations:
| Migration | Tables |
|-----------|--------|
| `0009_personas.sql` | `personas`, `persona_divisions` |
| `0010_skill_task_routing.sql` | `skill_task_routes` |
| `0011_org_charts.sql` | `companies`, `departments`, `org_positions`, `goals`, `approvals` |
| `0012_agent_backends.sql` | `agents.backend_type` column |
| `0013_webhook_callbacks.sql` | `webhook_callbacks` |
| `0014_knowledge_base.sql` | `kb_documents`, `kb_chunks`, `kb_chunks_fts` (FTS5) |
| `0015_messaging.sql` | `messaging_configs`, `notification_prefs` |

### 4.4 ADRs (Architecture Decision Records)

| ADR | Decision | Impact |
|-----|----------|--------|
| ADR-001 | Persona: Markdown files + DB projection (dual storage) | New crate pattern |
| ADR-002 | KB: SQLite FTS5, not vector DB | Zero-dep search |
| ADR-003 | Messaging: AstrBot sidecar first, native later | Docker Compose for messaging |
| ADR-004 | Middleware chain: explicit ordered `Vec<Box<dyn Middleware>>` | 8 → 14 middlewares |
| ADR-005 | Desktop: fork Open-Claude-Cowork, rewire to Forge API | Two UI codebases (Svelte + React) |

---

## 5. What's the Same (Shared Foundation)

These elements are unchanged between current and AgentForge:

- **Core tech stack**: Rust/Axum + SvelteKit 5 + SQLite WAL + rust-embed
- **Single binary philosophy**: all core in one binary, sidecars optional
- **EventBus architecture**: broadcast channel, all state changes emit events
- **BatchWriter**: 50-event or 2s flush batching
- **Safety primitives**: CircuitBreaker (3-state FSM), RateLimiter (token bucket), CostTracker
- **Zero warnings policy**: `cargo check` clean, `cargo clippy -- -D warnings`
- **ID newtypes**: `AgentId`, `SessionId`, etc. wrapping `uuid::Uuid`
- **Error hierarchy**: `ForgeError` / `ForgeResult<T>` in forge-core
- **MCP server mode**: rmcp v0.17, stdio transport
- **Git worktree isolation**: multi-agent file isolation
- **Frontend runes**: Svelte 5 `$state`, `$derived` across all pages

---

## 6. Existing Related Docs

The AgentForge expansion is already partially documented across these files:

| File | What it covers |
|------|---------------|
| `docs/EXPANSION_PLAN.md` | How 8 external repos map to Forge gaps |
| `docs/AGENTFORGE-BUILD-PLAN.md` | Alternative path using Paperclip (Node.js) as foundation |
| `docs/AGENTFORGE-EXECUTIVE-SUMMARY.md` | Business case and opportunity |
| `docs/AGENTFORGE_AGENT_ROLES.md` | Agent role definitions |
| `docs/product/` | Full structured breakdown: 9 epics, 77 stories, 300 points, ADRs, test strategy |
| `docs/EXTERNAL_REPOS/` | Analysis of Hermes Agent and OpenClaw repos |
| `docs/FORGE_KNOWLEDGE_MESSAGING_SCHEMA.md` | KB + messaging DB schema design |

**Note:** `docs/AGENTFORGE-BUILD-PLAN.md` proposes a **different foundation** (Paperclip/Node.js/PostgreSQL) than what the planning doc and current codebase use (Rust/SQLite). The planning doc follows the Rust expansion path.

---

## 7. Summary

The AgentForge planning document is a **20-sprint expansion roadmap** that builds on top of the current Claude Forge codebase. It does not replace anything already built — it extends it with:

- **6 new crates** (persona, org, governance, knowledge, 2 adapters, messaging)
- **7 new DB migrations** (11 → 22+ tables)
- **6 new middlewares** (8 → 14 in the pipeline)
- **15+ new event types** (35 → 50+)
- **16+ new API endpoint groups**
- **3 execution backends** (Claude-only → Claude + Hermes + OpenClaw)
- **Knowledge base** with FTS5 search and automatic context injection
- **Multi-company governance** with budgets, goals, and approval gates
- **Messaging** via AstrBot sidecar (16+ platforms)
- **Desktop app** via Electron fork

The current codebase is the working foundation. The planning doc is the blueprint for what comes next.
