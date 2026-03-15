# Agent R4-D: Final Verification + CLAUDE.md Update

> Final pass: verify all numbers match, update CLAUDE.md, create resolution document.

**Run this LAST — after all other agents (R4-A, R4-B, R4-C) are complete.**

## Step 1: Read and Verify Counts

Read actual source code and verify these numbers:

1. **ForgeEvent variants:** Count in `crates/forge-core/src/events.rs`
2. **MCP tools:** Count `#[tool]` in `crates/forge-mcp-bin/src/main.rs`
3. **Middleware stages:** Count in `crates/forge-api/src/routes/run.rs` chain construction
4. **Workspace crates:** Count `[package]` entries in `Cargo.toml` workspace members
5. **Frontend pages:** Count directories in `frontend/src/routes/`
6. **API routes:** Count `.route()` calls across `crates/forge-api/src/routes/*.rs`
7. **Repos:** Count in `crates/forge-app/src/main.rs` (Arc<XyzRepo> instances)
8. **Migrations:** Count in `crates/forge-db/src/migrations.rs`
9. **DB default path:** Verify in `crates/forge-app/src/main.rs` `default_db_path()`

## Step 2: Update CLAUDE.md

Read current `CLAUDE.md` and update any stale information:

- ForgeEvent variant count
- MCP tool count and `forge-mcp-bin` description
- Workspace crate count and descriptions (especially forge-core, forge-db, forge-safety, forge-process if changed)
- DB default path
- Any new env vars added (FORGE_MAX_CONCURRENT)
- Safety section: mention circuit breaker persistence, spawn limiter
- EventBus description: mention fan-out (mpsc + broadcast)

## Step 3: Update NORTH_STAR.md (if it exists)

Check if `NORTH_STAR.md` has stale numbers and update.

## Step 4: Verify Build

```bash
cargo check --workspace 2>&1 | head -20    # zero warnings
cargo test --workspace 2>&1 | tail -20      # all pass
cargo clippy --workspace -- -D warnings 2>&1 | tail -10  # zero warnings

cd frontend && pnpm check 2>&1 | tail -5   # type check
cd frontend && pnpm build 2>&1 | tail -5   # builds
```

## Step 5: Verify site/ not tracked

```bash
git ls-files site/ | head -5   # should return empty
```

## Step 6: Verify DB path

```bash
grep -n "claude-forge" crates/forge-app/src/main.rs   # should only appear in legacy fallback
grep -n "agentforge" crates/forge-app/src/main.rs      # should be the primary path
```

## Step 7: Create Resolution Document

Create `review-march-15/RESOLUTION.md`:

```markdown
# Review Findings Resolution — March 2026

## Summary

| Category | Total | Resolved | Deferred |
|----------|-------|----------|----------|
| Doc Inconsistencies | 12 | 12 | 0 |
| Fix Proposals | 11 | 10 | 1 (FP-11 already done) |
| Architecture | 10 | 7 | 3 (AR-1, AR-5, deferred) |
| UX Proposals | 10 | 8 | 2 (UX-8 keyboard nav, deferred) |
| **Total** | **43** | **37** | **6** |

## Wave R1 — Doc Fixes + Quick Wins
- [x] R1-A: Doc consistency sweep (12 issues fixed)
- [x] R1-B: DB path fix + git cleanup + CLAUDE.md
- [x] R1-C: Structured logging + cargo-deny

## Wave R2 — Architecture
- [x] R2-A: Event bus fan-out (mpsc + broadcast)
- [x] R2-B: Connection pooling (r2d2, busy_timeout)
- [x] R2-C: Safety persistence + spawn limits

## Wave R3 — Testing + UX
- [x] R3-A: API integration tests
- [x] R3-B: Loading/error/empty states + WS indicator
- [x] R3-C: Responsive sidebar + recursive org chart

## Wave R4 — Polish
- [x] R4-A: Architecture diagram + docs
- [x] R4-B: Accessibility (WCAG 2.1 AA subset)
- [x] R4-C: ForgeError stratification
- [x] R4-D: Final verification

## Deferred to Future Sprint
- AR-1: Unit of Work pattern (invasive refactor)
- AR-5: Middleware pipeline trait (1000+ line rewrite)
- UX-8: Full keyboard navigation
```

Update the resolution status based on what actually got done (check for REPORT_RXX.md files).

## Rules

- Read ALL report files in `docs/agents/REPORT_R*.md` to verify what was actually completed
- Update CLAUDE.md with verified numbers from source code
- Create `review-march-15/RESOLUTION.md`
- Run full verification suite
- Do NOT change any code — verification only (except CLAUDE.md and RESOLUTION.md)

## Report

When done, create `docs/agents/REPORT_R4D.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
VERIFIED_COUNTS:
  ForgeEvent: [N]
  MCP tools: [N]
  Middleware: [N]
  Repos: [N]
  Frontend pages: [N]
CARGO_CHECK: pass/fail
CARGO_TEST: pass/fail (total test count)
CARGO_CLIPPY: pass/fail
FRONTEND_CHECK: pass/fail
FRONTEND_BUILD: pass/fail
CLAUDE_MD_UPDATED: yes/no
RESOLUTION_CREATED: yes/no
```
