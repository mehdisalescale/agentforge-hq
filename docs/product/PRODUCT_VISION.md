# AgentForge — Product Vision & Strategy

> **Living document. Update after each sprint retrospective.**
>
> Owner: Product Owner | Last Updated: 2026-03-11

---

## 1. Product Vision Statement

**For** teams and individuals who want to run autonomous AI workforces,
**who** need more than a single chatbot — they need orchestrated, specialized, self-improving agents,
**AgentForge** is an open-source AI workforce platform
**that** lets you hire specialized agents from a 100+ persona catalog, organize them into companies with org charts and budgets, equip them with structured development methodologies, execute work through multiple runtime backends with 40+ tools, interact from any messaging platform or native desktop app, and build shared knowledge bases — all from a single Rust binary.
**Unlike** CrewAI (Python-only, no persistence, no UI), AutoGen (research-grade, not production), LangGraph (graph-only, no org structure), or Paperclip (Node.js, no single-binary),
**our product** delivers enterprise-grade orchestration in a zero-dependency single binary with a real-time UI, while remaining fully extensible through adapters, plugins, and messaging bridges.

---

## 2. Product Principles

| # | Principle | Implication |
|---|-----------|-------------|
| P1 | **Single binary first** | Core features must compile into one Rust binary. External services (Hermes, AstrBot, OpenClaw) are optional adapters, never requirements. |
| P2 | **Event-sourced truth** | Every state change is a ForgeEvent. The event log IS the audit trail. No silent mutations. |
| P3 | **Type-safe boundaries** | Newtype IDs, exhaustive enums, trait-based extension points. Compiler catches integration bugs before runtime. |
| P4 | **Graceful degradation** | If Hermes is down, Claude backend still works. If AstrBot is offline, web UI still works. Features degrade, never crash. |
| P5 | **Test-driven expansion** | Every new crate ships with unit tests. Every new API route ships with integration tests. Every new UI page ships with E2E coverage. No exceptions. |
| P6 | **Middleware-composable safety** | New business rules (budgets, approvals, security scans) plug into the middleware chain. No God-object controllers. |
| P7 | **Personas over prompts** | Agents have rich identities (personality, deliverables, metrics, workflows), not just system prompts. This is what makes a workforce, not a chatbot. |

---

## 3. Target Users & Personas

### User Persona 1: "Solo AI Entrepreneur" (Primary)

- **Who**: Technical founder running a one-person company
- **Goal**: Have AI agents do the work of a 5-person team
- **Pain**: Existing tools require gluing together 6+ services, no unified view
- **Value**: One `./forge` binary, hire 5 agents, see them work in real-time
- **Success metric**: Productive agent workforce running within 30 minutes

### User Persona 2: "AI-Augmented Dev Team Lead" (Secondary)

- **Who**: Engineering lead with 3-8 developers
- **Goal**: AI agents handle code review, testing, docs, security audits
- **Pain**: AI tools are single-task; no way to coordinate specialized AI roles
- **Value**: Hire AI Reviewer, Tester, SecurityAuditor; they follow TDD, produce auditable work
- **Success metric**: 50% reduction in manual code review time

### User Persona 3: "Agency Operator" (Tertiary)

- **Who**: Digital agency serving multiple clients
- **Goal**: Run separate AI companies per client, each with its own agent team
- **Pain**: No multi-tenant AI orchestration; manually managing prompts across clients
- **Value**: Multi-company isolation, per-client budgets, org chart per engagement
- **Success metric**: Manage 5+ client engagements from one dashboard

---

## 4. Competitive Landscape

| Platform | Language | UI | Persistence | Org Structure | Multi-Backend | Single Binary | Our Advantage |
|----------|---------|-----|-------------|---------------|--------------|---------------|---------------|
| CrewAI | Python | None | None | Flat roles | No | No | Full UI, persistence, org charts, typed safety |
| AutoGen | Python | Basic | None | Flat | Yes | No | Production-grade, not research-grade |
| LangGraph | Python | LangSmith | Cloud | Graph only | Yes | No | Self-hosted, org hierarchy, cost control |
| Semantic Kernel | C#/.NET | None | Optional | None | Yes | No | Lightweight, no runtime deps |
| Paperclip | Node.js | React | PostgreSQL | Yes | Yes | No | Single binary, Rust performance, lower ops burden |
| **AgentForge** | **Rust** | **Svelte 5** | **SQLite** | **Full org charts** | **Yes** | **Yes** | **All of the above, one binary** |

---

## 5. Domain Model (DDD Bounded Contexts)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        AgentForge Domain                            │
│                                                                     │
│  ┌─────────────────┐  ┌──────────────────┐  ┌───────────────────┐  │
│  │  IDENTITY        │  │  ORCHESTRATION    │  │  EXECUTION         │  │
│  │  Context         │  │  Context          │  │  Context           │  │
│  │                   │  │                    │  │                    │  │
│  │  • Persona        │  │  • Company         │  │  • Session         │  │
│  │  • Division       │  │  • Department      │  │  • ProcessBackend  │  │
│  │  • Agent          │  │  • OrgPosition     │  │  • ToolExecution   │  │
│  │  • AgentPreset    │  │  • Goal            │  │  • Worktree        │  │
│  │  • Skill          │  │  • Task (Issue)    │  │  • SubAgent        │  │
│  │  • Memory         │  │  • Approval        │  │  • Pipeline        │  │
│  │                   │  │  • Budget          │  │  • BestOfN         │  │
│  │                   │  │  • Schedule        │  │  • LoopDetector    │  │
│  └────────┬──────────┘  └────────┬───────────┘  └────────┬──────────┘  │
│           │                      │                        │             │
│  ┌────────┴──────────┐  ┌────────┴───────────┐  ┌────────┴──────────┐  │
│  │  KNOWLEDGE         │  │  COMMUNICATION     │  │  SAFETY            │  │
│  │  Context           │  │  Context           │  │  Context           │  │
│  │                    │  │                     │  │                    │  │
│  │  • Document        │  │  • Platform         │  │  • CircuitBreaker  │  │
│  │  • Chunk           │  │  • MessageBridge    │  │  • RateLimiter     │  │
│  │  • SearchIndex     │  │  • IntentRouter     │  │  • CostTracker     │  │
│  │  • KBQuery         │  │  • Notification     │  │  • BudgetEnforcer  │  │
│  │                    │  │  • WebhookCallback  │  │  • SecurityScanner │  │
│  │                    │  │                     │  │  • QualityGate     │  │
│  └────────────────────┘  └─────────────────────┘  └────────────────────┘  │
│                                                                           │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │  METHODOLOGY Context (Cross-Cutting)                               │   │
│  │  • DevWorkflow (TDD, Debugging, Brainstorming, Planning)           │   │
│  │  • CodeReview (Confidence Scoring, 6-Agent Parallel Review)        │   │
│  │  • SecurityAudit (9 OWASP Patterns)                                │   │
│  │  • HookRule (Event-Driven Custom Behaviors)                        │   │
│  └────────────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────────────┘
```

### Aggregate Roots

| Aggregate | Root Entity | Owned Entities |
|-----------|------------|---------------|
| Company | Company | Department, OrgPosition, Budget, Goal |
| Agent | Agent | Persona, Memory[], Skill[], Config |
| Session | Session | Event[], WorktreeInfo, Cost |
| Knowledge | Document | Chunk[], SearchIndex |
| Workflow | Workflow | PipelineStep[], WorkflowRun |
| Approval | Approval | ApprovalComment[] |

### Domain Events (extending existing 35 → 55+)

```
New events:
  CompanyCreated, CompanyUpdated, CompanyDeleted,
  DepartmentCreated, AgentHired, AgentFired, AgentTransferred,
  GoalCreated, GoalCompleted, GoalBlocked,
  ApprovalRequested, ApprovalGranted, ApprovalDenied,
  BudgetAllocated, BudgetExhausted,
  PersonaImported, PersonaCatalogRefreshed,
  DocumentUploaded, DocumentIndexed, KBSearchPerformed,
  MessageReceived, MessageRouted, NotificationSent,
  BackendHealthChanged, BackendSwitched,
  SecurityScanPassed, SecurityScanFailed,
  MethodologyActivated, TDDCycleCompleted, ReviewCompleted
```

---

## 6. Architecture Decision Records (Summary)

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-001 | Hexagonal architecture with ports & adapters | Multiple backends (Claude, Hermes, OpenClaw) need clean abstraction |
| ADR-002 | Event sourcing for audit trail | Regulatory compliance, debugging, time-travel debugging |
| ADR-003 | CQRS for read-heavy dashboards | Analytics, org chart, KB search are read-heavy; writes are event-driven |
| ADR-004 | Repository pattern for all persistence | Already established in v0.6.0; extend, don't replace |
| ADR-005 | Middleware chain for cross-cutting concerns | Already 8 middlewares; proven extensible pattern |
| ADR-006 | Trait-based adapters for backends | `ProcessBackend` trait with Claude/Hermes/OpenClaw implementations |
| ADR-007 | SQLite FTS5 for knowledge base | Zero external deps; fits single-binary principle |
| ADR-008 | Sidecar pattern for messaging | AstrBot as optional subprocess; native Rust for top 3 platforms later |
| ADR-009 | Persona-as-code (markdown) | Human-readable, version-controllable, LLM-native format |
| ADR-010 | Company as tenant boundary | All data scoped to company_id; backward-compat via default company |

See `docs/product/adrs/` for full ADR documents.

---

## 7. Quality Attributes (Non-Functional Requirements)

| Attribute | Target | Measurement |
|-----------|--------|-------------|
| **Startup time** | < 2 seconds | Time from `./forge` to HTTP 200 on /health |
| **Memory footprint** | < 100MB idle | RSS with no active sessions |
| **Concurrent sessions** | 50+ simultaneous | Load test with 50 parallel agent runs |
| **API latency (P95)** | < 50ms for CRUD | Excluding agent spawn (which is async) |
| **Event throughput** | 1000 events/sec | BatchWriter benchmark |
| **Test coverage** | > 80% line coverage | `cargo tarpaulin` |
| **Zero downtime deploy** | Graceful shutdown | In-flight requests complete before exit |
| **Data durability** | Zero event loss | WAL mode + BatchWriter flush guarantee |
| **Security** | No known CVEs | `cargo audit` clean |
| **Accessibility** | WCAG 2.1 AA | Frontend audit |

---

## 8. Release Strategy

| Version | Codename | Focus | Gate |
|---------|----------|-------|------|
| **v0.7.0** | "Roster" | 100+ personas + dev methodology | All persona tests pass, skill routing works |
| **v0.8.0** | "Org" | Multi-company, org charts, governance | Company CRUD, budget enforcement, approval flow |
| **v0.9.0** | "Engines" | Multi-backend (Hermes + OpenClaw) | Backend switching works, memory sync verified |
| **v0.10.0** | "Library" | Knowledge base + FTS5 search | Document upload, chunk search, context injection |
| **v0.11.0** | "Channels" | Multi-platform messaging | Telegram + Slack working, notifications routing |
| **v0.12.0** | "Desktop" | Electron desktop client | Desktop connects to Forge API, all views working |
| **v1.0.0** | "Forge" | Production polish, E2E, docs | All acceptance gates green, E2E suite passes |

---

## 9. Success Metrics (OKRs)

### Objective 1: Deliver a production-grade AI workforce platform

| Key Result | Target | Measurement |
|-----------|--------|-------------|
| KR1.1 | 100+ agent personas importable | Count of parseable persona files |
| KR1.2 | 3 execution backends working | Claude + Hermes + OpenClaw all spawn agents |
| KR1.3 | Multi-company with budget enforcement | Create 3 companies, budgets pause agents correctly |
| KR1.4 | < 5 min to first productive agent | Timed user test from `./forge` to working output |

### Objective 2: Enterprise-grade quality

| Key Result | Target | Measurement |
|-----------|--------|-------------|
| KR2.1 | > 80% test coverage | `cargo tarpaulin` report |
| KR2.2 | Zero critical bugs in release | Bug tracker at release time |
| KR2.3 | All ADRs documented | Count ADRs vs architectural decisions made |
| KR2.4 | Full OpenAPI spec | All routes documented in `/docs` endpoint |

### Objective 3: Community adoption

| Key Result | Target | Measurement |
|-----------|--------|-------------|
| KR3.1 | 500+ GitHub stars within 3 months of v1.0 | GitHub API |
| KR3.2 | 10+ community-contributed personas | PR count |
| KR3.3 | 3+ community adapter implementations | Adapter crate count |

---

## 10. Risks & Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|-----------|
| R1 | SQLite contention under 50+ concurrent agents | Medium | High | Connection pool with read replicas; CQRS read path |
| R2 | Hermes Python process crashes take down sessions | Medium | Medium | Adapter timeout + circuit breaker; session recovery |
| R3 | 100+ persona files slow startup | Low | Low | Lazy loading; only parse on first access |
| R4 | Messaging sidecar adds operational complexity | High | Medium | Optional; core works without it; Docker Compose template |
| R5 | Desktop client diverges from web UI | Medium | Medium | Shared API contract; generated TypeScript types from OpenAPI |
| R6 | Budget enforcement race condition | Medium | High | Atomic check-and-decrement in single SQLite transaction |
| R7 | Event bus overflow under burst load | Low | Medium | Bounded channel + backpressure; events persisted by BatchWriter |
