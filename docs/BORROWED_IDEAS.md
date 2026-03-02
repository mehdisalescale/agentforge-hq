# Borrowed Ideas — DeerFlow, Claude-Flow, Reference Repos & Industry Research → Forge

> Initial analysis: 2026-03-01
> **Corrected with deep code audit + industry research: 2026-03-02**
> See also: [FORGE_AUDIT_2026_03_02.md](./FORGE_AUDIT_2026_03_02.md) for full audit

---

## Forge Current State (Verified)

Rust/Axum + SvelteKit 5, single binary. 9 crates, **3,400 LOC Rust, 1,400 LOC frontend.**

**What works (verified by `cargo check` + `cargo test`):**
- Agent CRUD (9 presets) — API + UI, full validation, 7 tests
- Process spawn with stream-json parsing — real tokio::process, 7 tests
- Sessions CRUD + export (JSON/Markdown) — API + UI
- WebSocket event streaming — broadcast EventBus, frontend reconnection
- Safety layer — circuit breaker (3-state FSM, 10 tests), rate limiter (token bucket)
- SQLite WAL + BatchWriter (50 events / 2s flush) — 14 tests
- Embedded frontend via rust-embed — zero-config deployment
- Graceful shutdown — Ctrl+C → stop server → flush BatchWriter
- 33 tests total, all passing, zero compile warnings

**What's missing:**
- MCP server — forge-mcp is 50 lines of type stubs (Grade: D)
- Skills — table exists, read-only API exists, UI exists, **zero content or loader**
- Workflows — same as skills
- Middleware chain — run handler is inline monolith
- Multi-agent coordination — single process per run
- Cross-session memory — events persisted but no semantic recall
- Authentication — none anywhere

---

## DeerFlow — Verified Real Code

> **Audit method:** Read actual Python source files, not README.
> **Verdict: Legitimate.** ~10K lines of working Python, 898 commits.

| Feature | LOC | Verified? | Detail |
|---------|-----|-----------|--------|
| **8 Middlewares** | 1,089 | YES | 75-220 lines each. Thread-data → uploads → sandbox → dangling-calls → summarization → title → memory → view-image → clarification. **Note: 8, not 9 as docs claim.** |
| **Sub-Agent Executor** | 414 | YES | Real ThreadPool (3 scheduler + 3 execution workers), async background tasks, timeout handling with FuturesTimeoutError, status tracking (PENDING→RUNNING→COMPLETED/FAILED/TIMED_OUT) |
| **Memory System** | 815 | YES | LLM-powered updater (319 lines) + debounced queue (191 lines) + extraction prompts (261 lines). Sections: workContext, personalContext, topOfMind, recentMonths. Atomic file writes (temp + rename). Disabled by default. |
| **15 Skills** | 208 loader | YES | Real SKILL.md files with YAML frontmatter in `skills/public/`. deep-research, github-deep-research, data-analysis, image-generation, video-generation, podcast-generation, ppt-generation, chart-visualization, web-design-guidelines, frontend-design, skill-creator, find-skills, surprise-me, vercel-deploy, consulting-analysis |
| **Sandbox** | 903 | YES | Real subprocess.run() with path mapping, not a mock. Local (183 lines) + AIO (1,401 lines). |
| **Gateway API** | 1,283 | YES | FastAPI with models, MCP config, skills CRUD, memory, uploads, artifacts |
| **MCP Integration** | 269 | YES | Client implementation with tool caching and server config |

**What DeerFlow is NOT:**
- Not a stub project with 90% docs
- Not a demo or proof-of-concept
- Not vaporware

**One honest gap:** Docs claim "9 middlewares" but code has exactly 8.

---

## Claude-Flow — Mixed (~60% Real, ~40% Hype)

> **Audit method:** Read actual TypeScript source files, verified against 277KB README claims.
> **Verdict: Inflated.** README is marketing. Real value in swarm coordinator and architecture patterns.

| Claim | Reality | Verified Detail |
|-------|---------|-----------------|
| **15-agent hierarchical mesh** | REAL but minimal testing | Swarm package: 8,763 lines. Unified coordinator (1,844 lines), queen-coordinator (2,025 lines), topology manager. Architecture works, distributed scenarios untested. |
| **9 RL algorithms** | REAL but simplified | Q-Learning (310 lines), SARSA (320), DQN (390), PPO (430), Decision Transformer (310), A2C (340), Curiosity (350+). All Float32Array — no GPU, no TensorFlow. Proof-of-concept scale. |
| **HNSW vector search** | DELEGATED | 1,013-line wrapper, but heavy lifting done by `agentdb@2.0.0-alpha.3.7`. The "150x-12,500x faster" claim belongs to agentdb, not claude-flow. |
| **100+ MCP tools** | VAPORWARE | ~20-30 real implementations. Rest are `.d.ts` type definitions. |
| **42+ skills** | MOSTLY STUBS | 89 folders in `.agents/skills/`, but ~5-10 have actual code. Most contain only a single `skill.md` metadata file. |
| **RuVector intelligence** | EXTERNAL | Integration code exists (2,000+ lines), but SONA, EWC++, Flash Attention come from `@ruvector/*` packages. README conflates imports with own work. |
| **Production ready** | v2 ONLY | v2 (279K lines) is stable. v3 (393K lines) is alpha.44 with many incomplete packages. |
| **Consensus protocols** | REAL types, untested | Raft, Byzantine, Gossip, CRDT defined but no distributed test validation. |
| **Dual-mode execution** | CONCEPT | Claude + Codex parallel execution described in docs, minimal real integration. |

**What's genuinely good in Claude-Flow:**
- Swarm coordinator architecture (domain-based agent grouping)
- 10 ADRs (Architecture Decision Records) — good governance pattern
- Agent YAML definitions (role, backstory, tools, constraints)
- The *idea* of domain-based task routing (even if implementation is incomplete)

**What to ignore:**
- Feature counts (divide by 3-5x for reality)
- Performance claims (belong to external libraries)
- "Production-ready" label (v3 is alpha)

---

## Reference Repos — Top 4 (from 61 submodules)

> Located at `/Users/bm/claude-parent/refrence-repo/` with summaries in `/Users/bm/claude-parent/reference-map/`

### 1. hooks-multi-agent-observability — HIGHEST VALUE
- Real-time swimlane visualization with Vue 3 dashboard
- Intercepts all 12 hook event types
- SQLite + WAL persistence + WebSocket streaming
- Tool emoji system (Bash: `>_`, Read: eye, Write: pencil)
- Agent team tracking (Builder/Validator patterns)
- **For Forge:** Upgrade EventStream UI, add swimlane visualization

### 2. Claude-Code-Workflow — ORCHESTRATION REFERENCE
- 4-level complexity tiers (lite-lite-lite → brainstorm)
- Dependency-aware parallelism
- JSON state persistence (`IMPL-*.json`)
- CodexLens: FTS + semantic search over code
- **For Forge:** Workflow DSL design, task dependency graph

### 3. ralph-claude-code — SAFETY PATTERNS
- 3-state circuit breaker (matches Forge's existing design)
- 100/hr rate limiting
- Autonomous loop with graceful exit detection
- **For Forge:** Validates Forge's safety approach, adds exit detection pattern

### 4. infrastructure-showcase — SKILL ACTIVATION
- Skill auto-activation via hooks (UserPromptSubmit, PostToolUse)
- 500-line rule with progressive disclosure
- skill-rules.json configuration
- 5 production skills + 10 specialized agents
- **For Forge:** Skill activation pattern for Phase 2

### Notable Others
- **1code** — Best desktop GUI reference (worktree-per-session, Kanban, Electron)
- **claude-code-viewer** — Session browser with FTS fuzzy search, cron scheduling
- **claude-code-plugins-plus-skills** — 1,500 skills catalog
- **claude-code-templates** — npx installer, marketplace patterns

### Forge's Unique Position
Forge is the **only** Rust + Svelte 5 project in the 61-repo ecosystem. All others use TypeScript/Bun or Python. This is a genuine differentiator.

---

## Industry Research (2025-2026)

### Rust AI Frameworks — Production Exists

| Framework | Key Feature | Relevance to Forge |
|-----------|-------------|-------------------|
| **Rig** (rig.rs) | 20+ LLM providers, shipping in production | General Rust LLM patterns |
| **AutoAgents** | Ractor actor model, 5x memory advantage over Python | Concurrency model for multi-agent |
| **OpenFang** | 137K Rust, single ~32MB binary, 16 security systems | **Most similar architecture to Forge** |

### Key Industry Findings

**MCP is now universal:**
- 97M+ monthly SDK downloads, donated to Linux Foundation (Dec 2025)
- **rmcp** is the official Rust SDK — `#[tool]` macro, async tasks
- Forge should rewrite MCP with rmcp, not hand-rolled JSON-RPC

**Worktree isolation is standard:**
- Claude Code official: `claude --worktree` (Feb 2026)
- Hidden TeammateTool inside Claude Code (feature-flagged off)
- Community consensus: one worktree per agent

**Orchestration converging on supervisor + graph hybrid:**
- 72% of enterprise AI uses multi-agent (up from 23% in 2024)
- LangGraph 1.0 (Oct 2025) is the reference implementation
- Pattern: central orchestrator + conditional routing for complex tasks

**Memory needs four types:**
| Type | What | Forge Has? |
|------|------|------------|
| Working | In-memory context | YES (DashMap, broadcast) |
| Episodic | Event log | YES (SQLite events) |
| Semantic | Facts + embeddings | NO |
| Procedural | Skills/procedures | NO (stubs only) |

**Safety requires layers (NIST mandated 2025):**
1. Gateway/proxy — cost control, rate limiting → Forge: partial
2. I/O validation — before/after LLM → Forge: NO
3. Agent-level — circuit breaker, max-turns, tool allowlists → Forge: YES
4. Observability — tracing, anomaly detection → Forge: partial

---

## Tier 1 — Steal the Design Now

These fill gaps that exist in forge-project today. Ordered by sprint plan.

### 1. Middleware Chain (from DeerFlow)

**Problem:** Forge's `run::handle_run` is an inline monolith — rate limit check, circuit breaker check, spawn, persist, cost track all in one function. Every new feature (skills, memory, hooks) will make it worse.

**What to borrow:**
- DeerFlow's 8-middleware ordered pipeline with strict execution order
- Each middleware is 75-220 lines of focused, reusable logic
- Clear responsibility boundaries per stage

**Proposed chain for Forge:**

```
Request
  → RateLimitMiddleware       (token bucket check)
  → CircuitBreakerMiddleware  (failure threshold check)
  → SkillInjectionMiddleware  (match + inject skill into prompt)
  → ContextMiddleware         (load session history, summarize if needed)
  → SpawnMiddleware           (actually run claude process)
  → PersistMiddleware         (save events via BatchWriter)
  → CostMiddleware            (track cost, check budget)
Response
```

**Implementation path:**
1. Define `Middleware` trait in forge-api: `async fn process(&self, ctx: &mut RunContext, next: Next) -> Result<RunResponse>`
2. Build `MiddlewareChain` that runs in order
3. Refactor `run::handle_run` to use the chain

**Reference files (verified real):**
- `deer-flow/backend/src/agents/middlewares/` — 8 middleware implementations, 1,089 lines total
- `deer-flow/backend/src/agents/middlewares/__init__.py` — chain ordering

**Effort:** 1-2 days (refactor, not rewrite). **Sprint 2.**

---

### 2. Skill System (from DeerFlow + reference repos)

**Problem:** Skills table exists, API routes exist, UI page exists — zero content, no loader, no activation logic.

**What to borrow:**
- DeerFlow: Skills are **Markdown files** with YAML frontmatter (name, description, tags, tools)
- DeerFlow: `load_skills()` scans `skills/public/` and `skills/custom/` (208 lines of real loader code)
- DeerFlow: 15 real SKILL.md files with structured workflow content
- infrastructure-showcase: Skill auto-activation via hooks, progressive disclosure
- claude-code-plugins-plus-skills: 1,500 skills catalog for seeding

**Implementation path:**
1. Create `forge-project/skills/` directory with `*.md` files
2. Write loader: parse YAML frontmatter + Markdown body at startup → populate `skills` table
3. Seed 10-15 skills from reference repos (deep-research, code-review, refactor, test, debug, security-audit, document, architect, explore)
4. Add skill matching logic (keyword match on user prompt) to run pipeline
5. Inject matched skill content into agent system prompt before `claude -p`

**Reference files (verified real):**
- `deer-flow/skills/public/*/SKILL.md` — 15 skill directories
- `deer-flow/backend/src/skills/` — 208-line loader
- `refrence-repo/claude-code-infrastructure-showcase/` — skill-rules.json, auto-activation

**Effort:** 2-3 days. **Sprint 2.**

---

### 3. Git Worktree Isolation (from industry + Claude Code official)

**Problem:** Forge runs agents in the user's working directory. Multiple agents would step on each other.

**What to borrow:**
- Claude Code official `--worktree` support (Feb 2026)
- ccswarm: worktree-per-agent pattern
- 1code: worktree-per-session isolation

**Implementation path:**
1. On run: `git worktree add .worktrees/{session_id} -b forge/{session_id}`
2. Pass worktree path as working_dir to ProcessRunner::spawn
3. On completion: optionally merge branch or keep for review
4. Add `forge-git` module (git2 crate or shell commands)
5. Cleanup: prune worktrees on session delete

**Effort:** 2-3 days. **Sprint 2.**

---

### 4. Sub-Agent Parallel Spawning (from DeerFlow)

**Problem:** Forge spawns one Claude process per run. Real orchestration needs parallelism.

**What to borrow:**
- DeerFlow's SubagentExecutor (414 lines, verified real):
  - ThreadPool (3 scheduler + 3 execution workers)
  - Async background tasks with timeout handling
  - Status tracking: PENDING → RUNNING → COMPLETED/FAILED/TIMED_OUT
  - Tool filtering (allowlist/denylist)
  - Model inheritance from parent agent
- Lead agent decides when to spawn sub-agents via tool call

**Implementation path:**
1. Add `SubAgentRequest` / `SubAgentCompleted` / `SubAgentFailed` events to forge-core
2. Extend `ProcessRunner` to manage N concurrent spawns (configurable, default 3)
3. Each sub-agent gets its own worktree (depends on #3 above)
4. Coordinator aggregates results
5. WebSocket emits per-sub-agent progress events

**Reference files (verified real):**
- `deer-flow/backend/src/subagents/executor.py` — 414 lines, real threadpool + timeout

**Effort:** 3-5 days. Depends on #3. **Sprint 3.**

---

## Tier 2 — Port the Architecture

After Tier 1 is solid.

### 5. Cross-Session Memory (from DeerFlow + Mem0 pattern)

**Problem:** Agents restart from zero each session. No learning, no personalization.

**What to borrow:**
- DeerFlow memory system (815 lines, verified real):
  - `MemoryUpdater`: LLM-powered analysis of conversations (319 lines)
  - `MemoryQueue`: debounced updates to avoid LLM spam (191 lines)
  - Sections: workContext, personalContext, topOfMind, recentMonths
  - Confidence-scored facts with max_facts enforcement
  - Atomic file writes (temp + rename)
  - Disabled by default (opt-in)
- Mem0 ETL pattern ($24M Series A, 2025): Extract → Transform → Load
- Industry standard: 4 memory types (working, episodic, semantic, procedural)

**Implementation path:**
1. Add `memory` table: id, category, content, confidence, created_at, updated_at
2. Post-session middleware: send transcript to Claude, extract reusable facts
3. Store with confidence scores, enforce max per category
4. On new run: query relevant memories, prepend to system prompt
5. Memory management UI (view, edit, delete facts)
6. Debounce: don't re-extract until session ends

**Reference files (verified real):**
- `deer-flow/backend/src/agents/memory/updater.py` — 319 lines
- `deer-flow/backend/src/agents/memory/queue.py` — 191 lines
- `deer-flow/backend/src/agents/memory/prompt.py` — 261 lines

**Effort:** 3-4 days. **Sprint 4.**

---

### 6. Hook System (from Claude-Flow + reference repos)

**Problem:** No way to run actions before/after agent runs (lint, test, security scan, notify).

**What to borrow:**
- Claude-Flow concept: pre/post hooks on agent lifecycle events
- hooks-mastery repo: 13 hook types, builder/validator pattern
- hooks-observability repo: hook event interception + dashboard

**Corrected understanding:** Claude-Flow claims 17 hook types, but many are `.d.ts` stubs. The **reference repos** (hooks-mastery, hooks-observability) have more real implementations.

**Implementation path:**
1. Add `hooks` table: id, event_type, timing (pre/post), command, enabled
2. `HookRunner` in forge-process: execute shell commands around process spawn
3. Emit `HookStarted` / `HookCompleted` / `HookFailed` events
4. UI page to manage hooks
5. Seed with useful defaults: pre-run lint check, post-run test trigger

**Reference files:**
- `refrence-repo/claude-code-hooks-mastery/` — 13 hook types
- `refrence-repo/claude-code-hooks-multi-agent-observability/` — event interception

**Effort:** 2-3 days. **Sprint 4.**

---

### 7. Agent Domains + Coordinator (from Claude-Flow concepts)

**Problem:** Flat list of 9 agent presets. No coordination, no routing, no chaining.

**What to borrow:**
- Claude-Flow's domain-based grouping concept (the architecture idea, not the incomplete implementation):
  - Code domain: CodeWriter, Refactorer
  - Quality domain: Reviewer, Tester, Debugger
  - Ops domain: SecurityAuditor, Architect, Documenter, Explorer
- Coordinator pattern: receive task → analyze → delegate to domain-appropriate agent(s)
- Agent chaining: Architect → CodeWriter → Reviewer → Tester

**Implementation path:**
1. Add `domain: Option<String>` to Agent model
2. Assign domains to existing 9 presets
3. Build `Coordinator` agent that picks agents based on task analysis
4. Support sequential chains and parallel fan-out (depends on #4)

**Reference:** Study the *architecture* in `claude-flow/v3/@claude-flow/swarm/src/unified-coordinator.ts` (1,844 lines), adapt for Rust.

**Effort:** 2-3 days. Depends on #4. **Sprint 3.**

---

## Tier 3 — Study, Don't Copy

### 8. rmcp for MCP Server (from industry)

Not borrowed from deer-flow or claude-flow — this comes from **industry research.**

- `rmcp` is the official Rust MCP SDK (modelcontextprotocol/rust-sdk)
- `#[tool]` macro eliminates boilerplate
- Async tasks support (SEP-1686)
- Forge's forge-mcp-bin hand-rolls JSON-RPC — should be rewritten with rmcp

**This is Sprint 1 priority** (part of v0.2.0 MCP delivery).

---

### 9. Things to Study but NOT Copy

| Source | What to Study | Why Not Copy |
|--------|--------------|--------------|
| Claude-Flow: RuVector/SONA | RETRIEVE → JUDGE → DISTILL pattern | No usage data to train on yet |
| Claude-Flow: Consensus | Raft leader election | Agents are independent, don't need agreement |
| Claude-Flow: RL algorithms | Q-Learning for routing | Simplified JS, not production-grade |
| DeerFlow: Sandbox abstraction | Virtual path translation | Docker/K8s mode adds complexity without current value |
| DeerFlow: File upload + conversion | PDF/PPT → Markdown pipeline | Nice-to-have, not core to orchestration |
| OpenFang: 16 security systems | WASM sandbox, Ed25519 signing, Merkle audit, taint tracking | Overkill for current stage, study for Phase 5+ |
| Industry: Mem0 graph memory | Relational memory structures | SQLite + flat facts sufficient for now |
| Industry: LiteLLM proxy | Hierarchical budgets, multi-provider | Forge is Claude-only, add later if needed |

---

## Priority Order (Revised with Sprint Plan)

```
SPRINT 1 — v0.2.0 (Ship MCP + Fix Bugs)
  #8  MCP server rewrite with rmcp
  Fix: Dashboard null-safety bug
  Fix: Budget warning logic
  Fix: Preset serialization (Debug format → proper serde)

SPRINT 2 — v0.3.0 (Middleware + Skills + Worktrees)
  #1  Middleware chain (refactor run handler)
  #2  Skill system (loader + 10-15 seed skills)
  #3  Git worktree isolation

SPRINT 3 — v0.4.0 (Multi-Agent)
  #4  Sub-agent parallel spawning (needs #1, #3)
  #7  Agent domains + coordinator (needs #4)

SPRINT 4 — v0.5.0 (Memory + Hooks)
  #5  Cross-session memory
  #6  Hook system

LATER — Study and selectively adopt from Tier 3
```

---

## Source Repo Locations

| Repo | Path | Stack | Verified? |
|------|------|-------|-----------|
| DeerFlow | `/Users/bm/cod/trend/26-feb/deer-flow` | Python, LangGraph, FastAPI, Next.js | YES — ~10K real LOC, 898 commits |
| Claude-Flow | `/Users/bm/cod/trend/26-feb/claude-flow` | TypeScript, Node.js, pnpm monorepo | PARTIAL — ~60% real, ~40% stubs/hype |
| Reference Repos | `/Users/bm/claude-parent/refrence-repo/` | Mixed (61 submodules) | Cataloged, top 4 deep-read |
| Reference Map | `/Users/bm/claude-parent/reference-map/` | Markdown summaries | 13 categories indexed |
| Forge (ours) | `/Users/bm/claude-parent/forge-project` | Rust, Axum, SvelteKit 5, SQLite | YES — 3,400 LOC, 33 tests pass |

## Industry Sources

- **rmcp** — Official Rust MCP SDK (modelcontextprotocol/rust-sdk)
- **OpenFang** — 137K Rust agent OS, closest architecture to Forge (openfang.app)
- **AutoAgents** — Rust multi-agent, 5x memory vs Python (liquidos-ai/AutoAgents)
- **Rig** — Production Rust LLM framework (rig.rs)
- **LangGraph 1.0** — Graph orchestration reference, Oct 2025
- **Mem0** — Cross-session memory, $24M Series A (mem0.ai)
- **NIST AI RMF 2025** — Mandates layered safety for agentic AI
- **Claude Code worktrees** — Official multi-agent isolation, Feb 2026
