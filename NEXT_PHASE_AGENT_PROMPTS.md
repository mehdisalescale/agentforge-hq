# Next-Phase Agent Prompts (A / B / C / D)

> **Use this after Round 2 is merged.** Each agent gets shared context + one prompt. Include the 26-feb enhancement summary so agents can use it for design and skills.

---

## Shared context (paste into every agent)

**Current state**
- **Phase 0:** Foundational fixes done; 7 crates: forge-core, forge-agent, forge-db, forge-api, forge-process, forge-safety, forge-mcp. forge-api has health, agent CRUD, WebSocket at `/api/v1/*`.
- **What each of you did:** A = forge-safety stub (CircuitBreaker, RateLimiter, CostTracker); B = forge-mcp stub (McpRequest, McpResponse, McpTool, McpResource, etc.); C = forge-api (already implemented); D = forge-process stub (ProcessHandle, StreamJsonEvent).
- **Missing for Phase 0:** forge-app (binary), frontend shell (SvelteKit), rust-embed, single binary so `./forge` serves UI at localhost:4173 and `GET /api/v1/health` returns OK.
- **After Phase 0:** Phase 1 = Agent Engine (weeks 5–8): process spawning, real-time streaming, session management, agent CRUD wired to UI.

**26-feb enhancement repos (use for design and skills)**

When designing forge-process, MCP, plugins, workflows, or skills, refer to these. Full map: `08-reference/TREND_26FEB_ENHANCEMENT_MAP.md`.

| Impact | Repo | How it helps Forge |
|--------|------|---------------------|
| **High** | **claude-flow** | 15-agent mesh, plugin microkernel, MCP-first API, event sourcing, hybrid memory. Use ADRs and design to shape forge-process, plugins, MCP. |
| **High** | **ruvector** | Rust vector/semantic search (“sessions like this”, “skills like this”); optional memory backend. Phase 2. |
| **High** | **deer-flow** | Sub-agent harness, skills, sandboxes, context engineering. Orchestration, skill design, safety/worktree. |
| **High** | **superpowers** | Spec → design → plan → subagent TDD. Workflow templates and composable skills; quality gates. |
| **High** | **Agent-Skills-for-Context-Engineering** | Skill content: context, compression, multi-agent, tool design, evaluation, LLM-as-judge. Ingest into skill catalog. |
| **Medium** | **cc-switch** | Multi-provider (Claude/Codex/Gemini) desktop UX; compare with Svelte UI. |
| **Medium** | **Scrapling** | MCP “fetch URL / scrape” tool for docs/runbooks. Phase 2. |
| **Later** | SpacetimeDB, moonshine, learning/ | Real-time DB, voice, minibooks per repo. |

**Suggested order:** claude-flow + Agent-Skills (+ superpowers) first for design and skill content; then ruvector (semantic search) and Scrapling (MCP scrape) in Phase 2.

---

## Agent A — forge-app binary

**Context:** You added the forge-safety stub. Next: create the **forge-app** binary crate so the workspace has a runnable server.

**Your task**
1. Add `crates/forge-app/` as a **binary** crate (e.g. `[[bin]]` or binary target). Add it to root `Cargo.toml` `members`.
2. In `main.rs`: obtain a DB path (e.g. env or default `~/.claude-forge/forge.db`), create `DbPool`, run migrations via `Migrator::apply_pending`, create `AgentRepo` and `EventBus`, build `AppState`, then call `forge_api::serve(addr, state)` with e.g. `127.0.0.1:4173`. Dependencies: forge-api, forge-db, forge-core, forge-agent, tokio.
3. No frontend yet — just the API server. Ensure `cargo run -p forge-app` (or `cargo run --bin forge_app`) starts and `GET http://localhost:4173/api/v1/health` returns `{"status":"ok",...}`.
4. Run `cargo build --workspace` and `cargo test --workspace`. Commit with message: reason (Phase 0 runnable server), decision (forge-app binary), effect (single binary serves API).

**26-feb:** When we add real safety logic later, claude-flow’s plugin/MCP design and deer-flow’s sandbox patterns are relevant. Not required for this task.

---

## Agent B — Frontend shell

**Context:** You added the forge-mcp stub. Next: create the **frontend shell** (no Rust changes to forge-app in this task).

**Your task**
1. Create a **SvelteKit** app (e.g. `frontend/` in forge-project) with **adapter-static**. Build output: static files (e.g. `frontend/build/` or `frontend/static/`).
2. **Layout:** sidebar (nav), main content area, status bar. **Empty placeholder pages:** Dashboard, Agents, Sessions, Workflows, Skills, Settings (title + “Coming soon” or similar).
3. Ensure the app **builds** (e.g. `npm run build` or `pnpm build`) and produces static assets. Do not wire into the Rust binary yet — that is Agent C’s task.
4. Commit with message: reason (Phase 0 UI shell), decision (SvelteKit adapter-static), effect (static frontend ready for rust-embed).

**26-feb:** For Phase 1, cc-switch gives multi-provider desktop UX patterns; we’ll compare with this Svelte UI. Optional: add a link or note in the app to `TREND_26FEB_ENHANCEMENT_MAP.md` for future skill/workflow references.

---

## Agent C — rust-embed + single binary

**Context:** You implemented forge-api. Next: **embed the frontend** into forge-app and serve it so we have a single binary.

**Your task**
1. **Agent B has delivered:** static files are in **`frontend/build/`** (SvelteKit adapter-static; index.html, agents.html, …, _app/). Add **rust-embed** (or equivalent) to forge-app to embed that directory at compile time.
2. In forge-app, extend the router (or forge-api) so that: (a) `/api/v1/*` is unchanged (API); (b) `/` and non-API paths serve the embedded static files with **SPA fallback** (e.g. index.html for 404s on paths like `/agents`, `/sessions`).
3. Ensure **`cargo build --release`** produces one binary and that running it serves the UI at `http://localhost:4173` and `GET /api/v1/health` still returns OK.
4. Run `cargo build --workspace` and `cargo test --workspace`. Commit with message: reason (Phase 0 single binary), decision (rust-embed + SPA fallback), effect (./forge serves UI + API).

**26-feb:** Not required for this task. When we add MCP tools later, Scrapling’s “fetch URL / scrape” pattern is a good candidate (see TREND_26FEB_ENHANCEMENT_MAP.md).

---

## Agent D — Phase 0 checklist + 26-feb design notes

**Context:** You added the forge-process stub. Next: **verify Phase 0** and add a short **design note** for Phase 1 using 26-feb.

**Your task**
1. **Phase 0 checklist:** After A/B/C have merged, run `cargo build --release`, start the binary, and verify: (a) `./forge` (or the binary name) starts; (b) browser shows UI shell at `http://localhost:4173`; (c) `GET /api/v1/health` returns OK; (d) WebSocket at `GET /api/v1/ws` connects (optional: heartbeat or event). Update `PHASE0_REMAINING.md` or `PHASE0_WORK_CHECK.md` to mark “Phase 0 done when” items complete. If anything fails, report in a short CHECKLIST_RESULTS.md or in the commit message.
2. **Design note for Phase 1:** Add a short section (e.g. in `PHASE0_REMAINING.md` under “After Phase 0” or in a new `docs/PHASE1_DESIGN_NOTES.md`) that says: when implementing **forge-process** (real process spawn, stream-json) and **MCP/server** behavior, use **claude-flow** ADRs and plugin/MCP design as reference; when adding **skills/workflows**, use **Agent-Skills-for-Context-Engineering** and **superpowers** for content and spec→TDD workflow. Point to `08-reference/TREND_26FEB_ENHANCEMENT_MAP.md` for the full map.
3. Commit with message: reason (Phase 0 sign-off + Phase 1 design refs), decision (checklist + 26-feb notes), effect (doc set ready for Phase 1).

**26-feb:** You are the one explicitly wiring the enhancement map into the doc set so the next Phase 1 work (process, streaming, sessions) can use claude-flow, Agent-Skills, and superpowers without re-discovering them.

---

## Run order

1. **A and B in parallel** — forge-app (API-only server) and frontend shell.
2. **C after A and B** — add rust-embed to forge-app and serve frontend; single binary.
3. **D after C** — run Phase 0 checklist and add Phase 1 design notes (26-feb).

Merge after each step; run `cargo build --workspace && cargo test --workspace` before moving on.
