# Claude Forge -- North Star

> **Read this first in every session.** This is the single source of truth.
> Last updated: 2026-02-26

---

## What We're Building

A multi-agent Claude Code orchestrator: Rust/Axum + Svelte 5, single binary.
Absorbs the best of 62 community repos into one coherent tool.

**One-liner**: The IDE for agentic coding -- what VS Code is to text editing, Forge is to AI-assisted development.

---

## Current State

### Approach: From Scratch (Informed)
We are building Forge from scratch -- not refactoring old code. The existing Forge prototype
and 62 reference repos are **reference material**, not starting points.

### What Exists (Reference Only)
- Old Forge prototype: Agent CRUD, WebSocket streaming, SQLite persistence, multi-pane UI
- 34 design docs in `forge-project/` (vision through reference)
- 62 reference repos in `refrence-repo/` with upstream tracking
- CI/CD spec, submodule tracking spec (written, not implemented)

### What's Built (New Codebase)
Located in `forge-project/crates/` with workspace `Cargo.toml`:

**forge-core** (5 files) -- IDs, events, event bus, errors
- 5 newtype ID wrappers (AgentId, SessionId, EventId, WorkflowId, SkillId)
- 20 ForgeEvent variants (system, agent, process, session, workflow, safety)
- EventBus (tokio broadcast) + EventSink trait
- ForgeError hierarchy + ForgeResult type alias
- 6 tests passing

**forge-agent** (4 files) -- Agent model, presets, validation
- Agent/NewAgent/UpdateAgent structs (Option<Option<T>> PATCH pattern)
- 9 presets: CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer
- Input validation (name length, system prompt size)
- 7 tests passing

**forge-db** (6 files) -- Pool, migrations, batch writer, repos
- DbPool (WAL mode, in-memory for tests)
- Migrator with version tracking, full Phase 0-5 schema in `migrations/0001_init.sql`
- BatchWriter: crossbeam-channel, flush at 50 events or 2s
- AgentRepo (full CRUD) + EventRepo (query by session/type, count)
- 3 FTS5 tables (skills_fts, sessions_fts, events_fts)
- 12 tests passing

**Known issues to fix:**
- `forge-core` depends on `rusqlite` (layering violation via ForgeError)
- `validate_update_agent` defined but not exported or called by AgentRepo::update()
- No `[workspace.dependencies]` in workspace Cargo.toml
- Preset serialization uses `Debug` format (brittle)

**Not yet built:** forge-api, forge-app, forge-process, forge-safety, forge-mcp (5 of 8 planned crates). **Next:** [PHASE0_REMAINING.md](PHASE0_REMAINING.md) — foundational fixes → 5 crates → frontend shell + single binary (3–4 sessions).

---

## Development Priorities (Ordered)

| Priority | Task | Status | Est. |
|----------|------|--------|------|
| **P0** | **Phase 0: Finish foundation** | **~40% done.** 3/8 crates (core, agent, db). Next: (1) foundational fixes, (2) 5 crates (process, safety, mcp, api, app), (3) frontend shell + single binary. See [PHASE0_REMAINING.md](PHASE0_REMAINING.md). | 3-4 sessions |
| **P1** | Phase 1: Agent Engine | Blocked by P0. Weeks 5-8: agent CRUD wired to UI, process spawning, real-time streaming, session management. | 4-6 sessions |
| **P2** | Phase 2+4 parallel: Workflows + Safety | Blocked by P0+P1 | 5-8 sessions |
| **P3** | Run `migrate-to-submodules.sh --apply` | Not started | 1 session |
| **P4** | Set up CI/CD (GitHub Actions) | Not started | 1-2 sessions |

---

## Reference Repo Absorption Status

### Tier 1: Extract Now (Direct Code/Pattern Value)
| Repo | What to Extract | Status |
|------|-----------------|--------|
| `claude-code-tools` | Session search (Tantivy), safety hooks, tmux-cli | Pending |
| `claude-code-router` | Multi-provider routing, preset system, SSE stream | Pending |
| `claude-code-hooks-mastery` | 13 hook types, UV scripts, builder/validator pattern | Pending |
| `Claude-Code-Development-Kit` | 3-tier docs, MCP integration, sub-agent context | Pending |
| `claude-code-infrastructure-showcase` | Auto-skill activation, skill-rules.json, 500-line rule | Pending |
| `claude-code-skills` | 38 skills, marketplace manifest, YAML frontmatter | Pending |
| `awesome-claude-code-subagents` | 127+ agent definitions, model routing patterns | Pending |
| `claude-code-spec-workflow` | Spec-driven workflow, steering documents | Pending |
| `1code` | Multi-agent desktop, worktrees, Kanban, Git UI | Pending |
| `claude-code-action` | GitHub Action patterns, PR/issue automation | Pending |

### Tier 2: Study for Patterns
| Repo | What to Learn | Status |
|------|---------------|--------|
| `claude-code-plugins-plus-skills` | 270+ plugins, 1500+ skills, CCPI package manager | Pending |
| `ralph-claude-code` | Autonomous loop, exit detection, circuit breaker | Pending |
| `claude-code-templates` | 100+ agents, npx installer, web dashboard | Pending |
| `Claude-Code-Workflow` | JSON-driven workflows, 4-level execution | Pending |
| `claude_code_bridge` | Split-pane terminal, multi-CLI sync | Pending |
| `claude-code-hooks-multi-agent-observability` | Real-time agent monitoring | Pending |
| `Claude-Code-Usage-Monitor` | Usage tracking, predictions, warnings | Pending |

### Tier 3: Reference Only
All remaining repos -- consult as needed during implementation.

---

## Session Protocol

### Before Starting Work
1. Read this file
2. Check `SESSION_LOG.md` for recent sessions
3. Claim your work area (update SESSION_LOG.md)
4. Check for conflicts with other sessions

### During Work
- One session = one focused deliverable
- Commit early, commit often (feature branches)
- Update SESSION_LOG.md when done

### Parallel Development Rules
- **Never** two sessions on the same file
- Backend (Rust) and Frontend (Svelte) can run in parallel
- Different phases can run in parallel if no dependency
- Use feature branches: `feat/<phase>-<feature>`

### Handoff Protocol
When finishing a session:
1. Commit all changes
2. Log what was done in SESSION_LOG.md
3. Note any blockers or decisions needed
4. Update this NORTH_STAR.md if priorities changed

---

## Key Decisions Made

| Decision | Rationale | Date |
|----------|-----------|------|
| Rust + Svelte 5 single binary | Performance, no runtime deps, `rust-embed` | Pre-project |
| SQLite WAL mode | Single-file, concurrent reads, no server | Pre-project |
| **Start from scratch (not refactor)** | 62-repo analysis revealed architecture needs differ from prototype | 2026-02-25 |
| Existing Forge = reference only | Study patterns (process spawn, WS streaming), don't copy code | 2026-02-25 |
| 8 crates initially (not 12) | Fewer crates = faster builds; split later if needed | 2026-02-25 |
| Schema designed upfront in Phase 0 | Covers all phases; avoids migration pain | 2026-02-25 |
| Phase 4 parallel with Phase 2 | Safety/MCP has no workflow dependency; saves 4 weeks | 2026-02-25 |
| `.gitmodules` URL = upstream (not fork) | `git submodule update --recursive --remote` pulls all 61 | 2026-02-25 |
| Fork stays as `fork` remote | `cd refrence-repo/<name> && git push fork main` | 2026-02-25 |
| Session-based development protocol | Context limits mean parallel short sessions | 2026-02-25 |

## Open Questions

- [x] ~~Binary size budget~~ — Resolved: <35 MB target, <50 MB acceptable
- [x] ~~Skill count~~ — Resolved: 1,500+ (PRD updated)
- [x] ~~Phase 6 timeline~~ — Resolved: Phase 6 is post-1.0; Phases 0-5 = 24 weeks
- [x] ~~Refactor vs scratch~~ — Resolved: Start from scratch (Option B)
- [ ] d3-hierarchy vs chart.js for dashboards -- pick one
- [ ] Multi-CLI support (Codex, Gemini) -- how deep? Auth story?

---

## File Map

```
forge-project/
  NORTH_STAR.md          <-- You are here. Read first.
  SESSION_LOG.md          <-- Session history and handoffs
  00-vision/              <-- Why we exist
  01-strategy/            <-- Market, competition, positioning
  02-requirements/        <-- PRD, personas, stories, features
  03-architecture/        <-- System design, API, data model
  04-design/              <-- UI/UX, information architecture
  05-engineering/         <-- Tech stack, CI/CD, testing, standards
  06-planning/            <-- Roadmap, sprints, milestones, risks
  07-methodology/         <-- Dev process, absorption, quality gates
  08-reference/           <-- Glossary, tech refs, feature-source map
```
