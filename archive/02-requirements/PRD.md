# Claude Forge -- Product Requirements Document

**Version**: 1.0
**Date**: 2026-02-25
**Status**: Draft
**Authors**: Claude Forge Core Team

---

## Table of Contents

1. [Product Overview](#1-product-overview)
2. [Objectives and Key Results](#2-objectives-and-key-results)
3. [Functional Requirements](#3-functional-requirements)
4. [Non-Functional Requirements](#4-non-functional-requirements)
5. [Constraints and Assumptions](#5-constraints-and-assumptions)
6. [Dependencies](#6-dependencies)

---

## 1. Product Overview

### 1.1 Vision

Claude Forge is the ultimate agentic coding platform -- a single Rust binary that absorbs the capabilities of 62 reference repositories in the Claude Code ecosystem into one unified tool. It operates in two modes simultaneously:

- **Direction A (Platform)**: A complete agentic coding environment with a Rust/Axum backend and embedded Svelte 5 UI, served as a single binary with no external dependencies.
- **Direction B (MCP Server)**: A Model Context Protocol server that exposes all platform capabilities as tools, resources, and prompts -- enabling any other AI system to use Forge as a meta-agentic building block.

### 1.2 Problem Statement

The Claude Code ecosystem contains 62 community repositories, each solving a narrow slice of the agentic coding problem. Developers must discover, evaluate, install, and integrate multiple tools to get:

- Agent orchestration with safety guardrails
- Session management with search and scheduling
- Skills and plugins with quality grading
- Git integration with worktree isolation
- Real-time observability across multi-agent runs
- Remote notifications and CI/CD integration

No single tool provides all of these. Claude Forge unifies them.

### 1.3 Target Users

| Persona | Description |
|---------|-------------|
| Solo AI Developer | Uses Forge as their primary agentic coding tool |
| Team Lead | Manages agents, monitors costs, enforces policies |
| Tool Builder | Integrates Forge via MCP into custom systems |
| DevOps / CI Engineer | Runs Forge in pipelines for automated PR review |
| Open Source Contributor | Extends Forge with plugins, skills, and agents |

See [USER_PERSONAS.md](USER_PERSONAS.md) for detailed persona profiles.

### 1.4 Product Principles

1. **Single binary, zero dependencies**: Download one file, run it, done.
2. **Absorb, do not wrap**: Features are native Rust/Svelte, not shelled-out to other tools.
3. **Safety by default**: Circuit breaker, rate limiter, and exit gate are always active.
4. **MCP-native**: Every feature is accessible both via UI and via MCP protocol.
5. **Offline-first**: SQLite (WAL mode) for all persistence; no cloud required.
6. **Ecosystem compatibility**: Reads existing `CLAUDE.md`, `settings.json`, `SKILL.md`, and `.ralphrc` files.

### 1.5 Current State

- ~4,700 lines of code (Rust + Svelte)
- Agent CRUD with 9 presets
- Per-prompt process spawning with `--resume` session continuity
- Real-time WebSocket event streaming with rich rendering
- SQLite persistence (batch writes every 50 events or 2 seconds)
- Agent edit page, directory picker, export (JSON/Markdown)
- CLAUDE.md editor, MCP server editor, hooks editor
- Multi-pane tab layout with split view

### 1.6 Target State

- ~33,000 lines of code
- 200+ features absorbed from 62 reference repositories
- 12 bounded contexts fully implemented
- Dual-mode operation: embedded UI + MCP server
- Production-grade reliability: circuit breaker, rate limiter, file protection

---

## 2. Objectives and Key Results

### OKR 1: Complete Feature Absorption

**Objective**: Absorb all critical features from the 62-repository ecosystem into a single binary.

| Key Result | Metric | Target |
|------------|--------|--------|
| KR1.1 | P0 features implemented | 100% |
| KR1.2 | P1 features implemented | 90% |
| KR1.3 | Reference repos with zero features absorbed | 0 |
| KR1.4 | Total features in catalog | 200+ |

### OKR 2: Production-Grade Reliability

**Objective**: Ensure Forge is safe to run unattended for hours without human intervention.

| Key Result | Metric | Target |
|------------|--------|--------|
| KR2.1 | Circuit breaker prevents runaway agents | 100% of detected loops |
| KR2.2 | Rate limiter enforces budget constraints | Zero budget overruns |
| KR2.3 | Dual exit gate catches false completion | 99.5%+ accuracy |
| KR2.4 | File protection prevents critical file modification | Zero incidents |

### OKR 3: Developer Experience

**Objective**: Make Forge the fastest path from idea to working code.

| Key Result | Metric | Target |
|------------|--------|--------|
| KR3.1 | Time from download to first agent run | < 30 seconds |
| KR3.2 | Session search latency (FTS) | < 100ms for 10K sessions |
| KR3.3 | Skill installation time | < 5 seconds |
| KR3.4 | UI response time (P95) | < 200ms |

### OKR 4: Ecosystem Integration

**Objective**: Position Forge as the universal integration point for agentic coding tools.

| Key Result | Metric | Target |
|------------|--------|--------|
| KR4.1 | MCP tools exposed | 30+ |
| KR4.2 | MCP resources exposed | 15+ |
| KR4.3 | Notification channels supported | 5+ (webhook, Telegram, Discord, email, desktop) |
| KR4.4 | CI/CD providers with official support | 3+ (GitHub Actions, GitLab CI, Jenkins) |

### OKR 5: Community Growth

**Objective**: Build a sustainable ecosystem of community contributions.

| Key Result | Metric | Target |
|------------|--------|--------|
| KR5.1 | Skills available in catalog | 1,500+ (imported + community, from ecosystem analysis) |
| KR5.2 | Plugin types supported | 3 (WASM, MCP, native) |
| KR5.3 | Agent presets shipped | 100+ |
| KR5.4 | Documentation coverage | 100% of public APIs |

---

## 3. Functional Requirements

### 3.1 Agent Management

**Bounded Context**: Core agent lifecycle -- creation, configuration, execution, monitoring, and coordination.

**Sources**: 1code, ralph-claude-code, claude-code-subagents, claude-code-sub-agents, claude-code-agents, claude_code_bridge

#### FR-AM-001: Agent CRUD

Forge shall provide full create, read, update, and delete operations for agent definitions.

- Each agent has: name, model, system prompt, allowed tools, working directory, environment variables, MCP servers, hooks, and metadata.
- Agents persist in SQLite and survive restarts.
- Agent definitions are exportable as JSON and importable from JSON.

#### FR-AM-002: Agent Presets

Forge shall ship with 100+ pre-configured agent presets organized by domain.

- Presets cover 15+ categories: languages, web, mobile, databases, ORMs, infrastructure, services, messaging, testing, ML, monitoring, security, build tools, DevOps, and documentation.
- Users can create custom presets from existing agents.
- Presets include system prompt, recommended tools, and suggested MCP servers.

#### FR-AM-003: Multi-CLI Orchestration

Forge shall support spawning and managing processes for multiple AI CLI tools.

- Supported CLIs: Claude Code, Codex, Gemini CLI, Qwen, and custom binaries.
- CLI selection can be manual, per-agent, or automatic via semantic routing.
- Each CLI is abstracted behind a common process interface (spawn, stream events, kill).

#### FR-AM-004: Agent Teams

Forge shall support grouping agents into teams with defined roles.

- Team patterns: Builder + Validator, Lead + Workers, Pipeline (sequential handoff).
- Teams share a coordination context visible in the observability dashboard.
- Team membership is stored in SQLite with a many-to-many relation.

#### FR-AM-005: Agent Handoff

Forge shall support explicit handoff of context between agents.

- An agent can pass structured context (files changed, decisions made, open questions) to another agent.
- Handoff preserves session history for the receiving agent.
- Handoff can be triggered via UI, via API, or automatically when a workflow step completes.

#### FR-AM-006: Automatic Agent Invocation

Forge shall support automatic agent invocation based on context.

- Agents can declare trigger phrases or file patterns that cause them to be invoked.
- @mention syntax (e.g., `@security-reviewer`) invokes a specific agent.
- Auto-invocation respects rate limits and circuit breaker state.

#### FR-AM-007: Agent Process Management

Forge shall manage agent processes with full lifecycle control.

- Operations: start, pause (SIGSTOP), resume (SIGCONT), kill (SIGTERM then SIGKILL).
- Process output is captured as a stream-json event stream.
- Session continuity via `--resume` flag with configurable timeout (default 24 hours).
- Environment isolation: `env_remove("ANTHROPIC_API_KEY")` when `use_max=true`; `env_remove("CLAUDECODE")` to avoid nested-session guard.

#### FR-AM-008: Model Routing

Forge shall support intelligent model routing based on task characteristics.

- Scenario-based routing: background tasks, deep thinking, long context, web search.
- Router rules configurable per-agent or globally.
- Custom router scripts for advanced selection logic.
- Multi-provider support: Anthropic direct, AWS Bedrock, Google Vertex, OpenRouter.

---

### 3.2 Safety and Reliability

**Bounded Context**: Preventing runaway agents, wasted tokens, stuck loops, and unauthorized file modifications.

**Sources**: ralph-claude-code, claude-code-hub

#### FR-SR-001: Circuit Breaker

Forge shall implement a 3-state circuit breaker for every agent process.

- States: CLOSED (normal), OPEN (tripped -- agent halted), HALF_OPEN (testing recovery).
- Trips on: consecutive failures exceeding threshold, loop detection, error rate exceeding threshold.
- Auto-recovery: after configurable cooldown, transitions to HALF_OPEN and allows one probe request.
- State transitions are logged as events and visible in the observability dashboard.

#### FR-SR-002: Rate Limiter

Forge shall enforce configurable rate limits on agent API calls.

- Default: 100 calls per hour per agent (configurable).
- Additional limits: RPM (requests per minute), concurrent sessions, 5-hour budget window.
- Countdown timers visible in UI when approaching limits.
- Agents are paused (not killed) when rate limited, and auto-resume when the window resets.

#### FR-SR-003: Dual-Condition Exit Gate

Forge shall require two independent signals before considering an agent run complete.

- Signal 1: Completion indicators in agent output (semantic analysis).
- Signal 2: Explicit EXIT_SIGNAL token in the response.
- If only one signal is present, the agent continues with a clarification prompt.
- Exit gate is configurable per agent (strict, normal, relaxed modes).

#### FR-SR-004: Response Analyzer

Forge shall analyze agent responses in real-time for problematic patterns.

- Detection of: loops (repeated output), errors (stack traces, error codes), stuck states (no progress), and hallucinations (references to nonexistent files).
- Configurable thresholds for each pattern type.
- Actions on detection: warn, pause, kill, or escalate to human.

#### FR-SR-005: File Protection

Forge shall protect designated files and directories from agent modification.

- Protected paths configurable per-project and globally.
- Default protection for: `.env`, `credentials.json`, `.git/config`, CI configuration files.
- Protection enforced at the hook level (pre-write validation).
- Violations are logged and optionally trigger notifications.

#### FR-SR-006: Pre-Loop Integrity Validation

Forge shall validate the environment before starting an agent loop.

- Checks: working directory exists, required tools are available, Git state is clean (if required), configuration is valid.
- Failed validation prevents agent start and reports specific issues.

#### FR-SR-007: Budget Controls

Forge shall enforce cost budgets per agent, per team, and globally.

- Budget can be expressed in dollars, tokens, or API calls.
- Soft limit: warning notification at configurable threshold (default 80%).
- Hard limit: agent is paused at 100% of budget.
- Cost tracking uses model-specific pricing tables.

#### FR-SR-008: Fail-Open Degradation

Forge shall continue operating when optional subsystems are unavailable.

- If notification service is down: log locally, retry later.
- If MCP server is unreachable: agent continues without that tool.
- Degradation events are logged with severity.

---

### 3.3 Workflows

**Bounded Context**: Multi-step task execution with dependency management, parallelism, and templates.

**Sources**: Claude-Code-Workflow, claude-code-workflows, claude-code-spec-workflow, claude-code-development-kit

#### FR-WF-001: 4-Level Workflow Engine

Forge shall support workflows at four complexity levels.

| Level | Name | Description | Typical Duration |
|-------|------|-------------|------------------|
| L1 | lite-lite-lite | Quick single-step tasks | Seconds |
| L2 | lite-lite | Simple multi-step tasks | Minutes |
| L3 | lite | Standard development workflows | Minutes to hours |
| L4 | brainstorm | Deep exploration with multiple agents | Hours |

- Level selection can be automatic (based on prompt analysis) or manual.
- Each level has different defaults for parallelism, retry policy, and timeout.

#### FR-WF-002: DAG-Based Task Execution

Forge shall execute workflow steps as a directed acyclic graph (DAG).

- Steps declare dependencies on other steps.
- Independent steps execute in parallel up to a configurable concurrency limit.
- Failed steps can be retried, skipped, or cause the workflow to abort.
- DAG state is persisted to SQLite for crash recovery.

#### FR-WF-003: Workflow Templates

Forge shall provide a library of pre-built workflow templates.

- Templates for: feature implementation, bug fix, refactoring, code review, documentation, testing, release preparation, and security audit.
- Templates are parameterized (accept variables like branch name, file paths, issue description).
- Users can create custom templates from completed workflows.

#### FR-WF-004: Workflow State Persistence

Forge shall persist workflow state for crash recovery and inspection.

- State stored in SQLite with JSON workflow definitions.
- Incomplete workflows can be resumed after restart.
- Completed workflows are queryable for analytics and auditing.

#### FR-WF-005: Semantic CLI Selection

Forge shall automatically select the best CLI tool for each workflow step.

- Users describe intent in natural language; Forge routes to Claude, Codex, Gemini, or Qwen.
- Selection criteria: task type, model strengths, context window requirements, cost.
- Override available per step.

#### FR-WF-006: Workflow Visualization

Forge shall display workflow execution as an interactive graph.

- Nodes represent steps; edges represent dependencies.
- Node color indicates status: pending, running, succeeded, failed, skipped.
- Click a node to view its agent output, duration, and cost.

#### FR-WF-007: Issue Workflow

Forge shall support post-development maintenance workflows.

- Parse GitHub/GitLab issues into structured task descriptions.
- Auto-generate workflow from issue labels and description.
- Report results back to the issue as comments.

---

### 3.4 Session Management

**Bounded Context**: Browsing, searching, resuming, exporting, and scheduling agent sessions.

**Sources**: claude-code-viewer, 1code, claude-code-telegram

#### FR-SM-001: Session Browser

Forge shall provide a browsable, searchable interface to all sessions.

- Sessions are loaded from `~/.claude/projects/` directory structure.
- Display: session ID, project, start time, duration, message count, cost, status.
- Sort by: date, duration, cost, message count.
- Filter by: project, date range, status (active, completed, failed).

#### FR-SM-002: Full-Text Session Search

Forge shall support full-text search across all session content.

- FTS index built on session messages, tool calls, and agent output.
- Fuzzy matching with Cmd+K / Ctrl+K keyboard shortcut.
- Search results show matching message with highlighted context.
- Latency target: < 100ms for 10,000 sessions.

#### FR-SM-003: Session Resume

Forge shall support resuming any previous session.

- Resume uses `--resume` flag with the session ID.
- Session context is restored including conversation history and working directory.
- Configurable session timeout (default 24 hours; after which a new session starts).

#### FR-SM-004: Session Export

Forge shall export sessions in multiple formats.

- Formats: JSON (full fidelity), Markdown (human-readable), HTML (styled).
- Export can include or exclude: tool calls, system messages, timing data, cost data.
- Bulk export with date range and project filters.

#### FR-SM-005: Todo Extraction

Forge shall extract and display TodoWrite items from sessions.

- Parse `TodoWrite` events from the stream-json event stream.
- Display todos in a dedicated panel with status (pending, in-progress, done).
- Todos are linked back to the session and message that created them.

#### FR-SM-006: Cron Scheduler

Forge shall support scheduling agent runs with cron expressions.

- Standard cron syntax (minute, hour, day, month, weekday).
- Each scheduled job specifies: agent, prompt, working directory, and notification preferences.
- Job history with success/failure status.
- Jobs persist across restarts.

#### FR-SM-007: Session Kanban View

Forge shall display sessions as cards on a Kanban board.

- Columns: Queued, Running, Paused, Completed, Failed.
- Cards show: agent name, project, elapsed time, message count.
- Drag-and-drop to reorder queue priority.

#### FR-SM-008: Session Cost Tracking

Forge shall track and display cost data for every session.

- Cost calculated from model-specific pricing tables and token counts.
- Displayed per-session, per-agent, per-project, and in aggregate.
- Historical cost charts with daily/weekly/monthly views.

---

### 3.5 Skills

**Bounded Context**: Discovering, installing, managing, and auto-activating reusable agent skills.

**Sources**: claude-code-plugins-plus-skills, claude-code-skills, claude-code-skill-factory, claude-code-templates

#### FR-SK-001: Skill Catalog

Forge shall maintain a catalog of all available skills.

- Ships with 1,500+ imported skills from ecosystem analysis.
- Catalog displays: name, category, description, author, quality score, install count.
- Categories: DevOps, Security, ML, Data, API, Testing, Documentation, Web, Mobile, Database, Infrastructure, Monitoring, Build Tools, Languages, and General.
- Catalog is searchable by name, category, and keyword.

#### FR-SK-002: Skill Installation

Forge shall support installing skills from catalog and external sources.

- Install from catalog: one-click in UI or `forge skill install <name>` via CLI.
- Install from URL: GitHub repository, raw URL, or local path.
- Skills are stored in `~/.claude-forge/skills/` with metadata in SQLite.
- Installation time target: < 5 seconds.

#### FR-SK-003: Skill Search

Forge shall provide multi-modal skill search.

- Text search across skill names, descriptions, and content.
- Category filtering.
- Tag-based filtering.
- Relevance ranking by quality score and install count.

#### FR-SK-004: Auto-Activation

Forge shall automatically activate relevant skills based on conversation context.

- Skills declare trigger phrases, file patterns, or project types.
- When context matches, the skill is injected into the agent's system prompt.
- Auto-activation can be enabled/disabled per agent.
- Maximum concurrent auto-activated skills is configurable (default: 5).

#### FR-SK-005: Skill Quality Grading

Forge shall grade every skill on a 100-point scale.

- Grading criteria: metadata completeness (20 pts), content quality (30 pts), test coverage (20 pts), community rating (15 pts), freshness (15 pts).
- Grades displayed in catalog and used for search ranking.
- Community can submit reviews and ratings.

#### FR-SK-006: SKILL.md Standard

Forge shall support the SKILL.md frontmatter standard for skill definitions.

- Frontmatter fields: name, version, author, description, category, triggers, dependencies, and compatibility.
- Body contains the skill content (instructions, code, examples).
- Validation enforces required fields and format.

#### FR-SK-007: Skill Factory

Forge shall provide tools for generating skills programmatically.

- Interactive skill builder wizard in UI.
- Template-based generation from existing patterns.
- Agent factory: create agent definitions from skill descriptions.
- Command factory: generate slash commands from specifications.

---

### 3.6 Plugins

**Bounded Context**: Extending Forge with WASM modules, MCP servers, and native plugins.

**Sources**: claude-code-plugins-plus-skills, claude-code-templates, claude-code-mcp

#### FR-PL-001: WASM Plugin Host

Forge shall host WebAssembly plugins in a sandboxed runtime.

- Plugins run in a WASM sandbox with defined memory and CPU limits.
- Plugin API provides access to: agent state, session events, file system (scoped), and HTTP (scoped).
- Plugins are loaded at startup and can be hot-reloaded.

#### FR-PL-002: MCP Client

Forge shall act as an MCP client, connecting to external MCP servers.

- Connect to MCP servers over stdio, SSE, or WebSocket transports.
- Discover and invoke tools from connected servers.
- Access resources and prompts from connected servers.
- MCP server configuration per-agent and globally.

#### FR-PL-003: Native Plugin Interface

Forge shall support native plugins via a defined Rust trait.

- Plugin trait: `init()`, `on_event()`, `on_tool_call()`, `shutdown()`.
- Native plugins are compiled as shared libraries and loaded at startup.
- Plugin discovery via `~/.claude-forge/plugins/` directory.

#### FR-PL-004: Plugin Marketplace

Forge shall provide a browsable marketplace for discovering and installing plugins.

- Marketplace displays: name, type (WASM/MCP/native), description, author, install count, quality grade.
- One-click installation from marketplace.
- External sync from configured GitHub repositories.

#### FR-PL-005: Plugin Validation

Forge shall validate plugins before installation.

- Schema validation for plugin manifest.
- Security scan for known vulnerability patterns.
- Compatibility check against current Forge version.
- Quality grading using the 100-point scale.

---

### 3.7 Git Integration

**Bounded Context**: Source control operations, diff viewing, worktree management, and commit workflows.

**Sources**: 1code, codemcp, claude-code-viewer, claude-code-action

#### FR-GI-001: Git Status

Forge shall display the current Git status of the working directory.

- Staged, modified, untracked, and conflicted files.
- Real-time updates when files change.
- Status is accessible via UI panel and MCP resource.

#### FR-GI-002: Git Diff Viewer

Forge shall provide a rich diff viewer for file changes.

- Side-by-side and unified diff views.
- Syntax highlighting per language.
- Inline comments and annotations.
- Diff available for: working directory vs HEAD, staged vs HEAD, arbitrary commits.

#### FR-GI-003: Git Commit

Forge shall support creating commits from the UI.

- Stage/unstage individual files or hunks.
- Commit message editor with AI-generated suggestions.
- Conventional commit format support.
- Pre-commit hook execution.

#### FR-GI-004: Branch Management

Forge shall provide branch creation, switching, and deletion.

- Branch list with search and status display.
- Create branches with configurable naming patterns.
- Switch branches with uncommitted change handling (stash, commit, or abort).

#### FR-GI-005: Worktree-per-Agent

Forge shall isolate each agent in its own Git worktree.

- When enabled, starting an agent creates a new worktree from the current branch.
- Agent's working directory is set to the worktree.
- On completion, changes can be merged, cherry-picked, or discarded.
- Worktree cleanup is automatic on session end (configurable).

#### FR-GI-006: PR Creation

Forge shall support creating pull requests from the UI.

- Generate PR title and description from commit messages and agent context.
- Support for GitHub and GitLab.
- PR templates with configurable defaults.

#### FR-GI-007: Git-Versioned Edits

Forge shall optionally commit every agent file edit as a fine-grained Git commit.

- Each edit creates a commit with a descriptive message.
- Commits can be squashed on session end.
- Enables per-edit rollback without losing other changes.

---

### 3.8 Observability

**Bounded Context**: Real-time monitoring, visualization, cost tracking, and analytics for agent activity.

**Sources**: claude-code-hooks-multi-agent-observability, Claude-Code-Usage-Monitor, claude-code-hub

#### FR-OB-001: Event Capture Pipeline

Forge shall capture all 12 hook lifecycle event types.

- Events: PreToolUse, PostToolUse, Notification, Stop, SubagentStart, SubagentStop, and 6 others.
- Events are persisted to SQLite in batch (50 events or 2 seconds).
- Events are broadcast via WebSocket to connected UI clients.

#### FR-OB-002: Swim-Lane Visualization

Forge shall display multi-agent activity in a swim-lane timeline view.

- One lane per active agent.
- Events displayed as colored blocks on the timeline.
- Tool emoji system: Bash (>_), Read (eye), Write (pencil), Grep (magnifying glass), etc.
- Zoom and pan controls with time axis.

#### FR-OB-003: Pulse Chart

Forge shall display a real-time pulse chart of agent activity.

- Bars represent event frequency per time window.
- Color-coded by session identity.
- Configurable time window (1s, 5s, 15s, 60s).

#### FR-OB-004: Cost Analytics

Forge shall provide detailed cost analytics.

- Real-time token usage per agent and per session.
- Model-specific pricing tables (configurable and updatable).
- Views: real-time, daily, monthly.
- P90 percentile predictions with rolling window analysis.
- Plan support: Pro, Max5, Max20, Custom.
- Cost projection: estimated spend by end of billing period.

#### FR-OB-005: Usage Prediction

Forge shall predict when usage limits will be reached.

- ML-based prediction using historical usage patterns.
- P90 percentile analysis for conservative estimates.
- Automatic plan switching recommendations.
- Alert thresholds configurable per plan.

#### FR-OB-006: Chat Transcript Viewer

Forge shall render chat transcripts with full fidelity.

- Syntax highlighting for code blocks.
- Tool call expansion (show input/output).
- Search within transcript.
- Auto-scroll with manual override.

#### FR-OB-007: Decision Chain Logging

Forge shall log the full decision chain for auditing.

- Every routing decision (model selection, tool selection, agent selection) is logged.
- Decision chain is queryable by session, agent, or time range.
- Exportable for compliance and debugging.

---

### 3.9 Notifications

**Bounded Context**: Alerting users about agent events across multiple channels.

**Sources**: Claude-Code-Remote, claude-code-telegram, claude-code-hub

#### FR-NO-001: Webhook Notifications

Forge shall send notifications to configurable webhook URLs.

- Payload: JSON with event type, agent, session, message, timestamp.
- Retry policy: 3 attempts with exponential backoff.
- Signature verification (HMAC-SHA256) for receiver authentication.

#### FR-NO-002: Telegram Integration

Forge shall send and receive messages via Telegram bot.

- Bot setup with BotFather integration.
- Notifications on: agent completion, failure, budget warning, scheduled run results.
- Reply-to-send: execute commands by replying to notification messages.
- Multi-project thread management.

#### FR-NO-003: Discord Integration

Forge shall send notifications to Discord channels via webhook.

- Rich embeds with agent name, status, and summary.
- Thread creation for long-running agents.
- Reaction-based actions (e.g., react with stop emoji to kill agent).

#### FR-NO-004: Email Notifications

Forge shall send email notifications via SMTP.

- Configurable SMTP server settings.
- Email includes full execution trace (optional).
- HTML and plain text formats.

#### FR-NO-005: Desktop Notifications

Forge shall send desktop notifications via the operating system.

- macOS: native notification center.
- Linux: libnotify / D-Bus.
- Windows: toast notifications.
- Sound alerts for critical events (configurable).

#### FR-NO-006: Notification Rules

Forge shall support configurable notification rules.

- Rules specify: event type, severity threshold, channel(s), and message template.
- Default rules for: agent failure, budget exceeded, scheduled run complete.
- Users can create custom rules.

---

### 3.10 Security

**Bounded Context**: Permission management, audit logging, file protection, and vulnerability scanning.

**Sources**: claude-code-security-review, claude-code-config, ralph-claude-code

#### FR-SE-001: Permission System

Forge shall enforce a permission system for agent capabilities.

- Permissions: file read, file write, shell execute, network access, Git operations, MCP tool invocation.
- Permissions configurable per agent, per project, and globally.
- Default: restrictive (security-first, following Trail of Bits patterns).

#### FR-SE-002: Audit Log

Forge shall maintain an immutable audit log of all security-relevant events.

- Events: agent start/stop, file modifications, shell commands, permission changes, configuration changes.
- Audit log stored in SQLite with tamper-detection checksums.
- Exportable in JSONL format.
- Retention policy: configurable (default 90 days).

#### FR-SE-003: File Protection Rules

Forge shall prevent agents from modifying protected files.

- Protection rules specified as glob patterns.
- Default protected patterns: `**/.env*`, `**/credentials*`, `**/.git/config`, `**/secrets*`.
- Protection enforced via pre-write hooks.
- Violation attempts are logged to audit log and trigger notifications.

#### FR-SE-004: Security Scanning

Forge shall scan agent-generated code changes for security vulnerabilities.

- Diff-aware scanning: only scan changed lines.
- Detection categories: injection, authentication bypass, data exposure, cryptographic issues.
- Semantic analysis (not just pattern matching).
- False positive filtering with configurable sensitivity.
- Results displayed inline in the diff viewer and as a summary report.

#### FR-SE-005: Slash Command Security

Forge shall provide a `/security-review` command for on-demand scanning.

- Works both in UI and via CLI.
- Scans working directory or specified paths.
- Configurable rules and severity thresholds.
- Results include remediation suggestions.

---

### 3.11 Configuration

**Bounded Context**: Hierarchical configuration with scopes, validation, and import/export.

**Sources**: claude-code-config, claude-code-config2, claude-code-settings, claude-code-showcase

#### FR-CF-001: Hierarchical Configuration Scopes

Forge shall support configuration at four scopes, with narrower scopes overriding broader ones.

| Scope | Location | Priority |
|-------|----------|----------|
| Global | `~/.claude-forge/config.toml` | Lowest |
| User | `~/.claude-forge/user.toml` | |
| Project | `<project>/.claude-forge.toml` | |
| Agent | Inline in agent definition | Highest |

#### FR-CF-002: Configuration Validation

Forge shall validate all configuration on load and on change.

- Schema validation against TOML type definitions.
- Semantic validation (e.g., rate limit > 0, valid cron expressions, existing file paths).
- Invalid configuration produces clear error messages with line numbers and suggestions.

#### FR-CF-003: Configuration Import/Export

Forge shall support importing and exporting complete configurations.

- Export: produces a single TOML file with all scopes merged.
- Import: applies configuration from a TOML file, respecting scope hierarchy.
- Selective import: choose which sections to import.

#### FR-CF-004: Settings UI

Forge shall provide a graphical settings editor in the UI.

- Organized by section: agents, safety, workflows, notifications, security, appearance.
- Each setting shows: current value, default value, scope where it is set.
- Inline documentation for every setting.
- Real-time validation with error display.

#### FR-CF-005: Ecosystem Compatibility

Forge shall read and respect existing Claude Code configuration files.

- `CLAUDE.md`: project-level instructions (read as system prompt additions).
- `settings.json`: Claude Code settings (mapped to Forge equivalents).
- `.ralphrc`: ralph-claude-code configuration (safety settings imported).
- `codemcp.toml`: codemcp configuration (shell commands and project settings imported).
- `SKILL.md`: skill definitions (imported into skill catalog).

#### FR-CF-006: Health Check

Forge shall provide a diagnostic health check command.

- Validates: configuration, database integrity, MCP server connectivity, Git status, skill catalog, plugin status.
- Reports issues with severity and remediation steps.
- Available via UI, CLI (`forge health`), and MCP tool.

---

### 3.12 MCP Server

**Bounded Context**: Exposing all Forge capabilities via Model Context Protocol for meta-agentic orchestration.

**Sources**: claude-code-mcp, codemcp, claude-code-tools

#### FR-MC-001: MCP Tool Exposure

Forge shall expose 30+ tools via MCP protocol.

| Tool Category | Example Tools |
|---------------|--------------|
| Agent | `create_agent`, `run_agent`, `stop_agent`, `list_agents` |
| Session | `search_sessions`, `resume_session`, `export_session` |
| Workflow | `create_workflow`, `run_workflow`, `get_workflow_status` |
| Git | `git_status`, `git_diff`, `git_commit`, `create_worktree` |
| Skill | `search_skills`, `install_skill`, `activate_skill` |
| Safety | `get_circuit_breaker_state`, `set_rate_limit`, `check_file_protection` |
| Config | `get_config`, `set_config`, `health_check` |
| Observability | `get_events`, `get_cost_summary`, `get_usage_prediction` |

#### FR-MC-002: MCP Resource Exposure

Forge shall expose 15+ resources via MCP protocol.

| Resource | URI Pattern | Description |
|----------|------------|-------------|
| Agent list | `forge://agents` | All agent definitions |
| Session list | `forge://sessions` | Recent sessions with metadata |
| Skill catalog | `forge://skills` | All available skills |
| Config | `forge://config/{scope}` | Configuration at specified scope |
| Git status | `forge://git/status` | Current repository status |
| Cost summary | `forge://cost/{period}` | Cost data for period |
| Event stream | `forge://events` | Live event subscription |

#### FR-MC-003: MCP Prompt Exposure

Forge shall expose prompt templates via MCP protocol.

- Prompt templates for common operations: create feature, fix bug, review code, write tests.
- Templates accept parameters (e.g., file paths, issue descriptions).
- Templates reference the appropriate agent preset and workflow level.

#### FR-MC-004: MCP Transport Support

Forge shall support multiple MCP transport protocols.

- stdio: for direct process invocation.
- SSE (Server-Sent Events): for HTTP-based clients.
- WebSocket: for persistent connections.
- Transport is auto-detected or configurable.

#### FR-MC-005: Permission Bypass Mode

Forge shall support a `--dangerously-skip-permissions` mode for autonomous MCP operation.

- When invoked as an MCP server by a trusted orchestrator, all permission prompts are auto-approved.
- This mode is opt-in and logged to the audit trail.
- Rate limiter and circuit breaker remain active even in bypass mode.

#### FR-MC-006: MCP Client Compatibility

Forge's MCP server shall be compatible with major MCP clients.

- Tested with: Claude Desktop, Cursor, Windsurf, and custom MCP clients.
- Compliance with MCP specification version 1.0+.
- Tool selection exposure: clients can request specific tools only.

---

## 4. Non-Functional Requirements

### 4.1 Performance

| Metric | Requirement |
|--------|-------------|
| UI initial load time | < 500ms for cached, < 2s for uncached |
| UI interaction response (P95) | < 200ms |
| WebSocket event latency | < 50ms from event to UI render |
| FTS search latency | < 100ms for 10,000 sessions |
| Agent spawn time | < 1 second from request to first output |
| SQLite write throughput | >= 1,000 events/second |
| Memory usage (idle) | < 50 MB |
| Memory usage (10 concurrent agents) | < 500 MB |
| Binary size | < 30 MB (compressed) |

### 4.2 Reliability

| Metric | Requirement |
|--------|-------------|
| Uptime (when running) | 99.9% (< 8.7 hours downtime per year) |
| Data durability | Zero data loss on process crash (WAL mode) |
| Crash recovery time | < 5 seconds to full operation |
| Maximum concurrent agents | 50 |
| Workflow state recovery | 100% of in-progress workflows resume after crash |

### 4.3 Security

| Requirement | Detail |
|-------------|--------|
| Default permissions | Restrictive (deny by default) |
| Audit log integrity | Tamper-detection checksums |
| Secret protection | Never log API keys, tokens, or passwords |
| File protection | Configurable glob patterns enforced pre-write |
| MCP bypass | Opt-in only, always logged |
| Network isolation | Agents cannot make network requests unless explicitly permitted |

### 4.4 Usability

| Requirement | Detail |
|-------------|--------|
| Zero-dependency install | Single binary, no runtime dependencies |
| Time to first run | < 30 seconds from download |
| Documentation | 100% of public APIs documented |
| Keyboard navigation | All primary actions accessible via keyboard shortcuts |
| Dark/light theme | Automatic detection with manual override |
| Responsive UI | Usable on screens >= 1024px wide |
| Accessibility | WCAG 2.1 AA compliance for UI |

### 4.5 Scalability

| Metric | Requirement |
|--------|-------------|
| Sessions before performance degradation | 100,000+ |
| Skills in catalog | 10,000+ without search degradation |
| Concurrent WebSocket connections | 100+ |
| Events per session before UI lag | 50,000+ |

### 4.6 Compatibility

| Platform | Support Level |
|----------|--------------|
| macOS (Apple Silicon) | Tier 1 (primary development platform) |
| macOS (Intel) | Tier 1 |
| Linux (x86_64) | Tier 1 |
| Linux (aarch64) | Tier 2 |
| Windows (x86_64) | Tier 2 |
| Windows (aarch64) | Tier 3 |

### 4.7 Observability of the System Itself

| Metric | Requirement |
|--------|-------------|
| Structured logging | JSON-formatted logs with tracing spans |
| Health endpoint | `GET /health` returning system status |
| Metrics endpoint | `GET /metrics` in Prometheus format (optional) |
| Error reporting | Stack traces with context in debug mode |

---

## 5. Constraints and Assumptions

### 5.1 Constraints

| ID | Constraint | Rationale |
|----|-----------|-----------|
| C1 | Single binary distribution | Core product principle; no installer, no runtime dependencies |
| C2 | Rust backend | Performance, safety, and single-binary compilation |
| C3 | Svelte 5 frontend | Already in use; embedded via `rust-embed` |
| C4 | SQLite only (no PostgreSQL) | Offline-first, zero-dependency; WAL mode for concurrent access |
| C5 | No cloud service dependency | Forge must work fully offline (API keys are for LLM providers only) |
| C6 | Default port 4173 | Already established; configurable via CLI flag |
| C7 | Backward compatible with Claude Code | Must read existing `~/.claude/` directory structure |
| C8 | MCP specification compliance | Must pass MCP conformance tests |

### 5.2 Assumptions

| ID | Assumption | Risk if False |
|----|-----------|---------------|
| A1 | Users have Claude Code CLI installed | Agent spawning depends on `claude` binary in PATH |
| A2 | Users have Git installed | Git integration features require `git` binary |
| A3 | Users have internet access for LLM API calls | Agents cannot function without API access |
| A4 | SQLite WAL mode supports the concurrency we need | May need to shard or use connection pooling |
| A5 | WASM runtime overhead is acceptable | Plugin execution may need performance tuning |
| A6 | 33,000 LOC is sufficient for all P0/P1 features | May need to prioritize or defer some features |
| A7 | The `stream-json` output format is stable | Agent event parsing depends on this format |
| A8 | MCP specification will not have breaking changes | MCP server implementation depends on spec stability |

---

## 6. Dependencies

### 6.1 External Dependencies

| Dependency | Purpose | Version | Required |
|------------|---------|---------|----------|
| Claude Code CLI | Agent process spawning | Latest | Yes (for Claude agents) |
| Git | Version control integration | 2.30+ | Yes (for Git features) |
| Codex CLI | Multi-CLI orchestration | Latest | Optional |
| Gemini CLI | Multi-CLI orchestration | Latest | Optional |
| Qwen CLI | Multi-CLI orchestration | Latest | Optional |

### 6.2 Rust Crate Dependencies

| Crate | Purpose | Notes |
|-------|---------|-------|
| `axum` 0.8 | HTTP framework | With WebSocket support |
| `tokio` | Async runtime | Full features |
| `rusqlite` | SQLite (bundled) | WAL mode, FTS5 extension |
| `rust-embed` | Static file embedding | Embeds Svelte build output |
| `serde` / `serde_json` | Serialization | JSON and TOML |
| `dashmap` | Concurrent hashmap | Agent state, session cache |
| `tokio::sync::broadcast` | Event broadcasting | WebSocket fan-out |
| `wasmtime` | WASM plugin runtime | Sandboxed execution |
| `git2` | Git operations | libgit2 bindings |
| `tantivy` | Full-text search | Session FTS index |
| `tracing` | Structured logging | With JSON formatter |
| `clap` | CLI argument parsing | For `forge` binary |
| `toml` | Configuration parsing | TOML format |
| `reqwest` | HTTP client | Webhook notifications |
| `lettre` | Email sending | SMTP notifications |
| `notify` | File system watching | Real-time Git status |
| `cron` | Cron expression parsing | Scheduler |

### 6.3 Frontend Dependencies

| Package | Purpose | Notes |
|---------|---------|-------|
| SvelteKit | Application framework | adapter-static for embedding |
| Svelte 5 | UI framework | Runes ($state, $derived, $effect) |
| TailwindCSS 4 | Styling | Utility-first CSS |
| `@xterm/xterm` | Terminal emulation | WebSocket terminal panel |
| `d3` or `chart.js` | Visualization | Swim lanes, pulse charts, cost graphs |
| `monaco-editor` | Code editor | Diff viewer, CLAUDE.md editor |
| `fuse.js` | Fuzzy search | Client-side session/skill search |

### 6.4 Build Dependencies

| Tool | Purpose | Version |
|------|---------|---------|
| Rust / Cargo | Backend compilation | 1.85+ (2024 edition) |
| Node.js | Frontend build | 22+ (via mise) |
| pnpm | Package management | Latest (via mise) |

---

## Appendix A: Glossary

| Term | Definition |
|------|-----------|
| **Agent** | A configured AI assistant with a specific model, system prompt, tools, and working directory |
| **Bounded Context** | A logical grouping of related features with clear boundaries |
| **Circuit Breaker** | A 3-state safety mechanism that halts agents on repeated failures |
| **DAG** | Directed Acyclic Graph; used for modeling workflow step dependencies |
| **Exit Gate** | A dual-condition check ensuring agents truly completed their task |
| **FTS** | Full-Text Search; powered by SQLite FTS5 or Tantivy |
| **Hook** | A lifecycle callback triggered by Claude Code events |
| **MCP** | Model Context Protocol; a standard for AI tool interoperability |
| **Preset** | A pre-configured agent definition for a specific domain |
| **Skill** | A reusable instruction set that extends an agent's capabilities |
| **Swim Lane** | A visualization showing parallel agent activity on a shared timeline |
| **WAL** | Write-Ahead Logging; SQLite mode enabling concurrent reads and writes |
| **WASM** | WebAssembly; used for sandboxed plugin execution |
| **Worktree** | A Git feature providing an independent working directory for a branch |

## Appendix B: Document References

| Document | Description |
|----------|-------------|
| [USER_PERSONAS.md](USER_PERSONAS.md) | Detailed user persona profiles |
| [USER_STORIES.md](USER_STORIES.md) | Comprehensive user stories organized by epic |
| [FEATURE_CATALOG.md](FEATURE_CATALOG.md) | Complete catalog of 200+ features with priority and effort |
| [Reference Repository Map](../../reference-map/README.md) | Analysis of all 62 source repositories |
