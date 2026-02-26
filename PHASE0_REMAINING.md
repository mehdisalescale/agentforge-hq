# Phase 0 Remaining Work

> Phase 0 is **~40% done**. This doc is the ordered checklist for the next 3–4 sessions.
> After Phase 0, the roadmap continues with **Phase 1: Agent Engine** (weeks 5–8) — agent CRUD API wired to UI, process spawning, real-time streaming, session management.

---

## Progress

- **Done:** forge-core, forge-agent, forge-db (3/8 crates), full schema, migrations, batch writer, AgentRepo, EventRepo, FTS5 stubs
- **Remaining:** foundational fixes → 5 new crates → frontend shell + single binary

---

## 1. Fix foundational issues first (1 session)

- [ ] **Extract rusqlite from forge-core** — Move `ForgeError::Database` to a forge-db–specific error, or use `Box<dyn Error>` in core. Unblocks clean layering for all other crates.
- [ ] **Add `[workspace.dependencies]`** — Centralize tokio, serde, chrono, rusqlite, uuid versions in root `Cargo.toml`; use in crates.
- [ ] **Export + wire `validate_update_agent`** — Add to `forge-agent` lib.rs exports; call it in `AgentRepo::update()`.
- [ ] **Fix preset serialization** — Use serde `Serialize`/`Deserialize` string representation instead of `Debug`.

---

## 2. Build remaining 5 crates (1–2 sessions)

- [ ] **forge-process** — Claude CLI process spawning, stream-json parsing (stubs OK for Phase 0).
- [ ] **forge-safety** — Circuit breaker, rate limiter, cost tracker (stubs).
- [ ] **forge-mcp** — MCP protocol types (stubs).
- [ ] **forge-api** — Axum router, health check, agent CRUD endpoints, WebSocket handler.
- [ ] **forge-app** — Binary entry point, ties everything together, serves frontend.

---

## 3. Frontend shell + single binary (1 session)

- [ ] **SvelteKit project** with `adapter-static`.
- [ ] **Layout shell:** sidebar, main content area, status bar.
- [ ] **Empty pages:** Dashboard, Agents, Sessions, Workflows, Skills, Settings.
- [ ] **rust-embed** to serve frontend from the binary.
- [ ] **`cargo build --release`** produces working single binary.

---

## Phase 0 done when

- [ ] `./forge` starts; browser shows UI shell at `http://localhost:4173`
- [ ] WebSocket connects and receives heartbeat events
- [ ] All 8 crates compile; `cargo test` + `cargo clippy` clean
- [ ] Health check `GET /api/v1/health` returns OK

---

## After Phase 0

**Phase 1: Agent Engine** (weeks 5–8) — real functionality: agent CRUD API wired to the UI, process spawning, real-time streaming, session management.
