# Development Process

> How we work on Claude Forge: workflow, cadence, ceremonies, quality, and communication.

---

## Development Workflow

### Branch Strategy

Claude Forge uses a trunk-based development model with short-lived feature branches.

```
main (always deployable)
 |
 +-- feat/agent-circuit-breaker     (feature: new capability)
 +-- fix/websocket-reconnect        (bugfix)
 +-- refactor/session-store-trait    (refactoring)
 +-- docs/mcp-tool-reference        (documentation)
 +-- absorb/ralph-circuit-breaker   (reference repo absorption)
```

**Branch naming conventions:**

| Prefix | Purpose | Example |
|--------|---------|---------|
| `feat/` | New feature or capability | `feat/workflow-dag-editor` |
| `fix/` | Bug fix | `fix/sqlite-wal-checkpoint` |
| `refactor/` | Code restructuring, no behavior change | `refactor/event-store-api` |
| `docs/` | Documentation only | `docs/mcp-tools-reference` |
| `absorb/` | Absorbing patterns from a reference repo | `absorb/hooks-observability-swimlane` |
| `perf/` | Performance improvement | `perf/fts5-query-optimization` |
| `test/` | Test-only changes | `test/workflow-engine-integration` |

### Workflow Steps

1. **Pick work item** -- Select from sprint backlog (prioritized by sprint planning)
2. **Create branch** -- Branch from `main` with appropriate prefix
3. **Implement** -- Write code, tests, and documentation together (not sequentially)
4. **Self-review** -- Review your own diff before requesting review
5. **Open PR** -- Write a description that explains the WHY, not just the WHAT
6. **Pass quality gates** -- All 7 gates must pass (see QUALITY_GATES.md)
7. **Code review** -- At least one approving review required
8. **Merge** -- Squash-merge to main; delete branch
9. **Verify** -- Confirm the change works on main after merge

### Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `perf`, `build`, `ci`, `chore`

Scopes: `agent`, `session`, `workflow`, `observe`, `skill`, `git`, `mcp`, `ui`, `api`, `db`, `config`, `safety`

Examples:
```
feat(workflow): add DAG execution engine with topological sort
fix(session): prevent duplicate session IDs on rapid reconnect
absorb(observe): port swim-lane visualization from hooks-observability
refactor(agent): extract AgentRunner trait for testability
docs(mcp): document all 50+ MCP tool schemas
```

---

## Sprint Cadence

### 2-Week Sprints

| Day | Activity |
|-----|----------|
| **Sprint Day 1 (Monday)** | Sprint Planning (morning), begin implementation |
| **Days 2-4** | Implementation |
| **Day 5 (Friday)** | Mid-sprint check-in |
| **Days 6-8** | Implementation continues |
| **Day 9** | Feature freeze, focus on tests and docs |
| **Day 10 (Friday)** | Sprint Review + Retrospective, release candidate |

### Sprint Ceremonies

#### Sprint Planning (Day 1, 2 hours max)

**Purpose**: Decide what to build this sprint.

**Inputs**:
- Product backlog (prioritized features and absorption tasks)
- Previous sprint velocity (story points completed)
- Absorption pipeline status (which repos are in which phase)
- Bug and tech debt backlog

**Process**:
1. Review and adjust priorities based on current state
2. Pull items from backlog into sprint, respecting velocity
3. Break large items into tasks small enough for 1-2 days
4. Identify dependencies and blockers
5. Assign owners (or leave unassigned for self-selection)

**Output**: Sprint backlog with estimated story points per item.

**Estimation scale**: Fibonacci (1, 2, 3, 5, 8, 13). Anything above 8 must be split.

#### Daily Standup (async)

**Format**: Async written update posted by end of day in the project channel.

```
Done: [what I completed today]
Next: [what I plan to work on tomorrow]
Blocked: [anything preventing progress] (or "None")
```

**Rules**:
- No meetings. Written updates only.
- If you are blocked, tag the person who can unblock you.
- If "Blocked" is not "None" for two consecutive days, escalate.

#### Mid-Sprint Check-in (Day 5, 30 minutes)

**Purpose**: Catch scope creep and blockers before they derail the sprint.

**Agenda**:
1. Are we on track to complete the sprint backlog? (5 min)
2. Any items at risk? Can we cut scope? (10 min)
3. Any new blockers? (5 min)
4. Any discoveries that change priorities? (10 min)

#### Sprint Review (Day 10, 1 hour)

**Purpose**: Demonstrate what was built.

**Format**:
1. Each completed feature is demonstrated (working software, not slides)
2. Absorption progress reviewed (which repos advanced through pipeline)
3. Metrics reviewed (velocity, bug count, test coverage)
4. Stakeholder feedback captured

#### Sprint Retrospective (Day 10, 30 minutes)

**Purpose**: Improve how we work.

**Format**: Start/Stop/Continue
- **Start**: What should we begin doing?
- **Stop**: What is not working and should be dropped?
- **Continue**: What is working well?

**Output**: 1-3 specific action items assigned to specific people with due dates.

---

## Definition of Done

### Per Feature

A feature is DONE when ALL of the following are true:

- [ ] Code is implemented and compiles without warnings
- [ ] Unit tests written and passing (minimum 80% coverage for new code)
- [ ] Integration tests written for API endpoints
- [ ] API endpoint documented (if applicable)
- [ ] MCP tool documented (if applicable)
- [ ] UI component renders correctly in both dark and light themes
- [ ] Keyboard navigation works (if UI feature)
- [ ] No regressions in existing tests
- [ ] Code review approved
- [ ] Documentation updated (user-facing docs if behavior changed)
- [ ] ADR written (if architectural decision was made)
- [ ] Changelog entry added
- [ ] Performance verified (no regression beyond 10% on hot paths)

### Per Absorption Task

An absorption task is DONE when ALL of the following are true:

- [ ] All 5 pipeline phases completed (Analyze, Extract, Design, Implement, Validate)
- [ ] Reference behavior matched (or deviation documented with rationale)
- [ ] All three interfaces work: HTTP API, MCP tool, CLI (where applicable)
- [ ] Tests pass
- [ ] Feature source map updated
- [ ] Documentation updated

### Per Sprint

A sprint is DONE when:

- [ ] All items marked "Done" meet the per-feature definition
- [ ] Sprint review conducted with demo
- [ ] Sprint retrospective completed with action items
- [ ] Release candidate tagged (if release sprint)
- [ ] Velocity calculated and recorded

### Per Milestone

A milestone is DONE when:

- [ ] All sprint deliverables for the milestone are complete
- [ ] End-to-end integration test suite passes
- [ ] Performance benchmarks met
- [ ] Security audit completed (for safety-related milestones)
- [ ] User documentation published
- [ ] Binary published to release channel

---

## Code Review Process

### Who Reviews

- All PRs require at least one approving review
- PRs touching safety code (circuit breaker, permissions, rate limiting) require two reviews
- PRs touching database schema require review from someone who has worked on the DB layer
- Architecture-changing PRs require an ADR and broader review

### What Reviewers Check

1. **Correctness**: Does the code do what the PR description says?
2. **Safety**: Could this introduce a vulnerability, data loss, or runaway agent?
3. **Tests**: Are the tests meaningful (not just coverage padding)?
4. **API design**: Are the interfaces clean, consistent, and backward-compatible?
5. **Performance**: Any obvious hot loops, unnecessary allocations, or N+1 queries?
6. **Documentation**: Is the code self-documenting? Are complex parts commented?
7. **Consistency**: Does it follow established patterns in the codebase?

### Review Turnaround

- PRs should be reviewed within 24 hours of submission
- If a review is blocking, the author can escalate after 24 hours
- Nit-level feedback should be marked `nit:` so the author knows it is not blocking

### Review Etiquette

- Ask questions rather than make demands: "What if we..." not "You should..."
- Explain WHY, not just WHAT: "This could deadlock because..." not "Change this"
- Approve with nits: If the only feedback is cosmetic, approve and note the nits
- Do not block on style preferences; block on correctness, safety, and API design

---

## Release Process

### Versioning

Claude Forge follows Semantic Versioning (SemVer): `MAJOR.MINOR.PATCH`

| Component | When to bump | Example |
|-----------|-------------|---------|
| MAJOR | Breaking API change, incompatible DB migration, MCP tool removal | 1.0.0 -> 2.0.0 |
| MINOR | New feature, new MCP tool, new API endpoint, new agent preset | 1.0.0 -> 1.1.0 |
| PATCH | Bug fix, performance improvement, documentation update | 1.0.0 -> 1.0.1 |

Pre-release versions: `1.0.0-alpha.1`, `1.0.0-beta.1`, `1.0.0-rc.1`

### Release Cadence

- **Minor releases**: Every 2 sprints (monthly)
- **Patch releases**: As needed for critical bugs or security fixes
- **Major releases**: As needed for breaking changes (target: no more than 1 per quarter)

### Release Checklist

1. **Freeze**: Create release branch `release/vX.Y.Z` from main
2. **Changelog**: Generate changelog from commit messages, edit for clarity
3. **Version bump**: Update version in `Cargo.toml`, `package.json`, and any other version files
4. **Build**: Build release binaries for all targets (macOS arm64, macOS x86_64, Linux x86_64, Linux arm64)
5. **Test**: Run full test suite on release branch
6. **Tag**: Create git tag `vX.Y.Z`
7. **Publish**: Upload binaries to GitHub Releases
8. **Announce**: Post release notes
9. **Verify**: Download and test published binary on a clean machine

### Binary Publishing Targets

| Target | Triple | Notes |
|--------|--------|-------|
| macOS (Apple Silicon) | `aarch64-apple-darwin` | Primary development target |
| macOS (Intel) | `x86_64-apple-darwin` | Legacy support |
| Linux (x86_64) | `x86_64-unknown-linux-gnu` | Server and CI target |
| Linux (ARM) | `aarch64-unknown-linux-gnu` | Raspberry Pi, ARM servers |

Future: Windows targets when demand warrants.

---

## Documentation Requirements

### What Must Be Documented When

| Change Type | Required Documentation |
|-------------|----------------------|
| New API endpoint | OpenAPI spec entry, usage example |
| New MCP tool | Tool schema (JSON Schema), description, usage example |
| New CLI command | Help text, man page entry, usage example |
| New UI page | Page purpose, keyboard shortcuts, screenshot |
| New agent preset | Preset description, use cases, configuration options |
| Database schema change | Migration script, schema diagram update |
| Configuration change | Settings reference update, default value documentation |
| Architecture decision | ADR (Architecture Decision Record) |
| Bug fix | Changelog entry; update docs if the bug was in documented behavior |

### Documentation Locations

| Doc Type | Location |
|----------|----------|
| API reference | Auto-generated from code + `docs/api/` |
| MCP tool reference | `docs/mcp/` |
| User guide | `docs/guide/` |
| Architecture decisions | `forge-project/03-architecture/adr/` |
| Design documents | `forge-project/04-design/` |
| Changelog | `CHANGELOG.md` at repo root |

---

## Decision Making Process

### Architecture Decision Records (ADRs)

Any decision that affects the system architecture, public API, database schema, or MCP tool interface must be documented as an ADR.

**ADR Format**:

```markdown
# ADR-NNNN: [Title]

## Status
Proposed | Accepted | Deprecated | Superseded by ADR-MMMM

## Context
What is the situation that requires a decision?

## Decision
What is the decision we made?

## Consequences
What are the positive and negative consequences of this decision?

## Alternatives Considered
What other options were evaluated and why were they rejected?
```

**ADR Workflow**:
1. Author writes ADR as a PR
2. Team reviews and discusses (comments on the PR)
3. Decision is made (typically within 3 business days)
4. ADR is merged with status "Accepted"
5. Implementation proceeds

**When an ADR is NOT needed**:
- Bug fixes
- Performance improvements that do not change interfaces
- Documentation updates
- Test additions
- Refactoring that does not change public interfaces

---

## Communication

### Async-First

All communication defaults to asynchronous. Meetings are the exception, not the rule.

**Principles**:
1. Write it down. If it was discussed verbally, summarize it in writing.
2. Use threads. Do not bury decisions in chat streams.
3. Document decisions where they will be found. ADRs for architecture, PR descriptions for implementation choices, comments in code for non-obvious logic.
4. Do not expect instant responses. 24-hour SLA for non-urgent items, 4-hour SLA for blockers.

### Communication Channels

| Channel | Purpose | Response expectation |
|---------|---------|---------------------|
| GitHub Issues | Feature requests, bugs, planning | 48 hours |
| GitHub PRs | Code review, technical discussion | 24 hours |
| Project channel | Daily standups, quick questions | Same business day |
| ADR PRs | Architecture decisions | 3 business days |
| Emergency | Production-down, security incident | 1 hour |

### Document Decisions

Every decision should be traceable. Six months from now, someone should be able to answer "Why did we do it this way?" by finding:

- An ADR (for architecture)
- A PR description (for implementation)
- A GitHub Issue (for requirements)
- A sprint retrospective note (for process changes)

If a decision cannot be traced to a written record, it has not been made.
