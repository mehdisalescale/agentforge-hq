# Batch 2 Handoff — v0.2.0

**For:** Cursor agent
**From:** Supervisor
**Date:** 2026-02-27

---

## Instructions

For each task below, read the task card file and implement everything it describes. After completing each task:

1. Fill in the `## Report` section at the bottom of the task card
2. Run the verify commands listed in the task card
3. Commit your work with a descriptive message
4. Move to the next task

Task cards are in `docs/agents/TASK_XX.md`. Each one is self-contained with context, steps, file lists, and verification commands.

---

## Wave 1 — Do all 5 in parallel (no file overlap)

These tasks are completely independent. No shared files.

1. Read `docs/agents/TASK_11.md` and do the work — **Configurable host/port**
2. Read `docs/agents/TASK_12.md` and do the work — **E2E smoke test script**
3. Read `docs/agents/TASK_13.md` and do the work — **GitHub Release workflow**
4. Read `docs/agents/TASK_14.md` and do the work — **README.md**
5. Read `docs/agents/TASK_15.md` and do the work — **Update NORTH_STAR.md**

Verify after Wave 1:
```bash
cargo test --workspace
cargo clippy --workspace
```

---

## Wave 2 — Do these 2 in parallel (frontend vs backend, no overlap)

6. Read `docs/agents/TASK_16.md` and do the work — **Markdown rendering in output stream**
   - Frontend only: `frontend/src/routes/+page.svelte`, `frontend/package.json`
7. Read `docs/agents/TASK_18.md` and do the work — **Circuit breaker**
   - Backend only: `crates/forge-safety/`, `crates/forge-api/`, `crates/forge-app/`

Verify after Wave 2:
```bash
cargo test --workspace
cargo clippy --workspace
cd frontend && pnpm build
```

---

## Wave 3 — Sequential, one at a time

8. Read `docs/agents/TASK_17.md` and do the work — **Tool use collapsible panels**
   - Depends on TASK_16 (uses the markdown renderer it adds)
9. Read `docs/agents/TASK_19.md` and do the work — **Rate limiter**
   - Depends on TASK_18 (follows same pattern in forge-safety + AppState)

Verify after Wave 3:
```bash
cargo test --workspace
cargo clippy --workspace
cd frontend && pnpm build
```

---

## Wave 4 — Last

10. Read `docs/agents/TASK_20.md` and do the work — **Cost tracking**
    - Touches DB migration, session repo, run handler, API, and frontend
    - Do this last because it cuts across many files

Verify after Wave 4:
```bash
cargo test --workspace
cargo clippy --workspace
cd frontend && pnpm build
```

---

## Rules

- **Do not skip the verify step.** Every wave must pass `cargo test`, `cargo clippy`, and `pnpm build` before moving on.
- **Commit after each task**, not after each wave. One commit per task with a clear message.
- **Fill in the Report section** at the bottom of each task card when done.
- **Do not modify files outside** what the task card lists unless absolutely necessary. If you must, note it in your report.
- **If a task is unclear**, re-read the task card. The context section explains why. The task section explains what. The files section explains where.
