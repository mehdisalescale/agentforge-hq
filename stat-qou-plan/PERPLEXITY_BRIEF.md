# AgentForge HQ — Full Context for Research

> Copy this entire document into Perplexity Pro for research and strategic guidance.

---

## What AgentForge Is

AgentForge HQ is an open-source, self-improving AI workforce platform. Single binary (Rust/Axum backend + Svelte 5 frontend, embedded via rust-embed). It's the only Rust-native tool in this space.

**Repo**: github.com/mehdisalescale/agentforge-hq

**Core idea**: Users browse 100+ pre-built AI agent personas (engineers, designers, PMs, security auditors, etc.), hire them into organizational charts with budgets, and let them execute real work via Claude Code CLI — with governance controls (approvals, budgets, org scoping).

---

## Tech Stack

- **Backend**: Rust, Axum, SQLite (WAL mode, rusqlite), tokio
- **Frontend**: SvelteKit 5 (runes: $state, $derived), adapter-static, TailwindCSS 4
- **Execution engine**: Spawns `claude` CLI (Claude Code) as subprocess, streams JSON output via WebSocket
- **MCP Server**: rmcp v0.17 (official Rust MCP SDK), stdio transport, 10 tools
- **Safety**: Circuit breaker (3-state FSM), rate limiter (token bucket), CostTracker (budget)
- **13 workspace crates**, 229 tests, zero warnings

---

## What Actually Works Today

1. **Agent execution pipeline**: User picks agent → types prompt → 8-middleware chain processes → spawns `claude` CLI → streams output to browser via WebSocket. Sub-agent swim-lane view when Claude spawns sub-agents.

2. **Middleware chain** (8 deep): RateLimit → CircuitBreaker → CostCheck → SkillInjection → TaskTypeDetection → SecurityScan → Persist → Spawn

3. **Skills system**: 30 skills loaded at startup. TaskTypeDetector classifies prompts into 6 categories (NewFeature, BugFix, CodeReview, Refactor, Research, General). SkillRouter auto-injects relevant methodology skills into agent system prompts.

4. **Security scanning**: 9 OWASP regex patterns (command injection, XSS, eval, SQL injection, path traversal, etc.) run as post-execution middleware on output code blocks.

5. **Persona catalog**: 112 personas across 11 divisions. Browse, search, filter. Hire flow creates agent + org position automatically.

6. **Organization CRUD**: Companies (name, mission, budget), Departments, Org Chart visualization, Goals (hierarchical), Approvals (pending/approved/rejected).

7. **Demo seed**: First launch creates sample company, departments, agents, goals, approval so pages aren't empty.

8. **ProcessBackend trait**: Pluggable execution backend interface (currently only ClaudeBackend adapter).

9. **Review engine**: 6-aspect code review model with confidence scoring (data model only, not wired to execution yet).

---

## What Doesn't Work (Shells / Stubs)

| Feature | What exists | What's missing |
|---------|-------------|----------------|
| Workflows page | Empty UI | No execution engine, no DAG |
| Schedules page | Empty UI | Backend cron exists, no UI to manage |
| Memory page | Empty UI | Repo exists, agents can't read/write |
| Hooks page | Empty UI | Repo exists, no event triggers |
| Analytics page | Empty UI | Data collected, no dashboard |
| Settings page | Empty UI | All config via env vars only |
| Skills page | Empty UI | 30 skills loaded but invisible to user |
| Sessions | List only | Can't view past output |
| Company budgets | Display only | Not connected to actual cost tracking |
| Goals | Display only | Don't influence agent behavior |
| Approvals | Display only | Don't block any actions |

**Honest assessment: 8 of 16 pages are functional. The rest are decorative.**

---

## Our Plan (Two Phases)

### Phase 1: Finish First — Make Everything Real

Remove 4 shell pages from sidebar (Workflows, Memory, Hooks, Schedules). Wire remaining pages:

- **Wave A (Visibility)**: Skills page showing 30 loaded skills. Run metadata panel (task type, injected skills, security result). Session detail with past output. Agent cards with persona link and run count.

- **Wave B (Budget & Cost)**: Wire company budget to CostTracker. Analytics dashboard (run counts, costs, success rates). Settings page for runtime config.

- **Wave C (Governance)**: Inject active goals into agent context. Approval gating (block runs when approval required). Post-run approval for large changes.

- **Wave D (Cleanup)**: Remove shell pages from nav. Update onboarding. Health check for claude CLI. Test coverage to 250+.

**Result: 12 pages, all functional, zero fakes.**

### Phase 2: Evolve — AgentForge as AI Workforce Infrastructure

After everything works, evolve in two directions:

#### A. MCP Downstream Server
Expose AgentForge's capabilities as MCP tools so upstream AI tools (Claude Code, Cursor, ADK apps, Antigravity) can use the workforce:

- `forge_list_personas` / `forge_hire_persona` — workforce from any tool
- `forge_classify_task` / `forge_get_skills_for_task` — intelligence as a service
- `forge_security_scan` / `forge_code_review` — scanning from any tool
- `forge_run_agent` / `forge_get_session` — execution from any tool
- `forge_create_company` / `forge_request_approval` — governance from any tool
- Add HTTP SSE transport (currently stdio only) for remote access

#### B. Claude Code as the Engine (Don't Reinvent)
Stop duplicating what Claude Code already does (skills, memory, hooks, loop detection). Instead:

- **Orchestrate**: Spawn configured Claude Code instances per persona
- **Configure**: Generate CLAUDE.md per persona with role, goals, constraints, allowed tools
- **Observe**: Use Claude Code's native hooks to capture events back to AgentForge
- **Govern**: Enforce budgets, approvals, org scoping at the orchestration layer
- **Simplify middleware**: Drop SkillInjection/TaskTypeDetection/LoopDetection (Claude Code handles these). Keep only governance middleware (budget, approval, rate limit).

---

## Key Architecture Decisions Already Made

| Decision | Rationale |
|----------|-----------|
| Rust + Svelte 5 single binary | Performance, zero runtime deps, unique in space |
| SQLite WAL mode | Single-file, concurrent reads, no server to manage |
| Claude Code as execution engine | Don't build an LLM runtime, use the best one |
| Middleware chain pattern | Composable request processing (borrowed from DeerFlow) |
| Git worktree isolation | Each agent gets isolated repo copy for safe parallel work |
| rmcp for MCP | Official Rust MCP SDK, `#[tool]` macro |
| EventBus broadcast | All state changes emit typed events (37 variants) |
| BatchWriter | Durability: batch events to SQLite (50 per batch or 2s flush) |

---

## Questions We Need Research On

1. **MCP server architecture**: What's the best practice for building an MCP server that serves both as stdio (for Claude Code/Cursor) and HTTP SSE (for remote clients)? How are others doing multi-transport MCP servers in Rust?

2. **Claude Code hooks for event-driven orchestration**: Claude Code supports hooks (pre/post tool use). What's the best way to use these for reporting events back to an orchestrator? What hook events are available? How reliable is the hook system for production orchestration?

3. **Multi-agent coordination patterns**: What patterns exist for coordinating multiple Claude Code instances working on the same codebase? Beyond git worktrees, how do teams handle shared state, conflict resolution, task assignment?

4. **Competitive landscape**: What other tools serve as AI workforce management platforms? How do they handle persona management, budget governance, and multi-agent orchestration? Specifically: CrewAI, AutoGen, LangGraph, OpenHands, SWE-agent, Devin — how do they compare to our approach?

5. **MCP ecosystem adoption**: Which AI tools currently support MCP as clients? What tools would benefit most from an AgentForge MCP server? What's the current state of MCP adoption in the developer tools ecosystem (March 2026)?

6. **Budget and cost tracking for AI agents**: How are teams tracking and enforcing budgets for AI agent usage? What's the state of the art for cost attribution, per-agent metering, and budget alerts?

7. **Agent persona best practices**: Are there established patterns for defining AI agent personas/roles? How specific should persona definitions be? What makes a persona definition effective vs decorative?

8. **Approval workflows for AI agents**: How are teams implementing human-in-the-loop governance for AI agents? What actions should require approval? What's the right granularity — per-run, per-tool-use, per-file-change?

9. **Event-driven architecture for agent observability**: What event schemas and patterns are emerging for tracking AI agent behavior? How to build audit trails that are useful for debugging and compliance?

10. **Rust + MCP server deployment**: Best practices for deploying a Rust MCP server — Docker, systemd, cloud platforms? How to handle auth, multi-tenancy, rate limiting for a hosted MCP service?

---

## What We Want From This Research

- Validation or correction of our architectural direction
- Patterns and practices from teams who've built similar systems
- Gaps in our thinking we haven't considered
- Prioritization guidance: what matters most for a v1.0
- Technical recommendations for the MCP server evolution
- Competitive positioning: where AgentForge fits in the landscape
