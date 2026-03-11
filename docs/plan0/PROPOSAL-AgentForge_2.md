# AgentForge — A Self-Improving AI Workforce Platform

> **Proposal: Combining 8 Open-Source Repos into One Integrated Application**
>
> Sources: `/Users/bm/cod/trend/10-march` (4 repos) + `/Users/bm/cod/trend/11-march` (4 repos)
>
> Date: 2026-03-11

---

## Executive Summary

AgentForge is a unified platform that combines eight complementary open-source projects into a single AI workforce management system. It allows users to hire specialized AI agents from a curated catalog of 100+ personas, organize them into teams with org charts and budgets, equip them with structured development workflows and 40+ execution tools, let them learn from experience, interact with them through a native desktop app or 16+ messaging platforms, and extend everything through a plugin ecosystem.

---

## Source Repositories

### From `/Users/bm/cod/trend/10-march`

| # | Repo | What It Is | Role in AgentForge |
|---|------|-----------|-------------------|
| 1 | **Paperclip** | Multi-agent orchestration control plane | **Core Engine** — org charts, budgets, tasks, governance, heartbeat protocol |
| 2 | **Open-Claude-Cowork** | Electron desktop GUI for Claude Code | **Desktop Client** — native app with session management, streaming UI, permission control |
| 3 | **OpenClaw** | Autonomous AI agent runtime | **Agent Runtime Adapter** — Node.js gateway, Docker isolation, webhook-based execution |
| 4 | **Claude-Code Plugins** | 13 official plugins for Claude Code | **Plugin Library** — code review, feature dev, commit automation, security hooks, PR review |

### From `/Users/bm/cod/trend/11-march`

| # | Repo | What It Is | Role in AgentForge |
|---|------|-----------|-------------------|
| 5 | **Agency-Agents** | 100+ specialized AI agent personas | **Persona Catalog** — engineering, design, marketing, product, testing, support, and more |
| 6 | **AstrBot** | Multi-platform chatbot framework | **Communication Hub** — 16+ messaging platforms, knowledge base, 1000+ plugins |
| 7 | **Hermes-Agent** | Self-improving multi-platform AI agent | **Execution Engine** — 40+ tools, learning loop, persistent memory, 6 terminal backends |
| 8 | **Superpowers** | Development workflow skills system | **Dev Methodology** — TDD, systematic debugging, design brainstorming, planning, code review |

---

## Product Requirements Extracted

### From Paperclip (Core Engine)

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

### From Open-Claude-Cowork (Desktop Client)

- **PR-11**: Native desktop application (Electron) with persistent sessions
- **PR-12**: Real-time token-by-token streaming with markdown rendering and syntax highlighting
- **PR-13**: Interactive tool permission control — explicit approve/deny for sensitive actions
- **PR-14**: Session management — create, resume, search, and delete conversation sessions
- **PR-15**: SQLite-backed local session history surviving app restarts
- **PR-16**: Skills management UI — list, import, install, uninstall skills
- **PR-17**: Workspace/working directory management per session
- **PR-18**: Reuse of Claude Code settings (~/.claude/settings.json) for zero-config startup

### From OpenClaw (Agent Runtime Adapter)

- **PR-19**: Autonomous AI agent execution with persistent workspace and identity
- **PR-20**: Node.js gateway server with token-based auth and REST API
- **PR-21**: Web dashboard (Control UI) for monitoring agent execution
- **PR-22**: Docker Sandbox support (microVM-based isolation) with Docker Compose fallback
- **PR-23**: Webhook-based agent invocation for external orchestration systems
- **PR-24**: Model fallback chains for reliability (primary + fallback models)
- **PR-25**: Device pairing and credential management in isolated directories

### From Claude-Code Plugins (Plugin Library)

- **PR-26**: Automated code review with parallel agents and confidence-based scoring (80+ threshold)
- **PR-27**: 7-phase guided feature development workflow (explore → design → implement → review)
- **PR-28**: Git workflow automation — commits, pushes, PR creation
- **PR-29**: PR review toolkit with 6 specialized review agents (comments, tests, errors, types, quality, simplification)
- **PR-30**: Security hook system monitoring 9 patterns (command injection, XSS, eval, dangerous HTML, etc.)
- **PR-31**: Plugin development toolkit with 7 expert skills and AI-assisted plugin creation
- **PR-32**: Self-referential iteration loops (agent works on same task repeatedly until completion)
- **PR-33**: Hookify rule engine — create custom behavior rules via markdown files
- **PR-34**: Event-driven hook system (PreToolUse, PostToolUse, Stop, SessionStart, etc.)

### From Agency-Agents (Persona Catalog)

- **PR-35**: 100+ battle-tested agent personas across 11 divisions
- **PR-36**: Each agent has defined personality, deliverables, success metrics, and step-by-step workflows
- **PR-37**: Division-based organizational structure (engineering, design, marketing, product, PM, testing, support, spatial computing, specialized, game dev, paid media)
- **PR-38**: Multi-tool format compatibility (Claude Code, Cursor, Codex, OpenCode, Antigravity, Gemini CLI)
- **PR-39**: Agent orchestration patterns for parallel and sequential multi-agent work
- **PR-40**: MCP memory integration for cross-session agent handoffs
- **PR-41**: Conversion and installation infrastructure for deploying personas to any tool

### From AstrBot (Communication Hub)

- **PR-42**: 16+ messaging platform integrations (Telegram, Slack, Discord, WeChat, QQ, Feishu, DingTalk, LINE, WhatsApp, Signal, etc.)
- **PR-43**: Knowledge base with semantic search (FAISS vector DB, hybrid sparse/dense retrieval)
- **PR-44**: 1000+ community plugin ecosystem with one-click install and auto-update
- **PR-45**: Agent sandbox for safe, isolated code execution
- **PR-46**: Multi-LLM provider support (30+ providers: OpenAI, Anthropic, Google, DeepSeek, Ollama, etc.)
- **PR-47**: Web dashboard with real-time streaming, session management, and admin controls
- **PR-48**: Multi-stage event processing pipeline (preprocessing, routing, response, decoration)
- **PR-49**: Internationalization (i18n) support
- **PR-50**: Speech-to-text and text-to-speech from 8+ providers

### From Hermes-Agent (Execution Engine)

- **PR-51**: Closed learning loop — agents create and improve skills autonomously from experience
- **PR-52**: 40+ built-in tools organized by category (web, terminal, files, browser, code execution, vision, planning, cron)
- **PR-53**: Persistent agent memory (MEMORY.md for learned knowledge, USER.md for user preferences)
- **PR-54**: 6 terminal backends for execution isolation (local, Docker, SSH, Modal, Daytona, Singularity)
- **PR-55**: Session persistence with FTS5 full-text search across all conversations
- **PR-56**: Subagent delegation — spawn child agents for parallel or isolated workstreams
- **PR-57**: Mixture-of-agents — multi-model collaborative reasoning
- **PR-58**: Prompt injection detection and security scanning on memory/context files
- **PR-59**: Batch trajectory generation and compression for fine-tuning
- **PR-60**: Multi-platform gateway (Telegram, Discord, Slack, WhatsApp, Signal, Home Assistant)
- **PR-61**: Cron scheduling with natural language descriptions

### From Superpowers (Dev Methodology)

- **PR-62**: Socratic design brainstorming with alternative exploration and spec document generation
- **PR-63**: Implementation planning — bite-sized tasks (2-5 min each) with complete code samples and exact file paths
- **PR-64**: Subagent-driven development — fresh subagent per task with two-stage review (spec compliance + code quality)
- **PR-65**: Strict TDD workflow — RED-GREEN-REFACTOR cycle with mandatory test-first verification
- **PR-66**: Systematic debugging — 4-phase root cause analysis (investigate → analyze → hypothesize → implement)
- **PR-67**: Git worktree management for isolated development branches
- **PR-68**: Code review skills with severity classification (Critical, Important, Minor)
- **PR-69**: Branch completion workflow with merge/PR/keep/discard options
- **PR-70**: Parallel agent dispatching for concurrent workstreams

---

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          AgentForge Clients                             │
│                                                                         │
│  ┌──────────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  Desktop App          │  │  Web Dashboard    │  │  Messaging       │  │
│  │  (Open-Claude-Cowork) │  │  (Paperclip +     │  │  (AstrBot 16+   │  │
│  │                        │  │   AstrBot WebUI)  │  │   platforms)     │  │
│  │  • Electron native     │  │  • React 19       │  │  • Telegram      │  │
│  │  • Streaming UI        │  │  • Org charts     │  │  • Slack         │  │
│  │  • Permission control  │  │  • Task boards    │  │  • Discord       │  │
│  │  • Session management  │  │  • Cost dashboard │  │  • WeChat        │  │
│  │  • Skills management   │  │  • Agent catalog  │  │  • 12+ more      │  │
│  └──────────┬─────────────┘  └────────┬─────────┘  └────────┬─────────┘  │
└─────────────┼──────────────────────────┼──────────────────────┼──────────┘
              │         REST + WebSocket + IPC                  │
┌─────────────┴──────────────────────────┴──────────────────────┴──────────┐
│                         Unified API Gateway                              │
│                                                                          │
│  ┌────────────────┐ ┌────────────────┐ ┌────────────────┐ ┌───────────┐ │
│  │  Paperclip      │ │  AstrBot       │ │  Hermes        │ │  OpenClaw │ │
│  │  Orchestrator   │ │  Msg Gateway   │ │  Agent Runtime │ │  Gateway  │ │
│  │  (Express.js)   │ │  (Quart)       │ │  (Python)      │ │  (Node)   │ │
│  │                  │ │                │ │                │ │           │ │
│  │  • Org charts    │ │  • 16+ IM      │ │  • 40+ tools   │ │  • Docker │ │
│  │  • Budgets       │ │  • Knowledge   │ │  • Learning    │ │  • Webhook│ │
│  │  • Tasks         │ │  • Plugins     │ │  • Memory      │ │  • Auth   │ │
│  │  • Governance    │ │  • Sandbox     │ │  • Subagents   │ │  • Models │ │
│  │  • Heartbeat     │ │  • i18n        │ │  • 6 backends  │ │  • Pair   │ │
│  │  • Adapters      │ │  • STT/TTS     │ │  • MoA         │ │           │ │
│  └────────┬─────────┘ └───────┬────────┘ └───────┬────────┘ └─────┬─────┘ │
└───────────┼────────────────────┼──────────────────┼────────────────┼──────┘
            │                    │                  │                │
┌───────────┴────────────────────┴──────────────────┴────────────────┴──────┐
│                       Agent Intelligence Layer                            │
│                                                                           │
│  ┌──────────────────────────┐  ┌──────────────────────────────────────┐  │
│  │  Persona Catalog          │  │  Development Methodology              │  │
│  │  (agency-agents)          │  │  (superpowers + claude-code plugins)  │  │
│  │                            │  │                                        │  │
│  │  100+ personas:            │  │  Skills:                               │  │
│  │  • Engineering (16)        │  │  • TDD (RED-GREEN-REFACTOR)            │  │
│  │  • Design (8)              │  │  • Systematic Debugging (4-phase)      │  │
│  │  • Marketing (17)          │  │  • Design Brainstorming (Socratic)     │  │
│  │  • Product / PM (10)       │  │  • Implementation Planning             │  │
│  │  • Testing (8)             │  │  • Subagent-Driven Development         │  │
│  │  • Support (6)             │  │  • Git Worktree Management             │  │
│  │  • Spatial Computing (6)   │  │                                        │  │
│  │  • Specialized (15+)       │  │  Plugins:                              │  │
│  │  • Game Dev (5+)           │  │  • Code Review (confidence scoring)    │  │
│  │  • Paid Media (7)          │  │  • Feature Dev (7-phase workflow)      │  │
│  │                            │  │  • PR Review (6 specialist agents)     │  │
│  │  Each with:                │  │  • Security Hooks (9 patterns)         │  │
│  │  • Personality & voice     │  │  • Hookify Rule Engine                 │  │
│  │  • Deliverables            │  │  • Commit Automation                   │  │
│  │  • Success metrics         │  │  • Iterative Loop (Ralph Wiggum)       │  │
│  │  • Step-by-step workflows  │  │  • Plugin Dev Toolkit                  │  │
│  └──────────────────────────┘  └──────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────┘
            │                    │                  │                │
┌───────────┴────────────────────┴──────────────────┴────────────────┴──────┐
│                          Shared Data Layer                                │
│                                                                           │
│  PostgreSQL             SQLite              FAISS           Markdown       │
│  (Paperclip DB)         (Sessions)          (KB Vectors)   (Agent State)  │
│  • companies            • messages          • embeddings   • MEMORY.md    │
│  • agents               • sessions_fts      • doc chunks   • USER.md     │
│  • issues               • agent memory      • search idx   • Personas     │
│  • cost_events          • OCC sessions      │              • Skills       │
│  • heartbeat_runs       │                   │              • Specs/Plans  │
│  • approvals            │                   │              │              │
│  • agent_templates      │                   │              │              │
│  • 35+ tables           │                   │              │              │
└───────────────────────────────────────────────────────────────────────────┘
```

### Integration Map — How All 8 Connect

```
                        ┌──────────────┐
                        │  PAPERCLIP   │ ← Central orchestrator
                        │  (Core)      │
                        └──────┬───────┘
               ┌───────────────┼───────────────┬──────────────────┐
               ↓               ↓               ↓                  ↓
     ┌─────────────┐  ┌──────────────┐  ┌────────────┐  ┌──────────────┐
     │AGENCY-AGENTS│  │ HERMES-AGENT │  │  OPENCLAW   │  │   ASTRBOT    │
     │(Personas)   │  │ (Execution)  │  │ (Runtime)   │  │ (Messaging)  │
     └──────┬──────┘  └──────┬───────┘  └─────┬──────┘  └──────┬───────┘
            │                │                 │                 │
            │         ┌──────┴───────┐         │                 │
            │         ↓              ↓         │                 │
            │  ┌────────────┐ ┌───────────┐    │                 │
            │  │SUPERPOWERS │ │CL-CODE    │    │                 │
            │  │(Dev Skills)│ │PLUGINS    │    │                 │
            │  └────────────┘ └───────────┘    │                 │
            │                                  │                 │
            └──────────────────────────────────┘                 │
                              ↓                                  │
                    ┌──────────────────┐                          │
                    │ OPEN-CLAUDE-     │ ←───────────────────────┘
                    │ COWORK (Desktop) │
                    └──────────────────┘
```

### Integration Points (Detailed)

#### 1. Paperclip ← Agency-Agents (Persona Catalog)

Agency-agents personas become first-class **agent templates** within Paperclip's hiring flow.

```
agency-agents/engineering/frontend-developer.md
    ↓ import & parse
Paperclip DB: agent_templates table (persona, division, deliverables, metrics)
    ↓ user selects "Hire"
Paperclip DB: agents table (with persona injected into adapter config)
    ↓ org chart placement
Paperclip: department + reporting chain + task queue
```

- 11 divisions map to Paperclip org chart departments
- Persona workflows become task template suggestions
- Success metrics feed into Paperclip's performance tracking
- Multi-tool format support ensures personas work with any adapter (Hermes, OpenClaw, Claude)

#### 2. Paperclip ← Hermes-Agent (Primary Execution Adapter)

Hermes becomes Paperclip's most capable adapter (`adapter-hermes`).

```
Paperclip heartbeat fires
    ↓
adapter-hermes receives wakeup payload
    ↓
Hermes AIAgent.run() with:
  - persona from agency-agents (system prompt)
  - superpowers skills injected (dev methodology)
  - task from Paperclip (user prompt)
  - tools filtered by agent role
  - memory from previous runs
    ↓
Hermes executes with 40+ tools, learns from experience
    ↓
Results posted back to Paperclip via API (issue comments + audit log)
    ↓
Hermes MEMORY.md synced to Paperclip agent_runtime_state
```

- 6 terminal backends provide execution isolation options
- Learning loop feeds back into agent capability profiles
- Subagent delegation maps to Paperclip child tasks
- FTS5 session search feeds Paperclip's audit trail
- Mixture-of-agents enables collaborative multi-model reasoning per task

#### 3. Paperclip ← OpenClaw (Lightweight Runtime Adapter)

OpenClaw serves as a complementary adapter for simpler, webhook-driven workloads.

```
Paperclip heartbeat fires
    ↓
adapter-openclaw sends webhook to OpenClaw gateway
    ↓
OpenClaw agent executes with model fallback chain
    ↓
Results returned via webhook callback
    ↓
Paperclip records execution + costs
```

- Docker Sandbox provides microVM isolation for untrusted workloads
- Model fallback chains ensure reliability
- Lightweight alternative when Hermes' full tool suite isn't needed
- Gateway's Control UI provides per-agent monitoring

#### 4. Paperclip ← AstrBot (Communication Layer)

AstrBot becomes the messaging facade for the entire platform.

```
User sends message on Telegram/Slack/Discord/WeChat
    ↓
AstrBot platform adapter receives message
    ↓
AgentForge intent router:
  - "@agent-name do X" → route to specific Paperclip agent
  - "create task: X" → create Paperclip issue
  - "status?" → fetch from Paperclip API
  - "what do we know about X?" → AstrBot KB search
    ↓
Response sent back through originating platform
```

- 16+ platform adapters provide universal reach
- Knowledge base provides shared organizational context
- 1000+ community plugins extend capabilities
- STT/TTS enables voice-based agent interaction
- Event pipeline preprocesses/routes messages intelligently

#### 5. Hermes ← Superpowers (Dev Methodology Injection)

Superpowers skills are injected into Hermes' system prompt for engineering agents.

```
Paperclip assigns coding task to engineering agent
    ↓
adapter-hermes loads agent persona (e.g., "Backend Architect")
    ↓
Superpowers skills injected based on task type:
  - New feature → brainstorming + writing-plans + subagent-driven-development
  - Bug fix → systematic-debugging + test-driven-development
  - Code review → requesting-code-review
    ↓
Hermes executes with structured methodology
    ↓
Agent follows TDD cycle, creates plans, reviews own work
```

- Task type detection maps to appropriate skill set
- TDD ensures agents write tests before code
- Systematic debugging prevents random fix attempts
- Brainstorming produces design specs before implementation
- Git worktree management keeps work isolated

#### 6. Hermes ← Claude-Code Plugins (Enhanced Capabilities)

Claude-code plugins provide specialized development tools to Hermes-powered agents.

```
Engineering agent needs to review a PR
    ↓
Hermes loads PR Review Toolkit plugin skills
    ↓
6 parallel review agents analyze:
  - PR comments, tests, error handling, types, quality, simplification
    ↓
Results aggregated with confidence scores (80+ threshold)
    ↓
Consolidated review posted as Paperclip issue comment
```

- Code review plugin adds confidence-based quality gates
- Feature dev plugin provides structured 7-phase development
- Security hooks monitor for OWASP vulnerabilities in generated code
- Hookify allows custom rules per project/company
- Commit automation handles git workflow

#### 7. Open-Claude-Cowork ← Paperclip + AstrBot (Desktop Client)

The Electron app becomes the native desktop interface for AgentForge.

```
User opens AgentForge desktop app
    ↓
Open-Claude-Cowork UI shows:
  - Left: Session list (from Paperclip companies/agents)
  - Center: Chat with streaming (connected to Hermes/OpenClaw runtime)
  - Right: Org chart, tasks, costs (from Paperclip API)
    ↓
IPC bridge connects to:
  - Paperclip API (org/tasks/budgets)
  - AstrBot KB (knowledge search)
  - Hermes runtime (direct agent interaction)
```

- Existing Zustand state management extended for Paperclip data
- Permission control applies to all agent tool executions
- Session history includes full audit trail from Paperclip
- Skills management UI manages both Superpowers + Claude-Code plugins

#### 8. AstrBot ↔ Hermes (Cross-Platform Delivery)

```
Hermes cron job completes a scheduled report
    ↓
Output routed through AstrBot to user's preferred platform (Slack)
    ↓
User responds on Slack with feedback
    ↓
AstrBot routes response back to Hermes session
    ↓
Hermes' learning loop incorporates feedback into MEMORY.md
```

- Shared LLM provider configuration (30+ providers, one config)
- Both use SQLite for sessions — unified into single store
- AstrBot sandbox complements Hermes terminal backends
- Hermes web search tools enhance AstrBot knowledge base with live data

---

## Tech Stack

| Layer | Technology | Source Repos |
|-------|-----------|-------------|
| Desktop Client | Electron 39, React 19, Zustand 5, better-sqlite3 | Open-Claude-Cowork |
| Web Dashboard | React 19, Tailwind CSS 4, Radix UI, TanStack Query | Paperclip |
| API Server | Express.js 5 (primary), Quart bridge (messaging) | Paperclip + AstrBot |
| Database | PostgreSQL (Drizzle ORM) | Paperclip |
| Session Store | SQLite (WAL mode, FTS5) | Hermes + Open-Claude-Cowork |
| Vector DB | FAISS-CPU | AstrBot |
| Agent Runtime (Primary) | Python 3.12+, asyncio | Hermes |
| Agent Runtime (Secondary) | Node.js 22+, Docker Sandbox | OpenClaw |
| Messaging | Platform-specific SDKs (16+) | AstrBot + Hermes Gateway |
| LLM Providers | OpenAI, Anthropic, Google, DeepSeek, Ollama, 25+ more | AstrBot + Hermes |
| Agent Personas | Markdown files (100+) | Agency-Agents |
| Dev Skills | Markdown skill definitions (14 skills) | Superpowers |
| Dev Plugins | Markdown + Python + JSON (13 plugins) | Claude-Code Plugins |
| Auth | Better-auth (JWT) | Paperclip |
| Real-time | WebSocket (ws) + IPC (Electron) | Paperclip + Open-Claude-Cowork |
| Security | Hook system (9 patterns) + prompt injection detection | Claude-Code Plugins + Hermes |
| Audio | Whisper, Edge TTS, ElevenLabs, Azure TTS | AstrBot + Hermes |
| Deployment | Docker Compose, embedded Postgres | Paperclip |
| Package Mgmt | pnpm workspaces (monorepo) | Paperclip |
| Testing | Vitest + Playwright (E2E) | Paperclip |

---

## Implementation Plan

### Phase 1 — Core Foundation: Paperclip + Agency-Agents

**Goal**: Orchestration platform with agent persona catalog.

| Task | Description | Effort |
|------|-------------|--------|
| 1.1 | Create AgentForge monorepo, import Paperclip codebase as core package | S |
| 1.2 | Import agency-agents as `packages/personas` | S |
| 1.3 | Build persona importer — parse 100+ agent markdown files into `agent_templates` DB table | M |
| 1.4 | Build "Hire from Catalog" UI — browse by division, search, preview persona details | M |
| 1.5 | Map agency-agents divisions to Paperclip org chart departments | S |
| 1.6 | Auto-populate agent config (instructions, deliverables, metrics) from persona on hire | M |
| 1.7 | Add persona-based task template suggestions when creating issues | M |

**Deliverable**: Browse 100+ agent personas, hire into org charts, manage tasks with budgets.

---

### Phase 2 — Execution: + Hermes-Agent + OpenClaw

**Goal**: Agents execute real work with tools and isolated runtimes.

| Task | Description | Effort |
|------|-------------|--------|
| 2.1 | Import hermes-agent as `packages/runtime-hermes` | S |
| 2.2 | Build `adapter-hermes` Paperclip adapter package | L |
| 2.3 | Implement heartbeat → Hermes wakeup bridge | M |
| 2.4 | Inject agency-agents persona into Hermes system prompt at runtime | M |
| 2.5 | Map Hermes tool results back to Paperclip issue comments + audit log | M |
| 2.6 | Sync Hermes MEMORY.md with Paperclip `agent_runtime_state` | M |
| 2.7 | Import openclaw adapter from Paperclip's existing `adapter-openclaw-gateway` | S |
| 2.8 | Expose terminal backend selection (local/Docker/SSH/Modal/Daytona) in agent config UI | M |
| 2.9 | Wire Hermes skill creation events into agent capability tracking | M |
| 2.10 | Connect Hermes subagent delegation to Paperclip child task creation | L |
| 2.11 | Configure OpenClaw Docker Sandbox for untrusted workload isolation | M |

**Deliverable**: Agents execute work with 40+ tools (Hermes) or webhook-based isolation (OpenClaw), learn from experience, report back.

---

### Phase 3 — Dev Methodology: + Superpowers + Claude-Code Plugins

**Goal**: Engineering agents follow structured development workflows.

| Task | Description | Effort |
|------|-------------|--------|
| 3.1 | Import superpowers as `packages/skills-superpowers` | S |
| 3.2 | Import claude-code plugins as `packages/plugins-claude-code` | S |
| 3.3 | Build task-type → skill mapping engine (new feature → brainstorm+plan+TDD, bug → debug+TDD, review → code-review) | M |
| 3.4 | Inject superpowers skills into Hermes system prompt based on task type | M |
| 3.5 | Integrate code review plugin — run 6 parallel review agents on PRs, post results to Paperclip | L |
| 3.6 | Integrate security hooks — scan all agent-generated code for OWASP vulnerabilities | M |
| 3.7 | Wire feature dev plugin's 7-phase workflow into Paperclip task state transitions | M |
| 3.8 | Enable hookify rules per company — custom behavior constraints per org | M |
| 3.9 | Add "Dev Methodology" toggle in agent config (TDD-strict, TDD-lite, none) | S |
| 3.10 | Connect git worktree management to Paperclip's workspace system | M |

**Deliverable**: Engineering agents follow TDD, produce design specs before coding, self-review, and pass security scans.

---

### Phase 4 — Communication: + AstrBot

**Goal**: Interact with the AI workforce from any messaging platform.

| Task | Description | Effort |
|------|-------------|--------|
| 4.1 | Import astrbot as `packages/messaging-astrbot` | S |
| 4.2 | Build AstrBot ↔ Paperclip API bridge (message routing) | L |
| 4.3 | Create intent router (direct agent chat, task creation, status query, KB search) | M |
| 4.4 | Connect AstrBot knowledge base to Paperclip project context | M |
| 4.5 | Route Hermes cron outputs through AstrBot to user's preferred platform | M |
| 4.6 | Unify LLM provider configuration (single config for all subsystems) | M |
| 4.7 | Enable AstrBot's 1000+ community plugins as optional agent capabilities | M |
| 4.8 | Integrate STT/TTS for voice-based agent interaction | M |
| 4.9 | Add platform preference settings per user (where to receive notifications) | S |

**Deliverable**: Manage AI workforce from Slack, Telegram, Discord, WeChat, or any of 16+ platforms. Voice-enabled.

---

### Phase 5 — Desktop Client: + Open-Claude-Cowork

**Goal**: Native desktop application for power users.

| Task | Description | Effort |
|------|-------------|--------|
| 5.1 | Import open-claude-cowork as `packages/desktop` | S |
| 5.2 | Replace direct Claude SDK calls with AgentForge API (Paperclip + Hermes) | L |
| 5.3 | Add org chart sidebar panel (from Paperclip data) | M |
| 5.4 | Add task board view alongside chat interface | M |
| 5.5 | Add cost dashboard widget | M |
| 5.6 | Connect skills management UI to Superpowers + Claude-Code plugins | M |
| 5.7 | Add knowledge base search panel (from AstrBot KB) | M |
| 5.8 | Integrate permission control with Paperclip's approval gates | M |
| 5.9 | Add multi-company switcher in app header | S |

**Deliverable**: Native desktop app with full AgentForge functionality — chat, org charts, tasks, costs, KB search, skills.

---

### Phase 6 — Polish & Production

**Goal**: Production-ready, one-command deployment.

| Task | Description | Effort |
|------|-------------|--------|
| 6.1 | Unified authentication across all 8 subsystems | L |
| 6.2 | Single Docker Compose deployment with all services | M |
| 6.3 | Combined plugin/skill marketplace UI (Superpowers + Claude-Code + AstrBot plugins) | L |
| 6.4 | Mobile-optimized responsive web dashboard | M |
| 6.5 | Comprehensive E2E test suite | L |
| 6.6 | Documentation site with quickstart, guides, API reference | L |
| 6.7 | Company template gallery (pre-built orgs: startup, agency, dev team, support center) | M |
| 6.8 | Electron app auto-updater and platform builds (macOS, Windows, Linux) | M |
| 6.9 | Performance optimization and caching layer | M |
| 6.10 | Batch trajectory export for fine-tuning custom models (from Hermes) | M |

**Deliverable**: `docker compose up` or native desktop installer — complete AI workforce platform.

---

## Monorepo Structure

```
agentforge/
├── packages/
│   ├── core/                          # Paperclip (orchestration engine)
│   │   ├── server/                    # Express.js API server
│   │   ├── ui/                        # React web dashboard
│   │   └── packages/
│   │       ├── db/                    # PostgreSQL schema (Drizzle)
│   │       ├── shared/                # Shared types
│   │       └── adapters/
│   │           ├── hermes/            # adapter-hermes (new)
│   │           ├── openclaw-gateway/  # adapter-openclaw (existing)
│   │           ├── claude-local/      # adapter-claude (existing)
│   │           └── adapter-utils/     # Shared adapter utilities
│   │
│   ├── runtime-hermes/                # Hermes-Agent (execution engine)
│   │   ├── hermes_agent/              # Core agent + 40+ tools
│   │   ├── gateway/                   # Multi-platform messaging
│   │   └── skills/                    # Hermes skills directory
│   │
│   ├── runtime-openclaw/              # OpenClaw (lightweight runtime)
│   │   └── gateway/                   # Node.js gateway + Docker
│   │
│   ├── personas/                      # Agency-Agents (100+ personas)
│   │   ├── engineering/
│   │   ├── design/
│   │   ├── marketing/
│   │   ├── product/
│   │   ├── project-management/
│   │   ├── testing/
│   │   ├── support/
│   │   ├── spatial-computing/
│   │   ├── specialized/
│   │   ├── game-development/
│   │   └── paid-media/
│   │
│   ├── skills-superpowers/            # Superpowers (dev methodology)
│   │   └── skills/
│   │       ├── brainstorming/
│   │       ├── writing-plans/
│   │       ├── test-driven-development/
│   │       ├── systematic-debugging/
│   │       ├── subagent-driven-development/
│   │       └── ...
│   │
│   ├── plugins-claude-code/           # Claude-Code Plugins (dev tools)
│   │   └── plugins/
│   │       ├── code-review/
│   │       ├── feature-dev/
│   │       ├── pr-review-toolkit/
│   │       ├── security-guidance/
│   │       ├── hookify/
│   │       └── ...
│   │
│   ├── messaging-astrbot/            # AstrBot (communication hub)
│   │   ├── astrbot/                   # Core Python framework
│   │   ├── dashboard/                 # Vue 3 admin panel
│   │   └── plugins/                   # Plugin ecosystem
│   │
│   └── desktop/                       # Open-Claude-Cowork (Electron app)
│       ├── src/electron/              # Main process
│       └── src/ui/                    # React renderer
│
├── docker/
│   ├── docker-compose.yml             # Full platform deployment
│   ├── Dockerfile.core                # Paperclip server
│   ├── Dockerfile.hermes              # Hermes runtime
│   ├── Dockerfile.openclaw            # OpenClaw gateway
│   └── Dockerfile.astrbot             # AstrBot messaging
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

## User Workflows

### Workflow 1: "Launch an AI Startup"

```
1. User creates a new company in AgentForge (web or desktop)
2. Browses agent catalog (agency-agents) → hires:
   - CEO (from specialized/agents-orchestrator)
   - Backend Architect (from engineering/)
   - Frontend Developer (from engineering/)
   - Growth Hacker (from marketing/)
   - UI Designer (from design/)
3. Arranges org chart: CEO manages all others
4. Sets monthly budget: $500
5. Creates company goal: "Build and launch an MVP SaaS product"
6. CEO agent wakes up (Hermes runtime), breaks goal into tasks, delegates
7. Engineers execute via Hermes (Docker backend) with Superpowers TDD workflow:
   - Brainstorm → design spec → plan → RED test → GREEN code → REFACTOR
   - Code reviewed by Claude-Code review plugin (6 parallel agents, 80+ confidence)
   - Security hooks scan for OWASP vulnerabilities
8. Designer produces mockups, Growth Hacker plans GTM strategy
9. User monitors from Slack (via AstrBot) or desktop app (Open-Claude-Cowork)
10. All costs tracked, all work audited, all agents learning
```

### Workflow 2: "AI Customer Support Team"

```
1. User creates company, hires:
   - Support Responder (from support/)
   - Analytics Reporter (from support/)
2. Connects Telegram + Discord + Slack via AstrBot
3. Uploads product docs to AstrBot knowledge base (FAISS semantic search)
4. Customers message on any platform → AstrBot routes to Support Responder
5. Agent searches KB, responds with context-aware answers
6. Analytics Reporter runs weekly via Hermes cron → report sent to Slack
7. Agents learn from interactions (Hermes MEMORY.md), improve over time
8. User reviews from desktop app, adjusts personas as needed
```

### Workflow 3: "Development Agency"

```
1. User creates company, hires full engineering team:
   - Senior Developer (from engineering/)
   - Frontend Developer (from engineering/)
   - Backend Architect (from engineering/)
   - UI Designer (from design/)
   - Reality Checker (from testing/)
   - Project Shepherd (from project-management/)
2. Project Shepherd breaks client requirements into tasks
3. Engineering agents execute with Superpowers methodology:
   - Design brainstorming → Socratic refinement → spec document
   - Implementation plan → bite-sized tasks with code samples
   - Subagent-driven development → fresh context per task
   - TDD cycle → all code has tests before merge
   - Systematic debugging → 4-phase root cause analysis for bugs
4. Code review plugin runs on every PR (6 parallel agents)
5. Security hooks block any code with XSS, injection, or eval patterns
6. Git worktree management keeps all work on isolated branches
7. Client gets updates via Telegram, approves via desktop app
```

### Workflow 4: "Content Marketing Machine"

```
1. User creates company, hires:
   - Content Creator (from marketing/)
   - Twitter Strategist (from marketing/)
   - Reddit Community Builder (from marketing/)
   - SEO Specialist (from marketing/)
2. Sets weekly content calendar as recurring Hermes cron tasks
3. Content Creator writes articles using Hermes web search + browser tools
4. Platform strategists adapt content for each channel
5. User reviews via WeChat (AstrBot), approves before publishing
6. All costs tracked per agent, per task
7. Knowledge base grows with published content for future reference
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Time to first working agent | < 5 minutes |
| Agent personas available at launch | 100+ |
| Messaging platforms supported | 16+ |
| Tools available per agent | 40+ |
| Development skills/plugins | 14 skills + 13 plugins |
| LLM providers supported | 30+ |
| Deployment complexity | Single `docker compose up` |
| Desktop platforms | macOS, Windows, Linux |
| Cost tracking granularity | Per-token, per-agent, per-task |
| Security patterns monitored | 9 OWASP categories |

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Mixed tech stack (Node.js + Python + Electron) | Deployment complexity | Docker Compose with per-service containers; desktop app bundles its own runtime |
| Frontend merge (React + Vue) | Development effort | Paperclip React as primary; AstrBot Vue dashboard wrapped or progressively migrated |
| Session state across systems | Data consistency | PostgreSQL as single source of truth; SQLite/FAISS as read-optimized local caches |
| 100+ persona maintenance | Stale agents | Community contribution model (agency-agents is MIT + community-driven) |
| LLM cost runaway | Budget overruns | Paperclip's built-in budget enforcement with automatic agent pausing |
| Electron app size | Large download | Lazy-load heavy features; split core vs. full install |
| Adapter complexity (Hermes + OpenClaw) | Integration bugs | Shared adapter-utils package; comprehensive integration tests |
| Skill/plugin conflicts | Unexpected behavior | Namespace isolation; priority ordering; hookify rules as guardrails |

---

## License

All eight source repositories are open-source. AgentForge will be released under MIT license.

---

*This proposal was generated on 2026-03-11.*
*Sources: 10-march (Paperclip, Open-Claude-Cowork, OpenClaw, Claude-Code Plugins) + 11-march (Agency-Agents, AstrBot, Hermes-Agent, Superpowers)*
