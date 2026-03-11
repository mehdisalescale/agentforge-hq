# Consolidation Status: 8 Repos into Forge

> **Date:** 2026-03-11
> **Context:** Tracking progress of absorbing features from 8 external repos into forge-project.
> **Companion doc:** `EXPANSION_PLAN.md` (wave-by-wave implementation plan)

---

## Source Repos at a Glance

| Repo | Tech | Primary Value | Size |
|------|------|---------------|------|
| **agency-agents** | Markdown | 130+ agent personas, NEXUS orchestration framework, 11 divisions | 3.1 MB |
| **AstrBot** | Python | 15+ messaging platforms, knowledge base (RAG/FAISS), plugin system, pipeline stages | ~15K LOC |
| **claude-code** | TypeScript | 13 official plugins, hook system (Pre/PostToolUse), MCP integration, plugin marketplace | v2.1.72 |
| **hermes-agent** | Python | 40+ tools, self-improving skills, persistent memory, messaging gateway, browser automation | ~15K LOC |
| **Open-Claude-Cowork** | Electron/React | Desktop GUI, token streaming, tool permission UI | v0.1.0 |
| **openclaw** | TypeScript | Multi-channel gateway (Telegram/Discord/Slack/Signal/iMessage), device pairing | v2026.3.9 |
| **paperclip** | TypeScript | Multi-company orchestration, org charts, budgets, heartbeats, task checkout, governance | v0.3.0 |
| **superpowers** | Markdown | 14 dev workflow skills (brainstorm, TDD, debug, review, merge), multi-platform plugin | - |

---

## Feature Consolidation Matrix

### Legend
- **DONE** = Implemented and working in Forge
- **PARTIAL** = Code exists but incomplete (stub, no data, missing service layer)
- **PLANNED** = In EXPANSION_PLAN.md but not started
- **NOT PLANNED** = Not yet in any plan, worth considering

### Core Infrastructure

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Event-driven architecture | All | **DONE** | 35 ForgeEvent variants, EventBus broadcast, append-only log |
| SQLite WAL persistence | hermes, AstrBot | **DONE** | 10 migrations, 17 repos, BatchWriter |
| Middleware pipeline | AstrBot (onion model) | **DONE** | 8-layer chain: RateLimit -> CircuitBreaker -> CostCheck -> SkillInjection -> Persist -> Spawn -> ExitGate -> QualityGate |
| Circuit breaker | - | **DONE** | 3-state FSM, tested |
| Rate limiter | - | **DONE** | Token bucket, tested |
| Cost tracking | paperclip | **DONE** | Per-session, per-day analytics |
| WebSocket streaming | - | **DONE** | Real-time event feed |
| Git worktree isolation | claude-code | **DONE** | forge-git crate, 7 tests |
| Cron scheduling | hermes, AstrBot | **DONE** | 60s tick, auto-spawn agents |
| OpenAPI docs | - | **DONE** | utoipa + utoipa-scalar |

### Agent System

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Agent presets (10) | - | **DONE** | CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer, Coordinator |
| Persona catalog (130+) | agency-agents | **PARTIAL** | `forge-persona` crate exists with parser/catalog/mapper. **But 0 persona files imported.** Parser is ready, no data. |
| Persona divisions (11) | agency-agents | **PARTIAL** | Division whitelist in parser matches agency-agents. DB schema ready (migration 0009). No data loaded. |
| NEXUS orchestration framework | agency-agents | **NOT PLANNED** | 7-phase pipeline (Discovery -> Strategize -> Scaffold -> Build -> Harden -> Launch -> Operate). Not in expansion plan. |
| Self-improving skill creation | hermes-agent | **NOT PLANNED** | Agent auto-creates skills after 5+ tool calls. Very valuable, not in any plan yet. |
| Context compression | hermes-agent | **DONE** | Context pruner in forge-process |
| Loop detection | hermes-agent | **DONE** | Sliding-window hash dedup, exit gates |
| Best-of-N selection | - | **DONE** | Multiple paths, scoring |
| Multi-LLM provider support | AstrBot, hermes | **NOT PLANNED** | AstrBot: 28+ LLM providers. Hermes: OpenRouter, Anthropic, etc. Forge: Claude CLI only. |
| Agent domains/grouping | claude-code, paperclip | **PARTIAL** | Departments exist in forge-org. No task routing by domain yet. |

### Skills & Knowledge

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Basic skills (10) | - | **DONE** | architect, code-review, debug, deep-research, document, explore, fix-bug, refactor, security-audit, test-writer |
| Dev workflow skills (14) | superpowers | **PLANNED** | Wave 2: brainstorming, TDD, debugging, code review, etc. Not imported yet. |
| Claude-code plugin skills | claude-code | **PLANNED** | Wave 2: code-review, feature-dev, PR review, security. Not imported yet. |
| Skill auto-activation | claude-code (hooks) | **PARTIAL** | skill_rules table exists (migration 0008). No activation logic wired. |
| Skill FTS search | hermes, AstrBot | **DONE** | FTS5 virtual table on skills |
| Knowledge base (RAG) | AstrBot | **PLANNED** | Wave 6: `forge-knowledge` crate. Not started. |
| Document parsing (PDF/MD) | AstrBot | **PLANNED** | Wave 6. Not started. |
| Embedding/vector search | AstrBot | **PLANNED** | Wave 6: start with FTS5, add embeddings later. Not started. |
| Memory system | hermes (MEMORY.md) | **DONE** | Facts with confidence scores, 3 memory types, FTS search |
| Session FTS search | hermes | **DONE** | sessions_fts virtual table |

### Organization & Governance

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Companies (multi-tenant) | paperclip | **DONE** | CRUD, budget_limit, budget_used, mission |
| Departments | paperclip | **DONE** | company_id FK, name, description |
| Org positions | paperclip | **DONE** | agent_id, reports_to hierarchy, roles |
| Org chart tree view | paperclip | **DONE** | build_org_chart() service, frontend page |
| Goals hierarchy | paperclip | **PARTIAL** | Goal model exists (forge-governance). No service layer. |
| Approval gates | paperclip | **PARTIAL** | Approval model exists (forge-governance). No workflow logic. |
| Budget enforcement (per-company) | paperclip | **PARTIAL** | DB columns exist. Not wired into middleware CostCheck. |
| Heartbeat/wake-up system | paperclip | **NOT PLANNED** | Scheduled agent wake-ups. Forge has cron schedules but not heartbeat semantics. |
| Task checkout semantics | paperclip | **NOT PLANNED** | Ticket-based task assignment with checkout/completion. Not in any plan. |
| Activity audit trail | paperclip | **DONE** | Append-only events table |

### Communication & Messaging

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Telegram integration | hermes, openclaw, AstrBot | **PLANNED** | Wave 7. Not started. |
| Discord integration | hermes, openclaw, AstrBot | **PLANNED** | Wave 7. Not started. |
| Slack integration | hermes, openclaw, AstrBot | **PLANNED** | Wave 7. Not started. |
| Signal integration | openclaw | **PLANNED** | Wave 7. Not started. |
| iMessage integration | openclaw | **NOT PLANNED** | OpenClaw has it. Not in expansion plan. |
| WhatsApp integration | hermes, openclaw | **NOT PLANNED** | Both have it. Not in expansion plan. |
| Device pairing | openclaw | **NOT PLANNED** | Auth/pairing challenges. Not in any plan. |
| TTS/STT | AstrBot, hermes | **NOT PLANNED** | AstrBot: 7+ TTS providers. Hermes: Edge TTS. Not in any plan. |
| Notification routing | openclaw | **PLANNED** | Wave 7: ForgeEvent -> messaging platform. Not started. |

### Execution Backends

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Claude CLI spawn | - | **DONE** | Default backend, stream-json parsing |
| Hermes runtime adapter | hermes-agent | **PLANNED** | Wave 4: `forge-adapter-hermes`. Not started. |
| OpenClaw webhook adapter | openclaw | **PLANNED** | Wave 5: `forge-adapter-openclaw`. Not started. |
| Backend routing | paperclip (adapters) | **NOT PLANNED** | ProcessBackend enum (Claude/Hermes/OpenClaw). In expansion plan but not implemented. |
| Browser automation | hermes (Browserbase), openclaw (Playwright) | **NOT PLANNED** | Neither adapter nor native support. |
| Terminal backends (Docker/SSH/Modal) | hermes-agent | **NOT PLANNED** | 6 backends in Hermes. Not in any plan. |
| MCP server | - | **DONE** | forge-mcp-bin with rmcp v0.17, 10 tools |
| MCP client | hermes-agent | **NOT PLANNED** | Hermes has MCP client. Forge is server-only. |

### Hooks & Automation

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Event-triggered hooks | claude-code | **DONE** | hooks table, CRUD API, frontend page |
| Pre/Post timing | claude-code | **DONE** | Timing field on hooks |
| External script execution | claude-code | **DONE** | Shell command on trigger |
| Hook event filtering | claude-code | **PARTIAL** | 35 event types selectable. No pattern matching like claude-code's PreToolUse matchers. |
| Security pattern scanning | claude-code (security-guidance) | **PLANNED** | Wave 2: 9 OWASP patterns. Not implemented. |
| Plugin marketplace | claude-code | **NOT PLANNED** | Hot-installable plugins. Not in any plan. |
| Plugin hot-reload | AstrBot | **NOT PLANNED** | watchfiles-based reload. Not in any plan. |

### Desktop & UI

| Feature | Source Repo(s) | Forge Status | Details |
|---------|---------------|--------------|---------|
| Web dashboard (12 pages) | - | **DONE** | SvelteKit 5 + TailwindCSS 4, embedded in binary |
| Desktop GUI | Open-Claude-Cowork | **PLANNED** | Wave 8: Fork + rewire to Forge API. Not started. |
| Token streaming visualization | Open-Claude-Cowork | **DONE** | WebSocket streaming in dashboard |
| Tool permission UI | Open-Claude-Cowork | **NOT PLANNED** | Per-tool allow/deny. Not in any plan. |
| Swim-lane dashboard | - | **PLANNED** | v0.6.0 planned feature. Not implemented. |
| Pipeline builder UI | - | **PLANNED** | v0.6.0 planned feature. Not implemented. |

---

## Inconsistencies & Issues Found

### 1. Persona Crate Ready, Zero Data
**Severity: HIGH**
- `forge-persona` has a complete parser, catalog, and mapper
- Division whitelist matches agency-agents' 11 divisions
- DB migration 0009 creates personas and persona_divisions tables
- **But no persona .md files exist anywhere in the project**
- 130+ ready-to-import files sitting in `../agency-agents/`
- Fix: Run persona import from agency-agents directory

### 2. Missing Migration 0010
**Severity: MEDIUM**
- Migrations jump from 0009_personas.sql to 0011_org_charts.sql
- Migration 0010 was likely `skill_task_routing.sql` per EXPANSION_PLAN.md
- Either it was skipped or accidentally deleted
- Fix: Either add 0010 or renumber 0011

### 3. Governance Models Without Service Layer
**Severity: MEDIUM**
- `forge-governance` has Goal and Approval data models
- No service functions (create, approve, reject, list goals)
- No API routes for goals/approvals (forge-api doesn't wire governance)
- Frontend has no governance pages
- Fix: Add GovernanceService, API routes, frontend pages

### 4. Company Budget Not Wired to Middleware
**Severity: MEDIUM**
- Companies have budget_limit and budget_used columns
- CostCheck middleware only checks global budget (FORGE_BUDGET_WARN/LIMIT)
- Per-company budget enforcement is not connected
- Fix: Extend CostCheck middleware to query company budget before spawn

### 5. Skill Rules Table Exists, No Activation Logic
**Severity: LOW**
- Migration 0008 creates skill_rules table
- No code reads from skill_rules to auto-activate skills
- SkillInjection middleware exists but skill matching logic is basic
- Fix: Implement skill-rules-based activation in SkillInjection middleware

### 6. forge-mcp (old) vs forge-mcp-bin
**Severity: LOW**
- forge-mcp crate exists with JSON-RPC stubs
- forge-mcp-bin is the real implementation using rmcp
- forge-mcp appears deprecated but still in workspace
- Fix: Remove forge-mcp from workspace or mark deprecated clearly

### 7. EXPANSION_PLAN.md References Crates That Don't Exist Yet
**Severity: INFO**
- Plan mentions forge-knowledge, forge-messaging, forge-adapter-hermes, forge-adapter-openclaw
- These are Wave 4-7 items, correctly marked as "NEW" in the plan
- No inconsistency, just tracking

---

## Progress by Wave

| Wave | Target | Status | Blockers |
|------|--------|--------|----------|
| **Wave 1: Personas** | 100+ personas from agency-agents | **PARTIAL** | Crate ready. 0 persona files imported. Need `cp -r ../agency-agents/ personas/` + import run. |
| **Wave 2: Dev Methodology** | 14 superpowers skills + 6 claude-code plugin skills | **NOT STARTED** | Need to adapt markdown files to Forge skill format. |
| **Wave 3: Org Charts** | Multi-company, departments, goals, approvals | **PARTIAL** | Companies/departments/positions working. Goals/approvals are model-only stubs. |
| **Wave 4: Hermes Adapter** | Hermes as execution backend | **NOT STARTED** | Crate doesn't exist. |
| **Wave 5: OpenClaw Adapter** | OpenClaw webhook bridge | **NOT STARTED** | Crate doesn't exist. |
| **Wave 6: Knowledge Base** | Documents, chunks, FTS5 search | **NOT STARTED** | Crate doesn't exist. |
| **Wave 7: Messaging** | Telegram/Slack/Discord bridge | **NOT STARTED** | Crate doesn't exist. |
| **Wave 8: Desktop Client** | Electron app consuming Forge API | **NOT STARTED** | No desktop/ directory. |

---

## Feature Gap Analysis by Source Repo

### agency-agents -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Agent persona files | 130+ | 0 | **130+ files** |
| Division taxonomy | 11 divisions | 0 populated | **11 divisions** |
| NEXUS strategy framework | 7-phase pipeline | 0 | Not in plan |
| Multi-tool integration scripts | install.sh, convert.sh | 0 | Nice-to-have |
| Learning materials | 10 chapters | 0 | Documentation |

### AstrBot -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Pipeline stages (onion model) | 9 stages | Conceptually similar | Forge has 8-layer middleware |
| Platform adapters | 15+ | 0 | **All 15+ missing** |
| Knowledge base (RAG) | Full (FAISS+BM25) | 0 | **Entire subsystem** |
| Document parsers | PDF/DOCX/XLSX/TXT/MD | 0 | **All parsers** |
| Plugin hot-reload | watchfiles | 0 | Not planned |
| TTS/STT providers | 7+ TTS, 3 STT | 0 | Not planned |
| Multi-LLM providers | 28+ | 0 | Not planned |
| Sub-agent orchestration | SubAgentOrchestrator | 0 | Forge has ConcurrentRunner |
| Conversation management | Two-level hierarchy | 0 | Forge has flat sessions |

### claude-code -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Plugin architecture | 13 plugins | 0 | Skills adapted, not plugin system |
| PreToolUse/PostToolUse hooks | Full event system | Partial | Forge hooks lack pattern matchers |
| Feature-dev workflow (7-phase) | Complete | 0 | **Skill not imported** |
| Code review (confidence-scored) | Complete | 0 | **Skill not imported** |
| PR review (6 agents) | Complete | 0 | **Skill not imported** |
| Security guidance (9 patterns) | Complete | 0 | **Not implemented** |
| Plugin marketplace | Functional | 0 | Not planned |
| Hookify rules engine | Complete | 0 | Not planned |

### hermes-agent -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Self-improving skills | Auto-creation after tasks | 0 | **Not planned** |
| Persistent memory (MEMORY.md) | MEMORY.md + USER.md | Similar | Forge has memory table, different approach |
| Session FTS5 search | Full-text search | Done | Forge has sessions_fts |
| Messaging gateway | 6 platforms | 0 | **All missing** |
| Browser automation | Browserbase + vision | 0 | **Not planned** |
| 40+ tools | Categorized toolsets | 0 | Not directly applicable (Forge uses Claude CLI tools) |
| Terminal backends | 6 (local/docker/ssh/modal/daytona/singularity) | 0 | Not planned |
| Skin engine | 4 built-in + custom YAML | 0 | Not planned |
| MCP client | 1050+ lines | 0 | Not planned (Forge is server) |
| Tool registry | Centralized registration | 0 | Not directly applicable |

### Open-Claude-Cowork -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Desktop GUI (Electron) | Full app | 0 | **Wave 8, not started** |
| Token streaming UI | Real-time rendering | Done | Forge has WebSocket streaming |
| Tool permission control | Allow/deny per tool | 0 | Not planned |
| Multi-API support | Zhipu, MiniMax, DeepSeek | 0 | Not planned |

### openclaw -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Multi-channel gateway | 10+ channels | 0 | **All missing** |
| Extension system | 42+ extensions | 0 | Not applicable |
| Device pairing | Auth challenges | 0 | Not planned |
| Browser automation | Playwright | 0 | Not planned |
| Security modules | 31 directories | 0 | Not planned |
| SSH tunneling | Tunnel support | 0 | Not planned |

### paperclip -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Companies | Full CRUD + budget | **DONE** | Working |
| Departments | Full CRUD | **DONE** | Working |
| Org positions | Hierarchical | **DONE** | Working |
| Org chart tree | Visualization | **DONE** | Working |
| Goals hierarchy | Parent-child | **PARTIAL** | Model only, no service |
| Approval gates | Workflow | **PARTIAL** | Model only, no service |
| Budget enforcement | Hard-stop | **PARTIAL** | DB ready, not in middleware |
| Heartbeat system | Wake-ups | 0 | Not planned |
| Task checkout | Ticket semantics | 0 | Not planned |
| Multi-agent adapters | 7 adapters | 0 | **Planned Wave 4-5** |
| Activity logging | Immutable audit | **DONE** | Events table |

### superpowers -> Forge

| Feature | Available | Imported | Gap |
|---------|-----------|----------|-----|
| Brainstorming skill | Socratic design | 0 | **Planned Wave 2** |
| TDD skill | RED-GREEN-REFACTOR | 0 | **Planned Wave 2** |
| Debugging skill | 4-phase root cause | 0 | **Planned Wave 2** |
| Code review skill | Pre-review checklist | 0 | **Planned Wave 2** |
| Subagent-driven dev | Two-stage review | 0 | **Planned Wave 2** |
| Git worktree skill | Workspace management | 0 | **Planned Wave 2** |
| Plan execution skill | Batch checkpoints | 0 | **Planned Wave 2** |
| Writing skills skill | Meta-skill creation | 0 | **Planned Wave 2** |
| Multi-platform support | Claude/Cursor/Codex/OpenCode/Gemini | 0 | Not applicable (Forge-native) |

---

## Recommended Next Actions (Priority Order)

### Immediate (can do now, zero architecture changes)

1. **Import personas** — Copy 130+ .md files from agency-agents into `personas/` directory. Run Forge's persona parser to populate DB. *This completes Wave 1.*

2. **Import superpowers skills** — Adapt 14 skill .md files to Forge's YAML frontmatter format. Copy to `skills/superpowers/`. *This starts Wave 2.*

3. **Import claude-code plugin skills** — Extract core workflows from feature-dev, code-review, PR review, security-guidance plugins. Adapt to Forge skill format. *This continues Wave 2.*

### Short-term (1-2 sessions each)

4. **Wire company budget to middleware** — Extend CostCheck middleware to check per-company budget before spawn. Small change, high value.

5. **Add governance service layer** — Implement GovernanceService with CRUD for goals/approvals. Add API routes. Add frontend pages.

6. **Fix migration numbering** — Add 0010 or renumber 0011.

7. **Clean up forge-mcp** — Remove deprecated crate from workspace.

### Medium-term (per EXPANSION_PLAN.md waves)

8. **Wave 4: forge-adapter-hermes** — Hermes as execution backend
9. **Wave 5: forge-adapter-openclaw** — OpenClaw webhook bridge
10. **Wave 6: forge-knowledge** — Knowledge base with FTS5
11. **Wave 7: forge-messaging** — Platform bridges
12. **Wave 8: Desktop client** — Electron app

---

## Architecture Decision: Single Binary vs Satellite

**Current stance (from EXPANSION_PLAN.md):** Preserve single-binary for core features. External backends (Hermes, OpenClaw, AstrBot) are optional add-ons detected at runtime.

**What stays pure Rust (zero deps):**
- Persona catalog, skills, org charts, governance, knowledge base (FTS5), security scanning
- All existing features

**What requires external processes:**
- Hermes adapter (Python process)
- OpenClaw adapter (Node.js gateway)
- Messaging sidecar (AstrBot or native Rust)
- Desktop client (Electron binary)

**Recommendation:** This is the right approach. The core binary remains self-contained. Optional adapters extend reach without compromising the zero-deps philosophy.

---

## File References

| Document | Purpose |
|----------|---------|
| `docs/EXPANSION_PLAN.md` | Detailed wave-by-wave implementation plan (Waves 1-8) |
| `docs/CONSOLIDATION_STATUS.md` | This file: progress tracking, gap analysis, inconsistencies |
| `docs/BORROWED_IDEAS.md` | Earlier research from DeerFlow, Claude-Flow, 61 reference repos |
| `docs/RESEARCH_FINDINGS_2026_03_05.md` | Patterns from 67 repos |
| `NORTH_STAR.md` | Vision and current sprint |
| `MASTER_TASK_LIST.md` | Active sprint tasks |
