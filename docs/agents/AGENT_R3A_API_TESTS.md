# Agent R3-A: API Integration Test Suite

> Zero tests on HTTP routes today. Add comprehensive Axum TestClient integration tests for every route module. Also add doc-accuracy assertion tests.

## Step 1: Read Context

- `CLAUDE.md`
- `crates/forge-api/src/lib.rs` — full file, find existing test helpers (test_state, json_get, json_post patterns)
- `crates/forge-api/src/routes/mod.rs` — all route modules
- `crates/forge-api/src/state.rs` — AppState struct + constructor
- `crates/forge-db/src/pool.rs` — DbPool::in_memory() for test setup
- `crates/forge-db/src/lib.rs` — Migrator, all repo exports
- Read 3-4 route files to understand handler signatures: `routes/agents.rs`, `routes/org.rs`, `routes/governance.rs`, `routes/health.rs`

## Step 2: Understand the Test Pattern

Look at `crates/forge-api/src/lib.rs` — it likely already has some test infrastructure. If it has `test_state()` or similar helpers, build on those. If not, create them.

The test pattern is:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for oneshot()

    /// Build a test app with in-memory DB + all migrations applied.
    fn test_app() -> Router {
        let pool = forge_db::DbPool::in_memory().unwrap();
        {
            let conn = pool.connection();
            forge_db::Migrator::new(&conn).apply_pending().unwrap();
        }

        let conn_arc = pool.conn_arc();
        // ... create all repos from conn_arc ...
        // ... create AppState ...
        app(state)
    }

    async fn get(app: &Router, uri: &str) -> (StatusCode, String) {
        let req = Request::get(uri).body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    async fn post_json(app: &Router, uri: &str, json: serde_json::Value) -> (StatusCode, String) {
        let req = Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&json).unwrap()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }
}
```

## Step 3: Write Tests — Health

```rust
#[tokio::test]
async fn health_returns_ok() {
    let app = test_app();
    let (status, body) = get(&app, "/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "ok");
}
```

## Step 4: Write Tests — Agents CRUD

```rust
#[tokio::test]
async fn agents_crud_lifecycle() {
    let app = test_app();

    // List (should have demo agents from seeding, or be empty)
    let (status, _) = get(&app, "/api/v1/agents").await;
    assert_eq!(status, StatusCode::OK);

    // Create
    let (status, body) = post_json(&app, "/api/v1/agents", serde_json::json!({
        "name": "Test Agent"
    })).await;
    assert_eq!(status, StatusCode::CREATED); // or OK
    let created: serde_json::Value = serde_json::from_str(&body).unwrap();
    let agent_id = created["id"].as_str().unwrap();

    // Get by ID
    let (status, body) = get(&app, &format!("/api/v1/agents/{}", agent_id)).await;
    assert_eq!(status, StatusCode::OK);

    // Delete
    let (status, _) = delete(&app, &format!("/api/v1/agents/{}", agent_id)).await;
    assert!(status == StatusCode::OK || status == StatusCode::NO_CONTENT);
}
```

## Step 5: Write Tests — Companies + Org

```rust
#[tokio::test]
async fn companies_crud() {
    let app = test_app();

    let (status, body) = post_json(&app, "/api/v1/companies", serde_json::json!({
        "name": "Test Corp",
        "mission": "Testing",
        "budget_limit": 100.0
    })).await;
    assert!(status.is_success());
    let company: serde_json::Value = serde_json::from_str(&body).unwrap();
    let company_id = company["id"].as_str().unwrap();

    // Get by ID
    let (status, _) = get(&app, &format!("/api/v1/companies/{}", company_id)).await;
    assert_eq!(status, StatusCode::OK);

    // List
    let (status, _) = get(&app, "/api/v1/companies").await;
    assert_eq!(status, StatusCode::OK);

    // Departments
    let (status, _) = post_json(&app, "/api/v1/departments", serde_json::json!({
        "company_id": company_id,
        "name": "Engineering"
    })).await;
    assert!(status.is_success());

    // Org chart
    let (status, _) = get(&app, &format!("/api/v1/org-chart?company_id={}", company_id)).await;
    assert_eq!(status, StatusCode::OK);
}
```

## Step 6: Write Tests — Governance

```rust
#[tokio::test]
async fn goals_crud() {
    let app = test_app();
    // Create company first, then goals with company_id
    // ...
}

#[tokio::test]
async fn approvals_crud() {
    let app = test_app();
    // Create company, then approval request, then approve/reject
    // ...
}
```

## Step 7: Write Tests — Sessions, Skills, Personas

Cover each route module with at least one happy-path test:
- `GET /api/v1/sessions` — list
- `GET /api/v1/skills` — list
- `GET /api/v1/personas` — list
- `GET /api/v1/personas/divisions` — list divisions
- `GET /api/v1/analytics/summary` — analytics (if exists)

## Step 8: Doc Accuracy Assertion Test

Add a test that counts things from code and can be compared against docs:

```rust
#[test]
fn mcp_tool_count_matches_docs() {
    // Count #[tool] annotations in forge-mcp-bin
    // This is a compile-time check — if tools change, this test reminds us to update docs
    // The exact count should be updated when tools are added/removed
    let expected_tool_count = 19;
    // If you can introspect the tool count at compile time, assert here
    // Otherwise, just document the expected count as a reminder
    assert_eq!(expected_tool_count, 19, "Update site-docs/reference/mcp-tools.md if tool count changes");
}
```

## Step 9: Add Test Dependencies

In `crates/forge-api/Cargo.toml`, add dev-dependencies if needed:

```toml
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
tokio = { workspace = true }
serde_json = { workspace = true }
```

## Step 10: Verify

```bash
cargo test --workspace 2>&1 | tail -20
cargo test -p forge-api 2>&1     # focus on new tests
```

Aim for **at least 15 test functions** covering:
- Health (1)
- Agents CRUD (2-3)
- Companies + Departments + Org Chart (3-4)
- Goals + Approvals (2-3)
- Sessions (1-2)
- Skills (1)
- Personas (1-2)
- Settings (1)

## Rules

- Write tests in `crates/forge-api/src/lib.rs` (expand existing test module) or in a new file `crates/forge-api/tests/integration.rs`
- You may add dev-dependencies to `crates/forge-api/Cargo.toml`
- Do NOT modify any production code — tests only
- Do NOT touch `frontend/`, `site-docs/`, `CLAUDE.md`, `.github/workflows/`
- Run `cargo test --workspace` before reporting done

## Report

When done, create `docs/agents/REPORT_R3A.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
TEST_COUNT: [number] new test functions
ROUTE_COVERAGE: [list of route modules with tests]
UNCOVERED_ROUTES: [any routes not tested, with reason]
CARGO_TEST: pass/fail (with test count)
```
