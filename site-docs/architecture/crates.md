# Crate Map

13 workspace crates organized by layer.

## Binary

| Crate | Purpose |
|-------|---------|
| **forge-app** | Binary entry point: DB setup, API server, embedded frontend, graceful shutdown, cron scheduler, demo data seeding |
| **forge-mcp-bin** | MCP stdio server with 19 tools via rmcp |

## API Layer

| Crate | Purpose |
|-------|---------|
| **forge-api** | Axum HTTP + WebSocket, routes, CORS, TraceLayer, rust-embed SPA, 7-middleware chain, AgentConfigurator, HookReceiver |

## Core Infrastructure

| Crate | Purpose |
|-------|---------|
| **forge-core** | ForgeEvent (38 variants), EventBus broadcast, ForgeError hierarchy, typed IDs (AgentId, SessionId, etc.) |
| **forge-db** | SQLite WAL, 12 migrations, 16 repos, BatchWriter (50 events or 2s flush) |
| **forge-safety** | CircuitBreaker (3-state FSM), RateLimiter (token bucket), CostTracker (budget warn/limit), SecurityScanner (OWASP patterns) |

## Execution

| Crate | Purpose |
|-------|---------|
| **forge-process** | Claude CLI spawn, stream-JSON parsing, ConcurrentRunner, LoopDetector, SkillRouter, TaskTypeDetector |
| **forge-git** | Git worktree create/remove/list for multi-agent isolation |

## Domain

| Crate | Purpose |
|-------|---------|
| **forge-agent** | Agent model, 10 presets (CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer, Coordinator), validation |
| **forge-org** | Company, Department, OrgPosition models, org chart builder |
| **forge-persona** | 112 persona catalog across 11 divisions, parser, hire flow |
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
