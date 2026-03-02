# Documentation Index

> What's current, what's archived, what to read.
> Updated: 2026-03-02

---

## Active Documents (keep updated)

| File | Purpose | Update Frequency |
|------|---------|-----------------|
| `NORTH_STAR.md` | Single source of truth — vision, state, sprint plan | Every session |
| `MASTER_TASK_LIST.md` | Sprint tasks with What/Where/How/Verify | Every sprint |
| `README.md` | GitHub landing page — quick start, config, architecture | Every release |
| `docs/SESSION_LOG.md` | Session history | Every session |

## Current Reference (read as needed, don't update frequently)

| File | Purpose | Last Updated |
|------|---------|-------------|
| `docs/FORGE_AUDIT_2026_03_02.md` | Full audit: per-crate grades, gap analysis, proposal | 2026-03-02 |
| `docs/BORROWED_IDEAS.md` | DeerFlow + Claude-Flow + reference repos + industry research | 2026-03-02 |
| `docs/DOC_INDEX.md` | This file | 2026-03-02 |
| `docs/MCP_DESIGN.md` | MCP server design notes | 2026-02-27 |
| `docs/E2E_SMOKE_TEST.md` | E2E test documentation | 2026-02-27 |
| `docs/HARVESTER_INTEGRATION.md` | Harvester ↔ Forge integration assessment and roadmap | 2026-03-02 |

## Historical Task Records (read-only, don't update)

| File | Purpose |
|------|---------|
| `docs/agents/TASK_01.md` through `TASK_20.md` | Agent task completion records for Phase 0-B |
| `docs/agents/HANDOFF_BATCH_2.md` | Batch 2 handoff specification |
| `docs/agents/README.md` | Agent task system overview |

## Archived — Superseded by Audit (read for context only)

These docs were valid during earlier planning phases but have been **superseded** by the 2026-03-02 audit and new sprint plan. Do NOT update them.

| File | Superseded By |
|------|---------------|
| `docs/WHAT_TO_DO_NEXT.md` | `NORTH_STAR.md` sprint plan |
| `docs/REMAINING_APP_PLAN.md` | `MASTER_TASK_LIST.md` |
| `docs/PROPOSAL_2_3_4.md` | `docs/FORGE_AUDIT_2026_03_02.md` Phase 4 proposal |
| `docs/STRATEGIC_ASSESSMENT.md` | `docs/FORGE_AUDIT_2026_03_02.md` |
| `docs/EXECUTIVE_SUMMARY.md` | `docs/FORGE_AUDIT_2026_03_02.md` |
| `docs/AUDIT_REPORT.md` | `docs/FORGE_AUDIT_2026_03_02.md` |
| `docs/PRODUCT_JOURNEY.md` | `NORTH_STAR.md` |
| `docs/REFERENCE_REPOS.md` | `docs/BORROWED_IDEAS.md` |
| `docs/PHASE1_DESIGN_NOTES.md` | `MASTER_TASK_LIST.md` |

## Archived — Old Planning (frozen 2026-02-26)

These 14 files in `docs/planning/` were from the original multi-agent planning process. All superseded by the audit.

| File | Was For |
|------|---------|
| `docs/planning/PHASE0_IMPLEMENTATION_PLAN.md` | Phase 0 plan (completed) |
| `docs/planning/PHASE0_REMAINING.md` | Phase 0 remaining (completed) |
| `docs/planning/PHASE0_SHARED_CONTRACT.md` | Agent coordination contract |
| `docs/planning/PHASE0_WORK_CHECK.md` | Phase 0 verification |
| `docs/planning/PHASE1_6_AGENT_SPRINT.md` | Phase 1 sprint (completed) |
| `docs/planning/CURSOR_AGENT_PROMPTS.md` | Agent prompts for Cursor |
| `docs/planning/CURSOR_TASKS.md` | Task list for Cursor agents |
| `docs/planning/NEXT_PHASE_AGENT_PROMPTS.md` | Next phase prompts |
| `docs/planning/NEXT_SPRINT_AGENT_TASKS.md` | Next sprint tasks |
| `docs/planning/AGENT_WORK_CHECK.md` | Agent work verification |
| `docs/planning/AUDIT_REMEDIATION.md` | Doc audit remediation |
| `docs/planning/CHECKLIST_RESULTS.md` | Checklist results |
| `docs/planning/OLD_VS_NEW_PARITY.md` | Old vs new comparison |
| `docs/planning/REFERENCE_CODE_METHODOLOGY.md` | Reference code methodology |

## Frozen Reference Material (00-08 directories)

These 35 files across 9 numbered directories were the original planning docs generated in Session 2. Frozen as reference on 2026-02-26. Do NOT update.

```
00-vision/       PRODUCT_PRINCIPLES.md, VISION_AND_MISSION.md
01-strategy/     COMPETITIVE_LANDSCAPE.md, MARKET_ANALYSIS.md, VALUE_PROPOSITION.md, WARDLEY_MAP.md
02-requirements/ FEATURE_CATALOG.md, PRD.md, USER_PERSONAS.md, USER_STORIES.md
03-architecture/ API_DESIGN.md, BOUNDED_CONTEXTS.md, DATA_MODEL.md, EVENT_SYSTEM.md, MCP_INTERFACE.md, SYSTEM_ARCHITECTURE.md
04-design/       INFORMATION_ARCHITECTURE.md, UI_DESIGN.md
05-engineering/  CI_CD.md, CODING_STANDARDS.md, DEPENDENCY_GRAPH.md, TECH_STACK.md, TESTING_STRATEGY.md
06-planning/     MILESTONES.md, RISK_REGISTER.md, ROADMAP.md, SPRINT_PLAN.md
07-methodology/  ABSORPTION_PIPELINE.md, DEVELOPMENT_PROCESS.md, QUALITY_GATES.md, SUBMODULE_TRACKING.md
08-reference/    FEATURE_SOURCE_MAP.md, GLOSSARY.md, TECH_REFERENCES.md, TREND_26FEB_ENHANCEMENT_MAP.md
```

**When to consult these:** If you need the original design rationale for a specific feature (e.g., event system design → `03-architecture/EVENT_SYSTEM.md`). Don't use them as task lists — use `MASTER_TASK_LIST.md` instead.
