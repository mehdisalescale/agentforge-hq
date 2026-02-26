# Claude Forge -- Product Journey

> From empty workspace to production-ready 1.0.
> 6 phases. 24 weeks. One binary. Zero config.

---

## Phase 0: Foundation Build (Weeks 1-4)

**Goal**: Empty workspace to running binary with UI shell.

| Build | Details |
|-------|---------|
| 8 Rust workspace crates | `forge-core`, `forge-db`, `forge-api`, `forge-agent`, `forge-process`, `forge-safety`(stub), `forge-mcp`(stub), `forge-app` |
| Event system | `ForgeEvent` enum, broadcast bus, batch writer to SQLite |
| Database schema | Designed for ALL phases upfront -- agents, sessions, events, workflows, skills, schedules, audit, config + FTS5 |
| API skeleton | Axum router, health check, WebSocket endpoint, CORS, request tracing |
| Frontend shell | SvelteKit + Svelte 5 + TailwindCSS 4, layout with sidebar nav, empty pages |
| Single binary | `rust-embed` serves frontend, `./forge` opens in browser |

**User gets**: A binary that starts, shows a UI, connects WebSocket. Nothing functional yet.

---

## Phase 1: Agent Engine (Weeks 5-8)

**Goal**: Create agents, send prompts, watch responses stream live.

| Build | Details |
|-------|---------|
| Agent CRUD | Create/edit/delete agents, 9 presets (CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer) |
| Process spawning | `claude -p` with `--output-format stream-json`, `--resume`, environment handling |
| Real-time streaming | Process output -> event bus -> WebSocket -> Svelte UI with Markdown, code blocks, tool calls, thinking blocks |
| Multi-pane layout | Run multiple agents side-by-side in tabs |
| Session management | Session CRUD, history, resume, export (JSON/Markdown), browse existing `~/.claude/projects/` sessions |

**User gets**: Create an agent, send it a prompt, watch the response stream in real-time. Run 3 agents in parallel. Resume old sessions. Export results.

---

## Phase 2: Workflows + Skills (Weeks 9-13)

**Goal**: Chain agents into multi-step workflows, search 1,500+ skills.

| Build | Details |
|-------|---------|
| Workflow engine | YAML/JSON DSL, 5 step types (Prompt, Parallel, Conditional, Loop, Handoff), state machine, persistence, resume |
| Workflow UI | List, form-based builder, run visualization with step progress, run history |
| 5 built-in templates | review-code, refactor, test-write, doc-gen, debug |
| Skill catalog | Import 1,500+ skills from reference repos, FTS5 search, 13 categories |
| Skill execution | Skill -> prompt compilation, chaining, result formatting, usage tracking |
| Slash-command autocomplete | Type `/` -> fuzzy search skills |

**User gets**: Define "review this PR then write tests then update docs" as a workflow. Search skills by name. Autocomplete slash commands.

---

## Phase 4: Safety + MCP (Weeks 9-12, parallel with Phase 2)

**Goal**: Protect against runaway agents, expose Forge to external tools.

| Build | Details |
|-------|---------|
| Circuit breaker | 3-state machine (Closed/Open/HalfOpen), per-agent + global, dashboard widget |
| Rate limiter | Token bucket (per-agent, per-model, global) |
| Cost tracking | Token counts x model pricing, budget enforcement (hard/soft limits), daily/weekly/monthly reports |
| MCP server | JSON-RPC over stdio/SSE, 10 tools (agent_create, agent_run, workflow_run, skill_search...), 5 resources |
| Safety dashboard | Circuit states, rate limits, cost charts |

**User gets**: Set "$50/day budget" -- agents stop when reached. Connect Claude Desktop to Forge via MCP. Circuit breaker prevents cascading failures.

---

## Phase 3: Observability + Git (Weeks 14-18)

**Goal**: See everything agents do, track costs, manage code changes.

| Build | Details |
|-------|---------|
| Metrics | Token usage, latency, throughput, active agents -- all instrumented via `metrics` crate |
| Main dashboard | Active agents, recent runs, system health, real-time updates |
| Cost dashboard | Spend over time, per-agent breakdown, budget usage |
| Agent swim lanes | Parallel timeline showing what each agent is doing |
| Git integration | `forge-git` wrapping libgit2 -- status, diff, log, branch, worktree |
| Git panel | Status summary, diff viewer with syntax highlighting, commit log |
| Worktree management | Create/remove worktrees for agent isolation |

**User gets**: Dashboard showing 5 agents running, their costs, their git changes. Click to see diffs. Create isolated worktrees per agent.

---

## Phase 5: Plugins + Security + Polish (Weeks 19-24) -- 1.0 RELEASE

**Goal**: Community extensibility, hardened security, production-ready.

| Build | Details |
|-------|---------|
| WASM plugin host | Wasmtime runtime, WIT interface, resource limits (64MB mem, 1M fuel), scoped filesystem |
| Plugin lifecycle | Install/enable/disable/uninstall, manifest (TOML), registry UI |
| 3 example plugins | custom-formatter, slack-integration, metrics-exporter |
| Plugin SDK | Rust template for plugin authors |
| Audit log | All state changes logged with actor/timestamp/details, viewer with filtering |
| Permission model | What agents/plugins can access |
| Secret management | Encrypted storage for API keys |
| Security hardening | Input validation, CORS, CSP, rate limiting on all endpoints |
| Polish | Performance profiling, error messages, loading states, keyboard shortcuts, binary < 35MB |
| Documentation | User guide, API reference, plugin development guide |
| CI/CD | GitHub Actions: build for macOS/Linux/Windows, sign, notarize, publish to Releases |

**User gets**: Install community plugins. Audit trail of everything. Encrypted API key storage. Production binary on GitHub Releases for all platforms.

---

## Phase 6: Dev Environment (Post-1.0, Weeks 25-29)

**Goal**: Forge becomes an IDE-like experience.

| Build | Details |
|-------|---------|
| Code viewer | Syntax highlighting (shiki, 50+ languages), side-by-side diff, file tabs |
| Embedded terminal | PTY + xterm.js, multiple tabs, working directory sync |
| File explorer | Lazy-loading tree, file search, .gitignore respect, click -> opens in viewer |

**User gets**: View agent's code changes, open a terminal, browse files -- all without leaving Forge.

---

## The Full Picture

```
Phase 0    ./forge → browser shows empty UI
  │
Phase 1    Create agent → send prompt → watch streaming response
  │
Phase 2    Chain agents into workflows, search 1,500+ skills
  │         ↑ Phase 4 (parallel): safety controls + MCP server
Phase 3    Dashboards, cost tracking, git integration
  │
Phase 5    Plugins, security, polish → 1.0 RELEASE
  │
Phase 6    Code viewer, terminal, file explorer (post-1.0)
```

---

## Final Product (1.0)

One binary. Zero config.

```bash
curl -fsSL https://forge.dev/install.sh | sh
forge
```

Opens browser. You see:

- **Dashboard** -- active agents, costs, system health
- **Agents** -- 9 presets + custom, create in seconds
- **Workflows** -- multi-step automation (review -> test -> deploy)
- **Skills** -- 1,500+ searchable, slash-command autocomplete
- **Git** -- status, diff, worktrees per agent
- **Safety** -- circuit breakers, rate limits, budget caps
- **MCP** -- Claude Desktop connects to Forge as a tool server
- **Plugins** -- community extensions via WASM
- **Audit** -- full history of every action

All from a < 35MB binary that runs locally, never phones home, and tracks 62 upstream repos for continuous improvement.

---

## Reference Repos Feeding Each Phase

| Phase | Key Repos | What to Extract |
|-------|-----------|-----------------|
| 0 | `claude-code-tools` | Workspace structure, Tantivy FTS patterns |
| 1 | `ralph-claude-code`, `awesome-claude-code-subagents` | Process spawning, agent presets, exit detection |
| 2 | `Claude-Code-Workflow`, `claude-code-skills`, `claude-code-spec-workflow` | Workflow DSL, skill schema, 1,500+ skill definitions |
| 3 | `claude-code-hooks-multi-agent-observability`, `1code` | Dashboard patterns, git UI, worktree management |
| 4 | `claude-code-mcp`, `claude-code-hub` | MCP protocol, proxy patterns, safety hooks |
| 5 | `claude-code-plugins-plus-skills`, `claude-code-action` | Plugin architecture, GitHub Actions, security patterns |
| 6 | `1code`, `claude_code_bridge`, `claude-code-viewer` | Code viewer, terminal, file explorer patterns |

---

## Timeline

```
Weeks  1-4    Phase 0  Foundation Build
Weeks  5-8    Phase 1  Agent Engine
Weeks  9-13   Phase 2  Workflows + Skills        ← parallel
Weeks  9-12   Phase 4  Safety + MCP              ← parallel
Weeks  14-18  Phase 3  Observability + Git
Weeks  19-24  Phase 5  Plugins + Polish → 1.0
Weeks  25-29  Phase 6  Dev Environment (post-1.0)
```

**Critical path: 24 weeks** (under the 26-week vision target).
