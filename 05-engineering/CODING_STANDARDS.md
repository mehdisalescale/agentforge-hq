# Claude Forge -- Coding Standards

> Conventions for Rust backend (12 workspace crates) and Svelte 5 frontend.
> Every contributor must read this before submitting code.

---

## 1. Rust Standards

### 1.1 Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Crate names | `snake_case`, prefixed `forge-` | `forge-core`, `forge-mcp`, `forge-git` |
| Module files | `snake_case.rs` | `event_bus.rs`, `circuit_breaker.rs` |
| Types (struct, enum, trait) | `PascalCase` | `AgentConfig`, `WorkflowStep`, `EventKind` |
| Functions, methods | `snake_case` | `spawn_agent()`, `flush_events()` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_BATCH_SIZE`, `DEFAULT_PORT` |
| Type parameters | Single uppercase or descriptive | `T`, `E`, `S: State` |
| Feature flags | `kebab-case` | `plugins`, `full-text-search` |
| Error variants | `PascalCase`, descriptive | `AgentNotFound`, `DatabaseWriteFailed` |
| Builder methods | `with_*` for optional, `new()` for required | `AgentBuilder::new(name).with_model(m)` |

### 1.2 Module Organization

Each workspace crate follows this structure:

```
crates/forge-example/
  Cargo.toml
  src/
    lib.rs          # Public API, re-exports, #![forbid(unsafe_code)]
    error.rs        # Crate-specific error type
    types.rs        # Shared types for this crate
    config.rs       # Configuration structs (if applicable)
    mod.rs          # NOT USED -- prefer lib.rs + file-per-module
    tests/          # Integration tests for this crate
      mod.rs
      test_*.rs
```

**Rules:**
- One concept per file. If a file exceeds 400 lines, split it.
- `lib.rs` is the public API surface. Everything else is `pub(crate)` unless explicitly exported.
- No `mod.rs` files (use the `filename.rs` + `filename/` directory pattern).
- Crate-internal helper functions go in a `helpers.rs` or `util.rs` -- never in `lib.rs`.

### 1.3 Error Handling

**Library crates (all 12 workspace crates): `thiserror`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("agent not found: {id}")]
    NotFound { id: uuid::Uuid },

    #[error("agent name already exists: {name}")]
    DuplicateName { name: String },

    #[error("database error")]
    Database(#[from] rusqlite::Error),

    #[error("process spawn failed: {reason}")]
    SpawnFailed { reason: String },
}
```

**Application entry points (main.rs, CLI): `anyhow`**

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("failed to load configuration")?;
    // ...
}
```

**Rules:**
- Never use `unwrap()` in library code. Use `expect()` only for provably infallible cases with a message explaining why.
- Never use `anyhow` in library crates. It erases type information that callers need.
- Every error variant must have a human-readable `#[error("...")]` message.
- Use `#[from]` for automatic conversion from dependency errors.
- Add `.context("what we were doing")` at call sites for error chain richness.
- Return `Result<T, CrateError>` from all fallible public functions.

### 1.4 Async Patterns

```rust
// GOOD: Accept owned data across await points
async fn process_agent(agent: Agent) -> Result<()> { ... }

// BAD: Holding references across await points
async fn process_agent(agent: &Agent) -> Result<()> {
    // If this borrows something behind a Mutex, you'll deadlock
    some_async_call().await; // <- agent ref held across await
}
```

**Rules:**
- Never hold a `DashMap` ref guard across `.await` points. Clone the value, drop the guard, then await.
- Never hold a `MutexGuard` or `RwLockGuard` across `.await`. Use scoped blocks: `{ let val = lock.read().clone(); }` then use `val`.
- Prefer `tokio::spawn` for truly independent background work. Use `tokio::select!` for racing futures.
- All spawned tasks must be tracked (store `JoinHandle`) for graceful shutdown.
- Use `tokio::time::timeout` for any external call (process spawn, HTTP, DB).
- Channel capacities are always explicit: `mpsc::channel(1024)`, never unbounded.

### 1.5 Logging with `tracing`

```rust
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip(db), fields(agent_id = %id))]
pub async fn get_agent(db: &Database, id: Uuid) -> Result<Agent> {
    debug!("fetching agent from database");
    let agent = db.get_agent(id)
        .map_err(|e| {
            error!(error = %e, "database query failed");
            e
        })?;
    info!(name = %agent.name, "agent loaded");
    Ok(agent)
}
```

**Log level guide:**

| Level | Use For |
|-------|---------|
| `error!` | Failures requiring investigation. Lost data, unrecoverable errors. |
| `warn!` | Degraded operation. Circuit breaker open, retry needed, approaching limits. |
| `info!` | Significant state changes. Agent started, workflow completed, server listening. |
| `debug!` | Useful for development. DB queries, cache hits/misses, message routing. |
| `trace!` | Verbose internals. Raw bytes, full payloads, per-event details. |

**Rules:**
- Use `#[instrument]` on all public async functions. `skip` large parameters (bodies, DB handles).
- Include structured fields: `info!(agent_id = %id, status = "running")` not `info!("agent {} is running", id)`.
- Never log secrets. API keys, tokens, and passwords must be wrapped in `secrecy::Secret`.
- Use `%` for Display formatting, `?` for Debug formatting in structured fields.

### 1.6 Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_name_validation_rejects_empty() {
        let result = validate_agent_name("");
        assert!(result.is_err());
        assert!(matches!(result, Err(AgentError::InvalidName { .. })));
    }

    #[tokio::test]
    async fn spawn_agent_creates_process() {
        let db = test_db().await;
        let agent = create_test_agent(&db).await;
        let handle = spawn_agent(&agent).await.unwrap();
        assert!(handle.is_running());
    }
}
```

**Rules:**
- Test function names describe the scenario: `test_name_describes_behavior`, not `test1`, `test_agent`.
- Use `assert!(matches!(...))` for enum variant checking.
- Every public function has at least one test. Error paths are tested too.
- Use test helpers (`test_db()`, `create_test_agent()`) to reduce boilerplate. These live in `crates/forge-test-utils/`.
- Integration tests go in `tests/` directory at crate root.
- `#[tokio::test]` for all async tests.

### 1.7 Documentation

```rust
/// Spawns a new agent process with the given configuration.
///
/// The agent runs as a child process executing `claude -p` with
/// stream-json output. Events are forwarded to the event bus.
///
/// # Errors
///
/// Returns [`AgentError::SpawnFailed`] if the process cannot be started,
/// or [`AgentError::NotFound`] if the agent ID is invalid.
///
/// # Examples
///
/// ```no_run
/// let handle = spawn_agent(&config).await?;
/// handle.send_prompt("Write a function").await?;
/// ```
pub async fn spawn_agent(config: &AgentConfig) -> Result<ProcessHandle, AgentError> {
```

**Rules:**
- All `pub` items have doc comments (`///`).
- Include `# Errors` section for fallible functions.
- Include `# Panics` section if the function can panic (should be rare).
- Module-level docs (`//!`) explain the purpose and key types.
- No `# Examples` required for internal-only functions, but encouraged for complex APIs.

### 1.8 Unsafe Policy

```rust
// In every crate's lib.rs:
#![forbid(unsafe_code)]
```

**No exceptions.** All 12 workspace crates forbid unsafe code. Unsafe exists only in dependencies (rusqlite, git2, wasmtime) which have their own audit processes.

If you believe you need `unsafe`:
1. You are probably wrong. Find a safe alternative.
2. If you are certain, open a design discussion. Document the invariant. Get two approvals.
3. If approved, create a dedicated internal crate (`forge-unsafe-utils`) with `#![deny(unsafe_code)]` (deny, not forbid -- allowing explicit `#[allow]` per function) and exhaustive tests.

### 1.9 Clippy & Formatting

```toml
# .clippy.toml (workspace root)
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 8
```

**Enforced lints:**
```rust
// workspace Cargo.toml [workspace.lints.clippy]
pedantic = "warn"
nursery = "warn"
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"           // No panics in library code
todo = "warn"            // TODOs are flagged in CI
dbg_macro = "deny"       // No dbg! in committed code
print_stdout = "deny"    // Use tracing, not println
print_stderr = "deny"
```

**Formatting:**
- `cargo fmt` with default settings. No custom `rustfmt.toml`.
- CI fails on unformatted code.
- Imports sorted: std, external crates, workspace crates, crate-internal.

---

## 2. Svelte / TypeScript Standards

### 2.1 Component Naming

| Item | Convention | Example |
|------|-----------|---------|
| Components | `PascalCase.svelte` | `AgentCard.svelte`, `WorkflowEditor.svelte` |
| Route pages | `+page.svelte` (SvelteKit) | `routes/agents/+page.svelte` |
| Layout files | `+layout.svelte` | `routes/+layout.svelte` |
| Utility modules | `camelCase.ts` | `apiClient.ts`, `eventBus.ts` |
| Type files | `camelCase.ts` or `types/index.ts` | `types/index.ts`, `types/agent.ts` |
| Shared state modules | `camelCase.svelte.ts` | `agentState.svelte.ts` |
| CSS classes | Tailwind utilities | `class="flex items-center gap-2"` |

### 2.2 Svelte 5 Rune Patterns

```svelte
<script lang="ts">
  // Props using $props()
  let { agent, onDelete }: {
    agent: Agent;
    onDelete: (id: string) => void;
  } = $props();

  // Reactive state
  let isEditing = $state(false);
  let formData = $state({ name: agent.name, model: agent.model });

  // Derived values (replaces $: reactive statements)
  let isValid = $derived(formData.name.length > 0);
  let displayName = $derived(formData.name || 'Unnamed Agent');

  // Side effects
  $effect(() => {
    // Runs when dependencies change
    console.log(`Agent name changed to: ${formData.name}`);
  });

  // Event handlers are plain functions
  function handleSave() {
    if (!isValid) return;
    // save logic
    isEditing = false;
  }
</script>
```

**Rules:**
- `$state()` for all mutable component state. Never use `let x = value` for reactive data.
- `$derived()` for all computed values. Never use `$:` reactive declarations (legacy Svelte 4).
- `$effect()` sparingly. Most components should not need effects. If you find yourself writing many effects, refactor.
- `$props()` for all component inputs. Destructure with types.
- No `export let` (Svelte 4 syntax). Use `$props()` exclusively.
- No writable/readable stores. Use `.svelte.ts` modules with `$state` for shared state.

### 2.3 Shared State (Replacing Stores)

```typescript
// agentState.svelte.ts
import type { Agent } from './types';

let agents = $state<Agent[]>([]);
let selectedId = $state<string | null>(null);

export const agentState = {
  get agents() { return agents; },
  get selectedId() { return selectedId; },
  get selectedAgent() {
    return agents.find(a => a.id === selectedId) ?? null;
  },

  async load() {
    const response = await fetch('/api/agents');
    agents = await response.json();
  },

  select(id: string) {
    selectedId = id;
  },
};
```

### 2.4 Event Handling (Svelte 5)

```svelte
<!-- CORRECT: Svelte 5 event handling -->
<button onclick={handleClick}>Click</button>
<button onclick={(e) => { e.stopPropagation(); handleClick(); }}>Click</button>
<input oninput={(e) => { value = e.currentTarget.value; }} />

<!-- WRONG: Svelte 4 event handling (do not use) -->
<button on:click={handleClick}>Click</button>
<button on:click|stopPropagation={handleClick}>Click</button>
```

**Rules:**
- Use `onclick`, `oninput`, `onsubmit` etc. (lowercase, no colon).
- Modifiers (stopPropagation, preventDefault) are called explicitly inside the handler.
- Custom events from child components: pass callback props, not `createEventDispatcher`.
- Component re-initialization: wrap in `{#key value}` block when a component must remount on data change.

### 2.5 TypeScript Conventions

```typescript
// Types (not interfaces) for data shapes
type Agent = {
  id: string;
  name: string;
  model: string;
  systemPrompt: string | null;
  createdAt: string;
};

// Enums as const objects (not TypeScript enum)
const AgentStatus = {
  Idle: 'idle',
  Running: 'running',
  Error: 'error',
} as const;
type AgentStatus = typeof AgentStatus[keyof typeof AgentStatus];

// API client functions
async function fetchAgents(): Promise<Agent[]> {
  const response = await fetch('/api/agents');
  if (!response.ok) throw new Error(`Failed to fetch agents: ${response.status}`);
  return response.json();
}
```

**Rules:**
- Use `type` over `interface` for data shapes. `interface` only for implementing contracts.
- Use `as const` objects instead of TypeScript `enum` (better tree-shaking, more predictable).
- All API functions return typed Promises. No `any`.
- Explicit return types on exported functions.
- Nullability: use `T | null`, never `T | undefined` for API data. Use `undefined` only for optional parameters.

### 2.6 CSS & Styling

```svelte
<!-- Prefer Tailwind utilities -->
<div class="flex items-center gap-2 rounded-lg bg-gray-800 p-4">
  <span class="text-sm font-medium text-gray-300">{agent.name}</span>
</div>

<!-- For complex/reusable styles, use @apply in component <style> -->
<style>
  .agent-card {
    @apply flex items-center gap-2 rounded-lg bg-gray-800 p-4
           transition-colors hover:bg-gray-700;
  }
</style>
```

**Rules:**
- Tailwind utilities first. Inline `class="..."` for most styling.
- Component `<style>` blocks for complex, reusable patterns within a component.
- No global CSS except in `app.css` (Tailwind base, custom properties, font imports).
- Dark theme is the default and only theme.
- Consistent spacing scale: use Tailwind's default (4px base: `gap-1` = 4px, `gap-2` = 8px, etc.).
- No `!important`. If specificity issues arise, restructure the HTML.

---

## 3. Git Standards

### 3.1 Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**

| Type | Usage |
|------|-------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | Performance improvement |
| `test` | Adding or updating tests |
| `docs` | Documentation changes |
| `chore` | Build, CI, dependency updates |
| `style` | Formatting, whitespace (no code change) |

**Scopes:** Crate name or `frontend`, e.g., `feat(forge-mcp): add list-tools resource`.

**Rules:**
- Subject line: imperative mood, lowercase, no period, max 72 chars.
- Body: Explain *why*, not *what*. The diff shows *what*.
- Footer: `Closes #123`, `Breaking-Change: ...`, `Co-Authored-By: ...`.
- One logical change per commit. Refactors and features are separate commits.

**Examples:**
```
feat(forge-workflow): add parallel step execution

Steps within a parallel block now execute concurrently using
tokio::JoinSet. Each step gets its own span for tracing.

Closes #47
```

```
fix(forge-db): prevent WAL checkpoint during batch write

The WAL checkpoint was racing with batch inserts, causing
SQLITE_BUSY errors under load. Now checkpoint is deferred
until the batch transaction commits.
```

### 3.2 Branch Naming

```
<type>/<short-description>

feat/workflow-parallel-steps
fix/wal-checkpoint-race
refactor/extract-event-bus-crate
chore/update-wasmtime-27
```

**Rules:**
- Always branch from `main`.
- Delete branches after merge.
- No long-lived feature branches. Prefer feature flags for in-progress work.

### 3.3 Pull Request Template

```markdown
## Summary
<!-- 1-3 sentences: what and why -->

## Changes
<!-- Bullet list of specific changes -->
-
-

## Testing
<!-- How was this tested? -->
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed

## Screenshots
<!-- If frontend changes, include before/after -->

## Checklist
- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] No new `unwrap()` in library code
- [ ] Error types use `thiserror`
- [ ] Public functions have doc comments
- [ ] No secrets in code or logs
```

---

## 4. Code Review Checklist

### 4.1 Rust Review

- [ ] No `unwrap()` or `panic!` in library code
- [ ] Error types are specific (not `anyhow` in libraries)
- [ ] No `DashMap` guards or `Mutex` guards held across `.await`
- [ ] Channel capacities are bounded and explicit
- [ ] Spawned tasks store `JoinHandle` for shutdown
- [ ] `tracing` spans on public async functions
- [ ] No `println!` / `eprintln!` (use `tracing` macros)
- [ ] No `dbg!` macros
- [ ] Public items have doc comments
- [ ] New types implement `Debug`
- [ ] Errors implement `Display` via `thiserror`
- [ ] Feature flags used for optional large dependencies
- [ ] Tests cover happy path and at least one error path
- [ ] `clippy::pedantic` warnings addressed or explicitly allowed with reason

### 4.2 Svelte Review

- [ ] Uses `$state()`, `$derived()`, `$props()` (no legacy patterns)
- [ ] No `on:click` syntax (use `onclick`)
- [ ] No `export let` (use `$props()`)
- [ ] No writable/readable stores (use `.svelte.ts` state modules)
- [ ] TypeScript types for all props and API responses
- [ ] No `any` types
- [ ] Loading and error states handled in UI
- [ ] Accessibility: buttons have labels, images have alt text, focus management
- [ ] Responsive layout tested at minimum 1024px width
- [ ] No inline styles (use Tailwind)

### 4.3 General Review

- [ ] No secrets, API keys, or credentials in code
- [ ] No `TODO` or `FIXME` without an associated issue number
- [ ] Commit messages follow format
- [ ] PR description explains *why*, not just *what*
- [ ] Breaking changes documented in PR description and commit footer

---

## 5. Anti-Patterns (What NOT to Do)

### 5.1 Rust Anti-Patterns

**Do not use `clone()` to silence the borrow checker without understanding why.**
```rust
// BAD: Cloning to make the compiler happy
let data = shared_state.clone(); // Why? Is this expensive? Is it needed?

// GOOD: Clone intentionally with a comment
// Clone the agent config to move into the spawned task.
// The original stays in the DashMap for other readers.
let config = agent.config.clone();
tokio::spawn(async move { run_agent(config).await });
```

**Do not use `String` where an enum would be better.**
```rust
// BAD
fn set_status(status: &str) { ... } // "runnin" typo compiles fine

// GOOD
enum Status { Idle, Running, Error }
fn set_status(status: Status) { ... }
```

**Do not use `Box<dyn Error>` as a return type.**
```rust
// BAD: Callers cannot match on error variants
fn do_thing() -> Result<(), Box<dyn std::error::Error>> { ... }

// GOOD: Typed errors
fn do_thing() -> Result<(), ForgeError> { ... }
```

**Do not put business logic in Axum handlers.**
```rust
// BAD: Handler does everything
async fn create_agent(Json(body): Json<CreateAgent>) -> impl IntoResponse {
    let conn = get_db();
    conn.execute("INSERT INTO agents ...", params![body.name])?;
    // 50 more lines of logic
}

// GOOD: Handler is thin, delegates to service
async fn create_agent(
    State(ctx): State<AppContext>,
    Json(body): Json<CreateAgentRequest>,
) -> Result<Json<Agent>, AppError> {
    let agent = ctx.agent_service.create(body).await?;
    Ok(Json(agent))
}
```

**Do not use `lazy_static!` or `once_cell` for mutable global state.**
Application state flows through Axum's `State` extractor. Global mutable state makes testing impossible and creates hidden coupling.

### 5.2 Svelte Anti-Patterns

**Do not use Svelte 4 patterns.**
```svelte
<!-- BAD: Svelte 4 -->
<script>
  export let agent;     // Use $props()
  $: name = agent.name; // Use $derived()
  let count = 0;        // Use $state()
</script>
<button on:click={handler}>  <!-- Use onclick -->

<!-- GOOD: Svelte 5 -->
<script lang="ts">
  let { agent }: { agent: Agent } = $props();
  let name = $derived(agent.name);
  let count = $state(0);
</script>
<button onclick={handler}>
```

**Do not fetch data in `$effect()` without guards.**
```svelte
<!-- BAD: Infinite loop risk -->
<script lang="ts">
  let data = $state([]);
  $effect(() => {
    fetch('/api/data').then(r => r.json()).then(d => { data = d; });
    // data change triggers effect again -> infinite loop
  });
</script>

<!-- GOOD: Fetch in onMount or load function -->
<script lang="ts">
  import { onMount } from 'svelte';
  let data = $state([]);
  onMount(async () => {
    data = await fetch('/api/data').then(r => r.json());
  });
</script>
```

**Do not create deeply nested component hierarchies.** If a component tree is more than 4 levels deep, flatten it. Pass data explicitly via props, not through many layers.

### 5.3 Architecture Anti-Patterns

**Do not import between crates that are at the same layer.** Dependency flows downward: `forge-api` depends on `forge-core`, but `forge-core` never depends on `forge-api`. See `DEPENDENCY_GRAPH.md`.

**Do not put SQL in service logic.** SQL lives in repository modules within `forge-db`. Services call typed repository methods.

**Do not bypass the event bus for inter-component communication.** All significant state changes emit events. Components react to events, not direct function calls between unrelated modules.

**Do not add dependencies without checking binary size impact.** Run `cargo bloat --release --crates` before and after. If a new crate adds > 500 KB, justify it in the PR.
