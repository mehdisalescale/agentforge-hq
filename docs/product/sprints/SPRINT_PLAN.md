# Sprint Plan — v0.6.0 → v1.0.0

> **9 sprints. 77 stories. 300 story points. Single-binary AI workforce platform.**
>
> Sprint cadence: Weekly (adjustable based on velocity)
> Methodology: TDD-first. Every crate ships with tests before implementation.

---

## Sprint Overview

```
Sprint  Epics           Release    Points  Focus
──────  ──────────────  ─────────  ──────  ─────────────────────────────
S1      E1 + E2(a)      —          34      Personas + Skills + Detection
S2      E2(b) + E3      v0.7.0     32      Security + Code Review + Hexagonal
S3      E4(a)           —          28      Companies + Departments + Budgets
S4      E4(b) + E5(a)   v0.8.0     38      Approvals + UI + Hermes Adapter
S5      E5(b) + E6(a)   v0.9.0     33      OpenClaw + KB Foundation
S6      E6(b) + E7(a)   v0.10.0    30      KB Frontend + Telegram + Slack
S7      E7(b) + E8(a)   v0.11.0    33      Discord + Notifications + Desktop Start
S8      E8(b) + E9(a)   v0.12.0    37      Desktop Complete + Auth + Performance
S9      E9(b)           v1.0.0     35      E2E + Docs + Docker + Release
                                   ═══
                                   300
```

---

## Sprint 1: "Roster" — Personas + Dev Skills Foundation

**Goal**: 100+ personas browseable, task type detection working, 20 new skills loaded.

| # | Story | Points | Epic | Crate |
|---|-------|--------|------|-------|
| 1 | E1-S1: Persona Markdown Parser | 3 | E1 | forge-persona (new) |
| 2 | E1-S2: Persona DB Schema & Repo | 3 | E1 | forge-db |
| 3 | E1-S3: Division Taxonomy | 2 | E1 | forge-db |
| 4 | E1-S4: Persona API Endpoints | 3 | E1 | forge-api |
| 5 | E1-S5: Startup Loading | 2 | E1 | forge-app |
| 6 | E1-S7: Persona → Agent Config | 3 | E1 | forge-persona |
| 7 | E2-S1: Skill Import Pipeline | 3 | E2 | forge-db |
| 8 | E2-S2: Task Type Detection | 3 | E2 | forge-process |
| 9 | E2-S3: Skill Router | 3 | E2 | forge-process |
| 10 | E2-S4: Detection Middleware | 2 | E2 | forge-api |
| 11 | E2-S9: Skills Directory Population | 2 | E2 | skills/ |
| 12 | E1-S8: MCP Persona Tools | 2 | E1 | forge-mcp-bin |

**Sprint velocity target**: 34 points
**Key deliverable**: `GET /api/v1/personas` returns 100+ personas. Skills auto-inject by task type.
**Definition of Done**: All tests pass. `cargo check` clean. OpenAPI annotations on new routes.

**Sprint 1 Acceptance Test (End of Sprint):**
```bash
# 1. Start server
./forge &

# 2. Verify personas loaded
curl http://localhost:4173/api/v1/personas | jq '.[] | length'
# Expected: 100+

# 3. Verify divisions
curl http://localhost:4173/api/v1/personas/divisions | jq '.[].name'
# Expected: engineering, design, marketing, ...

# 4. Hire a persona
curl -X POST http://localhost:4173/api/v1/personas/$ID/hire \
  -d '{"name": "my-frontend-dev"}' | jq '.id'
# Expected: agent UUID

# 5. Run with task type detection
curl -X POST http://localhost:4173/api/v1/run \
  -d '{"agent_id": "...", "prompt": "fix the login bug"}' | jq '.status'
# Expected: "spawned" (TDD + debugging skills injected)
```

---

## Sprint 2: "Shield" — Security + Code Review + Hexagonal Architecture

**Goal**: Security scanner detects OWASP patterns. Code review engine works. Backend trait defined.

| # | Story | Points | Epic | Crate |
|---|-------|--------|------|-------|
| 1 | E2-S5: Security Scanner (9 patterns) | 5 | E2 | forge-safety |
| 2 | E2-S6: SecurityScan Middleware | 2 | E2 | forge-api |
| 3 | E2-S7: Code Review Engine | 8 | E2 | forge-process |
| 4 | E1-S6: Persona Catalog Frontend | 5 | E1 | frontend |
| 5 | E2-S8: Methodology Frontend | 3 | E2 | frontend |
| 6 | E3-S1: ProcessBackend Trait | 5 | E3 | forge-process |
| 7 | E3-S2: Claude Backend Extract | 5 | E3 | forge-process |

**Sprint velocity target**: 33 points (~same as S1 to calibrate)
**Release**: **v0.7.0 "Roster"** (personas + methodology + security)

**v0.7.0 Release Acceptance Test:**
```bash
# Security scanner catches eval injection
echo 'eval(user_input)' | forge-test-security
# Expected: FAILED - eval_injection (critical)

# Code review runs 6 parallel agents
curl -X POST /api/v1/run \
  -d '{"agent_id": "...", "prompt": "review this code: ..."}'
# Expected: 6 sub-agent events, aggregated review with confidence scores
```

---

## Sprint 3: "Company" — Companies + Departments + Budgets

**Goal**: Multi-company works. Agents belong to companies. Budgets enforce.

| # | Story | Points | Epic |
|---|-------|--------|------|
| 1 | E4-S1: Company Entity & Tenancy | 5 | E4 |
| 2 | E4-S2: Department & Org Position | 5 | E4 |
| 3 | E4-S3: Budget Enforcement | 5 | E4 |
| 4 | E4-S4: Goal Hierarchy | 3 | E4 |
| 5 | E3-S3: Backend Routing Middleware | 3 | E3 |
| 6 | E3-S4: Agent Backend Config | 3 | E3 |
| 7 | E3-S5: Backend Health Dashboard | 2 | E3 |
| 8 | E3-S6: Event Normalization | 3 | E3 |

**Sprint velocity target**: 29 points

---

## Sprint 4: "Governance" — Approvals + UI + Hermes Start

**Goal**: Approval gates work. Company UI complete. Hermes adapter functional.

| # | Story | Points | Epic |
|---|-------|--------|------|
| 1 | E4-S5: Approval Gates | 5 | E4 |
| 2 | E4-S6: Budget Middleware | 3 | E4 |
| 3 | E4-S7: Company API Endpoints | 5 | E4 |
| 4 | E4-S8: Companies Page | 3 | E4 |
| 5 | E4-S9: Org Chart Page | 5 | E4 |
| 6 | E4-S10: Goals Page | 3 | E4 |
| 7 | E4-S11: Approvals Page | 3 | E4 |
| 8 | E4-S12: Budget Dashboard | 3 | E4 |
| 9 | E5-S1: Hermes Backend Adapter | 8 | E5 |

**Sprint velocity target**: 38 points
**Release**: **v0.8.0 "Org"** (multi-company, org charts, governance)

---

## Sprint 5: "Engines" — Hermes Finish + OpenClaw + KB Start

| # | Story | Points | Epic |
|---|-------|--------|------|
| 1 | E5-S2: Hermes Memory Sync | 5 | E5 |
| 2 | E5-S3: Tool Filtering | 3 | E5 |
| 3 | E5-S4: OpenClaw Webhook Adapter | 5 | E5 |
| 4 | E5-S5: Callback Endpoint | 3 | E5 |
| 5 | E5-S6: Backend Failover | 5 | E5 |
| 6 | E6-S1: Document Model & Chunking | 5 | E6 |
| 7 | E6-S2: FTS5 Search Index | 5 | E6 |

**Release**: **v0.9.0 "Engines"** (Claude + Hermes + OpenClaw)

---

## Sprint 6: "Library" — KB Complete + Messaging Start

| # | Story | Points | Epic |
|---|-------|--------|------|
| 1 | E6-S3: KB API Endpoints | 3 | E6 |
| 2 | E6-S4: KnowledgeInjection Middleware | 3 | E6 |
| 3 | E6-S5: KB Frontend Page | 5 | E6 |
| 4 | E6-S6: Embedding (Future-Ready) | 3 | E6 |
| 5 | E6-S7: MCP Knowledge Tools | 2 | E6 |
| 6 | E7-S1: MessageBridge Trait | 3 | E7 |
| 7 | E7-S2: Telegram Adapter | 5 | E7 |
| 8 | E7-S3: Slack Adapter | 5 | E7 |

**Release**: **v0.10.0 "Library"** (knowledge base + FTS5)

---

## Sprint 7: "Channels" — Messaging Complete + Desktop Start

| # | Story | Points | Epic |
|---|-------|--------|------|
| 1 | E7-S4: Discord Adapter | 5 | E7 |
| 2 | E7-S5: Intent Router | 5 | E7 |
| 3 | E7-S6: Notification Router | 3 | E7 |
| 4 | E7-S7: Messaging Config & Frontend | 5 | E7 |
| 5 | E8-S1: Forge API Client Module | 5 | E8 |
| 6 | E8-S2: Chat Interface (Streaming) | 5 | E8 |
| 7 | E5-S7-10: Backend Frontend (carried) | 5 | E5 |

**Release**: **v0.11.0 "Channels"** (Telegram + Slack + Discord)

---

## Sprint 8: "Desktop" — Desktop Complete + Auth + Perf

| # | Story | Points | Epic |
|---|-------|--------|------|
| 1 | E8-S3: Company & Agent Sidebar | 5 | E8 |
| 2 | E8-S4: Org Chart View | 5 | E8 |
| 3 | E8-S5: Permission & Approval | 5 | E8 |
| 4 | E8-S6: KB Search | 3 | E8 |
| 5 | E8-S7: Build & Distribution | 5 | E8 |
| 6 | E9-S1: Authentication System | 5 | E9 |
| 7 | E9-S2: RBAC | 5 | E9 |
| 8 | E9-S4: Performance Benchmark | 3 | E9 |

**Release**: **v0.12.0 "Desktop"** (native app + auth)

---

## Sprint 9: "Forge" — Polish & v1.0.0

| # | Story | Points | Epic |
|---|-------|--------|------|
| 1 | E9-S3: E2E Test Suite | 8 | E9 |
| 2 | E9-S5: Connection Pooling | 5 | E9 |
| 3 | E9-S6: Docker Compose | 5 | E9 |
| 4 | E9-S7: Migration Path | 3 | E9 |
| 5 | E9-S8: OpenAPI Completion | 3 | E9 |
| 6 | E9-S9: Documentation Suite | 5 | E9 |
| 7 | E9-S10: Release Pipeline | 5 | E9 |
| 8 | E7-S8: AstrBot Sidecar Bridge | 3 | E7 |

**Release**: **v1.0.0 "Forge"** — Full AI workforce platform.

---

## Velocity Tracking

| Sprint | Planned | Completed | Velocity | Notes |
|--------|---------|-----------|----------|-------|
| S1 | 34 | — | — | — |
| S2 | 33 | — | — | — |
| S3 | 29 | — | — | — |
| S4 | 38 | — | — | — |
| S5 | 31 | — | — | — |
| S6 | 30 | — | — | — |
| S7 | 33 | — | — | — |
| S8 | 37 | — | — | — |
| S9 | 35 | — | — | — |

**Burndown rule**: If a sprint delivers <70% of planned points, reduce next sprint scope.
**Carry-over rule**: Incomplete stories carry to next sprint with priority.

---

## Risk Checkpoints

| After Sprint | Check | Action if Red |
|-------------|-------|---------------|
| S2 | Do personas + skills + hexagonal work together? | Simplify E4 scope |
| S4 | Is Hermes adapter stable? Memory sync reliable? | Fall back to Claude-only for v0.9 |
| S6 | Is FTS5 fast enough for 10K+ chunks? | Add caching layer |
| S8 | Does desktop app connect reliably? | Ship web-only for v1.0 |
