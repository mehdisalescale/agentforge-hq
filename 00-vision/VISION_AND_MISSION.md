# Claude Forge: Vision and Mission

## Vision

**Every developer has a single, intelligent command center that orchestrates all their coding agents, tools, and workflows -- safely, transparently, and without configuration.**

Claude Forge is the control plane for the agentic coding era: one binary that replaces sixty-one fragmented tools with a unified platform where humans and AI agents collaborate on code with full observability, safety guarantees, and zero setup cost.

## Mission

**Build the definitive open-source agentic coding platform -- a single Rust binary embedding a complete UI, MCP server, workflow engine, and safety layer -- that absorbs the best patterns from the entire Claude Code ecosystem and makes multi-agent software development accessible, safe, and observable for every developer.**

We do this by:
1. Consolidating 62 reference repositories (~200K+ LOC of proven patterns) into one cohesive tool
2. Serving three interfaces from one binary: embedded web UI, MCP server, and CLI
3. Making safety and observability first-class concerns, not afterthoughts
4. Treating MCP as the universal integration standard, not a proprietary protocol

---

## Core Beliefs

### 1. The Future of Coding Is Collaborative Multi-Agent Systems

Individual AI coding assistants are a transitional form. The steady state is teams of specialized agents -- a code writer, a reviewer, a tester, a security auditor, a deployment agent -- working in coordinated workflows. Developers will spend more time directing and reviewing agent work than writing code character by character. Gartner's 1,445% surge in multi-agent inquiries is a leading indicator, not an anomaly.

### 2. Fragmentation Is the Primary Barrier to Adoption

The Claude Code ecosystem has 62 repositories solving overlapping problems in incompatible ways. A developer who wants multi-agent orchestration AND session management AND git integration AND safety hooks must learn, install, configure, and maintain a half-dozen tools. This fragmentation means most developers use none of them. The biggest unlock is not inventing new capabilities but unifying existing ones.

### 3. Safety Must Be Structural, Not Optional

When AI agents can execute arbitrary code, modify files, make API calls, and push to repositories, safety cannot be a flag that users toggle off when it is inconvenient. Safety must be architectural: circuit breakers that halt runaway agents, permission boundaries that limit blast radius, audit trails that make every action traceable, and cost controls that prevent $500 API bills from a forgotten loop. The safe path must also be the easy path.

### 4. Observability Changes Behavior

Developers who can see what their agents are doing -- in real time, with structured event streams and visual timelines -- make fundamentally better decisions about when to intervene, what to delegate, and how to improve their prompts. The swim-lane view of concurrent agent activity is not a nice-to-have dashboard; it is the core interaction model that makes multi-agent systems usable by humans.

### 5. A Single Binary Is a Product Decision, Not a Technical One

Zero-dependency deployment removes the entire category of "installation and setup" problems. When `cargo install claude-forge` gives you a working server with embedded UI, SQLite persistence, MCP endpoints, and 100+ agent presets -- with no Docker, no Node runtime, no database server, no reverse proxy -- you have removed the single largest source of friction in developer tool adoption. Every dependency you add is a user you lose.

### 6. MCP Is the Integration Standard, Not a Feature

The Model Context Protocol is not something Forge "supports." It is the architecture. Every capability Forge exposes internally is also available as an MCP tool. This means Forge works standalone (Direction A) and as a tool that Claude Code, VS Code, CI/CD pipelines, or any MCP client can invoke (Direction B). Dual-mode operation is not an extra; it is the design.

### 7. Open Source Is a Moat, Not a Charity

In a market where trust is the scarcest resource -- developers are being asked to let AI agents write and execute code in their production repositories -- open source is the only viable trust model. Every line of Forge is auditable. Every safety mechanism is verifiable. Every data path is transparent. This is not altruism; it is the only strategy that works when your product has root access to someone's codebase.

---

## The Problem We Solve

### The Fragmentation Crisis

The Claude Code ecosystem is a case study in productive chaos. Sixty-one repositories have emerged, representing over 200,000 lines of code, building solutions for:

| Category | Repos | Examples |
|----------|-------|----------|
| Multi-agent orchestration | 8 | Claude-Code-Workflow, swarm-mcp, ultrathink |
| Session & project management | 6 | claude-code-viewer, sessionmgr, claude-crew |
| MCP servers & tools | 12 | claude-code-mcp, git-mcp, filesystem-mcp |
| Safety & governance | 5 | claude-code-sandbox, permission-mcp, audit-trail |
| IDE & UI integration | 9 | 1code, claudedev, claude-desktop-plus |
| Prompts & presets | 7 | claude-prompts, awesome-claude-code, mega-prompt |
| Git & DevOps | 6 | git-worktree-mcp, claude-ci, deploy-agent |
| Monitoring & observability | 4 | claude-telemetry, agent-dashboard, stream-viewer |
| Configuration & skills | 4 | claude-skills, plugin-registry, config-manager |

Each solves a real problem. None solves the whole problem. A developer who wants to:
- Run multiple agents in coordinated workflows
- See their activity in real time
- Control costs and permissions
- Manage git branches and worktrees per agent
- Review and approve agent actions before execution
- Export sessions for team review

...must today install 5-8 separate tools, learn 5-8 different configuration formats, manage 5-8 different processes, and hope they do not conflict.

### The Integration Tax

The hidden cost is not just installation. It is the ongoing integration tax:
- Config drift between tools that overlap
- Version incompatibilities after updates
- No shared event model (each tool logs differently)
- No unified permission model (each tool has its own safety story)
- No single source of truth for "what happened in this coding session"

Forge eliminates this tax entirely by absorbing the proven patterns from all 62 repos into a single, coherent system.

---

## Why Now

### 1. MCP Has Reached Critical Mass (2025-2026)

The Model Context Protocol went from proposal to de facto standard in under 18 months. Anthropic, OpenAI, Google, and dozens of tool vendors now support it. For the first time, there is a universal way for AI tools to expose and consume capabilities. Forge's MCP-first architecture rides this wave rather than fighting it.

### 2. Multi-Agent Systems Have Crossed the Chasm

Gartner's 1,445% increase in multi-agent inquiries, LangGraph's 400+ production deployments, and the proliferation of orchestration frameworks all signal that multi-agent is moving from "research curiosity" to "how we build software." But most multi-agent frameworks are Python-based, cloud-dependent, and designed for general AI workflows -- not specifically for coding. Forge is purpose-built for the coding domain.

### 3. Cost Optimization Is Now a Requirement

The era of "throw tokens at it" is ending. Organizations are discovering that uncontrolled AI agent usage can generate API bills in the thousands of dollars per developer per month. Circuit breakers, cost tracking, rate limiting, and intelligent prompt caching are no longer nice-to-haves. Forge builds these into the foundation.

### 4. The Rust + WASM Moment

Rust's ecosystem for web servers (Axum), databases (rusqlite), and frontend embedding (rust-embed) has matured to the point where a single compiled binary can serve a full web application with real-time WebSocket streaming, SQLite persistence, and sub-millisecond response times. Two years ago, this would have required Go or a polyglot stack.

### 5. Trust Deficit in AI Tooling

High-profile incidents of AI agents causing damage -- deleting files, pushing broken code, leaking secrets -- have created a market demand for verifiable safety. The tools that win will be the ones that can prove they are safe, not just promise it. Open source + structural safety is the answer.

---

## Success Metrics

### 6-Month Horizon (Mid-2026)

| Metric | Target | Rationale |
|--------|--------|-----------|
| GitHub stars | 2,000+ | Indicates developer awareness and interest |
| Monthly active installs | 500+ | Developers who actually use it |
| Features absorbed from reference repos | 80% of 1,537 identified skills | Core value proposition is consolidation |
| Lines of Rust code | ~33,000 | Complete platform implementation |
| Agent presets shipped | 100+ | Out-of-box value without configuration |
| MCP tools exposed | 50+ | Comprehensive MCP server surface |
| P0 bugs in safety engine | 0 | Safety is non-negotiable |
| Mean time to first agent run | < 60 seconds | Install, launch, run -- no config |

### 1-Year Horizon (Early 2027)

| Metric | Target | Rationale |
|--------|--------|-----------|
| GitHub stars | 10,000+ | Top-tier developer tool recognition |
| Monthly active installs | 5,000+ | Growing user base |
| Contributing developers | 50+ | Sustainable open-source community |
| Enterprise pilot customers | 3-5 | Revenue validation |
| Plugin ecosystem contributions | 20+ third-party plugins | Platform extensibility proven |
| Conference talks / blog posts about Forge | 10+ | Community advocacy |
| Uptime for hosted Forge instances | 99.9% | Production reliability |
| Agent workflow templates | 500+ | Rich template ecosystem |

### 3-Year Horizon (2029)

| Metric | Target | Rationale |
|--------|--------|-----------|
| GitHub stars | 50,000+ | Industry-standard tool |
| Monthly active installs | 50,000+ | Broad adoption |
| Enterprise customers | 50+ | Sustainable business |
| Languages / frameworks supported | All major ones | Universal applicability |
| Market share of agentic coding platforms | Top 3 | Competitive positioning |
| Community-maintained presets | 1,000+ | Self-sustaining ecosystem |
| Forge-native CI/CD integrations | 10+ platforms | DevOps standard |

---

## Non-Goals

We are explicit about what Forge does **not** do, to maintain focus and avoid scope creep.

### 1. We Are Not Building an LLM

Forge orchestrates AI agents; it does not train, fine-tune, or host language models. We consume models via API (Anthropic, OpenAI, local models via compatible APIs). We will never require users to download model weights.

### 2. We Are Not Replacing IDEs

Forge is a complement to VS Code, Neovim, JetBrains, and other editors -- not a replacement. The embedded UI is for agent orchestration, monitoring, and review. We do not build a code editor, syntax-aware autocomplete, or language server. We integrate with editors via MCP.

### 3. We Are Not a General-Purpose AI Orchestrator

LangGraph, CrewAI, and AutoGen handle arbitrary AI workflows (customer support, data analysis, content generation). Forge is purpose-built for software development. Every feature, preset, and safety mechanism is designed for the coding domain. This specialization is our advantage.

### 4. We Are Not a Cloud Service (By Default)

Forge runs locally by default. Your code never leaves your machine unless you explicitly configure remote MCP endpoints or cloud model APIs. We will offer optional hosted features (shared dashboards, team coordination) but the core product is local-first.

### 5. We Are Not Chasing Feature Parity with Every Tool

Absorbing patterns from 62 repos does not mean replicating every feature of every repo. We absorb the *best* patterns -- the ones that serve the most users for the most common workflows. Niche features that serve < 5% of use cases are candidates for plugins, not core features.

### 6. We Are Not Building a Package Manager

Forge has a plugin system, but we are not building npm/crates.io for agent tools. Plugins are local configurations (CLAUDE.md, MCP configs, hook definitions) that can be shared as files or git repos. We do not host a registry, manage versions, or resolve dependency trees.

### 7. We Are Not Optimizing for Benchmarks

We care about real-world developer productivity, not synthetic benchmarks. We will not add features solely to score well on SWE-bench or similar evaluations. Our metrics are: time to first useful result, developer satisfaction, and safety incident rate.

---

## The North Star

When a developer installs Forge, they should feel like they just hired a team of expert coding assistants who already know how to work together, who report their progress transparently, who ask permission before doing anything dangerous, and who never send a surprise API bill.

That is the product we are building.
