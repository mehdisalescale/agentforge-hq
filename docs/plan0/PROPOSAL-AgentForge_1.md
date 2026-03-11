# AgentForge — A Self-Improving AI Workforce Platform

> **Proposal: Combining 4 Open-Source Repos into One Integrated Application**
>
> Date: 2026-03-11

---

## Executive Summary

AgentForge is a unified platform that combines four complementary open-source projects into a single, cohesive AI workforce management system. It allows users to hire specialized AI agents from a curated catalog, organize them into teams with org charts and budgets, let them execute real work with 40+ tools while learning from experience, and communicate with them from any messaging platform.

---

## Source Repositories

| Repo | Role | Key Contribution |
|------|------|-----------------|
| **Paperclip** | Orchestration layer | Org charts, budgets, task management, governance, heartbeat protocol |
| **Agency-Agents** | Persona library | 100+ specialized agent personalities across 11 divisions |
| **Hermes-Agent** | Execution runtime | 40+ tools, self-improving learning loop, persistent memory, 6 terminal backends |
| **AstrBot** | Communication layer | 16+ messaging platforms, knowledge base, 1000+ plugins, web dashboard |

---

## Product Requirements

### From Paperclip (Orchestration)

- **PR-1**: Multi-agent orchestration with hierarchical org charts (CEO → managers → ICs)
- **PR-2**: Budget and cost control at company and per-agent level with automatic pausing at threshold
- **PR-3**: Task ticketing system with full audit trail (backlog → todo → in_progress → in_review → done)
- **PR-4**: Heartbeat-based agent wakeup protocol (scheduled or event-triggered)
- **PR-5**: Multi-company isolation — run multiple AI companies from a single deployment
- **PR-6**: Board-level approval gates for agent hiring and strategic decisions
- **PR-7**: Pluggable adapter system for integrating any agent runtime
- **PR-8**: Atomic task checkout to prevent double-work
- **PR-9**: Goal lineage — every task traces back to company mission
- **PR-10**: Portable company templates (export/import org blueprints)

### From Agency-Agents (Persona Library)

- **PR-11**: 100+ battle-tested agent personas across 11 divisions (engineering, design, marketing, product, PM, testing, support, spatial computing, specialized, game dev, paid media)
- **PR-12**: Each agent has defined personality, deliverables, success metrics, and step-by-step workflows
- **PR-13**: Division-based organizational structure mapping
- **PR-14**: Multi-tool format compatibility (Claude Code, Cursor, Codex, OpenCode, etc.)
- **PR-15**: Agent orchestration patterns for parallel and sequential multi-agent work
- **PR-16**: MCP memory integration for cross-session agent handoffs

### From Hermes-Agent (Execution Runtime)

- **PR-17**: Closed learning loop — agents create and improve skills autonomously from experience
- **PR-18**: 40+ built-in tools organized by category (web, terminal, files, browser, code execution, vision, planning, cron)
- **PR-19**: Persistent agent memory (MEMORY.md for learned knowledge, USER.md for user preferences)
- **PR-20**: 6 terminal backends for execution isolation (local, Docker, SSH, Modal, Daytona, Singularity)
- **PR-21**: Session persistence with FTS5 full-text search across all conversations
- **PR-22**: Subagent delegation — spawn child agents for parallel or isolated workstreams
- **PR-23**: Mixture-of-agents — multi-model collaborative reasoning
- **PR-24**: Prompt injection detection and security scanning on memory/context files
- **PR-25**: Batch trajectory generation and compression for fine-tuning

### From AstrBot (Communication Layer)

- **PR-26**: 16+ messaging platform integrations (Telegram, Slack, Discord, WeChat, QQ, Feishu, DingTalk, LINE, WhatsApp, Signal, etc.)
- **PR-27**: Knowledge base with semantic search (FAISS vector DB, hybrid sparse/dense retrieval)
- **PR-28**: 1000+ community plugin ecosystem with one-click install
- **PR-29**: Agent sandbox for safe, isolated code execution
- **PR-30**: Multi-LLM provider support (30+ providers including OpenAI, Anthropic, Google, DeepSeek, Ollama)
- **PR-31**: Web dashboard with real-time streaming, session management, and admin controls
- **PR-32**: Multi-stage event processing pipeline (preprocessing, routing, response, decoration)
- **PR-33**: Internationalization (i18n) support

---

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      AgentForge UI                          │
│           (Unified React Dashboard + AstrBot WebChat)       │
│                                                             │
│  Org Chart    Task Board    Cost Dashboard    Knowledge Base│
│  Agent Catalog    Chat Interface    Plugin Marketplace      │
└──────────────────────────┬──────────────────────────────────┘
                           │ REST + WebSocket
┌──────────────────────────┴──────────────────────────────────┐
│                    Unified API Gateway                       │
│                                                             │
│  ┌──────────────┐  ┌───────────────┐  ┌──────────────────┐ │
│  │  Paperclip    │  │   AstrBot     │  │  Hermes          │ │
│  │  Orchestrator │  │   Msg Gateway │  │  Agent Runtime   │ │
│  │  (Express.js) │  │   (Quart)     │  │  (Python)        │ │
│  │               │  │               │  │                  │ │
│  │  • Org charts │  │  • 16+ IM     │  │  • 40+ tools     │ │
│  │  • Budgets    │  │  • Knowledge  │  │  • Learning loop │ │
│  │  • Tasks      │  │  • Plugins    │  │  • Memory        │ │
│  │  • Governance │  │  • Sandbox    │  │  • Subagents     │ │
│  │  • Heartbeat  │  │  • Dashboard  │  │  • 6 backends    │ │
│  └───────┬──────┘  └───────┬───────┘  └────────┬─────────┘ │
└──────────┼─────────────────┼───────────────────┼────────────┘
           │                 │                   │
┌──────────┴─────────────────┴───────────────────┴────────────┐
│                  Agent Persona Layer                         │
│              (agency-agents: 100+ personas)                  │
│                                                             │
│  ┌───────────┐ ┌────────┐ ┌───────────┐ ┌───────────────┐  │
│  │Engineering│ │ Design │ │ Marketing │ │ Product / PM  │  │
│  │ 16 agents │ │8 agents│ │ 17 agents │ │  10 agents    │  │
│  └───────────┘ └────────┘ └───────────┘ └───────────────┘  │
│  ┌───────────┐ ┌────────┐ ┌───────────┐ ┌───────────────┐  │
│  │  Testing  │ │Support │ │  Spatial  │ │  Specialized  │  │
│  │ 8 agents  │ │6 agents│ │ 6 agents  │ │  15+ agents   │  │
│  └───────────┘ └────────┘ └───────────┘ └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
           │                 │                   │
┌──────────┴─────────────────┴───────────────────┴────────────┐
│                    Shared Data Layer                         │
│                                                             │
│  PostgreSQL          SQLite             FAISS               │
│  (Paperclip DB)      (Hermes sessions)  (AstrBot KB)        │
│  • companies         • messages         • embeddings        │
│  • agents            • sessions_fts     • document chunks   │
│  • issues            • agent memory     • search index      │
│  • cost_events                                              │
│  • heartbeat_runs    Markdown Files                         │
│  • approvals         • MEMORY.md (per agent)                │
│                      • USER.md (per user)                   │
│                      • Agent personas (.md)                 │
└─────────────────────────────────────────────────────────────┘
```

### Integration Points

#### 1. Paperclip ← Agency-Agents (Persona → Template)

Agency-agents personas become first-class **agent templates** within Paperclip's hiring flow.

```
agency-agents/engineering/frontend-developer.md
    ↓ import
Paperclip DB: agent_templates table
    ↓ hire
Paperclip DB: agents table (with persona injected into config)
    ↓ assign
Paperclip: org chart position + task queue
```

- Division structure (engineering, design, marketing...) maps to Paperclip org chart departments
- Persona workflows become task template suggestions
- Success metrics from personas feed into Paperclip's performance tracking
- Agent deliverables define expected output types per task

#### 2. Paperclip ← Hermes-Agent (Orchestration → Execution)

Hermes becomes a new Paperclip adapter (`adapter-hermes`).

```
Paperclip heartbeat fires
    ↓
adapter-hermes receives wakeup payload
    ↓
Hermes AIAgent.run() with:
  - persona from agency-agents (system prompt)
  - task from Paperclip (user prompt)
  - tools filtered by agent role
  - memory from previous runs
    ↓
Hermes executes with 40+ tools
    ↓
Results posted back to Paperclip via API
    ↓
Hermes MEMORY.md synced to Paperclip agent_runtime_state
```

- Hermes' 6 terminal backends provide Paperclip's execution isolation options
- Hermes' learning loop (skill creation) feeds back into the agent's capability profile
- Hermes' session DB provides detailed execution logs for Paperclip's audit trail
- Subagent delegation maps to Paperclip's task hierarchy (parent → child tasks)

#### 3. Paperclip ← AstrBot (Orchestration → Communication)

AstrBot becomes the messaging facade for the entire platform.

```
User sends message on Telegram/Slack/Discord
    ↓
AstrBot platform adapter receives message
    ↓
AgentForge router determines intent:
  - Direct agent chat → route to specific Paperclip agent
  - Task creation → create Paperclip issue
  - Status query → fetch from Paperclip API
  - Knowledge question → AstrBot KB search
    ↓
Response sent back through originating platform
```

- AstrBot's event pipeline preprocesses and routes messages to the right Paperclip agent/task
- Knowledge base provides shared organizational context accessible to all agents
- AstrBot's plugin ecosystem extends agent capabilities beyond built-in tools
- Web dashboard merges with Paperclip's React UI for a unified admin experience

#### 4. Hermes ↔ AstrBot (Runtime ↔ Communication)

Cross-system synergies between the execution and communication layers.

```
Hermes cron job completes
    ↓
Output routed through AstrBot to user's preferred platform
    ↓
User responds on Telegram
    ↓
AstrBot routes response back to Hermes session
```

- AstrBot's 30+ LLM providers feed into Hermes' flexible model selection
- Both share SQLite session persistence patterns — unified into single session store
- AstrBot's sandbox complements Hermes' terminal backends for code execution
- Hermes' web search tools enhance AstrBot's knowledge base with live data

---

## Tech Stack

| Layer | Technology | Source |
|-------|-----------|--------|
| Frontend | React 19, Tailwind CSS 4, Radix UI, TanStack Query | Paperclip |
| API Server | Express.js 5 (primary), Quart bridge (messaging) | Paperclip + AstrBot |
| Database | PostgreSQL (Drizzle ORM) | Paperclip |
| Session Store | SQLite (WAL mode, FTS5) | Hermes |
| Vector DB | FAISS-CPU | AstrBot |
| Agent Runtime | Python 3.12+, asyncio | Hermes |
| Messaging | Platform-specific SDKs (16+) | AstrBot |
| LLM Providers | OpenAI, Anthropic, Google, DeepSeek, Ollama, 25+ more | AstrBot + Hermes |
| Auth | Better-auth (JWT) | Paperclip |
| Real-time | WebSocket (ws) | Paperclip |
| Deployment | Docker Compose, embedded Postgres | Paperclip |
| Package Mgmt | pnpm workspaces (monorepo) | Paperclip |
| Testing | Vitest + Playwright (E2E) | Paperclip |

---

## Implementation Plan

### Phase 1 — Foundation: Paperclip + Agency-Agents

**Goal**: Hire specialized agents from a catalog, organize into teams, assign tasks.

| Task | Description | Effort |
|------|-------------|--------|
| 1.1 | Create AgentForge monorepo, import Paperclip codebase as core | S |
| 1.2 | Build persona importer — parse all 100+ agency-agents markdown files into DB | M |
| 1.3 | Create `agent_templates` table and migration | S |
| 1.4 | Build "Hire from Catalog" UI — browse by division, search, preview persona | M |
| 1.5 | Map agency-agents divisions to Paperclip org chart departments | S |
| 1.6 | Auto-populate agent config (instructions, deliverables, metrics) from persona | M |
| 1.7 | Add persona-based task template suggestions when creating issues | M |

**Deliverable**: Users can browse 100+ agent personas, hire them into org charts, and manage tasks with full budget tracking.

---

### Phase 2 — Runtime: + Hermes-Agent

**Goal**: Agents actually execute work with tools, learn from experience.

| Task | Description | Effort |
|------|-------------|--------|
| 2.1 | Build `adapter-hermes` Paperclip adapter package | L |
| 2.2 | Implement heartbeat → Hermes wakeup bridge | M |
| 2.3 | Inject agency-agents persona into Hermes system prompt at runtime | M |
| 2.4 | Map Hermes tool results back to Paperclip issue comments + audit log | M |
| 2.5 | Sync Hermes MEMORY.md with Paperclip `agent_runtime_state` | M |
| 2.6 | Expose terminal backend selection in Paperclip agent config UI | S |
| 2.7 | Wire Hermes skill creation events into agent capability tracking | M |
| 2.8 | Connect Hermes subagent delegation to Paperclip child task creation | L |
| 2.9 | Add Hermes session search to Paperclip's activity/audit views | M |

**Deliverable**: Agents execute real work using 40+ tools, learn autonomously, and report results back to the orchestration layer.

---

### Phase 3 — Communication: + AstrBot

**Goal**: Interact with the AI workforce from any messaging platform.

| Task | Description | Effort |
|------|-------------|--------|
| 3.1 | Build AstrBot ↔ Paperclip API bridge (message routing) | L |
| 3.2 | Create intent router (direct chat, task creation, status query, KB search) | M |
| 3.3 | Connect AstrBot knowledge base to Paperclip project context | M |
| 3.4 | Merge AstrBot web dashboard components into Paperclip React UI | L |
| 3.5 | Route Hermes cron outputs through AstrBot to user's preferred platform | M |
| 3.6 | Unify LLM provider configuration (single config for all providers) | M |
| 3.7 | Enable AstrBot plugins as optional agent capabilities in Paperclip | M |
| 3.8 | Add platform preference settings per user (where to receive notifications) | S |

**Deliverable**: Full platform — manage AI workforce from Slack, Telegram, Discord, or any of the 16+ supported platforms.

---

### Phase 4 — Polish & Production

**Goal**: Production-ready deployment with unified experience.

| Task | Description | Effort |
|------|-------------|--------|
| 4.1 | Unified authentication across all subsystems | M |
| 4.2 | Single Docker Compose deployment with all services | M |
| 4.3 | Combined plugin/skill marketplace UI | M |
| 4.4 | Mobile-optimized responsive dashboard | M |
| 4.5 | Comprehensive E2E test suite | L |
| 4.6 | Documentation site with quickstart, guides, and API reference | L |
| 4.7 | Company template gallery (pre-built orgs: startup, agency, dev team) | M |
| 4.8 | Performance optimization and caching layer | M |

**Deliverable**: One-command deployment of a complete AI workforce platform.

---

## User Workflows

### Workflow 1: "Launch an AI Startup"

```
1. User creates a new company in AgentForge
2. Browses agent catalog → hires:
   - CEO (from specialized/agents-orchestrator)
   - Backend Architect (from engineering/)
   - Frontend Developer (from engineering/)
   - Growth Hacker (from marketing/)
   - UI Designer (from design/)
3. Arranges org chart: CEO manages all others
4. Sets monthly budget: $500
5. Creates company goal: "Build and launch an MVP SaaS product"
6. CEO agent wakes up, breaks goal into tasks, delegates to team
7. Engineers execute via Hermes (Docker backend), commit code to Git
8. Designer produces mockups, Growth Hacker plans GTM strategy
9. User monitors progress from Slack, asks questions, approves key decisions
10. All costs tracked, all work audited, all agents learning
```

### Workflow 2: "AI Customer Support Team"

```
1. User creates company, hires:
   - Support Responder (from support/)
   - Analytics Reporter (from support/)
   - Knowledge Base Manager (custom)
2. Connects Telegram + Discord + Slack via AstrBot
3. Uploads product docs to knowledge base
4. Customers message on any platform → routed to Support Responder
5. Agent searches KB, responds with context-aware answers
6. Analytics Reporter generates weekly reports → sent to Slack
7. Agents learn from interactions, improve responses over time
```

### Workflow 3: "Content Marketing Machine"

```
1. User creates company, hires:
   - Content Creator (from marketing/)
   - Twitter Strategist (from marketing/)
   - Reddit Community Builder (from marketing/)
   - SEO Specialist (from marketing/)
2. Sets weekly content calendar as recurring tasks
3. Content Creator writes articles using web search tools
4. Platform strategists adapt content for each channel
5. User reviews and approves via Telegram before publishing
6. All costs tracked per agent, per task
```

---

## Why These 4 Repos?

### Why NOT the other 4:

| Excluded Repo | Reason |
|--------------|--------|
| **claude-code** (plugins) | Subset of what agency-agents + Hermes already provide; Claude-specific |
| **Open-Claude-Cowork** | Desktop-only Electron GUI; AstrBot's web dashboard + messaging is more flexible and accessible |
| **openclaw** | Already exists as a Paperclip adapter; Hermes is more capable (40+ tools vs gateway-only, plus learning loop) |
| **superpowers** | Dev workflow skills only; agency-agents includes dev personas with workflows, and Hermes' skill system is more general |

### Why THESE 4 work together:

```
Paperclip provides: WHO does WHAT, WHEN, and at WHAT COST
Agency-Agents provides: HOW each agent thinks, communicates, and delivers
Hermes-Agent provides: WHERE and WITH WHAT tools agents execute
AstrBot provides: THROUGH WHICH channels users interact with agents
```

Each repo fills a gap the others leave open. No significant overlap. Maximum synergy.

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Time to first working agent | < 5 minutes |
| Supported agent personas | 100+ at launch |
| Messaging platforms | 16+ |
| Available tools per agent | 40+ |
| Deployment complexity | Single `docker compose up` |
| LLM providers supported | 30+ |
| Cost tracking accuracy | Per-token, per-agent |

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Mixed tech stack (Node.js + Python) | Deployment complexity | Docker Compose with per-service containers |
| Frontend merge (React + Vue) | Development effort | Rebuild AstrBot dashboard pages in React progressively |
| Session state across 3 systems | Data consistency | PostgreSQL as single source of truth; SQLite/FAISS as read-optimized caches |
| 100+ persona maintenance | Stale agents | Community contribution model (agency-agents is already MIT + community-driven) |
| LLM cost runaway | Budget overruns | Paperclip's built-in budget enforcement with automatic agent pausing |

---

## License

All four source repositories are open-source (MIT). AgentForge will be released under MIT license.

---

*This proposal was generated on 2026-03-11.*
