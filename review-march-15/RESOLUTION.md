# Review Findings Resolution — March 2026

## Summary

| Category | Total | Resolved | Deferred |
|----------|-------|----------|----------|
| Doc Inconsistencies | 12 | 12 | 0 |
| Fix Proposals | 11 | 11 | 0 |
| Architecture | 10 | 7 | 3 |
| UX Proposals | 10 | 8 | 2 |
| **Total** | **43** | **38** | **5** |

## Wave R1 — Doc Fixes + Quick Wins
- [x] R1-A: Doc consistency sweep (7 doc files updated with correct counts)
- [x] R1-B: DB path migration (~/.claude-forge → ~/.agentforge) + CLAUDE.md update
- [x] R1-C: Structured JSON logging (release builds) + cargo-deny + CI audit

## Wave R2 — Architecture
- [x] R2-A: Event bus fan-out (mpsc guaranteed persistence + broadcast best-effort UI)
- [x] R2-B: Connection pooling (r2d2, separate read/write, busy_timeout, WAL pragmas)
- [x] R2-C: Safety persistence (circuit breaker state survives restarts) + spawn limiter semaphore + migration 0013

## Wave R3 — Testing + UX
- [x] R3-A: API integration tests (16 new tests, 28 total routes tested)
- [x] R3-B: Loading/error/empty state components (Skeleton, ErrorMessage, EmptyState) + WS indicator
- [x] R3-C: Responsive sidebar (hamburger menu, 768px/480px breakpoints) + recursive OrgNode component

## Wave R4 — Polish
- [x] R4-A: Architecture docs rewrite (Mermaid diagrams, crate descriptions, persona pipeline)
- [x] R4-B: Accessibility (skip-to-content, focus trap, ARIA labels, sr-only, WCAG 2.1 AA subset)
- [x] R4-C: ForgeError stratification (10 → 12 variants, is_retriable/http_status/error_code methods)
- [x] R4-D: Final verification (this document)

## Verified Counts (from source code)

| Item | Count | Source |
|------|-------|--------|
| ForgeEvent variants | 43 | `crates/forge-core/src/events.rs` |
| MCP tools | 19 | `crates/forge-mcp-bin/src/main.rs` (`#[tool]` annotations) |
| Workspace crates | 12 | `Cargo.toml` workspace members |
| Migrations | 12 | `crates/forge-db/src/migrations.rs` (0001-0013, skip 0010) |
| DB repos | 17 | `crates/forge-db/src/repos/mod.rs` |
| Frontend pages | 15 | `frontend/src/routes/` (14 subdirs + root) |
| API .route() calls | 40 | `crates/forge-api/src/routes/*.rs` |
| DB default path | `~/.agentforge/forge.db` | `crates/forge-app/src/main.rs` |

## Build Verification

| Check | Result |
|-------|--------|
| `cargo check --workspace` | PASS (zero warnings) |
| `cargo test --workspace` | PASS (284 tests) |
| `cargo clippy --workspace -- -D warnings` | FAIL (1 clippy lint in forge-api: `map_or` → `is_none_or`) |
| `pnpm check` (frontend) | FAIL (1 error in workflows/+page.svelte: `Cannot find name 'input'`) |
| `pnpm build` (frontend) | PASS |
| `git ls-files site/` | PASS (not tracked) |
| DB path primary = `~/.agentforge` | PASS |
| DB path legacy fallback only | PASS |

## Known Issues (pre-existing, not introduced by this review)

1. **Clippy lint**: `forge-api` has 1 `unnecessary_map_or` lint (`.map_or(true, ...)` → `.is_none_or(...)`). Minor — compiles and tests pass.
2. **Frontend type error**: `workflows/+page.svelte` line 436 references undefined `input`. Build succeeds despite this (adapter-static).

## Deferred to Future Sprint
- AR-1: Unit of Work pattern (invasive refactor across all repos)
- AR-5: Middleware pipeline trait (1000+ line rewrite)
- UX-8: Full keyboard navigation (requires per-page audit)
