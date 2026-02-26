# Claude Forge -- Milestones

> 8 milestones (M0-M7) with exit criteria, demo scenarios, and acceptance checklists.
> A milestone is complete only when ALL acceptance criteria are met.

---

## Milestone Summary

| ID | Name | Target Week | Phase | Status |
|----|------|------------|-------|--------|
| M0 | Foundation Complete | Week 3 | Phase 0 | Planned |
| M1 | Safety + MCP | Week 7 | Phase 1 | Planned |
| M2 | Workflows + Skills | Week 12 | Phase 2 | Planned |
| M3 | Observability + Git | Week 17 | Phase 3 | Planned |
| M4 | Notifications + Scheduler | Week 21 | Phase 4 | Planned |
| M5 | Plugins + Security | Week 27 | Phase 5 | Planned |
| M6 | Dev Environment | Week 32 | Phase 6 | Planned |
| M7 | Production Ready | Week 32+ | Post-Phase 6 | Planned |

---

## M0: Foundation Complete

**Target:** Week 3
**Theme:** Clean workspace structure, event bus, and database schema supporting all future features.

### Deliverables Checklist

- [ ] 12 workspace crates created and compiling
- [ ] `forge-core` crate: Event types, Agent/Session/Workflow types, traits (EventSink, Repository)
- [ ] `forge-db` crate: SQLite connection pooling, WAL mode, migration runner
- [ ] `forge-api` crate: Axum server with existing routes migrated
- [ ] Database v2 schema with tables for: agents, sessions, events, workflows, skills, schedules, plugins, audit_log
- [ ] FTS5 virtual tables for: skills, sessions, events
- [ ] Event bus: broadcast channel with filtering, batch writer (50 events / 2s flush)
- [ ] All existing features working: agent CRUD, process spawning, WebSocket streaming, session browser
- [ ] `forge-test-utils` crate with TestDb, fixtures, builders
- [ ] CI pipeline: fmt, clippy, nextest, build
- [ ] `#![forbid(unsafe_code)]` in all workspace crates

### Demo Scenario

> Start the Forge binary. Create an agent via the UI. Run the agent with a prompt.
> See real-time output streaming via WebSocket. Verify the agent and events are
> persisted in the database. Stop the binary and restart -- data is preserved.
> Run `cargo test --workspace` -- all tests pass. Run `cargo clippy --workspace` -- no warnings.

### Acceptance Criteria

1. `cargo build --release` produces a single binary under 40 MB.
2. The binary starts and serves the UI at `http://localhost:4173`.
3. All pre-refactor features pass manual smoke testing.
4. `cargo test --workspace` passes with zero failures.
5. `cargo clippy --workspace -- -D warnings` produces zero warnings.
6. Database migration from v1 to v2 succeeds without data loss (tested on a real DB).
7. Event bus delivers events to WebSocket clients with < 10ms latency.
8. Batch writer flushes 50 events in a single transaction in < 50ms.
9. FTS5 search returns results for test queries in < 20ms.

---

## M1: Safety + MCP

**Target:** Week 7
**Theme:** Agents operate within safety boundaries. External tools connect via MCP.

### Deliverables Checklist

- [ ] `forge-safety` crate: CircuitBreaker, RateLimiter, CostTracker
- [ ] Circuit breaker: three-state machine (Closed/Open/HalfOpen), per-agent and global
- [ ] Rate limiter: token bucket, configurable per-agent and per-model
- [ ] Cost tracker: token counting, model pricing, budget enforcement
- [ ] Cost dashboard: daily/weekly/monthly charts, per-agent breakdown
- [ ] `forge-mcp` crate: MCP protocol implementation (JSON-RPC)
- [ ] MCP transport: stdio and SSE
- [ ] 10 MCP tools: agent_create, agent_list, agent_get, agent_run, agent_stop, session_list, session_get, workflow_run, skill_search, config_get
- [ ] 5 MCP resources: agent://list, session://list, skill://catalog, config://current, status://health
- [ ] MCP protocol compliance tests
- [ ] Safety dashboard in frontend: circuit states, rate limits, cost budgets

### Demo Scenario

> Set a cost budget of $5/day. Run three agents that collectively would cost $8/day.
> Observe the cost tracker warning at $4 (soft limit) and blocking at $5 (hard limit).
> The third agent is rejected with a clear "budget exceeded" error.
>
> Force-fail an agent 5 times in a row. Observe the circuit breaker opening and
> subsequent requests being rejected immediately (fail-fast). Wait for the timeout
> period. Observe the circuit entering half-open. Send one successful request.
> Observe the circuit closing.
>
> Connect Claude Desktop to Forge via MCP. Use the `agent_list` tool to see agents.
> Use `agent_run` to start an agent. See results stream back through MCP.

### Acceptance Criteria

1. Circuit breaker opens after configurable N consecutive failures.
2. Circuit breaker transitions HalfOpen -> Closed after configurable N successes.
3. Rate limiter rejects requests exceeding configured rate, returns 429 with Retry-After header.
4. Cost tracker calculates costs within 5% of actual API charges.
5. Budget enforcement blocks agent execution when hard limit is reached.
6. MCP server starts and accepts connections via stdio.
7. All 10 MCP tools respond correctly to valid requests.
8. MCP tools return proper JSON-RPC errors for invalid requests.
9. MCP resources return current data in standard format.
10. Claude Desktop can connect and list agents via MCP.

---

## M2: Workflows + Skills

**Target:** Week 12
**Theme:** Multi-step automated workflows and a searchable catalog of 1,537 skills.

### Deliverables Checklist

- [ ] `forge-workflow` crate: WorkflowEngine, step types, state machine
- [ ] Step types: Prompt, Parallel, Conditional, Loop, Handoff
- [ ] Workflow persistence: survives binary restart
- [ ] Workflow YAML/JSON definition format
- [ ] 5 built-in workflow templates: review-code, refactor, test-write, doc-gen, debug
- [ ] Workflow builder UI (form-based)
- [ ] Workflow run visualization (step progress, current step)
- [ ] `forge-skills` crate: SkillCatalog, search, categories
- [ ] 1,537 skills indexed in FTS5
- [ ] 13 top-level skill categories
- [ ] Skill detail view with parameters and examples
- [ ] Slash-command autocomplete (`/` trigger)
- [ ] Skill-to-prompt compilation
- [ ] Skill usage tracking (most-used, recently-used)

### Demo Scenario

> Open the workflow builder. Create a "Code Review" workflow with 3 steps:
> 1. Agent reads the file and identifies issues (Prompt step)
> 2. Three agents review for security, performance, and style in parallel (Parallel step)
> 3. A summarizer agent combines all reviews (Prompt step)
>
> Run the workflow on a target file. Watch all 5 agents execute, with the parallel
> group running concurrently. See the final summary combining all reviews.
>
> Navigate to the Skills page. Type "test" in the search box. See relevant skills
> appear instantly. Click "Write Unit Tests" skill. See the parameter form.
> Execute the skill -- it compiles into a prompt and runs an agent.
>
> In the agent prompt box, type `/`. See the autocomplete dropdown with
> fuzzy-matched skills. Select one and it fills in the prompt.

### Acceptance Criteria

1. Sequential workflow with 5 steps completes, passing output between steps.
2. Parallel workflow with 3 branches runs all branches concurrently.
3. Conditional workflow chooses correct branch based on prior step output.
4. Loop workflow iterates N times or until condition is met.
5. Handoff step passes context from one agent to another.
6. Workflow persists to DB and can be resumed after restart.
7. All 1,537 skills are searchable via FTS5.
8. Skill search returns relevant results in < 20ms for common queries.
9. Slash-command autocomplete appears within 100ms of typing `/`.
10. Skill-to-prompt compilation produces valid agent prompts.

---

## M3: Observability + Git

**Target:** Week 17
**Theme:** Full operational visibility and native git integration.

### Deliverables Checklist

- [ ] `forge-observe` crate: MetricsCollector, Dashboard data aggregation
- [ ] Metrics instrumentation across all crates
- [ ] Main dashboard: active agents, recent runs, system health, uptime
- [ ] Cost dashboard: time-series charts, per-agent breakdown, budget utilization
- [ ] Agent swim lane visualization (parallel timeline)
- [ ] Session timeline (event sequence)
- [ ] `forge-git` crate: GitRepo wrapper over git2
- [ ] Git status: modified, staged, untracked, ignored files
- [ ] Git diff: file-level and hunk-level with syntax highlighting
- [ ] Git log: commit history with stats
- [ ] Git branch: list, create, delete
- [ ] Git worktree: list, create, remove
- [ ] Git panel in frontend: status summary, diff viewer, log
- [ ] Real-time dashboard updates via WebSocket

### Demo Scenario

> Open the main dashboard. See 3 agents currently running with live token counts.
> Click into the cost dashboard -- see today's spend at $2.47 across 12 agent runs,
> with a bar chart showing per-agent costs. Budget shows 49% utilized.
>
> Switch to the swim lane view. See all 3 running agents as parallel horizontal
> bars, with events marked on the timeline. Click an event to see its details.
>
> Open the Git panel. See the current repo status: 3 modified files, 1 staged.
> Click a modified file to see the diff with syntax highlighting. Navigate to
> the commit log -- see the last 20 commits with messages and authors.
>
> Create a worktree for an agent to work in isolation. Verify the agent's
> changes do not affect the main working tree.

### Acceptance Criteria

1. Dashboard loads and renders in < 1 second with 100 historical agent runs.
2. Cost tracking matches actual API bills within 5% margin.
3. Swim lane correctly shows parallel agent execution with accurate timing.
4. Git status matches `git status` output for 5 test repositories.
5. Git diff displays correct hunks with syntax highlighting.
6. Git log shows correct commit history (verified against `git log`).
7. Worktree creation completes in < 2 seconds.
8. Worktree provides isolated working directory (verified by file operations).
9. Dashboard updates in real-time (< 2 second delay) when new events arrive.
10. All git operations work on repositories with 10K+ files.

---

## M4: Notifications + Scheduler

**Target:** Week 21
**Theme:** Unattended operation with automatic notifications and scheduled executions.

### Deliverables Checklist

- [ ] `forge-notify` crate: NotificationService, 4 channels
- [ ] WebSocket notifications (in-app, real-time)
- [ ] Desktop notifications (macOS/Linux native)
- [ ] Webhook notifications (HTTP POST with configurable URL)
- [ ] Email notifications (SMTP)
- [ ] Notification routing: per-agent channel preferences
- [ ] Notification templates (Markdown-based)
- [ ] Notification center UI: bell icon, unread count, history
- [ ] `forge-scheduler` crate: Scheduler, cron expression support
- [ ] Job types: agent run, workflow execution, report generation
- [ ] Job persistence (survives restart)
- [ ] Scheduler dashboard: upcoming jobs, recent completions, job history
- [ ] Hierarchical configuration: defaults < global < project < agent
- [ ] Settings UI with sections
- [ ] Config export/import

### Demo Scenario

> Set up a webhook notification for the "deployment-agent" to POST to a Slack
> webhook URL when it completes. Run the agent. When it finishes, the webhook
> fires within 1 second. The Slack channel receives a formatted message with
> the agent name, duration, and result summary.
>
> Create a scheduled job: run the "code-review" workflow every weekday at 9am
> on the main branch. Verify the schedule appears in the scheduler dashboard
> with the correct next-run time. (In the demo, trigger manually to show execution.)
>
> Open Settings. Change the default model from sonnet to opus at the global level.
> Navigate to a specific agent and override the model back to sonnet. Verify
> the agent uses sonnet while other agents use opus.

### Acceptance Criteria

1. WebSocket notification appears in-app within 2 seconds of trigger event.
2. Desktop notification appears within 5 seconds of trigger event.
3. Webhook delivers HTTP POST within 1 second of trigger event, with correct payload.
4. Email notification sent within 10 seconds of trigger event.
5. Notification routing correctly sends to configured channels per agent.
6. Notification center shows unread count and supports mark-as-read.
7. Scheduled job executes within 1 second of cron-specified time.
8. Scheduled jobs survive binary restart (persisted in DB).
9. Configuration hierarchy resolves correctly: agent > project > global > defaults.
10. Settings UI updates take effect immediately (no restart required).

---

## M5: Plugins + Security

**Target:** Week 27
**Theme:** Community extensibility via WASM plugins, production-grade security, and polish.

### Deliverables Checklist

- [ ] `forge-plugins` crate: PluginHost (Wasmtime), WIT interface
- [ ] Plugin API: read agents, emit events, access config, call skills
- [ ] Resource limits: memory (64 MB), fuel (1M instructions), filesystem (scoped)
- [ ] Plugin lifecycle: install, enable, disable, uninstall, upgrade
- [ ] Plugin manifest (TOML): name, version, author, permissions, entry point
- [ ] Plugin registry UI: browse, install, configure, enable/disable
- [ ] 3 example plugins: custom-formatter, slack-integration, metrics-exporter
- [ ] Plugin SDK: Rust template with build instructions
- [ ] Audit log: all state-changing operations recorded
- [ ] Audit log viewer with filtering (by actor, action, time range)
- [ ] Permission model: agent access scoping, plugin capability restrictions
- [ ] Secret management: encrypted at-rest storage for API keys, tokens
- [ ] Security hardening: input validation, CORS, CSP headers, rate limiting on all endpoints
- [ ] Performance optimization pass
- [ ] Accessibility audit (WCAG 2.1 AA)
- [ ] Keyboard shortcuts
- [ ] User documentation (user guide, API reference, plugin dev guide)

### Demo Scenario

> Install the "slack-integration" plugin from a local .wasm file. The plugin
> manifest requests permissions: read-agents, emit-events, network (slack.com only).
> Approve the permissions. The plugin appears in the registry as enabled.
>
> The plugin subscribes to agent-completed events and posts summaries to a
> Slack channel. Run an agent. See the Slack message appear. Verify the plugin
> cannot access the filesystem (attempts are denied and logged in the audit log).
>
> Open the audit log viewer. Filter by "plugin" actions. See the installation,
> permission grant, event subscription, and the denied filesystem access attempt.
>
> Navigate to the security settings. See the secret manager with stored API keys
> (values hidden). Add a new key. Verify it persists across restarts (encrypted).

### Acceptance Criteria

1. WASM plugin loads from .wasm file in < 100ms.
2. Plugin executes within fuel limit (verified: exceeding fuel aborts cleanly).
3. Plugin cannot exceed memory limit (verified: OOM results in clean abort).
4. Plugin cannot access files outside its scoped directory (verified: access denied).
5. Plugin API allows reading agents, emitting events, and accessing config.
6. All 3 example plugins install and function correctly.
7. Plugin SDK produces a buildable template that compiles to a valid .wasm.
8. Audit log captures all CRUD operations, agent runs, and plugin actions.
9. Audit log viewer supports filtering by actor, action type, and time range.
10. Secrets are stored encrypted at rest (verified: raw DB inspection shows no plaintext).
11. All API endpoints return proper error responses for invalid input (no 500s on bad data).
12. Lighthouse accessibility score > 90 for all pages.
13. Binary size < 50 MB with all features, < 25 MB without plugins.

---

## M6: Dev Environment

**Target:** Week 32
**Theme:** Full development environment within Forge -- code viewing, terminal, and file exploration.

### Deliverables Checklist

- [ ] File read API: read files with line range support, size limit enforcement
- [ ] Code viewer: syntax highlighting (50+ languages via shiki)
- [ ] Code viewer: line numbers, word wrap toggle, in-file search
- [ ] Code viewer: go-to-line, file tabs, unsaved indicators
- [ ] Side-by-side diff view (integrated with Phase 3 git diff)
- [ ] PTY management: allocate, resize, destroy
- [ ] Embedded terminal: xterm.js with WebSocket connection
- [ ] Terminal: multiple tabs, shell detection, working directory sync
- [ ] Terminal: scrollback buffer (10,000 lines default)
- [ ] File explorer: directory tree with lazy loading
- [ ] File explorer: file type icons, .gitignore filtering
- [ ] File explorer: right-click context menu
- [ ] File search: fuzzy find by path
- [ ] Integration: file explorer -> code viewer, context menu -> terminal

### Demo Scenario

> Open Forge. The left panel shows a file explorer for the current project.
> Expand `src/` to see the directory tree. Click `main.rs` -- it opens in the
> code viewer with Rust syntax highlighting and line numbers.
>
> Open a second file tab for `Cargo.toml`. Switch between tabs. Use Cmd+G to
> go to line 42 in main.rs.
>
> Right-click a file in the explorer. Select "Open in Terminal". A terminal tab
> opens, cd'd to the file's directory. Run `cargo test` -- see output in real-time
> with proper color rendering.
>
> Open the git panel. See the diff for a modified file. Click "View Side-by-Side"
> to see old and new versions with syntax highlighting.

### Acceptance Criteria

1. Code viewer opens files < 100KB in < 500ms with correct syntax highlighting.
2. Code viewer handles files > 1MB gracefully (virtual scrolling or size limit message).
3. Diff viewer shows correct side-by-side comparison with highlighted changes.
4. Terminal opens in < 500ms with user's default shell.
5. Terminal keystroke latency < 50ms (measured: keypress to display).
6. Terminal handles 256-color output correctly.
7. Terminal resize works smoothly when pane is resized.
8. File explorer loads directory with 1000+ files in < 1 second.
9. File explorer respects .gitignore (hidden files not shown by default).
10. File search returns results in < 200ms for projects with 5000+ files.
11. All integrations work: explorer -> viewer, explorer -> terminal, git -> diff viewer.

---

## M7: Production Ready

**Target:** Post-Week 32 (1.0 release prep)
**Theme:** Production hardening, documentation, and release infrastructure.

### Deliverables Checklist

- [ ] Performance: all targets from TECH_STACK.md met
- [ ] Reliability: 24-hour soak test with 10 concurrent agents, no crashes, no memory leaks
- [ ] Binary distribution: macOS (arm64, x86_64), Linux (x86_64, aarch64)
- [ ] Install script: `curl -sSf https://install.claude-forge.dev | sh`
- [ ] Homebrew formula: `brew install claude-forge`
- [ ] Changelog: complete from 0.1 to 1.0
- [ ] User guide: getting started, agent management, workflows, skills, plugins
- [ ] API reference: all endpoints documented with request/response examples
- [ ] Plugin developer guide: SDK setup, API reference, publishing
- [ ] Contributing guide: development setup, coding standards, PR process
- [ ] CI/CD: automated release pipeline (tag -> build -> test -> publish)
- [ ] Telemetry: opt-in anonymous usage metrics (with clear disclosure)
- [ ] Update checker: notify user when new version is available
- [ ] Error reporting: opt-in crash reporting with redacted stack traces
- [ ] Migration: clean upgrade path from any previous version

### Demo Scenario

> Install Forge on a fresh macOS machine using `brew install claude-forge`.
> Run `claude-forge` -- it starts on port 4173. Open the browser.
>
> Follow the Getting Started guide: create an agent, run a task, view results.
> Install a community plugin. Set up a scheduled workflow. Configure notifications.
>
> Leave Forge running for 24 hours with periodic agent runs. Verify memory
> stays stable, no crashes, no data corruption. Check the cost dashboard --
> all runs accounted for accurately.

### Acceptance Criteria

1. Binary installs and starts in < 10 seconds on a fresh machine.
2. No panics or crashes in 24-hour soak test.
3. Memory usage stable (no growth beyond expected baseline) over 24 hours.
4. All documented features work as described in user guide.
5. API reference matches actual endpoint behavior (automated verification).
6. Plugin SDK produces working plugin on first try (tested by external tester).
7. Upgrade from any version to 1.0 preserves all data (tested migration chain).
8. Release pipeline produces binaries for all 4 platform targets.
9. Install script works on macOS and Linux with single command.
10. `claude-forge --version` shows correct version and build info.

---

## Milestone Dependency Chain

```
M0 (Foundation) ---- required for all subsequent milestones
  |
  +---> M1 (Safety + MCP)
  |       |
  |       +---> M2 (Workflows + Skills)
  |       |       |
  |       |       +---> M3 (Observability + Git)
  |       |       |       |
  |       |       |       +---> M6 (Dev Environment)
  |       |       |
  |       |       +---> M4 (Notifications + Scheduler)
  |       |
  |       +---> M5 (Plugins + Security)
  |
  +---> M7 (Production Ready) ---- requires ALL milestones M0-M6
```

**Key constraints:**
- M2 requires M1 (safety controls limit workflow costs)
- M3 requires M1 (cost data flows from safety crate)
- M4 requires M2 (scheduled workflows need workflow engine)
- M5 requires M1 (plugin resource limits use safety infrastructure)
- M6 requires M3 (diff viewer uses git integration)
- M7 requires everything (production readiness validates all features)
