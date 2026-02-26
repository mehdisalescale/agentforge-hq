# TASK 07 — WorkflowRepo + GET /api/v1/workflows

**Status:** done
**Priority:** medium
**Track:** Phase 2 seed

---

## Context

Same pattern as TASK_02 (SkillRepo) but for the `workflows` table. The migration already has it. The frontend already has a `/workflows` placeholder page.

Check the actual `workflows` table schema in `migrations/0001_init.sql` — use the real columns, not assumptions.

## Task

1. Create `crates/forge-db/src/repos/workflows.rs`:
   - `Workflow` struct matching the table columns (serde Serialize/Deserialize)
   - `WorkflowRepo` with `list()` and `get(id)`
   - Follow `SkillRepo` pattern exactly
2. Add `ForgeError::WorkflowNotFound(String)` to `crates/forge-core/src/error.rs`
3. Register in `crates/forge-db/src/repos/mod.rs`, export from `crates/forge-db/src/lib.rs`
4. Create `crates/forge-api/src/routes/workflows.rs`: GET /workflows, GET /workflows/:id
5. Register routes in `crates/forge-api/src/routes/mod.rs`
6. Add `workflow_repo: Arc<WorkflowRepo>` to `AppState`, wire in `main.rs`
7. Add `WorkflowNotFound` to `api_error` in `crates/forge-api/src/error.rs`
8. Add test: `workflow_repo_list_empty` (forge-db), `workflows_list_returns_200` (forge-api)

## Files to read first

- `migrations/0001_init.sql` — workflows table schema (READ THIS FIRST)
- `crates/forge-db/src/repos/skills.rs` — pattern to copy
- `crates/forge-api/src/routes/skills.rs` — route pattern to copy

## Files to edit

- `crates/forge-core/src/error.rs`
- `crates/forge-db/src/repos/workflows.rs` (new)
- `crates/forge-db/src/repos/mod.rs`
- `crates/forge-db/src/lib.rs`
- `crates/forge-api/src/routes/workflows.rs` (new)
- `crates/forge-api/src/routes/mod.rs`
- `crates/forge-api/src/error.rs`
- `crates/forge-api/src/state.rs`
- `crates/forge-app/src/main.rs`
- Update existing tests that construct `AppState::new` (add workflow_repo param)

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: WorkflowRepo + GET /api/v1/workflows and GET /api/v1/workflows/:id; ForgeError::WorkflowNotFound; AppState.workflow_repo; api_error mapping; forge-db test workflow_repo_list_empty, forge-api test workflows_list_returns_200.
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [ ] Notes: Datetime parsing in workflows repo supports both RFC3339 and SQLite `datetime('now')` format. forge-api tests require `crates/frontend/build` to exist for RustEmbed when running `cargo test -p forge-api`.
