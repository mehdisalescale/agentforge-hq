# Claude Forge -- Testing Strategy

> Comprehensive testing plan for 12 workspace crates + Svelte 5 frontend.
> Target: high confidence in correctness with fast feedback loops.

---

## 1. Testing Philosophy

1. **Tests are documentation.** A test name should describe behavior in plain language.
2. **Fast tests run first.** Unit tests gate every commit. Slow tests run in CI.
3. **Test the contract, not the implementation.** Public API behavior matters. Internal refactors should not break tests.
4. **Every bug gets a regression test.** Before fixing, write a test that reproduces the failure.
5. **No flaky tests.** A flaky test is worse than no test. If a test intermittently fails, fix it or delete it. Never ignore.

---

## 2. Test Pyramid

```
                    /\
                   /  \          E2E Tests
                  / 5% \         (Playwright)
                 /------\
                /        \       Integration Tests
               /   15%    \      (cross-crate, API, DB)
              /------------\
             /              \    Unit Tests
            /      80%       \   (per-crate, pure functions)
           /------------------\
```

| Layer | Count Target | Execution Time | Runs When |
|-------|-------------|---------------|-----------|
| Unit tests | ~500+ | < 30 seconds total | Every commit (pre-push hook) |
| Integration tests | ~100+ | < 2 minutes total | Every PR, CI pipeline |
| E2E tests | ~30+ | < 5 minutes total | Nightly, pre-release |

---

## 3. Unit Tests

### 3.1 Scope

Unit tests validate individual functions, methods, and types within a single crate. They do not touch the filesystem, network, or database.

### 3.2 Per-Crate Coverage Targets

| Crate | Min Coverage | Focus Areas |
|-------|-------------|-------------|
| `forge-core` | 90% | Event types, serialization, validation, ID generation |
| `forge-db` | 85% | SQL query building, migration logic, batch writer logic |
| `forge-mcp` | 85% | Message parsing, tool registry, protocol compliance |
| `forge-workflow` | 90% | Step execution order, condition evaluation, state machine transitions |
| `forge-skills` | 80% | Skill registry, search/filter, parameter validation |
| `forge-git` | 75% | Status parsing, diff formatting (lower because git2 is hard to mock) |
| `forge-safety` | 95% | Circuit breaker states, rate limiter math, cost calculations |
| `forge-notify` | 80% | Message formatting, channel routing, rate limiting |
| `forge-scheduler` | 90% | Cron parsing, next-run calculation, job state machine |
| `forge-plugins` | 80% | WASM host functions, resource limits, API surface |
| `forge-observe` | 75% | Metric aggregation, span formatting |
| `forge-api` | 70% | Request validation, response formatting (handlers are thin) |

### 3.3 Mocking Strategy

**Traits for dependency injection:**
```rust
// Define trait in the crate that needs the abstraction
pub trait EventSink: Send + Sync {
    fn emit(&self, event: Event) -> Result<(), EventError>;
}

// Real implementation
pub struct BroadcastEventSink { /* ... */ }
impl EventSink for BroadcastEventSink { /* ... */ }

// Test mock
#[cfg(test)]
pub struct MockEventSink {
    pub events: std::sync::Mutex<Vec<Event>>,
}

#[cfg(test)]
impl EventSink for MockEventSink {
    fn emit(&self, event: Event) -> Result<(), EventError> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}
```

**Rules:**
- No mocking framework (no `mockall`, no `mockito`). Hand-written mocks using trait impls.
- Mocks live in `#[cfg(test)]` modules within the same crate.
- Shared test utilities (test DB, fixtures, builders) live in `crates/forge-test-utils/`.
- If a function is hard to test, it is a design smell -- refactor it.

### 3.4 Test Organization

```rust
// In src/circuit_breaker.rs

#[cfg(test)]
mod tests {
    use super::*;

    // Group by behavior, not by function name
    mod when_closed {
        use super::*;

        #[test]
        fn allows_requests_through() { /* ... */ }

        #[test]
        fn opens_after_failure_threshold() { /* ... */ }

        #[test]
        fn resets_failure_count_on_success() { /* ... */ }
    }

    mod when_open {
        use super::*;

        #[test]
        fn rejects_requests_immediately() { /* ... */ }

        #[test]
        fn transitions_to_half_open_after_timeout() { /* ... */ }
    }

    mod when_half_open {
        use super::*;

        #[test]
        fn closes_on_success() { /* ... */ }

        #[test]
        fn opens_on_failure() { /* ... */ }
    }
}
```

---

## 4. Integration Tests

### 4.1 Cross-Crate Tests

Integration tests validate interactions between crates. They live in a dedicated `tests/` workspace member.

```
crates/forge-integration-tests/
  Cargo.toml           # Depends on all workspace crates
  tests/
    agent_lifecycle.rs # Create -> spawn -> event -> persist
    workflow_run.rs    # Workflow engine + event bus + DB
    mcp_roundtrip.rs   # MCP server receives request -> executes tool -> returns result
    skill_search.rs    # Skill catalog + FTS5 search + API response
```

**Key scenarios:**

| Test | Crates Involved | What It Validates |
|------|----------------|-------------------|
| Agent lifecycle | core, db, api | Create agent, update, spawn, receive events, query history |
| Workflow execution | workflow, core, db | Define workflow, execute steps, handle failures, persist state |
| MCP roundtrip | mcp, core, api | Client connects, lists tools, calls tool, receives result |
| Event persistence | core, db | Events emitted -> batch writer -> queryable in DB |
| Skill search | skills, db | Index 1537 skills, FTS5 search, ranked results |
| Circuit breaker | safety, core | Failure threshold -> open -> half-open -> close |
| Git operations | git, api | Status, diff, log, branch operations via libgit2 |

### 4.2 Database Tests

```rust
use forge_test_utils::TestDb;

#[tokio::test]
async fn batch_writer_flushes_on_threshold() {
    let db = TestDb::new().await; // In-memory SQLite, migrations applied
    let writer = BatchWriter::new(db.pool(), BatchConfig {
        max_batch_size: 5,
        flush_interval: Duration::from_secs(60), // Long, so we test threshold
    });

    for i in 0..5 {
        writer.enqueue(test_event(i)).await;
    }

    // Batch should auto-flush at 5 events
    tokio::time::sleep(Duration::from_millis(50)).await;
    let count = db.count_events().await;
    assert_eq!(count, 5);
}
```

**Database test rules:**
- Every database test gets a fresh in-memory SQLite instance.
- Migrations run automatically in `TestDb::new()`.
- No shared state between database tests. Tests run in parallel safely.
- Test the migration chain: a test that applies all migrations to an empty DB and validates the schema.

### 4.3 API Tests

```rust
use axum::http::StatusCode;
use axum_test::TestServer; // from axum-test crate

#[tokio::test]
async fn create_agent_returns_201() {
    let app = test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/agents")
        .json(&json!({
            "name": "Test Agent",
            "model": "claude-sonnet-4-20250514",
            "system_prompt": "You are a test agent."
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let agent: Agent = response.json();
    assert_eq!(agent.name, "Test Agent");
    assert!(uuid::Uuid::parse_str(&agent.id).is_ok());
}

#[tokio::test]
async fn create_agent_validates_name() {
    let app = test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/agents")
        .json(&json!({ "name": "", "model": "claude-sonnet-4-20250514" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}
```

**API test coverage matrix:**

| Endpoint | Happy Path | Validation Error | Not Found | Auth (future) |
|----------|-----------|-----------------|-----------|---------------|
| `POST /api/agents` | 201 + body | 422 + error detail | N/A | 401 |
| `GET /api/agents` | 200 + list | N/A | N/A | 401 |
| `GET /api/agents/:id` | 200 + body | 400 (bad UUID) | 404 | 401 |
| `PATCH /api/agents/:id` | 200 + body | 422 | 404 | 401 |
| `DELETE /api/agents/:id` | 204 | N/A | 404 | 401 |
| `POST /api/agents/:id/run` | 202 + handle | 422 | 404 | 401 |
| `GET /api/sessions` | 200 + list | N/A | N/A | 401 |
| `GET /api/workflows` | 200 + list | N/A | N/A | 401 |
| `POST /api/workflows/:id/run` | 202 | 422 | 404 | 401 |
| `GET /api/skills` | 200 + list | N/A | N/A | 401 |
| `GET /api/skills/search` | 200 + results | 400 (empty q) | N/A | 401 |
| `WS /api/ws` | Upgrade + events | N/A | N/A | 401 |

---

## 5. MCP Tests

### 5.1 Protocol Compliance

```rust
#[tokio::test]
async fn mcp_initialize_handshake() {
    let server = test_mcp_server().await;

    let response = server.handle_message(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        }
    })).await;

    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert!(response["result"]["capabilities"]["tools"].is_object());
}
```

### 5.2 Tool Validation

For each MCP tool:
- Input schema validation (required fields, types, constraints)
- Successful execution with valid input
- Graceful error on invalid input
- Timeout handling for long-running tools
- Resource cleanup after tool execution

### 5.3 MCP Test Inventory

| Tool Category | Tools | Test Count |
|--------------|-------|-----------|
| Agent management | create, list, update, delete, run | 15 |
| Session management | list, get, search | 9 |
| Workflow management | create, list, run, status | 12 |
| Skill operations | search, get, execute | 9 |
| Git operations | status, diff, log, branch | 12 |
| Configuration | get, set, reset | 6 |
| **Total** | | **~63** |

---

## 6. Frontend Tests

### 6.1 Component Tests (Vitest + @testing-library/svelte)

```typescript
// AgentCard.test.ts
import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import AgentCard from './AgentCard.svelte';

describe('AgentCard', () => {
  const mockAgent = {
    id: '123',
    name: 'Test Agent',
    model: 'claude-sonnet-4-20250514',
    status: 'idle',
  };

  it('renders agent name and model', () => {
    render(AgentCard, { props: { agent: mockAgent } });
    expect(screen.getByText('Test Agent')).toBeInTheDocument();
    expect(screen.getByText('claude-sonnet-4-20250514')).toBeInTheDocument();
  });

  it('calls onDelete when delete button is clicked', async () => {
    const onDelete = vi.fn();
    render(AgentCard, { props: { agent: mockAgent, onDelete } });

    await fireEvent.click(screen.getByRole('button', { name: /delete/i }));
    expect(onDelete).toHaveBeenCalledWith('123');
  });

  it('shows running indicator when status is running', () => {
    render(AgentCard, {
      props: { agent: { ...mockAgent, status: 'running' } }
    });
    expect(screen.getByTestId('status-indicator')).toHaveClass('bg-green-500');
  });
});
```

**Component test coverage targets:**

| Component Category | Coverage | Focus |
|-------------------|----------|-------|
| Agent components | 85% | CRUD operations, status display |
| Workflow components | 80% | Step rendering, state transitions |
| Skill components | 75% | Search, filtering, display |
| Layout components | 70% | Navigation, responsive behavior |
| Shared components | 90% | Buttons, inputs, modals -- high reuse |

### 6.2 E2E Tests (Playwright)

```typescript
// tests/e2e/agent-management.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Agent Management', () => {
  test('create a new agent', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-testid="create-agent-btn"]');
    await page.fill('[data-testid="agent-name-input"]', 'E2E Test Agent');
    await page.selectOption('[data-testid="model-select"]', 'claude-sonnet-4-20250514');
    await page.click('[data-testid="save-agent-btn"]');

    await expect(page.locator('text=E2E Test Agent')).toBeVisible();
  });

  test('run agent and see output', async ({ page }) => {
    await page.goto('/agents');
    await page.click('text=E2E Test Agent');
    await page.fill('[data-testid="prompt-input"]', 'Say hello');
    await page.click('[data-testid="run-btn"]');

    // Wait for WebSocket event
    await expect(page.locator('[data-testid="agent-output"]')).toContainText(
      /.+/,  // Any non-empty output
      { timeout: 30000 }
    );
  });
});
```

**E2E test scenarios:**

| Scenario | Priority | Estimated Duration |
|----------|----------|-------------------|
| Agent CRUD | P0 | 15s |
| Agent run + output | P0 | 30s |
| Workflow create + execute | P0 | 30s |
| Skill search + view | P1 | 10s |
| Session browser | P1 | 10s |
| Settings management | P1 | 10s |
| Git status + diff viewer | P1 | 15s |
| Multi-pane layout | P2 | 10s |
| WebSocket reconnection | P2 | 20s |
| Export (JSON + Markdown) | P2 | 10s |

**E2E environment:**
- Playwright starts the Forge binary with `--test` flag (uses in-memory DB, mock claude CLI).
- A mock `claude` binary (`scripts/mock-claude.sh`) returns canned stream-json output.
- Tests run against the real embedded frontend (no mocking of UI components).

---

## 7. Performance Tests

### 7.1 Benchmarks (criterion)

```rust
// benches/batch_writer.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_batch_write(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(setup_test_db());

    c.bench_function("batch_write_50_events", |b| {
        b.iter(|| {
            rt.block_on(async {
                let events = generate_events(50);
                write_batch(&db, &events).await.unwrap();
            });
        });
    });
}

criterion_group!(benches, bench_batch_write);
criterion_main!(benches);
```

**Hot paths to benchmark:**

| Benchmark | Target | What It Measures |
|-----------|--------|-----------------|
| `batch_write_50_events` | < 50 ms | SQLite transaction with 50 INSERT statements |
| `batch_write_100_events` | < 90 ms | Scaling behavior of batch writes |
| `fts5_search_1500_skills` | < 20 ms | Full-text search across skill catalog |
| `event_serialize` | < 1 us/event | serde_json serialization of Event struct |
| `event_deserialize` | < 2 us/event | serde_json deserialization |
| `websocket_fanout_100` | < 5 ms | Broadcasting one event to 100 WebSocket clients |
| `circuit_breaker_check` | < 100 ns | Hot-path check when circuit is closed |
| `agent_spawn` | < 200 ms | Time from request to child process running |
| `api_response_simple` | < 5 ms p99 | GET /api/agents with 50 agents in DB |
| `git_status_large_repo` | < 500 ms | git2 status on a repo with 10K files |

### 7.2 Load Tests

Using `k6` or `wrk` for HTTP load testing:

```javascript
// k6/load-test.js
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: 50,
  duration: '30s',
};

export default function () {
  const res = http.get('http://localhost:4173/api/agents');
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 50ms': (r) => r.timings.duration < 50,
  });
}
```

**Load test targets:**

| Scenario | Concurrent Users | Target RPS | Target p99 Latency |
|----------|-----------------|------------|-------------------|
| Agent list | 50 | 1000 | 20 ms |
| Skill search | 50 | 500 | 50 ms |
| Session history | 20 | 200 | 100 ms |
| WebSocket connect | 100 | N/A | 50 ms (handshake) |
| Mixed workload | 50 | 500 | 50 ms |

---

## 8. Test Data Management

### 8.1 Fixtures

```rust
// crates/forge-test-utils/src/fixtures.rs

pub fn test_agent() -> Agent {
    Agent {
        id: Uuid::new_v7(Timestamp::now(NoContext)),
        name: "Test Agent".into(),
        model: "claude-sonnet-4-20250514".into(),
        system_prompt: Some("You are a test agent.".into()),
        working_directory: Some("/tmp/test".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn test_event(sequence: u64) -> Event {
    Event {
        id: Uuid::new_v7(Timestamp::now(NoContext)),
        agent_id: Uuid::nil(),
        sequence,
        kind: EventKind::AssistantMessage {
            content: format!("Test message {sequence}"),
        },
        timestamp: Utc::now(),
    }
}
```

### 8.2 Builders (for complex objects)

```rust
pub struct AgentBuilder {
    name: String,
    model: String,
    system_prompt: Option<String>,
    tools: Vec<String>,
    mcp_servers: Vec<McpServerConfig>,
}

impl AgentBuilder {
    pub fn new(name: &str) -> Self { /* defaults */ }
    pub fn with_model(mut self, model: &str) -> Self { /* ... */ }
    pub fn with_prompt(mut self, prompt: &str) -> Self { /* ... */ }
    pub fn with_tool(mut self, tool: &str) -> Self { /* ... */ }
    pub fn build(self) -> Agent { /* ... */ }
}

// Usage:
let agent = AgentBuilder::new("Research Agent")
    .with_model("claude-opus-4-20250514")
    .with_tool("web_search")
    .build();
```

### 8.3 Test Database

```rust
pub struct TestDb {
    pool: Pool<SqliteConnectionManager>,
}

impl TestDb {
    pub async fn new() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder().max_size(1).build(manager).unwrap();
        run_migrations(&pool).await.unwrap();
        Self { pool }
    }

    pub async fn with_agents(mut self, agents: &[Agent]) -> Self {
        for agent in agents {
            insert_agent(&self.pool, agent).await.unwrap();
        }
        self
    }

    pub async fn with_events(mut self, events: &[Event]) -> Self {
        for event in events {
            insert_event(&self.pool, event).await.unwrap();
        }
        self
    }
}
```

### 8.4 Skill Catalog Test Data

The full 1,537-skill catalog is too large for unit tests. Strategy:
- **Unit tests:** Use a 10-skill subset covering all categories.
- **Integration tests:** Use a 100-skill subset with realistic FTS5 data.
- **E2E tests:** Load the full catalog from the embedded JSON.

---

## 9. CI Test Pipeline

### 9.1 Pipeline Stages

```yaml
# .github/workflows/ci.yml (conceptual)
stages:
  - lint:        # 1 min
      - cargo fmt --check
      - cargo clippy -- -D warnings
      - pnpm lint (frontend)
      - cargo deny check

  - unit-test:   # 2 min
      - cargo nextest run --workspace
      - pnpm test (frontend component tests)

  - integration: # 3 min
      - cargo nextest run -p forge-integration-tests
      - API endpoint tests

  - build:       # 5 min
      - pnpm build (frontend)
      - cargo build --release
      - Binary size check (fail if > 50 MB)

  - e2e:         # 5 min (nightly only)
      - Start forge binary
      - Playwright test suite
      - Performance smoke tests

  - coverage:    # 3 min (weekly)
      - cargo llvm-cov --workspace
      - Upload to coverage tracker
      - Fail if below thresholds
```

### 9.2 CI Rules

| Rule | Enforcement |
|------|------------|
| All tests pass | PR merge blocked on failure |
| No clippy warnings | `-- -D warnings` flag |
| Code formatted | `cargo fmt --check` |
| No new `unwrap()` | Custom lint via clippy config |
| Binary size < 50 MB | Script checks release binary |
| No dependency vulnerabilities | `cargo deny check advisories` |
| No unapproved licenses | `cargo deny check licenses` |
| Coverage does not decrease | Diff coverage > 80% on new code |

### 9.3 Pre-Push Hook

```bash
#!/bin/bash
# .git/hooks/pre-push
set -e
echo "Running pre-push checks..."
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace --status-level fail
echo "All checks passed."
```

---

## 10. Coverage Targets

### 10.1 Overall Targets

| Metric | Target | Hard Minimum |
|--------|--------|-------------|
| Line coverage (Rust) | 80% | 70% |
| Branch coverage (Rust) | 70% | 60% |
| Line coverage (Frontend) | 75% | 65% |
| New code coverage (PR diff) | 85% | 80% |

### 10.2 Per-Module Targets

| Module | Line Target | Rationale |
|--------|------------|-----------|
| `forge-core` | 90% | Foundational types and logic, everything depends on it |
| `forge-safety` | 95% | Safety-critical: circuit breaker, rate limiter, cost tracking |
| `forge-workflow` | 90% | Complex state machine, many edge cases |
| `forge-scheduler` | 90% | Time-based logic is tricky, needs thorough testing |
| `forge-db` | 85% | SQL correctness, migration chain, batch writer |
| `forge-mcp` | 85% | Protocol compliance, tool execution |
| `forge-skills` | 80% | Search, filtering, catalog management |
| `forge-notify` | 80% | Channel routing, message formatting |
| `forge-plugins` | 80% | WASM host, resource limits |
| `forge-git` | 75% | Lower because libgit2 operations need real repos |
| `forge-observe` | 75% | Metric aggregation, formatting |
| `forge-api` | 70% | Handlers are thin; most logic is in services |

### 10.3 What We Do NOT Test

- **Third-party library internals.** We test our usage of `rusqlite`, not `rusqlite` itself.
- **Generated code.** Serde derive output, clap derive output.
- **Exact UI pixel positioning.** We test behavior, not layout.
- **Claude CLI behavior.** We test our process management, not the `claude` binary. E2E tests use a mock.
- **Network conditions.** We do not test flaky networks in CI. Timeout handling is unit-tested.

---

## 11. Test Environment Requirements

| Environment | Database | Claude CLI | Git Repos | Network |
|-------------|----------|-----------|-----------|---------|
| Unit tests | In-memory SQLite | Not used | Not used | Not used |
| Integration tests | In-memory SQLite | Mock binary | Temp git init | Localhost only |
| API tests | In-memory SQLite | Not used | Not used | Localhost only |
| E2E tests | File-based SQLite (temp) | Mock binary | Temp git repos | Localhost only |
| Performance tests | File-based SQLite (temp) | Not used | Real repos (cloned) | Localhost only |
| Manual testing | `~/.claude-forge/forge.db` | Real CLI | Real repos | Real network |

**Key principle:** No test ever depends on external services, network access, or pre-existing state. Every test creates what it needs and cleans up after itself.
