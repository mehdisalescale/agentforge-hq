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

Derived from audit (2026-03-02) + enhancement proposal, merged into one plan. See `MASTER_TASK_LIST.md` for full task breakdown with What/Where/How/Verify.

### Sprint 1 → v0.2.0 — Fix + MCP + Ship

| Task | Detail |
|------|--------|
| Fix Dashboard null-safety (F1) | Check `outputBlocks.length` before accessing last element |
| Fix budget warning logic (F2) | `warn <= cost < limit` |
| Fix preset serialization (F3) | Proper serde instead of Debug format fallback |
| Rewrite MCP server with rmcp (M1-M5) | Official Rust MCP SDK, `#[tool]` macros, `forge --mcp` flag, compliance tests |
| Create CLAUDE.md (D1) | Project context file for AI/human sessions |
| Doc consolidation (D2) | Merge 50+ docs → ~15 active files |

### Sprint 2 → v0.3.0 — Worktrees + Middleware + Skills

| Task | Detail |
|------|--------|
| Git worktree isolation (WT1-WT3) | New `forge-git` crate, worktree per session, cleanup/merge UI. Industry-standard pattern, prerequisite for Sprint 3. |
| Middleware chain (MW1-MW2) | Refactor run handler into pipeline: RateLimit → CircuitBreaker → SkillInjection → Spawn → Persist → Cost. Pattern from DeerFlow (8 middlewares, 1,089 LOC). |
| Skill system (SK1-SK3) | Markdown skills with YAML frontmatter, loader at startup, seed 10-15 skills, keyword-matching injection. Pattern from DeerFlow (15 SKILL.md, 208-line loader). |
| Integration test (T1) | Happy path E2E: start server → create agent → run → stream → verify session. |

### Sprint 3 → v0.4.0 — Multi-Agent + Memory + Hooks

| Task | Detail |
|------|--------|
| Sub-agent parallel spawning (SA1-SA5) | Coordinator agent, up to N concurrent Claude processes in worktrees, result aggregation, per-sub-agent WebSocket events, multi-agent dashboard UI. Pattern from DeerFlow SubagentExecutor (414 lines). |
| Cross-session memory (ME1-ME4) | Memory table, post-session LLM extraction, confidence scoring, injection middleware, management UI. Pattern from DeerFlow + Mem0. |
| Hook system (HK1-HK3) | Hook table, pre/post lifecycle hooks, HookRunner, hook events, management UI. Pattern from hooks-mastery + hooks-observability. |
| Polish (SA6, P1-P3) | Frontend pagination, shutdown timeout, Svelte 5 rune normalization, loading states. |

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
