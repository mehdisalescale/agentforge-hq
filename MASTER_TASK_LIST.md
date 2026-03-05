# Claude Forge — Master Task List

> **Updated:** 2026-03-05 (v0.5.0 shipped, v0.6.0 planned)
> **Source:** `docs/FORGE_AUDIT_2026_03_02.md` (audit), `docs/RESEARCH_FINDINGS_2026_03_05.md` (67 repos)
> **Agent task cards:** `docs/agents/HANDOFF_SPRINT_2_3.md` (v0.4.0), `docs/agents/V060_WAVE_PROMPTS.md` (v0.6.0)

---

## How To Use This File

**Solo developer:** Pick tasks top-to-bottom within the current sprint/wave.

**Parallel agents:** Read `docs/agents/HANDOFF_SPRINT_2_3.md`. Launch agents per wave. Run verification gate between waves.

**Rules:**
1. Mark tasks `[x]` when done.
2. Run `cargo test --workspace && cargo clippy --workspace` after every task.
3. Commit after each task.

---

## Principles

1. **Ship small, ship often.** Three releases, each one usable.
2. **Code over docs.** Build features, not plans.
3. **Borrow proven patterns.** Middleware, skills, sub-agents — all verified in DeerFlow (~10K real LOC).
4. **Parallel by default.** Tasks with no shared files run simultaneously.

---

## Completed Work

<details>
<summary>Phase 0: Foundation — ALL DONE</summary>

- [x] S1-S8: BatchWriter wiring, EventBus capacity, frontend directory field, session status, error propagation, prompt validation, clippy, CORS

</details>

<details>
<summary>Phase A: Ship v0.1.0 — ALL DONE</summary>

- [x] A1-A9: rust-embed, graceful shutdown, TraceLayer, configurable host/port, E2E smoke, CI, release workflow, README, NORTH_STAR

</details>

<details>
<summary>Phase B: Safety — ALL DONE</summary>

- [x] B1-B6: Session status auto-update, markdown rendering, tool panels, circuit breaker, rate limiter, cost tracking

</details>

<details>
<summary>Bug Fixes F1-F3 — ALL DONE (Session 12)</summary>

- [x] F1: Dashboard null-safety (check outputBlocks.length before last access)
- [x] F2: Budget warning logic (check limit first, then warn)
- [x] F3: Preset serialization (serde_json instead of Debug format)

</details>

---

## SPRINT 1 → v0.2.0 — MCP + Ship (Sequential) — COMPLETE

> **Goal:** Rewrite MCP server with official SDK, consolidate docs, ship tagged release.
> **Status:** ALL DONE (Sessions 12-13). 94 tests pass, clippy clean.
> **Mode:** Sequential (one developer)

### MCP Rewrite

#### M1: Add rmcp dependency
- [x] **Done**

**What:** Replace hand-rolled JSON-RPC with official Rust MCP SDK.

**Where:** `Cargo.toml` (workspace), `crates/forge-mcp/Cargo.toml`

**How:**
1. Add `rmcp` to workspace dependencies
2. Research rmcp API: `#[tool]` macro, server builder, stdio transport
3. Decide: rewrite forge-mcp or replace forge-mcp-bin (likely replace forge-mcp-bin, keep forge-mcp as thin types)

**Verify:** `cargo check` passes with rmcp in dependency tree.

---

#### M2: Implement MCP tools with rmcp
- [x] **Done**

**What:** 10 tools using `#[tool]` macro.

| Tool | Description |
|------|-------------|
| `forge_agent_create` | Create agent (name, model, preset) |
| `forge_agent_list` | List all agents |
| `forge_agent_get` | Get agent by ID |
| `forge_agent_delete` | Delete agent |
| `forge_run` | Run agent (agent_id, prompt, directory) |
| `forge_session_list` | List sessions (optional agent_id filter) |
| `forge_session_get` | Get session details |
| `forge_session_export` | Export session (session_id, format) |
| `forge_config_get` | Get configuration |
| `forge_health` | Health check + uptime |

**Verify:** Each tool callable via MCP protocol over stdio.

---

#### M3: Implement MCP resources
- [x] **Done**

**What:** 5 resources: `forge://agents`, `forge://sessions`, `forge://config`, `forge://health`, `forge://skills`

**Verify:** `resources/list` returns all 5. `resources/read` returns valid JSON.

---

#### M4: MCP stdio server entry point
- [x] **Done**

**What:** `forge --mcp` flag starts MCP mode (stdio) instead of HTTP.

**Where:** `crates/forge-app/src/main.rs`

**Verify:** `echo '{"jsonrpc":"2.0","method":"initialize","id":1}' | ./forge --mcp` returns valid response.

---

#### M5: MCP protocol compliance tests
- [x] **Done**

**What:** Tests covering handshake, all tools, all resources, error handling.

**Verify:** `cargo test -p forge-mcp` passes with comprehensive coverage.

---

### Housekeeping

#### D1: Create CLAUDE.md
- [x] **Done**

**What:** Project context file for AI/human sessions. Under 200 lines.

---

#### D2: Doc consolidation
- [x] **Done**

**What:** Merge 35 frozen `00-08/` files → `docs/ORIGINAL_DESIGN_REFERENCE.md`. Merge 14 `docs/planning/` → `docs/PLANNING_ARCHIVE.md`. Delete 10 superseded docs. Result: ~15 active docs.

---

#### R1: Tag and ship v0.2.0
- [ ] **Done**

**Verify:** Download binary from GitHub Releases, run it, MCP mode works, HTTP mode works.

---

## SPRINTS 2-3 → v0.3.0 / v0.4.0 — Parallel Wave Execution

> **Goal:** Worktrees, middleware, skills, sub-agents, memory, hooks — all features remaining to reach orchestrator status.
> **Mode:** Parallel agents per wave. See `docs/agents/HANDOFF_SPRINT_2_3.md` for full task cards.
> **Status:** Wave 1 DONE (d6cd408). Wave 2 DONE (7084758). Wave 3 DONE (b802a27). Wave 4 NEXT.

### Wave Overview

```
WAVE 1 — 5 agents in parallel (all NEW files, zero conflicts)
  ├── Agent A: forge-git crate (WT1+WT2)
  ├── Agent B: Middleware trait + chain (MW1)
  ├── Agent C: Skill loader + 10 seed files (SK1+SK2)
  ├── Agent D: Memory table + repo + routes (ME1)
  └── Agent E: Hook table + repo + routes (HK1+HK2)
          │
          ▼ GATE: cargo test && cargo clippy && pnpm build
          │
WAVE 2 — 1 integration agent (wires shared files)
  └── Agent F: Migrations, state.rs, routes/mod.rs, run.rs, events.rs, main.rs
          │
          ▼ GATE: cargo test && cargo clippy && pnpm build
          │
WAVE 3 — 3 agents in parallel (depend on Wave 2)
  ├── Agent G: Middleware extraction + skill injection (MW2+SK3)
  ├── Agent H: Memory extraction + injection (ME2+ME3)
  └── Agent I: Sub-agent runner + coordinator (SA1-SA3)
          │
          ▼ GATE: cargo test && cargo clippy && pnpm build
          │
WAVE 4 — 4 agents in parallel (frontend + polish)
  ├── Agent J: Worktree UI + integration test (WT3+T1)
  ├── Agent K: Memory UI + Hook UI (ME4+HK3)
  ├── Agent L: Multi-agent dashboard + domains (SA4+SA5)
  └── Agent M: Polish (SA6+P1+P2+P3)
```

### Wave 1 — Build Components (5 parallel agents)

All agents create NEW files/crates. Zero shared files. Can run simultaneously.

#### Agent A: forge-git crate (WT1+WT2)
- [x] **Done** (Session 13 — 215 LOC, 7 tests)

**Exclusive files:** `crates/forge-git/` (NEW crate), `Cargo.toml` (add workspace member only)

**Delivers:**
- New crate `forge-git` with `create_worktree(repo_dir, session_id) → PathBuf`
- `remove_worktree(repo_dir, session_id)` for cleanup
- `list_worktrees(repo_dir) → Vec<WorktreeInfo>`
- Unit tests for create/remove/list

---

#### Agent B: Middleware trait + chain (MW1)
- [x] **Done** (Session 13 — 211 LOC, 3 tests)

**Exclusive files:** `crates/forge-api/src/middleware.rs` (NEW file)

**Delivers:**
- `Middleware` trait with `process(&self, ctx, next) → Result`
- `MiddlewareChain` struct with `add()` and `execute()`
- `RunContext` struct (agent, prompt, session_id, working_dir)
- `Next` type for chain progression
- Unit tests for chain execution order

**Pattern:** DeerFlow — 8 middlewares, 1,089 LOC. See `docs/BORROWED_IDEAS.md` §1.

---

#### Agent C: Skill loader + seed files (SK1+SK2)
- [x] **Done** (Session 13 — 10 skill files, +139 LOC skills.rs, 9 tests)

**Exclusive files:** `skills/` (NEW dir, 10+ `.md` files), `crates/forge-db/src/repos/skills.rs` (existing read-only stubs → add write + loader)

**Delivers:**
- YAML frontmatter parser (name, description, tags, tools)
- `load_skills_from_dir(dir) → Vec<Skill>` function
- `SkillRepo::upsert()` method
- 10 seed skill files: deep-research, code-review, refactor, test-writer, debug, security-audit, document, architect, explore, fix-bug
- Unit tests for parsing + loading

**Pattern:** DeerFlow — 15 SKILL.md files, 208-line loader. See `docs/BORROWED_IDEAS.md` §2.

---

#### Agent D: Memory table + repo + routes (ME1)
- [x] **Done** (Session 13 — 417 LOC repo, 76 LOC routes, migration, 8 tests)

**Exclusive files:** `crates/forge-db/src/repos/memory.rs` (NEW), `crates/forge-api/src/routes/memory.rs` (NEW), `migrations/0003_add_memory.sql` (NEW)

**Delivers:**
- Migration: `memory` table (id, category, content, confidence, source_session_id, created_at, updated_at)
- `MemoryRepo` with CRUD (create, list, get, update, delete, search)
- API routes: GET/POST `/api/v1/memory`, GET/PUT/DELETE `/api/v1/memory/:id`
- Unit tests for repo CRUD

---

#### Agent E: Hook table + repo + routes (HK1+HK2)
- [x] **Done** (Session 13 — 519 LOC repo, 71 LOC routes, migration, 10+ tests)

**Exclusive files:** `crates/forge-db/src/repos/hooks.rs` (NEW), `crates/forge-api/src/routes/hooks.rs` (NEW), `migrations/0004_add_hooks.sql` (NEW)

**Delivers:**
- Migration: `hooks` table (id, name, event_type, timing pre/post, command, enabled, created_at)
- `HookRepo` with CRUD
- `HookRunner` struct with `run_hooks(event_type, timing) → Result`
- API routes: GET/POST `/api/v1/hooks`, GET/PUT/DELETE `/api/v1/hooks/:id`
- Unit tests for repo + runner

**Pattern:** hooks-mastery (13 types), hooks-observability (event interception). See `docs/BORROWED_IDEAS.md` §6.

---

### ⬛ GATE 1: Verify Wave 1

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

All 5 agents' work must compile and pass tests before Wave 2.

---

### Wave 2 — Integration Wiring (1 agent, sequential) — DONE

Touches shared files. Must run alone after Wave 1 gate passes.

#### Agent F: Integration wiring
- [x] **Done** (Session 13-14 — wired migrations 0003/0004, repos, state, routes, 7 new event variants, skill loading)

**Shared files:** `forge-db/src/migrations.rs`, `forge-db/src/repos/mod.rs`, `forge-db/src/lib.rs`, `forge-api/src/state.rs`, `forge-api/src/routes/mod.rs`, `forge-api/src/routes/run.rs`, `forge-api/src/lib.rs`, `forge-core/src/events.rs`, `forge-app/src/main.rs`

**Delivers:**
- Apply migrations 0003 (memory) + 0004 (hooks) in `migrations.rs`
- Export MemoryRepo + HookRepo in `repos/mod.rs` and `lib.rs`
- Add `memory_repo` + `hook_repo` + `worktree_manager` to AppState
- Nest memory + hook routes in `routes/mod.rs`
- Add worktree path extraction in `run.rs`
- Add middleware chain initialization in `lib.rs`
- Add event types to `events.rs`: `HookStarted`, `HookCompleted`, `HookFailed`, `SubAgentRequested`, `SubAgentStarted`, `SubAgentCompleted`, `SubAgentFailed`
- Wire skill loader at startup in `main.rs`
- All tests pass, all routes resolve

---

### ⬛ GATE 2: Verify Wave 2

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

---

### Wave 3 — Feature Logic (3 parallel agents) — DONE

Depends on Wave 2. Each agent owns distinct files.

#### Agent G: Middleware extraction + skill injection (MW2+SK3)
- [x] **Done** (Session 14 — 862 LOC middleware.rs with 6 middlewares, 14 tests. run.rs refactored to 136 lines)

**Files:** `crates/forge-api/src/middleware.rs` (extend), `crates/forge-api/src/routes/run.rs` (refactor)

**Delivers:**
- Extract run.rs inline logic into 6 middlewares: RateLimitMiddleware, CircuitBreakerMiddleware, SkillInjectionMiddleware, SpawnMiddleware, PersistMiddleware, CostMiddleware
- Skill injection: keyword-match prompt against skill tags, append skill body to system prompt
- `run.rs` becomes thin: parse request → build RunContext → execute chain → return response
- All existing tests pass + new middleware tests

---

#### Agent H: Memory extraction + injection (ME2+ME3)
- [x] **Done** (Session 14 — 700 LOC memory.rs, extract_facts + store_extracted + inject_context, 8 new tests)

**Files:** `crates/forge-db/src/repos/memory.rs` (extend)

**Delivers:**
- Post-session extraction: after session completes, send transcript to Claude with extraction prompt, parse structured facts, store with confidence scores
- Memory injection: on new run, query relevant memories by category/keyword, format as context block, prepend to system prompt
- Tests for extraction parsing + injection formatting

**Pattern:** DeerFlow MemoryUpdater — 319 lines. See `docs/BORROWED_IDEAS.md` §5.

---

#### Agent I: Sub-agent runner + coordinator (SA1-SA3)
- [x] **Done** (Session 14 — 307 LOC concurrent.rs, ConcurrentRunner + Coordinator preset, 3 tests)

**Files:** `crates/forge-process/src/concurrent.rs` (NEW), `crates/forge-agent/src/lib.rs` (add Coordinator preset)

**Delivers:**
- `ConcurrentRunner` wrapping N `ProcessRunner` instances, each in own worktree
- Configurable concurrency limit (default 3), uses `tokio::JoinSet`
- New `Coordinator` agent preset with task decomposition system prompt
- Result aggregation: collect sub-agent outputs, synthesize final response
- Tests for concurrent spawn + aggregation

**Pattern:** DeerFlow SubagentExecutor — 414 lines. See `docs/BORROWED_IDEAS.md` §4.

---

### ⬛ GATE 3: Verify Wave 3

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

### Wave 4 — Frontend + Polish (4 parallel agents) — DONE

All frontend work. No backend file conflicts between agents.

#### Agent J: Worktree UI + integration test (WT3+T1)
- [x] **Done** (Session 14 — worktree badges, $state runes, integration_test.sh)

**Files:** `frontend/src/routes/sessions/` (modify), `tests/` (NEW integration test)

**Delivers:**
- Session detail page shows worktree branch name, status
- Merge button: merge worktree branch back to main
- Delete button: cleanup worktree + branch
- Integration test: start server → create agent → run → stream → verify session + events

---

#### Agent K: Memory UI + Hook UI (ME4+HK3)
- [x] **Done** (Session 14 — memory + hooks CRUD pages, API functions, nav links)

**Files:** `frontend/src/routes/memory/` (NEW), `frontend/src/routes/hooks/` (NEW)

**Delivers:**
- Memory page: list facts, edit content/confidence, delete, search
- Hook page: list hooks, create new, enable/disable toggle, delete
- Both use `$state` runes consistently

---

#### Agent L: Multi-agent dashboard + domains (SA4+SA5)
- [x] **Done** (Session 14 — sub-agent panel, $state runes, statusbar update)

**Files:** `frontend/src/routes/+page.svelte` (modify), `frontend/src/lib/api.ts` (extend)

**Delivers:**
- Per-sub-agent progress panels when coordinator is running
- Status indicators: pending/running/completed/failed per sub-agent
- Agent domain badges (code/quality/ops) in agent list
- WebSocket event handling for SubAgent* events

---

#### Agent M: Polish (SA6+P1+P2+P3)
- [x] **Done** (Session 14 — skills tags/filter, workflows diagram, settings dashboard, Coordinator + domain badges)

**Files:** `frontend/` (various pages, no overlap with J/K/L)

**Delivers:**
- Pagination (limit/offset) on agents, sessions, skills list pages
- Shutdown timeout (10s default) in `forge-app/src/main.rs`
- Svelte 5 `$state` rune normalization across all pages
- Loading spinners during API calls on all forms

---

### ⬛ GATE 4: Final Verification

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
# Run E2E smoke test
./scripts/e2e-smoke.sh
```

### R2: Tag and ship v0.3.0 + v0.4.0
- [x] **Done** (Session 14 — tagged v0.4.0)

Tag after Wave 2 gate (v0.3.0) or after Wave 4 gate (v0.4.0), depending on desired release cadence.

---

## Timeline (Parallel Execution)

```
Week 1:  Sprint 1 — MCP rewrite (M1-M5) + D1 + D2 → ship v0.2.0
Week 2:  Wave 1 (5 agents) + Wave 2 (1 agent)
Week 3:  Wave 3 (3 agents) + Wave 4 (4 agents)         → ship v0.4.0
```

**~3 weeks parallel vs ~5-7 weeks sequential.**

---

## v0.5.0 — Scheduler, Analytics, Loop Detection — COMPLETE

> **Status:** ALL DONE. Tagged v0.5.0 (2026-03-05). 150 tests pass.
> Committed as f9de905 (+2,257 lines), docs as c866ee1.

- [x] Cron scheduler (ScheduleRepo, 468 LOC, 10 tests, background tick, CRUD API + UI)
- [x] Usage analytics (AnalyticsRepo, 297 LOC, 7 tests, P90, projected monthly, dashboard)
- [x] Loop detection (LoopDetector, 201 LOC, 10 tests, exit gate config)
- [x] Quality gate + exit gate middleware variants (3 tests)
- [x] Session HTML export
- [x] 9 new ForgeEvent variants (35 total)

---

## v0.6.0 — Best-of-N, Pipelines, Swim Lanes — PLANNED

> **Goal:** Make agents smarter (best-of-N, context pruning, typed memory) and workflows real (pipeline engine, swim-lane dashboard).
> **Plan:** `docs/V060_SPRINT_PLAN.md`
> **Agent prompts:** `docs/agents/V060_WAVE_PROMPTS.md`
> **Mode:** 7 agents across 3 waves (parallel execution)

### Wave 5 — New Components (4 parallel agents)

- [ ] **Agent N:** Best-of-N selection mode — `best_of_n.rs`, `strategy.rs`
- [ ] **Agent O:** Context pruner + memory compaction — `context_pruner.rs`, `compaction.rs`, migration 0006
- [ ] **Agent P:** Pipeline engine + WorkflowRepo CRUD — `pipeline.rs`, extend `workflows.rs`, migration 0007
- [ ] **Agent R:** OpenAPI auto-docs — `openapi.rs` (utoipa + Scalar UI)

### Wave 6 — Integration Wiring (1 agent, sequential)

- [ ] **Agent S:** Wire Wave 5 outputs — migrations, exports, AppState, routes, events

### Wave 7 — Smart Features + UI (2 parallel agents)

- [ ] **Agent U:** Three-type memory + auto-activating skills — extend `memory.rs`, `skills.rs`, migration 0008
- [ ] **Agent W:** Swim-lane dashboard + pipeline builder UI — extend `+page.svelte`, rewrite `workflows/+page.svelte`

---

## PARKED (Not in scope)

| Feature | Why Parked |
|---------|------------|
| WASM plugin runtime | MCP is the extension mechanism |
| Multi-LLM routing | Claude-only by design |
| Consensus protocols | Agents are independent |
| RL/learning layer | No usage data yet |
| Plugin marketplace | Need users first |
| ~~Cron scheduler~~ | **Shipped in v0.5.0** |
| Dev environment | Post-1.0 if ever |
| Authentication | Add when deploying remotely |
| Audit log + permissions | Post-1.0 |
| Harvester integration | Deferred to post-Sprint 2. See `docs/HARVESTER_INTEGRATION.md` |

---

## Success Criteria

| Release | Key Metric |
|---------|------------|
| v0.2.0 | MCP server passes compliance test, docs consolidated to ~15 |
| v0.3.0 | Worktree isolation works, middleware chain active, skills loaded |
| v0.4.0 | 3 sub-agents run in parallel worktrees, memory persists, hooks fire |
| v0.5.0 | Cron scheduler runs, analytics dashboard shows costs, loop detection works |
| v0.6.0 | Best-of-N selection, pipeline engine executes, swim-lane dashboard renders, 175+ tests |

---

## Post-Task Verification

Run after every task:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
# If frontend was touched:
cd frontend && pnpm build && cd ..
```
