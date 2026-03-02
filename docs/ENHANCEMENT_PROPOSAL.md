# Enhancement Proposal — Focused Sprint Plan

> Author: Claude Opus 4.6
> Date: 2026-03-02
> Status: DRAFT — Awaiting approval
> Supersedes: Sprints 2-4 of `FORGE_AUDIT_2026_03_02.md` Phase 4 proposal

---

## Thesis

Forge has 50+ docs for 3,400 LOC of Rust. The planning-to-building ratio is inverted. The audit's 4-sprint / 7-feature plan risks repeating the pattern — designing more than shipping.

This proposal trims to **3 sprints, 3 releases**, each shippable and building on the last. Everything else deferred until usage demands it.

---

## Sprint 1 — Ship v0.2.0 (Fix + Finish + Ship)

**Goal:** Close Phase B. Zero new features. Just finish and release.

| ID | Item | Crate/Area | Effort | Notes |
|----|------|-----------|--------|-------|
| F1 | Dashboard null-safety: check `outputBlocks.length` before `last` access | frontend | 15 min | Critical bug |
| F2 | Budget warning logic: `warn <= cost < limit` | forge-api | 30 min | Medium bug |
| F3 | Preset serialization: proper serde, not Debug format | forge-db | 1 hr | Medium bug |
| B1 | MCP server rewrite with `rmcp` | forge-mcp-bin | 2-3 days | Official Rust MCP SDK, `#[tool]` macros. Delete forge-mcp type stubs. |
| B2 | Wire CostTracker to session cost data | forge-safety | 1 day | Emit BudgetWarning/BudgetExceeded events |
| D1 | Create CLAUDE.md | root | 30 min | Project context for every AI/human session |
| D2 | Doc consolidation (see below) | docs/ | 1 hr | Cut doc count from 50+ to ~15 |
| — | Tag v0.2.0 release | — | — | — |

### D2: Doc Consolidation Plan

| Action | Files Affected |
|--------|---------------|
| Merge 35 frozen `00-08/` files into `docs/ORIGINAL_DESIGN_REFERENCE.md` | 35 files → 1 |
| Merge 14 `docs/planning/` files into `docs/PLANNING_ARCHIVE.md` | 14 files → 1 |
| Delete 10 superseded docs already marked in DOC_INDEX | WHAT_TO_DO_NEXT, REMAINING_APP_PLAN, PROPOSAL_2_3_4, STRATEGIC_ASSESSMENT, EXECUTIVE_SUMMARY, AUDIT_REPORT, PRODUCT_JOURNEY, REFERENCE_REPOS, PHASE1_DESIGN_NOTES, OLD_VS_NEW_PARITY |
| Update DOC_INDEX.md | Reflect new structure |

**Result:** ~15 active docs instead of 50+.

---

## Sprint 2 — Ship v0.3.0 (Worktree Isolation)

**Goal:** One differentiator. The industry-standard pattern for multi-agent isolation.

| ID | Item | Crate/Area | Effort | Notes |
|----|------|-----------|--------|-------|
| C3 | New `forge-git` crate | forge-git | 2-3 days | Wraps git2 or shell commands |
| C3a | Worktree create on run | forge-api | 1 day | `git worktree add .worktrees/{session_id} -b forge/{session_id}` |
| C3b | Pass worktree path as working_dir to spawn | forge-process | 0.5 day | Thread through SpawnConfig |
| C3c | Worktree cleanup / merge UI | frontend | 1 day | Branch status, merge/delete controls |
| T1 | Integration test: happy path E2E | tests/ | 1 day | Start server → create agent → run → stream → verify session |
| — | Tag v0.3.0 release | — | — | — |

### Why Worktrees First (Not Middleware or Skills)

| Alternative | Why Deferred |
|-------------|-------------|
| Middleware chain (C1) | Run handler is 600 LOC — not yet a monolith. Premature abstraction until 5+ concerns exist. |
| Skill system (C2) | Zero users asking for it. Build when there's demand. |
| Hooks (C6) | Solve when someone hits the problem. |

### Why Worktrees Win

- **Industry-converged pattern** — Claude official (`--worktree`), ccswarm, every serious multi-agent tool
- **Prerequisite** for sub-agent parallelism (Sprint 3 depends on this)
- **Self-contained** — no middleware refactoring or skill system required
- **Demo-able** — visible isolated branches per agent run
- **Small surface** — estimated ~300-500 LOC

---

## Sprint 3 — Ship v0.4.0 (Sub-Agent Parallel Spawning)

**Goal:** The actual value proposition. Forge stops being "a UI for Claude Code" and becomes "an orchestrator."

| ID | Item | Crate/Area | Effort | Notes |
|----|------|-----------|--------|-------|
| C4 | Coordinator agent: analyze task, decide sub-agents | forge-agent | 2 days | New coordinator preset, task decomposition prompt |
| C4a | Parallel spawn: up to N concurrent Claude processes | forge-process | 2-3 days | Configurable concurrency (default 3), each in own worktree |
| C4b | Per-sub-agent WebSocket progress events | forge-api | 1 day | New event variants: SubAgentStarted, SubAgentProgress, SubAgentCompleted |
| C4c | Result aggregation back to coordinator | forge-process | 1-2 days | Collect outputs, present unified result |
| C4d | Multi-agent dashboard UI | frontend | 2 days | Per-agent progress panels, status indicators |
| — | Tag v0.4.0 release | — | — | — |

### Dependencies

```
Sprint 1 (v0.2.0)
  └── Sprint 2 (v0.3.0) — worktree isolation
        └── Sprint 3 (v0.4.0) — parallel spawning uses worktrees
```

---

## Explicitly Deferred

These are real features from the audit. They are not rejected — they are deferred until usage data or user demand justifies them.

| Feature | Audit ID | Revisit When |
|---------|----------|-------------|
| Middleware chain | C1 | Run handler exceeds ~1000 LOC or 5+ cross-cutting concerns |
| Skill system | C2 | Users request skill injection or prompt templates |
| Cross-session memory | C5 | Multiple users report re-explaining context across sessions |
| Hook system | C6 | Users need pre/post-run automation (lint, test, scan) |
| Agent domains + coordinator | C7 | Sub-agent spawning works and needs smarter routing |
| Frontend pagination | B4 | Any list exceeds ~50 items in real usage |
| Svelte 5 rune normalization | F5 | Next frontend-heavy sprint |
| Shutdown timeout | F4 | Reported hang on Ctrl+C |

---

## Principles

1. **Ship small, ship often.** Three releases, each one usable.
2. **One feature per sprint.** Resist the urge to bundle.
3. **Let usage decide.** Don't build skills, memory, or hooks until someone needs them.
4. **Code over docs.** The 50+ doc era is over. Write CLAUDE.md, consolidate the rest, then build.
5. **Honest scope.** 3,400 LOC that works beats 30,000 LOC that's half stubs.

---

## Success Criteria

| Release | Ship Date Target | Key Metric |
|---------|-----------------|------------|
| v0.2.0 | Sprint 1 end | MCP server passes compliance test, all bugs fixed |
| v0.3.0 | Sprint 2 end | Agent run creates worktree, runs in isolation, branch visible in git |
| v0.4.0 | Sprint 3 end | 3 sub-agents run in parallel worktrees, UI shows per-agent progress |
