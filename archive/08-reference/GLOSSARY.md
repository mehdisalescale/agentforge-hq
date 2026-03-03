# Glossary

> All domain, technical, and project-specific terms used in Claude Forge.

---

## Domain Terms

### Agent
A configured instance of an AI coding assistant. An agent has a name, model, system prompt, MCP servers, hooks, and permission settings. Agents are the primary unit of work in Forge. One Forge instance can manage dozens of agents simultaneously.

### Agent Preset
A pre-configured agent definition that can be instantiated with one click. Forge ships with 100+ presets covering domains like code review, testing, security, documentation, and refactoring. Presets are imported from reference repositories.

### Auto-Activation
A skill loading mechanism where skills are automatically engaged when the conversation context matches predefined trigger patterns. For example, a "Dockerfile skill" auto-activates when the user mentions Docker or container-related terms. Sourced from `claude-code-infrastructure-showcase`.

### Bounded Context
A DDD (Domain-Driven Design) concept. A bounded context is a logical boundary within the system where a particular domain model applies. Forge has 10 bounded contexts: Agent Management, Session Lifecycle, Workflow Engine, Observability, Skill Marketplace, Git Operations, Safety and Governance, Configuration, Remote Control, and MCP Foundation.

### CCPI
Claude Code Plugin Installer. A CLI package manager for installing, searching, and updating skills and plugins. Originated in `claude-code-plugins-plus-skills`. Forge absorbs this as the skill install mechanism.

### CLAUDE.md
A Markdown file placed in a project root that provides persistent context to Claude Code across sessions. Functions as a "memory bank" that the agent reads at the start of every conversation. Forge includes an editor for managing CLAUDE.md files.

### Circuit Breaker
A safety mechanism borrowed from electrical engineering and microservices. Prevents runaway agents by tracking consecutive failures. Three states: CLOSED (normal operation), OPEN (halted, rejecting requests), HALF_OPEN (testing recovery with limited calls). Sourced from `ralph-claude-code`.

### Command
A slash command (e.g., `/review`, `/test`, `/deploy`) that triggers a predefined agent action. Commands are shortcuts for complex prompts. Forge supports custom command definitions.

### Contractor Mode
A structured 6-phase development workflow: Plan, Implement, Verify, Review, Fix, Score. Each phase must complete before the next begins. Sourced from `claude-code-my-workflow`.

### DAG (Directed Acyclic Graph)
A graph with directed edges and no cycles. Used in Forge's Workflow Engine to represent task dependencies. Each node is an agent step; edges define execution order. Nodes without incoming dependencies can execute in parallel.

### Dual Exit Gate
A safety mechanism requiring BOTH completion indicators AND an explicit EXIT_SIGNAL before a loop terminates. Prevents premature or missed exits. Sourced from `ralph-claude-code`.

### Event
A structured record of something that happened during an agent session. Events include: messages (user and assistant), tool calls, tool results, errors, status changes, and subagent spawns. Events are persisted to SQLite and streamed via WebSocket.

### Hook
A user-defined script that runs at specific points in the Claude Code lifecycle. There are 13 hook event types (e.g., `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Stop`). Hooks enable custom validation, logging, transformation, and safety enforcement.

### MCP (Model Context Protocol)
An open protocol for AI tools to expose and consume capabilities. Forge uses MCP as its universal integration standard. Every Forge capability is exposed as an MCP tool, and Forge can consume external MCP servers. See the MCP specification for full details.

### Orchestration
The coordination of multiple agents working on related tasks. Orchestration patterns include: sequential (A then B), parallel (A and B simultaneously), validation (A writes, B reviews), and iterative (repeat until quality threshold met).

### Plugin
An extension that adds capabilities to an agent. In the Claude Code ecosystem, 98% of plugins are AI instruction files (Markdown that shapes agent behavior) and 2% are MCP servers (runtime tools). Forge treats both types uniformly.

### Prompt Preset
A reusable prompt template that can be applied to any agent. Examples: "Review this code for security vulnerabilities", "Refactor this function for readability". Forge ships with 69+ presets from `claude-code-skills`.

### Session
A single conversation between a user and an agent. Sessions have a unique ID, a start time, a sequence of events, and a status (active, completed, errored). Sessions can be resumed, exported, and searched.

### Skill
A self-contained capability that an agent can invoke. Defined in SKILL.md format with frontmatter metadata (name, category, triggers, dependencies). Skills range from simple (code formatting) to complex (multi-step deployment). Forge's Skill Marketplace hosts 1,500+ skills.

### SKILL.md
The standardized format for defining agent skills. Includes YAML frontmatter (name, category, triggers, quality score) followed by Markdown content (instructions, examples). Originated in `claude-code-plugins-plus-skills`.

### Subagent
An agent spawned by another agent to handle a subtask. Subagents inherit context from their parent and return results when complete. Forge tracks subagent lifecycles and displays them in the observability swim-lane view.

### Swim-Lane View
A visualization showing parallel agent activity over time. Each agent gets a vertical column (lane); events appear as blocks positioned vertically by timestamp. Enables at-a-glance understanding of multi-agent coordination. Sourced from `hooks-observability`.

### Todo
A structured task extracted from agent sessions via the TodoWrite tool. Todos have a status (pending, in-progress, completed) and are aggregated across sessions for a project-level task view.

### Workflow
A defined sequence of agent steps with dependencies, conditions, and parallel groups. Workflows are the primary mechanism for multi-agent coordination in Forge. Represented as DAGs and edited visually.

### Worktree
A Git worktree -- a separate working directory linked to the same repository. Forge uses worktrees to isolate agent work: each agent can have its own worktree, preventing file conflicts between concurrent agents. Sourced from `1code`.

---

## Technical Terms

### ADR (Architecture Decision Record)
A document recording a significant architectural decision, the context that led to it, the alternatives considered, and the consequences. Forge uses ADRs for all decisions affecting public APIs, database schema, or MCP tools.

### Axum
A Rust web framework built on top of `hyper` and `tokio`. Forge's HTTP server, WebSocket server, and API endpoints are all built with Axum 0.8.

### Broadcast Channel
A `tokio::sync::broadcast` channel that allows one sender to distribute messages to multiple receivers. Forge uses broadcast channels for real-time event streaming from the backend to all connected WebSocket clients.

### Command Palette
A keyboard-triggered overlay (Ctrl/Cmd+K) that provides fuzzy search across all searchable content: pages, agents, sessions, skills, workflows, settings, and actions. Inspired by VS Code's Ctrl+P and Raycast.

### DashMap
A concurrent hash map for Rust (`dashmap` crate). Forge uses DashMap for in-memory state that is accessed from multiple async tasks (agent registry, session state, circuit breaker state).

### FTS5
Full-Text Search 5, a SQLite extension for high-performance full-text search. Forge uses FTS5 to index session content, skill descriptions, and event data. Supports tokenized search, prefix queries, phrase matching, and result ranking.

### JSON-RPC
A remote procedure call protocol encoded in JSON. Used by MCP for tool invocations. A JSON-RPC message has: method, params, id, and either result or error.

### MCP Server
A process that exposes tools and resources via the Model Context Protocol. Forge itself is an MCP server (exposing 50+ tools) and can connect to external MCP servers (for additional capabilities like filesystem access, web search, or database queries).

### MCP Tool
A single capability exposed via MCP. Defined by a name, description, and JSON Schema for input validation. Example: `forge_create_agent`, `forge_search_sessions`, `forge_run_workflow`.

### MCP Resource
A data source exposed via MCP for AI models to read. Resources are identified by URIs and return structured content. Example: `forge://agents/list`, `forge://sessions/{id}/transcript`.

### Progressive Disclosure
A UI design pattern where information is revealed incrementally. Primary information is always visible; secondary appears on hover; tertiary appears on click. Prevents information overload while maintaining information density.

### PTY (Pseudo-Terminal)
A software interface that emulates a hardware terminal. Forge uses PTY to provide a browser-based terminal via WebSocket. The browser sends keystrokes to the server, which passes them to the PTY; output flows back via WebSocket.

### Pulse Chart
A real-time activity visualization showing horizontal bars per agent over time. Bar height or color intensity represents activity level. Enables quick identification of which agents are busy, idle, or stalled. Sourced from `hooks-observability`.

### Rate Limiter
A mechanism that limits the frequency of operations. Forge supports configurable rate limits per agent (e.g., 100 calls/hour) to prevent runaway token consumption. Implemented as a token bucket algorithm.

### Rune
Svelte 5's reactivity primitive. Runes are compiler directives that create reactive state: `$state` (reactive variable), `$derived` (computed value), `$effect` (side effect), `$props` (component inputs). Forge uses runes exclusively for all reactive state management.

### rust-embed
A Rust crate that embeds static files (the compiled Svelte frontend) directly into the Rust binary at compile time. This is how Forge achieves single-binary deployment: the entire web UI is inside the executable.

### rusqlite
A Rust binding for SQLite. Forge uses rusqlite with the "bundled" feature (SQLite compiled into the binary) and WAL mode for concurrent read/write access.

### SemVer (Semantic Versioning)
Versioning scheme: MAJOR.MINOR.PATCH. MAJOR for breaking changes, MINOR for new features, PATCH for bug fixes. Forge follows SemVer for all releases.

### SSE (Server-Sent Events)
A protocol for server-to-client event streaming over HTTP. Some reference repos use SSE; Forge uses WebSocket instead for bidirectional communication.

### Svelte 5
The frontend framework used by Forge. Svelte 5 introduces runes (compiler-based reactivity), replacing Svelte 4's store-based approach. Forge uses SvelteKit with `adapter-static` for static site generation, embedded into the Rust binary.

### TailwindCSS 4
A utility-first CSS framework. Forge uses TailwindCSS 4 for all styling. Components are styled with utility classes rather than custom CSS.

### Topological Sort
An algorithm that orders nodes in a DAG such that for every directed edge (u, v), node u comes before node v. Used by the Workflow Engine to determine execution order of workflow steps.

### WAL (Write-Ahead Logging)
A SQLite journaling mode that allows concurrent readers and a single writer. Forge uses WAL mode for all SQLite operations, enabling the WebSocket event stream to read from the database while the batch writer is writing.

### WASI (WebAssembly System Interface)
A system interface for running WebAssembly outside the browser. Potential future use in Forge for running sandboxed plugins.

### WebSocket
A protocol providing full-duplex communication over a single TCP connection. Forge uses WebSocket for real-time event streaming from backend to frontend, and for the terminal PTY interface.

---

## Abbreviations

| Abbreviation | Expansion |
|-------------|-----------|
| ADR | Architecture Decision Record |
| API | Application Programming Interface |
| CCPI | Claude Code Plugin Installer |
| CI/CD | Continuous Integration / Continuous Deployment |
| CLI | Command-Line Interface |
| CORS | Cross-Origin Resource Sharing |
| CRUD | Create, Read, Update, Delete |
| DAG | Directed Acyclic Graph |
| DDD | Domain-Driven Design |
| FSM | Finite State Machine |
| FTS | Full-Text Search |
| GUI | Graphical User Interface |
| HTTP | Hypertext Transfer Protocol |
| IDE | Integrated Development Environment |
| JSON | JavaScript Object Notation |
| JWT | JSON Web Token |
| LOC | Lines of Code |
| LLM | Large Language Model |
| MCP | Model Context Protocol |
| MVP | Minimum Viable Product |
| ORM | Object-Relational Mapping |
| OWASP | Open Web Application Security Project |
| P90/P99 | 90th/99th percentile |
| PR | Pull Request |
| PTY | Pseudo-Terminal |
| PWA | Progressive Web App |
| REST | Representational State Transfer |
| RPI | Read-Plan-Implement (workflow pattern) |
| RPM | Requests Per Minute |
| SemVer | Semantic Versioning |
| SPA | Single Page Application |
| SQL | Structured Query Language |
| SRP | Single Responsibility Principle |
| SSE | Server-Sent Events |
| TDD | Test-Driven Development |
| TLS | Transport Layer Security |
| TTS | Text-to-Speech |
| UI/UX | User Interface / User Experience |
| URI | Uniform Resource Identifier |
| UUID | Universally Unique Identifier |
| WAL | Write-Ahead Logging |
| WASI | WebAssembly System Interface |
| WS | WebSocket |
| YAML | YAML Ain't Markup Language |
