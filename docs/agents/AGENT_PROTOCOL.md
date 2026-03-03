# Agent Coordination Protocol

> **Read this before starting work.** This is the protocol for Wave 1 parallel agents.

---

## Your Identity

You are one of 5 parallel agents (A through E) working on Wave 1. You each create NEW files with zero overlap. You should never need to edit the same file as another agent.

## Before Starting

1. Read `CLAUDE.md` (project context)
2. Read `NORTH_STAR.md` (current state)
3. Read your task card in `docs/agents/HANDOFF_SPRINT_2_3.md` (search for your TASK_W1x section)
4. Read `docs/agents/WAVE1_STATUS.md` (coordination file)
5. Update your section in WAVE1_STATUS.md: set status to `in_progress`

## While Working

### File Ownership Rules

Each agent owns specific files. **Do NOT touch files outside your ownership list.**

| Agent | Owns (exclusive) | Shared (read-only) |
|-------|-------------------|---------------------|
| A | `crates/forge-git/**` | `Cargo.toml` (workspace members line only) |
| B | `crates/forge-api/src/middleware.rs` | — |
| C | `skills/**`, `crates/forge-db/src/repos/skills.rs` | — |
| D | `crates/forge-db/src/repos/memory.rs`, `crates/forge-api/src/routes/memory.rs` | — |
| E | `crates/forge-db/src/repos/hooks.rs`, `crates/forge-api/src/routes/hooks.rs` | — |

**Cargo.toml conflict zone:** Only Agent A adds `forge-git` to workspace members. No other agent touches `Cargo.toml`.

### Progress Updates

Update `docs/agents/WAVE1_STATUS.md` at these checkpoints:
1. **Starting:** Set status to `in_progress`
2. **Files created:** List all files you created
3. **Testing:** Set status to `testing`, list tests added
4. **Done:** Set status to `done`, fill in final report
5. **Blocked:** Set status to `blocked`, describe the issue in Shared Issues table

### If You Hit a Problem

1. Check if it's in your scope — if yes, fix it
2. If it involves another agent's files — write it to the Shared Issues table in WAVE1_STATUS.md
3. If it's a build/compile error from existing code — note it in your report, do NOT fix other agents' code
4. If you need a decision — write it to Shared Issues and set yourself to `blocked`

## When Done

1. Run your verify commands (listed in your task card)
2. Update WAVE1_STATUS.md: set status to `done`, fill in all fields
3. **Do NOT commit.** The coordinator will review and commit your work.
4. **Do NOT start Wave 2 tasks.** Wait for coordinator.

## What NOT To Do

- Do NOT edit files outside your ownership list
- Do NOT run `cargo test --workspace` (may fail if other agents haven't finished)
- Do NOT modify `NORTH_STAR.md`, `MASTER_TASK_LIST.md`, or `CLAUDE.md`
- Do NOT create migration SQL files (Wave 2 integration agent handles that)
- Do NOT add routes to `routes/mod.rs` or `state.rs` (Wave 2 integration agent handles that)
- Do NOT push to git (coordinator handles merging)
