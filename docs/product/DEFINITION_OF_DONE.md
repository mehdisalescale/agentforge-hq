# Definition of Done (DoD)

> **Every work item must satisfy ALL applicable criteria before it can be marked "Done".**
>
> This is non-negotiable. No exceptions without explicit Product Owner approval.

---

## DoD for User Stories

### Code Quality
- [ ] Code compiles with `cargo check` — zero warnings
- [ ] `cargo clippy` passes — zero warnings
- [ ] `#![forbid(unsafe_code)]` maintained in all crates
- [ ] No `unwrap()` or `expect()` in production code (only in tests)
- [ ] All public types have doc comments (`///`)
- [ ] No TODO/FIXME without linked issue number

### Testing
- [ ] Unit tests written BEFORE implementation (TDD red-green-refactor)
- [ ] All new public functions have at least one unit test
- [ ] All new API routes have at least one integration test
- [ ] All new middleware has isolation test + chain integration test
- [ ] Edge cases covered: empty input, boundary values, error paths
- [ ] `cargo test --workspace` passes — all tests green
- [ ] Test names follow pattern: `test_{function}_{scenario}_{expected_result}`

### Database
- [ ] New tables have migration SQL file (sequential number)
- [ ] Migration is idempotent (can run twice safely)
- [ ] All queries go through Repository pattern (no raw SQL in handlers)
- [ ] Foreign keys have ON DELETE behavior specified
- [ ] Indexes added for columns used in WHERE/JOIN clauses

### API
- [ ] New routes documented in OpenAPI spec (utoipa annotations)
- [ ] Request/response types have `#[derive(Serialize, Deserialize, ToSchema)]`
- [ ] Error responses use `ForgeError` → HTTP status code mapping
- [ ] Content-Type headers set correctly
- [ ] CORS behavior verified

### Events
- [ ] State changes emit appropriate `ForgeEvent` variant
- [ ] New event variants added to `ForgeEvent` enum
- [ ] Event serialization test added
- [ ] BatchWriter persists new events correctly
- [ ] WebSocket subscribers receive new events

### Frontend
- [ ] New page uses Svelte 5 runes (`$state`, `$derived`, `$effect`)
- [ ] Loading states shown (skeleton/spinner)
- [ ] Error states handled and displayed
- [ ] Empty states shown with helpful message
- [ ] Responsive layout (works on 768px+ width)
- [ ] Keyboard accessible (tab navigation works)
- [ ] No console errors in browser

### Security
- [ ] No secrets in source code
- [ ] Input validation on all user-provided data
- [ ] SQL injection prevented (parameterized queries via rusqlite)
- [ ] XSS prevented (DOMPurify on rendered markdown)
- [ ] Rate limiting applies to new endpoints

### Documentation
- [ ] CLAUDE.md updated if new crates/env vars added
- [ ] CHANGELOG entry written
- [ ] New ADR if architectural decision was made

---

## DoD for Epics

All Story-level DoD criteria PLUS:

- [ ] All child stories completed and verified
- [ ] Integration test covering the full epic workflow (happy path)
- [ ] E2E smoke test script updated (`scripts/e2e-smoke.sh`)
- [ ] Performance benchmark run (startup time, memory, API latency)
- [ ] Product Owner demo completed and accepted
- [ ] NORTH_STAR.md updated with new capabilities

---

## DoD for Releases

All Epic-level DoD criteria PLUS:

- [ ] Full regression: `cargo test --workspace` (all tests)
- [ ] `cargo audit` — no known vulnerabilities
- [ ] Binary builds for all platforms (macOS ARM64, macOS x64, Linux x64)
- [ ] GitHub Release created with changelog
- [ ] README.md updated with new features
- [ ] OpenAPI spec regenerated and verified at `/docs`
- [ ] Docker image built and tested (if applicable)
- [ ] Migration path documented (v(N-1) → vN)

---

## DoD Enforcement

1. **PR Review**: Reviewer checks DoD checklist before approving
2. **CI Pipeline**: Automated checks for compile, test, clippy, audit
3. **Sprint Review**: PO verifies acceptance criteria against DoD
4. **Release Gate**: Release manager runs full DoD checklist before tag
