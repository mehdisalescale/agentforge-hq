# Claude Forge -- North Star

> **Read this first in every session.** This is the single source of truth.
> Last updated: 2026-02-26 (Strategic reset — ship-first approach)

---

## What We're Building

A multi-agent Claude Code orchestrator: Rust/Axum + Svelte 5, single binary.
The only Rust-native tool in the space — everyone else is TypeScript/Electron or Python.

**One-liner**: Spawn Claude Code agents, see their output in a real-time UI, keep them safe. One binary, zero deps.

---

## Strategic Reset (2026-02-26)

After auditing the full project state — 44K lines of docs, 3K lines of code, 61 reference repos, and the competitive landscape — we're resetting priorities.

**What we learned:**
- The tech stack (Rust + Svelte 5 + single binary) is our real differentiator
- MCP-first is the right integration bet (97M+ monthly SDK downloads, industry standard)
- The existing prototype in `claude-forge/` already works — 8 crates, agent CRUD, process spawning, WebSocket streaming, SQLite persistence
- The 7-phase, 27-week roadmap was over-scoped for a solo/small team
- No competitor in the orchestration tier has broken out commercially yet — the window is open but closing
- Every successful tool (Cursor, Aider, Claude Code) started with one interaction loop and iterated

**What changes:**
- Ship the existing prototype, don't rewrite it
- 4 lean phases replace the 7-phase roadmap
- Cut WASM plugins, ML predictions, multi-CLI, 1,500 skills, 5 notification channels
- Stop maintaining 61 submodules and 44K lines of docs — freeze as reference
- Get users before building more features

See `STRATEGIC_ASSESSMENT.md` for the full analysis.

---

## Current State

### What Works (in `claude-forge/`)
- **8 Rust crates**: forge-core, forge-agent, forge-db, forge-api, forge-process, forge-safety, forge-mcp, forge-app
- Agent CRUD + 9 presets
- Process spawning with `--resume` session continuity
- Real-time WebSocket event streaming + rich rendering
- SQLite persistence (batch writes every 50 events or 2s, WAL mode)
- Agent edit page + shared AgentForm component
- Directory picker, Export (JSON/Markdown), CLAUDE.md editor
- MCP server editor, Hooks editor
- Multi-pane tab layout with split view
- Session browser backend (`GET /api/sessions`, `/api/sessions/projects`)

### What's Missing for v0.1.0
- Session browser frontend (`+page.svelte` + nav link)
- End-to-end smoke test
- GitHub Release with binary

---

## Development Priorities

### Phase 1 sprint status (forge-project codebase)

Phase 1 complete: Tracks **A–F** implemented (spawn, runner, sessions, run with real spawn, Agents CRUD, Run + Sessions UI). **Next sprint:** [NEXT_SPRINT_AGENT_TASKS.md](NEXT_SPRINT_AGENT_TASKS.md) — tasks A–F for Phase 1 polish + Phase 2 seed (directory, BatchWriter, E2E, workflow/skill stubs). See [PHASE1_6_AGENT_SPRINT.md](PHASE1_6_AGENT_SPRINT.md) for Phase 1 detail.

### Phase A: Ship What We Have (2 weeks)

| Task | Status | Notes |
|------|--------|-------|
| Finish session browser frontend | Not started | Backend done, needs UI page |
| End-to-end smoke test | Not started | Create agent -> run prompt -> see output -> export |
| Fix known code issues | Not started | See `AUDIT_REPORT.md` Rust section |
| Ship `v0.1.0` on GitHub Releases | Not started | Single binary, macOS first |
| Get 5 people to try it | Not started | Real users, real feedback |

### Phase B: Core Loop + MCP (4 weeks)

| Task | Status | Notes |
|------|--------|-------|
| Fix user-reported issues | Blocked by Phase A | Whatever breaks |
| MCP server (10 tools, stdio) | Not started | agent_create, agent_run, session_list, etc. |
| Circuit breaker | Not started | 3-state FSM from ralph-claude-code pattern |
| Rate limiter | Not started | Token bucket, per-agent + global |
| Ship `v0.2.0` | Blocked by above | |

### Phase C: Differentiate (4 weeks)

Pick ONE feature no competitor does well:
- **Option 1**: Multi-agent swim-lane visualization (observability)
- **Option 2**: Worktree-per-agent isolation (safety)
- **Option 3**: Workflow DAG execution (automation)

Decision deferred until after Phase A user feedback.

### Phase D: User-Driven Iteration

Build what users ask for. No pre-planned scope.

---

## What's Cut (Parked for Later)

These are NOT in scope until users demand them:

| Feature | Why Cut |
|---------|---------|
| WASM plugin runtime | No competitor uses WASM plugins; MCP servers are the extension mechanism |
| Telegram/Discord/email notifications | Webhook is enough for now |
| Multi-CLI orchestration (Codex, Gemini, Qwen) | Nobody is routing between CLIs yet; Claude Code only |
| Plugin marketplace | Need users before a marketplace |
| 1,500+ skills catalog | Ship with 0 skills; add 10-20 when users ask |
| 100+ agent presets | 9 good presets > 100 mediocre ones |
| ML-based usage prediction | Simple token counting + budget threshold |
| Cron scheduler | Run manually now, automate later |
| Kanban session view | Simple session list with filters |
| Security scanning with semantic analysis | File protection rules (glob patterns) |
| Dev environment (code viewer, terminal, file explorer) | Post-1.0 if ever |

---

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Rust + Svelte 5 single binary | Performance, no runtime deps, unique in space | Pre-project |
| SQLite WAL mode | Single-file, concurrent reads, no server | Pre-project |
| Ship existing prototype (don't rewrite) | 3K lines of working code > 44K lines of planning | 2026-02-26 |
| 4 lean phases (not 7) | Ship in weeks, not months; iterate on user feedback | 2026-02-26 |
| Claude Code only (no multi-CLI) | Speculative demand; add others when users ask | 2026-02-26 |
| Cut WASM plugins | No competitor uses them; MCP is the extension mechanism | 2026-02-26 |
| Freeze docs at 44K lines | Reference only; stop updating planning docs | 2026-02-26 |
| Archive 61-submodule setup | Knowledge extracted; maintenance overhead not justified | 2026-02-26 |
| MCP server as Phase B priority | Market standard (97M+ SDK downloads); table stakes | 2026-02-26 |

---

## Session Protocol

### Before Starting Work
1. Read this file
2. Check `SESSION_LOG.md` for recent sessions
3. Pick a task from the current phase

### During Work
- One session = one focused deliverable
- Commit early, commit often
- Don't update planning docs — write code

### When Done
1. Commit all changes
2. Log what was done in SESSION_LOG.md
3. Update this file only if priorities changed

---

## File Map

```
claude-forge/               <-- THE PRODUCT. All code lives here.
  src/                      <-- Rust source (8 crates)
  frontend/                 <-- SvelteKit app
  Cargo.toml                <-- Workspace root

forge-project/              <-- Planning docs (FROZEN — reference only)
  NORTH_STAR.md             <-- You are here. The only doc that gets updated.
  STRATEGIC_ASSESSMENT.md   <-- Why we reset (2026-02-26)
  AUDIT_REPORT.md           <-- Doc-level inconsistency audit
  SESSION_LOG.md            <-- Session history
  00-vision/ through 08-reference/  <-- Frozen reference material

refrence-repo/              <-- 61 reference repos (ARCHIVED — consult as needed)
reference-map/              <-- Repo analysis summaries (ARCHIVED)
```
