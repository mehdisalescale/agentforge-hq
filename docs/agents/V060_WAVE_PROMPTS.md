# v0.6.0 Wave Prompts — Copy-Paste for Parallel Agents

> **Plan:** `docs/V060_SPRINT_PLAN.md`
> **Predecessor:** v0.5.0 (150 tests, 35 events, 12 pages, 8 repos)
> **Mode:** 7 agents across 3 waves

---

## Wave 5 — 4 Parallel Agents (all NEW files)

> Run all 4 simultaneously. Zero file conflicts.
> **Gate:** `cargo test --workspace && cargo clippy --workspace -- -D warnings && cd frontend && pnpm build`

---

### Agent N — Best-of-N Selection Mode

```
You are Agent N in a parallel wave execution for forge-project (Rust/Axum multi-agent orchestrator).

## PROTOCOL
- You own ONLY these files:
  - `crates/forge-process/src/best_of_n.rs` (NEW)
  - `crates/forge-agent/src/strategy.rs` (NEW)
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that

## CONTEXT — Read these first
- `crates/forge-process/src/concurrent.rs` (307 lines) — ConcurrentRunner you'll wrap
- `crates/forge-process/src/lib.rs` — current exports (you'll add yours later in Wave 6)
- `crates/forge-core/src/events.rs` — ForgeEvent enum (35 variants, SubAgent* events exist)

## EXISTING API (do NOT modify)
```rust
// concurrent.rs
pub struct SubTask { pub agent_id: AgentId, pub prompt: String, pub working_dir: String }
pub struct SubTaskResult { pub agent_id: AgentId, pub session_id: SessionId, pub output: String, pub exit_code: i32, pub success: bool }
pub struct ConcurrentRunner { /* event_bus, max_concurrent, spawn_config */ }
impl ConcurrentRunner {
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self;
    pub async fn run_all(&self, parent_session_id: &SessionId, tasks: Vec<SubTask>) -> Vec<SubTaskResult>;
}
```

## TASK

### Step 1: Create `crates/forge-agent/src/strategy.rs`

Define strategy types:

```rust
pub struct Strategy {
    pub name: String,
    pub system_prompt_suffix: String,
}

pub struct StrategySet {
    pub strategies: Vec<Strategy>,
}

impl StrategySet {
    pub fn default_three() -> Self; // "minimal changes", "modular refactor", "thorough with tests"
}
```

Each strategy has a short suffix (2-3 sentences) appended to the agent's system prompt to bias its approach.

### Step 2: Create `crates/forge-process/src/best_of_n.rs`

```rust
pub struct SelectionResult {
    pub chosen_index: usize,
    pub reason: String,
    pub improvements: Vec<String>,
}

pub struct BestOfNRunner {
    runner: ConcurrentRunner,
}

impl BestOfNRunner {
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self;

    /// Runs the same prompt with N strategies, returns all results + selection
    pub async fn run_best_of_n(
        &self,
        parent_session_id: &SessionId,
        base_task: SubTask,
        strategies: &StrategySet,
    ) -> (Vec<SubTaskResult>, SelectionResult);
}

/// Compares results and picks the best one.
/// Heuristics: prefer success (exit_code 0), longer output, fewer "error"/"fail" keywords.
pub fn select_best(results: &[SubTaskResult]) -> SelectionResult;
```

Implementation:
1. For each strategy, clone base_task and append strategy suffix to prompt
2. Call `self.runner.run_all()` with all N tasks
3. Call `select_best()` on results
4. Return (all_results, selection)

`select_best` scoring: +10 if success, +1 per 100 chars output, -5 per "error"/"Error" occurrence, -3 per "fail"/"Fail" occurrence.

### Step 3: Tests (in best_of_n.rs)

- `strategy_set_has_three` — default set has 3 strategies
- `select_best_prefers_success` — success beats failure regardless of length
- `select_best_prefers_longer_on_tie` — same exit code, longer wins
- `select_best_penalizes_errors` — output with "error" scores lower

## VERIFY
```bash
cargo check -p forge-process -p forge-agent
cargo test -p forge-process -- best_of_n
cargo test -p forge-agent -- strategy
cargo clippy -p forge-process -p forge-agent
```

## CONSTRAINTS
- Do NOT modify concurrent.rs, lib.rs, or any existing files
- Use `forge_core::{AgentId, SessionId, EventBus}` for types
- Keep `select_best` as a pure function (no async, no I/O) — easy to test
- The BestOfNRunner wraps ConcurrentRunner, does not duplicate its logic
```

---

### Agent O — Context Pruner + Memory Compaction

```
You are Agent O in a parallel wave execution for forge-project (Rust/Axum multi-agent orchestrator).

## PROTOCOL
- You own ONLY these files:
  - `crates/forge-process/src/context_pruner.rs` (NEW)
  - `crates/forge-db/src/repos/compaction.rs` (NEW)
  - `migrations/0006_add_compactions.sql` (NEW)
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that

## CONTEXT — Read these first
- `crates/forge-db/src/repos/memory.rs` (700 lines) — MemoryRepo pattern to follow
- `crates/forge-db/src/repos/schedules.rs` — ScheduleRepo for CRUD pattern reference
- `crates/forge-core/src/error.rs` — ForgeError::Database variant

## EXISTING PATTERNS
```rust
// All repos follow this pattern:
pub struct SomeRepo { conn: Arc<Mutex<Connection>> }
impl SomeRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self { Self { conn } }
    // Methods return ForgeResult<T>, errors use ForgeError::Database(msg)
}
```

## TASK

### Step 1: Create `migrations/0006_add_compactions.sql`

```sql
CREATE TABLE IF NOT EXISTS compactions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    original_token_count INTEGER NOT NULL,
    compacted_token_count INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_compactions_session ON compactions(session_id);
```

### Step 2: Create `crates/forge-db/src/repos/compaction.rs`

```rust
pub struct Compaction {
    pub id: String,
    pub session_id: String,
    pub summary: String,
    pub original_token_count: i64,
    pub compacted_token_count: i64,
    pub created_at: String,
}

pub struct CompactionRepo { conn: Arc<Mutex<Connection>> }

impl CompactionRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn create(&self, session_id: &str, summary: &str, original_tokens: i64, compacted_tokens: i64) -> ForgeResult<Compaction>;
    pub fn list_for_session(&self, session_id: &str) -> ForgeResult<Vec<Compaction>>;
    pub fn get_latest(&self, session_id: &str) -> ForgeResult<Option<Compaction>>;
}
```

Use `uuid::Uuid::new_v4().to_string()` for IDs (uuid is already a workspace dep).

### Step 3: Create `crates/forge-process/src/context_pruner.rs`

```rust
pub struct ContextPruner;

impl ContextPruner {
    /// Summarize a tool call into a one-line string
    pub fn summarize_tool_call(tool_name: &str, args: &str, result: &str) -> String;
    // Format: "[tool_name] args_preview... → result_preview..."
    // args_preview = first 50 chars, result_preview = first 100 chars

    /// Truncate text: 80% from head, 20% from tail, marker in middle
    pub fn truncate_text(text: &str, max_chars: usize) -> String;
    // If text.len() <= max_chars, return as-is
    // Otherwise: head (80% of max_chars) + "\n[...truncated...]\ " + tail (20% of max_chars)

    /// Estimate token count (chars / 3 heuristic)
    pub fn estimate_tokens(text: &str) -> usize;

    /// Prune a list of messages to fit within max_tokens
    /// Summarizes oldest messages first, keeps newest intact
    pub fn prune_messages(messages: &[String], max_tokens: usize) -> Vec<String>;
    // 1. Calculate total tokens
    // 2. If under max, return clone
    // 3. Otherwise: keep last 50% of messages intact, summarize older ones
    //    (summary = first 80 chars + "...")
    // 4. Repeat until under limit
}
```

### Step 4: Tests

In `context_pruner.rs`:
- `truncate_short_text_unchanged` — text under limit returns unchanged
- `truncate_long_text_splits_80_20` — verify 80/20 split with marker
- `estimate_tokens_approximation` — "hello world" → ~4 tokens
- `prune_reduces_below_limit` — 10 long messages pruned to fit 100 tokens
- `summarize_tool_call_format` — verify "[tool] args → result" format

In `compaction.rs`:
- `compaction_crud` — create + list_for_session + get_latest

## VERIFY
```bash
cargo check -p forge-process -p forge-db
cargo test -p forge-process -- context_pruner
cargo test -p forge-db -- compaction
cargo clippy -p forge-process -p forge-db
```

## CONSTRAINTS
- Do NOT modify existing files (lib.rs, mod.rs, migrations.rs)
- CompactionRepo uses same `Arc<Mutex<Connection>>` pattern as all other repos
- context_pruner is a standalone module with no deps on forge-db (pure functions)
- The migration file will be wired in Wave 6 by Agent S
```

---

### Agent P — Pipeline Engine + WorkflowRepo CRUD

```
You are Agent P in a parallel wave execution for forge-project (Rust/Axum multi-agent orchestrator).

## PROTOCOL
- You own ONLY these files:
  - `crates/forge-process/src/pipeline.rs` (NEW)
  - `migrations/0007_add_workflow_columns.sql` (NEW)
- You ALSO extend (no conflict with other Wave 5 agents):
  - `crates/forge-db/src/repos/workflows.rs` — add create/update/delete methods
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that

## CONTEXT — Read these first
- `crates/forge-process/src/concurrent.rs` (307 lines) — ConcurrentRunner for fanout steps
- `crates/forge-db/src/repos/workflows.rs` (89 lines) — current Workflow struct + list/get
- `crates/forge-core/src/events.rs` — ForgeEvent (has WorkflowStarted/Completed/Failed already)

## EXISTING API
```rust
// workflows.rs — current state
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub struct WorkflowRepo { conn: Arc<Mutex<Connection>> }
impl WorkflowRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn list(&self) -> ForgeResult<Vec<Workflow>>;
    pub fn get(&self, id: &str) -> ForgeResult<Workflow>;
}

// ConcurrentRunner
pub struct SubTask { pub agent_id: AgentId, pub prompt: String, pub working_dir: String }
pub struct SubTaskResult { pub agent_id: AgentId, pub session_id: SessionId, pub output: String, pub exit_code: i32, pub success: bool }
```

## TASK

### Step 1: Extend workflows.rs with CRUD

Add these methods to the existing `WorkflowRepo`:

```rust
pub fn create(&self, name: &str, description: Option<&str>, definition_json: &str) -> ForgeResult<Workflow>;
pub fn update(&self, id: &str, name: Option<&str>, description: Option<&str>, definition_json: Option<&str>) -> ForgeResult<Workflow>;
pub fn delete(&self, id: &str) -> ForgeResult<()>;
```

- `create`: INSERT with uuid::Uuid::new_v4(), datetime('now') for both timestamps
- `update`: UPDATE with SET for non-None fields, update updated_at
- `delete`: DELETE FROM workflows WHERE id = ?

### Step 2: Create `migrations/0007_add_workflow_columns.sql`

Ensure the workflows table exists with proper schema. The table may already exist from a prior migration, so use IF NOT EXISTS:

```sql
-- Ensure workflows table exists (may have been created by init migration)
CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    definition_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Step 3: Create `crates/forge-process/src/pipeline.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStep {
    Sequential { agent_id: String, prompt_template: String },
    Fanout { agent_ids: Vec<String>, prompt_template: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
}

#[derive(Debug)]
pub struct StepResult {
    pub step_index: usize,
    pub outputs: Vec<SubTaskResult>,
    pub success: bool,
}

pub struct PipelineRunner {
    event_bus: Arc<EventBus>,
    max_concurrent: usize,
}

impl PipelineRunner {
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self;

    /// Execute a pipeline: each step feeds output to the next
    pub async fn run(
        &self,
        parent_session_id: &SessionId,
        pipeline: &Pipeline,
        initial_input: &str,
        working_dir: &str,
    ) -> Vec<StepResult>;
}
```

Implementation:
1. For Sequential: create one SubTask with agent_id + prompt_template (replace `{input}` placeholder with previous output), run via ConcurrentRunner with max=1
2. For Fanout: create N SubTasks (one per agent_id), all with same prompt_template + input substitution, run via ConcurrentRunner
3. Chain: previous step's output (concatenated) becomes next step's input
4. Stop on first failed step (any SubTaskResult with success=false)

### Step 4: Tests

In `pipeline.rs`:
- `pipeline_step_serializes` — PipelineStep round-trips through serde_json
- `pipeline_from_json` — Pipeline deserializes from JSON definition
- `step_result_tracks_index` — StepResult has correct step_index

In `workflows.rs` (add to existing test module):
- `workflow_create_and_get` — create returns valid workflow, get retrieves it
- `workflow_update_name` — update changes name, preserves other fields
- `workflow_delete` — delete removes, subsequent get returns NotFound

## VERIFY
```bash
cargo check -p forge-process -p forge-db
cargo test -p forge-process -- pipeline
cargo test -p forge-db -- workflow
cargo clippy -p forge-process -p forge-db
```

## CONSTRAINTS
- Do NOT change the existing Workflow struct or list()/get() methods
- Pipeline/PipelineStep must be Serialize+Deserialize (stored as definition_json)
- PipelineRunner uses ConcurrentRunner internally (import from concurrent module)
- Use `{input}` as the placeholder in prompt_template for previous step output
```

---

### Agent R — OpenAPI Auto-Docs

```
You are Agent R in a parallel wave execution for forge-project (Rust/Axum multi-agent orchestrator).

## PROTOCOL
- You own ONLY this file:
  - `crates/forge-api/src/openapi.rs` (NEW)
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that

## CONTEXT — Read these first
- `crates/forge-api/Cargo.toml` — current dependencies (you'll note what to add)
- `crates/forge-api/src/state.rs` — AppState (for Router<AppState>)
- `crates/forge-api/src/routes/` — all route files for path listing
- `crates/forge-api/src/routes/run.rs` — RunRequest, RunResponse structs
- `crates/forge-api/src/routes/agents.rs` — agent route structs
- `crates/forge-api/src/routes/sessions.rs` — session route structs
- `crates/forge-db/src/repos/` — model structs (Agent, Session, Skill, Workflow, Memory, Hook, Schedule)

## EXISTING ROUTE PATHS
```
GET    /api/v1/health
GET    /api/v1/agents
POST   /api/v1/agents
GET    /api/v1/agents/:id
PUT    /api/v1/agents/:id
DELETE /api/v1/agents/:id
POST   /api/v1/run
GET    /api/v1/sessions
GET    /api/v1/sessions/:id
GET    /api/v1/sessions/:id/events
GET    /api/v1/sessions/:id/export
DELETE /api/v1/sessions/:id
GET    /api/v1/skills
GET    /api/v1/skills/:id
GET    /api/v1/workflows
GET    /api/v1/workflows/:id
GET    /api/v1/memory
POST   /api/v1/memory
GET    /api/v1/memory/:id
PUT    /api/v1/memory/:id
DELETE /api/v1/memory/:id
GET    /api/v1/hooks
POST   /api/v1/hooks
GET    /api/v1/hooks/:id
PUT    /api/v1/hooks/:id
DELETE /api/v1/hooks/:id
GET    /api/v1/schedules
POST   /api/v1/schedules
GET    /api/v1/schedules/:id
PUT    /api/v1/schedules/:id
DELETE /api/v1/schedules/:id
POST   /api/v1/schedules/:id/trigger
GET    /api/v1/analytics/usage
GET    /ws
```

## TASK

### Step 1: Note dependency additions needed

The integration agent (Wave 6) will add these to Cargo.toml. Document what's needed at the top of your file:

```rust
// NOTE for Agent S (Wave 6): Add to forge-api/Cargo.toml:
//   utoipa = { version = "5", features = ["axum_extras"] }
//   utoipa-scalar = { version = "0.2", features = ["axum"] }
```

### Step 2: Create `crates/forge-api/src/openapi.rs`

```rust
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use axum::{Router, Json, routing::get};

// Define the OpenAPI doc struct listing all paths
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Claude Forge API",
        version = "0.6.0",
        description = "Multi-agent Claude Code orchestrator"
    ),
    paths(
        // List all handler paths here
    ),
    components(schemas(
        // List all schema types here
    ))
)]
pub struct ApiDoc;

/// GET /api/openapi.json — returns the OpenAPI 3.1 spec
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Returns a Router with OpenAPI JSON + Scalar UI
pub fn openapi_routes() -> Router<crate::state::AppState> {
    Router::new()
        .route("/api/openapi.json", get(openapi_json))
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}
```

Since we can't add `#[utoipa::path(...)]` annotations to existing route handlers without modifying them, manually build the OpenAPI spec by listing paths in the `#[openapi(paths(...))]` attribute. For types that need `ToSchema`, create wrapper types in this file that mirror the main types:

```rust
/// Mirror types for OpenAPI schema generation
/// These are schema-only — not used in actual request handling
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct RunRequestSchema {
    pub agent_id: String,
    pub prompt: String,
    pub session_id: Option<String>,
    pub working_dir: Option<String>,
}
// ... etc for other request/response types
```

### Step 3: Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_spec_deserializes() {
        let spec = ApiDoc::openapi();
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("Claude Forge API"));
        assert!(json.contains("/api/v1/agents"));
    }
}
```

## VERIFY
```bash
cargo check -p forge-api
cargo test -p forge-api -- openapi
cargo clippy -p forge-api
```

## CONSTRAINTS
- Do NOT modify any existing route handlers or model structs
- Create mirror/schema types in openapi.rs rather than adding derives to existing types
- The openapi_routes() function returns Router<AppState> — Agent S will merge it in lib.rs
- utoipa 5 + utoipa-scalar 0.2 — Agent S adds these deps in Wave 6
- If utoipa compile errors occur because deps aren't added yet, that's expected — Agent S fixes in Wave 6
```

---

## Wave 6 — Integration Wiring (1 agent, sequential)

> Run AFTER Wave 5 gate passes.
> **Gate:** `cargo test --workspace && cargo clippy --workspace -- -D warnings && cd frontend && pnpm build`

---

### Agent S — Integration Wiring

```
You are Agent S, the integration agent for Wave 6. Your job: wire all Wave 5 outputs into the shared files so the app compiles and runs with all new features.

## PROTOCOL
- You modify SHARED files that multiple crates depend on
- You must run AFTER Wave 5 gate passes (all 4 agents' code compiles independently)
- Do NOT commit or push — the coordinator handles that

## SHARED FILES TO MODIFY

1. `Cargo.toml` (workspace root) — add utoipa, utoipa-scalar workspace deps
2. `crates/forge-api/Cargo.toml` — add utoipa, utoipa-scalar deps
3. `crates/forge-db/src/migrations.rs` — apply migrations 0006 + 0007
4. `crates/forge-db/src/repos/mod.rs` — add `pub mod compaction;`
5. `crates/forge-db/src/lib.rs` — re-export CompactionRepo
6. `crates/forge-api/src/state.rs` — add `compaction_repo: Arc<CompactionRepo>` to AppState
7. `crates/forge-api/src/routes/mod.rs` — merge openapi routes, add workflow mutation routes
8. `crates/forge-api/src/routes/workflows.rs` — add POST/PUT/DELETE/run handlers
9. `crates/forge-api/src/lib.rs` — mount openapi + scalar, update test AppState
10. `crates/forge-core/src/events.rs` — add PipelineStarted, PipelineStepCompleted, PipelineCompleted, CompactionCompleted
11. `crates/forge-db/src/batch_writer.rs` — add match arms for new event variants
12. `crates/forge-process/src/lib.rs` — export best_of_n, context_pruner, pipeline modules
13. `crates/forge-app/src/main.rs` — init CompactionRepo, pass to AppState

## CONTEXT — Read these first
- All Wave 5 agent files (best_of_n.rs, strategy.rs, context_pruner.rs, compaction.rs, pipeline.rs, openapi.rs)
- Current versions of all shared files listed above
- `crates/forge-api/src/middleware.rs` — for understanding how run.rs uses middleware chain

## TASK (execute in order)

### Step 1: Workspace deps
Add to root `Cargo.toml` [workspace.dependencies]:
```toml
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-scalar = { version = "0.2", features = ["axum"] }
```

Add to `crates/forge-api/Cargo.toml` [dependencies]:
```toml
utoipa = { workspace = true }
utoipa-scalar = { workspace = true }
```

### Step 2: Migrations
In `crates/forge-db/src/migrations.rs`:
- Add MIGRATION_006 (include_str 0006_add_compactions.sql)
- Add MIGRATION_007 (include_str 0007_add_workflow_columns.sql)
- Update apply_pending() to apply 6 and 7

### Step 3: Repo exports
In `crates/forge-db/src/repos/mod.rs` add: `pub mod compaction;`
In `crates/forge-db/src/lib.rs` add: re-export CompactionRepo

### Step 4: AppState
In `crates/forge-api/src/state.rs` add field: `pub compaction_repo: Arc<CompactionRepo>`

### Step 5: Events
In `crates/forge-core/src/events.rs` add to ForgeEvent enum:
- `PipelineStarted { session_id: String, workflow_id: String, step_count: usize }`
- `PipelineStepCompleted { session_id: String, step_index: usize, success: bool }`
- `PipelineCompleted { session_id: String, workflow_id: String, success: bool }`
- `CompactionCompleted { session_id: String, original_tokens: i64, compacted_tokens: i64 }`

### Step 6: BatchWriter
In `crates/forge-db/src/batch_writer.rs` add match arms for new variants (store as JSON in events table like other events).

### Step 7: Process exports
In `crates/forge-process/src/lib.rs` add:
```rust
pub mod best_of_n;
pub mod context_pruner;
pub mod pipeline;
pub use best_of_n::{BestOfNRunner, SelectionResult};
pub use context_pruner::ContextPruner;
pub use pipeline::{Pipeline, PipelineRunner, PipelineStep, StepResult};
```

### Step 8: Workflow routes
In `crates/forge-api/src/routes/workflows.rs` add handlers:
- `POST /api/v1/workflows` — create_workflow (name, description, definition_json)
- `PUT /api/v1/workflows/:id` — update_workflow
- `DELETE /api/v1/workflows/:id` — delete_workflow
- `POST /api/v1/workflows/:id/run` — run_workflow (prompt, working_dir) → spawn pipeline

### Step 9: Route mounting
In `crates/forge-api/src/routes/mod.rs`:
- Import and merge `openapi::openapi_routes()`
- Update workflows routes to include POST/PUT/DELETE/run

### Step 10: Main wiring
In `crates/forge-app/src/main.rs`:
- Create `CompactionRepo::new(conn.clone())`
- Pass to AppState constructor

### Step 11: Update lib.rs tests
In `crates/forge-api/src/lib.rs`:
- Add `compaction_repo` to test AppState initialization
- Mount openapi routes in the test app if needed

## VERIFY
```bash
cargo build --workspace
cargo test --workspace        # all 150+ existing tests must pass
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

## CONSTRAINTS
- Do NOT change any Wave 5 agent files (best_of_n.rs, etc.) — only wire them
- Keep all existing tests passing — if a test breaks, fix the wiring, not the test
- Follow existing patterns: Arc<Repo>, ForgeResult<T>, ForgeError::Database
- The OpenAPI routes should be accessible without auth (no middleware wrap)
```

---

## Wave 7 — Smart Features + UI (2 parallel agents)

> Run AFTER Wave 6 gate passes.
> **Gate:** `cargo test --workspace && cargo clippy --workspace -- -D warnings && cd frontend && pnpm build`

---

### Agent U — Three-Type Memory + Auto-Activating Skills

```
You are Agent U in a parallel wave execution for forge-project (Rust/Axum multi-agent orchestrator).

## PROTOCOL
- You EXTEND these files:
  - `crates/forge-db/src/repos/memory.rs` (existing, 700 lines)
  - `crates/forge-db/src/repos/skills.rs` (existing, 467 lines)
- You own:
  - `migrations/0008_memory_types_and_skill_rules.sql` (NEW)
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that

## CONTEXT — Read these first
- `crates/forge-db/src/repos/memory.rs` — full file, understand Memory struct, extract_facts, store_extracted, inject_context
- `crates/forge-db/src/repos/skills.rs` — full file, understand Skill struct, list, get, upsert, load_from_dir

## EXISTING API
```rust
// memory.rs
pub struct Memory { pub id: String, pub category: String, pub content: String, pub confidence: f64, pub source_session_id: Option<String>, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc> }
pub struct ExtractedFact { pub category: String, pub content: String, pub confidence: f64 }
pub fn extract_facts(transcript: &[String]) -> Vec<ExtractedFact>;
pub fn store_extracted(&self, facts: &[ExtractedFact], session_id: &str) -> ForgeResult<usize>;
pub fn inject_context(&self, prompt: &str, max_memories: usize) -> ForgeResult<Option<String>>;

// skills.rs
pub struct Skill { pub id: String, pub name: String, pub description: Option<String>, pub category: Option<String>, pub content: Option<String>, pub parameters_json: Option<String>, ... }
pub fn list(&self) -> ForgeResult<Vec<Skill>>;
pub fn get(&self, id: &str) -> ForgeResult<Skill>;
pub fn upsert(&self, input: &UpsertSkill) -> ForgeResult<()>;
```

## TASK

### Step 1: Create `migrations/0008_memory_types_and_skill_rules.sql`

```sql
-- Add memory_type column (default 'personal' for existing rows)
ALTER TABLE memory ADD COLUMN memory_type TEXT NOT NULL DEFAULT 'personal';

-- Skill activation rules
CREATE TABLE IF NOT EXISTS skill_rules (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL,
    trigger_type TEXT NOT NULL,  -- 'file_pattern' or 'keyword'
    trigger_pattern TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_skill_rules_skill ON skill_rules(skill_id);
```

### Step 2: Extend memory.rs — typed memory

Add method to classify facts:
```rust
/// Classify a fact as "personal", "task", or "tool" based on content patterns
pub fn classify_fact(fact: &ExtractedFact) -> &'static str {
    let content_lower = fact.content.to_lowercase();
    if content_lower.contains("prefers") || content_lower.contains("always") || content_lower.contains("style") {
        "personal"
    } else if content_lower.contains("tool") || content_lower.contains("command") || content_lower.contains("cli") || content_lower.contains("api") {
        "tool"
    } else {
        "task"
    }
}
```

Add type-filtered search:
```rust
pub fn search_by_type(&self, memory_type: &str, query: &str) -> ForgeResult<Vec<Memory>>;
// SELECT * FROM memory WHERE memory_type = ? AND content LIKE '%' || ? || '%' ORDER BY confidence DESC LIMIT 20
```

Update `store_extracted` to set `memory_type` via `classify_fact()` on each fact.

Update `inject_context` to retrieve by type with priority weighting:
- task memories: weight 3x (retrieve 3x as many)
- tool memories: weight 2x
- personal memories: weight 1x
- Total still limited by max_memories param

### Step 3: Extend skills.rs — auto-activation rules

Add skill rules CRUD:
```rust
pub struct SkillRule {
    pub id: String,
    pub skill_id: String,
    pub trigger_type: String,   // "file_pattern" or "keyword"
    pub trigger_pattern: String,
    pub enabled: bool,
    pub created_at: String,
}

impl SkillRepo {
    pub fn create_rule(&self, skill_id: &str, trigger_type: &str, trigger_pattern: &str) -> ForgeResult<SkillRule>;
    pub fn list_rules(&self, skill_id: &str) -> ForgeResult<Vec<SkillRule>>;
    pub fn delete_rule(&self, id: &str) -> ForgeResult<()>;

    /// Find skills whose rules match the current context
    pub fn find_matching_rules(&self, working_dir: &str, prompt: &str) -> ForgeResult<Vec<Skill>>;
}
```

`find_matching_rules` logic:
1. Query all enabled rules
2. For `file_pattern` rules: check if any file matching the glob pattern exists in working_dir (e.g., "Cargo.toml" → Rust skills). Use simple string contains check, not full glob.
3. For `keyword` rules: check if prompt contains the trigger_pattern (case-insensitive)
4. Return the distinct skills that matched

### Step 4: Tests

In `memory.rs` tests:
- `classify_fact_detects_personal` — "user prefers dark mode" → "personal"
- `classify_fact_detects_tool` — "use cargo test command" → "tool"
- `classify_fact_detects_task` — "implement the login page" → "task"
- `memory_type_filtering` — store 3 facts of different types, search_by_type returns only matching type

In `skills.rs` tests:
- `skill_rule_crud` — create rule, list rules, delete rule
- `skill_rule_matching_keyword` — create keyword rule "rust", prompt "write rust code" → matches
- `auto_activation_no_match_returns_empty` — no matching rules → empty vec

## VERIFY
```bash
cargo check -p forge-db
cargo test -p forge-db -- memory
cargo test -p forge-db -- skill
cargo clippy -p forge-db
```

## CONSTRAINTS
- Do NOT break existing tests — all current memory + skill tests must still pass
- The ALTER TABLE may fail if column exists — wrap in IF NOT EXISTS or handle gracefully
- classify_fact is a simple heuristic — don't overthink it, keyword matching is fine
- find_matching_rules does filesystem checks — use std::path::Path::new(working_dir).join(pattern).exists() for file patterns
```

---

### Agent W — Swim-Lane Dashboard + Pipeline Builder UI

```
You are Agent W in a parallel wave execution for forge-project (Rust/Axum multi-agent orchestrator, SvelteKit 5 frontend).

## PROTOCOL
- You EXTEND these files:
  - `frontend/src/routes/+page.svelte` (existing dashboard)
  - `frontend/src/routes/workflows/+page.svelte` (existing placeholder)
  - `frontend/src/lib/api.ts` (existing API client)
- Do NOT modify any backend files
- Do NOT commit or push — the coordinator handles that

## CONTEXT — Read these first
- `frontend/src/routes/+page.svelte` — current dashboard with agent selector, prompt, WebSocket streaming, sub-agent panel
- `frontend/src/routes/workflows/+page.svelte` — current placeholder with visual diagram and card layout
- `frontend/src/lib/api.ts` — API client functions (fetchAgents, fetchSessions, etc.)
- `frontend/src/app.css` — CSS variables and theme
- `frontend/src/routes/agents/+page.svelte` — reference for modal/form patterns

## EXISTING PATTERNS
- Svelte 5 runes: `let agents = $state([])`, `let loading = $state(false)`
- API calls: `const res = await fetch(\`/api/v1/...\`); const data = await res.json();`
- Modals: `let showCreateModal = $state(false)` → `{#if showCreateModal}` → form
- Badge colors: `#3b82f6` blue, `#22c55e` green, `#ef4444` red, `#f59e0b` amber, `#6b7280` gray
- CSS vars: `var(--bg-primary)`, `var(--bg-secondary)`, `var(--text-primary)`, `var(--text-muted)`, `var(--border-color)`

## TASK

### Part 1: Swim-Lane Dashboard (extend +page.svelte)

Add a swim-lane view that activates when sub-agents are running:

1. **Detection:** Track `SubAgentStarted` / `SubAgentCompleted` / `SubAgentFailed` events from WebSocket
2. **Layout:** When sub-agents active, show columns (one per agent) in a horizontal flex container
3. **Each column:**
   - Header: agent name/ID, status badge (running/done/failed)
   - Body: event blocks stacked vertically (newest at bottom)
   - Each event: type icon + content preview
4. **Status colors:**
   - Running: `#3b82f6` (blue)
   - Done: `#22c55e` (green)
   - Failed: `#ef4444` (red)
   - Pending: `#6b7280` (gray)
5. **Icons per OutputKind:** assistant=💬, tool_use=🔧, tool_result=📋, thinking=🧠, result=✅
6. **Auto-scroll** with "pin to bottom" toggle checkbox
7. **Fallback:** When no sub-agents active, show the existing flat event log (don't break it)

Implementation approach:
```svelte
let subAgentEvents = $state({});  // { [agentId]: { name, status, events: [] } }
let swimLaneMode = $derived(Object.keys(subAgentEvents).length > 0);
```

### Part 2: Pipeline Builder (rewrite workflows/+page.svelte)

Replace the placeholder with a real pipeline builder:

1. **List view:** Show existing workflows as cards (fetch from GET /api/v1/workflows)
2. **Create button:** Opens editor
3. **Editor:**
   - Name + description inputs
   - "Add Step" button → dropdown: Sequential or Fanout
   - Each step as a card:
     - Type badge (Sequential = blue, Fanout = amber)
     - Agent selector dropdown (fetch agents list)
     - Prompt template textarea
     - Remove button (X)
   - Step reordering: Up/Down buttons on each card
4. **Actions:**
   - "Save" → POST /api/v1/workflows (create) or PUT /api/v1/workflows/:id (update)
   - "Run" → POST /api/v1/workflows/:id/run with prompt input
   - "Delete" → DELETE /api/v1/workflows/:id (with confirmation)
5. **Run status:** After clicking Run, show step-by-step progress (pending → running → done)
6. **definition_json format:**
```json
{
  "steps": [
    { "Sequential": { "agent_id": "...", "prompt_template": "..." } },
    { "Fanout": { "agent_ids": ["..."], "prompt_template": "..." } }
  ]
}
```

### Part 3: API Client additions (extend api.ts)

Add these functions:
```typescript
export async function createWorkflow(data: { name: string; description?: string; definition_json: string }): Promise<any> {
    const res = await fetch('/api/v1/workflows', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) });
    return res.json();
}

export async function updateWorkflow(id: string, data: Partial<{ name: string; description: string; definition_json: string }>): Promise<any> {
    const res = await fetch(`/api/v1/workflows/${id}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) });
    return res.json();
}

export async function deleteWorkflow(id: string): Promise<void> {
    await fetch(`/api/v1/workflows/${id}`, { method: 'DELETE' });
}

export async function runWorkflow(id: string, prompt: string, working_dir?: string): Promise<any> {
    const res = await fetch(`/api/v1/workflows/${id}/run`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ prompt, working_dir }) });
    return res.json();
}
```

## VERIFY
```bash
cd frontend && pnpm build && cd ..
```

## CONSTRAINTS
- Use Svelte 5 runes ($state, $derived, $effect) — NO let/reactive stores
- Follow existing CSS variable patterns — do NOT hardcode colors, use var(--bg-secondary) etc.
- Do NOT break the existing dashboard functionality — swim lanes are an ADDITION
- The workflow editor must produce valid JSON matching the Pipeline struct schema
- Keep the existing workflow list/card view — add the editor as a modal or separate view
- Test by building: `pnpm build` must succeed with zero errors
```

---

## Verification Gates Summary

| Gate | When | Command | Expected |
|------|------|---------|----------|
| Gate 5 | After Wave 5 | `cargo build && cargo test && cargo clippy -- -D warnings && cd frontend && pnpm build` | ~160+ tests, zero warnings |
| Gate 6 | After Wave 6 | Same | ~165+ tests, all routes resolve |
| Gate 7 | After Wave 7 | Same | ~175+ tests, frontend builds clean |

After Gate 7: `git tag -a v0.6.0 -m "v0.6.0: best-of-N, pipelines, swim lanes, OpenAPI, typed memory"`
