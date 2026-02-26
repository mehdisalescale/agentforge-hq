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
| TASK_05 | rust-embed: single binary | pending | TASK_09, TASK_10 |
| TASK_06 | Graceful shutdown | pending | TASK_07, TASK_09, TASK_10 |
| TASK_07 | WorkflowRepo + API | pending | TASK_06, TASK_09, TASK_10 |
| TASK_08 | Frontend /skills + /workflows | pending (needs TASK_07) | TASK_09, TASK_10 |
| TASK_09 | TraceLayer logging | pending | any |
| TASK_10 | CI GitHub Actions | pending | any |

## Recommended execution order

1. **TASK_05** (critical — v0.1.0 blocker)
2. **TASK_07 + TASK_09 + TASK_10** in parallel (no file overlap)
3. **TASK_06** (touches main.rs and lib.rs — do after TASK_05)
4. **TASK_08** last (needs TASK_07 done for workflows)
