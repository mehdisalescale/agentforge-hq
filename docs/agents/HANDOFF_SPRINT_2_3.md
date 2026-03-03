# Handoff: Sprint 2-3 Parallel Agent Execution

> **Date:** 2026-03-03
> **Prerequisite:** Sprint 1 (v0.2.0) shipped — MCP rewrite complete, bugs fixed
> **Master plan:** `MASTER_TASK_LIST.md`
> **Previous batch:** `docs/agents/HANDOFF_BATCH_2.md` (same pattern)

---

## Execution Protocol

1. **Launch all agents in a wave simultaneously.** Each agent gets its task card below.
2. **Wait for all agents in the wave to complete.**
3. **Run the verification gate** before starting the next wave.
4. **One commit per agent.** Merge all commits, then run gate.
5. **If gate fails:** fix issues before proceeding. Do NOT start the next wave with broken tests.

---

## Wave 1 — Build Components (5 agents, zero file conflicts)

All agents create NEW files/crates. No shared files between agents.

---

### TASK_W1A: forge-git Crate (Agent A)

**Context:** Forge needs git worktree isolation per agent session. This is the industry-standard pattern for multi-agent safety (Claude Code `--worktree`, ccswarm). This crate wraps git commands into a reusable library.

**Files to create/edit (exclusive ownership):**
- `crates/forge-git/Cargo.toml` (NEW)
- `crates/forge-git/src/lib.rs` (NEW)
- `Cargo.toml` — add `"crates/forge-git"` to `[workspace] members` only

**Task:**

1. Create `crates/forge-git/Cargo.toml`:
```toml
[package]
name = "forge-git"
version = "0.1.0"
edition = "2021"

[dependencies]
forge-core = { path = "../forge-core" }
```

2. Create `crates/forge-git/src/lib.rs` with:
```rust
use std::path::{Path, PathBuf};
use std::process::Command;
use forge_core::ForgeError;

pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub session_id: String,
}

/// Create a git worktree for a session.
/// Returns the worktree directory path.
pub fn create_worktree(repo_dir: &Path, session_id: &str) -> Result<PathBuf, ForgeError> {
    let worktree_dir = repo_dir.join(".worktrees").join(session_id);
    let branch = format!("forge/{}", session_id);
    let status = Command::new("git")
        .args(["worktree", "add", &worktree_dir.to_string_lossy(), "-b", &branch])
        .current_dir(repo_dir)
        .status()
        .map_err(|e| ForgeError::Internal(format!("git worktree add failed: {}", e)))?;
    if !status.success() {
        return Err(ForgeError::Internal("git worktree add returned non-zero".into()));
    }
    Ok(worktree_dir)
}

/// Remove a worktree and its branch.
pub fn remove_worktree(repo_dir: &Path, session_id: &str) -> Result<(), ForgeError> {
    let worktree_dir = repo_dir.join(".worktrees").join(session_id);
    let branch = format!("forge/{}", session_id);
    // Remove worktree
    let _ = Command::new("git")
        .args(["worktree", "remove", &worktree_dir.to_string_lossy(), "--force"])
        .current_dir(repo_dir)
        .status();
    // Delete branch
    let _ = Command::new("git")
        .args(["branch", "-D", &branch])
        .current_dir(repo_dir)
        .status();
    Ok(())
}

/// List all forge worktrees.
pub fn list_worktrees(repo_dir: &Path) -> Result<Vec<WorktreeInfo>, ForgeError> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| ForgeError::Internal(format!("git worktree list failed: {}", e)))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse porcelain output: "worktree /path\nHEAD abc123\nbranch refs/heads/forge/xxx\n\n"
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = Some(branch.to_string());
        } else if line.is_empty() {
            if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take()) {
                if let Some(sid) = branch.strip_prefix("forge/") {
                    worktrees.push(WorktreeInfo {
                        path,
                        branch,
                        session_id: sid.to_string(),
                    });
                }
            }
        }
    }
    Ok(worktrees)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn setup_test_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["commit", "--allow-empty", "-m", "init"]).current_dir(dir.path()).output().unwrap();
        dir
    }

    #[test]
    fn create_and_remove_worktree() {
        let repo = setup_test_repo();
        let path = create_worktree(repo.path(), "test-session-1").unwrap();
        assert!(path.exists());
        assert!(path.join(".git").exists());
        remove_worktree(repo.path(), "test-session-1").unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn list_worktrees_finds_forge_branches() {
        let repo = setup_test_repo();
        create_worktree(repo.path(), "sess-a").unwrap();
        create_worktree(repo.path(), "sess-b").unwrap();
        let wts = list_worktrees(repo.path()).unwrap();
        assert_eq!(wts.len(), 2);
        assert!(wts.iter().any(|w| w.session_id == "sess-a"));
        assert!(wts.iter().any(|w| w.session_id == "sess-b"));
        remove_worktree(repo.path(), "sess-a").unwrap();
        remove_worktree(repo.path(), "sess-b").unwrap();
    }
}
```

3. Add `"crates/forge-git"` to workspace members in root `Cargo.toml`.

**Verify:**
```bash
cargo test -p forge-git
cargo clippy -p forge-git -- -D warnings
```

**Report:**
- [ ] Crate compiles
- [ ] Tests pass
- [ ] Notes: ___

---

### TASK_W1B: Middleware Trait + Chain (Agent B)

**Context:** The run handler (`routes/run.rs`) is a 200+ LOC monolith. The middleware pattern (from DeerFlow, 8 real middlewares) makes it extensible. This task creates the trait and chain infrastructure — the actual extraction of logic into middlewares happens in Wave 3.

**Files to create/edit (exclusive ownership):**
- `crates/forge-api/src/middleware.rs` (NEW file)

**DO NOT touch:** `routes/run.rs`, `lib.rs`, `mod.rs` — those are Wave 2 integration work.

**Task:**

1. Create `crates/forge-api/src/middleware.rs`:
```rust
use async_trait::async_trait;
use std::sync::Arc;

/// Context passed through the middleware chain.
pub struct RunContext {
    pub agent_id: String,
    pub prompt: String,
    pub session_id: String,
    pub working_dir: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Response from the middleware chain.
pub struct RunResponse {
    pub session_id: String,
    pub status: String,
}

/// Errors from middleware processing.
#[derive(Debug)]
pub enum MiddlewareError {
    RateLimited,
    CircuitOpen,
    BudgetExceeded { cost: f64, limit: f64 },
    Internal(String),
}

/// The Next function — calls the next middleware in the chain.
pub struct Next<'a> {
    middlewares: &'a [Arc<dyn Middleware>],
    index: usize,
}

impl<'a> Next<'a> {
    pub async fn run(self, ctx: &mut RunContext) -> Result<RunResponse, MiddlewareError> {
        if self.index < self.middlewares.len() {
            let middleware = &self.middlewares[self.index];
            let next = Next {
                middlewares: self.middlewares,
                index: self.index + 1,
            };
            middleware.process(ctx, next).await
        } else {
            // End of chain — return default response
            Ok(RunResponse {
                session_id: ctx.session_id.clone(),
                status: "completed".to_string(),
            })
        }
    }
}

/// Middleware trait — implement this for each concern.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process the request. Call `next.run(ctx)` to continue the chain.
    async fn process(&self, ctx: &mut RunContext, next: Next<'_>) -> Result<RunResponse, MiddlewareError>;

    /// Name for logging/debugging.
    fn name(&self) -> &str;
}

/// Ordered chain of middlewares.
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    pub async fn execute(&self, ctx: &mut RunContext) -> Result<RunResponse, MiddlewareError> {
        let next = Next {
            middlewares: &self.middlewares,
            index: 0,
        };
        next.run(ctx).await
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LogMiddleware { name: String }

    #[async_trait]
    impl Middleware for LogMiddleware {
        async fn process(&self, ctx: &mut RunContext, next: Next<'_>) -> Result<RunResponse, MiddlewareError> {
            ctx.metadata.insert(format!("{}_entered", self.name), "true".into());
            let result = next.run(ctx).await;
            ctx.metadata.insert(format!("{}_exited", self.name), "true".into());
            result
        }
        fn name(&self) -> &str { &self.name }
    }

    struct BlockMiddleware;

    #[async_trait]
    impl Middleware for BlockMiddleware {
        async fn process(&self, _ctx: &mut RunContext, _next: Next<'_>) -> Result<RunResponse, MiddlewareError> {
            Err(MiddlewareError::RateLimited)
        }
        fn name(&self) -> &str { "block" }
    }

    fn test_context() -> RunContext {
        RunContext {
            agent_id: "agent-1".into(),
            prompt: "test".into(),
            session_id: "sess-1".into(),
            working_dir: None,
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn chain_executes_in_order() {
        let mut chain = MiddlewareChain::new();
        chain.add(LogMiddleware { name: "first".into() });
        chain.add(LogMiddleware { name: "second".into() });
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert_eq!(ctx.metadata.get("first_entered"), Some(&"true".to_string()));
        assert_eq!(ctx.metadata.get("second_entered"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn middleware_can_short_circuit() {
        let mut chain = MiddlewareChain::new();
        chain.add(BlockMiddleware);
        chain.add(LogMiddleware { name: "never".into() });
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(matches!(result, Err(MiddlewareError::RateLimited)));
        assert!(ctx.metadata.get("never_entered").is_none());
    }

    #[tokio::test]
    async fn empty_chain_returns_ok() {
        let chain = MiddlewareChain::new();
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
    }
}
```

2. Do NOT add `mod middleware;` to `lib.rs` — that's Wave 2 (Agent F).

**Verify:**
```bash
cargo test -p forge-api
cargo clippy -p forge-api -- -D warnings
```

**Report:**
- [ ] File created and compiles
- [ ] Tests pass
- [ ] Notes: ___

---

### TASK_W1C: Skill Loader + 10 Seed Files (Agent C)

**Context:** The skills table exists in the DB but is empty. DeerFlow has 15 SKILL.md files with a 208-line loader. We replicate that pattern: Markdown files with YAML frontmatter, loaded at startup, stored in the skills table.

**Files to create/edit (exclusive ownership):**
- `skills/*.md` (NEW dir, 10 files)
- `crates/forge-db/src/repos/skills.rs` (existing — add write methods + loader)

**DO NOT touch:** `routes/skills.rs` (already has GET endpoints), `mod.rs`, `lib.rs` — those are Wave 2.

**Task:**

1. Add `upsert` and `load_from_dir` methods to `SkillRepo` in `skills.rs`.

2. Create `skills/` directory with 10 Markdown skill files. Each follows this format:
```markdown
---
name: code-review
description: Thorough code review methodology
tags: [review, quality, code]
tools: [Read, Grep, Glob]
---

# Code Review

## When to Use
Use when asked to review code, find bugs, suggest improvements...

## Methodology
1. Read the code under review
2. Check for correctness, edge cases, error handling
3. Evaluate naming, structure, readability
4. Look for security issues (OWASP top 10)
5. Check test coverage
6. Suggest specific improvements with code examples

## Output Format
- Summary (1-2 sentences)
- Issues found (severity: critical/major/minor)
- Suggestions (with code snippets)
- Overall assessment
```

3. Create all 10 skill files: `deep-research.md`, `code-review.md`, `refactor.md`, `test-writer.md`, `debug.md`, `security-audit.md`, `document.md`, `architect.md`, `explore.md`, `fix-bug.md`.

**Verify:**
```bash
cargo test -p forge-db
ls skills/*.md | wc -l  # should be 10+
```

**Report:**
- [ ] 10 skill files created
- [ ] SkillRepo has upsert + loader methods
- [ ] Tests pass
- [ ] Notes: ___

---

### TASK_W1D: Memory Table + Repo + Routes (Agent D)

**Context:** Cross-session memory lets agents learn from past sessions. This task creates the data layer — table, repo, API routes. The extraction and injection logic comes in Wave 3.

**Files to create/edit (exclusive ownership):**
- `migrations/0003_add_memory.sql` (NEW)
- `crates/forge-db/src/repos/memory.rs` (NEW)
- `crates/forge-api/src/routes/memory.rs` (NEW)

**DO NOT touch:** `migrations.rs`, `repos/mod.rs`, `lib.rs`, `routes/mod.rs`, `state.rs` — those are Wave 2.

**Task:**

1. Create `migrations/0003_add_memory.sql`:
```sql
CREATE TABLE IF NOT EXISTS memory (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL DEFAULT 'general',
    content TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    source_session_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_memory_category ON memory(category);
```

2. Create `crates/forge-db/src/repos/memory.rs` with `MemoryRepo`:
   - `create(category, content, confidence, source_session_id) → Memory`
   - `list(limit, offset) → Vec<Memory>`
   - `get(id) → Option<Memory>`
   - `update(id, content, confidence) → Result`
   - `delete(id) → Result`
   - `search(query) → Vec<Memory>` (LIKE-based for now)

3. Create `crates/forge-api/src/routes/memory.rs` with:
   - `GET /api/v1/memory` → list with pagination
   - `POST /api/v1/memory` → create
   - `GET /api/v1/memory/:id` → get
   - `PUT /api/v1/memory/:id` → update
   - `DELETE /api/v1/memory/:id` → delete

**Verify:**
```bash
cargo test -p forge-db
cargo clippy -p forge-db -- -D warnings
```

**Report:**
- [ ] Migration file created
- [ ] MemoryRepo with CRUD
- [ ] Routes defined
- [ ] Tests pass
- [ ] Notes: ___

---

### TASK_W1E: Hook Table + Repo + Routes (Agent E)

**Context:** Hooks let users run shell commands before/after agent operations (e.g., lint before commit, tests after code change). Pattern from hooks-mastery (13 types) and hooks-observability.

**Files to create/edit (exclusive ownership):**
- `migrations/0004_add_hooks.sql` (NEW)
- `crates/forge-db/src/repos/hooks.rs` (NEW)
- `crates/forge-api/src/routes/hooks.rs` (NEW)

**DO NOT touch:** `migrations.rs`, `repos/mod.rs`, `lib.rs`, `routes/mod.rs`, `state.rs`, `events.rs` — those are Wave 2.

**Task:**

1. Create `migrations/0004_add_hooks.sql`:
```sql
CREATE TABLE IF NOT EXISTS hooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    event_type TEXT NOT NULL,
    timing TEXT NOT NULL CHECK (timing IN ('pre', 'post')),
    command TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

2. Create `crates/forge-db/src/repos/hooks.rs` with `HookRepo`:
   - `create(name, event_type, timing, command) → Hook`
   - `list() → Vec<Hook>`
   - `get(id) → Option<Hook>`
   - `update(id, name?, command?, enabled?) → Result`
   - `delete(id) → Result`
   - `find_by_event(event_type, timing) → Vec<Hook>`

3. Create `HookRunner`:
```rust
pub struct HookRunner;
impl HookRunner {
    pub async fn run_hooks(hooks: &[Hook]) -> Vec<HookResult> {
        // Execute each hook command via Command::new("sh").arg("-c").arg(&hook.command)
        // Capture stdout/stderr, success/failure
        // Return results for each hook
    }
}
```

4. Create `crates/forge-api/src/routes/hooks.rs` with:
   - `GET /api/v1/hooks` → list
   - `POST /api/v1/hooks` → create
   - `GET /api/v1/hooks/:id` → get
   - `PUT /api/v1/hooks/:id` → update
   - `DELETE /api/v1/hooks/:id` → delete

**Verify:**
```bash
cargo test -p forge-db
cargo clippy -p forge-db -- -D warnings
```

**Report:**
- [ ] Migration file created
- [ ] HookRepo with CRUD + find_by_event
- [ ] HookRunner with shell execution
- [ ] Routes defined
- [ ] Tests pass
- [ ] Notes: ___

---

## ⬛ GATE 1

After ALL Wave 1 agents complete:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

All must pass before starting Wave 2. Fix any issues.

---

## Wave 2 — Integration Wiring (1 agent, sequential)

### TASK_W2F: Integration Wiring (Agent F)

**Context:** Wave 1 created isolated components. This agent wires them into the app by touching all the shared files that Wave 1 agents were forbidden from modifying.

**Files to edit (shared — run alone):**
- `crates/forge-db/src/migrations.rs`
- `crates/forge-db/src/repos/mod.rs`
- `crates/forge-db/src/lib.rs`
- `crates/forge-api/src/state.rs`
- `crates/forge-api/src/routes/mod.rs`
- `crates/forge-api/src/lib.rs`
- `crates/forge-core/src/events.rs`
- `crates/forge-app/src/main.rs`

**Task:**

1. `forge-db/src/migrations.rs` — Apply migrations 0003 (memory) and 0004 (hooks). Add to the migration list.

2. `forge-db/src/repos/mod.rs` — Add `pub mod memory;` and `pub mod hooks;`. Export `MemoryRepo` and `HookRepo`.

3. `forge-db/src/lib.rs` — Re-export `MemoryRepo` and `HookRepo`.

4. `forge-api/src/state.rs` — Add to `AppState`:
   - `pub memory_repo: Arc<MemoryRepo>`
   - `pub hook_repo: Arc<HookRepo>`

5. `forge-api/src/routes/mod.rs` — Nest new routes:
   - `.nest("/api/v1/memory", memory::routes())`
   - `.nest("/api/v1/hooks", hooks::routes())`

6. `forge-api/src/lib.rs` — Add `mod middleware;`. Add `pub mod memory;` and `pub mod hooks;` to routes.

7. `forge-core/src/events.rs` — Add event variants:
   - `HookStarted { hook_id, hook_name, event_type, timestamp }`
   - `HookCompleted { hook_id, hook_name, duration_ms, timestamp }`
   - `HookFailed { hook_id, hook_name, error, timestamp }`
   - `SubAgentRequested { parent_session_id, sub_agent_id, prompt, timestamp }`
   - `SubAgentStarted { parent_session_id, sub_agent_id, session_id, timestamp }`
   - `SubAgentCompleted { parent_session_id, sub_agent_id, session_id, timestamp }`
   - `SubAgentFailed { parent_session_id, sub_agent_id, error, timestamp }`
   Update `timestamp()` match and serialization tests.

8. `forge-app/src/main.rs` — Initialize `MemoryRepo` and `HookRepo`, add to `AppState`. Call skill loader at startup.

**Verify:**
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

**Report:**
- [ ] All repos wired into AppState
- [ ] All routes nested
- [ ] All events added
- [ ] Migrations apply
- [ ] Skill loader runs at startup
- [ ] All tests pass
- [ ] Notes: ___

---

## ⬛ GATE 2

```bash
cargo build --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
```

---

## Wave 3 — Feature Logic (3 agents in parallel)

### TASK_W3G: Middleware Extraction + Skill Injection (Agent G)

**Files:** `crates/forge-api/src/middleware.rs` (extend), `crates/forge-api/src/routes/run.rs` (refactor)

**Task:**
1. Create 6 concrete middleware structs in `middleware.rs`:
   - `RateLimitMiddleware` — check token bucket, return `MiddlewareError::RateLimited` if empty
   - `CircuitBreakerMiddleware` — check CB state, return `MiddlewareError::CircuitOpen` if open
   - `SkillInjectionMiddleware` — match prompt keywords against skill tags, append skill body to `ctx.metadata["system_prompt_extra"]`
   - `SpawnMiddleware` — spawn Claude process, stream output, emit events (moves the core logic from run.rs)
   - `PersistMiddleware` — save events via BatchWriter
   - `CostMiddleware` — track cost, check budget warn/limit

2. Refactor `run.rs` to be thin: parse request → build `RunContext` → call `chain.execute(ctx)` → return response.

**Verify:**
```bash
cargo test --workspace
# E2E: start server, run an agent, verify output streams correctly
```

**Report:**
- [ ] 6 middlewares created
- [ ] run.rs refactored to use chain
- [ ] Existing behavior preserved
- [ ] Tests pass
- [ ] Notes: ___

---

### TASK_W3H: Memory Extraction + Injection (Agent H)

**Files:** `crates/forge-db/src/repos/memory.rs` (extend)

**Task:**
1. Add `extract_memories_from_transcript(transcript: &str) → Vec<MemoryFact>`:
   - Send transcript to Claude with extraction prompt
   - Parse structured JSON response: `[{category, content, confidence}]`
   - Return facts for storage

2. Add `get_relevant_memories(prompt: &str, limit: usize) → Vec<Memory>`:
   - Search memories by keyword overlap with prompt
   - Return top N by confidence score
   - Format as context block for system prompt injection

3. Hook into session completion: when a session completes, call `extract_memories_from_transcript` and store results.

**Verify:**
```bash
cargo test -p forge-db
```

**Report:**
- [ ] Extraction function implemented
- [ ] Injection query implemented
- [ ] Tests pass
- [ ] Notes: ___

---

### TASK_W3I: Sub-Agent Runner + Coordinator (Agent I)

**Files:** `crates/forge-process/src/concurrent.rs` (NEW), `crates/forge-agent/src/lib.rs` (add Coordinator)

**Task:**
1. Create `crates/forge-process/src/concurrent.rs`:
   - `ConcurrentRunner` struct: wraps N `ProcessRunner` instances
   - `spawn_parallel(configs: Vec<SpawnConfig>) → Vec<ProcessHandle>`: launch up to N concurrent processes using `tokio::JoinSet`
   - Configurable limit (default 3, from `FORGE_CONCURRENCY_LIMIT` env)
   - Each process runs in its own worktree (path from `SpawnConfig.working_dir`)
   - Aggregate results: collect all outputs, return combined result

2. Add `Coordinator` preset to `crates/forge-agent/src/lib.rs`:
   - System prompt: "You are a coordinator. Analyze the task, decompose into sub-tasks, assign to specialized agents, synthesize results."
   - Allowed tools: all
   - Model: same default

3. Add `pub mod concurrent;` to `crates/forge-process/src/lib.rs`.

**Verify:**
```bash
cargo test -p forge-process
cargo test -p forge-agent
```

**Report:**
- [ ] ConcurrentRunner created
- [ ] Coordinator preset added
- [ ] Tests pass
- [ ] Notes: ___

---

## ⬛ GATE 3

```bash
cargo build --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

---

## Wave 4 — Frontend + Polish (4 agents in parallel)

### TASK_W4J: Worktree UI + Integration Test (Agent J)

**Files:** `frontend/src/routes/sessions/` (modify), `tests/` (NEW)

**Task:**
1. Add worktree info to session detail page: branch name, worktree path, status.
2. Add merge button: calls `POST /api/v1/sessions/:id/merge` (merge worktree branch).
3. Add cleanup button: calls `DELETE /api/v1/sessions/:id/worktree`.
4. Create `tests/integration.rs`: start server → create agent → run prompt → verify session created → verify events in DB.

**Verify:**
```bash
cd frontend && pnpm build && cd ..
cargo test --test integration
```

---

### TASK_W4K: Memory UI + Hook UI (Agent K)

**Files:** `frontend/src/routes/memory/` (NEW), `frontend/src/routes/hooks/` (NEW)

**Task:**
1. Memory page (`/memory`): list facts in table, create form, edit inline, delete with confirm, search box. Use `$state` runes.
2. Hook page (`/hooks`): list hooks in table, create form (name, event_type dropdown, timing radio, command textarea), enable/disable toggle, delete.

**Verify:**
```bash
cd frontend && pnpm build && cd ..
```

---

### TASK_W4L: Multi-Agent Dashboard (Agent L)

**Files:** `frontend/src/routes/+page.svelte` (modify), `frontend/src/lib/api.ts` (extend)

**Task:**
1. When a coordinator agent runs, show per-sub-agent progress panels.
2. Each panel: agent name, status badge (pending/running/completed/failed), output stream.
3. Handle `SubAgentStarted`, `SubAgentCompleted`, `SubAgentFailed` WebSocket events.
4. Add domain badges (code/quality/ops) to agent cards.

**Verify:**
```bash
cd frontend && pnpm build && cd ..
```

---

### TASK_W4M: Polish (Agent M)

**Files:** `frontend/` (various, no overlap with J/K/L), `crates/forge-app/src/main.rs` (shutdown timeout only)

**Task:**
1. Add pagination (limit/offset params) to agents list, sessions list, skills list pages.
2. Add shutdown timeout (10s default) to `main.rs` graceful shutdown.
3. Normalize all pages to use Svelte 5 `$state` runes (Dashboard and Sessions still use `let`).
4. Add loading spinners during API calls on all forms.

**Verify:**
```bash
cd frontend && pnpm build && cd ..
cargo test --workspace
```

---

## ⬛ GATE 4 — Final

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && pnpm build && cd ..
./scripts/e2e-smoke.sh
```

**If all pass:** Tag v0.3.0 (or v0.4.0 if shipping everything together).
