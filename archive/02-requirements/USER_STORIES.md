# Claude Forge -- User Stories

**Version**: 1.0
**Date**: 2026-02-25

---

## Table of Contents

1. [Epic 1: Agent Management](#epic-1-agent-management)
2. [Epic 2: Safety and Reliability](#epic-2-safety-and-reliability)
3. [Epic 3: Workflows](#epic-3-workflows)
4. [Epic 4: Session Management](#epic-4-session-management)
5. [Epic 5: Skills and Plugins](#epic-5-skills-and-plugins)
6. [Epic 6: Development Environment](#epic-6-development-environment)
7. [Epic 7: Observability](#epic-7-observability)
8. [Epic 8: MCP Integration](#epic-8-mcp-integration)
9. [Epic 9: Configuration and Security](#epic-9-configuration-and-security)
10. [Epic 10: Notifications and Scheduling](#epic-10-notifications-and-scheduling)

---

## Conventions

- **Personas**: Maya (Solo Dev), David (Team Lead), Rina (Tool Builder), James (DevOps), Sven (Contributor)
- **Priority**: P0 = must have, P1 = should have, P2 = nice to have, P3 = future
- **Story Points**: Fibonacci (1, 2, 3, 5, 8, 13)
- **Acceptance Criteria**: Testable conditions that must be true for the story to be complete

---

## Epic 1: Agent Management

> As a user, I can create, configure, execute, and coordinate AI agents to accomplish development tasks.

### US-AM-001: Create Agent from Scratch

**As** Maya (Solo Dev), **I want** to create a new agent by specifying its name, model, system prompt, and working directory **so that** I have a tailored assistant for my specific project.

**Priority**: P0 | **Points**: 3

**Acceptance Criteria**:
- [ ] User can fill in a form with: name (required), model (dropdown with defaults), system prompt (text area), working directory (directory picker), and description (optional).
- [ ] Agent is persisted to SQLite and appears in the agent list immediately.
- [ ] Agent can be started immediately after creation.
- [ ] Validation errors are displayed inline (e.g., name already exists, directory does not exist).

### US-AM-002: Create Agent from Preset

**As** Maya (Solo Dev), **I want** to create an agent from a pre-configured preset **so that** I can start working immediately without configuring everything manually.

**Priority**: P0 | **Points**: 2

**Acceptance Criteria**:
- [ ] Preset browser shows 100+ presets organized by category (15+ categories).
- [ ] Each preset shows: name, description, category, model, and key capabilities.
- [ ] Selecting a preset pre-fills the agent creation form.
- [ ] User can modify any pre-filled field before saving.
- [ ] Preset search supports text filtering by name and category.

### US-AM-003: Edit Agent Configuration

**As** Maya (Solo Dev), **I want** to edit an existing agent's configuration **so that** I can adjust its behavior as project needs change.

**Priority**: P0 | **Points**: 2

**Acceptance Criteria**:
- [ ] All fields editable: name, model, system prompt, working directory, allowed tools, environment variables, MCP servers, hooks.
- [ ] Changes are saved to SQLite immediately.
- [ ] Editing a running agent shows a warning that changes take effect on next run.
- [ ] Version history of configuration changes is maintained (last 10 versions).

### US-AM-004: Delete Agent

**As** Maya (Solo Dev), **I want** to delete an agent I no longer need **so that** my agent list stays organized.

**Priority**: P0 | **Points**: 1

**Acceptance Criteria**:
- [ ] Confirmation dialog before deletion.
- [ ] Running agents cannot be deleted (must be stopped first).
- [ ] Deletion removes the agent definition but preserves session history.
- [ ] Deletion is recorded in the audit log.

### US-AM-005: Run Multiple Agents in Parallel

**As** Maya (Solo Dev), **I want** to run multiple agents simultaneously on different projects **so that** I can maximize my productivity.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] User can start agents for different projects while other agents are running.
- [ ] Each running agent has its own tab in the multi-pane layout.
- [ ] Agent processes are independent (one failing does not affect others).
- [ ] Resource usage (memory, CPU) is visible per agent.
- [ ] Maximum concurrent agents is configurable (default: 10).

### US-AM-006: Select CLI Backend per Agent

**As** Maya (Solo Dev), **I want** to choose which AI CLI tool an agent uses (Claude, Codex, Gemini, Qwen) **so that** I can use the best model for each task.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Agent configuration includes a "CLI Backend" dropdown with all supported CLIs.
- [ ] Custom CLI binary path configurable for unlisted tools.
- [ ] Each CLI is abstracted behind a common process interface (spawn, stream events, kill).
- [ ] Unsupported CLIs produce a clear error with installation instructions.

### US-AM-007: Create Agent Team

**As** David (Team Lead), **I want** to group agents into a team with defined roles **so that** multiple agents can collaborate on a complex task.

**Priority**: P1 | **Points**: 8

**Acceptance Criteria**:
- [ ] User can create a team, assign agents, and define roles (Lead, Builder, Validator, Worker).
- [ ] Team has a shared context that all member agents can access.
- [ ] Team execution shows all agents in the swim-lane view with team color.
- [ ] Team patterns available: Builder+Validator, Lead+Workers, Pipeline.
- [ ] Team membership stored in SQLite with many-to-many relation.

### US-AM-008: Hand Off Context Between Agents

**As** Maya (Solo Dev), **I want** to pass context from one agent to another **so that** the second agent can continue where the first left off.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] User can select an agent and click "Hand Off" to choose a target agent.
- [ ] Handoff payload includes: files changed, decisions made, open questions, and session summary.
- [ ] Target agent receives the context as a prefilled prompt.
- [ ] Handoff is recorded in both agents' session histories.
- [ ] Handoff can also be triggered via workflow step completion.

### US-AM-009: Automatically Invoke Agent by Context

**As** Maya (Solo Dev), **I want** agents to be invoked automatically when the conversation context matches their triggers **so that** I get specialized help without manual switching.

**Priority**: P2 | **Points**: 5

**Acceptance Criteria**:
- [ ] Agents can declare trigger phrases (e.g., "security", "database migration") and file patterns (e.g., `*.sql`).
- [ ] When a prompt matches a trigger, the relevant agent is suggested or auto-invoked.
- [ ] @mention syntax (`@security-reviewer`) directly invokes a named agent.
- [ ] Auto-invocation respects rate limits and circuit breaker state.
- [ ] User can disable auto-invocation per agent or globally.

### US-AM-010: Route Tasks to Optimal Model

**As** Maya (Solo Dev), **I want** Forge to automatically select the best model for each task **so that** I get the best results without manually switching models.

**Priority**: P2 | **Points**: 8

**Acceptance Criteria**:
- [ ] Router supports scenario categories: background tasks, deep thinking, long context, web search, code generation.
- [ ] Default routing rules are provided and configurable.
- [ ] Custom router scripts (JavaScript) can be loaded for advanced logic.
- [ ] Router decision is logged and visible in observability.
- [ ] Override is available per prompt via a model selector dropdown.

---

## Epic 2: Safety and Reliability

> As a user, I can trust that my agents operate safely, stay within budget, and do not damage my codebase.

### US-SR-001: Circuit Breaker Halts Runaway Agent

**As** Maya (Solo Dev), **I want** the circuit breaker to automatically stop an agent that is stuck in a loop **so that** I do not waste tokens and time.

**Priority**: P0 | **Points**: 8

**Acceptance Criteria**:
- [ ] Circuit breaker tracks consecutive failures per agent.
- [ ] When failure count exceeds threshold (configurable, default 5), circuit opens and agent is paused.
- [ ] After cooldown period (configurable, default 60s), circuit transitions to HALF_OPEN.
- [ ] In HALF_OPEN, one probe request is allowed; success closes the circuit, failure reopens.
- [ ] State transitions are displayed in the observability dashboard.
- [ ] Circuit breaker state is accessible via MCP (`get_circuit_breaker_state`).

### US-SR-002: Rate Limiter Enforces API Budget

**As** David (Team Lead), **I want** per-agent and per-team rate limits **so that** no single agent or developer can exhaust the API budget.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] Rate limit configurable per agent, per user, and globally (calls/hour, RPM, budget/period).
- [ ] Agent is paused (not killed) when rate limited.
- [ ] Countdown timer shows time until rate limit resets.
- [ ] Agent auto-resumes when rate limit window resets.
- [ ] Rate limit status is visible in the agent card and accessible via MCP.

### US-SR-003: Dual Exit Gate Validates Completion

**As** Maya (Solo Dev), **I want** the exit gate to verify that my agent truly finished its task **so that** I do not accept incomplete work.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Agent output is analyzed for completion indicators (semantic analysis).
- [ ] Agent output is checked for explicit EXIT_SIGNAL token.
- [ ] Both signals required for "complete" status; one signal triggers a clarification prompt.
- [ ] Exit gate mode configurable per agent: strict (both required), normal (one sufficient with confirmation), relaxed (any signal).
- [ ] False completion rate is tracked and visible in analytics.

### US-SR-004: Response Analyzer Detects Problems

**As** Maya (Solo Dev), **I want** real-time analysis of agent responses for loops, errors, and stuck states **so that** problems are caught before they waste resources.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Analyzer detects: repeated output (loop), stack traces (errors), no progress (stuck), references to nonexistent files (hallucination).
- [ ] Configurable thresholds for each pattern type.
- [ ] Configurable actions: warn (show banner), pause, kill, or escalate (notification).
- [ ] Detection events are logged to the observability timeline.
- [ ] Analyzer latency does not exceed 100ms per response.

### US-SR-005: File Protection Prevents Critical File Modification

**As** James (DevOps), **I want** agents to be blocked from modifying protected files **so that** secrets, infrastructure, and CI configuration are never accidentally changed.

**Priority**: P0 | **Points**: 3

**Acceptance Criteria**:
- [ ] Protected paths specified as glob patterns (e.g., `**/.env*`, `**/credentials*`).
- [ ] Default protection list includes: `.env*`, `credentials*`, `.git/config`, `docker-compose*.yml`, `Dockerfile`, `*.tfstate`.
- [ ] Protection enforced at pre-write hook level.
- [ ] Violation attempts logged to audit log with full context (agent, file, content).
- [ ] Notification sent on violation (configurable channel).

### US-SR-006: Pre-Run Validation Checks Environment

**As** Maya (Solo Dev), **I want** Forge to validate my environment before starting an agent **so that** I do not waste time on runs that will fail immediately.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Pre-run checks: working directory exists, CLI binary is in PATH, Git repo is valid (if Git features needed), required MCP servers are reachable.
- [ ] Failed checks produce specific, actionable error messages.
- [ ] Agent cannot start until all checks pass (override available with `--force`).
- [ ] Checks complete in under 2 seconds.

### US-SR-007: Set Budget Hard Limits

**As** James (DevOps), **I want** to set hard dollar limits per agent, per project, and globally **so that** costs never exceed my budget.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] Budget limits configurable in dollars, tokens, or API calls.
- [ ] Soft limit at configurable threshold (default 80%) triggers warning notification.
- [ ] Hard limit at 100% pauses the agent immediately.
- [ ] Budget tracking uses model-specific pricing tables.
- [ ] Budget status visible in agent card, cost dashboard, and via MCP.
- [ ] Daily, weekly, and monthly budget periods supported.

### US-SR-008: Graceful Degradation on Subsystem Failure

**As** Rina (Tool Builder), **I want** Forge to continue operating when optional subsystems fail **so that** my production integration stays available.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] If notification service is down: log locally, retry on next event.
- [ ] If MCP server is unreachable: agent continues without that tool, warning logged.
- [ ] If FTS index is corrupted: session browser works without search, background rebuild starts.
- [ ] Degradation events logged with severity level.
- [ ] Health check endpoint reports degraded subsystems.

---

## Epic 3: Workflows

> As a user, I can define and execute multi-step workflows with dependency management and parallelism.

### US-WF-001: Run a Quick Single-Step Workflow

**As** Maya (Solo Dev), **I want** to run a simple one-shot task with minimal configuration **so that** I can get quick answers without setting up a full workflow.

**Priority**: P0 | **Points**: 2

**Acceptance Criteria**:
- [ ] User types a prompt and clicks "Run" -- this is a lite-lite-lite (L1) workflow.
- [ ] No dependency graph, no parallelism -- just a single agent run.
- [ ] Result displayed immediately in the active pane.
- [ ] L1 is the default level when no workflow is specified.

### US-WF-002: Create Multi-Step Workflow with Dependencies

**As** Maya (Solo Dev), **I want** to define a workflow with multiple steps and specify which steps depend on others **so that** complex tasks execute in the correct order.

**Priority**: P1 | **Points**: 8

**Acceptance Criteria**:
- [ ] Workflow editor shows steps as nodes in a graph.
- [ ] User can add steps, define dependencies (edges), and configure each step (agent, prompt, timeout).
- [ ] Dependency validation: no cycles allowed (DAG enforcement).
- [ ] Workflow saved to SQLite and can be reused as a template.
- [ ] Workflow state is persisted for crash recovery.

### US-WF-003: Execute Parallel Workflow Steps

**As** Maya (Solo Dev), **I want** independent workflow steps to execute in parallel **so that** workflows complete faster.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Steps with no mutual dependencies run concurrently.
- [ ] Configurable concurrency limit (default: 4 parallel steps).
- [ ] Parallel steps are visible in the workflow graph with "running" status.
- [ ] If a parallel step fails, other parallel steps continue (configurable: continue/abort).
- [ ] Total workflow duration is bounded by the critical path, not the sum of all steps.

### US-WF-004: Use Workflow Templates

**As** David (Team Lead), **I want** to create and share workflow templates **so that** my team follows consistent processes.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Pre-built templates for: feature implementation, bug fix, refactoring, code review, test writing, documentation, release prep, security audit.
- [ ] Templates accept parameters (e.g., branch name, file paths, issue URL).
- [ ] User can create a custom template from any completed workflow.
- [ ] Templates are stored in SQLite and exportable as JSON.
- [ ] Team-wide templates settable via project-scope configuration.

### US-WF-005: Automatically Select Workflow Level

**As** Maya (Solo Dev), **I want** Forge to suggest the appropriate workflow level based on my prompt **so that** I do not over-engineer simple tasks.

**Priority**: P2 | **Points**: 5

**Acceptance Criteria**:
- [ ] Prompt analysis categorizes tasks: trivial (L1), simple (L2), standard (L3), complex (L4).
- [ ] Level suggestion shown to user with explanation.
- [ ] User can accept or override the suggestion.
- [ ] Suggestion algorithm improves based on historical completion data.

### US-WF-006: Visualize Workflow Execution

**As** David (Team Lead), **I want** to see a visual representation of workflow execution **so that** I can understand progress and identify bottlenecks.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Interactive DAG visualization with nodes colored by status: pending (gray), running (blue), succeeded (green), failed (red), skipped (yellow).
- [ ] Click a node to see: agent output, duration, cost, error (if failed).
- [ ] Critical path highlighted.
- [ ] Real-time updates as steps complete.
- [ ] Elapsed time and estimated remaining time displayed.

### US-WF-007: Resume Failed Workflow

**As** Maya (Solo Dev), **I want** to resume a workflow from the point of failure **so that** I do not re-run steps that already succeeded.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Failed workflows can be resumed with "Resume" button.
- [ ] Completed steps are skipped; failed step is retried.
- [ ] User can edit the failed step's configuration before retry.
- [ ] Resume preserves the original workflow context and outputs.

### US-WF-008: Auto-Generate Workflow from Issue

**As** Maya (Solo Dev), **I want** Forge to generate a workflow from a GitHub issue **so that** I can start implementing features immediately.

**Priority**: P2 | **Points**: 8

**Acceptance Criteria**:
- [ ] User pastes a GitHub/GitLab issue URL.
- [ ] Forge parses the issue title, description, and labels.
- [ ] A multi-step workflow is generated with appropriate agents and prompts.
- [ ] User can review and edit the generated workflow before execution.
- [ ] Results are posted back to the issue as a comment.

---

## Epic 4: Session Management

> As a user, I can browse, search, resume, export, and organize my agent sessions.

### US-SM-001: Browse All Sessions

**As** Maya (Solo Dev), **I want** to see all my past sessions in a browsable list **so that** I can find and review previous work.

**Priority**: P0 | **Points**: 3

**Acceptance Criteria**:
- [ ] Session list loads from `~/.claude/projects/` directory structure.
- [ ] Each row shows: session ID (truncated), project name, start time, duration, message count, cost, status badge.
- [ ] Sortable by any column.
- [ ] Filterable by project, date range, and status.
- [ ] Pagination for large lists (default 50 per page).

### US-SM-002: Search Sessions with Full-Text Search

**As** Maya (Solo Dev), **I want** to search across all session content with fuzzy matching **so that** I can find past solutions quickly.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] Search triggered by Cmd+K / Ctrl+K keyboard shortcut.
- [ ] FTS index covers: user messages, agent responses, tool call descriptions.
- [ ] Fuzzy matching handles typos and partial words.
- [ ] Results show: matching message with highlighted search terms, session name, date.
- [ ] Click a result to open the session at the matching message.
- [ ] Search latency < 100ms for 10,000 sessions.

### US-SM-003: Resume a Previous Session

**As** Maya (Solo Dev), **I want** to resume a previous session **so that** I can continue work without re-explaining context.

**Priority**: P0 | **Points**: 3

**Acceptance Criteria**:
- [ ] "Resume" button on each session in the browser.
- [ ] Resume uses `--resume` flag with the session ID.
- [ ] Session context (conversation history, working directory) is restored.
- [ ] If session has expired (> 24 hours, configurable), user is warned and offered a new session with the old context.
- [ ] Resume opens the session in a new tab in the multi-pane layout.

### US-SM-004: Export Session

**As** Maya (Solo Dev), **I want** to export a session in Markdown format **so that** I can share it with my clients.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Export formats: JSON (full fidelity), Markdown (human-readable), HTML (styled).
- [ ] Export options: include/exclude tool calls, system messages, timing data, cost data.
- [ ] Single session export via button on session detail.
- [ ] Bulk export with date range and project filters.
- [ ] Exported files download to the user's configured directory.

### US-SM-005: View Extracted Todos

**As** Maya (Solo Dev), **I want** to see all TodoWrite items extracted from my sessions **so that** I can track remaining work.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Todo panel shows all TodoWrite items from the active session.
- [ ] Each todo shows: description, status (pending, in-progress, done), and link to source message.
- [ ] Aggregate todo view across all sessions (filterable by project).
- [ ] Todos can be manually marked as done.
- [ ] Todo count badge on the session card.

### US-SM-006: View Session as Kanban Board

**As** Maya (Solo Dev), **I want** to see my sessions as cards on a Kanban board **so that** I can visualize the status of all my active work.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Columns: Queued, Running, Paused, Completed, Failed.
- [ ] Cards show: agent name, project, elapsed time, message count.
- [ ] Cards are draggable for reordering queue priority.
- [ ] Color coding by project or agent.
- [ ] View toggle between list and Kanban.

### US-SM-007: View Session Cost Breakdown

**As** David (Team Lead), **I want** to see a detailed cost breakdown for each session **so that** I can identify expensive operations.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Session detail shows: total cost, cost per message, cost per tool call, input tokens, output tokens.
- [ ] Model-specific pricing applied correctly.
- [ ] Cost timeline: chart showing cost accumulation over session duration.
- [ ] Aggregate cost views: per-agent, per-project, per-day.

### US-SM-008: Delete Sessions

**As** Maya (Solo Dev), **I want** to delete old sessions I no longer need **so that** my session list stays manageable and storage is reclaimed.

**Priority**: P2 | **Points**: 2

**Acceptance Criteria**:
- [ ] Single and bulk session deletion.
- [ ] Confirmation dialog with session count.
- [ ] FTS index is updated after deletion.
- [ ] Running sessions cannot be deleted.
- [ ] Deleted sessions are removed from the Kanban board immediately.

---

## Epic 5: Skills and Plugins

> As a user, I can discover, install, create, and manage skills and plugins that extend agent capabilities.

### US-SP-001: Browse Skill Catalog

**As** Maya (Solo Dev), **I want** to browse the skill catalog with categories and search **so that** I can find skills to make my agents more capable.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Catalog page shows skills organized by category (15+ categories).
- [ ] Each skill card shows: name, description, category, quality grade (0-100), install count.
- [ ] Text search across name, description, and tags.
- [ ] Category filter sidebar.
- [ ] Sort by: quality grade, install count, name, date added.
- [ ] Catalog loads within 500ms for 1,500+ skills.

### US-SP-002: Install a Skill

**As** Maya (Solo Dev), **I want** to install a skill with one click **so that** I can start using it immediately.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] "Install" button on each skill card.
- [ ] Skill files are written to `~/.claude-forge/skills/<skill-name>/`.
- [ ] Metadata recorded in SQLite (name, version, install date, source).
- [ ] Installation completes in under 5 seconds.
- [ ] Installed skills are immediately available to all agents (or configurable per-agent).
- [ ] Install from URL (GitHub repo or raw URL) also supported.

### US-SP-003: Auto-Activate Skills by Context

**As** Maya (Solo Dev), **I want** relevant skills to be automatically injected into my agent's context **so that** I get specialized help without manually configuring skills per agent.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Skills declare triggers: phrases (e.g., "deploy to AWS"), file patterns (e.g., `*.tf`), project types (e.g., "Next.js").
- [ ] When context matches a trigger, the skill's instructions are appended to the system prompt.
- [ ] Maximum concurrent auto-activated skills configurable (default 5).
- [ ] Auto-activation can be disabled per agent or globally.
- [ ] Active skills shown in a sidebar indicator.

### US-SP-004: View Skill Quality Grade

**As** Sven (Contributor), **I want** to see the quality grade breakdown for any skill **so that** I know what to improve in my own skills.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Skill detail page shows overall grade (0-100) and breakdown: metadata (20), content (30), tests (20), community (15), freshness (15).
- [ ] Each criterion has a description of what earns points.
- [ ] Improvement suggestions shown for skills below 80.
- [ ] Grade history chart showing changes over time.

### US-SP-005: Create a New Skill with Builder Wizard

**As** Sven (Contributor), **I want** to use a guided builder to create a new skill **so that** I produce correctly formatted skills without memorizing the SKILL.md spec.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Step-by-step wizard: name, description, category, triggers, content, examples.
- [ ] SKILL.md frontmatter is auto-generated from wizard inputs.
- [ ] Live preview shows the skill as it will appear in the catalog.
- [ ] Validation runs in real-time (red indicators for invalid fields).
- [ ] "Publish" button submits to the local catalog (and optionally to a remote registry).

### US-SP-006: Install and Manage WASM Plugins

**As** Rina (Tool Builder), **I want** to install WASM plugins that extend Forge with custom functionality **so that** I can integrate with my company's internal tools.

**Priority**: P1 | **Points**: 8

**Acceptance Criteria**:
- [ ] Plugin marketplace shows WASM plugins with descriptions and compatibility info.
- [ ] One-click install downloads and registers the plugin.
- [ ] Plugins run in a sandboxed WASM runtime with defined memory/CPU limits.
- [ ] Plugin API exposes: agent state, session events, file system (scoped), HTTP (scoped).
- [ ] Plugins can be enabled/disabled without uninstalling.
- [ ] Plugin errors are caught and do not crash Forge.

### US-SP-007: Connect External MCP Servers as Plugins

**As** Rina (Tool Builder), **I want** to connect external MCP servers to Forge **so that** my agents can use tools provided by those servers.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] MCP server editor allows adding servers with: name, command (stdio), URL (SSE/WS), and environment variables.
- [ ] Server connection status shown (connected, disconnected, error).
- [ ] Tools from connected servers appear in the agent's available tools list.
- [ ] Resources from connected servers are accessible via `forge://mcp/<server>/<resource>`.
- [ ] MCP servers configurable per-agent and globally.

### US-SP-008: Publish Skill to Community Catalog

**As** Sven (Contributor), **I want** to publish my skill to a shared catalog **so that** other users can discover and install it.

**Priority**: P2 | **Points**: 5

**Acceptance Criteria**:
- [ ] "Publish" action on any locally created skill.
- [ ] Skill must pass validation (minimum quality grade of 60).
- [ ] Published skills sync to a configured registry (GitHub-based or custom).
- [ ] Install count and community ratings tracked.
- [ ] Author attribution displayed in catalog.

---

## Epic 6: Development Environment

> As a user, I can use Git, view code, and manage my development environment from within Forge.

### US-DE-001: View Git Status

**As** Maya (Solo Dev), **I want** to see the current Git status in a panel **so that** I know what files have changed without switching to a terminal.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Git status panel shows: staged files, modified files, untracked files, and conflicts.
- [ ] File status icons: green (staged), yellow (modified), red (conflicted), gray (untracked).
- [ ] Real-time updates when files change (via file system watcher).
- [ ] Click a file to open its diff.

### US-DE-002: View and Navigate Diffs

**As** Maya (Solo Dev), **I want** to view file diffs with syntax highlighting **so that** I can review agent changes before committing.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Side-by-side and unified diff views.
- [ ] Syntax highlighting per language (detected from file extension).
- [ ] Diff available for: working directory vs HEAD, staged vs HEAD, between arbitrary commits.
- [ ] Navigate between changed hunks with keyboard shortcuts.
- [ ] Line count of additions and deletions shown per file.

### US-DE-003: Stage and Commit from Forge

**As** Maya (Solo Dev), **I want** to stage files and create commits **so that** I can manage version control without leaving Forge.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Checkboxes to stage/unstage individual files.
- [ ] Commit message editor with AI-generated suggestion based on diff.
- [ ] Conventional commit format support (type selector: feat, fix, docs, refactor, test, chore).
- [ ] Pre-commit hooks execute before commit.
- [ ] Commit success/failure feedback with error details.

### US-DE-004: Manage Branches

**As** Maya (Solo Dev), **I want** to create, switch, and delete branches **so that** I can manage my Git workflow from Forge.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Branch list with search filter.
- [ ] Current branch indicator.
- [ ] Create branch with naming pattern enforcement (optional).
- [ ] Switch branches with uncommitted change handling (stash, commit, or abort).
- [ ] Delete branches with force option and confirmation.

### US-DE-005: Isolate Agent in Worktree

**As** Maya (Solo Dev), **I want** each agent to run in its own Git worktree **so that** agents cannot interfere with each other's file changes.

**Priority**: P1 | **Points**: 8

**Acceptance Criteria**:
- [ ] "Worktree Isolation" toggle in agent settings (default: off).
- [ ] When enabled, starting the agent creates a worktree from the current branch.
- [ ] Agent's working directory is set to the worktree path.
- [ ] On completion: merge, cherry-pick, or discard changes (user chooses).
- [ ] Worktree cleanup is automatic on session end (configurable to keep).
- [ ] Worktree list panel shows all active worktrees.

### US-DE-006: Create Pull Request from Forge

**As** Maya (Solo Dev), **I want** to create a pull request from the Forge UI **so that** I can submit work for review without switching to GitHub.

**Priority**: P2 | **Points**: 5

**Acceptance Criteria**:
- [ ] "Create PR" button in the Git panel.
- [ ] PR title and description auto-generated from commit messages and agent context.
- [ ] Support for GitHub (via `gh` CLI) and GitLab (via `glab` CLI).
- [ ] PR template support (uses project's `.github/pull_request_template.md` if present).
- [ ] PR URL displayed after creation with link to open in browser.

### US-DE-007: View and Edit Files

**As** Maya (Solo Dev), **I want** to view and edit files from within Forge **so that** I can make quick changes without switching to my editor.

**Priority**: P2 | **Points**: 5

**Acceptance Criteria**:
- [ ] File browser tree with Cmd+P / Ctrl+P quick-open.
- [ ] Syntax highlighting for 50+ languages.
- [ ] Read-only by default; edit mode enabled with toggle.
- [ ] Changes tracked and committable via the Git panel.
- [ ] File content accessible via MCP resource (`forge://files/{path}`).

### US-DE-008: Git-Versioned Agent Edits

**As** Maya (Solo Dev), **I want** every file edit by an agent to be committed as a separate Git commit **so that** I can roll back individual changes.

**Priority**: P2 | **Points**: 5

**Acceptance Criteria**:
- [ ] "Git-Versioned Edits" toggle in agent settings (default: off).
- [ ] Each agent write creates a commit with a descriptive message (e.g., "forge: edit src/main.rs -- add error handling").
- [ ] Commits can be squashed on session end with a single-click action.
- [ ] Works correctly with worktree isolation.
- [ ] Commit log for the session viewable in the session detail.

---

## Epic 7: Observability

> As a user, I can monitor agent activity in real time with rich visualizations and analytics.

### US-OB-001: View Real-Time Event Stream

**As** Maya (Solo Dev), **I want** to see a live stream of events from my running agents **so that** I know what they are doing without reading full transcripts.

**Priority**: P0 | **Points**: 3

**Acceptance Criteria**:
- [ ] Event stream panel shows events in real-time via WebSocket.
- [ ] Each event shows: timestamp, agent name, event type, tool emoji, summary.
- [ ] Auto-scroll with manual override (click to stop, button to resume).
- [ ] Filter by event type, agent, and severity.
- [ ] Events persist to SQLite in batches (50 events or 2 seconds).

### US-OB-002: View Multi-Agent Swim Lanes

**As** David (Team Lead), **I want** to see all active agents in a swim-lane timeline **so that** I can understand the team's parallel activity at a glance.

**Priority**: P0 | **Points**: 8

**Acceptance Criteria**:
- [ ] One horizontal lane per active agent.
- [ ] Events displayed as colored blocks on the timeline.
- [ ] Tool emoji system: Bash (>_), Read (eye), Write (pencil), Grep (magnifying glass), Edit (scissors), WebFetch (globe).
- [ ] Zoom and pan controls with time axis.
- [ ] Dual-color coding: app color + session color.
- [ ] Click an event block to see details.

### US-OB-003: View Pulse Chart

**As** Maya (Solo Dev), **I want** to see a real-time pulse chart **so that** I can tell at a glance which agents are active and how busy they are.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Bars represent event frequency per time window.
- [ ] Color-coded by session/agent identity.
- [ ] Configurable time window: 1s, 5s, 15s, 60s.
- [ ] Chart auto-scrolls with latest data.
- [ ] Idle agents show flat bars.

### US-OB-004: Track Token Usage and Costs in Real Time

**As** Maya (Solo Dev), **I want** to see real-time token usage and cost for each running agent **so that** I can stop expensive runs before they exceed my budget.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] Per-agent counters: input tokens, output tokens, total cost.
- [ ] Running total across all agents.
- [ ] Budget progress bar (percentage of configured limit).
- [ ] Model-specific pricing (Claude Opus, Sonnet, Haiku, etc.).
- [ ] Cost-per-minute rate indicator.

### US-OB-005: View Usage Predictions

**As** Maya (Solo Dev), **I want** to see predictions of when I will hit my usage limits **so that** I can plan my work accordingly.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Prediction based on rolling window of historical usage.
- [ ] P90 percentile analysis (conservative estimate).
- [ ] Display: "At current rate, you will reach your limit in X hours/days."
- [ ] Plan support: Pro, Max5, Max20, Custom.
- [ ] Automatic plan switching recommendation when limit is approaching.

### US-OB-006: View Chat Transcript with Rich Rendering

**As** Maya (Solo Dev), **I want** to view the full chat transcript with syntax highlighting and tool call expansion **so that** I can understand exactly what happened in a session.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] Messages rendered with Markdown formatting.
- [ ] Code blocks syntax-highlighted by language.
- [ ] Tool calls expandable to show input and output.
- [ ] Search within transcript (Ctrl+F / Cmd+F).
- [ ] Auto-scroll with manual override.
- [ ] Large transcripts (50K+ events) render without lag (virtualized list).

### US-OB-007: Export Observability Data

**As** David (Team Lead), **I want** to export cost and usage data **so that** I can report to leadership and finance.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Export formats: CSV, JSON.
- [ ] Filter by: date range, agent, project, team.
- [ ] Includes: session ID, agent, project, start time, duration, input tokens, output tokens, cost, status.
- [ ] Scheduled export (daily/weekly email) configurable via cron.

### US-OB-008: Decision Chain Audit Trail

**As** James (DevOps), **I want** to see the full decision chain for every agent action **so that** I can audit what the system decided and why.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Every routing decision (model selection, tool selection, agent selection) logged with timestamp and reasoning.
- [ ] Decision chain viewable per session.
- [ ] Filterable by decision type.
- [ ] Exportable in JSONL format.
- [ ] Retained for configurable period (default 90 days).

---

## Epic 8: MCP Integration

> As a user, I can use Forge as an MCP server to integrate it with other AI tools and platforms.

### US-MC-001: Start Forge as MCP Server

**As** Rina (Tool Builder), **I want** to start Forge in MCP server mode **so that** other tools can invoke Forge capabilities programmatically.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] `forge --mcp` starts Forge in MCP server mode (no UI).
- [ ] `forge --mcp --transport stdio` for direct process invocation.
- [ ] `forge --mcp --transport sse --port 4174` for HTTP-based clients.
- [ ] Server responds to MCP handshake and capability negotiation.
- [ ] All MCP tools, resources, and prompts are advertised on connection.

### US-MC-002: Invoke Agent via MCP Tool

**As** Rina (Tool Builder), **I want** to create and run agents via MCP tool calls **so that** my platform can orchestrate agents programmatically.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] `create_agent` tool accepts: name, model, system_prompt, working_directory, and returns agent_id.
- [ ] `run_agent` tool accepts: agent_id, prompt, and returns session_id.
- [ ] `stop_agent` tool accepts: agent_id and returns success/failure.
- [ ] `list_agents` tool returns all agent definitions.
- [ ] `get_agent_status` tool returns: running/stopped, session_id, circuit_breaker_state, cost_so_far.
- [ ] All responses are structured JSON.

### US-MC-003: Subscribe to Events via MCP Resource

**As** Rina (Tool Builder), **I want** to subscribe to the event stream via MCP **so that** my platform can show real-time progress.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] `forge://events` resource provides a subscription to the event stream.
- [ ] Events include: type, agent_id, session_id, timestamp, data.
- [ ] Filter parameter to subscribe to specific event types or agents.
- [ ] Back-pressure handling: events buffered if client is slow, oldest dropped after buffer full.
- [ ] Reconnection re-subscribes without replaying missed events (or optionally with replay from cursor).

### US-MC-004: Query Sessions via MCP

**As** Rina (Tool Builder), **I want** to search and export sessions via MCP **so that** my platform can display session history.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] `search_sessions` tool accepts: query (FTS), project, date_range, and returns matching sessions.
- [ ] `get_session` tool accepts: session_id and returns full session with messages and metadata.
- [ ] `export_session` tool accepts: session_id, format (json/markdown), and returns the exported content.
- [ ] `resume_session` tool accepts: session_id and returns a new running session.

### US-MC-005: Manage Skills via MCP

**As** Rina (Tool Builder), **I want** to search and install skills via MCP **so that** my platform can configure agents with the right skills.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] `search_skills` tool accepts: query, category, and returns matching skills with grades.
- [ ] `install_skill` tool accepts: skill_id or URL and returns success/failure.
- [ ] `activate_skill` tool accepts: skill_id, agent_id and activates the skill for that agent.
- [ ] `list_installed_skills` tool returns all installed skills with metadata.

### US-MC-006: Run Git Operations via MCP

**As** Rina (Tool Builder), **I want** to perform Git operations via MCP **so that** my platform can manage code changes programmatically.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] `git_status` tool returns staged, modified, untracked files.
- [ ] `git_diff` tool accepts: path (optional), base (optional), and returns diff content.
- [ ] `git_commit` tool accepts: message, files (optional), and creates a commit.
- [ ] `create_worktree` tool accepts: branch_name, path, and creates a worktree.
- [ ] `remove_worktree` tool accepts: path and removes a worktree.

### US-MC-007: Use MCP Prompt Templates

**As** Rina (Tool Builder), **I want** to use Forge's prompt templates via MCP **so that** my platform generates consistent, high-quality prompts.

**Priority**: P2 | **Points**: 3

**Acceptance Criteria**:
- [ ] Prompt templates for: create_feature, fix_bug, review_code, write_tests, refactor, document.
- [ ] Each template accepts parameters (e.g., file_paths, issue_description, language).
- [ ] Templates return formatted prompts ready to use with `run_agent`.
- [ ] Templates reference appropriate agent presets and workflow levels.

### US-MC-008: Permission Bypass for Trusted Orchestrators

**As** Rina (Tool Builder), **I want** to run Forge in permission-bypass mode **so that** my automated platform does not get blocked by permission prompts.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] `--dangerously-skip-permissions` flag enables auto-approval of all permission prompts.
- [ ] Flag only works in MCP mode (not in UI mode).
- [ ] All auto-approved actions are logged to the audit trail with "bypass" flag.
- [ ] Circuit breaker and rate limiter remain active in bypass mode.
- [ ] Startup warning displayed when bypass mode is active.

---

## Epic 9: Configuration and Security

> As a user, I can configure Forge with hierarchical scopes and trust that security policies are enforced.

### US-CS-001: Configure Settings via UI

**As** Maya (Solo Dev), **I want** a graphical settings page **so that** I can configure Forge without editing TOML files.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Settings organized by section: agents, safety, workflows, notifications, security, appearance.
- [ ] Each setting shows: current value, default value, scope (global/user/project/agent).
- [ ] Inline documentation for every setting.
- [ ] Real-time validation with error display.
- [ ] Changes saved immediately to the appropriate scope file.

### US-CS-002: Override Settings at Project Scope

**As** David (Team Lead), **I want** to set project-specific configurations that override global defaults **so that** each project has appropriate settings.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] `.claude-forge.toml` in project root overrides global and user settings.
- [ ] Settings UI shows which scope each setting comes from.
- [ ] Merge order: global < user < project < agent.
- [ ] Invalid project config produces warning but does not prevent Forge from starting.

### US-CS-003: Import and Export Configuration

**As** David (Team Lead), **I want** to export my configuration and import it on another machine **so that** I can standardize settings across the team.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Export produces a single TOML file with all scopes merged.
- [ ] Import applies settings, respecting scope hierarchy.
- [ ] Selective import: choose which sections to apply.
- [ ] Dry-run mode: show what would change without applying.
- [ ] Import/export available via UI button and CLI command.

### US-CS-004: Read Existing Claude Code Config Files

**As** Maya (Solo Dev), **I want** Forge to read my existing `CLAUDE.md`, `settings.json`, and `.ralphrc` files **so that** I do not lose my existing configuration.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] `CLAUDE.md` content injected as system prompt additions for agents in that project.
- [ ] `settings.json` mapped to equivalent Forge settings.
- [ ] `.ralphrc` safety settings imported (rate limits, circuit breaker thresholds).
- [ ] `codemcp.toml` shell commands imported.
- [ ] `SKILL.md` files discovered and imported into skill catalog.
- [ ] Import happens automatically on first run; user can re-trigger manually.

### US-CS-005: View and Export Audit Log

**As** James (DevOps), **I want** to view and export the audit log **so that** I can verify compliance and investigate incidents.

**Priority**: P0 | **Points**: 5

**Acceptance Criteria**:
- [ ] Audit log page shows: timestamp, event type, actor (agent/user), target, details.
- [ ] Filterable by: event type, agent, date range, severity.
- [ ] Export in JSONL format for SIEM integration.
- [ ] Tamper-detection checksums on audit entries.
- [ ] Retention configurable (default 90 days); old entries archived before deletion.

### US-CS-006: Run Health Check

**As** Rina (Tool Builder), **I want** to run a health check that validates the entire system **so that** I can diagnose issues in my production deployment.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Health check validates: config files, database integrity, MCP server connectivity, Git availability, skill catalog, plugin status.
- [ ] Each check reports: pass, warn, or fail with details.
- [ ] Available via: UI button, CLI (`forge health`), MCP tool, and HTTP endpoint (`GET /health`).
- [ ] JSON response format for programmatic consumption.
- [ ] Completes in under 5 seconds.

---

## Epic 10: Notifications and Scheduling

> As a user, I can receive notifications about agent events and schedule agent runs for later.

### US-NS-001: Receive Desktop Notifications

**As** Maya (Solo Dev), **I want** to receive desktop notifications when agents complete or fail **so that** I do not have to keep Forge in the foreground.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] macOS notifications via native notification center.
- [ ] Linux notifications via libnotify.
- [ ] Notification on: agent complete, agent failed, budget warning.
- [ ] Sound alert for failures (configurable).
- [ ] Click notification to open Forge and navigate to the relevant session.

### US-NS-002: Send Webhook Notifications

**As** David (Team Lead), **I want** to send notifications to a webhook URL **so that** I can integrate with Slack, Teams, or custom systems.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Webhook URL configurable per notification rule.
- [ ] Payload: JSON with event type, agent, session, message, timestamp.
- [ ] HMAC-SHA256 signature header for receiver authentication.
- [ ] Retry policy: 3 attempts with exponential backoff (1s, 4s, 16s).
- [ ] Delivery status logged (success, failed after retries).

### US-NS-003: Send Telegram Notifications

**As** Maya (Solo Dev), **I want** to receive Telegram notifications and reply to execute commands **so that** I can monitor and control agents from my phone.

**Priority**: P2 | **Points**: 5

**Acceptance Criteria**:
- [ ] Telegram bot setup wizard (BotFather token entry).
- [ ] Notifications: agent complete, agent failed, budget warning, scheduled run result.
- [ ] Reply-to-send: replying to a notification executes the reply text as a prompt for that agent.
- [ ] Multi-project thread management (separate Telegram threads per project).
- [ ] Rate limiting on incoming commands (prevent abuse).

### US-NS-004: Schedule Agent Runs with Cron

**As** Maya (Solo Dev), **I want** to schedule agent runs using cron expressions **so that** I can automate recurring tasks like nightly test runs.

**Priority**: P1 | **Points**: 5

**Acceptance Criteria**:
- [ ] Schedule editor with cron expression input and human-readable preview (e.g., "Every day at 2:00 AM").
- [ ] Each job specifies: agent, prompt, working directory, notification channel.
- [ ] Job history with: last run time, status (success/failure), cost, duration.
- [ ] Jobs persist across Forge restarts.
- [ ] Maximum concurrent scheduled jobs configurable (default: 3).

### US-NS-005: Configure Notification Rules

**As** David (Team Lead), **I want** to define notification rules that route specific events to specific channels **so that** the right people get the right alerts.

**Priority**: P1 | **Points**: 3

**Acceptance Criteria**:
- [ ] Rules specify: event type pattern, severity threshold, channel(s), and message template.
- [ ] Default rules: agent failure -> desktop + webhook, budget exceeded -> all channels, scheduled complete -> email.
- [ ] Rule priorities for when multiple rules match.
- [ ] Test button to send a test notification via each configured channel.
- [ ] Rules stored in configuration (TOML) and editable in settings UI.

### US-NS-006: Send Email Notifications

**As** James (DevOps), **I want** to receive email notifications about CI agent results **so that** the results integrate with my existing email-based workflow.

**Priority**: P2 | **Points**: 3

**Acceptance Criteria**:
- [ ] SMTP server configuration in settings (host, port, username, password, TLS).
- [ ] Email includes: subject with event type and agent name, body with session summary.
- [ ] HTML and plain text formats.
- [ ] Optional attachment: full session export.
- [ ] Email delivery logged in audit trail.

---

## Story Summary Statistics

| Epic | Stories | P0 | P1 | P2 | Total Points |
|------|---------|----|----|----|----|
| 1. Agent Management | 10 | 5 | 3 | 2 | 44 |
| 2. Safety and Reliability | 8 | 4 | 3 | 1 (P1 counted) | 37 |
| 3. Workflows | 8 | 1 | 5 | 2 | 41 |
| 4. Session Management | 8 | 3 | 4 | 1 | 27 |
| 5. Skills and Plugins | 8 | 1 | 6 | 1 | 39 |
| 6. Development Environment | 8 | 0 | 5 | 3 | 39 |
| 7. Observability | 8 | 3 | 4 | 1 (P1 counted) | 37 |
| 8. MCP Integration | 8 | 2 | 5 | 1 | 32 |
| 9. Configuration and Security | 6 | 2 | 3 | 1 (P1 counted) | 24 |
| 10. Notifications and Scheduling | 6 | 0 | 4 | 2 | 22 |
| **Total** | **78** | **21** | **42** | **15** | **342** |
