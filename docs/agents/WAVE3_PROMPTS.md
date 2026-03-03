# Wave 3 — Agent Prompts (Copy-Paste)

> **Prerequisite:** Wave 2 gate passed (94 tests, clippy clean, commit 7084758).
> **Mode:** 3 agents in parallel. Each owns distinct files.

---

## Agent G — Middleware Extraction + Skill Injection

```
You are Agent G in a parallel wave execution. Your task: extract the monolithic run.rs handler into 6 middleware implementations using the existing middleware chain infrastructure, and add skill injection.

## PROTOCOL
- You own ONLY these files: `crates/forge-api/src/middleware.rs` (EXTEND), `crates/forge-api/src/routes/run.rs` (REFACTOR)
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that
- When done, write a completion summary as a comment at the end

## CONTEXT — Read these first
- `crates/forge-api/src/middleware.rs` — Middleware trait, MiddlewareChain, RunContext, Next (211 lines, 3 tests)
- `crates/forge-api/src/routes/run.rs` — Current monolithic run handler (209 lines)
- `crates/forge-api/src/state.rs` — AppState with all repos + safety
- `crates/forge-db/src/repos/skills.rs` — SkillRepo with list(), get(), search via parameters_json (tags)
- `crates/forge-safety/src/lib.rs` — CircuitBreaker (check/record_success/record_failure), RateLimiter (try_acquire), CostTracker (check → BudgetStatus)

## TASK

### Step 1: Add 6 concrete middleware implementations to middleware.rs

Each implements the existing `Middleware` trait (Pin<Box<dyn Future>>, no async_trait).

1. **RateLimitMiddleware** — wraps `RateLimiter::try_acquire()`, short-circuits with `MiddlewareError::RateLimited`
2. **CircuitBreakerMiddleware** — wraps `CircuitBreaker::check()`, short-circuits with `MiddlewareError::CircuitOpen`
3. **CostCheckMiddleware** — wraps `CostTracker::check()`, short-circuits with `MiddlewareError::BudgetExceeded`
4. **SkillInjectionMiddleware** — queries SkillRepo, keyword-matches prompt against skill tags (parameters_json), appends matched skill content to `ctx.metadata["injected_skills"]`
5. **SpawnMiddleware** — this is the terminal middleware (does NOT call `next.run()`). It spawns the Claude CLI process, streams output, emits events. Extracts the spawn+stream logic from current run.rs lines 80-198.
6. **PersistMiddleware** — wraps next, after completion updates session status and emits ProcessCompleted/ProcessFailed events

Each middleware needs:
- A struct holding the necessary Arc<...> references (e.g., `Arc<RateLimiter>`)
- `impl Middleware for ...` with `process()` and `name()`
- At least 1 unit test

### Step 2: Refactor run.rs to use the chain

Replace the inline logic in `run_handler` with:
1. Parse request, validate agent_id, create/resolve session (keep this)
2. Build `RunContext` from the request
3. Construct `MiddlewareChain` and add middlewares in order: RateLimit → CircuitBreaker → CostCheck → SkillInjection → Persist → Spawn
4. Execute chain, map `MiddlewareError` variants to HTTP responses
5. Return 202 with session_id

The handler should shrink from ~160 lines to ~40 lines.

### Step 3: Update RunContext if needed

You may need to add fields to `RunContext` to carry state between middlewares:
- `event_bus: Arc<EventBus>` — for middlewares that emit events
- `session_repo: Arc<SessionRepo>` — for persist middleware
- `agent_repo: Arc<AgentRepo>` — for validation
- Or use `metadata: HashMap<String, String>` for lightweight data passing

Keep the approach simple — prefer adding typed fields over stringly-typed metadata.

## VERIFY
```bash
cargo check -p forge-api
cargo test -p forge-api
cargo test -p forge-db   # ensure skill tests still pass
cargo clippy -p forge-api
```

## IMPORTANT CONSTRAINTS
- The `Middleware` trait signature is FIXED (Pin<Box<dyn Future>>). Do not change it.
- `MiddlewareChain`, `Next`, `RunContext`, `RunResponse`, `MiddlewareError` already exist. Extend, don't replace.
- SpawnMiddleware needs tokio (already in forge-api deps) for async process spawning
- The SkillInjectionMiddleware needs access to `SkillRepo` — add it as a field on the struct
- Keep existing tests passing. Add new tests for each middleware.
```

---

## Agent H — Memory Extraction + Injection

```
You are Agent H in a parallel wave execution. Your task: add memory extraction (post-session) and memory injection (pre-run) logic to the memory system.

## PROTOCOL
- You own ONLY this file: `crates/forge-db/src/repos/memory.rs` (EXTEND)
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that
- When done, write a completion summary as a comment at the end

## CONTEXT — Read these first
- `crates/forge-db/src/repos/memory.rs` — MemoryRepo with CRUD + search (417 lines, 8 tests)
- `crates/forge-process/src/runner.rs` — ProcessRunner emits ForgeEvent (understand event flow)
- `crates/forge-core/src/events.rs` — ForgeEvent variants (ProcessOutput, ProcessCompleted, etc.)

## TASK

### Step 1: Add memory extraction logic

Add a function that takes a session transcript (Vec of output strings) and extracts structured facts:

```rust
pub struct ExtractedFact {
    pub category: String,
    pub content: String,
    pub confidence: f64,
}

impl MemoryRepo {
    /// Extract facts from a session transcript.
    /// Parses structured patterns like key decisions, error solutions,
    /// codebase patterns, and user preferences.
    pub fn extract_facts(transcript: &[String]) -> Vec<ExtractedFact> {
        // Pattern-based extraction (no LLM call — that's a future enhancement)
        // Look for:
        // 1. Lines containing "decided to", "chose", "prefer" → category: "decisions"
        // 2. Lines containing "fixed by", "solved by", "the fix was" → category: "solutions"
        // 3. Lines containing "pattern:", "convention:", "always" → category: "patterns"
        // 4. Lines with file paths + descriptions → category: "codebase"
        // Return with confidence 0.5-0.9 based on match strength
    }

    /// Store extracted facts from a completed session.
    /// Deduplicates against existing memories by content similarity.
    pub fn store_extracted(&self, facts: &[ExtractedFact], session_id: &str) -> ForgeResult<usize> {
        // For each fact:
        // 1. Search existing memories for similar content (LIKE match)
        // 2. If similar exists with higher confidence, skip
        // 3. If similar exists with lower confidence, update
        // 4. If no similar, create new
    }
}
```

### Step 2: Add memory injection logic

Add a function that retrieves relevant memories for a given prompt:

```rust
impl MemoryRepo {
    /// Find memories relevant to a prompt.
    /// Matches by keyword overlap between prompt words and memory content/category.
    /// Returns formatted context block for prepending to system prompt.
    pub fn inject_context(&self, prompt: &str, max_memories: usize) -> ForgeResult<Option<String>> {
        // 1. Extract keywords from prompt (split, filter stopwords, lowercase)
        // 2. Search memories matching any keyword
        // 3. Sort by confidence DESC, take top N
        // 4. If none found, return None
        // 5. Format as markdown block:
        //    "## Relevant Context (from previous sessions)\n\n- {fact1}\n- {fact2}\n..."
    }
}
```

### Step 3: Add tests

- `extract_facts_finds_decisions` — transcript with "decided to use X" → extracts decision fact
- `extract_facts_finds_solutions` — transcript with "fixed by doing Y" → extracts solution fact
- `extract_facts_empty_transcript` — returns empty vec
- `store_extracted_deduplicates` — same fact twice → only 1 stored
- `store_extracted_updates_higher_confidence` — re-extract with higher confidence updates
- `inject_context_returns_relevant` — create memories, inject with matching prompt
- `inject_context_returns_none_when_empty` — no memories → None
- `inject_context_respects_max` — more memories than max → truncates

## VERIFY
```bash
cargo test -p forge-db -- memory
cargo clippy -p forge-db
```

## IMPORTANT CONSTRAINTS
- No external dependencies — use only what's already in forge-db/Cargo.toml
- Pattern-based extraction only (no LLM calls, no HTTP requests)
- Keep the existing 8 tests passing
- Stopwords list: ["the", "a", "an", "is", "are", "was", "were", "to", "in", "for", "of", "and", "or", "it", "this", "that", "with"]
- Confidence scores: decisions=0.8, solutions=0.9, patterns=0.7, codebase=0.6
```

---

## Agent I — Sub-agent Concurrent Runner + Coordinator

```
You are Agent I in a parallel wave execution. Your task: add concurrent sub-agent spawning to forge-process and a Coordinator preset to forge-agent.

## PROTOCOL
- You own ONLY these files:
  - `crates/forge-process/src/concurrent.rs` (NEW file)
  - `crates/forge-process/src/lib.rs` (add `pub mod concurrent;` and re-exports)
  - `crates/forge-agent/src/preset.rs` (add Coordinator variant)
  - `crates/forge-agent/src/lib.rs` (no changes needed unless re-exports change)
- Do NOT modify any other files
- Do NOT commit or push — the coordinator handles that
- When done, write a completion summary as a comment at the end

## CONTEXT — Read these first
- `crates/forge-process/src/spawn.rs` — SpawnConfig, spawn(), ProcessHandle (take_stdout, kill, wait)
- `crates/forge-process/src/runner.rs` — ProcessRunner, emit(), emit_parsed_event()
- `crates/forge-process/src/lib.rs` — current exports
- `crates/forge-agent/src/preset.rs` — 9 presets (CodeWriter, Reviewer, etc.), PresetDefaults
- `crates/forge-agent/src/lib.rs` — re-exports
- `crates/forge-core/src/events.rs` — SubAgentRequested/Started/Completed/Failed events
- `crates/forge-core/src/ids.rs` — AgentId, SessionId

## TASK

### Step 1: Create `crates/forge-process/src/concurrent.rs`

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::spawn::{SpawnConfig, spawn, ProcessHandle};
use forge_core::event_bus::EventBus;
use forge_core::events::ForgeEvent;
use forge_core::ids::{AgentId, SessionId};
use forge_core::ForgeResult;

/// A sub-task to be executed concurrently.
pub struct SubTask {
    pub agent_id: AgentId,
    pub prompt: String,
    pub working_dir: String,
}

/// Result from a completed sub-task.
pub struct SubTaskResult {
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub output: String,
    pub exit_code: i32,
    pub success: bool,
}

/// Runs multiple sub-agent processes concurrently with a configurable
/// concurrency limit using a semaphore.
pub struct ConcurrentRunner {
    event_bus: Arc<EventBus>,
    max_concurrent: usize,
}

impl ConcurrentRunner {
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self {
        Self { event_bus, max_concurrent }
    }

    /// Run all sub-tasks concurrently (up to max_concurrent at a time).
    /// Emits SubAgent* events for each task.
    /// Returns results for all tasks (success or failure).
    pub async fn run_all(
        &self,
        parent_session_id: &SessionId,
        tasks: Vec<SubTask>,
    ) -> Vec<SubTaskResult> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut handles = Vec::new();

        for task in tasks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let event_bus = Arc::clone(&self.event_bus);
            let parent_sid = parent_session_id.clone();

            // Emit SubAgentRequested
            let _ = event_bus.emit(ForgeEvent::SubAgentRequested {
                parent_session_id: parent_sid.clone(),
                sub_agent_id: task.agent_id.clone(),
                prompt: task.prompt.clone(),
                timestamp: chrono::Utc::now(),
            });

            let handle = tokio::spawn(async move {
                let session_id = SessionId::new();

                // Emit SubAgentStarted
                let _ = event_bus.emit(ForgeEvent::SubAgentStarted {
                    parent_session_id: parent_sid.clone(),
                    sub_agent_id: task.agent_id.clone(),
                    session_id: session_id.clone(),
                    timestamp: chrono::Utc::now(),
                });

                let config = SpawnConfig::from_env()
                    .with_working_dir(&task.working_dir);

                let result = match spawn(&config, &task.prompt, None).await {
                    Ok(mut proc_handle) => {
                        // Read all stdout
                        let output = collect_output(&mut proc_handle).await;
                        let status = proc_handle.wait().await;
                        let exit_code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
                        let success = exit_code == 0;

                        if success {
                            let _ = event_bus.emit(ForgeEvent::SubAgentCompleted {
                                parent_session_id: parent_sid.clone(),
                                sub_agent_id: task.agent_id.clone(),
                                session_id: session_id.clone(),
                                timestamp: chrono::Utc::now(),
                            });
                        } else {
                            let _ = event_bus.emit(ForgeEvent::SubAgentFailed {
                                parent_session_id: parent_sid.clone(),
                                sub_agent_id: task.agent_id.clone(),
                                error: format!("exit code {}", exit_code),
                                timestamp: chrono::Utc::now(),
                            });
                        }

                        SubTaskResult {
                            agent_id: task.agent_id,
                            session_id,
                            output,
                            exit_code,
                            success,
                        }
                    }
                    Err(e) => {
                        let _ = event_bus.emit(ForgeEvent::SubAgentFailed {
                            parent_session_id: parent_sid,
                            sub_agent_id: task.agent_id.clone(),
                            error: e.to_string(),
                            timestamp: chrono::Utc::now(),
                        });

                        SubTaskResult {
                            agent_id: task.agent_id,
                            session_id,
                            output: String::new(),
                            exit_code: -1,
                            success: false,
                        }
                    }
                };

                drop(permit); // Release semaphore
                result
            });

            handles.push(handle);
        }

        // Collect all results
        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        results
    }
}

/// Collect all stdout from a process handle into a single string.
async fn collect_output(handle: &mut ProcessHandle) -> String {
    use tokio::io::AsyncReadExt;
    let mut output = String::new();
    if let Some(mut stdout) = handle.take_stdout() {
        let _ = stdout.read_to_string(&mut output).await;
    }
    output
}

/// Aggregate sub-task results into a summary.
pub fn aggregate_results(results: &[SubTaskResult]) -> String {
    let total = results.len();
    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = total - succeeded;

    let mut summary = format!("## Sub-agent Results: {}/{} succeeded\n\n", succeeded, total);

    for (i, result) in results.iter().enumerate() {
        let status = if result.success { "OK" } else { "FAILED" };
        summary.push_str(&format!(
            "### Sub-task {} [{}]\n{}\n\n",
            i + 1,
            status,
            if result.output.is_empty() {
                "(no output)".to_string()
            } else {
                // Truncate long outputs
                let truncated: String = result.output.chars().take(2000).collect();
                truncated
            }
        ));
    }

    summary
}
```

The above is a GUIDE — adapt as needed based on actual types. Key points:
- Use `tokio::sync::Semaphore` for concurrency limiting (default 3)
- Emit SubAgent* events at each lifecycle point
- `collect_output` reads full stdout (not streaming — sub-agents run to completion)
- `aggregate_results` formats a summary for the coordinator

### Step 2: Wire into forge-process/src/lib.rs

Add `pub mod concurrent;` and re-export:
```rust
pub use concurrent::{ConcurrentRunner, SubTask, SubTaskResult, aggregate_results};
```

### Step 3: Add Coordinator preset to forge-agent/src/preset.rs

Add a new variant to `AgentPreset`:
```rust
Coordinator,
```

With defaults:
```rust
Self::Coordinator => PresetDefaults {
    system_prompt: "You are a task coordinator. Break down complex tasks into independent sub-tasks that can run in parallel. For each sub-task, specify: the agent type (CodeWriter, Tester, Reviewer, etc.), a clear prompt, and the working directory. After all sub-tasks complete, synthesize their outputs into a coherent final response.".into(),
    model: "claude-sonnet-4-20250514".into(),
    allowed_tools: None,
},
```

Add `Self::Coordinator` to the `all()` array.

### Step 4: Add tests

In concurrent.rs:
- `concurrent_runner_respects_semaphore` — mock test showing semaphore limits concurrency
- `aggregate_results_formats_summary` — test the summary formatter
- `sub_task_result_tracks_success` — basic struct tests

In preset.rs tests (forge-agent):
- Update `all_presets_have_non_empty_system_prompt` — should still pass with Coordinator added
- `coordinator_preset_has_system_prompt` — verify Coordinator defaults

## VERIFY
```bash
cargo check -p forge-process -p forge-agent
cargo test -p forge-process
cargo test -p forge-agent
cargo clippy -p forge-process -p forge-agent
```

## IMPORTANT CONSTRAINTS
- forge-process already has tokio in deps (with sync, rt, macros, test-util features)
- You need `tokio::sync::Semaphore` — check if the `sync` feature is enabled (it is)
- Do NOT add new crate dependencies
- `ProcessHandle` from spawn.rs has: `take_stdout() -> Option<ChildStdout>`, `wait() -> io::Result<ExitStatus>`, `kill() -> io::Result<()>`, `id() -> Option<u32>`
- SpawnConfig has `from_env()` and `.with_working_dir(&str)`
- The `spawn()` function signature: `pub async fn spawn(config: &SpawnConfig, prompt: &str, resume: Option<&str>) -> Result<ProcessHandle, SpawnError>`
- Keep all existing tests passing (12 in forge-process, 8 in forge-agent)
```

---

## Verification Gate (run between waves)

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
