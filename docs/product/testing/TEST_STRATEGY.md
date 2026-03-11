# Test Strategy — AgentForge

> **Testing pyramid: Unit → Integration → E2E. TDD-first. No code without a failing test.**

---

## Testing Pyramid

```
          ╱╲
         ╱  ╲
        ╱ E2E ╲          ~30 tests   (Playwright + shell scripts)
       ╱────────╲
      ╱Integration╲      ~80 tests   (API + multi-crate)
     ╱──────────────╲
    ╱   Unit Tests    ╲   ~300 tests  (per-function, per-crate)
   ╱────────────────────╲
  ╱   Static Analysis    ╲  cargo check + clippy + audit
 ╱────────────────────────╲
```

**Target totals by v1.0.0**: 400+ tests (up from 150 in v0.6.0)

---

## 1. Static Analysis (Pre-Commit)

| Tool | What | When | Fail Action |
|------|------|------|------------|
| `cargo check` | Compilation, zero warnings | Every commit | Block commit |
| `cargo clippy` | Lint, idiomatic Rust | Every commit | Block commit |
| `cargo audit` | Known CVEs in dependencies | Weekly CI + release | Block release |
| `cargo fmt --check` | Code formatting | Every commit | Block commit |
| `#![forbid(unsafe_code)]` | No unsafe blocks | Compile time | Won't compile |

---

## 2. Unit Tests (Per-Crate)

### Test Naming Convention
```
test_{function}_{scenario}_{expected_result}

Examples:
  test_parse_persona_valid_file_returns_persona
  test_parse_persona_missing_name_returns_error
  test_budget_check_over_limit_returns_exceeded
  test_security_scan_eval_injection_detected
```

### Coverage Targets by Crate

| Crate | Current Tests | Target Tests | Target Coverage |
|-------|--------------|-------------|----------------|
| forge-core | 15 | 25 | 90% |
| forge-agent | 10 | 20 | 85% |
| forge-db | 40 | 80 | 80% |
| forge-process | 15 | 40 | 75% |
| forge-safety | 15 | 35 | 90% |
| forge-api | 30 | 50 | 75% |
| forge-git | 7 | 12 | 85% |
| forge-persona (new) | — | 25 | 85% |
| forge-org (new) | — | 30 | 85% |
| forge-governance (new) | — | 20 | 85% |
| forge-knowledge (new) | — | 20 | 80% |
| forge-messaging (new) | — | 15 | 75% |
| forge-adapter-hermes (new) | — | 15 | 75% |
| forge-adapter-openclaw (new) | — | 10 | 75% |

### TDD Workflow (Enforced)

```
1. Write test (RED)
   → cargo test → test FAILS
   → Verify it fails for the RIGHT reason

2. Write minimal implementation (GREEN)
   → cargo test → test PASSES
   → No extra code beyond what makes the test pass

3. Refactor (REFACTOR)
   → cargo test → still PASSES
   → Clean up, extract, simplify

4. Repeat
```

### Unit Test Patterns

**Repository CRUD Pattern:**
```rust
#[test]
fn test_create_and_get_roundtrip() {
    let repo = setup_test_db();
    let created = repo.create(&new_entity()).unwrap();
    let fetched = repo.get(&created.id).unwrap();
    assert_eq!(created.name, fetched.name);
}

#[test]
fn test_create_duplicate_returns_error() {
    let repo = setup_test_db();
    repo.create(&new_entity()).unwrap();
    let result = repo.create(&new_entity());
    assert!(matches!(result, Err(ForgeError::Validation(_))));
}
```

**Middleware Pattern:**
```rust
#[tokio::test]
async fn test_middleware_allows_when_condition_met() {
    let middleware = MyMiddleware::new(config);
    let mut ctx = test_context();
    let next = mock_next_that_succeeds();
    let result = middleware.process(&mut ctx, next).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_middleware_rejects_when_condition_failed() {
    let middleware = MyMiddleware::new(strict_config);
    let mut ctx = test_context();
    let next = mock_next_that_succeeds();
    let result = middleware.process(&mut ctx, next).await;
    assert!(matches!(result, Err(MiddlewareError::Rejected(_))));
}
```

**Event Pattern:**
```rust
#[test]
fn test_event_serializes_and_deserializes() {
    let event = ForgeEvent::MyNewEvent { field: "value".into() };
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ForgeEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
```

---

## 3. Integration Tests (Cross-Crate)

### API Integration Tests

```rust
// tests/api_integration.rs

#[tokio::test]
async fn test_persona_hire_creates_agent_with_correct_config() {
    let app = spawn_test_server().await;

    // Import personas
    let response = app.post("/api/v1/personas/import").send().await;
    assert_eq!(response.status(), 200);

    // List personas
    let personas: Vec<Persona> = app.get("/api/v1/personas").json().await;
    assert!(personas.len() >= 100);

    // Hire first persona
    let hired: Agent = app
        .post(&format!("/api/v1/personas/{}/hire", personas[0].id))
        .json(&json!({"name": "test-agent"}))
        .json()
        .await;

    assert!(hired.system_prompt.unwrap().contains(&personas[0].personality));
}
```

### Multi-Backend Integration Tests

```rust
#[tokio::test]
async fn test_backend_switching() {
    let app = spawn_test_server().await;

    // Create agent with claude backend
    let agent = create_agent(&app, "claude").await;
    assert_eq!(agent.config["backend"], "claude");

    // Switch to hermes (mock)
    let updated = update_agent_backend(&app, &agent.id, "mock-hermes").await;
    assert_eq!(updated.config["backend"], "mock-hermes");

    // Run uses correct backend
    let session = run_agent(&app, &agent.id, "test prompt").await;
    assert_eq!(session.metadata["backend_type"], "mock-hermes");
}
```

### Budget Enforcement Integration Test

```rust
#[tokio::test]
async fn test_company_budget_enforces_limit() {
    let app = spawn_test_server().await;

    // Create company with $10 budget
    let company = create_company(&app, "Test Co", 10.0).await;
    let agent = create_agent_in_company(&app, &company.id).await;

    // Simulate $9 spent
    update_company_budget_used(&app, &company.id, 9.0).await;

    // Next run should warn
    let result = run_agent(&app, &agent.id, "small task").await;
    assert!(result.events.iter().any(|e| e.event_type == "BudgetWarning"));

    // Simulate $10 spent
    update_company_budget_used(&app, &company.id, 10.0).await;

    // Next run should be rejected
    let result = run_agent(&app, &agent.id, "another task").await;
    assert_eq!(result.status, 402);
}
```

---

## 4. End-to-End Tests

### E2E Smoke Test (Shell Script)

```bash
#!/bin/bash
# scripts/e2e-smoke.sh — Run after every release build

set -euo pipefail

# Start server
./forge &
FORGE_PID=$!
sleep 2

# Health check
curl -sf http://localhost:4173/api/v1/health | jq .status | grep -q "ok"

# Persona catalog
PERSONA_COUNT=$(curl -s http://localhost:4173/api/v1/personas | jq length)
[ "$PERSONA_COUNT" -ge 100 ] || exit 1

# Company creation
COMPANY_ID=$(curl -s -X POST http://localhost:4173/api/v1/companies \
  -H 'Content-Type: application/json' \
  -d '{"name":"E2E Test Co","mission":"Test","budget_limit":100}' | jq -r .id)
[ -n "$COMPANY_ID" ] || exit 1

# Hire persona
AGENT_ID=$(curl -s -X POST "http://localhost:4173/api/v1/personas/$(curl -s http://localhost:4173/api/v1/personas | jq -r '.[0].id')/hire" \
  -H 'Content-Type: application/json' \
  -d "{\"name\":\"e2e-agent\",\"company_id\":\"$COMPANY_ID\"}" | jq -r .id)
[ -n "$AGENT_ID" ] || exit 1

# Knowledge base
curl -sf -X POST http://localhost:4173/api/v1/knowledge/documents \
  -F "file=@README.md" -F "company_id=$COMPANY_ID" | jq .id

# Search KB
curl -sf http://localhost:4173/api/v1/knowledge/search \
  -d '{"query":"forge","company_id":"'$COMPANY_ID'"}' | jq '.[0].content'

# Backends list
curl -sf http://localhost:4173/api/v1/backends | jq '.[].name' | grep -q "claude"

# Cleanup
kill $FORGE_PID
echo "E2E smoke test passed!"
```

### Playwright E2E Tests (Frontend)

```typescript
// tests/e2e/persona-catalog.spec.ts

test('browse and hire persona', async ({ page }) => {
  await page.goto('/personas');

  // Division sidebar visible
  await expect(page.getByText('Engineering')).toBeVisible();
  await expect(page.getByText('Design')).toBeVisible();

  // Click engineering division
  await page.getByText('Engineering').click();

  // Cards appear
  const cards = page.locator('[data-testid="persona-card"]');
  await expect(cards).toHaveCount(16); // 16 engineering personas

  // Click first card
  await cards.first().click();

  // Detail modal
  await expect(page.getByText('Personality')).toBeVisible();
  await expect(page.getByText('Deliverables')).toBeVisible();

  // Hire
  await page.getByRole('button', { name: 'Hire' }).click();
  await page.fill('[name="agent-name"]', 'test-frontend-dev');
  await page.getByRole('button', { name: 'Confirm' }).click();

  // Redirected to agents page
  await expect(page).toHaveURL('/agents');
  await expect(page.getByText('test-frontend-dev')).toBeVisible();
});
```

---

## 5. Specialized Test Categories

### Property-Based Tests (Concurrent Safety)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_rate_limiter_never_exceeds_max(
        requests in 1..1000u32,
        max_tokens in 1..100u32,
    ) {
        let limiter = RateLimiter::new(max_tokens, Duration::from_secs(1));
        let mut acquired = 0;
        for _ in 0..requests {
            if limiter.try_acquire() { acquired += 1; }
        }
        prop_assert!(acquired <= max_tokens);
    }
}
```

### Contract Tests (Backend Adapters)

```rust
// Every backend adapter must pass these tests

fn backend_contract_tests(backend: impl ProcessBackend) {
    // Must report health
    let health = backend.health_check();
    assert!(matches!(health.status, Healthy | Degraded | Unavailable));

    // Must report capabilities
    let caps = backend.capabilities();
    assert!(!caps.supported_models.is_empty());

    // Must return a name
    assert!(!backend.name().is_empty());
}

#[test]
fn test_claude_backend_contract() {
    backend_contract_tests(ClaudeBackend::new());
}

#[test]
fn test_hermes_backend_contract() {
    backend_contract_tests(HermesBackend::new());
}
```

### Chaos Tests (Failure Handling)

```rust
#[tokio::test]
async fn test_backend_crash_mid_execution() {
    let backend = CrashingBackend::new(crash_after: 5); // crashes after 5 events
    let runner = ProcessRunner::new(backend);
    let result = runner.run("test prompt").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ForgeError::ProcessFailed(_)));
    // Session status should be "failed"
    // Circuit breaker should record failure
}

#[tokio::test]
async fn test_db_connection_lost_during_batch_write() {
    // BatchWriter should buffer and retry
}
```

### Security Tests

```rust
#[test]
fn test_sql_injection_prevented() {
    let repo = setup_test_db();
    let malicious_name = "'; DROP TABLE agents; --";
    let result = repo.create(&NewAgent { name: malicious_name.into(), .. });
    assert!(result.is_err()); // validation catches it
}

#[test]
fn test_path_traversal_prevented() {
    let result = validate_working_dir("../../etc/passwd");
    assert!(result.is_err());
}
```

---

## 6. CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  check:
    steps:
      - cargo fmt --check
      - cargo check --workspace
      - cargo clippy --workspace -- -D warnings
      - cargo test --workspace
      - cargo audit

  coverage:
    steps:
      - cargo tarpaulin --workspace --out xml
      - upload coverage to codecov

  e2e:
    needs: check
    steps:
      - cargo build --release
      - ./scripts/e2e-smoke.sh
      - npx playwright test (if frontend changes)
```

---

## 7. Test Data Management

### Fixtures

```
tests/fixtures/
├── personas/
│   ├── valid_frontend_dev.md
│   ├── valid_backend_arch.md
│   ├── missing_name.md          (validation test)
│   └── malformed_yaml.md        (parser test)
├── skills/
│   ├── valid_tdd.md
│   └── invalid_frontmatter.md
├── documents/
│   ├── sample_api_docs.md
│   └── sample_readme.txt
└── security/
    ├── code_with_eval.py
    ├── code_with_xss.js
    ├── code_with_sqli.py
    └── clean_code.rs
```

### Test Database

```rust
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    apply_all_migrations(&conn);
    conn
}
```

Every test uses an in-memory SQLite database. No shared state between tests. No cleanup needed.
