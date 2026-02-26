# Quality Gates

> Seven gates that every change must pass before shipping. No exceptions.

---

## Overview

Quality gates are sequential checkpoints. A change cannot advance past a gate until that gate's criteria are met. Each gate has a clear owner, explicit criteria, and a defined escalation path.

```
  Gate 0        Gate 1        Gate 2        Gate 3        Gate 4        Gate 5        Gate 6
  DESIGN        CODE          TESTING       DOCS          PERFORMANCE   SECURITY      UX
  REVIEW        REVIEW
  ────────>   ────────>    ────────>     ────────>     ────────>     ────────>     ────────>  SHIP
```

---

## Gate 0: Design Review

**When**: Before implementation begins on any M, L, or XL feature. S features may skip this gate if they follow an established pattern.

**Purpose**: Catch architectural mistakes before code is written. It is 10x cheaper to change a design document than to refactor working code.

### Criteria

| # | Criterion | Pass Condition |
|---|-----------|----------------|
| 0.1 | Bounded context identified | Feature is assigned to exactly one bounded context |
| 0.2 | Interfaces defined | Rust trait signatures, API endpoint paths, MCP tool schemas are specified |
| 0.3 | Data model defined | New types, database tables, and relationships are documented |
| 0.4 | Event flow documented | Events emitted and consumed by this feature are listed |
| 0.5 | Dependencies identified | All required modules that must exist first are listed |
| 0.6 | Backward compatibility assessed | Breaking changes are flagged and justified |
| 0.7 | ADR written (if applicable) | Architectural decisions with alternatives are documented |

### Who Approves

- Any team member who has worked on the target bounded context
- For cross-context features: one reviewer per affected context

### What Blocks

- Missing interface definitions: cannot implement what is not defined
- Unresolved dependency conflicts: cannot build on a foundation that does not exist
- Breaking change without ADR: cannot change public interfaces without documented rationale

### Escalation

If reviewers disagree on design approach:
1. Document both approaches with pros/cons in the ADR
2. If no consensus after 3 business days, the project lead decides
3. Decision is recorded in the ADR regardless of outcome

---

## Gate 1: Code Review

**When**: Before any PR is merged to main.

**Purpose**: Catch bugs, enforce consistency, share knowledge.

### Criteria

| # | Criterion | Pass Condition |
|---|-----------|----------------|
| 1.1 | Code compiles | `cargo build` succeeds with zero warnings |
| 1.2 | Code is formatted | `cargo fmt --check` passes |
| 1.3 | Linter passes | `cargo clippy` passes with zero warnings |
| 1.4 | Frontend builds | `pnpm build` in `frontend/` succeeds |
| 1.5 | Frontend lints | `pnpm lint` passes (if configured) |
| 1.6 | PR description explains WHY | Description covers motivation, not just what changed |
| 1.7 | No hardcoded secrets | No API keys, tokens, or passwords in code |
| 1.8 | Error handling is explicit | No `unwrap()` in production code paths (only in tests and infallible cases) |
| 1.9 | Types are used correctly | `Option<Option<T>>` pattern for nullable PATCH fields, proper `Result<>` usage |
| 1.10 | Naming is consistent | Follows existing codebase conventions |

### Who Approves

- At least one team member who did not write the code
- Two reviewers required for: safety-related code, database schema changes, MCP tool changes

### What Blocks

- Compiler warnings: must be zero
- Clippy warnings: must be zero
- Failed formatting: must pass `cargo fmt`
- No tests for new logic: every non-trivial function needs at least one test
- `unwrap()` in production path: use `?` or explicit error handling

### Escalation

If a reviewer and author disagree:
1. Author responds to feedback with rationale
2. If still unresolved, a third reviewer breaks the tie
3. For style disagreements: defer to existing codebase patterns; if no precedent, defer to Rust conventions

---

## Gate 2: Testing

**When**: Before PR merge. Automated via CI.

**Purpose**: Verify correctness at unit, integration, and API levels.

### Criteria

| # | Criterion | Pass Condition |
|---|-----------|----------------|
| 2.1 | Unit tests pass | `cargo test` -- all tests green |
| 2.2 | Integration tests pass | `cargo test --test '*'` -- all tests green |
| 2.3 | New code has tests | Minimum 80% line coverage on new code |
| 2.4 | No test regressions | No previously passing tests now fail |
| 2.5 | API endpoint tests | Every new endpoint has at least: one happy path test, one error case test |
| 2.6 | MCP tool tests | Every new MCP tool has at least: one invocation test, one validation test |
| 2.7 | Edge cases covered | Null inputs, empty strings, concurrent access, large payloads |
| 2.8 | Frontend type-checks | TypeScript/Svelte type checking passes |

### Test Categories

| Category | Location | What It Tests | Run Time Target |
|----------|----------|---------------|-----------------|
| Unit | `src/**/mod.rs` (`#[cfg(test)]`) | Individual functions and types | < 30 seconds total |
| Integration | `tests/` | API endpoints, database operations, MCP tools | < 2 minutes total |
| End-to-End | `e2e/` (future) | Full user workflows through the UI | < 5 minutes total |

### Who Approves

- Automated: CI pipeline must pass
- Manual: reviewer verifies test quality (not just quantity)

### What Blocks

- Any test failure: zero tolerance for failing tests
- Missing tests for new public functions
- Flaky tests: tests that pass/fail non-deterministically must be fixed or quarantined

### Escalation

- Flaky test detected: create a bug ticket immediately, quarantine the test (mark `#[ignore]` with a tracking issue link), fix within the current sprint
- Coverage drops below 70% overall: sprint planning must include a "test debt" task

---

## Gate 3: Documentation

**When**: Before PR merge for any user-facing change.

**Purpose**: Ensure every feature is documented for all three audiences: API consumers, MCP clients, and UI users.

### Criteria

| # | Criterion | Pass Condition |
|---|-----------|----------------|
| 3.1 | API documented | New/changed endpoints have OpenAPI-style documentation |
| 3.2 | MCP tools documented | New/changed tools have JSON Schema + description + example |
| 3.3 | User-facing docs updated | If UI behavior changed, user guide reflects it |
| 3.4 | Inline code docs | Public functions and traits have `///` doc comments |
| 3.5 | Changelog entry | `CHANGELOG.md` updated with user-facing description |
| 3.6 | Configuration documented | New settings, env vars, or config options are in the reference |
| 3.7 | ADR exists (if applicable) | Architecture decisions recorded |
| 3.8 | Error messages are helpful | Error responses include what went wrong and how to fix it |

### Documentation Standards

**Rust doc comments**:
```rust
/// Create a new agent with the given configuration.
///
/// # Errors
///
/// Returns `AgentError::DuplicateId` if an agent with this ID already exists.
/// Returns `AgentError::InvalidConfig` if the configuration is malformed.
pub fn create_agent(&self, config: AgentConfig) -> Result<Agent, AgentError> {
```

**API documentation**:
```
POST /api/agents
Content-Type: application/json

Request:  { "name": "reviewer", "model": "sonnet", "system_prompt": "..." }
Response: 201 Created { "id": "abc123", "name": "reviewer", ... }
Error:    409 Conflict { "error": "agent_exists", "message": "Agent 'reviewer' already exists" }
```

**MCP tool documentation**:
```
Tool: forge_create_agent
Description: Create a new agent with the specified configuration
Input: { name: string (required), model: string, system_prompt: string }
Output: { id: string, name: string, status: string }
Example: forge_create_agent({ name: "reviewer", model: "sonnet" }) -> { id: "abc123", ... }
```

### Who Approves

- Code reviewer also reviews documentation (same PR)
- For major features: someone unfamiliar with the feature reads the docs and confirms they are understandable

### What Blocks

- Missing changelog entry for user-facing change
- Missing doc comments on public API
- Error messages that say "internal error" without context

### Escalation

- Documentation debt accumulating: allocate 10% of each sprint to documentation tasks
- Major feature shipped without docs: the person who merged it is responsible for writing docs before the next sprint review

---

## Gate 4: Performance

**When**: Before PR merge for any change touching hot paths (request handling, event processing, database queries, WebSocket streaming).

**Purpose**: Prevent performance regressions.

### Criteria

| # | Criterion | Pass Condition |
|---|-----------|----------------|
| 4.1 | No regression | Benchmark results within 10% of baseline |
| 4.2 | Memory bounded | No unbounded growth (leaks) in long-running tests |
| 4.3 | Database queries efficient | No N+1 queries; explain plans reviewed for new queries |
| 4.4 | WebSocket latency | Event delivery < 50ms from emission to UI receipt |
| 4.5 | Startup time | Binary starts and serves first request in < 2 seconds |
| 4.6 | Binary size | Release binary < 50MB (including embedded frontend) |
| 4.7 | SQLite performance | Write batching: 50 events or 2 seconds (whichever first) |

### Performance Budget

| Operation | Budget | Measurement |
|-----------|--------|-------------|
| API endpoint (simple read) | < 5ms p99 | Time from request receipt to response send |
| API endpoint (database query) | < 20ms p99 | Time from request receipt to response send |
| API endpoint (FTS5 search) | < 100ms p99 | Time for full-text search with 10K sessions |
| WebSocket event broadcast | < 10ms p99 | Time from event emit to all subscribers notified |
| Agent spawn | < 500ms | Time from spawn request to process running |
| Frontend initial load | < 1.5 seconds | Time to interactive on localhost |
| SQLite batch write | < 50ms | Time to flush a batch of 50 events |

### Who Approves

- Automated: CI benchmarks (when available)
- Manual: reviewer checks for obvious performance issues (N+1 queries, unnecessary allocations, blocking in async context)

### What Blocks

- Regression > 10% on any benchmarked operation
- Unbounded memory growth detected in stress test
- Blocking call in async context (`std::thread::sleep` in `tokio` task)
- Missing `EXPLAIN QUERY PLAN` for complex SQL queries

### Escalation

- If a feature inherently requires more than the budget (e.g., a complex DAG computation), document the expected performance characteristics and adjust the budget with an ADR

---

## Gate 5: Security

**When**: Before PR merge for any change touching: permissions, authentication, file access, process spawning, network requests, user input handling, or database queries.

**Purpose**: Prevent vulnerabilities in a tool that has access to users' codebases.

### Criteria

| # | Criterion | Pass Condition |
|---|-----------|----------------|
| 5.1 | No new vulnerabilities | `cargo audit` passes with zero advisories |
| 5.2 | Input validated | All user inputs (HTTP, MCP, CLI) are validated before use |
| 5.3 | Path traversal prevented | File paths are canonicalized and checked against allowed roots |
| 5.4 | SQL injection prevented | All queries use parameterized statements (never string concatenation) |
| 5.5 | Process spawning safe | Agent commands are not constructed from unvalidated user input |
| 5.6 | Secrets not logged | API keys, tokens, and passwords are never written to logs or events |
| 5.7 | Audit trail maintained | Security-relevant actions are logged with actor, action, and timestamp |
| 5.8 | CORS configured | WebSocket and HTTP endpoints have appropriate origin restrictions |
| 5.9 | Rate limiting in place | Public-facing endpoints have rate limits |
| 5.10 | Dependency audit | No new dependencies with known CVEs |

### Security-Sensitive Areas

| Area | Risk | Mitigation |
|------|------|------------|
| Agent process spawning | Arbitrary code execution | Validate command, sanitize environment, use permission model |
| File system access | Path traversal, data exfiltration | Canonicalize paths, restrict to project root |
| WebSocket connections | Unauthorized event access | Validate origin, authenticate connections |
| MCP tool invocation | Unintended tool execution | Validate tool names, enforce permission model |
| Database queries | SQL injection | Parameterized queries only |
| Configuration loading | Code injection via config | Parse configs as data, never eval |
| API key handling | Credential exposure | Never log, never embed in events, env_remove pattern |

### Who Approves

- Any team member for routine changes
- Two reviewers for changes to: permission model, process spawning, file access, authentication

### What Blocks

- `cargo audit` finding any advisory
- String-concatenated SQL query
- User input used directly in process spawn command
- API key or credential in log output
- Missing input validation on any endpoint

### Escalation

- Critical vulnerability found in dependency: patch or replace within 24 hours
- Security issue found in production: follow incident response process, patch and release within 48 hours

---

## Gate 6: UX

**When**: Before PR merge for any change that adds or modifies UI components.

**Purpose**: Maintain a consistent, accessible, keyboard-friendly interface.

### Criteria

| # | Criterion | Pass Condition |
|---|-----------|----------------|
| 6.1 | Dark theme works | Component renders correctly in dark mode |
| 6.2 | Light theme works | Component renders correctly in light mode |
| 6.3 | Keyboard navigable | All interactive elements are reachable via keyboard |
| 6.4 | Focus visible | Focused elements have visible focus indicators |
| 6.5 | Responsive layout | Component works at 1024px, 1440px, and 1920px widths |
| 6.6 | Loading states | Async operations show loading indicators |
| 6.7 | Error states | Failed operations show user-friendly error messages |
| 6.8 | Empty states | Empty lists/tables show helpful messages (not blank space) |
| 6.9 | Consistent styling | Uses existing TailwindCSS patterns, not ad-hoc styles |
| 6.10 | No layout shift | Content does not jump around as it loads |

### Accessibility Baseline

Claude Forge does not aim for WCAG AAA compliance, but maintains a practical accessibility baseline:

| Standard | Requirement |
|----------|-------------|
| Contrast | Text meets WCAG AA contrast ratio (4.5:1 for normal text, 3:1 for large) |
| Keyboard | All functionality accessible via keyboard |
| Focus | Focus order follows visual layout |
| Labels | Form inputs have associated labels |
| Alt text | Informational images have alt text |
| Screen reader | Key navigation landmarks have ARIA labels |
| Motion | Animations respect `prefers-reduced-motion` |

### Who Approves

- Code reviewer checks against the criteria above
- For major UI changes: test on at least two screen sizes

### What Blocks

- Component unusable without mouse (no keyboard support)
- Component broken in dark or light theme
- Missing loading or error states for async operations
- Text unreadable due to contrast issues

### Escalation

- UX disagreements: defer to the design principles in UI_DESIGN.md
- If no relevant principle exists, the person closest to the user's perspective decides
- Create a new design principle to prevent the same disagreement from recurring

---

## Gate Summary Matrix

| Gate | Automated? | Reviewer Count | Blocks Merge? | Typical Time |
|------|-----------|----------------|---------------|--------------|
| Gate 0: Design Review | No | 1-2 | Yes (M/L/XL only) | 1-3 days |
| Gate 1: Code Review | Partially (lint/fmt) | 1-2 | Yes | 1-2 days |
| Gate 2: Testing | Yes (CI) | 0 (automated) | Yes | < 10 minutes |
| Gate 3: Documentation | No | 1 | Yes (user-facing) | Same as code review |
| Gate 4: Performance | Partially (benchmarks) | 1 (hot paths only) | Yes (hot paths) | < 10 minutes |
| Gate 5: Security | Partially (cargo audit) | 1-2 | Yes | Same as code review |
| Gate 6: UX | No | 1 | Yes (UI changes) | Same as code review |

---

## Continuous Improvement

At each sprint retrospective, review:
1. Were any gates too slow? (bottleneck analysis)
2. Were any gates bypassed? (process compliance)
3. Did any shipped bug slip through a gate? (gate effectiveness)
4. Can any manual gate be automated? (automation opportunities)

Adjust gate criteria based on findings. Gates are living documents, not bureaucratic fixtures.
