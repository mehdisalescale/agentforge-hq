# TASK 02 — Add SkillRepo and GET /api/v1/skills

**Status:** done
**Priority:** medium
**Track:** C (Phase 2 seed)

---

## Context

The database migration (`migrations/0001_init.sql`) already has a `skills` table:

```sql
CREATE TABLE skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    skill_type TEXT NOT NULL DEFAULT 'prompt',
    definition TEXT NOT NULL DEFAULT '{}',
    version TEXT NOT NULL DEFAULT '1.0.0',
    tags TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

But there's no repo or API route to access it. We need a minimal SkillRepo + one GET route.

## Task

1. Create `crates/forge-db/src/repos/skills.rs`:
   - `Skill` struct matching the table columns (with serde Serialize/Deserialize)
   - `SkillRepo` with `list() -> Vec<Skill>` and `get(id) -> Skill`
   - Follow the same pattern as `AgentRepo` or `SessionRepo`
2. Register the module in `crates/forge-db/src/repos/mod.rs`
3. Export `SkillRepo` and `Skill` from `crates/forge-db/src/lib.rs`
4. Create `crates/forge-api/src/routes/skills.rs`:
   - `GET /skills` → returns JSON array (empty if no skills)
   - `GET /skills/:id` → returns single skill or 404
5. Register the skill routes in `crates/forge-api/src/routes/mod.rs`
6. Add `skill_repo: Arc<SkillRepo>` to `AppState` in `crates/forge-api/src/state.rs`
7. Wire up in `crates/forge-app/src/main.rs`: create SkillRepo, pass to AppState
8. Add at least one test (repo list returns empty, HTTP route returns 200 with `[]`)

## Files to read first

- `crates/forge-db/src/repos/agents.rs` — pattern to follow
- `crates/forge-db/src/repos/mod.rs` — module registration
- `crates/forge-api/src/routes/agents.rs` — route pattern to follow
- `crates/forge-api/src/routes/mod.rs` — route registration
- `crates/forge-api/src/state.rs` — AppState struct
- `migrations/0001_init.sql` — skills table schema

## Files to edit

- `crates/forge-db/src/repos/skills.rs` (new)
- `crates/forge-db/src/repos/mod.rs`
- `crates/forge-db/src/lib.rs`
- `crates/forge-api/src/routes/skills.rs` (new)
- `crates/forge-api/src/routes/mod.rs`
- `crates/forge-api/src/state.rs`
- `crates/forge-app/src/main.rs`

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
# Then:
curl -s http://127.0.0.1:4173/api/v1/skills | jq .
# Should return: []
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed:
  - **forge-db:** Added `repos/skills.rs` with `Skill` (id, name, description, category, subcategory, content, source_repo, parameters_json, examples_json, usage_count, created_at) and `SkillRepo::list()` / `SkillRepo::get(id)`. Registered in `repos/mod.rs`, exported from `lib.rs`. Schema matches actual migration (no updated_at, skill_type, definition, version, tags).
  - **forge-core:** Added `ForgeError::SkillNotFound(String)` so GET /skills/:id returns 404.
  - **forge-api:** Added `routes/skills.rs` (GET /skills, GET /skills/:id); `skill_repo` in `AppState` and `api_error` mapping for `SkillNotFound`; all tests updated to pass `SkillRepo` into `AppState::new`.
  - **forge-app:** Created `SkillRepo`, passed into `AppState::new`.
  - **Tests:** `forge-db`: `skill_repo_list_empty_after_migration`. `forge-api`: `skills_list_returns_200_and_empty_array`.
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [ ] Notes: GET /api/v1/skills and GET /api/v1/skills/:id are under the existing `/api/v1` nest. Optional: run `curl -s http://127.0.0.1:4173/api/v1/skills | jq .` after starting the server to confirm `[]`.
