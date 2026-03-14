# Agent D: E1 Backend Polish

> You are Agent D. Your job: add missing backend CRUD endpoints for companies, departments, and persona divisions.

## Step 1: Read Context

```
CLAUDE.md                                         — project rules
NORTH_STAR.md                                      — current state
crates/forge-api/src/routes/org.rs                 — existing company/department/position routes
crates/forge-api/src/routes/personas.rs            — persona list, get, hire
crates/forge-api/src/routes/governance.rs          — goals, approvals (pattern reference)
crates/forge-api/src/lib.rs                        — route wiring + test helpers (test_state, json_post, json_get)
crates/forge-db/src/repos/companies.rs             — CompanyRepo
crates/forge-db/src/repos/departments.rs           — DepartmentRepo
crates/forge-db/src/repos/org_positions.rs         — OrgPositionRepo
crates/forge-db/src/repos/personas.rs              — PersonaRepo
```

## Step 2: Identify Missing Endpoints

Check what already exists in the routes. Then add what's missing from this list:

### Companies (in `routes/org.rs`)
- `GET /api/v1/companies/:id` — single company by ID
- `PATCH /api/v1/companies/:id` — update name, mission, budget_limit (all optional)
- `DELETE /api/v1/companies/:id` — delete a company

### Departments (in `routes/org.rs`)
- `GET /api/v1/departments/:id` — single department by ID
- `PATCH /api/v1/departments/:id` — update name, description
- `DELETE /api/v1/departments/:id` — delete a department

### Persona Divisions (in `routes/personas.rs`)
- `GET /api/v1/personas/divisions` — list all divisions with agent counts (for frontend filter dropdown)

## Step 3: Add Repo Methods

For each new endpoint, add the corresponding repo method if it doesn't exist:

**CompanyRepo** — likely needs: `get(id)`, `update(id, fields)`, `delete(id)`
**DepartmentRepo** — likely needs: `get(id)`, `update(id, fields)`, `delete(id)`
**PersonaRepo** — likely needs: `list_divisions()` returning `Vec<PersonaDivision>`

Follow existing patterns:
- Use `self.conn.lock().expect("db mutex poisoned")`
- Return `ForgeResult<T>`
- Map errors with `.map_err(|e| ForgeError::Database(Box::new(e)))`

## Step 4: Add Route Handlers

Follow existing patterns in the routes:
- Use `State(state): State<AppState>` for access to repos
- Use `Path(id): Path<String>` for URL params
- Use `Json<T>` for request/response bodies
- Use `api_error()` for error mapping
- Create `#[derive(Deserialize)]` structs for update payloads

Example update handler:
```rust
#[derive(Deserialize)]
struct UpdateCompanyBody {
    name: Option<String>,
    mission: Option<String>,
    budget_limit: Option<f64>,
}

async fn update_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateCompanyBody>,
) -> Result<Json<Company>, axum::response::Response> {
    let company = state.company_repo.update(&id, body.name.as_deref(), body.mission.as_deref(), body.budget_limit).map_err(api_error)?;
    Ok(Json(company))
}
```

## Step 5: Wire Routes

Add new routes to the existing router in each `routes()` function. Example:
```rust
Router::new()
    .route("/companies", get(list_companies).post(create_company))
    .route("/companies/:id", get(get_company).patch(update_company).delete(delete_company))
```

**Important for persona divisions**: The route `/personas/divisions` must be registered BEFORE `/personas/:id` to avoid the path parameter capturing "divisions" as an ID.

## Step 6: Write Tests

Add integration tests in `crates/forge-api/src/lib.rs` using existing helpers:

```rust
#[tokio::test]
async fn epic1_company_detail_and_update() {
    let state = test_state();
    let app = forge_api::app(state);
    // Create company
    let body = json_post(&app, "/api/v1/companies", r#"{"name":"TestCo"}"#).await;
    let id = body["id"].as_str().unwrap();
    // Get single
    let detail = json_get(&app, &format!("/api/v1/companies/{id}")).await;
    assert_eq!(detail["name"], "TestCo");
    // Update
    let updated = json_patch(&app, &format!("/api/v1/companies/{id}"), r#"{"mission":"New mission"}"#).await;
    assert_eq!(updated["mission"], "New mission");
}
```

Test each new endpoint: get-by-id, update, delete, divisions list.

## Step 7: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-api 2>&1         # all tests pass
cargo test -p forge-db 2>&1          # all tests pass
```

## Rules

- Only modify files in `crates/forge-api/src/routes/` and `crates/forge-db/src/repos/`
- Only add tests in `crates/forge-api/src/lib.rs`
- Do NOT modify AppState struct
- Do NOT create new migrations
- Do NOT touch frontend code
- Do NOT touch middleware
- Follow existing patterns exactly (study the code before writing)
- Commit with message: `feat(api): add company/department CRUD and persona divisions endpoint`

## Report

When done, output:
```
STATUS: done | blocked
FILES_MODIFIED: [list]
ENDPOINTS_ADDED: [list with HTTP method + path]
REPO_METHODS_ADDED: [list]
TESTS_ADDED: N
ISSUES: [any problems]
```
