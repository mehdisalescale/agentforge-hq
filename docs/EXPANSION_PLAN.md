# Claude Forge — Expansion Plan: Absorbing 8 Repos

> **Expanding Claude Forge (v0.6.0) into AgentForge — a full AI workforce platform.**
>
> This plan extends the existing Rust+Svelte single-binary foundation with capabilities
> from 8 external repos, preserving the zero-deps philosophy where possible.
>
> Date: 2026-03-11

---

## Current State Recap

Claude Forge v0.6.0 already has:
- 9 Rust crates (~12.7K LOC), 150+ tests, zero warnings
- 10 agent presets, 10 skills, 35 event types, 8-middleware pipeline
- SQLite WAL, memory system (3 types), hooks, schedules, analytics
- SvelteKit 5 frontend (12 pages), WebSocket streaming
- MCP server mode, git worktree isolation
- Best-of-N, pipelines, loop detection, context pruner, cost tracking

**What it lacks**: multi-platform messaging, 100+ personas, structured dev workflows,
desktop app, knowledge base, multi-LLM support, multi-company isolation.

---

## What Each Repo Brings (Mapped to Forge Gaps)

| Repo | Gap It Fills | Integration Strategy |
|------|-------------|---------------------|
| **Paperclip** | Multi-company, org charts, budgets, governance, approval gates | **DB schema + API routes** — new crates: `forge-org`, `forge-governance` |
| **Agency-Agents** | 100+ personas (vs Forge's 10 presets) | **Persona importer** — markdown → skills DB + new `forge-persona` crate |
| **Hermes-Agent** | 40+ tools, learning loop, 6 terminal backends | **Adapter crate** — `forge-adapter-hermes` spawns Hermes as execution backend |
| **AstrBot** | 16+ messaging platforms, knowledge base, plugins | **Sidecar service** — `forge-messaging` crate + AstrBot as subprocess |
| **Open-Claude-Cowork** | Desktop Electron app | **Companion app** — connects to Forge API (already REST + WebSocket) |
| **OpenClaw** | Webhook-based agent runtime, Docker sandbox | **Adapter crate** — `forge-adapter-openclaw` webhook bridge |
| **Superpowers** | TDD, debugging, brainstorming, planning workflows | **Skill import** — 14 skills → Forge skills directory + task-type routing |
| **Claude-Code Plugins** | Code review, feature dev, security hooks, PR review | **Plugin import** — plugins → skills + hooks + middleware extensions |

---

## New Crates to Add

```
Existing (9):
  forge-core, forge-agent, forge-db, forge-api, forge-process,
  forge-safety, forge-git, forge-mcp-bin, forge-app

New (7):
  forge-org            Org charts, companies, departments, roles, budgets
  forge-governance     Approval gates, board decisions, goal lineage
  forge-persona        100+ persona catalog, division taxonomy, persona → agent mapper
  forge-messaging      Multi-platform messaging bridge (AstrBot protocol)
  forge-knowledge      Knowledge base with vector search (SQLite + FTS5 or embedded vectors)
  forge-adapter-hermes   Hermes runtime bridge (spawn Python process, relay tools)
  forge-adapter-openclaw OpenClaw webhook bridge (HTTP callbacks)

Total: 16 crates
```

---

## Implementation Waves

### Wave 1: Persona Catalog (agency-agents → Forge)

**Goal**: Expand from 10 presets to 100+ rich personas with divisions.

**What changes:**

```
New crate: forge-persona (~400 LOC)
  - PersonaParser: Read agency-agents/*.md → Persona struct
  - PersonaCatalog: In-memory index, search by division/name/tag
  - PersonaMapper: Persona → NewAgent (maps personality, tools, metrics to agent config)

New DB migration: 0009_personas.sql
  - personas (id, name, division, description, personality, deliverables,
              success_metrics, workflow_steps, tags, source_file, timestamps)
  - persona_divisions (id, name, description, agent_count)

New API routes: /api/v1/personas
  - GET /personas          — list all, filter by division
  - GET /personas/:id      — detail with full personality
  - POST /personas/import  — bulk import from directory
  - POST /personas/:id/hire — create agent from persona

New frontend page: Personas
  - Division sidebar (11 categories)
  - Card grid with persona preview (name, emoji, division, description)
  - Detail modal (personality, deliverables, metrics, workflows)
  - "Hire" button → creates agent with persona config injected

Skills directory expansion:
  skills/                    (existing 10)
  personas/                  (new — import from agency-agents)
    engineering/             (16 persona files)
    design/                  (8)
    marketing/               (17)
    paid-media/              (7)
    product/                 (4)
    project-management/      (6)
    testing/                 (8)
    support/                 (6)
    spatial-computing/       (6)
    specialized/             (15)
    game-development/        (5+)
```

**Source mapping:**
```
agency-agents/engineering/frontend-developer.md
  → personas/engineering/frontend-developer.md    (copy)
  → DB: personas table entry                       (parsed)
  → API: GET /personas?division=engineering        (served)
  → UI: Persona catalog card                       (displayed)
  → Action: "Hire" → POST /agents with config_json containing persona
```

**Effort**: M (2-3 sessions). No external deps. Pure Rust parsing + DB + API + UI.

---

### Wave 2: Dev Methodology (Superpowers + Claude-Code Plugins → Forge Skills)

**Goal**: Engineering agents follow structured TDD, debugging, design workflows.

**What changes:**

```
Skills directory expansion:
  skills/
    (existing 10 seed skills)
    superpowers/                     (new — imported from superpowers)
      brainstorming.md
      writing-plans.md
      test-driven-development.md
      systematic-debugging.md
      subagent-driven-development.md
      requesting-code-review.md
      finishing-a-development-branch.md
      using-git-worktrees.md
      dispatching-parallel-agents.md
      executing-plans.md
      verification-before-completion.md
      receiving-code-review.md
      writing-skills.md
      using-superpowers.md
    claude-code-plugins/             (new — imported from claude-code plugins)
      code-review.md                 (confidence-scored parallel review)
      feature-dev.md                 (7-phase workflow)
      pr-review.md                   (6 specialist agents)
      security-guidance.md           (9 OWASP patterns)
      hookify-rules.md               (custom rule engine)
      commit-automation.md           (git workflow)

New DB migration: 0010_skill_task_routing.sql
  - skill_task_routes (id, task_type, skill_ids, priority, enabled)
    task_type: 'new_feature' | 'bug_fix' | 'code_review' | 'refactor' | 'research'

Extend forge-process:
  - TaskTypeDetector: Analyze prompt → determine task type
  - SkillRouter: task_type → load matching skills into system prompt
  - SecurityScanner: Post-execution hook checking for 9 OWASP patterns

Extend forge-api middleware chain (8 → 10):
  - Add TaskTypeDetection middleware (after SkillInjection)
  - Add SecurityScan middleware (after QualityGate)

New frontend: Skills page enhancement
  - "Methodology" tab showing workflow diagrams
  - Task type → skill mapping configuration
  - Security scan results in session detail

Extend forge-safety:
  - SecurityPatterns: 9 regex patterns for command injection, XSS, eval, etc.
  - SecurityScanResult: Pass/Fail with flagged lines
  - Wire into QualityGate middleware
```

**Source mapping:**
```
superpowers/skills/test-driven-development/SKILL.md
  → skills/superpowers/test-driven-development.md  (adapted to Forge YAML frontmatter)
  → DB: skills table + skill_rules (trigger: task_type=bug_fix)
  → Middleware: TaskTypeDetector → SkillRouter → inject into system prompt

claude-code/plugins/security-guidance/hooks/pretooluse.py
  → forge-safety: SecurityPatterns (Rust regex, not Python)
  → Middleware: SecurityScan runs after every agent execution
```

**Effort**: M-L (3-4 sessions). Mostly markdown import + middleware extension.

---

### Wave 3: Org Charts & Governance (Paperclip → Forge)

**Goal**: Multi-company, departments, hierarchies, budgets, approval gates.

**What changes:**

```
New crate: forge-org (~800 LOC)
  - Company: id, name, mission, budget_limit, budget_used
  - Department: id, company_id, name (maps to persona divisions)
  - OrgPosition: id, agent_id, department_id, reports_to (self-ref), role (ceo/manager/ic)
  - BudgetEnforcer: Per-company cost tracking, auto-pause at limit

New crate: forge-governance (~500 LOC)
  - Approval: id, company_id, type (hire/strategy/budget), status, requester, approver
  - Goal: id, company_id, parent_goal_id, title, description, status
  - GoalLineage: Every task traces back to company mission via parent chain

New DB migration: 0011_org_charts.sql
  - companies (id, name, mission, budget_limit, budget_used, timestamps)
  - departments (id, company_id, name, description)
  - org_positions (id, agent_id, company_id, department_id, reports_to, role)
  - goals (id, company_id, parent_id, title, description, status)
  - approvals (id, company_id, type, status, data_json, timestamps)

Extend forge-agent:
  - Agent gets company_id (nullable for backward compat)
  - Agent gets department_id, reports_to

Extend forge-api:
  - /api/v1/companies      — CRUD + budget dashboard
  - /api/v1/departments    — CRUD, auto-create from persona divisions
  - /api/v1/org-chart      — tree view of agents by reporting chain
  - /api/v1/goals          — CRUD with parent-child hierarchy
  - /api/v1/approvals      — create, approve, reject

Extend forge-safety:
  - BudgetEnforcer: Check company budget before agent spawn
  - Wire into CostCheck middleware (per-company, not just global)

New frontend pages:
  - Companies: list, create, budget overview
  - Org Chart: tree visualization (CSS/SVG, no external lib)
  - Goals: hierarchical view with status badges
  - Approvals: pending list with approve/reject buttons
  - Department view: agents grouped by department

Data flow:
  - Create company → set mission + budget
  - Hire agents from persona catalog → assign to department + reporting chain
  - Create goal (linked to mission) → break into tasks → assign to agents
  - Agent spawn checks company budget → approve/reject
  - Task completion updates goal progress
```

**Source mapping:**
```
Paperclip DB: companies, agents, issues, cost_events, approvals
  → forge-org: Company, Department, OrgPosition
  → forge-governance: Goal, Approval
  → forge-db: New repos (CompanyRepo, DepartmentRepo, GoalRepo, ApprovalRepo)
  → forge-safety: BudgetEnforcer (per-company)

Paperclip: heartbeat protocol
  → forge-process: Already has ConcurrentRunner + schedules
  → Map: Paperclip heartbeat ≈ Forge schedule + ConcurrentRunner trigger
```

**Effort**: L (4-5 sessions). New crates, DB schema, API, UI.

---

### Wave 4: Hermes Runtime Adapter (Hermes-Agent → Forge)

**Goal**: Agents can execute via Hermes (40+ tools, learning loop, 6 backends).

**What changes:**

```
New crate: forge-adapter-hermes (~600 LOC)
  - HermesAdapter: Spawn `hermes` Python process with structured I/O
  - HermesConfig: model, enabled_toolsets, terminal_backend, memory_path
  - HermesResult: Parse Hermes output → ForgeEvent stream
  - MemorySync: Hermes MEMORY.md ↔ Forge memory table bidirectional sync
  - ToolFilter: Filter Hermes toolsets by agent persona/role

Extend forge-process:
  - ProcessBackend enum: Claude | Hermes | OpenClaw
  - ProcessSpawner: Route to correct backend based on agent config
  - Hermes backend: spawn `hermes chat --session-id <forge-session> --model <model>`

Extend forge-agent:
  - Agent.config_json gains: backend ("claude"|"hermes"|"openclaw"), hermes_config

Extend forge-db:
  - New migration 0012_agent_backends.sql
  - agents table: add backend_type column (default "claude")

New API:
  - PUT /api/v1/agents/:id/backend — switch agent backend
  - GET /api/v1/backends            — list available backends + health

Extend frontend:
  - Agent create/edit: Backend selector (Claude / Hermes / OpenClaw)
  - Hermes config panel: terminal backend, toolsets, model selection
  - Session view: Show which backend executed the run

Prerequisites:
  - Hermes must be installed: `pip install hermes-agent` or available in PATH
  - Forge detects available backends on startup via health check
```

**Source mapping:**
```
Hermes run_agent.py AIAgent.run()
  → forge-adapter-hermes: HermesAdapter.spawn()
  → Forge middleware chain wraps Hermes execution
  → Events flow: Hermes stdout → JSON parse → ForgeEvent → EventBus → WebSocket → UI

Hermes tools/registry.py (40+ tools)
  → Exposed via Hermes process (Forge doesn't re-implement)
  → Tool filter config on agent level

Hermes MEMORY.md + USER.md
  → forge-adapter-hermes: MemorySync
  → On session start: Export Forge memories → temp MEMORY.md for Hermes
  → On session end: Parse Hermes MEMORY.md → import new entries to Forge memory table
```

**Effort**: L (3-4 sessions). Python process management, bidirectional sync.

---

### Wave 5: OpenClaw Adapter (OpenClaw → Forge)

**Goal**: Lightweight webhook-based execution for simpler workloads.

**What changes:**

```
New crate: forge-adapter-openclaw (~350 LOC)
  - OpenClawAdapter: HTTP webhook client to OpenClaw gateway
  - OpenClawConfig: gateway_url, auth_token, model, workspace_path
  - WebhookHandler: Receive results via callback URL
  - ResultParser: OpenClaw response → ForgeEvent

Extend forge-api:
  - POST /api/v1/webhooks/openclaw — callback endpoint for OpenClaw results
  - Auto-register callback URL with OpenClaw on agent spawn

Extend forge-process:
  - ProcessBackend::OpenClaw variant
  - OpenClaw backend: POST wakeup payload → wait for webhook callback

New DB migration: 0013_webhook_callbacks.sql
  - webhook_callbacks (id, session_id, source, status, payload, timestamps)

Prerequisites:
  - OpenClaw gateway running (local or Docker)
  - FORGE_OPENCLAW_URL and FORGE_OPENCLAW_TOKEN env vars
```

**Effort**: M (2 sessions). HTTP client + webhook handler.

---

### Wave 6: Knowledge Base (AstrBot KB concepts → Forge)

**Goal**: Shared organizational knowledge accessible to all agents.

**What changes:**

```
New crate: forge-knowledge (~700 LOC)
  - Document: id, title, content, chunks[], source_type (file/url/text)
  - DocumentParser: PDF, markdown, text → chunk splitting
  - ChunkIndex: SQLite FTS5 for text search (leverage existing rusqlite)
  - EmbeddingStore: Optional — store embeddings for semantic search
    (Start with FTS5 keyword search, add embeddings later via external API)
  - KnowledgeQuery: Search across all documents, return ranked chunks
  - ContextInjection: Top-K relevant chunks → prepend to agent system prompt

New DB migration: 0014_knowledge_base.sql
  - kb_documents (id, company_id, title, source_type, source_path, timestamps)
  - kb_chunks (id, document_id, content, chunk_index, tokens)
  - kb_chunks_fts (FTS5 virtual table on kb_chunks.content)

Extend forge-api:
  - /api/v1/knowledge                — list documents
  - POST /api/v1/knowledge/upload    — upload file → parse → chunk → index
  - POST /api/v1/knowledge/search    — FTS5 search, return ranked chunks
  - DELETE /api/v1/knowledge/:id     — remove document + chunks

Extend forge-api middleware:
  - KnowledgeInjection middleware: Before agent spawn, search KB for relevant context
  - Add after SkillInjection in chain (8 → 11 middlewares total now)

New frontend page: Knowledge Base
  - Document list with source type badges
  - Upload form (drag & drop)
  - Search bar with results preview
  - Per-company knowledge isolation
```

**Source mapping:**
```
AstrBot: FAISS vector DB + hybrid retrieval
  → forge-knowledge: SQLite FTS5 (no external deps, fits single-binary philosophy)
  → Future: Add optional embedding endpoint (call external API like OpenAI embeddings)
  → FTS5 is sufficient for most use cases, keeps zero-deps promise

AstrBot: Document parsing (pypdf, markitdown)
  → forge-knowledge: DocumentParser
  → Start with plain text + markdown
  → Add PDF via rust crate (pdf-extract or lopdf) later
```

**Effort**: M-L (3 sessions). FTS5 is already in rusqlite feature set.

---

### Wave 7: Multi-Platform Messaging (AstrBot → Forge)

**Goal**: Receive commands and send notifications via Telegram/Slack/Discord/etc.

**What changes:**

```
New crate: forge-messaging (~900 LOC)
  - MessageBridge: Trait for platform adapters
  - TelegramAdapter: Bot API via reqwest (no external SDK — HTTP calls only)
  - SlackAdapter: Webhook + Events API via reqwest
  - DiscordAdapter: Bot gateway via tungstenite WebSocket
  - IntentRouter: Parse incoming message → action:
    - "@agent do X" → spawn agent run
    - "status" → return active sessions summary
    - "tasks" → list pending tasks
    - "search KB: X" → knowledge base search
  - NotificationRouter: ForgeEvent → send to user's preferred platform
  - Platform config stored in DB per user

New DB migration: 0015_messaging.sql
  - messaging_configs (id, company_id, platform, config_json, enabled, timestamps)
  - notification_prefs (id, user_identifier, platform, channel_id, event_filters)

Extend forge-api:
  - /api/v1/messaging/configs    — CRUD platform connections
  - /api/v1/messaging/test       — send test message
  - POST /api/v1/webhooks/slack  — Slack events endpoint
  - POST /api/v1/webhooks/telegram — Telegram webhook endpoint
  - POST /api/v1/webhooks/discord  — Discord interactions endpoint

Extend forge-core EventBus:
  - NotificationSubscriber: Listen for events → route to messaging platforms
  - Filter by notification_prefs (user chooses which events to receive where)

New frontend page: Messaging
  - Platform cards (Telegram, Slack, Discord) with connect/disconnect
  - Notification preferences per platform
  - Test message button
  - Message log

Alternative strategy (sidecar):
  If full Rust reimplementation is too much, run AstrBot as a sidecar:
  - forge-messaging: Thin HTTP bridge to AstrBot API
  - AstrBot handles all platform-specific logic
  - Forge sends/receives via AstrBot's REST API
  - Docker Compose: forge + astrbot containers
  - Trade-off: Loses single-binary, gains 16+ platforms immediately

Recommended: Start with sidecar (Wave 7a), migrate to native Rust (Wave 7b).
```

**Wave 7a (Sidecar — quick):**
```
forge-messaging (~300 LOC):
  - AstrBotBridge: HTTP client to AstrBot REST API
  - Forward agent results to AstrBot → AstrBot delivers to platforms
  - Receive messages from AstrBot → parse → route to Forge agents
  - FORGE_ASTRBOT_URL env var
```

**Wave 7b (Native — later):**
```
Reimplement top 3 platforms in Rust:
  - Telegram: HTTP bot API (well-documented, simple)
  - Slack: Webhook + Events API
  - Discord: WebSocket gateway
```

**Effort**: M (sidecar) or XL (native). Recommend sidecar first.

---

### Wave 8: Desktop Client (Open-Claude-Cowork → Forge)

**Goal**: Native desktop app that connects to Forge backend.

**What changes:**

```
Approach: Fork Open-Claude-Cowork, rewire to use Forge API instead of Claude SDK directly.

desktop/                            (new directory at repo root)
  package.json                      (Electron + React)
  src/
    electron/
      main.ts                       (unchanged window management)
      ipc-handlers.ts               (rewired: Claude SDK → Forge REST API)
      libs/
        forge-client.ts             (new: HTTP + WebSocket client to Forge API)
        session-store.ts            (kept: local cache, syncs with Forge)
    ui/
      App.tsx                       (extended with Forge views)
      components/
        OrgChart.tsx                (new: from Forge /api/v1/org-chart)
        TaskBoard.tsx               (new: from Forge /api/v1/sessions)
        CostDashboard.tsx           (new: from Forge /api/v1/analytics)
        PersonaCatalog.tsx          (new: from Forge /api/v1/personas)
        KnowledgeSearch.tsx         (new: from Forge /api/v1/knowledge)
      store/
        useAppStore.ts              (extended with Forge state)
        useForgeStore.ts            (new: companies, agents, personas, KB)

Key changes from original:
  1. Replace @anthropic-ai/claude-agent-sdk → forge-client.ts (REST + WS)
  2. Sessions created via Forge API (not local Claude process)
  3. Add sidebar sections: Org Chart, Personas, Knowledge, Analytics
  4. Permission control → Forge governance approvals
  5. Skills management → Forge skills + superpowers + plugins
```

**Build:**
```bash
cd desktop && pnpm install && pnpm build
# Produces: desktop/dist/AgentForge-{mac,win,linux}
```

**Effort**: L (4-5 sessions). Fork + rewire + add new views.

---

## Updated Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
  # Existing (9)
  "crates/forge-core",
  "crates/forge-agent",
  "crates/forge-db",
  "crates/forge-api",
  "crates/forge-process",
  "crates/forge-safety",
  "crates/forge-git",
  "crates/forge-mcp-bin",
  "crates/forge-app",
  # New (7)
  "crates/forge-org",
  "crates/forge-governance",
  "crates/forge-persona",
  "crates/forge-knowledge",
  "crates/forge-messaging",
  "crates/forge-adapter-hermes",
  "crates/forge-adapter-openclaw",
]
```

---

## Updated Middleware Chain

```
Current (10 middlewares):
  RateLimit → CircuitBreaker → CostCheck → SkillInjection →
  TaskTypeDetection → Persist → Spawn → ExitGate → QualityGate → SecurityScan

After expansion (14 middlewares):
  RateLimit → CircuitBreaker → CompanyBudgetCheck → CostCheck →
  SkillInjection → TaskTypeDetection → KnowledgeInjection → PersonaInjection →
  Persist → BackendRoute → Spawn → ExitGate → QualityGate → SecurityScan
                                      ↓
                            ┌─────────┴──────────┐
                            │  BackendRoute       │
                            │  claude | hermes    │
                            │  openclaw           │
                            └─────────────────────┘
```

---

## Updated DB Schema (11 → 22+ tables)

```
Existing (11 tables):
  agents, sessions, events, memory, hooks, skills, schedules,
  analytics, compactions, skill_rules, workflow_runs

New (11+ tables):
  personas, persona_divisions,                    (Wave 1)
  skill_task_routes,                               (Wave 2)
  companies, departments, org_positions,           (Wave 3)
  goals, approvals,                                (Wave 3)
  webhook_callbacks,                               (Wave 5)
  kb_documents, kb_chunks, kb_chunks_fts,          (Wave 6)
  messaging_configs, notification_prefs,           (Wave 7)

Total: ~22 tables
```

---

## Updated Frontend (12 → 20+ pages)

```
Existing (12 pages):
  Dashboard, Agents, Sessions, Memory, Hooks, Skills,
  Workflows, Schedules, Analytics, Settings, (+ sub-pages)

New pages:
  Personas       — 100+ agent catalog, division browser, hire flow    (Wave 1)
  Methodology    — TDD/debug/design workflow visualization            (Wave 2)
  Companies      — multi-company management, budget overview          (Wave 3)
  Org Chart      — tree visualization of agent hierarchy              (Wave 3)
  Goals          — hierarchical goal tracking                         (Wave 3)
  Approvals      — pending approval queue                             (Wave 3)
  Knowledge Base — document upload, search, chunk preview             (Wave 6)
  Messaging      — platform connections, notification preferences     (Wave 7)

Total: ~20 pages
```

---

## Updated Directory Structure

```
forge-project/
├── crates/
│   ├── forge-core/              (existing — extend events)
│   ├── forge-agent/             (existing — add company_id, backend_type)
│   ├── forge-db/                (existing — add new repos)
│   ├── forge-api/               (existing — add new routes)
│   ├── forge-process/           (existing — add backend routing)
│   ├── forge-safety/            (existing — add company budgets, security scan)
│   ├── forge-git/               (existing — unchanged)
│   ├── forge-mcp-bin/           (existing — extend with new tools)
│   ├── forge-app/               (existing — wire new crates)
│   ├── forge-org/               (NEW — companies, departments, org positions)
│   ├── forge-governance/        (NEW — goals, approvals, lineage)
│   ├── forge-persona/           (NEW — persona parser, catalog, mapper)
│   ├── forge-knowledge/         (NEW — documents, chunks, FTS5 search)
│   ├── forge-messaging/         (NEW — platform bridges, notifications)
│   ├── forge-adapter-hermes/    (NEW — Hermes process bridge)
│   └── forge-adapter-openclaw/  (NEW — OpenClaw webhook bridge)
│
├── frontend/                    (existing SvelteKit — extend with new pages)
│
├── desktop/                     (NEW — Electron app, forked from Open-Claude-Cowork)
│
├── personas/                    (NEW — 100+ persona .md files from agency-agents)
│   ├── engineering/
│   ├── design/
│   ├── marketing/
│   └── ... (11 divisions)
│
├── skills/                      (existing 10 + new imports)
│   ├── (existing 10 seeds)
│   ├── superpowers/             (NEW — 14 skills from superpowers)
│   └── claude-code-plugins/     (NEW — plugin skills adapted)
│
├── migrations/
│   ├── 0001-0008               (existing)
│   ├── 0009_personas.sql
│   ├── 0010_skill_task_routing.sql
│   ├── 0011_org_charts.sql
│   ├── 0012_agent_backends.sql
│   ├── 0013_webhook_callbacks.sql
│   ├── 0014_knowledge_base.sql
│   └── 0015_messaging.sql
│
├── docker/                      (NEW — for sidecar services)
│   ├── docker-compose.yml       (forge + astrbot + openclaw)
│   ├── Dockerfile.forge
│   └── Dockerfile.astrbot
│
├── docs/
│   ├── EXPANSION_PLAN.md        (this file)
│   └── (existing docs)
│
├── Cargo.toml                   (updated workspace members)
├── CLAUDE.md                    (updated)
├── NORTH_STAR.md                (updated)
└── README.md                    (updated)
```

---

## Wave Summary & Dependencies

```
Wave 1: Personas ──────────────────────── (no deps, start immediately)
Wave 2: Dev Methodology ───────────────── (no deps, start immediately)
Wave 3: Org Charts & Governance ───────── (depends on Wave 1 for divisions)
Wave 4: Hermes Adapter ────────────────── (depends on Wave 2 for skills)
Wave 5: OpenClaw Adapter ──────────────── (depends on Wave 3 for companies)
Wave 6: Knowledge Base ────────────────── (depends on Wave 3 for companies)
Wave 7: Messaging ─────────────────────── (depends on Wave 3 + 6)
Wave 8: Desktop Client ────────────────── (depends on all API waves)
```

**Parallel execution possible:**
- Wave 1 + Wave 2 can run in parallel (no deps)
- Wave 4 + Wave 5 can run in parallel (both are adapters)
- Wave 6 + Wave 7 can overlap (KB builds while messaging starts)

```
Timeline (parallel):

Session 1-2:  [Wave 1: Personas]  [Wave 2: Dev Methodology]
Session 3-5:  [Wave 3: Org Charts & Governance]
Session 5-7:  [Wave 4: Hermes]  [Wave 5: OpenClaw]
Session 7-9:  [Wave 6: Knowledge]  [Wave 7a: Messaging sidecar]
Session 9-12: [Wave 8: Desktop Client]
Session 12-13: Polish, test, docs
```

---

## Version Mapping

| Version | Waves | Headline |
|---------|-------|----------|
| **v0.7.0** | Wave 1 + 2 | "100+ Personas + Dev Methodology" |
| **v0.8.0** | Wave 3 | "Multi-Company Org Charts" |
| **v0.9.0** | Wave 4 + 5 | "Multi-Backend Execution (Hermes + OpenClaw)" |
| **v0.10.0** | Wave 6 + 7 | "Knowledge Base + Multi-Platform Messaging" |
| **v1.0.0** | Wave 8 + Polish | "AgentForge — Desktop + Web + Messaging" |

---

## What Stays Pure Rust (Single Binary)

The core philosophy of Claude Forge — **one binary, zero deps** — is preserved for:
- All existing functionality
- Persona catalog (Wave 1) — pure Rust markdown parsing
- Dev methodology skills (Wave 2) — markdown files
- Org charts & governance (Wave 3) — pure Rust + SQLite
- Knowledge base (Wave 6) — SQLite FTS5, no external vector DB
- Security scanning (Wave 2) — Rust regex

**What requires external processes:**
- Hermes adapter (Wave 4) — needs `hermes` Python process
- OpenClaw adapter (Wave 5) — needs OpenClaw gateway
- Messaging sidecar (Wave 7a) — needs AstrBot process
- Desktop client (Wave 8) — separate Electron binary

**Trade-off**: The single binary still works standalone with Claude backend, 100+ personas, dev workflows, org charts, KB, and security scanning. External backends (Hermes, OpenClaw, AstrBot) are optional add-ons detected at runtime.

---

## Environment Variables (New)

| Var | Default | Wave | Purpose |
|-----|---------|------|---------|
| `FORGE_PERSONAS_DIR` | `./personas` | 1 | Persona markdown directory |
| `FORGE_HERMES_COMMAND` | `hermes` | 4 | Hermes CLI path |
| `FORGE_HERMES_HOME` | `~/.hermes` | 4 | Hermes config/state directory |
| `FORGE_OPENCLAW_URL` | *(none)* | 5 | OpenClaw gateway URL |
| `FORGE_OPENCLAW_TOKEN` | *(none)* | 5 | OpenClaw auth token |
| `FORGE_ASTRBOT_URL` | *(none)* | 7 | AstrBot API URL |
| `FORGE_TELEGRAM_TOKEN` | *(none)* | 7 | Telegram bot token (native) |
| `FORGE_SLACK_TOKEN` | *(none)* | 7 | Slack bot token (native) |
| `FORGE_DISCORD_TOKEN` | *(none)* | 7 | Discord bot token (native) |

---

## MCP Server Extensions

Extend `forge-mcp-bin` with new tools:

```
Existing (10 tools):
  forge_agent_create/list/get/delete, forge_run,
  forge_session_list/get/export, forge_config_get, forge_health

New tools (10+):
  forge_persona_list        — browse persona catalog
  forge_persona_hire        — create agent from persona
  forge_company_create      — create company
  forge_org_chart           — get org chart tree
  forge_goal_create         — create goal
  forge_knowledge_search    — search knowledge base
  forge_knowledge_upload    — add document to KB
  forge_approval_list       — list pending approvals
  forge_approval_decide     — approve/reject
  forge_message_send        — send message via platform

New resources (5+):
  forge://personas          — persona catalog
  forge://companies         — company list
  forge://org-chart/:id     — org chart for company
  forge://knowledge         — knowledge base documents
  forge://approvals         — pending approvals
```

---

*This expansion plan preserves Claude Forge's core strengths (Rust performance, single binary,
zero deps, 150+ tests) while systematically absorbing the best capabilities from all 8 repos.*
