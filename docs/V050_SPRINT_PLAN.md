# v0.5.0 Sprint Plan — Quality, Pipelines, Observability

> **Created:** 2026-03-05
> **Based on:** `docs/RESEARCH_FINDINGS_2026_03_05.md`
> **Predecessor:** v0.4.0 (13 agents, 4 waves, 118 tests, 10 pages)
> **Goal:** Make agents smarter, sessions longer, workflows real, and the dashboard useful at scale.

---

## Sprint Theme

v0.4.0 built the platform. v0.5.0 makes it **good**:
- Agents produce better output (best-of-N, quality gates)
- Sessions survive long runs (context pruning, memory compaction)
- Workflows do real things (sequential + fanout pipelines, cron)
- Dashboard shows what's happening (swim lanes, predictive budgets)
- API is self-documenting (OpenAPI)

---

## Wave Structure

```
WAVE 5 — 5 agents in parallel (all NEW files/modules, zero conflicts)
  ├── Agent N: Best-of-N selection mode for ConcurrentRunner
  ├── Agent O: Context pruner + memory compaction middleware
  ├── Agent P: Sequential + Fanout pipeline engine
  ├── Agent Q: Cron scheduler (table + background task + API)
  └── Agent R: OpenAPI auto-docs (utoipa + Scalar UI)
          │
          ▼ GATE 5: cargo test && cargo clippy && pnpm build
          │
WAVE 6 — 1 integration agent (wires shared files)
  └── Agent S: Wire Wave 5 into app (state, routes, middleware chain, main.rs)
          │
          ▼ GATE 6: cargo test && cargo clippy && pnpm build
          │
WAVE 7 — 4 agents in parallel (depend on Wave 6)
  ├── Agent T: Quality gates (critic-fixer loop middleware)
  ├── Agent U: Three-type memory + auto-activating skills
  ├── Agent V: Predictive usage budgeting + cost forecast API
  └── Agent W: Swim-lane dashboard + pipeline UI
          │
          ▼ GATE 7: cargo test && cargo clippy && pnpm build → tag v0.5.0
```

**10 agents across 3 waves. ~2-3 days parallel execution.**

---

## Wave 5 — New Components (5 parallel agents)

All agents create NEW files or isolated modules. Zero shared file conflicts.

---

### Agent N: Best-of-N Selection Mode

**What:** Add a `BestOfN` execution mode to ConcurrentRunner. Spawn N sub-agents with different strategy prompts for the same task. A selector sub-agent compares all outputs and picks the best.

**Exclusive files:**
- `crates/forge-process/src/best_of_n.rs` (NEW)
- `crates/forge-agent/src/strategy.rs` (NEW — strategy prompt variants)

**Delivers:**
1. `BestOfNRunner` struct wrapping `ConcurrentRunner`
2. `StrategySet` — 3 built-in strategies: "minimal changes", "modular refactor", "thorough with tests"
3. `SelectionResult` — chosen implementation ID, reason, improvements extracted from runners-up
4. Selector prompt template that compares N unified diffs
5. Integration with existing `SubAgent*` events (SubAgentRequested/Started/Completed per strategy)
6. Tests: best_of_n_selects_best, best_of_n_respects_concurrency, strategy_set_generates_variants

**Pattern reference:** codebuff `agents/editor/best-of-n/`

**Verify:**
```bash
cargo test -p forge-process -- best_of_n
cargo clippy -p forge-process
```

---

### Agent O: Context Pruner + Memory Compaction Middleware

**What:** Two middleware components that manage context window size: (1) a context pruner that summarizes tool calls and truncates long text before each sub-agent invocation, (2) a memory compactor that auto-summarizes old messages when token count exceeds threshold.

**Exclusive files:**
- `crates/forge-process/src/context_pruner.rs` (NEW)
- `crates/forge-db/src/repos/compaction.rs` (NEW)

**Delivers:**
1. `ContextPruner` struct with:
   - `summarize_tool_call(tool_name, args, result) → String` — one-liner summaries
   - `truncate_text(text, max_tokens) → String` — 80/20 head/tail split
   - `estimate_tokens(text) → usize` — ~3 chars/token heuristic
   - `prune_history(messages, max_tokens) → Vec<Message>` — apply all rules
2. `MemoryCompactor` struct with:
   - `should_compact(session_id) → bool` — check token threshold (configurable, default 80K)
   - `compact(session_id) → CompactionResult` — summarize old messages, store summary, replace originals
   - `store_compaction(session_id, summary)` — write to `compactions` table
3. Migration: `compactions` table (id, session_id, summary, original_token_count, compacted_token_count, created_at)
4. Tests: truncation preserves boundaries, compaction threshold triggers, token estimation accuracy

**Pattern reference:** codebuff `context-pruner.ts` + ReMe `reme_copaw.py`

**Verify:**
```bash
cargo test -p forge-process -- context_pruner
cargo test -p forge-db -- compaction
cargo clippy --workspace
```

---

### Agent P: Sequential + Fanout Pipeline Engine

**What:** First-class pipeline primitives that make the Workflows page functional. A `Pipeline` is a DAG of steps, each step is either a single agent run or a fanout to N agents.

**Exclusive files:**
- `crates/forge-process/src/pipeline.rs` (NEW)
- `crates/forge-db/src/repos/workflows.rs` (extend existing stubs)
- `migrations/0005_add_workflows.sql` (NEW)

**Delivers:**
1. `Pipeline` struct: ordered list of `PipelineStep`
2. `PipelineStep` enum: `Sequential(agent_id, prompt_template)` | `Fanout(Vec<agent_id>, prompt_template)` | `BestOfN(agent_id, n, strategies)`
3. `PipelineRunner` that executes steps in order, passing output of step N as input to step N+1
4. `WorkflowRepo` with CRUD for workflow definitions (stored as JSON pipeline in SQLite)
5. Migration: `workflows` table (id, name, description, pipeline_json, created_at, updated_at)
6. API routes: GET/POST `/api/v1/workflows`, GET/PUT/DELETE `/api/v1/workflows/:id`, POST `/api/v1/workflows/:id/run`
7. Tests: sequential_passes_output, fanout_collects_all, pipeline_stops_on_error

**Pattern reference:** agentscope `pipeline/_functional.py`

**Verify:**
```bash
cargo test -p forge-process -- pipeline
cargo test -p forge-db -- workflow
cargo clippy --workspace
```

---

### Agent Q: Cron Scheduler

**What:** Schedule agent runs with cron expressions. Background tokio task checks every 60 seconds and fires matching jobs through the existing run infrastructure.

**Exclusive files:**
- `crates/forge-process/src/scheduler.rs` (NEW)
- `crates/forge-db/src/repos/schedules.rs` (NEW)
- `migrations/0006_add_schedules.sql` (NEW)

**Delivers:**
1. `ScheduleRepo` with CRUD: create, list, get, update, delete, list_due
2. Migration: `schedules` table (id, name, agent_id, prompt, cron_expr, enabled, last_run_at, next_run_at, created_at)
3. `Scheduler` struct with `start(interval: Duration)` → spawns tokio background task
4. `check_and_fire()` — query due schedules, create session, fire through existing `ProcessRunner`
5. Cron parsing via `cron` crate (lightweight, no-std compatible)
6. API routes: GET/POST `/api/v1/schedules`, GET/PUT/DELETE `/api/v1/schedules/:id`
7. Tests: cron_parsing, next_run_calculation, scheduler_fires_due_jobs

**Pattern reference:** reference-map `claude-code-viewer`

**Verify:**
```bash
cargo test -p forge-process -- scheduler
cargo test -p forge-db -- schedule
cargo clippy --workspace
```

---

### Agent R: OpenAPI Auto-Docs

**What:** Auto-generate OpenAPI 3.1 spec from Axum routes using `utoipa`. Serve Scalar UI at `/docs`.

**Exclusive files:**
- `crates/forge-api/src/openapi.rs` (NEW)

**Delivers:**
1. Add `utoipa` + `utoipa-scalar` to forge-api dependencies
2. `ApiDoc` struct implementing `utoipa::OpenApi` with all existing routes annotated
3. Scalar UI served at `/docs` (interactive API explorer)
4. JSON spec at `/api/openapi.json`
5. `#[utoipa::path]` annotations on all route handlers in `routes/*.rs`
6. Request/response types annotated with `#[derive(ToSchema)]`
7. Test: openapi_spec_is_valid

**Pattern reference:** reference-map `claude-code-hub`

**Verify:**
```bash
cargo test -p forge-api -- openapi
cargo clippy -p forge-api
# Manual: open http://127.0.0.1:4173/docs and verify routes render
```

---

## GATE 5: Verify Wave 5

```bash
cargo build --workspace
cargo test --workspace    # should be ~140+ tests
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

All 5 agents' work must compile and pass before Wave 6.

---

## Wave 6 — Integration Wiring (1 agent, sequential)

Touches shared files. Must run alone.

---

### Agent S: Integration Wiring

**Shared files:**
- `Cargo.toml` — add `cron` dependency
- `crates/forge-db/src/migrations.rs` — apply migrations 0005 + 0006
- `crates/forge-db/src/repos/mod.rs` — export WorkflowRepo, ScheduleRepo, CompactionRepo
- `crates/forge-db/src/lib.rs` — re-export new repos
- `crates/forge-api/src/state.rs` — add workflow_repo, schedule_repo, compaction_repo to AppState
- `crates/forge-api/src/routes/mod.rs` — nest workflow, schedule routes + OpenAPI
- `crates/forge-api/src/lib.rs` — mount Scalar UI, add OpenAPI json route
- `crates/forge-api/src/middleware.rs` — add ContextPrunerMiddleware to chain (7th middleware)
- `crates/forge-core/src/events.rs` — add events: WorkflowStarted, WorkflowStepCompleted, WorkflowCompleted, ScheduleFired, CompactionCompleted
- `crates/forge-app/src/main.rs` — start Scheduler background task, init new repos

**Delivers:**
1. All Wave 5 modules wired and accessible via HTTP API
2. Scheduler starts automatically on app boot
3. Context pruner runs as 7th middleware in chain
4. OpenAPI UI accessible at `/docs`
5. All existing 118+ tests still pass + new integration points work
6. 5 new event types flowing through EventBus

**Verify:**
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

---

## GATE 6: Verify Wave 6

Same as above. All routes resolve, scheduler starts, OpenAPI renders.

---

## Wave 7 — Quality + UI (4 parallel agents)

Depends on Wave 6. Each agent owns distinct files.

---

### Agent T: Quality Gates (Critic-Fixer Loop)

**What:** Post-execution quality validation. A critic agent scores output; if below threshold, a fixer re-runs with feedback. Configurable per workflow step.

**Files:**
- `crates/forge-process/src/quality_gate.rs` (NEW)
- `crates/forge-api/src/middleware.rs` (extend — add QualityGateMiddleware as 8th)

**Delivers:**
1. `QualityGate` struct: `threshold: u8` (0-100), `max_iterations: u8`, `critic_agent_id: AgentId`
2. `QualityResult`: score, feedback, iteration_count, passed
3. `QualityGateMiddleware` — optionally wraps SpawnMiddleware. If gate configured on agent, runs critic after spawn completes.
4. Critic prompt template: receives agent output, scores on completion (0-40), correctness (0-30), quality (0-30)
5. Fixer re-invocation: if score < threshold, re-run original agent with critic feedback prepended
6. Events: QualityCheckStarted, QualityCheckPassed, QualityCheckFailed
7. Tests: gate_passes_above_threshold, gate_retries_below_threshold, gate_stops_at_max_iterations

**Pattern reference:** reference-map `claude-code-my-workflow` + codebuff `evals/buffbench/judge.ts`

**Verify:**
```bash
cargo test -p forge-process -- quality_gate
cargo clippy --workspace
```

---

### Agent U: Three-Type Memory + Auto-Activating Skills

**What:** Split memory into 3 types with type-specific retrieval. Add skill auto-activation based on working directory and prompt keywords.

**Files:**
- `crates/forge-db/src/repos/memory.rs` (extend — add memory_type column)
- `crates/forge-db/src/repos/skills.rs` (extend — add skill_rules)
- `migrations/0007_memory_types_and_skill_rules.sql` (NEW)

**Delivers:**
1. Migration: add `memory_type` column (enum: personal/task/tool, default personal) to `memory` table
2. Migration: add `skill_rules` table (id, skill_id, trigger_type, trigger_pattern, enabled)
3. `MemoryRepo::search_by_type(memory_type, query)` — type-specific retrieval
4. `MemoryRepo::extract_typed(transcript, session_metadata)` — extract into correct type based on content classification
5. `SkillRepo::find_matching_rules(working_dir, prompt)` — scan for file patterns + keyword triggers
6. `auto_activate_skills(working_dir, prompt) → Vec<Skill>` — returns skills whose rules match
7. Update memory injection middleware to retrieve by type with priority weighting (task > tool > personal)
8. Tests: memory_type_filtering, skill_rule_matching, auto_activation_detects_cargo_toml

**Pattern reference:** ReMe `memory/vector_based/` + reference-map `claude-code-infrastructure-showcase`

**Verify:**
```bash
cargo test -p forge-db -- memory
cargo test -p forge-db -- skill
cargo clippy --workspace
```

---

### Agent V: Predictive Usage Budgeting

**What:** Rolling-window analytics over existing events table. P90 projections, burn-down estimates, forecast API endpoint.

**Files:**
- `crates/forge-safety/src/forecast.rs` (NEW)
- `crates/forge-api/src/routes/usage.rs` (NEW)

**Delivers:**
1. `UsageForecast` struct: current_spend, p90_daily_rate, projected_monthly, days_until_limit, recommendation
2. `ForecastEngine::calculate(events, budget_limit) → UsageForecast` — rolling 7-day window, P90 percentile
3. `ForecastEngine::burn_down(events, budget_limit) → Vec<BurnDownPoint>` — daily projected balance
4. API route: GET `/api/v1/usage/forecast` — returns forecast JSON
5. API route: GET `/api/v1/usage/history` — returns daily usage aggregates
6. Tests: forecast_with_steady_usage, forecast_with_spike, burn_down_projects_correctly

**Pattern reference:** reference-map `Claude-Code-Usage-Monitor`

**Verify:**
```bash
cargo test -p forge-safety -- forecast
cargo test -p forge-api -- usage
cargo clippy --workspace
```

---

### Agent W: Swim-Lane Dashboard + Pipeline UI

**What:** Frontend: (1) swim-lane visualization for parallel sub-agents, (2) pipeline builder/viewer for workflows page.

**Files:**
- `frontend/src/routes/+page.svelte` (extend — swim lanes)
- `frontend/src/routes/workflows/+page.svelte` (rewrite — pipeline builder)
- `frontend/src/lib/api.ts` (extend — workflow CRUD, usage forecast, schedule CRUD)
- `frontend/src/routes/schedules/+page.svelte` (NEW)

**Delivers:**
1. **Swim lanes**: Replace flat event log with column-per-agent layout. Each column shows events as colored blocks. Status colors: blue=running, green=done, red=failed, gray=pending. Tool emoji indicators.
2. **Pipeline builder**: Visual step editor. Add sequential/fanout/best-of-N steps. Connect steps with arrows. Drag to reorder. Run button executes pipeline.
3. **Schedule page**: List scheduled jobs, create/edit with cron expression input, enable/disable toggle, last/next run display.
4. **Usage forecast widget**: Burn-down chart on dashboard sidebar. Shows projected spend, days until limit.
5. API client functions: `listWorkflows`, `createWorkflow`, `updateWorkflow`, `deleteWorkflow`, `runWorkflow`, `getUsageForecast`, `listSchedules`, `createSchedule`, `updateSchedule`, `deleteSchedule`
6. All components use `$state` runes

**Verify:**
```bash
cd frontend && pnpm build && cd ..
# Manual: verify swim lanes, pipeline builder, schedule page, forecast widget
```

---

## GATE 7: Final Verification

```bash
cargo build --workspace
cargo test --workspace    # target: ~160+ tests
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
./scripts/e2e-smoke.sh
```

---

## Tag v0.5.0

After Gate 7 passes:
```bash
git tag -a v0.5.0 -m "v0.5.0: quality gates, pipelines, cron, swim lanes, OpenAPI"
git push origin v0.5.0
```

---

## Dependency Graph

```
                 Wave 5 (parallel)
    ┌────┬────┬────┬────┬────┐
    N    O    P    Q    R
    │    │    │    │    │
    └────┴────┴────┴────┘
              │
         Wave 6 (sequential)
              S
              │
         Wave 7 (parallel)
    ┌────┬────┬────┐
    T    U    V    W
    │    │    │    │
    └────┴────┴────┘
              │
          v0.5.0
```

---

## File Ownership Matrix (Wave 5)

No conflicts — all agents work on NEW files:

| File | N | O | P | Q | R |
|------|---|---|---|---|---|
| `forge-process/src/best_of_n.rs` | OWN | | | | |
| `forge-agent/src/strategy.rs` | OWN | | | | |
| `forge-process/src/context_pruner.rs` | | OWN | | | |
| `forge-db/src/repos/compaction.rs` | | OWN | | | |
| `forge-process/src/pipeline.rs` | | | OWN | | |
| `forge-db/src/repos/workflows.rs` | | | OWN | | |
| `migrations/0005_add_workflows.sql` | | | OWN | | |
| `forge-process/src/scheduler.rs` | | | | OWN | |
| `forge-db/src/repos/schedules.rs` | | | | OWN | |
| `migrations/0006_add_schedules.sql` | | | | OWN | |
| `forge-api/src/openapi.rs` | | | | | OWN |

## File Ownership Matrix (Wave 7)

| File | T | U | V | W |
|------|---|---|---|---|
| `forge-process/src/quality_gate.rs` | OWN | | | |
| `forge-api/src/middleware.rs` | EXT | | | |
| `forge-db/src/repos/memory.rs` | | EXT | | |
| `forge-db/src/repos/skills.rs` | | EXT | | |
| `migrations/0007_*` | | OWN | | |
| `forge-safety/src/forecast.rs` | | | OWN | |
| `forge-api/src/routes/usage.rs` | | | OWN | |
| `frontend/src/routes/+page.svelte` | | | | OWN |
| `frontend/src/routes/workflows/+page.svelte` | | | | OWN |
| `frontend/src/routes/schedules/+page.svelte` | | | | OWN |
| `frontend/src/lib/api.ts` | | | | EXT |

OWN = creates/owns, EXT = extends existing (no conflict with other agents in same wave)

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Tests | 160+ (up from 118) |
| Crates | 9 (no new crates, new modules within existing) |
| Frontend pages | 12 (add Schedules, upgrade Workflows) |
| Middlewares | 8 (add ContextPruner + QualityGate) |
| New DB tables | 3 (compactions, workflows, schedules) |
| Event types | 32+ (up from 27) |

---

## What's Deferred to v0.6.0+

| Feature | Why Deferred |
|---------|-------------|
| Conversation rewind/branching | Schema-heavy, needs design |
| Multi-provider model routing | Breaks Claude-only assumption |
| Generator-based step control | Needs Rust async state machine trait design |
| OpenTelemetry tracing | Production ops, not user-facing |
| Hybrid vector + BM25 search | Needs vector extension evaluation |
| Plugin SDK | MCP covers extension for now |
| Hierarchical agent delegation | Flat ConcurrentRunner is sufficient |
| Session transcript export | Nice but not core |
| Kanban session view | Nice but not core |
