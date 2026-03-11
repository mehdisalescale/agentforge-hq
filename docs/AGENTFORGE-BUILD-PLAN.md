# AgentForge — End-to-End Build Plan

> **From Proposal to Complete Product**
>
> Date: 2026-03-11
>
> Foundation: Paperclip (the orchestration engine with adapter system, org charts, budgets, tasks, 35+ table PostgreSQL schema)

---

## Reality Check: Two Products, Pick One

There are **two separate products** in this workspace:

| Product | Stack | Status | What It Is |
|---------|-------|--------|-----------|
| **forge-project** | Rust + Svelte + SQLite | v0.5.0, 150 tests | Standalone local orchestrator. Does NOT integrate the 8 repos. |
| **AgentForge (this plan)** | Node.js + React + PostgreSQL | Proposal stage | Combines all 8 repos with Paperclip as foundation. |

**This plan follows the AgentForge path** — using Paperclip as the base and integrating the other 7 repos into it.

### Why Paperclip as Foundation

- Production-grade adapter system designed to plug in new runtimes
- 35+ table PostgreSQL schema with multi-tenancy, budgets, approvals, audit trail
- Express 5 API + React 19 UI (full CRUD for agents, tasks, org charts, costs)
- Existing `adapter-openclaw-gateway` proves the adapter pattern works
- Auth (better-auth), WebSocket real-time, Docker Compose deployment
- pnpm monorepo structure ready for new packages

---

## Phase 1 — Persona Catalog (1-2 days)

**Goal:** Users browse 100+ agent personas and hire them into org charts with one click.

**Source:** `agency-agents` (100+ markdown persona files across 11 divisions)

### Tasks

| # | Task | How | Effort |
|---|------|-----|--------|
| 1.1 | Copy `agency-agents/` into Paperclip as `packages/personas/` | File copy, add to pnpm workspace | 10 min |
| 1.2 | Write persona parser — read each `.md` file, extract structured data | TypeScript script (~200 LOC). Parse: name, role, division, system prompt, deliverables, success metrics, step-by-step workflows | 2-3 hrs |
| 1.3 | Create `agent_templates` seed script | Drizzle insert into Paperclip's database. Each persona → one template row with: `name`, `division`, `persona_markdown`, `deliverables_json`, `metrics_json`, `default_adapter_config` | 1-2 hrs |
| 1.4 | Build "Hire from Catalog" UI page | New React page at `/catalog`. Division filter sidebar, search bar, persona cards with preview modal. On "Hire" → prefill NewAgent form | 4-6 hrs |
| 1.5 | Wire persona into agent config on hire | When user clicks "Hire", auto-populate: `agent.name`, `agent.role`, `agent.adapterConfig.instructions` (from persona markdown), department placement (from division) | 2-3 hrs |
| 1.6 | Map 11 divisions to Paperclip org chart departments | `engineering/` → Engineering dept, `design/` → Design dept, etc. Auto-create departments if they don't exist | 1-2 hrs |
| 1.7 | Add persona-based task template suggestions | When creating an issue for an agent, suggest task templates based on that agent's persona workflows | 2-3 hrs |

### Deliverable

Browse 100+ agent personas by division → hire into org chart → agent comes pre-configured with personality, deliverables, success metrics, and workflows.

### Integration Points

```
agency-agents/engineering/frontend-developer.md
    ↓ parse (1.2)
agent_templates table (name, division, persona, deliverables, metrics)
    ↓ user clicks "Hire" (1.4)
agents table (adapterConfig.instructions = persona system prompt)
    ↓ org chart placement (1.6)
department + reporting chain + task queue
```

---

## Phase 2 — Hermes Execution Engine (1-2 weeks)

**Goal:** Agents execute real work with 40+ tools, learn from experience, report back.

**Source:** `hermes-agent` (Python agent runtime with tools, memory, learning loop, 6 terminal backends)

### Tasks

| # | Task | How | Effort |
|---|------|-----|--------|
| 2.1 | Add `hermes-agent/` as `packages/runtime-hermes/` | Copy into monorepo. Add Dockerfile for Python environment. Keep as separate service (not bundled into Node server) | 2-3 hrs |
| 2.2 | Create `adapter-hermes` package | Copy `adapter-openclaw-gateway/` as template. New package at `packages/adapters/hermes/`. Implement `ServerAdapterModule` interface | 1-2 days |
| 2.3 | Implement `execute()` in adapter-hermes | Call Hermes Python agent via HTTP (Hermes has a gateway server) or subprocess. Pass: persona system prompt, task description, allowed tools, workspace path, memory context | 2-3 days |
| 2.4 | Implement heartbeat → Hermes wakeup bridge | When Paperclip heartbeat fires for a Hermes-adapted agent: create `AdapterExecutionContext` → call adapter-hermes `execute()` → stream logs back | 1 day |
| 2.5 | Inject persona into Hermes system prompt at runtime | adapter-hermes reads agent's `adapterConfig.instructions` (populated from persona in Phase 1) → passes as Hermes `system_prompt` parameter | 3-4 hrs |
| 2.6 | Map Hermes results back to Paperclip | Parse Hermes output → create issue comments (work summary), cost events (token usage), activity log entries. Update issue status based on exit code | 1 day |
| 2.7 | Sync Hermes MEMORY.md with Paperclip | After each run, read agent's `MEMORY.md` → store in `agent_runtime_state` table. On next run, inject previous memory into Hermes context | 4-6 hrs |
| 2.8 | Expose terminal backend selection in UI | Add dropdown to agent config: Local, Docker, SSH, Modal, Daytona, Singularity. Store in `adapterConfig.terminalBackend` | 3-4 hrs |
| 2.9 | Wire Hermes subagent delegation to Paperclip child tasks | When Hermes spawns a subagent, create a child issue in Paperclip. Link parent/child. Track cost separately | 1 day |
| 2.10 | Wire Hermes skill creation into capability tracking | When Hermes learning loop creates a new skill, log it in activity trail. Update agent's capability profile | 4-6 hrs |
| 2.11 | Configure model fallback with OpenClaw | For simpler tasks, route through OpenClaw adapter (already exists). Use Hermes for complex tasks requiring tools. Add routing logic to heartbeat scheduler | 1 day |

### Deliverable

Agents execute work with 40+ tools (web browsing, terminal, file operations, code execution, vision, planning). They learn from experience via MEMORY.md. Results flow back into Paperclip's task system with full cost tracking and audit trail.

### Integration Points

```
Paperclip heartbeat fires
    ↓
adapter-hermes receives AdapterExecutionContext
    ↓
Hermes AIAgent.run() with:
  - persona from agency-agents (system prompt)
  - task from Paperclip issue (user prompt)
  - tools filtered by agent role
  - memory from previous runs (MEMORY.md)
  - terminal backend from agent config
    ↓
Hermes executes with 40+ tools, spawns subagents if needed
    ↓
Results posted back:
  - Issue comments (work summary)
  - Cost events (token usage)
  - Activity log (audit trail)
  - MEMORY.md synced to agent_runtime_state
  - Child issues created for subagent work
```

---

## Phase 3 — Dev Methodology (3-5 days)

**Goal:** Engineering agents follow structured development workflows — TDD, design brainstorming, systematic debugging, code review.

**Source:** `superpowers` (dev skills) + `claude-code` (13 plugins)

### Tasks

| # | Task | How | Effort |
|---|------|-----|--------|
| 3.1 | Copy `superpowers/` into `packages/skills-superpowers/` | File copy. These are markdown skill definitions — no build step needed | 30 min |
| 3.2 | Copy `claude-code/plugins/` into `packages/plugins-claude-code/` | File copy. Plugin definitions are markdown + config files | 30 min |
| 3.3 | Build task-type → skill mapping engine | TypeScript module. Maps task labels/types to skill sets: `bug` → `systematic-debugging` + `TDD`, `feature` → `brainstorming` + `writing-plans` + `subagent-driven-development`, `review` → `code-review` | 4-6 hrs |
| 3.4 | Inject skills into Hermes system prompt based on task type | adapter-hermes reads task labels → loads matching skill markdown → appends to system prompt before Hermes execution | 3-4 hrs |
| 3.5 | Integrate code review plugin | When a PR/review task is assigned: load PR Review Toolkit plugin → run 6 parallel review agents (comments, tests, errors, types, quality, simplification) → aggregate results with confidence scores (80+ threshold) → post as issue comment | 1-2 days |
| 3.6 | Integrate security hooks | Post-processing step in adapter-hermes: scan all agent-generated code for 9 OWASP patterns (command injection, XSS, eval, dangerous HTML, etc.). Block commit if violations found. Log in activity trail | 4-6 hrs |
| 3.7 | Wire feature dev plugin into Paperclip task states | Map 7-phase workflow (explore → design → implement → review) to Paperclip issue status transitions. Auto-advance status as agent completes each phase | 4-6 hrs |
| 3.8 | Enable hookify rules per company | Load custom behavior rules from markdown files per company. Rules stored in company settings. Applied as guardrails during agent execution | 4-6 hrs |
| 3.9 | Add "Dev Methodology" toggle in agent config UI | Dropdown in agent settings: `strict-tdd` (RED-GREEN-REFACTOR mandatory), `tdd-lite` (tests encouraged), `none`. Stored in `adapterConfig.devMethodology` | 2-3 hrs |
| 3.10 | Connect git worktree management | Each engineering agent gets an isolated git worktree for their work. Managed via superpowers worktree skill. Workspace path passed to Hermes | 4-6 hrs |

### Deliverable

Engineering agents follow TDD (write tests before code), produce design specs before implementation, self-review with 6 parallel agents, and pass security scans. Non-engineering agents are unaffected.

### Skill Injection Flow

```
Paperclip assigns task with label "feature" to Backend Architect agent
    ↓
adapter-hermes detects task type → loads skills:
  - superpowers/brainstorming/SKILL.md
  - superpowers/writing-plans/SKILL.md
  - superpowers/subagent-driven-development/SKILL.md
  - superpowers/test-driven-development/SKILL.md
    ↓
Skills appended to Hermes system prompt (after persona, before task)
    ↓
Agent follows structured methodology:
  1. Brainstorm → Socratic design exploration → spec document
  2. Plan → bite-sized tasks with code samples
  3. Implement → RED test → GREEN code → REFACTOR
  4. Review → 6 parallel agents, confidence scoring
  5. Security scan → OWASP pattern check
    ↓
Results posted to Paperclip with methodology compliance report
```

---

## Phase 4 — Communication Hub (1-2 weeks)

**Goal:** Interact with the AI workforce from any messaging platform (Slack, Telegram, Discord, WeChat, 12+ more).

**Source:** `AstrBot` (Python multi-platform chatbot framework with knowledge base, 1000+ plugins)

### Tasks

| # | Task | How | Effort |
|---|------|-----|--------|
| 4.1 | Add `AstrBot/` as `packages/messaging-astrbot/` | Copy into monorepo. Add Dockerfile for Python environment. Runs as separate service | 2-3 hrs |
| 4.2 | Build AstrBot ↔ Paperclip API bridge | AstrBot plugin ("star") that calls Paperclip REST API. Translates chat messages into API calls and API responses into chat messages | 1-2 days |
| 4.3 | Create intent router | Message parsing logic in AstrBot plugin: `@agent-name do X` → route to specific agent (create task), `create task: X` → create Paperclip issue, `status?` → fetch company dashboard, `what about X?` → AstrBot KB search | 1-2 days |
| 4.4 | Connect AstrBot knowledge base to Paperclip context | Feed project docs, issue history, agent outputs into AstrBot's FAISS vector DB. Enable semantic search across all organizational knowledge | 1 day |
| 4.5 | Route Hermes outputs through AstrBot | When Hermes completes a task or cron job, send notification through AstrBot to user's preferred platform. Configurable per user | 4-6 hrs |
| 4.6 | Unify LLM provider configuration | Single config file for all LLM providers (OpenAI, Anthropic, Google, DeepSeek, Ollama, 25+ more). Shared between Paperclip, Hermes, and AstrBot | 4-6 hrs |
| 4.7 | Enable AstrBot community plugins | Expose AstrBot's 1000+ community plugins as optional capabilities. Install/uninstall from Paperclip admin UI | 1 day |
| 4.8 | Integrate STT/TTS for voice interaction | Enable voice messages on supported platforms. AstrBot handles transcription (Whisper) and synthesis (Edge TTS, ElevenLabs). Route transcribed text through normal intent router | 1 day |
| 4.9 | Add platform preference settings per user | User settings page: preferred notification platform, language, voice preferences. Stored in Paperclip user profile | 3-4 hrs |

### Deliverable

Manage the entire AI workforce from Slack, Telegram, Discord, WeChat, or any of 16+ platforms. Create tasks, check status, chat with agents, search knowledge base — all from your messaging app. Voice-enabled.

### Message Flow

```
User sends message on Telegram: "@backend-architect add user auth to the API"
    ↓
AstrBot Telegram adapter receives message
    ↓
AgentForge intent router parses:
  - Target: "backend-architect" (agent name)
  - Action: "add user auth to the API" (task description)
    ↓
Bridge calls Paperclip API:
  POST /api/issues { title: "Add user auth to the API", assigneeAgentId: "backend-architect-id" }
    ↓
Paperclip creates issue, triggers heartbeat for assigned agent
    ↓
Agent executes via Hermes (Phase 2) with dev skills (Phase 3)
    ↓
On completion, result routed back through AstrBot → Telegram
    ↓
User receives: "✓ Task PAP-42 completed. Added JWT auth with refresh tokens. PR #17 ready for review."
```

---

## Phase 5 — Desktop Client (1-2 weeks)

**Goal:** Native desktop application for power users with full AgentForge functionality.

**Source:** `Open-Claude-Cowork` (Electron + React + Zustand + better-sqlite3)

### Tasks

| # | Task | How | Effort |
|---|------|-----|--------|
| 5.1 | Add `Open-Claude-Cowork/` as `packages/desktop/` | Copy into monorepo. Keep Electron build separate from web UI | 2-3 hrs |
| 5.2 | Replace Claude SDK calls with Paperclip API | Swap `@anthropic-ai/claude-agent-sdk` calls with fetch calls to Paperclip Express API. Streaming via WebSocket (already supported by Paperclip) | 2-3 days |
| 5.3 | Add org chart sidebar panel | New React component. Fetches from `GET /api/agents` with `reportsTo` hierarchy. Renders tree view. Click agent → open chat | 1 day |
| 5.4 | Add task board view | Kanban board alongside chat. Fetches from `GET /api/issues`. Columns: backlog, in_progress, in_review, done. Drag to reassign | 1 day |
| 5.5 | Add cost dashboard widget | Summary card in sidebar. Fetches from `GET /api/costs`. Shows: monthly spend, per-agent breakdown, budget remaining | 4-6 hrs |
| 5.6 | Connect skills management UI | List installed superpowers skills + claude-code plugins. Enable/disable per agent. Install new skills from file | 1 day |
| 5.7 | Add knowledge base search panel | Search panel that queries AstrBot KB (via Paperclip API proxy). Show results with source attribution | 4-6 hrs |
| 5.8 | Integrate permission control with approval gates | When agent requests sensitive action, show approval dialog in desktop app. Connect to Paperclip's approval system | 1 day |
| 5.9 | Add multi-company switcher | Header dropdown to switch between companies. Persists selection in local SQLite | 3-4 hrs |

### Deliverable

Native desktop app (macOS, Windows, Linux) with: real-time agent chat with streaming, org chart visualization, kanban task board, cost dashboard, knowledge base search, skills management, and multi-company support.

### Desktop Architecture

```
Electron Main Process
    ├── IPC Bridge → Paperclip REST API (agents, tasks, budgets, approvals)
    ├── IPC Bridge → Paperclip WebSocket (real-time streaming, live events)
    ├── IPC Bridge → AstrBot KB API (knowledge search)
    └── Local SQLite (session history, UI preferences, cached data)

Electron Renderer (React)
    ├── Left Sidebar: Company switcher, org chart tree, agent list
    ├── Center: Chat view (streaming) OR task board (kanban)
    ├── Right Panel: Agent detail, cost widget, KB search
    └── Bottom: Skills/plugins management bar
```

---

## Phase 6 — Production Polish (1-2 weeks)

**Goal:** One-command deployment, unified auth, marketplace, documentation.

### Tasks

| # | Task | How | Effort |
|---|------|-----|--------|
| 6.1 | Unified authentication | Paperclip's better-auth as SSO for all services. AstrBot and Hermes validate JWT tokens issued by Paperclip. Single login across web, desktop, and messaging | 2-3 days |
| 6.2 | Single Docker Compose deployment | `docker-compose.yml` with services: `core` (Paperclip), `hermes` (Python runtime), `astrbot` (messaging), `openclaw` (lightweight runtime), `postgres`, `redis` (optional caching). One `docker compose up` | 1-2 days |
| 6.3 | Combined plugin/skill marketplace UI | New page in Paperclip UI. Three tabs: Superpowers Skills, Claude-Code Plugins, AstrBot Plugins. Browse, install, uninstall, configure. Version tracking | 2-3 days |
| 6.4 | Mobile-responsive web dashboard | Tailwind responsive breakpoints on all Paperclip UI pages. Touch-friendly task board. Mobile nav menu | 1-2 days |
| 6.5 | E2E test suite | Playwright tests covering: hire agent from catalog, create task, agent executes via Hermes, results appear in UI, messaging integration | 2-3 days |
| 6.6 | Documentation site | Quickstart guide, architecture overview, API reference (OpenAPI), deployment guide, plugin development guide. Built with Astro or VitePress | 2-3 days |
| 6.7 | Company template gallery | Pre-built org blueprints: "AI Startup" (CEO + engineers + designer + marketer), "Dev Agency" (PM + full eng team), "Support Center" (responders + analytics), "Content Team" (writers + strategists). One-click deploy | 1 day |
| 6.8 | Desktop app distribution | Electron Forge for platform builds. Auto-updater via GitHub Releases. Code signing for macOS. MSI installer for Windows | 1-2 days |
| 6.9 | Performance optimization | Redis caching layer for hot API paths. Connection pooling for PostgreSQL. Batch writes for cost events. WebSocket message throttling | 1-2 days |
| 6.10 | Fine-tuning export | Export agent interaction trajectories from Hermes in JSONL format for fine-tuning custom models. Compression and filtering options | 1 day |

### Deliverable

```bash
# Web + API + all services
docker compose up

# OR native desktop
brew install --cask agentforge   # macOS
winget install agentforge        # Windows
```

Complete AI workforce platform — hire 100+ agent personas, organize in org charts, execute with 40+ tools, follow TDD methodology, interact from 16+ messaging platforms or native desktop app. Full cost tracking, audit trail, knowledge base, and plugin ecosystem.

---

## Docker Compose (Target)

```yaml
services:
  postgres:
    image: postgres:17
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: agentforge
      POSTGRES_PASSWORD: ${DB_PASSWORD}

  core:
    build: ./docker/Dockerfile.core
    ports:
      - "3100:3100"
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://postgres:${DB_PASSWORD}@postgres:5432/agentforge
      AUTH_SECRET: ${AUTH_SECRET}

  hermes:
    build: ./docker/Dockerfile.hermes
    depends_on:
      - core
    environment:
      PAPERCLIP_API_URL: http://core:3100/api
      PAPERCLIP_API_KEY: ${HERMES_API_KEY}

  astrbot:
    build: ./docker/Dockerfile.astrbot
    depends_on:
      - core
    environment:
      PAPERCLIP_API_URL: http://core:3100/api
      PAPERCLIP_API_KEY: ${ASTRBOT_API_KEY}
      TELEGRAM_TOKEN: ${TELEGRAM_TOKEN}
      SLACK_TOKEN: ${SLACK_TOKEN}
      DISCORD_TOKEN: ${DISCORD_TOKEN}

  openclaw:
    build: ./docker/Dockerfile.openclaw
    depends_on:
      - core
    environment:
      PAPERCLIP_API_URL: http://core:3100/api

volumes:
  pgdata:
```

---

## Monorepo Structure (Target)

```
agentforge/
├── packages/
│   ├── core/                          # Paperclip (orchestration engine)
│   │   ├── server/                    # Express.js API
│   │   ├── ui/                        # React 19 web dashboard
│   │   ├── cli/                       # CLI tool
│   │   └── packages/
│   │       ├── db/                    # PostgreSQL schema (Drizzle)
│   │       ├── shared/                # Shared types
│   │       └── adapters/
│   │           ├── hermes/            # NEW — adapter-hermes
│   │           ├── openclaw-gateway/  # Existing
│   │           ├── claude-local/      # Existing
│   │           └── adapter-utils/     # Shared interface
│   │
│   ├── runtime-hermes/                # Hermes-Agent (Python)
│   │   ├── hermes_agent/              # Core + 40+ tools
│   │   └── gateway/                   # HTTP gateway
│   │
│   ├── runtime-openclaw/              # OpenClaw (Node.js)
│   │   └── gateway/                   # Gateway + Docker sandbox
│   │
│   ├── personas/                      # Agency-Agents (100+ .md files)
│   │   ├── engineering/               # 16 personas
│   │   ├── design/                    # 8 personas
│   │   ├── marketing/                 # 17 personas
│   │   ├── product/                   # 10 personas
│   │   ├── testing/                   # 8 personas
│   │   ├── support/                   # 6 personas
│   │   └── .../                       # 5 more divisions
│   │
│   ├── skills-superpowers/            # Dev methodology (markdown)
│   │   └── skills/
│   │       ├── brainstorming/
│   │       ├── test-driven-development/
│   │       ├── systematic-debugging/
│   │       └── .../
│   │
│   ├── plugins-claude-code/           # Dev plugins (13 plugins)
│   │   └── plugins/
│   │       ├── code-review/
│   │       ├── feature-dev/
│   │       ├── pr-review-toolkit/
│   │       ├── security-guidance/
│   │       └── .../
│   │
│   ├── messaging-astrbot/            # AstrBot (Python)
│   │   ├── astrbot/                   # Core framework
│   │   └── dashboard/                 # Admin panel
│   │
│   └── desktop/                       # Electron app
│       ├── src/electron/              # Main process
│       └── src/ui/                    # React renderer
│
├── docker/
│   ├── docker-compose.yml
│   ├── Dockerfile.core
│   ├── Dockerfile.hermes
│   ├── Dockerfile.openclaw
│   └── Dockerfile.astrbot
│
├── docs/
│   ├── quickstart.md
│   ├── architecture.md
│   ├── api-reference.md
│   └── deployment.md
│
├── pnpm-workspace.yaml
├── package.json
└── README.md
```

---

## What NOT to Do

| Don't | Why |
|-------|-----|
| Merge forge-project into this | Rust+Svelte+SQLite vs Node+React+PostgreSQL — completely different architectures |
| Merge AstrBot's Vue dashboard into Paperclip's React UI | Keep AstrBot as a backend service, use Paperclip UI as the primary frontend |
| Rewrite any of the 8 repos | The whole point is integration, not reimplementation |
| Try all 6 phases at once | Phase 1 alone makes this demoable in a day |
| Build custom messaging adapters | AstrBot already has 16+ — just use them |
| Create a new database schema | Paperclip's 35+ table schema already covers everything |

---

## Timeline Summary

| Phase | What | Time | Cumulative |
|-------|------|------|-----------|
| 1 | Persona Catalog (agency-agents → Paperclip) | 1-2 days | 2 days |
| 2 | Execution Engine (Hermes adapter) | 1-2 weeks | 2 weeks |
| 3 | Dev Methodology (superpowers + plugins) | 3-5 days | 3 weeks |
| 4 | Communication Hub (AstrBot integration) | 1-2 weeks | 5 weeks |
| 5 | Desktop Client (Open-Claude-Cowork) | 1-2 weeks | 7 weeks |
| 6 | Production Polish | 1-2 weeks | 8-9 weeks |

**MVP (Phases 1-2):** ~2 weeks — hire agents from catalog, they execute real work with 40+ tools.

**Full Product (Phases 1-6):** ~8-9 weeks — complete AI workforce platform.

---

## Start Here

**Phase 1, Task 1.1:** Copy agency-agents into Paperclip monorepo and write the persona parser. This is a 2-hour task that immediately makes the platform feel like a workforce marketplace.

```bash
# Step 1: Copy personas
cp -r /Users/bm/cod/trend/11-march/agency-agents/ /Users/bm/cod/trend/10-march/paperclip/packages/personas/

# Step 2: Write persona parser (see Phase 1, Task 1.2)
# Step 3: Seed database (see Phase 1, Task 1.3)
# Step 4: Build catalog UI (see Phase 1, Task 1.4)
```

---

*Generated 2026-03-11. Based on analysis of all 8 source repositories + existing forge-project.*
