# Claude Forge — Executive Summary

**Last updated:** 2026-02-26

---

## Vision

**Claude Forge** is a multi-agent Claude Code orchestrator: one coherent tool that absorbs the best of 62 community repos into a **single binary** (Rust/Axum backend, Svelte 5 frontend).

**One-liner:** The IDE for agentic coding — what VS Code is to text editing, Forge is to AI-assisted development.

---

## Current Status

| Metric | Value |
|--------|--------|
| **Phase** | **Phase 0 complete** |
| **Crates** | 8 (forge-core, forge-agent, forge-db, forge-api, forge-process, forge-safety, forge-mcp, forge-app) |
| **Runnable** | **claude-forge** — single binary `./forge`, port 4173, UI + API + WebSocket |
| **Frontend** | SvelteKit adapter-static in `frontend/`, rust-embed in forge-app |
| **Design docs** | 34 + CHECKLIST_RESULTS, PHASE1_DESIGN_NOTES, TREND_26FEB_ENHANCEMENT_MAP |
| **Reference repos** | 62 in registry; 26-feb for design/skills (claude-flow, Agent-Skills, superpowers) |

**Delivered:** Event bus, agent model (9 presets), full schema, batch writer, AgentRepo/EventRepo, FTS5; forge-api (health, agent CRUD, WebSocket); forge-app (binary + embedded UI); frontend shell (layout, placeholder pages). Verification: [CHECKLIST_RESULTS.md](CHECKLIST_RESULTS.md).

---

## Next Steps (Ordered)

1. **Phase 1: Agent Engine (weeks 5–8)** — Process spawning (claude CLI, stream-json), real-time streaming, session management, agent CRUD wired to UI. Design refs: [docs/PHASE1_DESIGN_NOTES.md](docs/PHASE1_DESIGN_NOTES.md), [08-reference/TREND_26FEB_ENHANCEMENT_MAP.md](08-reference/TREND_26FEB_ENHANCEMENT_MAP.md).
2. **Phases 2–5 (to 1.0)** — Workflows + skills, observability + Git, safety + MCP, plugins + security. Target: shippable 1.0 in ~27 weeks from Phase 0 start.
3. **Build order (Phase 0):** In claude-forge, run `make build-frontend` (or `pnpm build` in frontend/) before `cargo build --release` so rust-embed has `frontend/build/`.

---

## Risks & Dependencies

- **Phase 0 complete** — Runnable stack in claude-forge; forge-project = planning, frontend, docs.
- **No refactor of legacy code** — Build is from scratch; 62 repos and existing Forge prototype are reference only.
- **CI/CD and submodules** — Specs written; implementation (GitHub Actions, 62-repo submodule layout) not yet started.

---

## References

- **North star / priorities:** [NORTH_STAR.md](NORTH_STAR.md)
- **Phase 0 checklist:** [PHASE0_REMAINING.md](PHASE0_REMAINING.md)
- **Roadmap (7 phases):** [06-planning/ROADMAP.md](06-planning/ROADMAP.md)
- **Reference repos (62 + 26-feb):** [REFERENCE_REPOS.md](REFERENCE_REPOS.md)
