# Claude Forge -- North Star

> **Read this first in every session.** This is the single source of truth.
> Last updated: 2026-02-25

---

## What We're Building

A multi-agent Claude Code orchestrator: Rust/Axum + Svelte 5, single binary.
Absorbs the best of 61 community repos into one coherent tool.

**One-liner**: The IDE for agentic coding -- what VS Code is to text editing, Forge is to AI-assisted development.

---

## Current State

### Approach: From Scratch (Informed)
We are building Forge from scratch -- not refactoring old code. The existing Forge prototype
and 61 reference repos are **reference material**, not starting points.

### What Exists (Reference Only)
- Old Forge prototype: Agent CRUD, WebSocket streaming, SQLite persistence, multi-pane UI
- 34 design docs in `forge-project/` (vision through reference)
- 61 reference repos in `refrence-repo/` with upstream tracking
- CI/CD spec, submodule tracking spec (written, not implemented)

### What's Built (New Codebase)
_Nothing yet. Phase 0 starts fresh._

---

## Development Priorities (Ordered)

| Priority | Task | Why First | Est. |
|----------|------|-----------|------|
| **P0** | Run `migrate-to-submodules.sh --apply` | Formalize 61 repos as submodules | 1 session |
| **P1** | Set up CI/CD (GitHub Actions) | Need cross-platform builds before users | 1-2 sessions |
| **P2** | Phase 0: Foundation Build | Workspace crates, event bus, DB schema, API+UI skeleton | 4-6 sessions |
| **P3** | Phase 1: Agent Engine | Agent CRUD, process spawn, streaming, sessions | 4-6 sessions |
| **P4** | Phase 2+4 parallel: Workflows + Safety | Workflow engine, skills, MCP, circuit breaker | 5-8 sessions |

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
| **Start from scratch (not refactor)** | 61-repo analysis revealed architecture needs differ from prototype | 2026-02-25 |
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
