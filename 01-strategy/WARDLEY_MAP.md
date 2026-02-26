# Claude Forge: Strategic Positioning via Wardley Mapping

## Overview

Wardley Mapping positions components on two axes: **visibility** (how visible to the user, top = most visible) and **evolution** (how mature/commoditized, left = novel, right = commodity). This analysis maps Forge's components to identify where to invest (left side = moat), where to use commodity (right side = leverage existing), and what strategic moves to make.

---

## The Wardley Map

```
VISIBILITY (User Need)
  ^
  |
  |  [Developer Productivity]
  |       |
  |       +--[Multi-Agent Workflows]----+--[Single-Agent Tasks]
  |       |                             |
  |       +--[Safety & Cost Control]    +--[Code Completion]
  |       |                                    |
  |  [Agentic UI]                         [IDE Integration]
  |       |                                    |
  |       +--[Swim-Lane Timeline]         [VS Code / Cursor]........
  |       |
  |       +--[Session Browser]
  |       |
  |       +--[Agent Presets]
  |       |
  |  [Orchestration Engine]
  |       |
  |       +--[Workflow DAG Engine]--+--[Approval Gates]
  |       |                        |
  |       +--[Agent Coordination]  +--[Circuit Breaker]
  |       |                        |
  |       +--[Event Bus]           +--[Rate Limiter]
  |       |                        |
  |       +--[Cost Tracker]        +--[Permission Engine]
  |       |
  |  [Integration Layer]
  |       |
  |       +--[MCP Server]----------+--[MCP Client]
  |       |                        |
  |       +--[Git Integration]     +--[Session Persistence]
  |       |
  |       +--[Plugin System]
  |       |
  |  [Infrastructure]
  |       |
  |       +--[Embedded Web UI]-----+--[WebSocket Streaming]
  |       |                        |
  |       +--[SQLite + WAL]        +--[Process Spawning]
  |       |                        |
  |       +--[rust-embed]          +--[Tokio Runtime]
  |       |                        |
  |       +--[Axum HTTP]           +--[DashMap Concurrency]
  |
  +------------------------------------------------------------------------>
         Genesis          Custom           Product          Commodity
         (Novel)          (Emerging)       (Converging)     (Standard)
```

### Positioned Components

```
GENESIS (Invest heavily -- these ARE the moat)
|---------------------------------------------------------|
| Swim-Lane Timeline     - No competitor has this for     |
|                          coding agents. Novel UX.       |
| Safety Engine          - Circuit breaker + cost control |
|   (integrated)           + approval gates as default.   |
|                          No one else does all three.    |
| Agent Coordination     - Multi-agent handoff, broadcast |
|   Protocol               groups -- purpose-built for    |
|                          coding workflows.              |
| 61-Repo Absorption     - Systematic pattern extraction  |
|   Strategy               from entire ecosystem.         |
|                          Unique methodology.            |
|---------------------------------------------------------|

CUSTOM (Build thoughtfully -- differentiation opportunity)
|---------------------------------------------------------|
| Workflow DAG Engine    - LangGraph has one for general  |
|                          AI. Ours is coding-specific.   |
| MCP Dual-Mode          - Forge as both platform AND     |
|   Architecture           MCP server. Emerging pattern.  |
| Agent Presets           - 100+ coding-specific presets.  |
|   (curated library)      Quality curation is the value. |
| Permission Engine      - Directory/command restrictions  |
|                          for coding agents.             |
| Plugin System          - WASI-based secure extension    |
|   (WASI-sandboxed)       mechanism.                     |
| Cost Tracker           - Real-time per-agent cost       |
|                          accumulation and budgets.      |
|---------------------------------------------------------|

PRODUCT (Leverage existing patterns -- don't reinvent)
|---------------------------------------------------------|
| Event Bus              - Tokio broadcast channels.      |
|                          Well-understood pattern.       |
| Session Persistence    - SQLite + batch writes.         |
|                          Standard approach.             |
| Git Integration        - git2-rs or shell commands.     |
|                          Well-known APIs.               |
| Embedded Web UI        - Svelte 5 + rust-embed.         |
|                          Established approach.          |
| WebSocket Streaming    - Axum + tokio-tungstenite.      |
|                          Commodity protocol.            |
| Process Spawning       - tokio::process::Command.       |
|                          Standard library.              |
|---------------------------------------------------------|

COMMODITY (Use off-the-shelf -- zero custom work)
|---------------------------------------------------------|
| SQLite + WAL           - rusqlite with bundled SQLite.  |
|                          Battle-tested. No custom DB.   |
| Axum HTTP              - Axum 0.8. Standard web         |
|                          framework. Use as-is.          |
| Tokio Runtime          - Async runtime. Commodity.      |
| DashMap                - Concurrent hashmap. Commodity.  |
| rust-embed             - Static file embedding.         |
|                          Commodity crate.               |
| TailwindCSS            - Utility CSS. Commodity.        |
| serde / serde_json     - Serialization. Commodity.      |
|---------------------------------------------------------|
```

---

## Component Evolution Analysis

### 1. Swim-Lane Timeline (Genesis -> Custom)

**Current position:** Genesis. No agentic coding tool has a real-time swim-lane view of concurrent agent activity aligned on a shared timeline. LangGraph Studio has workflow visualization, but it shows graph structure, not temporal execution.

**Evolution path:** Over 18-24 months, other tools will copy this pattern. Forge must establish it as the reference implementation and continue innovating (predictive cost projections on the timeline, anomaly highlighting, workflow playback).

**Investment level:** HIGH. This is the signature UX that differentiates Forge visually and functionally.

### 2. Safety Engine (Genesis -> Custom)

**Current position:** Genesis. Individual components exist elsewhere (LangGraph has human-in-the-loop, some MCP tools have rate limiting), but no one has an integrated safety engine with circuit breakers + cost budgets + rate limiting + permission boundaries + approval gates, all enabled by default and designed for coding agents.

**Evolution path:** Safety will become a standard requirement within 24 months. First movers who establish the safety patterns will define the standard. Forge should aim to be the reference implementation.

**Investment level:** HIGH. Safety is both a product feature and a market positioning tool.

### 3. Agent Coordination Protocol (Genesis -> Custom)

**Current position:** Genesis. Multi-agent coordination for coding tasks (handoff from writer to reviewer, broadcast to testing agents, group coordination for parallel refactoring) is mostly unexplored territory in purpose-built tools.

**Evolution path:** As multi-agent coding becomes mainstream, coordination patterns will standardize. Forge should define these patterns early.

**Investment level:** HIGH. This is where the multi-agent value proposition lives.

### 4. MCP Dual-Mode Architecture (Custom)

**Current position:** Custom. MCP is standardized, but operating as both a standalone platform and an MCP server is an emerging architectural pattern. A few tools do one or the other; very few do both comprehensively.

**Evolution path:** Dual-mode will become the expected architecture for AI tools within 12 months. Being early gives Forge integration advantages across the MCP ecosystem.

**Investment level:** MEDIUM-HIGH. The architectural investment is largely made; ongoing work is ensuring comprehensive tool coverage.

### 5. Workflow DAG Engine (Custom -> Product)

**Current position:** Custom. LangGraph has proven the graph-based workflow model. Forge's version is coding-specific (understands git operations, file modifications, test results as first-class workflow events).

**Evolution path:** Workflow engines for AI agents will commoditize within 18 months. Forge's advantage is domain specialization, not the workflow engine itself.

**Investment level:** MEDIUM. Build a solid engine, but don't over-engineer. The value is in coding-specific workflow templates, not novel graph theory.

### 6. Agent Presets (Custom)

**Current position:** Custom. Curated, high-quality agent presets for coding tasks (code reviewer, test writer, security auditor, documentation generator, refactoring specialist) are a differentiator. The value is in the curation quality and coverage, not the preset mechanism.

**Evolution path:** Preset libraries will proliferate. Forge's advantage is being first to offer 100+ well-tested presets with an opinionated quality bar.

**Investment level:** MEDIUM. The preset format is simple; the investment is in testing and refining each preset.

### 7. Event Bus, Session Persistence, Git Integration (Product)

**Current position:** Product. These are well-understood patterns with established best practices. The implementation should be solid and reliable but does not need to be novel.

**Evolution path:** Already converging. Use standard approaches.

**Investment level:** LOW-MEDIUM. Implement well, test thoroughly, but do not innovate here.

### 8. SQLite, Axum, Tokio, DashMap, rust-embed (Commodity)

**Current position:** Commodity. These are mature, well-maintained crates. Use them as-is.

**Evolution path:** Already commoditized. Upgrade when new versions ship.

**Investment level:** ZERO custom work. Use the crates. Contribute bug fixes upstream if found.

---

## Build vs Buy vs Absorb Decisions

### Build (Genesis/Custom components -- our code, our design)

| Component | Rationale |
|-----------|-----------|
| Swim-lane timeline UI | Novel UX; no existing implementation to absorb |
| Safety engine (integrated) | Must be architecturally integrated; cannot bolt on |
| Agent coordination protocol | Coding-specific; no general-purpose framework fits |
| Cost tracker with circuit breaker | Must integrate with safety engine and event bus |
| Permission engine | Coding-specific (file paths, git operations, shell commands) |

### Absorb (Patterns from reference repos -- their design, our implementation)

| Component | Source Repos | What We Absorb |
|-----------|-------------|-----------------|
| Workflow DAG engine | Claude-Code-Workflow, ultrathink | Declarative workflow model, branching/looping patterns |
| Agent presets | claude-prompts, awesome-claude-code, mega-prompt | Prompt engineering patterns, role definitions |
| Session management | claude-code-viewer, sessionmgr | Session data model, export formats, browser UI patterns |
| MCP tools | claude-code-mcp, git-mcp, filesystem-mcp | Tool schemas, capability definitions |
| Git integration | git-worktree-mcp, claude-ci | Worktree management, branch strategies |
| Plugin system | claude-skills, plugin-registry | Plugin discovery, loading, and configuration patterns |
| Hooks and observability | claude-telemetry, agent-dashboard | Event schemas, dashboard layouts, filtering patterns |

### Buy/Use (Commodity -- existing crates, zero custom work)

| Component | Crate(s) | Notes |
|-----------|----------|-------|
| HTTP server | axum 0.8 | Use as-is |
| Async runtime | tokio | Use as-is |
| Database | rusqlite (bundled) | Use as-is; WAL mode for concurrency |
| Serialization | serde, serde_json | Use as-is |
| Concurrent state | dashmap | Use as-is |
| Static embedding | rust-embed | Use as-is |
| WebSocket | tokio-tungstenite | Use as-is |
| CLI parsing | clap | Use as-is |
| UUID generation | uuid | Use as-is |
| Date/time | chrono | Use as-is |
| Git operations | git2 or shell | Evaluate; git2 for library use, shell for simplicity |
| Syntax highlighting | syntect or tree-sitter | Evaluate; syntect is simpler, tree-sitter is more powerful |
| Markdown rendering | pulldown-cmark | Use as-is |

---

## Strategic Moves (Ordered by Priority)

### Move 1: Secure the Genesis Components (Now - Month 3)

**What:** Complete the swim-lane timeline, safety engine, and agent coordination protocol to production quality.

**Why:** These are the components where Forge has a first-mover advantage. Every month of delay is a month for competitors to catch up. Genesis components are the only ones that create lasting differentiation.

**Success criteria:**
- Swim-lane timeline shows real-time concurrent agent activity with cost overlay
- Circuit breaker halts runaway agents within 5 seconds of threshold breach
- Two or more agents can coordinate via handoff and broadcast patterns

### Move 2: Absorb the Reference Repos (Month 2 - Month 6)

**What:** Systematically extract patterns from all 61 reference repos and implement them in Forge. Prioritize by user impact: presets and session management first, then git integration and plugins.

**Why:** The absorption strategy is Forge's primary feature development accelerator. Each absorbed pattern is a feature that would otherwise take weeks to design from scratch. The reference repos have already validated the design with real users.

**Success criteria:**
- 80% of 1,537 identified skills absorbed
- 100+ agent presets shipping out of the box
- Traceability matrix linking every feature to its source repo(s)

### Move 3: Ship MCP Dual-Mode to the Ecosystem (Month 3 - Month 5)

**What:** Ensure every Forge capability is available as an MCP tool with comprehensive schemas, descriptions, and examples. Register Forge in the MCP tool ecosystem directories.

**Why:** MCP ecosystem presence is a distribution channel. Every MCP client that connects to Forge is a user who did not need to discover Forge through marketing. The MCP ecosystem is growing rapidly; being listed early captures attention.

**Success criteria:**
- 50+ MCP tools registered with full schemas
- Listed in major MCP tool directories
- Integration guides for Claude Code, VS Code, and CI/CD

### Move 4: Build Community Around Presets (Month 4 - Month 8)

**What:** Create a contribution model for agent presets and workflow templates. Make it trivially easy for developers to share their configurations as Forge-compatible presets.

**Why:** The preset library is a network effect. More presets attract more users; more users create more presets. This is the virtuous cycle that makes Forge sticky.

**Success criteria:**
- Preset contribution guide published
- 20+ community-contributed presets accepted
- Preset search and tagging in the UI

### Move 5: Enterprise Hardening (Month 6 - Month 12)

**What:** Add SSO/SAML, RBAC, SIEM export, and compliance documentation. Begin enterprise pilot engagements.

**Why:** Enterprise customers provide revenue sustainability and validate the product for the broader market. Enterprise features also benefit teams and advanced solo users.

**Success criteria:**
- 3-5 enterprise pilot customers
- SOC 2 Type 1 compliance documentation
- Enterprise deployment guide

### Move 6: Platform Ecosystem (Month 9 - Month 18)

**What:** Expand the plugin system to support third-party extensions. Create SDKs for common languages. Build a plugin discovery mechanism.

**Why:** Platform ecosystems create defensible moats. When developers have invested in building Forge plugins and workflows, switching costs prevent migration to competitors.

**Success criteria:**
- WASI plugin runtime shipping
- 20+ third-party plugins
- Plugin development SDK and documentation

---

## Inertia and Constraints

### Components with Dangerous Inertia

**SQLite single-node:** Forge's SQLite architecture is perfect for single-developer use but creates scaling constraints for team/enterprise scenarios. Migrating to a distributed database later would be a major architectural change.

**Mitigation:** Design the storage layer with an abstraction boundary. SQLite is the default implementation, but the trait boundary allows future PostgreSQL or distributed alternatives without touching the rest of the codebase.

**Single binary assumption:** The zero-dependency promise constrains future features. Adding computer vision (for screenshot analysis), local model inference, or rich media rendering would balloon the binary size.

**Mitigation:** Core features stay in the binary. Optional capabilities are available as separate processes that Forge can spawn and manage. The plugin system (WASI) handles extensions cleanly.

**Rust rewrite cost:** Absorbing patterns from Python/TypeScript repos requires rewriting them in Rust. This is slower than using them directly.

**Mitigation:** Absorb the *pattern*, not the code. A Python function's logic can be implemented in Rust in 20% of the time it would take to design from scratch. The reference repos provide the design; Rust provides the implementation.

---

## Key Insight: The Evolution Gap

The most important insight from this Wardley Map is the **evolution gap** between where users need to be (multi-agent orchestration, safety, observability) and where most tools are (single-agent, no safety, limited visibility).

```
Where users need to be:    [Multi-Agent] [Safety] [Observability]
                                  |          |          |
                           ------GAP---------GAP-------GAP------
                                  |          |          |
Where most tools are:      [Single Agent] [No Safety] [Logs]
```

Forge's strategy is to bridge this gap completely. Not incrementally. Not one feature at a time. The entire gap, in one binary, with one install.

The competitor who bridges the full gap first wins. Partial bridges (multi-agent without safety, safety without observability, observability without multi-agent) leave users in the gap. Forge is the complete bridge.

---

## Summary of Strategic Position

| Dimension | Position | Action |
|-----------|----------|--------|
| Genesis components | Strong first-mover | Invest aggressively, ship fast |
| Custom components | Good position | Build thoughtfully, absorb from repos |
| Product components | Standard | Use proven patterns, don't innovate |
| Commodity components | Fully leveraged | Use crates as-is, zero custom work |
| Market timing | Favorable | 18-24 month window, act now |
| Competitive position | Unique combination | No single competitor matches all features |
| Biggest risk | Execution speed | Must ship before window closes |
| Biggest advantage | 61-repo absorption | 12-18 month feature head start |
