# Feature Source Map

> Complete mapping of all 62 reference repositories to the features they contribute to Claude Forge.

---

## Master Table

| # | Repo | Category | Features Contributed | Target Context | Type | Priority |
|---|------|----------|---------------------|----------------|------|----------|
| 1 | 1code | 01-Desktop & IDEs | Worktree-per-session, Kanban agent view, Git UI, cloud agents, plugin marketplace, PWA monitoring | Agent Mgmt, Git Ops, Observability | UI + Logic | P0 |
| 2 | claude-code-viewer | 01-Desktop & IDEs | Fuzzy session search (FTS5), cron scheduler, todo extraction, zero-loss validation, terminal panel, branch switcher | Session Lifecycle, Git Ops | UI + Logic | P0 |
| 3 | idea-claude-code-gui | 01-Desktop & IDEs | Conversation rewind, @file references, image context, dual AI engine | Session Lifecycle | UI | P1 |
| 4 | CodexBar | 01-Desktop & IDEs | Multi-provider usage meters, reset countdowns, incident badges | Observability | UI + Logic | P2 |
| 5 | claude-code.nvim | 01-Desktop & IDEs | Auto-reload on change, git root detection, multi-instance | Agent Mgmt | Logic | P3 |
| 6 | claude-code-ide.el | 01-Desktop & IDEs | WebSocket + JSON-RPC pattern, MCP tool routing | MCP Foundation | Logic | P2 |
| 7 | claude-code-chat | 01-Desktop & IDEs | Checkpoint system, clipboard integration | Session Lifecycle | UI | P2 |
| 8 | claude-code-webui | 01-Desktop & IDEs | Mobile-responsive chat, permission mode switcher | UI Framework | UI | P2 |
| 9 | Claude-Code-Workflow | 02-Orchestration | 4-level workflows, semantic CLI selection, dependency parallelism, JSON state files, CodexLens search, issue workflow | Workflow Engine | Logic | P0 |
| 10 | ralph-claude-code | 02-Orchestration | Circuit breaker (3-state FSM), rate limiter, dual exit gate, response analyzer, session timeout, file protection, integrity validation | Safety & Governance | Logic | P0 |
| 11 | claude_code_bridge | 02-Orchestration | Cross-model sync, auto-spawning daemons, split-pane layout | Agent Mgmt, Remote | Logic + UI | P1 |
| 12 | claude-code-workflows | 02-Orchestration | Automated code/security/design review, dual-loop architecture, Playwright integration | Workflow Engine | Logic | P1 |
| 13 | claude-code-spec-workflow | 02-Orchestration | Spec-to-code pipeline, steering documents, context optimization (60-80% token reduction) | Workflow Engine | Logic | P1 |
| 14 | claude-code-router | 02-Orchestration | Scenario-based model routing, router scripts, preset sharing | Configuration | Logic | P1 |
| 15 | Claude-Code-Development-Kit | 02-Orchestration | 3-tier docs auto-loading, context injection hooks, security scanner hook | Configuration, Safety | Data + Logic | P2 |
| 16 | hooks-observability | 03-Hooks & Observability | Swim-lane visualization, pulse chart, tool emoji system, hook-HTTP-WebSocket pipeline, dual-color coding, agent team tracking, auto-scroll + manual override | Observability | UI + Logic | P0 |
| 17 | hooks-mastery | 03-Hooks & Observability | All 13 hook event types, UV single-file scripts, TTS integration, team validation, quality validators (Ruff/Ty) | Safety & Governance | Logic + Data | P1 |
| 18 | usage-monitor | 03-Hooks & Observability | P90 usage predictions, multi-plan support, rolling window analytics, cost projection | Observability | Logic | P1 |
| 19 | claude-code-templates | 04-Templates & Skills | 100+ component catalog, npx installer, web catalog, analytics dashboard, health check, multi-source attribution | Skill Marketplace | Data + UI | P1 |
| 20 | claude-code-plugins-plus-skills | 04-Templates & Skills | 1,537 skills, 270 plugins, CCPI package manager, SKILL.md format, auto-activation, 100-point grading, external sync | Skill Marketplace | Data + Logic | P0 |
| 21 | claude-code-skills | 04-Templates & Skills | 38 production skills, skill-creator meta-skill, 69 prompt presets | Skill Marketplace | Data | P1 |
| 22 | everything-claude-code | 04-Templates & Skills | 13 agents, 44 skills, 32 commands, token optimization patterns, cross-platform configs | Configuration, Skill Marketplace | Data | P1 |
| 23 | claude-code-skill-factory | 04-Templates & Skills | Skill factory, agent factory, command factory | Skill Marketplace | Logic | P2 |
| 24 | claude-code-tresor | 04-Templates & Skills | 141 agents, 19 commands, Tresor Workflow Framework, orchestration commands | Agent Mgmt, Workflow Engine | Data + Logic | P1 |
| 25 | my-claude-code-setup | 04-Templates & Skills | Memory bank system, context management, subagent delegation | Configuration | Data | P2 |
| 26 | claude-code-subagents | 05-Subagents | 100+ domain agents, auto-invocation, @mention invocation, 15+ domain categories | Agent Mgmt | Data | P0 |
| 27 | claude-code-sub-agents | 05-Subagents | 33 agents, 4 orchestration patterns (sequential/parallel/validation/iterative), meta-orchestrator, agent communication | Agent Mgmt, Workflow Engine | Data + Logic | P0 |
| 28 | ClaudeCodeAgents | 05-Subagents | Verification agents (Jenny, Karen), over-engineering detection, completion assessment, UI testing agent | Safety & Governance | Data | P1 |
| 29 | claude-code-mcp | 06-MCP & Tooling | Agent-in-agent MCP pattern, permission bypass, tool selection, CLI abstraction | MCP Foundation | Logic | P0 |
| 30 | codemcp | 06-MCP & Tooling | Git-versioned AI edits, auto-accept mode, project config (codemcp.toml) | Git Ops, Safety | Logic | P1 |
| 31 | claude-code-tools | 06-MCP & Tooling | Credential vault, env protection, session search (Rust), session repair | Safety & Governance, Session | Logic | P1 |
| 32 | claude-code-hub | 07-Remote & Infra | Proxy pipeline, fail-open degradation, weight+priority routing, decision chain logging, session caching, auto-generated API docs | Remote Control | Logic | P1 |
| 33 | claude-code-telegram | 07-Remote & Infra | Chat-based agent control, output verbosity levels, multi-project threads, audit logging | Remote Control | Logic | P2 |
| 34 | Claude-Code-Remote | 07-Remote & Infra | Multi-channel notifications (email/Telegram/Discord/LINE), reply-to-send, token-based sessions, PTY injection | Remote Control | Logic | P2 |
| 35 | claude-code-proxy | 07-Remote & Infra | Model mapping, LiteLLM routing, config-driven proxy | Configuration, Remote | Logic | P2 |
| 36 | claude-code-action | 08-Automation | Structured JSON output, visual progress tracking, multi-provider auth, @mention triggering | Workflow Engine | Logic | P1 |
| 37 | claude-code-config | 09-Config | Security-first defaults, production patterns from Trail of Bits | Configuration, Safety | Data | P1 |
| 38 | claude-code-config2 | 09-Config | Multi-project config, hook chaining | Configuration | Data | P2 |
| 39 | claude-code-settings | 09-Config | Full settings catalog (37 settings, 84 env vars), hierarchical config | Configuration | Data | P0 |
| 40 | claude-code-showcase | 09-Config | Complete example project, GitHub Actions integration | Configuration | Data | P2 |
| 41 | awesome-claude-code | 10-Guides | Ecosystem index | Documentation | Data | P3 |
| 42 | awesome-claude-code-subagents | 10-Guides | 141+ agent marketplace, tech stack auto-selection, multi-phase orchestration | Agent Mgmt | Data | P1 |
| 43 | claude-code-guide | 10-Guides | Multi-platform install guide, progressive learning | Documentation | Data | P3 |
| 44 | claude-code-best-practice | 10-Guides | RPI workflow, multi-agent patterns, context engineering, CLAUDE.md templates | Documentation, Configuration | Data | P2 |
| 45 | claude-code-tips | 10-Guides | System prompt slimming, worktree patterns, container patterns | Documentation | Data | P3 |
| 46 | claude-code-cheat-sheet | 10-Guides | Progressive disclosure command reference, level-based learning | Documentation | Data | P3 |
| 47 | claude-code-mastering | 10-Guides | Complete 13-chapter curriculum, team patterns | Documentation | Data | P3 |
| 48 | claude-code-docs | 11-Docs & Internals | Auto-sync docs, /docs integration | Documentation | Data + Logic | P2 |
| 49 | claude-code-system-prompts | 11-Docs & Internals | 110+ system prompts, token counts, version tracking | Documentation, Safety | Data | P2 |
| 50 | claude-code-prompt-improver | 12-Prompts & Learning | Progressive prompt enhancement, bypass prefixes, clarifying questions, 31% token reduction | Agent Mgmt | Logic | P1 |
| 51 | claude-code-pm-course | 12-Prompts & Learning | Config-driven curriculum, interactive teaching | Documentation | Data | P3 |
| 52 | claude-code-requirements-builder | 12-Prompts & Learning | Codebase-aware discovery, two-phase questioning, smart defaults, MUST/SHOULD/MAY framework | Workflow Engine | Logic | P1 |
| 53 | claude-code-transcripts | 13-Misc | Session-to-HTML export, Gist publishing, batch conversion | Session Lifecycle | Logic | P1 |
| 54 | claude-code-security-review | 13-Misc | Diff-aware security scanning, semantic analysis, false positive filtering, slash command + CI | Safety & Governance | Logic | P0 |
| 55 | claude-code-my-workflow | 13-Misc | Quality gates (80/90/95 thresholds), adversarial QA, contractor mode, context survival | Workflow Engine, Safety | Logic | P1 |
| 56 | claude-code-infrastructure-showcase | 13-Misc | Auto-activating skills, 500-line rule, progressive disclosure, skill-rules.json | Skill Marketplace | Logic + Data | P1 |
| 57 | claude-code-reverse | 13-Misc | API interception, conversation flow visualization, context compaction analysis | Observability | Logic | P2 |
| 58 | Claude-Code-Communication | 13-Misc | Hierarchical delegation, inter-agent messaging, role-based instructions | Agent Mgmt, Workflow Engine | Logic | P1 |
| 59 | edmunds-claude-code | 13-Misc | Stack-specific commands, architect agents, type-safe enforcement | Agent Mgmt, Skill Marketplace | Data | P2 |
| 60 | claude-coder | 13-Misc | Mockup-to-code, all-level UX | UI Framework | UI | P3 |
| 61 | -- | -- | -- | -- | -- | -- |

Note: Row 61 is a placeholder. The 62nd repo count includes the category README files and overlapping entries. All unique repos are listed above (60 individual repositories).

---

## Summary Statistics

### Features Per Bounded Context

| Bounded Context | Contributing Repos | Key Feature Count |
|----------------|-------------------|-------------------|
| Agent Management | 15 | 25+ (presets, lifecycle, circuit breaker, coordination) |
| Session Lifecycle | 8 | 15+ (search, resume, export, todos, transcript) |
| Workflow Engine | 10 | 20+ (DAG, templates, review loops, spec-to-code) |
| Observability | 7 | 15+ (swim-lane, pulse, cost, usage, metrics) |
| Skill Marketplace | 9 | 20+ (catalog, install, auto-activate, factory, grading) |
| Safety & Governance | 8 | 15+ (circuit breaker, rate limit, security scan, permissions) |
| Git Operations | 4 | 10+ (worktrees, diff, staging, branches, versioned edits) |
| Configuration | 9 | 15+ (settings, env vars, hierarchical config, routing) |
| Remote Control | 5 | 10+ (proxy, notifications, chat control, auth) |
| MCP Foundation | 4 | 8+ (tool exposure, protocol, routing, IDE integration) |
| Documentation | 8 | Data resources (guides, tips, references) |

### Repos Per Type

| Type | Count | Description |
|------|-------|-------------|
| **Data repos** | 22 | Contribute presets, skills, configs, templates -- no runtime logic |
| **Logic repos** | 20 | Contribute algorithms, state machines, pipelines -- core behavior |
| **UI repos** | 8 | Contribute component designs, layouts, visualizations |
| **Mixed (Data + Logic)** | 7 | Contribute both content and behavioral patterns |
| **Mixed (Logic + UI)** | 3 | Contribute both algorithms and visual components |

### Priority Distribution

| Priority | Count | Description |
|----------|-------|-------------|
| **P0: Absorb Now** | 10 | Core platform features needed for MVP |
| **P1: Absorb Soon** | 22 | High-value features for 3-month horizon |
| **P2: Absorb Later** | 16 | Important but not urgent, or dependency-blocked |
| **P3: Consider for Plugin** | 12 | Niche or documentation-only, low absorption effort |

---

## Data Repos (Contribute Presets, Skills, Configs)

These repos contribute content that can be imported directly into Forge's database without implementing new logic.

| Repo | Content Type | Item Count | Import Effort |
|------|-------------|------------|---------------|
| claude-code-subagents | Agent presets | 100+ agents | S (JSON import) |
| claude-code-sub-agents | Agent presets + orchestration patterns | 33 agents | S (JSON import) |
| ClaudeCodeAgents | QA agent presets | 6 agents | S (JSON import) |
| claude-code-plugins-plus-skills | Skill definitions | 1,537 skills | M (SKILL.md parser) |
| claude-code-skills | Skill definitions + prompt presets | 38 skills, 69 prompts | S (import) |
| everything-claude-code | Agents, skills, commands | 13 agents, 44 skills, 32 commands | S (import) |
| claude-code-tresor | Agents, commands | 141 agents, 19 commands | S (import) |
| claude-code-templates | Component catalog | 100+ components | S (import) |
| awesome-claude-code-subagents | Agent marketplace | 141+ agents | S (import) |
| edmunds-claude-code | Commands, agents | 14 commands, 11 agents | S (import) |
| claude-code-settings | Settings reference | 37 settings, 84 env vars | S (reference data) |
| claude-code-config | Config templates | Security defaults | S (import) |
| claude-code-config2 | Config templates | Multi-project patterns | S (import) |
| claude-code-showcase | Example configs | Working examples | S (import) |
| my-claude-code-setup | Memory bank templates | Context patterns | S (import) |

**Total importable content**: 500+ agent presets, 1,600+ skills, 100+ commands, 100+ config templates

---

## Pattern Repos (Contribute Algorithms, Architectures)

These repos contribute behavioral patterns that must be re-implemented in Rust.

| Repo | Pattern | Complexity | Rust Implementation Notes |
|------|---------|------------|---------------------------|
| ralph-claude-code | Circuit breaker FSM | M | `DashMap<String, CircuitState>`, tokio timers |
| ralph-claude-code | Rate limiter | M | Token bucket with `Instant` tracking |
| ralph-claude-code | Dual exit gate | M | State machine with two condition flags |
| ralph-claude-code | Response analyzer | L | Pattern matching on agent output stream |
| Claude-Code-Workflow | DAG executor | XL | Topological sort + tokio::JoinSet for parallelism |
| Claude-Code-Workflow | 4-level workflow | L | Enum-based workflow complexity selection |
| hooks-observability | Hook-HTTP-WebSocket pipeline | M | Axum + broadcast channels (already partially implemented) |
| hooks-observability | Swim-lane renderer | L | Svelte component with time-aligned columns |
| usage-monitor | P90 prediction | M | Percentile calculation on rolling window |
| claude-code-hub | Proxy pipeline | XL | Axum middleware chain |
| claude-code-hub | Fail-open degradation | M | Fallback logic when optional services unavailable |
| claude-code-viewer | FTS5 session search | M | SQLite FTS5 virtual table + rusqlite |
| claude-code-viewer | Cron scheduler | M | Cron expression parser + tokio scheduler |
| claude-code-mcp | Agent-in-agent MCP | L | MCP server exposing Forge as tool |
| claude-code-security-review | Diff-aware scanning | L | Parse diffs, send to LLM for analysis |
| claude-code-prompt-improver | Progressive enhancement | M | Hook that conditionally enriches prompts |
| claude-code-infrastructure-showcase | Auto-activating skills | M | Rule engine matching context to skills |
| Claude-Code-Communication | Hierarchical delegation | L | Tree-structured agent coordination |

---

## UI Repos (Contribute Component Designs)

These repos contribute visual patterns to be re-implemented in Svelte.

| Repo | Component | Source Framework | Svelte Effort |
|------|-----------|-----------------|---------------|
| 1code | Kanban board | React 19 | M (Svelte drag-and-drop) |
| 1code | Git client UI | React 19 | L (staging, diff, PR) |
| 1code | Worktree manager | React 19 | M |
| claude-code-viewer | Session search UI | React + TanStack | M (FTS5 integration) |
| claude-code-viewer | Todo viewer | React | S |
| claude-code-viewer | Terminal panel | React + WebSocket | M (xterm.js integration) |
| claude-code-viewer | Branch switcher | React | S |
| hooks-observability | Swim-lane view | Vue 3 | L (real-time layout) |
| hooks-observability | Pulse chart | Vue 3 | M (chart component) |
| hooks-observability | Tool emoji system | Vue 3 | S (icon mapping) |
| claude-code-webui | Mobile-responsive chat | React | M |
| claude-code-chat | Checkpoint UI | TypeScript WebView | S |
| claude-code-templates | Web catalog | Astro 5 / React | M (browse/filter/install) |
| claude-coder | All-level UX | VS Code WebView | S (design reference only) |
