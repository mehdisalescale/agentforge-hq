# Architecture Decision Records

| ADR | Status | Decision |
|-----|--------|----------|
| [ADR-001](./ADR-001-hexagonal-architecture.md) | Accepted | Hexagonal architecture with ports & adapters for multi-backend |
| [ADR-002](./ADR-002-event-sourcing.md) | Accepted | Event sourcing for audit trail and debugging |
| [ADR-003](./ADR-003-cqrs-read-path.md) | Proposed | CQRS for read-heavy dashboards |
| [ADR-004](./ADR-004-persona-as-code.md) | Accepted | Personas as markdown files (version-controllable, LLM-native) |
| [ADR-005](./ADR-005-company-tenancy.md) | Accepted | Company as tenant boundary with backward-compat default |
| [ADR-006](./ADR-006-fts5-knowledge-base.md) | Accepted | SQLite FTS5 for KB search (zero external deps) |
| [ADR-007](./ADR-007-messaging-sidecar.md) | Accepted | Native Rust for top-3 platforms; AstrBot sidecar for 16+ |
| [ADR-008](./ADR-008-middleware-chain.md) | Accepted | Middleware chain pattern for cross-cutting concerns |
| [ADR-009](./ADR-009-tdd-enforcement.md) | Accepted | TDD-first for all new code |
| [ADR-010](./ADR-010-single-binary-boundary.md) | Accepted | Core features must stay in single binary; external adapters optional |
