# Architecture Overview

<!-- Last updated: 2026-03-15 -->

AgentForge is a Rust workspace with 13 crates, compiled into a single binary that serves both the API and the embedded Svelte frontend.

## High-Level Architecture

```mermaid
graph TB
    subgraph Clients
        FE[Frontend SPA<br/>Svelte 5 + TailwindCSS]
        CLI[Claude Code<br/>via Hooks]
        MCP[MCP Clients<br/>Claude Code / Cursor / ADK]
    end

    subgraph "API Layer (forge-api)"
        HTTP[Axum HTTP Routes<br/>14 route modules]
        WS[WebSocket Stream<br/>real-time events]
        HOOKS[HookReceiver<br/>pre-tool / post-tool / stop]
    end

    subgraph "Orchestration"
        CONFIG[AgentConfigurator<br/>CLAUDE.md + hooks.json]
        MW[Middleware Pipeline<br/>Rate → CB → Cost → Gov → Sec → Persist → Spawn]
        EB[EventBus fan-out<br/>mpsc: persistence<br/>broadcast: UI]
    end

    subgraph "Safety (forge-safety)"
        CB[CircuitBreaker<br/>3-state FSM]
        RL[RateLimiter<br/>token bucket]
        CT[CostTracker<br/>budget warn/limit]
        SS[SecurityScanner<br/>OWASP patterns]
    end

    subgraph "Process (forge-process)"
        CR[ConcurrentRunner<br/>semaphore-limited]
        PROC[Claude CLI Process<br/>stream-JSON parsing]
    end

    subgraph "Data (forge-db)"
        POOL[DbPool r2d2<br/>write:1 read:N]
        BW[BatchWriter<br/>50 events / 2s flush]
        REPOS[17 Repositories]
    end

    subgraph "Governance (forge-org + forge-governance)"
        COMP[Companies]
        DEPT[Departments]
        ORG[OrgPositions]
        GOALS[Goals]
        APPROVALS[Approvals]
    end

    subgraph "Personas (forge-persona)"
        MD[Markdown Catalog<br/>98 personas / 11 divisions]
        PARSER[PersonaParser<br/>frontmatter + sections]
    end

    FE -->|HTTP/WS| HTTP
    FE --> WS
    CLI -->|POST| HOOKS
    MCP -->|stdio| MCPBIN[forge-mcp-bin<br/>19 MCP tools]

    HTTP --> MW
    HOOKS --> EB
    MW --> CONFIG
    MW --> CB & RL & CT
    CONFIG --> CR
    CR --> PROC

    EB -->|mpsc guaranteed| BW
    EB -->|broadcast best-effort| WS
    BW --> POOL
    REPOS --> POOL

    MW --> REPOS
    HTTP --> REPOS
    MCPBIN --> REPOS

    MD --> PARSER -->|startup seed| REPOS
```

## Design Principles

1. **Single binary** — no runtime dependencies, no Docker required
2. **SQLite WAL** — concurrent reads, single-file database, zero config
3. **Embedded frontend** — Svelte build files compiled into the binary via rust-embed
4. **Event-driven** — 43 ForgeEvent variants fanned out through EventBus (mpsc for persistence, broadcast for UI)
5. **Middleware chain** — 7-stage composable pipeline for run execution
6. **MCP-first** — 19 tools accessible from any MCP client
7. **Configure, don't inject** — AgentConfigurator writes CLAUDE.md + hooks.json per persona instead of middleware injection

## Request Flow

A typical `POST /api/v1/run` request passes through:

1. **RateLimitMiddleware** — token bucket check
2. **CircuitBreakerMiddleware** — fail-fast if circuit is open
3. **CostCheckMiddleware** — budget enforcement
4. **GovernanceMiddleware** — inject company context, goals, pending approvals
5. **SecurityScanMiddleware** — OWASP pattern scanning on output
6. **PersistMiddleware** — set session to running, emit lifecycle events
7. **SpawnMiddleware** — AgentConfigurator writes workspace files, then spawns Claude CLI

## Hook Receiver Flow

Claude Code instances report back to AgentForge via HTTP hooks:

1. **PreToolUse** → `POST /api/v1/hooks/pre-tool` — gate tool usage (currently allow-all)
2. **PostToolUse** → `POST /api/v1/hooks/post-tool` — security scan on tool output
3. **Stop** → `POST /api/v1/hooks/stop` — mark session completed
