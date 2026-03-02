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

| File | Summary | Status | Commit |
|------|---------|--------|--------|
| TASK_11 | Configurable host/port | done | `f189045` |
| TASK_12 | E2E smoke test script | done | `b9bd76c` |
| TASK_13 | GitHub Release workflow | done | `905a646` |
| TASK_14 | README.md | done | `98f9152` |
| TASK_15 | Update NORTH_STAR.md | done | `3227afb` |
| TASK_16 | Markdown rendering | done | `6ce369f` |
| TASK_17 | Tool use collapsible panels | done | `470047f` |
| TASK_18 | Circuit breaker | done | `43a32ae` |
| TASK_19 | Rate limiter | done | `eba0550` |
| TASK_20 | Cost tracking | done | `a1e0484` |

## Recommended execution order

### Batch 1 (v0.1.0) — COMPLETE

1. ~~TASK_05~~ (critical — v0.1.0 blocker)
2. ~~TASK_07 + TASK_09 + TASK_10~~ in parallel
3. ~~TASK_06~~ (touches main.rs and lib.rs)
4. ~~TASK_08~~ last (needed TASK_07)

### Batch 2 (v0.2.0) — COMPLETE

1. ~~TASK_11 + TASK_12 + TASK_13 + TASK_14 + TASK_15~~ all in parallel
2. ~~TASK_16 + TASK_18~~ in parallel
3. ~~TASK_17~~ after TASK_16
4. ~~TASK_19~~ after TASK_18
5. ~~TASK_20~~ last

Post-batch fixes: SafetyState refactor (`9879974`), release workflow fix (`0e76a0b`), BatchWriter timestamp fix (`cb22ee1`), agent name validation (`fc5b951`), budget events (`d88c650`), MCP design doc (`2d07376`).
