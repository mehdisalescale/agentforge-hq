# Forge Project Audit Report

> Full audit of all files in `forge-project/` — 161 files, 34,316 lines.
> Audited: 2026-02-26. Auditor: Claude (principal architect role).

---

## Summary

| Category | Critical | Major | Minor | Info | Total |
|----------|----------|-------|-------|------|-------|
| Vision/Strategy/Requirements | 3 | 12 | 10 | 6 | 31 |
| Architecture & Design | 6 | 12 | 10 | 3 | 31 |
| Engineering/Planning/Methodology | 5 | 16 | 14 | 4 | 39 |
| Rust Code & SQL | 3 | 7 | 9 | 9 | 28 |
| Cross-doc Consistency | 2 | 3 | 5 | 1 | 11 |
| **TOTAL** | **19** | **50** | **48** | **23** | **140** |

---

## Top 10 Systemic Issues

### 1. "61 repos" everywhere — should be 62

**Severity: Major | 20+ instances across 15+ files**

Every file written by the original 7 agents uses "61" but the actual submodule count (after adding `claude-code`) is 62. Affected files include: VISION_AND_MISSION.md, PRODUCT_PRINCIPLES.md, COMPETITIVE_LANDSCAPE.md, MARKET_ANALYSIS.md, VALUE_PROPOSITION.md, WARDLEY_MAP.md, PRD.md, FEATURE_CATALOG.md, ROADMAP.md, RISK_REGISTER.md, ABSORPTION_PIPELINE.md, FEATURE_SOURCE_MAP.md, TECH_REFERENCES.md, NORTH_STAR.md.

Only SUBMODULE_TRACKING.md and PRODUCT_JOURNEY.md correctly say 62.

### 2. SPRINT_PLAN and MILESTONES use refactoring language

**Severity: Critical | SPRINT_PLAN.md, MILESTONES.md**

These two files still say "extract," "migrate v1 to v2," "preserve existing functionality," and "pre-refactor." The decision was **from-scratch** (ROADMAP decision log, line 588). These files were never updated after the rewrite decision.

Specific instances:
- SPRINT_PLAN line 32: "extract core types and traits"
- SPRINT_PLAN line 37: "Preserve all existing functionality"
- SPRINT_PLAN line 48: "Move process spawning logic"
- SPRINT_PLAN line 95: "migration from v1 to v2"
- MILESTONES line 34: "Database v2 schema"
- MILESTONES line 55: "database migration from v1 to v2"
- MILESTONES line 56: "pre-refactor features"

### 3. Phase ordering mismatch: ROADMAP vs SPRINT_PLAN vs MILESTONES

**Severity: Critical | 3 files**

The three planning documents describe different phase orderings:

| Phase | ROADMAP (canonical) | SPRINT_PLAN | MILESTONES |
|-------|-------------------|-------------|------------|
| Foundation | Phase 0, Wk 1-4 | Sprint 1-2, Wk 1-3 | M0, Wk 3 |
| Agent Engine | Phase 1, Wk 5-8 | **Missing** | **Missing** |
| Safety+MCP | Phase 4, Wk 9-12 (parallel) | Sprint 3-4, Wk 4-7 (**wrong**) | M1, Wk 7 (**wrong**) |
| Workflows | Phase 2, Wk 9-13 | Sprint 5-8, Wk 8-15 | M2, Wk 12 |
| Observability | Phase 3, Wk 14-18 | Sprint 9-10, Wk 16-19 | M3, Wk 17 |
| Notifications | Not a phase | Not present | M4, Wk 21 (**invented**) |
| Plugins | Phase 5, Wk 19-24 | Sprint 11-12, Wk 20-23 | M5, Wk 27 |
| Dev Environment | Phase 6, Wk 25-29 | Not present | M6, Wk 32 |

Agent Engine (ROADMAP Phase 1) is completely absent from both SPRINT_PLAN and MILESTONES.

### 4. Two conflicting agent schemas

**Severity: Critical | DATA_MODEL.md vs PHASE0_IMPLEMENTATION_PLAN.md**

DATA_MODEL.md defines agents as:
```
agents(id, name, config TEXT, session_id, status, usage, created_at, updated_at)
```
Config is a single JSON blob containing all agent configuration.

PHASE0_IMPLEMENTATION_PLAN.md defines agents as:
```
agents(id, name, model, system_prompt, allowed_tools, max_turns, use_max, preset, config_json, created_at, updated_at)
```
Denormalized columns for each field.

The Phase 0 plan is the intended approach for the new from-scratch codebase. DATA_MODEL.md documents the old Forge prototype schema.

### 5. Two conflicting event models

**Severity: Critical | EVENT_SYSTEM.md vs PHASE0_IMPLEMENTATION_PLAN.md**

EVENT_SYSTEM.md documents `TaggedEvent` wrapping raw `serde_json::Value` from Claude Code stdout. It defines an `EventType` enum with 23 variants mixing Claude event types with Forge internal events. Serde strategy: `#[serde(untagged)]` and `#[serde(tag = "kind")]`.

PHASE0_IMPLEMENTATION_PLAN.md defines `ForgeEvent` as a typed Rust enum with exactly 20 variants. Serde strategy: `#[serde(tag = "type", content = "data")]`.

These are incompatible approaches. The Phase 0 plan is the intended approach for the new codebase.

### 6. Crate count: 8 vs 12 vs 14

**Severity: Major | TECH_STACK.md, CODING_STANDARDS.md, SYSTEM_ARCHITECTURE.md**

| Document | Count |
|----------|-------|
| ROADMAP.md | 8 initially |
| PHASE0_IMPLEMENTATION_PLAN.md | 8 crates |
| TECH_STACK.md | "12 workspace crates" (8 instances) |
| CODING_STANDARDS.md | "12 workspace crates" (4 instances) |
| SYSTEM_ARCHITECTURE.md | 14 crates listed |

The initial Phase 0 build has 8 crates. Later phases add more. The final count depends on how features are organized but is ~14, not 12.

### 7. FEATURE_CATALOG summary stats are wrong

**Severity: Critical | FEATURE_CATALOG.md**

The summary section (line 480+) claims 213 total features. Summing all 12 bounded context subtotals: 30+23+20+25+25+17+25+30+20+22+25+43 = **305**. All derived statistics (priority breakdown, effort distribution, status counts) are based on the wrong denominator.

### 8. SK-002 says "500+ skills" — should be "1,500+"

**Severity: Critical | FEATURE_CATALOG.md line 188**

PRD (FR-SK-001): "1,500+ imported skills". OKR (KR5.1): "1,500+". Vision doc: "80% of 1,537 identified skills". FEATURE_CATALOG SK-002: "Import 500+ skills." This is an early draft value that was not updated.

### 9. Redundant planning docs

**Severity: Major | 3 files**

Three files are redundant and should be removed or archived:

| File | Superseded By | Reason |
|------|--------------|--------|
| `PHASE0_PLAN_FOR_CLAUDE.md` | `PHASE0_IMPLEMENTATION_PLAN.md` | Condensed copy, zero unique content |
| `PHASE0_PARALLEL_TRACKS.md` | `CURSOR_AGENT_PROMPTS.md` | 6-track model conflicts with the 4-agent model actually used |
| `PARALLEL_AGENTS.md` | `CURSOR_AGENT_PROMPTS.md` | Shorter versions of the same prompts |

### 10. NORTH_STAR.md and SESSION_LOG.md are stale

**Severity: Critical | 2 files**

NORTH_STAR.md line 30: "What's Built: Nothing yet" — but forge-core, forge-agent, and forge-db exist with 13 passing tests (documented in AGENT_WORK_CHECK.md).

SESSION_LOG.md: Shows only 3 sessions (up to 2026-02-25). Missing at least 2 sessions: the 4-agent parallel execution and the audit/fix session.

---

## Detailed Findings by Category

---

### Category 1: Vision, Strategy, Requirements

#### 00-vision/VISION_AND_MISSION.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 7 | "sixty-one fragmented tools" → should be 62 |
| Major | 14 | "Consolidating 61+ reference repositories" → 62 |
| Minor | 41 | "100+ agent presets" is aspirational but not distinguished from current 9 |
| Info | 57-69 | Fragmentation table sums to 61 (correct for community repos, but doesn't note Claude Code as 62nd) |

#### 00-vision/PRODUCT_PRINCIPLES.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 11 | "The 61 reference repos" → 62 |
| Major | 80 | "The 61 reference repos" → 62 |

#### 01-strategy/COMPETITIVE_LANDSCAPE.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 346 | "61+ reference repositories" → 62 |
| Major | 410 | "61-repo absorption" → 62 |

#### 01-strategy/MARKET_ANALYSIS.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 5, 48, 63, 81, 98, 299, 315 | All say "61" → 62 (7 instances) |

#### 01-strategy/VALUE_PROPOSITION.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 11, 143, 186 | All say "61" → 62 (3 instances) |

#### 01-strategy/WARDLEY_MAP.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 75, 256, 358 | All say "61" → 62 (3 instances) |

#### 02-requirements/PRD.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 25, 32, 78, 89, 1133 | All say "61" → 62 (5 instances) |
| Minor | 6 | "Version: 1.0" + "Status: Draft" contradictory |
| Minor | 96-97 | KR1.4 says "200+ features" but catalog has 305 |
| Minor | 1101 | "Rust 1.75+ (2024 edition)" — 2024 edition requires 1.85+ |
| Info | — | No mention of 8 crate structure, default model, or 24-week timeline |

#### 02-requirements/FEATURE_CATALOG.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 188 | SK-002: "Import 500+ skills" → 1,500+ |
| Critical | 480, 488-511 | Total says 213, actual is 305. All derived stats wrong |
| Major | 612 | "61 Source Repositories" → 62 |
| Minor | 482 | "Percentages sum to > 100% due to rounding" — misleading, denominator is wrong |
| Minor | 624-641 | Priority distribution chart based on wrong totals |

#### 02-requirements/USER_STORIES.md

| Severity | Line | Issue |
|----------|------|-------|
| Minor | 55 | US-AM-002: "Preset browser shows 100+ presets" — initial ship has 9 |
| Minor | 1103-1115 | Confusing "(P1 counted)" annotations in summary |

#### 02-requirements/USER_PERSONAS.md

No issues found. Well-constructed, internally consistent.

---

### Category 2: Architecture & Design

#### 03-architecture/SYSTEM_ARCHITECTURE.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 509-574 | Lists 14 crates, not 8. Missing `forge-agent`. Has extras: `forge-events`, `forge-ws`, `forge-workflow`, `forge-git`, `forge-sessions`, `forge-skills`, `forge-presets`, `forge-assets`, `forge-bin` |
| Major | 100-101 | API route prefix: `/api/*` unversioned, but Phase 0 uses `/api/v1/` |
| Minor | 4 | LOC estimate "~33K" but crate sums total ~29.5K |

#### 03-architecture/API_DESIGN.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 4 | WebSocket path: `/ws` here, `/api/v1/ws` in Phase 0 plan |
| Major | 149-316 | Agent request body has `mcp_servers`, `hooks`, `max_budget_usd` — not in Phase 0 schema |
| Major | 400 | Preset names: `planner, reviewer, bug-hunter, refactor, security, fullstack, tester, docs, quick` — completely different from Phase 0's `CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer` |
| Minor | 1266 | WebSocket listed as `GET /ws` — minor nit |

#### 03-architecture/DATA_MODEL.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 49-57 | Agent table schema conflicts with Phase 0 plan (JSON blob vs denormalized columns) |
| Critical | 320-406 | Only 1 FTS5 table (`fts_events`), Phase 0 plan has 3 (`skills_fts`, `sessions_fts`, `events_fts`) |
| Major | 339 | FTS trigger uses `json_extract(new.event, '$.message.content[0].text')` for `result` events — wrong JSON path |
| Major | 325-327 | Content-synced FTS5 uses `content_rowid='id'` but `id` may be TEXT, not INTEGER |
| Minor | 30 | `Arc<Mutex<Connection>>` correct for DB but should note this is DB-specific |

#### 03-architecture/EVENT_SYSTEM.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 43-73 | `EventType` has 23 variants (6 Claude + 17 Forge), Phase 0 plan has 20 typed `ForgeEvent` variants — different models |
| Major | 107-109 | Serde: `#[serde(untagged)]` and `#[serde(tag = "kind")]` — Phase 0 plan uses `#[serde(tag = "type", content = "data")]` |
| Minor | 234-238 | `tool_result` arrives as `"type": "user"` but stored as `event_type = "tool_result"` — classification logic undocumented |

#### 03-architecture/MCP_INTERFACE.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 897-899 | `forge_read_claude_md`, `forge_write_claude_md`, `forge_list_worktrees`, `forge_remove_worktree` listed in summary but have no tool definitions |
| Minor | 93-113 | MCP protocol version `"2024-11-05"` — may need updating |

#### 03-architecture/BOUNDED_CONTEXTS.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 232 | `register_agent()` in Process Execution — belongs in Agent Management |
| Minor | 357 | "Database Tables Owned" lists `workflow_steps` but omits `workflow_runs` |
| Minor | 443 | `POST /api/presets` listed but not in API_DESIGN.md endpoint summary |

#### 04-design/UI_DESIGN.md

| Severity | Line | Issue |
|----------|------|-------|
| Minor | 54 | TailwindCSS 4 config — Phase 0 plan still references `tailwind.config.js` (v3 pattern) |
| Info | 327-337 | Model name format inconsistent: `sonnet-4` vs `claude-sonnet-4-20250514` |

#### 04-design/INFORMATION_ARCHITECTURE.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 158-182 | FTS5 table schemas differ from DATA_MODEL.md and Phase 0 plan |
| Minor | 413-478 | WebSocket message types differ from API_DESIGN.md protocol |

---

### Category 3: Engineering, Planning, Methodology

#### 06-planning/ROADMAP.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 3 | "61 reference repos" → 62 |
| Critical | 4 | "~27 weeks" → 24 weeks (matches line 546) |
| Major | 59, 588 | "61 reference repos" and "61-repo analysis" → 62 |
| Major | 77-83 | "8 initial crates" — correct, but TECH_STACK says 12 |
| Minor | 134 | "single binary < 15 MB" — could confuse vs 30-50 MB final target |

#### 06-planning/SPRINT_PLAN.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 11-24 | Phase mapping completely wrong vs ROADMAP (see Systemic Issue #3) |
| Major | 32-35 | "Establish the 12-crate workspace structure" — should be 8, uses refactoring language |
| Major | 37 | "Preserve all existing functionality" — contradicts from-scratch decision |
| Major | 48 | "Move process spawning logic" — refactoring language |
| Major | 95-96 | "migration from v1 to v2" — no v1 exists in from-scratch build |
| Minor | 1 | Covers "Phases 0-3" in 24 weeks but ROADMAP shows 0-3 ends at Week 18 |

#### 06-planning/MILESTONES.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 10-19 | Milestone phases completely reordered vs ROADMAP, Agent Engine missing |
| Major | 34 | "Database v2 schema" — no v2 in from-scratch build |
| Major | 55 | "database migration from v1 to v2" — contradicts from-scratch |
| Major | 56 | "pre-refactor features" — contradicts from-scratch |

#### 06-planning/RISK_REGISTER.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 159 | "Absorbing 61 reference repos" → 62 |
| Minor | 213 | "12 crate tests" → "all workspace crate tests" |

#### 05-engineering/TECH_STACK.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 4, 50, 206, 214, 289 | "12 workspace crates" → "8+ initially" (8 instances total) |
| Minor | 14 | TypeScript "5.6" → "5.6+" |

#### 05-engineering/CI_CD.md

| Severity | Line | Issue |
|----------|------|-------|
| Minor | 139-140 | Windows build targets listed but TECH_STACK says "Windows: Not supported" |
| Minor | 208-209 | `lto = "fat"` conflicts with TECH_STACK recommendation of `lto = "thin"` |

#### 05-engineering/CODING_STANDARDS.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 3, 50, 214, 289 | "12 workspace crates" → "all workspace crates" (4 instances) |

#### 07-methodology/ABSORPTION_PIPELINE.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 9, 486 | "61 repositories" → 62 (2 instances) |

#### 07-methodology/DEVELOPMENT_PROCESS.md

| Severity | Line | Issue |
|----------|------|-------|
| Info | 166 | "renders correctly in both dark and light themes" — CODING_STANDARDS says dark only |

#### 07-methodology/QUALITY_GATES.md

| Severity | Line | Issue |
|----------|------|-------|
| Minor | 328-329 | "Dark theme works" + "Light theme works" — CODING_STANDARDS says dark only |

#### 08-reference/FEATURE_SOURCE_MAP.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 3 | "all 61 reference repositories" — table only lists 60 individual repos, canonical count is 62 |
| Minor | 73 | Confusing placeholder note should be removed after fixing |

#### 08-reference/TECH_REFERENCES.md

| Severity | Line | Issue |
|----------|------|-------|
| Major | 91 | "All 61 Reference Repositories" → 62 |
| Minor | 218 | "GitHub URLs are placeholders" — TODO not tracked |

---

### Category 4: Rust Code & SQL Migration

#### Cargo.toml (workspace root)

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 1-3 | Missing `[workspace.dependencies]` — each crate inlines versions independently |
| Critical | 3 | `members` lists only 3 crates, plan says `["crates/*"]` for all 8 |

#### forge-core/Cargo.toml

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 14 | `rusqlite` dependency in core crate — forces all consumers to pull sqlite. Exists because `ForgeError::Database(#[from] rusqlite::Error)`. Matches plan but is an architectural concern |
| Major | 14 | `rusqlite` missing `fts5` and `vtab` features |

#### forge-db/src/batch_writer.rs

| Severity | Line | Issue |
|----------|------|-------|
| Major | 93 | `conn.unchecked_transaction()` — should use `conn.transaction()` for safety |
| Major | 104-105 | Timestamp uses `Utc::now()` (flush time) instead of event's embedded timestamp |
| Minor | 46 | `shutdown(self)` consumes self — cannot be called when stored in `Arc<BatchWriter>` |
| Minor | 152-153 | Redundant `use forge_core::events::ForgeEvent` inside function body |

#### forge-db/src/repos/agents.rs

| Severity | Line | Issue |
|----------|------|-------|
| Major | 39 | Preset serialized via `format!("{:?}", p)` (Debug trait) — fragile, should use serde or `as_str()` |
| Major | 169, 189, 192 | UUID/timestamp parse errors use `InvalidParameterName` — semantically wrong error type |
| Minor | 96-152 | `update` method does not call `validate_update_agent` |

#### forge-db/src/pool.rs

| Severity | Line | Issue |
|----------|------|-------|
| Major | 8-10 | `Arc<Mutex<Connection>>` — requires careful `drop` before re-acquiring in create/update methods. Fragile pattern |
| Minor | 32-36 | `in_memory()` skips WAL/sync pragmas — correct for in-memory but undocumented |

#### forge-agent/src/validation.rs

| Severity | Line | Issue |
|----------|------|-------|
| Minor | 7-24 | No character-set validation — plan says "alphanumeric + hyphens + underscores" |
| Minor | 1 | `validate_update_agent` not re-exported from lib.rs |

#### forge-core/src/events.rs

| Severity | Line | Issue |
|----------|------|-------|
| Minor | 10 | No `PartialEq` derive — `f64` fields make it debatable, but would simplify testing |

#### forge-core/src/ids.rs

| Severity | Line | Issue |
|----------|------|-------|
| Minor | 7-20 | No `From<Uuid>` or `TryFrom<String>` impls — consumers access `.0` directly |

#### What's Correct (Confirmed)

- ForgeEvent: 20 variants, correct `#[serde(tag = "type", content = "data")]`
- All 5 ID types: AgentId, SessionId, EventId, WorkflowId, SkillId
- ForgeError: 8 variants with correct `#[from]` attributes
- Agent struct: all 11 fields correct
- AgentPreset: all 9 variants with non-empty system prompts
- BatchWriter: 50-event/2s flush with crossbeam-channel
- Migration SQL: character-for-character match with plan
- DbPool: WAL mode, foreign_keys ON, cache_size -8000

---

### Category 5: Cross-Document Consistency

#### Variant count claims

| Severity | File | Issue |
|----------|------|-------|
| Minor | PHASE0_PLAN_FOR_CLAUDE.md line 65 | "~22 variants" → actual is 20 |
| Minor | AGENT_WORK_CHECK.md line 20 | "22 variants" → actual is 20 |

#### 4-agent vs 6-track decomposition conflict

| Severity | File | Issue |
|----------|------|-------|
| Major | PHASE0_PARALLEL_TRACKS.md | 6 tracks (A-F): Core, DB, Agent, API, Frontend, App |
| — | PARALLEL_AGENTS.md | 4 agents (A-D): Core+Agent, DB, Frontend, API+App |
| — | CURSOR_AGENT_PROMPTS.md | 4 agents (A-D): same as PARALLEL_AGENTS |

The 4-agent model was actually used (per AGENT_WORK_CHECK.md). The 6-track model was never executed and creates confusion.

#### Stale state in NORTH_STAR.md

| Severity | Line | Issue |
|----------|------|-------|
| Critical | 30 | "What's Built: Nothing yet" — forge-core, forge-agent, forge-db exist with 13 tests |
| Minor | 36-43 | Priority table lists "Run migrate-to-submodules.sh" as P0 — already done |
| Minor | 27 | "34 design docs" — count may be stale |

#### Missing sessions in SESSION_LOG.md

| Severity | Issue |
|----------|-------|
| Critical | Only 3 sessions logged. Missing: 4-agent parallel execution session and audit/fix session |

#### include_str! path

| Severity | File | Issue |
|----------|------|-------|
| Minor | CURSOR_AGENT_PROMPTS.md Agent B | `include_str!("../../migrations/0001_init.sql")` — should be `"../../../migrations/0001_init.sql"` (3 levels up from `crates/forge-db/src/`) |

---

## Architecture Conflict Summary Table

| Topic | Doc A | Doc B | Which Is Canonical |
|-------|-------|-------|--------------------|
| API route prefix | API_DESIGN: `/api/agents` | Phase 0: `/api/v1/agents` | Phase 0 plan |
| WebSocket path | API_DESIGN: `/ws` | Phase 0: `/api/v1/ws` | Phase 0 plan |
| Crate count | SYSTEM_ARCHITECTURE: 14 | Phase 0: 8 initially | Phase 0 plan (8 now, more later) |
| Agent schema | DATA_MODEL: JSON blob | Phase 0: denormalized columns | Phase 0 plan |
| Event model | EVENT_SYSTEM: TaggedEvent | Phase 0: ForgeEvent enum | Phase 0 plan |
| FTS5 tables | DATA_MODEL: 1 table | Phase 0: 3 tables | Phase 0 plan |
| Preset names | API_DESIGN: `bug-hunter, planner...` | Phase 0: `Debugger, Architect...` | Phase 0 plan |
| Theme support | CODING_STANDARDS: dark only | QUALITY_GATES: dark + light | Dark only for now |
| Rust edition | PRD: 1.75+ | Reality: 2024 edition needs 1.85+ | 1.85+ |
| LTO strategy | TECH_STACK: thin | CI_CD: fat | Decide (thin is faster builds) |
| Windows support | CI_CD: builds Windows | TECH_STACK: not supported | Clarify intent |

---

## Document Disposition Recommendation

### Keep (Essential)

| File | Status |
|------|--------|
| NORTH_STAR.md | **Needs update** — stale "nothing built" state |
| SESSION_LOG.md | **Needs update** — missing 2+ sessions |
| PRODUCT_JOURNEY.md | Good — unique phase narrative |
| PHASE0_IMPLEMENTATION_PLAN.md | Good — canonical detailed plan |
| CURSOR_AGENT_PROMPTS.md | Good — self-contained Cursor prompts |
| AGENT_WORK_CHECK.md | Good — post-execution audit record |
| PHASE0_SHARED_CONTRACT.md | Good — quick-reference contracts |

### Remove (Redundant or Conflicting)

| File | Superseded By | Reason |
|------|--------------|--------|
| PHASE0_PLAN_FOR_CLAUDE.md | PHASE0_IMPLEMENTATION_PLAN.md | Condensed copy, zero unique content |
| PHASE0_PARALLEL_TRACKS.md | CURSOR_AGENT_PROMPTS.md | 6-track model conflicts with 4-agent model actually used |
| PARALLEL_AGENTS.md | CURSOR_AGENT_PROMPTS.md | Shorter versions of the same prompts |

### Needs Major Rewrite

| File | Reason |
|------|--------|
| SPRINT_PLAN.md | Wrong phase order, refactoring language throughout |
| MILESTONES.md | Missing Agent Engine, refactoring language, wrong phase order |
| FEATURE_CATALOG.md | Wrong totals (213→305), wrong skill count (500→1,500) |

### Needs Minor Updates (61→62, crate count, etc.)

All files in 00-vision/, 01-strategy/, 02-requirements/PRD.md, 05-engineering/, 06-planning/ROADMAP.md, 06-planning/RISK_REGISTER.md, 07-methodology/, 08-reference/.

---

## Rust Code Fix Priority

| Priority | File | Fix |
|----------|------|-----|
| 1 | Cargo.toml | Add `[workspace.dependencies]`, use `workspace = true` in sub-crates |
| 2 | rusqlite dep | Add `fts5` and `vtab` features |
| 3 | batch_writer.rs | Use event's embedded timestamp, not `Utc::now()` at flush time |
| 4 | agents.rs | Use serde or `as_str()` for preset serialization, not `Debug` |
| 5 | agents.rs | Fix `InvalidParameterName` error hack in row parsing |
| 6 | batch_writer.rs | Replace `unchecked_transaction()` with `transaction()` |
| 7 | validation.rs | Add character-set validation for agent names |
| 8 | validation.rs | Export `validate_update_agent` from lib.rs |
| 9 | agents.rs | Call `validate_update_agent` in `AgentRepo::update` |
| 10 | batch_writer.rs | Resolve `shutdown(self)` vs `Arc<BatchWriter>` conflict |
