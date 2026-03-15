# Crate Map

<!-- Last updated: 2026-03-15 -->

13 workspace crates organized by layer.

## Binary

| Crate | Purpose |
|-------|---------|
| **forge-app** | Binary entry point: DB setup, API server, embedded frontend, graceful shutdown, cron scheduler, persona seeding, demo data seeding |
| **forge-mcp-bin** | MCP stdio server with 19 tools via rmcp v0.17 |

## API Layer

| Crate | Purpose |
|-------|---------|
| **forge-api** | Axum HTTP + WebSocket, 14 route modules, CORS, TraceLayer, rust-embed SPA, 7-stage middleware chain, AgentConfigurator (CLAUDE.md + hooks.json generation), HookReceiver (pre-tool / post-tool / stop) |

## Core Infrastructure

| Crate | Purpose |
|-------|---------|
| **forge-core** | ForgeEvent (43 variants), EventBus with fan-out (mpsc for guaranteed persistence delivery, broadcast for best-effort UI), ForgeError hierarchy, typed IDs (AgentId, SessionId, etc.) |
| **forge-db** | SQLite WAL, r2d2 connection pool (write:1, read:N, busy_timeout=5000), 12 migrations, 17 repos, BatchWriter (50 events or 2s flush), SafetyRepo for persistent circuit breaker state |
| **forge-safety** | CircuitBreaker (3-state FSM with persistence via export/restore), RateLimiter (token bucket), CostTracker (budget warn/limit), SecurityScanner (OWASP patterns) |

## Execution

| Crate | Purpose |
|-------|---------|
| **forge-process** | Claude CLI spawn, stream-JSON parsing, ConcurrentRunner (semaphore-based concurrency limiting, max_concurrent + max_output_bytes), LoopDetector, SkillRouter, TaskTypeDetector |
| **forge-git** | Git worktree create/remove/list for multi-agent isolation |

## Domain

| Crate | Purpose |
|-------|---------|
| **forge-agent** | Agent model, 10 presets (CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer, Coordinator), validation |
| **forge-org** | Company, Department, OrgPosition models, org chart builder |
| **forge-persona** | 98 persona catalog across 11 divisions, markdown parser (frontmatter + sections), hire flow (creates Agent + OrgPosition) |
| **forge-governance** | Goal and Approval models |

## Protocol

| Crate | Purpose |
|-------|---------|
| **forge-mcp** | MCP protocol type stubs |

## Dependency Graph

```
forge-app
├── forge-api
│   ├── forge-core
│   ├── forge-db
│   ├── forge-process
│   ├── forge-safety
│   ├── forge-org
│   ├── forge-persona
│   └── forge-governance
├── forge-db
│   └── forge-core
├── forge-process
│   └── forge-core
└── forge-safety
    └── forge-core
```
