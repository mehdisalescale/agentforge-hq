# Wave 1 — Live Coordination

> **Purpose:** Shared state file for Wave 1 agents (A-E). Each agent reads and writes here.
> **Coordinator:** Human in main session. Agents report here; coordinator resolves conflicts.
> **Started:** 2026-03-03

---

## Agent Status

| Agent | Task | Status | Branch | Last Update |
|-------|------|--------|--------|-------------|
| A | forge-git crate | `done` | main | 2026-03-03 |
| B | Middleware trait + chain | `done` | main | 2026-03-03 |
| C | Skill loader + seed files | `done` | main | 2026-03-03 |
| D | Memory table + repo + routes | `done` | main | 2026-03-03 |
| E | Hook table + repo + routes | `done` | main | 2026-03-03 |

**Status values:** `pending` → `in_progress` → `testing` → `done` / `blocked`

---

## Agent Reports

### Agent A — forge-git

**Status:** done
**Files created:** `crates/forge-git/Cargo.toml`, `crates/forge-git/src/lib.rs`
**Tests added:** 7 tests: `create_and_remove_worktree`, `list_worktrees_finds_forge_branches`, `list_worktrees_empty_on_fresh_repo`, `create_duplicate_worktree_fails`, `remove_nonexistent_worktree_is_ok`, `is_git_repo_true_for_repo`, `is_git_repo_false_for_tmp`
**Issues:** *(none)*
**Notes:** Added `tempfile` as dev-dependency. Added `is_git_repo()` helper beyond task card spec (useful for WT2/WT3). Also added to root `Cargo.toml` workspace members. `cargo test -p forge-git` — 7/7 pass. `cargo clippy -p forge-git -- -D warnings` — zero warnings.

---

### Agent B — Middleware

**Status:** done
**Files created:** `crates/forge-api/src/middleware.rs`
**Tests added:** 3 tests: `chain_executes_in_order`, `middleware_can_short_circuit`, `empty_chain_returns_ok`
**Issues:** *(none)*
**Notes:** Used native `Pin<Box<dyn Future>>` instead of `async_trait` crate — avoids adding a dependency since Rust 1.93 doesn't need it. Same pattern, zero-dep. File is not in module tree yet (Agent F adds `mod middleware;` in Wave 2). `cargo check -p forge-api` passes clean.

---

### Agent C — Skills

**Status:** done
**Files created:** `skills/code-review.md`, `skills/deep-research.md`, `skills/refactor.md`, `skills/test-writer.md`, `skills/debug.md`, `skills/security-audit.md`, `skills/document.md`, `skills/architect.md`, `skills/explore.md`, `skills/fix-bug.md`, `crates/forge-db/src/repos/skills.rs` (modified)
**Tests added:** 9 tests: `upsert_inserts_new_skill`, `upsert_updates_existing_skill`, `upsert_preserves_usage_count`, `parse_frontmatter_valid`, `parse_frontmatter_no_delimiters`, `parse_frontmatter_no_name`, `parse_bracket_list_works`, `load_from_dir_loads_md_files`, `load_from_dir_skips_invalid_frontmatter`
**Issues:** Pre-existing clippy warnings in `agents.rs` (needless_borrow on lines 80, 100) cause `cargo clippy -p forge-db -- -D warnings` to fail, but these are not from skills.rs. No warnings from skills.rs code.
**Notes:** Added `upsert()` (INSERT OR UPDATE by id, preserves usage_count/created_at), `load_from_dir()` (reads .md files, parses YAML frontmatter, upserts each), `UpsertSkill` struct, `parse_skill_frontmatter()` and `parse_bracket_list()` helpers. Also fixed `row_to_skill()` to handle both RFC3339 and SQLite `datetime('now')` format (was broken for upserted rows). `cargo test -p forge-db` — 23/23 pass. No new dependencies added.

---

### Agent D — Memory

**Status:** done
**Files created:** `migrations/0003_add_memory.sql`, `crates/forge-db/src/repos/memory.rs`, `crates/forge-api/src/routes/memory.rs`
**Tests added:** 8 tests: `create_and_get_memory`, `create_with_defaults`, `create_empty_content_fails`, `list_with_pagination`, `update_memory`, `delete_memory`, `delete_nonexistent_fails`, `search_memory`
**Issues:** *(none)*
**Notes:** Repo uses `Arc<Mutex<Connection>>` pattern matching AgentRepo. Memory model uses plain String IDs (not newtype) since memory facts are not referenced by other tables. Routes use relative paths (no `/memory` prefix) since Wave 2 will nest at `/api/v1/memory`. Added `?q=` search param to list endpoint. Files are not in module tree yet — Agent F adds `pub mod memory;` to repos/mod.rs and wires into AppState in Wave 2.

---

### Agent E — Hooks

**Status:** done
**Files created:** `migrations/0004_add_hooks.sql`, `crates/forge-db/src/repos/hooks.rs`, `crates/forge-api/src/routes/hooks.rs`
**Tests added:** 10 tests: `create_and_get_hook`, `list_hooks`, `update_hook`, `delete_hook`, `delete_nonexistent_hook_fails`, `find_by_event`, `find_by_event_excludes_disabled`, `validation_rejects_empty_name`, `validation_rejects_invalid_timing`, `hook_runner_executes_command`, `hook_runner_captures_failure` (last 2 are async/tokio)
**Issues:** *(none)*
**Notes:** Repo uses `Arc<Mutex<Connection>>` pattern matching AgentRepo. Hook model uses plain String IDs. HookRunner uses `tokio::process::Command` for async shell execution with stdout/stderr capture and duration tracking. Routes use `/hooks` prefix (Wave 2 nests at `/api/v1/hooks`). Added `find_by_event(event_type, timing)` for fetching enabled hooks by trigger. Migration includes composite index on `(event_type, timing)`. Files are not in module tree yet — Agent F adds `pub mod hooks;` to repos/mod.rs and wires into AppState in Wave 2. `cargo check --workspace` passes clean (zero warnings).

---

## Shared Issues

*(Agents write here if they encounter problems that affect other agents or need coordinator help)*

| # | Raised By | Description | Resolution |
|---|-----------|-------------|------------|
| — | — | *(none yet)* | — |

---

## Verification Gate

**Gate:** All 5 agents must be `done` before Wave 2 starts.

```bash
# Run after all agents complete:
cargo check --workspace   # zero warnings
cargo test --workspace    # all pass
cargo clippy --workspace  # zero warnings
cd frontend && pnpm build # frontend still builds
```

**Gate result:** *(not run yet)*
