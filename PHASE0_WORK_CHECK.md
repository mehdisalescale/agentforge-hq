# Phase 0 Work Check

**Date:** 2026-02-26  
**Scope:** Foundational fixes (Round 1) + 5 crates (Round 2). No forge-app or frontend yet.

---

## Summary

| Area | Status | Notes |
|------|--------|--------|
| **Foundational fixes** | Done | rusqlite out of core, workspace.dependencies, validate_update_agent wired, preset serde |
| **5 new crates** | Done | forge-process, forge-safety, forge-mcp (stubs), forge-api (Axum, health, agent CRUD, WebSocket) |
| **forge-app** | Missing | Not in workspace; no binary entry point |
| **Frontend** | Missing | No SvelteKit / rust-embed |
| **Build** | Pass | `cargo build --workspace` OK |
| **Tests** | Pass | 24 tests (forge-core 6, forge-agent 7, forge-db 10, forge-api 1) |
| **Clippy** | Pass | Fixed during check: Default for IDs, redundant closure, unused-import allow |

---

## 1. Foundational fixes (Round 1)

| Item | Status | Evidence |
|------|--------|----------|
| Extract rusqlite from forge-core | Done | forge-core has no rusqlite dep; `ForgeError::Database(Box<dyn Error + Send + Sync>)` in `error.rs` |
| Add [workspace.dependencies] | Done | Root `Cargo.toml` has tokio, serde, serde_json, chrono, uuid, rusqlite, axum, tower-http; forge-db and forge-api use `workspace = true` |
| Export + wire validate_update_agent | Done | `forge-agent/lib.rs` exports it; `AgentRepo::update()` calls `validate_update_agent(input)?` first |
| Fix preset serialization | Done | `agents.rs` uses `serde_json::to_string(p)` / `serde_json::from_str(s).ok().or_else(|| parse_preset(s))`; `AgentPreset` has Serialize/Deserialize; fallback `parse_preset()` for fixed names |

---

## 2. Five crates (Round 2)

| Crate | Status | Notes |
|-------|--------|--------|
| forge-process | Present | Stub; depends on forge-core |
| forge-safety | Present | Stub; depends on forge-core |
| forge-mcp | Present | Stub; depends on forge-core |
| forge-api | Present | Axum router, `/api/v1` nest, health route, agent CRUD (list, get, create, update, delete), WebSocket route; uses AgentRepo + EventBus |
| forge-app | **Missing** | Not in workspace `members`; no binary |

---

## 3. Fixes applied during check

- **forge-core/ids.rs:** Implemented `Default` for `AgentId`, `SessionId`, `EventId`, `WorkflowId`, `SkillId` (satisfies clippy `new_without_default`).
- **forge-api/routes/agents.rs:** `#[allow(unused_imports)]` on axum routing imports (clippy reported unused delete/post/put; they are used in route definitions).
- **forge-db/repos/agents.rs:** Replaced `.query_map([], \|row\| row_to_agent(row))` with `.query_map([], row_to_agent)` (redundant closure).

---

## 4. Remaining for Phase 0

- [ ] **forge-app** — Binary crate: `main()`, run migrations, create EventBus + AgentRepo, mount forge-api, bind port (e.g. 4173).
- [ ] **Frontend shell** — SvelteKit adapter-static, layout (sidebar, main, status bar), empty pages (Dashboard, Agents, Sessions, Workflows, Skills, Settings).
- [ ] **rust-embed** — Embed built frontend in binary, serve at `/`, API at `/api/v1/*`.
- [ ] **Phase 0 done when:** `./forge` starts, UI at localhost:4173, WebSocket heartbeat, `cargo test` + `cargo clippy` clean, `GET /api/v1/health` OK.

---

## 5. Workspace state

- **Members:** forge-core, forge-agent, forge-db, forge-api, forge-process, forge-safety, forge-mcp (7 crates).
- **rusqlite features:** workspace has `bundled`, `vtab`; audit suggested adding `fts5` if FTS5 is used at runtime (migrations already define FTS5 tables).
