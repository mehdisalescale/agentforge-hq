# Architecture Overview

AgentForge is a Rust workspace with 13 crates, compiled into a single binary that serves both the API and the embedded Svelte frontend.

## High-Level Architecture

```mermaid
graph TB
    subgraph Clients
        Browser[Web UI]
        CC[Claude Code]
        Cursor[Cursor]
        ADK[ADK Apps]
    end

    subgraph AgentForge
        API[forge-api<br/>Axum HTTP + WS]
        MCP[forge-mcp-bin<br/>19 MCP tools]
        MW[Middleware Chain<br/>7 middlewares]
        CONF[AgentConfigurator<br/>CLAUDE.md generation]
        HOOK[HookReceiver<br/>Event capture]
    end

    subgraph Core
        DB[forge-db<br/>SQLite WAL]
        CORE[forge-core<br/>EventBus]
        SAFETY[forge-safety<br/>Circuit breaker<br/>Rate limiter]
        PROC[forge-process<br/>CLI spawn]
    end

    subgraph Domain
        ORG[forge-org<br/>Companies, Depts]
        PERSONA[forge-persona<br/>112 personas]
        GOV[forge-governance<br/>Goals, Approvals]
    end

    Browser -->|HTTP/WS| API
    CC -->|stdio MCP| MCP
    Cursor -->|stdio MCP| MCP
    ADK -->|HTTP MCP| MCP

    API --> MW --> PROC
    API --> CONF
    HOOK --> API

    API --> DB
    API --> CORE
    MW --> SAFETY
    PROC -->|spawn| Claude[Claude Code CLI]

    MCP --> DB
    MCP --> ORG
    MCP --> PERSONA
    MCP --> GOV
```

## Design Principles

1. **Single binary** — no runtime dependencies, no Docker required
2. **SQLite WAL** — concurrent reads, single-file database, zero config
3. **Embedded frontend** — Svelte build files compiled into the binary via rust-embed
4. **Event-driven** — 38 ForgeEvent variants broadcast through EventBus
5. **Middleware chain** — composable pipeline for run execution
6. **MCP-first** — 19 tools accessible from any MCP client
7. **Configure, don't inject** — AgentConfigurator writes CLAUDE.md per persona instead of middleware injection
