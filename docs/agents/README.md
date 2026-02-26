# Agent Task Cards

Each file is a self-contained task for a Cursor agent.

## How it works

1. **Supervisor** (you) tells Cursor: "Read `docs/agents/TASK_XX.md` and do the work"
2. **Cursor** reads the task, implements it, then fills in the `## Report` section at the bottom
3. **Supervisor** reviews the report and code, then marks status as `done`

## Current tasks

| File | Summary | Status | Can parallel with |
|------|---------|--------|-------------------|
| TASK_01 | Session status in run handler | done | — |
| TASK_02 | SkillRepo + GET /api/v1/skills | done | — |
| TASK_03 | Clippy fixes | done | — |
| TASK_04 | CORS hardening | done | — |
| TASK_05 | rust-embed: single binary | done | — |
| TASK_06 | Graceful shutdown | done | — |
| TASK_07 | WorkflowRepo + API | done | — |
| TASK_08 | Frontend /skills + /workflows | done | — |
| TASK_09 | TraceLayer logging | done | — |
| TASK_10 | CI GitHub Actions | done | — |

### Phase A polish + Phase B (v0.2.0)

| File | Summary | Status | Can parallel with |
|------|---------|--------|-------------------|
| TASK_11 | Configurable host/port | pending | TASK_12, TASK_13, TASK_14, TASK_15 |
| TASK_12 | E2E smoke test script | pending | TASK_11, TASK_13, TASK_14, TASK_15 |
| TASK_13 | GitHub Release workflow | pending | TASK_11, TASK_12, TASK_14, TASK_15 |
| TASK_14 | README.md | pending | TASK_11, TASK_12, TASK_13, TASK_15 |
| TASK_15 | Update NORTH_STAR.md | pending | TASK_11, TASK_12, TASK_13, TASK_14 |
| TASK_16 | Markdown rendering | pending | TASK_18, TASK_19 |
| TASK_17 | Tool use collapsible panels | pending (needs TASK_16) | TASK_18, TASK_19, TASK_20 |
| TASK_18 | Circuit breaker | pending | TASK_16, TASK_17 |
| TASK_19 | Rate limiter | pending (needs TASK_18) | TASK_16, TASK_17 |
| TASK_20 | Cost tracking | pending | TASK_16, TASK_17 |

## Recommended execution order

### Batch 1 (v0.1.0) — COMPLETE

1. ~~TASK_05~~ (critical — v0.1.0 blocker)
2. ~~TASK_07 + TASK_09 + TASK_10~~ in parallel
3. ~~TASK_06~~ (touches main.rs and lib.rs)
4. ~~TASK_08~~ last (needed TASK_07)

### Batch 2 (v0.2.0)

1. **TASK_11 + TASK_12 + TASK_13 + TASK_14 + TASK_15** all in parallel (no file overlap, all standalone)
2. **TASK_16 + TASK_18** in parallel (frontend vs backend, no overlap)
3. **TASK_17** after TASK_16 (depends on markdown rendering)
4. **TASK_19** after TASK_18 (shares forge-safety + AppState)
5. **TASK_20** last (touches DB migration, API state, frontend, and run handler)
