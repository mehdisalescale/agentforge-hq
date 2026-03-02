# Claude Forge — North Star

> **Read this first in every session.** This is the single source of truth.
> Last updated: 2026-03-02 (Post-audit — 4-sprint plan)

---

## What We're Building

A multi-agent Claude Code orchestrator: Rust/Axum + Svelte 5, single binary.
The only Rust-native tool in the space — everyone else is TypeScript/Electron or Python.

**One-liner**: Spawn Claude Code agents, see their output in a real-time UI, keep them safe. One binary, zero deps.

---

## Current State (Verified 2026-03-02)

Full audit completed. See `docs/FORGE_AUDIT_2026_03_02.md` for details.

### Build Status
- `cargo check` — clean, zero warnings, all 9 crates compile
- `cargo test` — 33 tests, all pass
- Frontend — built and embedded (SvelteKit 5 + adapter-static)

### What Works (verified in code, 3,400 LOC Rust + 1,400 LOC frontend)

**Backend (9 crates):**
- forge-core (A): ForgeEvent (20 variants), EventBus broadcast, ForgeError, typed IDs
- forge-agent (A-): 9 presets, Agent/NewAgent/UpdateAgent, name validation
- forge-db (A): SQLite WAL, migrations, BatchWriter (50/2s), AgentRepo CRUD, SessionRepo CRUD, EventRepo
- forge-process (B+): Claude CLI spawn with stream-json, content block parsing
- forge-safety (B+): CircuitBreaker (3-state FSM), RateLimiter (token bucket)
- forge-api (A-): Full HTTP API + WebSocket, CORS, TraceLayer, rust-embed SPA
- forge-app (A): Binary wiring, graceful shutdown, env config
- forge-mcp (D): Type stubs only — needs rewrite with rmcp
- forge-mcp-bin (B): Stdio JSON-RPC dispatch, functional but untested

**Frontend:**
- Dashboard — agent selector, prompt input, WebSocket streaming, markdown rendering, session resume
- Agents — full CRUD, 9 presets, form validation
- Sessions — two-pane layout, status badges, export (JSON/Markdown)
- Skills/Workflows — read-only placeholders (empty data)
- Settings — empty placeholder

**Infrastructure:**
- GitHub Actions CI (test + clippy + build)
- GitHub Release workflow (tag → binaries)
- E2E smoke test script
- Configurable: FORGE_HOST, FORGE_PORT, FORGE_DB_PATH, FORGE_CORS_ORIGIN, FORGE_CLI_COMMAND, FORGE_RATE_LIMIT_*, FORGE_BUDGET_*

### What's Missing

| Gap | Severity | Sprint |
|-----|----------|--------|
| MCP server (should use rmcp, not hand-rolled) | High | 1 |
| Skills system (table exists, zero content/loader) | High | 2 |
| Middleware chain (run handler is monolith) | High | 2 |
| Git worktree isolation (no multi-agent safety) | High | 2 |
| Sub-agent parallelism (single process only) | High | 3 |
| Cross-session memory (no semantic recall) | Medium | 4 |
| Hook system (no pre/post actions) | Medium | 4 |
| Authentication (none anywhere) | Medium | Later |

### Known Bugs

| Bug | File | Severity |
|-----|------|----------|
| Dashboard null-safety: `last.content += content` when outputBlocks empty | frontend/+page.svelte | Critical |
| Budget warning logic confusing (`cost >= warn AND cost < limit`) | forge-api/routes/run.rs | Medium |
| Preset serialization uses Debug format fallback (brittle) | forge-db/repos/agents.rs | Medium |
| No shutdown timeout (server can hang on Ctrl+C) | forge-app/main.rs | Low |
| Default model hardcoded in Agents UI | frontend/agents/+page.svelte | Low |

---

## Sprint Plan

Derived from comprehensive audit (2026-03-02). See `docs/FORGE_AUDIT_2026_03_02.md` and `docs/BORROWED_IDEAS.md` for full rationale.

### Sprint 1 → v0.2.0 — Ship MCP + Fix Bugs

| Task | Detail |
|------|--------|
| Fix Dashboard null-safety | Check `outputBlocks.length` before accessing last element |
| Fix budget warning logic | `warn <= cost < limit` |
| Fix preset serialization | Proper serde instead of Debug format fallback |
| Rewrite MCP server with rmcp | Official Rust MCP SDK, `#[tool]` macros, proper protocol compliance |
| Wire CostTracker | Connect to session cost data, emit events |

### Sprint 2 → v0.3.0 — Middleware + Skills + Worktrees

| Task | Detail |
|------|--------|
| Middleware chain | Refactor run handler into ordered pipeline: RateLimit → CircuitBreaker → SkillInjection → Context → Spawn → Persist → Cost |
| Skill system | Markdown-based skills with YAML frontmatter, loader at startup, keyword matching, prompt injection. Seed 10-15 skills. Pattern from DeerFlow (verified real). |
| Git worktree isolation | `git worktree add` per session, pass as working_dir to spawn, cleanup on delete. Now standard for multi-Claude-Code (official support Feb 2026). |

### Sprint 3 → v0.4.0 — Multi-Agent

| Task | Detail |
|------|--------|
| Sub-agent parallel spawning | Up to N concurrent Claude processes, each in own worktree, coordinator aggregates results. Pattern from DeerFlow SubagentExecutor (414 lines, verified real). |
| Agent domains + coordinator | Group presets into domains (code, quality, ops), coordinator routes tasks by analysis. |
| Frontend pagination | Add limit/offset to all list endpoints and UI. |

### Sprint 4 → v0.5.0 — Memory + Hooks

| Task | Detail |
|------|--------|
| Cross-session memory | Post-session LLM extraction of facts, confidence scoring, inject into future prompts. ETL pattern from DeerFlow + Mem0. |
| Hook system | Pre/post hooks on agent lifecycle, shell commands or agent invocations, HookRunner + events. |
| Polish | Shutdown timeout, Svelte 5 rune normalization, loading states. |

---

## What's Cut

These are NOT in scope until users demand them:

| Feature | Why Cut |
|---------|---------|
| WASM plugin runtime | MCP servers are the extension mechanism |
| Multi-LLM routing | Forge is Claude-first; no current user demand |
| Consensus protocols | Agents are independent; don't need agreement |
| RL/learning layer | No usage data to train on yet |
| 305-feature roadmap | Focus on ~20 features that matter |
| Notification system (20 features) | Webhooks (1 feature) covers 90% |
| Plugin marketplace | Need users first |
| Cron scheduler | Manual for now |
| Dev environment (code viewer, terminal) | Post-1.0 if ever |

---

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Rust + Svelte 5 single binary | Performance, no runtime deps, unique in space | Pre-project |
| SQLite WAL mode | Single-file, concurrent reads, no server | Pre-project |
| Ship existing prototype (don't rewrite) | 3K lines of working code > 44K lines of planning | 2026-02-26 |
| 4 sprints (not 7 phases) | Ship in weeks, not months | 2026-03-02 |
| Use rmcp for MCP | Official Rust SDK, `#[tool]` macro, maintained by protocol team | 2026-03-02 |
| Middleware chain pattern | Borrowed from DeerFlow (verified: 8 real middlewares, 1,089 LOC) | 2026-03-02 |
| Markdown-based skills | Borrowed from DeerFlow (verified: 15 real SKILL.md files, 208-line loader) | 2026-03-02 |
| Git worktree isolation | Industry standard for multi-Claude-Code (official support Feb 2026) | 2026-03-02 |
| Sub-agent parallelism | Borrowed from DeerFlow (verified: 414-line executor with real threadpool) | 2026-03-02 |
| Cut WASM plugins | No competitor uses them; MCP is the extension mechanism | 2026-02-26 |
| Freeze 00-08 planning dirs | Reference only; stop updating | 2026-02-26 |

---

## Session Protocol

### Before Starting Work
1. Read this file
2. Check `docs/SESSION_LOG.md` for recent sessions
3. Pick a task from the current sprint

### During Work
- One session = one focused deliverable
- Commit early, commit often
- Don't update frozen planning docs — write code

### When Done
1. Commit all changes
2. Log what was done in `docs/SESSION_LOG.md`
3. Update this file only if priorities changed

---

## File Map

```
forge-project/                    <-- Everything lives here
  crates/                         <-- 9 Rust crates (workspace)
    forge-core/                   Event types, EventBus, errors, IDs
    forge-agent/                  Agent model, 9 presets, validation
    forge-db/                     SQLite WAL, migrations, repos, BatchWriter
    forge-process/                Claude CLI spawn, stream-json parsing
    forge-safety/                 CircuitBreaker, RateLimiter
    forge-api/                    Axum HTTP + WebSocket + embedded frontend
    forge-app/                    Binary entry point, wiring, shutdown
    forge-mcp/                    MCP types (needs rewrite with rmcp)
    forge-mcp-bin/                MCP stdio server
  frontend/                       SvelteKit 5 + TailwindCSS 4
  migrations/                     0001_init.sql, 0002_add_cost.sql
  skills/                         (Sprint 2: Markdown skill files go here)
  scripts/                        e2e-smoke.sh
  .github/workflows/              ci.yml, release.yml
  docs/                           Current docs (audit, borrowed ideas, tasks)
    FORGE_AUDIT_2026_03_02.md     Full audit report
    BORROWED_IDEAS.md             DeerFlow + Claude-Flow + industry analysis
    agents/TASK_*.md              Historical agent task completions
    planning/                     Old planning docs (archived)
  00-vision/ through 08-reference/  Frozen reference material (don't update)
  NORTH_STAR.md                   YOU ARE HERE
  MASTER_TASK_LIST.md             Sprint tasks
  README.md                       GitHub landing page
```

---

## Reference Material

### In This Repo

| Resource | Location | Notes |
|----------|----------|-------|
| Full audit report | `docs/FORGE_AUDIT_2026_03_02.md` | Per-crate grades, gap analysis, proposal |
| Borrowed ideas | `docs/BORROWED_IDEAS.md` | DeerFlow, Claude-Flow, reference repos, industry research |

### Companion Repo: claude-parent

> **GitHub:** [mbaneshi/claude-parent](https://github.com/mbaneshi/claude-parent)
> **Local:** `/Users/bm/claude-parent/` (forge-project was split out of this repo on 2026-03-02)

| Resource | Location | Notes |
|----------|----------|-------|
| 61 reference repos | `refrence-repo/` (local) or [online](https://github.com/mbaneshi/claude-parent) | Top 4: hooks-observability, Workflow, ralph, infrastructure-showcase |
| Reference map | `reference-map/` | 13-category taxonomy of all 61 repos |
| Capability map | `docs/CLAUDE_CODE_CAPABILITY_MAP_AND_FORGE_ROADMAP.md` | Claude Code capabilities mapped to Forge features |
| AI doc landscape | `docs/ai-documentation-landscape.md` | AGENTS.md standard, industry state (2026) |

### External (Local)

| Resource | Location | Notes |
|----------|----------|-------|
| DeerFlow (verified) | `/Users/bm/cod/trend/26-feb/deer-flow` | ~10K real LOC Python, middleware + skills + memory + sub-agents |
| Claude-Flow (mixed) | `/Users/bm/cod/trend/26-feb/claude-flow` | ~60% real, study swarm coordinator + ADR pattern |
