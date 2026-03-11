# AgentForge — Product Documentation

> **Complete product planning for expanding Claude Forge (v0.6.0) into AgentForge (v1.0.0).**
>
> Absorbing capabilities from 8 external repos into a single Rust+Svelte AI workforce platform.

---

## Document Index

### Strategy & Vision
| Document | Purpose |
|----------|---------|
| [Product Vision](./PRODUCT_VISION.md) | Vision statement, personas, competitive landscape, domain model, NFRs, OKRs |
| [Definition of Done](./DEFINITION_OF_DONE.md) | Quality gates for stories, epics, and releases |

### Epics & User Stories
| Document | Stories | Points | Sprints |
|----------|---------|--------|---------|
| [Epic Index](./epics/EPIC_INDEX.md) | Overview & dependencies | — | — |
| [E1: Persona Catalog](./epics/E1_PERSONA_CATALOG.md) | 8 stories | 23 | S1 |
| [E2: Dev Methodology](./epics/E2_DEV_METHODOLOGY.md) | 9 stories | 31 | S1-S2 |
| [E3: Hexagonal Backends](./epics/E3_HEXAGONAL_BACKENDS.md) | 6 stories | 21 | S2 |
| [E4: Org Structure](./epics/E4_ORG_STRUCTURE.md) | 12 stories | 48 | S3-S4 |
| [E5: Multi-Backend](./epics/E5_MULTI_BACKEND.md) | 10 stories | 37 | S4-S5 |
| [E6: Knowledge Base](./epics/E6_KNOWLEDGE_BASE.md) | 7 stories | 28 | S5-S6 |
| [E7: Messaging](./epics/E7_MESSAGING.md) | 8 stories | 32 | S6-S7 |
| [E8: Desktop Client](./epics/E8_DESKTOP_CLIENT.md) | 7 stories | 35 | S7-S8 |
| [E9: Prod Hardening](./epics/E9_PROD_HARDENING.md) | 10 stories | 45 | S8-S9 |
| **Total** | **77 stories** | **300 pts** | **9 sprints** |

### Planning
| Document | Purpose |
|----------|---------|
| [Sprint Plan](./sprints/SPRINT_PLAN.md) | 9-sprint roadmap with velocity tracking |

### Architecture
| Document | Purpose |
|----------|---------|
| [ADR Index](./adrs/ADR_INDEX.md) | All architecture decision records |
| [ADR-001: Hexagonal](./adrs/ADR-001-hexagonal-architecture.md) | Ports & adapters for multi-backend |
| [ADR-005: Company Tenancy](./adrs/ADR-005-company-tenancy.md) | Multi-company isolation |

### Quality
| Document | Purpose |
|----------|---------|
| [Test Strategy](./testing/TEST_STRATEGY.md) | Testing pyramid, TDD workflow, CI pipeline, coverage targets |

---

## How to Use This Documentation

### For Product Owner
1. Start with [Product Vision](./PRODUCT_VISION.md) — understand the "why"
2. Review [Epic Index](./epics/EPIC_INDEX.md) — prioritize and sequence
3. Track progress in [Sprint Plan](./sprints/SPRINT_PLAN.md) — update velocity each sprint
4. Verify quality with [Definition of Done](./DEFINITION_OF_DONE.md) — enforce at sprint review

### For Product Manager
1. Break down epics into sprint commitments using the story point estimates
2. Monitor dependencies between epics (E3 must precede E5, E4 must precede E7)
3. Adjust scope based on velocity (if <70% delivered, reduce next sprint)
4. Run risk checkpoints after S2, S4, S6, S8

### For Engineers
1. Pick a story from the current sprint in [Sprint Plan](./sprints/SPRINT_PLAN.md)
2. Read its acceptance criteria (Given/When/Then)
3. Follow TDD: write failing test → implement → refactor
4. Verify against [Definition of Done](./DEFINITION_OF_DONE.md) before marking done
5. Check relevant [ADRs](./adrs/ADR_INDEX.md) for architectural guidance

### For QA
1. Read [Test Strategy](./testing/TEST_STRATEGY.md) for the testing pyramid
2. Write E2E tests for each sprint's acceptance tests
3. Run `scripts/e2e-smoke.sh` after each release build
4. Track coverage with `cargo tarpaulin`

---

## What Claude Can Help Build

Given full access to this documentation, Claude can autonomously execute:

### Per Sprint
1. **Read the sprint plan** → identify stories to implement
2. **Read acceptance criteria** → write failing tests (TDD RED phase)
3. **Implement code** → make tests pass (TDD GREEN phase)
4. **Refactor** → clean up while tests stay green (TDD REFACTOR phase)
5. **Verify DoD** → cargo check, clippy, test, doc comments

### Per Epic
1. **Create new crate** → Cargo.toml, lib.rs, module structure
2. **Write migration** → SQL file with new tables
3. **Implement repository** → CRUD + search + tests
4. **Implement API routes** → Axum handlers + OpenAPI annotations
5. **Implement middleware** → trait impl + chain integration
6. **Build frontend page** → Svelte 5 rune-based components

### Per Release
1. **Run full test suite** → cargo test --workspace
2. **Update NORTH_STAR.md** → reflect new capabilities
3. **Update CLAUDE.md** → add new crates, env vars
4. **Update README.md** → feature list
5. **Tag and build** → GitHub Release

### Creative Enhancements Claude Can Propose
- **Agent marketplace**: Community-contributed personas published as a registry
- **Workflow templates**: Pre-built multi-agent workflows (e.g., "Sprint Planning", "Code Review Pipeline")
- **Cost optimization engine**: Analyze agent usage patterns, suggest cheaper models for simple tasks
- **Agent performance scoring**: Track which personas deliver best results per task type
- **Natural language org builder**: "Build me a 5-person startup" → auto-creates company, hires personas, sets up org chart
- **Cross-company benchmarking**: Compare agent productivity across companies
- **Skill evolution**: Track which skills produce best outcomes, auto-tune injection
- **Visual workflow builder**: Drag-and-drop pipeline construction in the frontend
- **Agent pairing**: Two agents review each other's work (adversarial quality)
- **Memory-powered onboarding**: New agents inherit relevant memories from departing agents

---

## Metrics at a Glance

| Metric | v0.6.0 (Now) | v1.0.0 (Target) |
|--------|-------------|----------------|
| Rust crates | 9 | 16 |
| DB tables | 11 | 22+ |
| API routes | 40+ | 80+ |
| Frontend pages | 12 | 20+ |
| Tests | 150 | 400+ |
| Agent presets/personas | 10 | 110+ |
| Skills | 10 | 30+ |
| Middlewares | 8 | 14 |
| Event types | 35 | 55+ |
| Execution backends | 1 | 3 |
| Messaging platforms | 0 | 3+ (native) / 16+ (sidecar) |
| LOC (Rust) | ~12.7K | ~25K+ |
