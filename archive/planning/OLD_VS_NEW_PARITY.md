# Old Prototype vs New Codebase — Parity and Sync

> Reference: side-by-side comparison and correction steps. **forge-project** currently has 8 crates + frontend shell; runnable stack with rust-embed is in **claude-forge**.

---

## Comparison (as reported)

| Component | Old (claude-forge/) | New (forge-project/crates/) |
|-----------|---------------------|-----------------------------|
| forge-core | Identical; uses [workspace.dependencies] | Was: no workspace deps. **Now:** uses workspace deps; `ForgeError::Database(Box<dyn Error>)` (no rusqlite in core). |
| forge-agent | Identical | Identical; validate_update_agent exported. |
| forge-db | Identical | Identical; preset serde + parse_preset; validate_update_agent called in update(). |
| forge-api | Full Axum: router, agent CRUD, WebSocket, health | **Now present** (added by agents). |
| forge-app | Full binary: clap, DB, migrations, event bus, batch writer, rust-embed, shutdown | **Now present**; in forge-project may be API-only; in claude-forge has rust-embed + frontend. |
| forge-process | Stub | **Now present** (stub). |
| forge-safety | Stub | **Now present** (stub). |
| forge-mcp | Stub | **Now present** (stub). |
| Frontend | Full SvelteKit: dashboard, agent CRUD, WebSocket store, markdown, dark theme, panes | **forge-project:** frontend shell (layout, placeholder pages). **claude-forge:** may have full UI. |
| CI/CD | GitHub Actions: fmt, clippy, test | forge-project: often missing. |
| Makefile | dev, build-frontend, build, test, check, release, clean | forge-project: often missing. |
| Cargo.toml | [workspace.dependencies] + all shared | **forge-project:** has [workspace.dependencies] and 8 members. |

---

## Critical finding (from report)

The new codebase was initially a near-exact copy of 3 crates (forge-core, forge-agent, forge-db) without the other 5 crates, frontend, or build infrastructure. After Round 1–3 agent work, **forge-project** has 8 crates, workspace.dependencies, frontend shell, and (in claude-forge) a full runnable binary with rust-embed. Remaining gaps for **forge-project** may be: Makefile, CI, and/or rust-embed in forge-app if you want forge-project to be runnable without claude-forge.

---

## Instructions to correct (reference — many already done in forge-project)

Use this list to sync **forge-project** with **claude-forge** or to bring a minimal fork up to parity.

### 1. Fix workspace Cargo.toml — shared dependencies

- **Status in forge-project:** Done. Root has `[workspace.dependencies]` (tokio, serde, chrono, uuid, rusqlite, axum, tower-http, rust-embed) and 8 members.
- **If starting from 3 crates only:** Add `[workspace.dependencies]` and set each crate to `workspace = true` for shared deps.

### 2. Fix rusqlite layering in forge-core

- **Status in forge-project:** Done. `ForgeError::Database(Box<dyn std::error::Error + Send + Sync>)`; no rusqlite in forge-core/Cargo.toml; forge-db maps with `ForgeError::Database(Box::new(e))`.

### 3. Export and wire validate_update_agent

- **Status in forge-project:** Done. forge-agent exports it; AgentRepo::update() calls `validate_update_agent(input)?` first.

### 4. Fix preset serialization (serde, not Debug)

- **Status in forge-project:** Done. AgentPreset has Serialize/Deserialize; agents.rs uses serde_json::to_string/from_str and parse_preset() fallback.

### 5. Bring over the 5 missing crates

- **Status in forge-project:** Done. forge-api, forge-app, forge-process, forge-safety, forge-mcp all present under crates/.

### 6. Bring over the frontend

- **Status in forge-project:** Frontend shell present (frontend/ with SvelteKit, adapter-static, layout, placeholder pages). For **full** UI (dashboard, agent CRUD forms, WebSocket store, markdown, panes), copy or merge from claude-forge/frontend if desired.

### 7. Bring over build infrastructure

- **Status in forge-project:** Often missing.
- **To do:** Copy claude-forge/Makefile to forge-project/Makefile; copy claude-forge/.github/ to forge-project/.github/; adjust paths if needed.

### 8. Optional: rust-embed in forge-project forge-app

- If the runnable binary should live in forge-project too: add rust-embed to forge-app, embed `frontend/build/`, and add SPA fallback (as in claude-forge). Build order: `pnpm build` in frontend/ then `cargo build --release`.

### 9. Update docs after sync

- NORTH_STAR, PHASE0_REMAINING, CHECKLIST_RESULTS already state Phase 0 complete and runnable stack in claude-forge. If forge-project becomes the single canonical codebase, update those to point to forge-project and add build/run instructions.

---

## Summary: order of operations (if starting from minimal)

1. Fix workspace Cargo.toml (shared deps)  
2. Fix rusqlite layering in forge-core  
3. Export + wire validate_update_agent  
4. Fix preset serialization  
5. Copy 5 missing crates from old prototype  
6. Copy frontend  
7. Copy Makefile + CI/CD  
8. Run `cargo test --workspace` to verify  
9. Update docs  

**In forge-project today:** 1–6 are done (8 crates, frontend shell). Remaining for parity with claude-forge: (7) Makefile + CI, (8) optional rust-embed in forge-app, (9) doc updates if canonical source changes.

---

## Decision: canonical codebase

- **Option A:** **claude-forge** = canonical runnable app; forge-project = planning, docs, and reference crates. No sync needed except doc links.
- **Option B:** **forge-project** = single repo; copy Makefile + CI from claude-forge; add rust-embed to forge-app here; treat claude-forge as a one-time source.
- **Option C:** Keep both in sync via script or manual copy of crates/frontend/Makefile/CI from claude-forge into forge-project (or vice versa).

Choose one and update NORTH_STAR and this file accordingly.
