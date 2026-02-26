# Forge Project Audit — Remediation Checklist

> Ordered fixes from the 5-agent audit. Execute in this order; check off as done.
> **Do not start until explicit approval.** After approval, tackle 1 → 2 → 3 → 4 → 5 → 6.

---

## 1. Global 61 → 62 (repos count)

**Scope:** 20+ instances across 15+ files. Find-replace "61 repos" / "61 reference" / "61-repo" etc. with 62 (submodule count after adding claude-code).

**Action:** `grep -rl "61 repo\|61 reference\|61-repo\|61 repo" forge-project --include="*.md"` then replace with 62. Check NORTH_STAR, PRD, ROADMAP, REFERENCE_REPOS, PRODUCT_JOURNEY, and all 00–08 docs.

- [x] Run grep to list files
- [x] Replace in each file (preserve "62 repos" where already correct)

---

## 2. Remove 3 redundant files

**Per audit:** These conflict or duplicate; remove so one source of truth remains.

| File | Reason |
|------|--------|
| `PHASE0_PLAN_FOR_CLAUDE.md` | Duplicate of PHASE0_IMPLEMENTATION_PLAN.md |
| `PHASE0_PARALLEL_TRACKS.md` | 6-track model; 4-agent model (CURSOR_AGENT_PROMPTS) is used |
| `PARALLEL_AGENTS.md` | Superseded by CURSOR_AGENT_PROMPTS.md |

- [x] Delete PHASE0_PLAN_FOR_CLAUDE.md
- [x] Delete PHASE0_PARALLEL_TRACKS.md
- [x] Delete PARALLEL_AGENTS.md

---

## 3. SPRINT_PLAN.md + MILESTONES.md — rewrite for from-scratch

**Issue:** Both use refactoring language ("extract," "migrate v1 to v2," "preserve existing functionality," "pre-refactor"). Decision: from-scratch (ROADMAP). Phase ordering in SPRINT_PLAN wrong (Safety+MCP at 4–7 vs ROADMAP 9–12). MILESTONES omits Agent Engine.

**Action:**
- **SPRINT_PLAN.md:** Align phase order with ROADMAP (Phase 0→1→2∥4→3→5→6). Remove all "extract/migrate/preserve/pre-refactor" language. Describe sprints as building new crates/features, not migrating.
- **MILESTONES.md:** Add Agent Engine milestone. Remove refactoring wording. Align phase list with ROADMAP.

- [x] Rewrite SPRINT_PLAN.md (S1/S2 + calendar; S3–S12 pending)
- [x] Rewrite MILESTONES.md

---

## 4. FEATURE_CATALOG.md — fix stats and skill count

**Issues:**
- Total says 213 features; sum of categories = 305. Recompute and fix total and all derived stats (priority breakdown, effort distribution).
- SK-002 says "500+ skills"; should be "1,500+" (PRD, Vision, OKRs).

**Action:**
- [x] Re-sum categories; set correct total (305 or actual).
- [x] Update all summary stats/tables that depend on total.
- [x] Replace "500+ skills" with "1,500+ skills" (SK-002 and any other).

---

## 5. NORTH_STAR.md + SESSION_LOG.md — current state

**Issues:**
- NORTH_STAR says "What's Built: Nothing yet" but forge-core, forge-agent, forge-db exist with tests.
- SESSION_LOG missing at least 2 sessions.

**Action:**
- [x] NORTH_STAR: Update "What's Built" to list Phase 0 progress (forge-core, forge-agent, forge-db in forge-project or claude-forge; frontend; scaffold; 4-agent delivery).
- [x] SESSION_LOG: Append missing sessions (scaffold, Agent A/B/C/D summaries, audit).

---

## 6. Rust code + workspace (reference crates in forge-project)

**Execution order:** Follow [PHASE0_REMAINING.md](PHASE0_REMAINING.md): (1) foundational fixes first, (2) then 5 crates, (3) then frontend. The table below aligns with step 1.

**If** the code under `forge-project/crates/` is the reference to fix (not claude-forge):

| # | Severity | File | Fix |
|---|----------|------|-----|
| 1 | Critical | Root Cargo.toml | Add `[workspace.dependencies]` with shared deps; use in crates. |
| 2 | Critical | Root Cargo.toml | Set `members = ["crates/*"]` (or list all 8 crates when present). |
| 3 | Major | rusqlite (workspace or forge-db) | Add features `bundled`, `vtab`, `fts5`. |
| 4 | Major | batch_writer.rs | Persist event's own timestamp; do not use `Utc::now()` at flush. |
| 5 | Major | agents.rs | Serialize preset via stable format (e.g. serde or fixed string), not Debug. |
| 6 | Major | agents.rs | Use proper rusqlite error for UUID parse failure (not InvalidParameterName). |
| 7 | Major | batch_writer.rs | Use `transaction()` instead of `unchecked_transaction()` unless explicitly required. |
| 8 | Minor | validation.rs | Add character-set validation for agent names (e.g. alphanumeric, hyphen, underscore). |
| 9 | Minor | forge-agent lib.rs | Re-export `validate_update_agent` if it exists. |
| 10 | Minor | batch_writer / AppState | Document or resolve: `shutdown(self)` vs `Arc<BatchWriter>` (e.g. shutdown on drop or explicit API). |

- [ ] Cargo.toml workspace.dependencies + members
- [ ] rusqlite features
- [ ] batch_writer: event timestamp; transaction()
- [ ] agents.rs: preset serialization; UUID error
- [ ] validation + lib.rs exports
- [ ] BatchWriter shutdown vs Arc

---

## 7. Architecture doc conflicts (acknowledge or fix)

**Resolution table** — update the losing doc or add a short "Canonical: X" note:

| Conflict | Canonical (Phase 0 / current) | Update other doc |
|----------|------------------------------|-------------------|
| API prefix | /api/v1/agents | API_DESIGN: note v1 or change to v1 |
| WebSocket path | /api/v1/ws | API_DESIGN: note or change |
| Crate count | 8 initial, ~14 full | SYSTEM_ARCHITECTURE: note "8 initially" |
| FTS5 tables | 3 (skills_fts, sessions_fts, events_fts) | DATA_MODEL: align with 3 |
| Agent preset names | CodeWriter, Reviewer, Tester... | API_DESIGN: use Phase 0 names |
| Theme | Dark only for Phase 0 | QUALITY_GATES or CODING_STANDARDS: pick one |
| Rust edition | 1.85+ for 2024 | PRD: fix 1.75+ → 1.85+ |

- [x] API_DESIGN: v1 prefix and /api/v1/ws
- [x] DATA_MODEL: FTS5 = 3 tables; agent schema = denormalized columns (Phase 0 plan)
- [x] EVENT_SYSTEM: Note "Canonical event type: ForgeEvent (typed enum); see PHASE0_IMPLEMENTATION_PLAN"
- [x] PRD: Rust 1.85+
- [x] Theme: one line in QUALITY_GATES or CODING_STANDARDS

---

## 8. DATA_MODEL + EVENT_SYSTEM (canonical source of truth)

**Audit:** DATA_MODEL has agents as single config TEXT; Phase 0 uses denormalized columns. EVENT_SYSTEM has TaggedEvent; Phase 0 uses ForgeEvent enum.

**Action:** In DATA_MODEL and EVENT_SYSTEM, add a short "Phase 0 / current implementation" note that points to the Phase 0 plan (denormalized agents, ForgeEvent enum) as the implemented standard. Optionally add a "Legacy / alternative" subsection for the old shape so readers know the difference.

- [x] DATA_MODEL: note Phase 0 agent table shape
- [x] EVENT_SYSTEM: note ForgeEvent is canonical

---

## Completion

When all sections are checked:

- [ ] Run `cargo build --workspace` and `cargo test --workspace` (if Rust fixes applied)
- [x] Grep for "61 repo" again to confirm no stray 61s
- [ ] Commit with message: `docs: audit remediation — 62 repos, remove redundant docs, align SPRINT/MILESTONES/FEATURE_CATALOG, update NORTH_STAR and SESSION_LOG, architecture notes; Rust workspace/code fixes pending`

**Done (docs):** 1–5, 7, 8. **Pending:** 6 (Rust code + workspace), final commit.

---

*Generated from the consolidated 5-agent audit report. Start with section 1; proceed in order.*
