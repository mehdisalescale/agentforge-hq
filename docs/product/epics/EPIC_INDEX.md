# Epic Index

> **All epics for AgentForge expansion. Each epic has its own document with user stories.**

---

## Epic Map

```
                              ┌──────────────┐
                              │  E0: CORE    │ (existing v0.6.0)
                              │  FOUNDATION  │
                              └──────┬───────┘
                    ┌────────────────┼────────────────┐
                    ↓                ↓                ↓
           ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
           │ E1: PERSONA  │ │ E2: DEV      │ │ E3: HEXAGONAL│
           │ CATALOG      │ │ METHODOLOGY  │ │ BACKENDS     │
           └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
                  │                │                  │
                  ↓                ↓                  ↓
           ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
           │ E4: ORG      │ │ E5: MULTI-   │ │ E6: KNOWLEDGE│
           │ STRUCTURE    │ │ BACKEND EXEC │ │ BASE         │
           └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
                  │                │                  │
                  └────────────────┼──────────────────┘
                                   ↓
                          ┌──────────────┐
                          │ E7: MESSAGING│
                          │ & COMMS      │
                          └──────┬───────┘
                                 ↓
                          ┌──────────────┐
                          │ E8: DESKTOP  │
                          │ CLIENT       │
                          └──────┬───────┘
                                 ↓
                          ┌──────────────┐
                          │ E9: PROD     │
                          │ HARDENING    │
                          └──────────────┘
```

## Epic Summary

| Epic | Name | Stories | Sprint | Release |
|------|------|---------|--------|---------|
| **E1** | [Persona Catalog](./E1_PERSONA_CATALOG.md) | 8 | S1 | v0.7.0 |
| **E2** | [Dev Methodology](./E2_DEV_METHODOLOGY.md) | 9 | S1-S2 | v0.7.0 |
| **E3** | [Hexagonal Backend Architecture](./E3_HEXAGONAL_BACKENDS.md) | 6 | S2 | v0.8.0 |
| **E4** | [Org Structure & Governance](./E4_ORG_STRUCTURE.md) | 12 | S3-S4 | v0.8.0 |
| **E5** | [Multi-Backend Execution](./E5_MULTI_BACKEND.md) | 10 | S4-S5 | v0.9.0 |
| **E6** | [Knowledge Base](./E6_KNOWLEDGE_BASE.md) | 7 | S5-S6 | v0.10.0 |
| **E7** | [Messaging & Communications](./E7_MESSAGING.md) | 8 | S6-S7 | v0.11.0 |
| **E8** | [Desktop Client](./E8_DESKTOP_CLIENT.md) | 7 | S7-S8 | v0.12.0 |
| **E9** | [Production Hardening](./E9_PROD_HARDENING.md) | 10 | S8-S9 | v1.0.0 |

**Total**: 77 user stories across 9 epics, targeting 9 sprints.

---

## Dependencies

| Epic | Depends On | Reason |
|------|-----------|--------|
| E1 | — | No dependencies, can start immediately |
| E2 | — | No dependencies, can start immediately |
| E3 | — | Refactors existing process spawning; no deps |
| E4 | E1 | Divisions from personas map to departments |
| E5 | E3 | Backend trait must exist before Hermes/OpenClaw adapters |
| E6 | E4 | KB scoped to companies |
| E7 | E4, E6 | Messaging needs company context + KB search |
| E8 | E4, E5, E6 | Desktop needs all backend APIs |
| E9 | All | Hardening requires all features present |

**Critical path**: E3 → E5 (backend architecture must be done before multi-backend)
**Parallel lanes**: E1 ∥ E2 ∥ E3 (all sprint 1-2, no deps)
