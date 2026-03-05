# v0.6.0 Sprint Plan — Best-of-N, Pipelines, Swim Lanes

> **Created:** 2026-03-05
> **Supersedes:** `docs/V050_SPRINT_PLAN.md` (partially shipped as v0.5.0)
> **Predecessor:** v0.5.0 (150 tests, 12 pages, scheduler, analytics, loop detection, quality gates)
> **Based on:** `docs/RESEARCH_FINDINGS_2026_03_05.md` (67 repos analyzed)

---

## What Shipped in v0.5.0 (remove from plan)

These were planned but already delivered:
- ~~Agent Q: Cron scheduler~~ — ScheduleRepo (468 LOC, 10 tests), background tick, CRUD API + UI
- ~~Agent V: Predictive usage budgeting~~ — AnalyticsRepo (297 LOC, 7 tests), P90, projected monthly, dashboard
- ~~Loop detection~~ — LoopDetector (201 LOC, 10 tests), ExitGateConfig
- ~~Quality gate middleware~~ — MiddlewareError variants, QualityCritic trait, 3 tests

## What Remains

7 agents across 3 waves (down from 10).

---

## Sprint Theme

v0.5.0 added scheduling + analytics. v0.6.0 makes agents **smarter** and workflows **real**:
- Best-of-N: parallel competing approaches → best result
- Context pruning: sessions survive long runs
- Pipelines: sequential + fanout workflows with real execution
- Swim lanes: see what 5+ parallel agents are doing
- OpenAPI: self-documenting API
- Typed memory + auto-skills: smarter context injection

---

## Wave Structure

```
WAVE 5 — 4 agents in parallel (all NEW files, zero conflicts)
  ├── Agent N: Best-of-N selection mode for ConcurrentRunner
  ├── Agent O: Context pruner + memory compaction
  ├── Agent P: Pipeline engine + WorkflowRepo CRUD
  └── Agent R: OpenAPI auto-docs (utoipa + Scalar UI)
          │
          ▼ GATE 5: cargo test && cargo clippy && pnpm build
          │
WAVE 6 — 1 integration agent (wires shared files)
  └── Agent S: Wire Wave 5 into app
          │
          ▼ GATE 6: cargo test && cargo clippy && pnpm build
          │
WAVE 7 — 2 agents in parallel (backend + frontend)
  ├── Agent U: Three-type memory + auto-activating skills
  └── Agent W: Swim-lane dashboard + pipeline builder UI
          │
          ▼ GATE 7: cargo test && cargo clippy && pnpm build → tag v0.6.0
```

**7 agents across 3 waves. ~1-2 days parallel execution.**

---

## Wave 5 — New Components (4 parallel agents)

All agents create NEW files. Zero shared file conflicts.

---

### Agent N: Best-of-N Selection Mode

**What:** Add a `BestOfN` execution mode to ConcurrentRunner. Spawn N sub-agents with different strategy prompts for the same task. A selector sub-agent compares all outputs and picks the best.

**Exclusive files:**
- `crates/forge-process/src/best_of_n.rs` (NEW)
- `crates/forge-agent/src/strategy.rs` (NEW)

**Delivers:**
1. `Strategy` struct: `name: String`, `system_prompt_suffix: String`
2. `StrategySet` with 3 built-in strategies: "minimal changes", "modular refactor", "thorough with tests"
3. `BestOfNRunner` struct wrapping `ConcurrentRunner` — fans out same prompt to N sub-agents, each with a different strategy suffix appended to system prompt
4. `SelectionResult`: `chosen_index: usize`, `reason: String`, `improvements: Vec<String>`
5. `select_best(results: &[SubTaskResult]) -> SelectionResult` — compares outputs by length, exit code, keyword heuristics (tests found, error count)
6. Integration with existing `SubAgent*` events
7. Tests: `best_of_n_returns_all_results`, `select_best_prefers_success`, `strategy_set_has_three`, `best_of_n_respects_concurrency`

**Context — existing API:**
```rust
// ConcurrentRunner in concurrent.rs
pub struct SubTask { pub agent_id: AgentId, pub prompt: String, pub working_dir: String }
pub struct SubTaskResult { pub agent_id: AgentId, pub session_id: SessionId, pub output: String, pub exit_code: i32, pub success: bool }
pub struct ConcurrentRunner { /* event_bus, max_concurrent, spawn_config */ }
impl ConcurrentRunner {
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self;
    pub async fn run_all(&self, parent_session_id: &SessionId, tasks: Vec<SubTask>) -> Vec<SubTaskResult>;
}
```

**Verify:**
```bash
cargo test -p forge-process -- best_of_n
cargo clippy -p forge-process
```

---

### Agent O: Context Pruner + Memory Compaction

**What:** Two modules that manage context size: (1) context pruner summarizes tool calls and truncates long text, (2) memory compactor auto-summarizes when token count exceeds threshold.

**Exclusive files:**
- `crates/forge-process/src/context_pruner.rs` (NEW)
- `crates/forge-db/src/repos/compaction.rs` (NEW)
- `migrations/0006_add_compactions.sql` (NEW)

**Delivers:**
1. `ContextPruner` struct with:
   - `summarize_tool_call(tool_name: &str, args: &str, result: &str) -> String` — one-liner summary
   - `truncate_text(text: &str, max_chars: usize) -> String` — 80% head / 20% tail with `[...truncated...]` marker
   - `estimate_tokens(text: &str) -> usize` — chars / 3 heuristic
   - `prune_messages(messages: &[String], max_tokens: usize) -> Vec<String>` — summarize oldest, keep newest
2. `CompactionRepo` struct with:
   - `create(session_id: &str, summary: &str, original_tokens: i64, compacted_tokens: i64) -> ForgeResult<Compaction>`
   - `list_for_session(session_id: &str) -> ForgeResult<Vec<Compaction>>`
   - `get_latest(session_id: &str) -> ForgeResult<Option<Compaction>>`
3. Migration 0006: `compactions` table (id TEXT PK, session_id TEXT, summary TEXT, original_token_count INTEGER, compacted_token_count INTEGER, created_at TEXT DEFAULT datetime('now'))
4. Tests: `truncate_short_text_unchanged`, `truncate_long_text_splits_80_20`, `estimate_tokens_approximation`, `prune_reduces_below_limit`, `compaction_crud`

**Context — repo pattern:**
```rust
pub struct CompactionRepo { conn: Arc<Mutex<Connection>> }
// Follow same pattern as ScheduleRepo/AnalyticsRepo: new(conn), ForgeResult<T>, ForgeError::Database
```

**Verify:**
```bash
cargo test -p forge-process -- context_pruner
cargo test -p forge-db -- compaction
cargo clippy --workspace
```

---

### Agent P: Pipeline Engine + WorkflowRepo CRUD

**What:** First-class pipeline execution engine. Extends the existing read-only WorkflowRepo with full CRUD. Adds a `PipelineRunner` that executes sequential and fanout steps.

**Exclusive files:**
- `crates/forge-process/src/pipeline.rs` (NEW)
- `migrations/0007_add_workflow_columns.sql` (NEW)

**Also extends (no conflict with other Wave 5 agents):**
- `crates/forge-db/src/repos/workflows.rs` — add `create`, `update`, `delete` methods

**Delivers:**
1. `PipelineStep` enum:
   - `Sequential { agent_id: String, prompt_template: String }`
   - `Fanout { agent_ids: Vec<String>, prompt_template: String }`
2. `Pipeline` struct: `steps: Vec<PipelineStep>`
3. `PipelineRunner` struct:
   - `new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self`
   - `async run(&self, pipeline: &Pipeline, initial_input: &str, working_dir: &str) -> Vec<StepResult>`
4. `StepResult`: `step_index: usize`, `outputs: Vec<SubTaskResult>`, `success: bool`
5. Pipeline execution: Sequential passes previous output as next input. Fanout runs N agents concurrently via ConcurrentRunner.
6. `WorkflowRepo` additions:
   - `create(name: &str, description: Option<&str>, definition_json: &str) -> ForgeResult<Workflow>`
   - `update(id: &str, name: Option<&str>, description: Option<&str>, definition_json: Option<&str>) -> ForgeResult<Workflow>`
   - `delete(id: &str) -> ForgeResult<()>`
7. API routes additions: POST `/api/v1/workflows`, PUT/DELETE `/api/v1/workflows/:id`, POST `/api/v1/workflows/:id/run`
8. Migration 0007: ensure `workflows` table exists with proper schema if not created by prior migrations
9. Tests: `sequential_passes_output`, `fanout_collects_all`, `pipeline_stops_on_error`, `workflow_crud`

**Context — existing WorkflowRepo:**
```rust
pub struct Workflow { pub id: String, pub name: String, pub description: Option<String>, pub definition_json: String, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc> }
pub struct WorkflowRepo { conn: Arc<Mutex<Connection>> }
// Currently only has: list(), get() — no create/update/delete
```

**Verify:**
```bash
cargo test -p forge-process -- pipeline
cargo test -p forge-db -- workflow
cargo clippy --workspace
```

---

### Agent R: OpenAPI Auto-Docs

**What:** Auto-generate OpenAPI 3.1 spec from Axum routes. Serve Scalar UI at `/docs`.

**Exclusive files:**
- `crates/forge-api/src/openapi.rs` (NEW)

**Delivers:**
1. Add `utoipa = "5"` and `utoipa-scalar = "0.2"` to `forge-api/Cargo.toml`
2. `ApiDoc` struct implementing `utoipa::OpenApi` — lists all route paths and schemas
3. `#[derive(ToSchema)]` on request/response types: `RunRequest`, `RunResponse`, `Session`, `Agent`, `NewAgent`, `Skill`, `Workflow`, `Memory`, `NewMemory`, `Hook`, `NewHook`, `Schedule`, `NewSchedule`, `UsageReport`
4. `openapi_json()` handler at `GET /api/openapi.json` — returns spec
5. `scalar_ui()` handler at `GET /docs` — serves Scalar interactive explorer
6. `pub fn openapi_routes() -> Router<AppState>` — mounts both endpoints
7. Test: `openapi_spec_deserializes`

**Verify:**
```bash
cargo test -p forge-api -- openapi
cargo clippy -p forge-api
```

---

## GATE 5: Verify Wave 5

```bash
cargo build --workspace
cargo test --workspace    # target: ~170+ tests
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

---

## Wave 6 — Integration Wiring (1 agent, sequential)

---

### Agent S: Integration Wiring

**Shared files to modify:**
- `Cargo.toml` — add `utoipa`, `utoipa-scalar` workspace deps
- `crates/forge-api/Cargo.toml` — add utoipa deps
- `crates/forge-db/src/migrations.rs` — apply migrations 0006 + 0007
- `crates/forge-db/src/repos/mod.rs` — export CompactionRepo
- `crates/forge-db/src/lib.rs` — re-export CompactionRepo
- `crates/forge-api/src/state.rs` — add `compaction_repo: Arc<CompactionRepo>` to AppState
- `crates/forge-api/src/routes/mod.rs` — merge openapi routes, add workflow mutation routes
- `crates/forge-api/src/routes/workflows.rs` — add POST/PUT/DELETE handlers using WorkflowRepo CRUD + run handler using PipelineRunner
- `crates/forge-api/src/lib.rs` — mount OpenAPI + Scalar UI, update tests to include compaction_repo
- `crates/forge-core/src/events.rs` — add: `PipelineStarted`, `PipelineStepCompleted`, `PipelineCompleted`, `CompactionCompleted`
- `crates/forge-db/src/batch_writer.rs` — add new event variant matches
- `crates/forge-process/src/lib.rs` — export new modules (best_of_n, context_pruner, pipeline)
- `crates/forge-app/src/main.rs` — init CompactionRepo, pass to AppState

**Delivers:**
1. All Wave 5 modules wired and accessible
2. OpenAPI UI at `/docs`, JSON spec at `/api/openapi.json`
3. Workflow CRUD endpoints functional
4. Pipeline execution via POST `/api/v1/workflows/:id/run`
5. 4 new ForgeEvent variants flowing through EventBus
6. All existing 149+ tests still pass

**Verify:**
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

---

## GATE 6: Verify Wave 6

Same as above. All routes resolve, OpenAPI renders.

---

## Wave 7 — Smart Features + UI (2 parallel agents)

---

### Agent U: Three-Type Memory + Auto-Activating Skills

**Files:**
- `crates/forge-db/src/repos/memory.rs` (extend)
- `crates/forge-db/src/repos/skills.rs` (extend)
- `migrations/0008_memory_types_and_skill_rules.sql` (NEW)

**Delivers:**
1. Migration 0008: `ALTER TABLE memory ADD COLUMN memory_type TEXT DEFAULT 'personal'`; CREATE TABLE `skill_rules` (id TEXT PK, skill_id TEXT, trigger_type TEXT, trigger_pattern TEXT, enabled INTEGER DEFAULT 1)
2. `MemoryRepo::search_by_type(memory_type: &str, query: &str) -> ForgeResult<Vec<Memory>>` — filter by type + keyword
3. `MemoryRepo::classify_fact(fact: &ExtractedFact) -> &str` — returns "personal"/"task"/"tool" based on content patterns
4. Update `store_extracted()` to set `memory_type` via classification
5. Update `inject_context()` to retrieve by type with priority weighting (task 3x > tool 2x > personal 1x)
6. `SkillRepo::create_rule(skill_id: &str, trigger_type: &str, trigger_pattern: &str) -> ForgeResult<()>`
7. `SkillRepo::find_matching_rules(working_dir: &str, prompt: &str) -> ForgeResult<Vec<Skill>>` — scan for file patterns (Cargo.toml → Rust skills) + keyword triggers
8. `SkillRepo::delete_rule(id: &str) -> ForgeResult<()>`
9. Tests: `memory_type_filtering`, `classify_fact_detects_tool`, `classify_fact_detects_task`, `skill_rule_matching_file_pattern`, `auto_activation_no_match_returns_empty`

**Context — existing MemoryRepo:**
```rust
pub struct Memory { pub id: String, pub category: String, pub content: String, pub confidence: f64, pub source_session_id: Option<String>, ... }
pub fn extract_facts(transcript: &[String]) -> Vec<ExtractedFact>;
pub fn store_extracted(&self, facts: &[ExtractedFact], session_id: &str) -> ForgeResult<usize>;
pub fn inject_context(&self, prompt: &str, max_memories: usize) -> ForgeResult<Option<String>>;
```

**Verify:**
```bash
cargo test -p forge-db -- memory
cargo test -p forge-db -- skill
cargo clippy --workspace
```

---

### Agent W: Swim-Lane Dashboard + Pipeline Builder UI

**Files:**
- `frontend/src/routes/+page.svelte` (extend)
- `frontend/src/routes/workflows/+page.svelte` (rewrite)
- `frontend/src/lib/api.ts` (extend)

**Delivers:**

**1. Swim-lane dashboard** (in `+page.svelte`):
- Replace flat event log with column-per-agent layout when sub-agents are active
- Each column: agent name header, event blocks stacked vertically
- Status colors: `#3b82f6` running, `#22c55e` done, `#ef4444` failed, `#6b7280` pending
- Tool indicators: emoji or icon per OutputKind (assistant=💬, tool_use=🔧, tool_result=📋, thinking=🧠, result=✅)
- Auto-scroll with "pin to bottom" toggle
- Falls back to flat log when no sub-agents active

**2. Pipeline builder** (rewrite `workflows/+page.svelte`):
- Visual step editor: "Add Step" button → choose Sequential or Fanout
- Each step card: type badge, agent selector dropdown, prompt template textarea
- Drag handle for reorder (CSS-only, swap on click if drag too complex)
- "Run" button → POST `/api/v1/workflows/:id/run` with prompt input
- "Save" button → POST/PUT `/api/v1/workflows` with `definition_json`
- "Delete" button with confirmation
- Step execution status when running (pending/running/done per step)

**3. API client additions** (in `api.ts`):
- `createWorkflow(data: { name: string, description?: string, definition_json: string }): Promise<Workflow>`
- `updateWorkflow(id: string, data: Partial<Workflow>): Promise<Workflow>`
- `deleteWorkflow(id: string): Promise<void>`
- `runWorkflow(id: string, prompt: string): Promise<RunResponse>`

**Style guide** — follow existing patterns:
- CSS variables from `app.css` (e.g., `var(--bg-secondary)`, `var(--text-primary)`)
- Svelte 5 runes: `$state`, `$derived`
- Modal pattern from agents page for create/edit forms
- Badge colors: `#3b82f6` blue, `#22c55e` green, `#ef4444` red, `#f59e0b` amber

**Verify:**
```bash
cd frontend && pnpm build && cd ..
```

---

## GATE 7: Final Verification

```bash
cargo build --workspace
cargo test --workspace    # target: ~175+ tests
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

---

## Tag v0.6.0

```bash
git tag -a v0.6.0 -m "v0.6.0: best-of-N, pipelines, swim lanes, OpenAPI, typed memory"
git push origin v0.6.0
```

---

## Dependency Graph

```
         Wave 5 (parallel)
    ┌────┬────┬────┐
    N    O    P    R
    │    │    │    │
    └────┴────┴────┘
           │
      Wave 6 (sequential)
           S
           │
      Wave 7 (parallel)
      ┌────┐
      U    W
      │    │
      └────┘
           │
       v0.6.0
```

---

## File Ownership Matrix (Wave 5)

| File | N | O | P | R |
|------|---|---|---|---|
| `forge-process/src/best_of_n.rs` | OWN | | | |
| `forge-agent/src/strategy.rs` | OWN | | | |
| `forge-process/src/context_pruner.rs` | | OWN | | |
| `forge-db/src/repos/compaction.rs` | | OWN | | |
| `migrations/0006_add_compactions.sql` | | OWN | | |
| `forge-process/src/pipeline.rs` | | | OWN | |
| `forge-db/src/repos/workflows.rs` | | | EXT | |
| `migrations/0007_add_workflow_columns.sql` | | | OWN | |
| `forge-api/src/openapi.rs` | | | | OWN |

## File Ownership Matrix (Wave 7)

| File | U | W |
|------|---|---|
| `forge-db/src/repos/memory.rs` | EXT | |
| `forge-db/src/repos/skills.rs` | EXT | |
| `migrations/0008_*` | OWN | |
| `frontend/src/routes/+page.svelte` | | EXT |
| `frontend/src/routes/workflows/+page.svelte` | | OWN |
| `frontend/src/lib/api.ts` | | EXT |

OWN = creates, EXT = extends (no conflict within same wave)

---

## Success Criteria

| Metric | v0.5.0 | v0.6.0 Target |
|--------|--------|---------------|
| Tests | 150 | 175+ |
| Frontend pages | 12 | 12 (upgraded) |
| Event types | 35 | 39+ |
| Middlewares | 8 | 8 (reuse existing) |
| DB tables | 8 | 10 (add compactions, skill_rules) |
| DB repos | 8 | 9 (add CompactionRepo) |

---

## What's Deferred to v0.7.0+

| Feature | Why Deferred |
|---------|-------------|
| Conversation rewind/branching | Schema-heavy, needs design |
| Multi-provider model routing | Breaks Claude-only assumption |
| Generator-based step control | Needs Rust async state machine trait design |
| OpenTelemetry tracing | Production ops, not user-facing |
| Hybrid vector + BM25 search | Needs vector extension evaluation |
| MsgHub broadcast communication | Good idea, defer until multi-agent patterns mature |
| Session transcript export | Nice but not core |
| Kanban session view | Nice but not core |
| Critic-fixer loop (full) | QualityGate middleware exists but critic agent spawning needs pipeline engine first |
