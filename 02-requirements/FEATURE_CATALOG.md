# Claude Forge -- Feature Catalog

**Version**: 1.0
**Date**: 2026-02-25
**Total Features**: 213

---

## Table of Contents

1. [Agent Management](#1-agent-management)
2. [Safety and Reliability](#2-safety-and-reliability)
3. [Workflows](#3-workflows)
4. [Session Management](#4-session-management)
5. [Skills](#5-skills)
6. [Plugins](#6-plugins)
7. [Git Integration](#7-git-integration)
8. [Observability](#8-observability)
9. [Notifications](#9-notifications)
10. [Security](#10-security)
11. [Configuration](#11-configuration)
12. [MCP Server](#12-mcp-server)
13. [Summary Statistics](#13-summary-statistics)
14. [Absorption Matrix](#14-absorption-matrix)

---

## Legend

**Priority**:
- **P0**: Must have for initial release. Blocking.
- **P1**: Should have. Important for adoption.
- **P2**: Nice to have. Differentiating.
- **P3**: Future. Tracked for later.

**Effort**:
- **S**: Small (< 1 day, < 200 LOC)
- **M**: Medium (1-3 days, 200-800 LOC)
- **L**: Large (3-7 days, 800-2000 LOC)
- **XL**: Extra Large (1-3 weeks, 2000+ LOC)

**Status**:
- DONE: Implemented in current codebase
- WIP: Partially implemented
- TODO: Not yet started

---

## 1. Agent Management

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| AM-001 | Agent CRUD | Create, read, update, delete agent definitions | 1code, claude-code-subagents | P0 | M | SQLite | DONE |
| AM-002 | Agent presets (9) | Ship with 9 pre-configured agent presets | claude-code-subagents, claude-code-agents | P0 | S | AM-001 | DONE |
| AM-003 | Agent presets (100+) | Expand to 100+ presets across 15 categories | claude-code-subagents, claude-code-sub-agents, claude-code-agents | P1 | L | AM-001 | TODO |
| AM-004 | Agent form UI | Graphical form for agent creation and editing | 1code | P0 | M | AM-001 | DONE |
| AM-005 | Directory picker | Browse and select working directory | 1code | P0 | S | AM-004 | DONE |
| AM-006 | Process spawning | Spawn agent as child process with stream-json output | 1code, ralph-claude-code | P0 | L | AM-001 | DONE |
| AM-007 | Session resume | Resume previous session with --resume flag | ralph-claude-code, claude-code-viewer | P0 | M | AM-006 | DONE |
| AM-008 | Multi-CLI: Claude | Spawn Claude Code CLI processes | 1code | P0 | S | AM-006 | DONE |
| AM-009 | Multi-CLI: Codex | Spawn Codex CLI processes | Claude-Code-Workflow, claude_code_bridge | P1 | M | AM-006 | TODO |
| AM-010 | Multi-CLI: Gemini | Spawn Gemini CLI processes | Claude-Code-Workflow, claude_code_bridge | P1 | M | AM-006 | TODO |
| AM-011 | Multi-CLI: Qwen | Spawn Qwen CLI processes | Claude-Code-Workflow | P2 | M | AM-006 | TODO |
| AM-012 | Multi-CLI: Custom binary | Configure arbitrary CLI binary path | claude-code-mcp, claude-code-router | P2 | S | AM-006 | TODO |
| AM-013 | Agent teams | Group agents into teams with roles | hooks-observability | P1 | L | AM-001 | TODO |
| AM-014 | Team patterns | Builder+Validator, Lead+Workers, Pipeline patterns | hooks-observability | P1 | M | AM-013 | TODO |
| AM-015 | Agent handoff | Pass structured context between agents | hooks-observability | P1 | L | AM-001 | TODO |
| AM-016 | Automatic invocation | Auto-invoke agents based on trigger phrases | claude-code-subagents | P2 | M | AM-001 | TODO |
| AM-017 | @mention invocation | Invoke agents with @agent-name syntax | claude-code-subagents | P2 | S | AM-016 | TODO |
| AM-018 | Process lifecycle | Pause (SIGSTOP), resume (SIGCONT), kill | ralph-claude-code | P0 | M | AM-006 | WIP |
| AM-019 | Environment isolation | env_remove for nested-session and API key guards | -- | P0 | S | AM-006 | DONE |
| AM-020 | Model routing | Scenario-based automatic model selection | claude-code-router | P2 | L | AM-006 | TODO |
| AM-021 | Router scripts | Custom JavaScript router logic | claude-code-router | P3 | M | AM-020 | TODO |
| AM-022 | Provider support: Bedrock | AWS Bedrock API integration | claude-code-action, claude-code-hub | P2 | M | AM-006 | TODO |
| AM-023 | Provider support: Vertex | Google Vertex AI integration | claude-code-action | P2 | M | AM-006 | TODO |
| AM-024 | Provider support: OpenRouter | OpenRouter multi-model integration | claude-code-router | P3 | M | AM-006 | TODO |
| AM-025 | Agent export/import | Export agent as JSON, import from JSON | 1code | P1 | S | AM-001 | DONE |
| AM-026 | Split-pane multi-agent | Simultaneous agent output in split panes | claude_code_bridge, 1code | P0 | L | AM-006 | DONE |
| AM-027 | Auto-spawning daemons | Start agents on demand, idle timeout | claude_code_bridge | P3 | M | AM-006 | TODO |
| AM-028 | Plan mode | Structured planning before execution | 1code | P2 | M | AM-006 | TODO |
| AM-029 | Extended thinking support | Enable extended thinking mode per agent | 1code | P2 | S | AM-006 | TODO |
| AM-030 | Voice input | Voice-to-text for agent prompts | 1code | P3 | L | AM-004 | TODO |

**Subtotal**: 30 features

---

## 2. Safety and Reliability

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| SR-001 | Circuit breaker (3-state) | CLOSED/OPEN/HALF_OPEN FSM per agent | ralph-claude-code | P0 | L | AM-006 | TODO |
| SR-002 | Rate limiter (calls/hour) | Configurable rate limit per agent | ralph-claude-code | P0 | M | AM-006 | TODO |
| SR-003 | Rate limiter (RPM) | Requests per minute limit | claude-code-hub | P1 | S | SR-002 | TODO |
| SR-004 | Rate limiter (budget window) | 5-hour rolling budget window | claude-code-hub | P1 | M | SR-002 | TODO |
| SR-005 | Rate limiter (concurrent) | Max concurrent sessions per user | claude-code-hub | P1 | S | SR-002 | TODO |
| SR-006 | Dual exit gate | Completion indicators + EXIT_SIGNAL required | ralph-claude-code | P0 | L | AM-006 | TODO |
| SR-007 | Exit gate modes | Strict, normal, relaxed per agent | ralph-claude-code | P1 | S | SR-006 | TODO |
| SR-008 | Response analyzer: loops | Detect repeated output patterns | ralph-claude-code | P0 | M | AM-006 | TODO |
| SR-009 | Response analyzer: errors | Detect stack traces and error codes | ralph-claude-code | P1 | M | SR-008 | TODO |
| SR-010 | Response analyzer: stuck | Detect no-progress states | ralph-claude-code | P1 | M | SR-008 | TODO |
| SR-011 | Response analyzer: hallucination | Detect references to nonexistent files | -- | P2 | L | SR-008 | TODO |
| SR-012 | File protection (glob rules) | Block agent writes to protected paths | ralph-claude-code | P0 | M | AM-006 | TODO |
| SR-013 | File protection defaults | Default rules for .env, credentials, .git/config | ralph-claude-code | P0 | S | SR-012 | TODO |
| SR-014 | Pre-run validation | Check environment before agent start | ralph-claude-code | P1 | M | AM-006 | TODO |
| SR-015 | Budget: soft limit | Warning notification at threshold | claude-code-hub | P0 | M | OB-007 | TODO |
| SR-016 | Budget: hard limit | Pause agent at 100% | claude-code-hub | P0 | M | SR-015 | TODO |
| SR-017 | Budget: per-agent | Track and enforce per-agent | claude-code-hub | P0 | S | SR-016 | TODO |
| SR-018 | Budget: per-project | Track and enforce per-project | claude-code-hub | P1 | S | SR-016 | TODO |
| SR-019 | Budget: global | Track and enforce globally | claude-code-hub | P1 | S | SR-016 | TODO |
| SR-020 | Fail-open degradation | Continue when optional subsystems fail | claude-code-hub | P1 | M | -- | TODO |
| SR-021 | Session timeout | Auto-expire sessions after configurable period | ralph-claude-code | P1 | S | AM-007 | TODO |
| SR-022 | Auto-reset triggers | Reset circuit breaker on configurable conditions | ralph-claude-code | P2 | S | SR-001 | TODO |
| SR-023 | Countdown timers | UI timers for rate limits and circuit breaker cooldown | ralph-claude-code | P1 | S | SR-001, SR-002 | TODO |

**Subtotal**: 23 features

---

## 3. Workflows

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| WF-001 | L1: lite-lite-lite | Quick single-step task execution | Claude-Code-Workflow | P0 | S | AM-006 | DONE |
| WF-002 | L2: lite-lite | Simple multi-step tasks | Claude-Code-Workflow | P1 | M | WF-001 | TODO |
| WF-003 | L3: lite | Standard development workflows | Claude-Code-Workflow | P1 | L | WF-002 | TODO |
| WF-004 | L4: brainstorm | Deep exploration with multiple agents | Claude-Code-Workflow | P2 | XL | WF-003, AM-013 | TODO |
| WF-005 | DAG task graph | Directed acyclic graph for step dependencies | Claude-Code-Workflow | P1 | XL | WF-002 | TODO |
| WF-006 | Parallel execution | Run independent steps concurrently | Claude-Code-Workflow | P1 | L | WF-005 | TODO |
| WF-007 | Workflow state persistence | Save and restore workflow state | Claude-Code-Workflow | P1 | M | WF-005 | TODO |
| WF-008 | Crash recovery | Resume incomplete workflows after restart | Claude-Code-Workflow | P1 | M | WF-007 | TODO |
| WF-009 | Workflow templates (built-in) | 8 templates: feature, bugfix, refactor, review, test, docs, release, audit | claude-code-spec-workflow, claude-code-development-kit | P1 | L | WF-005 | TODO |
| WF-010 | Custom workflow templates | Create templates from completed workflows | Claude-Code-Workflow | P2 | M | WF-009 | TODO |
| WF-011 | Template parameters | Accept variables in templates | Claude-Code-Workflow | P1 | M | WF-009 | TODO |
| WF-012 | Semantic CLI selection | Auto-select best CLI per workflow step | Claude-Code-Workflow | P2 | L | AM-009, AM-010 | TODO |
| WF-013 | DAG visualization | Interactive workflow graph in UI | Claude-Code-Workflow | P1 | L | WF-005 | TODO |
| WF-014 | Node status colors | Pending/running/succeeded/failed/skipped colors | Claude-Code-Workflow | P1 | S | WF-013 | TODO |
| WF-015 | Workflow level auto-select | Suggest level based on prompt analysis | Claude-Code-Workflow | P2 | M | WF-001 | TODO |
| WF-016 | Issue workflow | Parse GitHub issues into workflows | claude-code-action | P2 | L | WF-005 | TODO |
| WF-017 | Issue result posting | Post workflow results to issues | claude-code-action | P2 | M | WF-016 | TODO |
| WF-018 | Retry policy per step | Configurable retries with backoff | Claude-Code-Workflow | P1 | M | WF-005 | TODO |
| WF-019 | Step timeout | Per-step timeout with kill on exceed | Claude-Code-Workflow | P1 | S | WF-005 | TODO |
| WF-020 | Workflow abort | Cancel running workflow, cleanup active steps | Claude-Code-Workflow | P0 | M | WF-005 | TODO |

**Subtotal**: 20 features

---

## 4. Session Management

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| SM-001 | Session scanning | Scan ~/.claude/projects/ for sessions | claude-code-viewer | P0 | M | -- | DONE |
| SM-002 | Session list API | GET /api/sessions endpoint | claude-code-viewer | P0 | M | SM-001 | DONE |
| SM-003 | Projects list API | GET /api/sessions/projects endpoint | claude-code-viewer | P0 | S | SM-001 | DONE |
| SM-004 | Session browser UI | Browsable list with sort and filter | claude-code-viewer | P0 | L | SM-002 | WIP |
| SM-005 | Session detail view | Full session display with messages | claude-code-viewer | P0 | L | SM-004 | TODO |
| SM-006 | Full-text search (FTS) | FTS5 index across session content | claude-code-viewer | P0 | L | SM-001 | TODO |
| SM-007 | Fuzzy search (Cmd+K) | Quick fuzzy search with keyboard shortcut | claude-code-viewer | P0 | M | SM-006 | TODO |
| SM-008 | Session resume from browser | Resume session directly from browser | claude-code-viewer | P0 | M | SM-004, AM-007 | TODO |
| SM-009 | Export: JSON | Full-fidelity session export | claude-code-viewer, 1code | P1 | M | SM-005 | DONE |
| SM-010 | Export: Markdown | Human-readable Markdown export | claude-code-viewer | P1 | M | SM-005 | DONE |
| SM-011 | Export: HTML | Styled HTML export | -- | P2 | M | SM-005 | TODO |
| SM-012 | Bulk export | Export multiple sessions with filters | claude-code-viewer | P2 | M | SM-009 | TODO |
| SM-013 | Todo extraction | Parse TodoWrite events into todo list | claude-code-viewer | P1 | M | SM-005 | TODO |
| SM-014 | Todo panel | Display todos with status tracking | claude-code-viewer | P1 | M | SM-013 | TODO |
| SM-015 | Todo aggregate view | Cross-session todo view | claude-code-viewer | P2 | M | SM-014 | TODO |
| SM-016 | Kanban view | Sessions as cards on status columns | 1code | P1 | L | SM-004 | TODO |
| SM-017 | Kanban drag-and-drop | Reorder queue priority | 1code | P2 | M | SM-016 | TODO |
| SM-018 | Cost per session | Calculate and display session cost | Claude-Code-Usage-Monitor | P0 | M | OB-007 | TODO |
| SM-019 | Cost timeline chart | Cost accumulation chart per session | Claude-Code-Usage-Monitor | P1 | M | SM-018 | TODO |
| SM-020 | Session deletion | Delete individual and bulk sessions | -- | P2 | S | SM-004 | TODO |
| SM-021 | Session filter: project | Filter by project | claude-code-viewer | P0 | S | SM-004 | TODO |
| SM-022 | Session filter: date range | Filter by date range | claude-code-viewer | P1 | S | SM-004 | TODO |
| SM-023 | Session filter: status | Filter by active/completed/failed | claude-code-viewer | P1 | S | SM-004 | TODO |
| SM-024 | Session sort | Sort by date, duration, cost, messages | claude-code-viewer | P0 | S | SM-004 | TODO |
| SM-025 | Session pagination | Paginate large session lists | claude-code-viewer | P1 | S | SM-004 | TODO |

**Subtotal**: 25 features

---

## 5. Skills

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| SK-001 | Skill catalog (data model) | SQLite schema for skills | claude-code-plugins-plus-skills | P1 | M | -- | TODO |
| SK-002 | Skill import (ecosystem) | Import 500+ skills from ecosystem repos | claude-code-plugins-plus-skills, claude-code-skills, claude-code-templates | P1 | L | SK-001 | TODO |
| SK-003 | Skill catalog UI | Browse skills by category with cards | claude-code-plugins-plus-skills | P1 | L | SK-001 | TODO |
| SK-004 | Skill search (text) | Search by name, description, tags | claude-code-plugins-plus-skills | P1 | M | SK-001 | TODO |
| SK-005 | Skill category filter | Filter by 15+ categories | claude-code-plugins-plus-skills | P1 | S | SK-003 | TODO |
| SK-006 | Skill detail page | Full skill info with content preview | claude-code-plugins-plus-skills | P1 | M | SK-003 | TODO |
| SK-007 | Skill install (from catalog) | One-click install to local filesystem | claude-code-plugins-plus-skills | P1 | M | SK-001 | TODO |
| SK-008 | Skill install (from URL) | Install from GitHub URL or local path | claude-code-plugins-plus-skills | P1 | M | SK-007 | TODO |
| SK-009 | Skill uninstall | Remove installed skill | -- | P1 | S | SK-007 | TODO |
| SK-010 | Skill update | Update to latest version | claude-code-plugins-plus-skills | P2 | M | SK-007 | TODO |
| SK-011 | Auto-activation: triggers | Skills declare trigger phrases | claude-code-plugins-plus-skills | P1 | M | SK-001 | TODO |
| SK-012 | Auto-activation: file patterns | Skills trigger on file type | claude-code-plugins-plus-skills | P1 | S | SK-011 | TODO |
| SK-013 | Auto-activation: project type | Skills trigger on project tech stack | claude-code-plugins-plus-skills | P2 | M | SK-011 | TODO |
| SK-014 | Auto-activation: max limit | Configurable max concurrent skills | -- | P1 | S | SK-011 | TODO |
| SK-015 | Quality grading (100-point) | Grade skills on metadata, content, tests, community, freshness | claude-code-plugins-plus-skills | P1 | L | SK-001 | TODO |
| SK-016 | SKILL.md parser | Parse SKILL.md frontmatter format | claude-code-plugins-plus-skills, claude-code-skills | P1 | M | -- | TODO |
| SK-017 | SKILL.md validator | Validate required fields and format | claude-code-plugins-plus-skills | P1 | M | SK-016 | TODO |
| SK-018 | Skill builder wizard | Interactive skill creation UI | claude-code-skill-factory | P1 | L | SK-016 | TODO |
| SK-019 | Skill live preview | Preview skill in catalog before publishing | claude-code-skill-factory | P2 | M | SK-018 | TODO |
| SK-020 | Skill publish (local) | Register skill in local catalog | -- | P1 | S | SK-001 | TODO |
| SK-021 | Skill publish (remote) | Push skill to remote registry | claude-code-plugins-plus-skills | P2 | L | SK-020 | TODO |
| SK-022 | External sync | Sync skills from configured GitHub repos | claude-code-plugins-plus-skills | P2 | L | SK-001 | TODO |
| SK-023 | Skill ratings | Community ratings and reviews | claude-code-plugins-plus-skills | P3 | M | SK-001 | TODO |
| SK-024 | Prompt presets | 69+ reusable prompt templates | claude-code-skills | P2 | M | -- | TODO |
| SK-025 | Skill-creator meta-skill | A skill that generates other skills | claude-code-skills | P2 | M | SK-016 | TODO |

**Subtotal**: 25 features

---

## 6. Plugins

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| PL-001 | WASM runtime | wasmtime-based sandboxed execution | -- | P1 | XL | -- | TODO |
| PL-002 | WASM plugin API | Defined API surface for plugins | -- | P1 | XL | PL-001 | TODO |
| PL-003 | WASM memory limits | Configurable memory per plugin | -- | P1 | S | PL-001 | TODO |
| PL-004 | WASM CPU limits | Configurable CPU time per plugin | -- | P2 | M | PL-001 | TODO |
| PL-005 | Plugin hot-reload | Reload plugins without restart | -- | P2 | L | PL-001 | TODO |
| PL-006 | MCP client (stdio) | Connect to MCP servers over stdio | claude-code-mcp | P0 | L | -- | DONE |
| PL-007 | MCP client (SSE) | Connect to MCP servers over SSE | claude-code-mcp | P1 | M | PL-006 | TODO |
| PL-008 | MCP client (WebSocket) | Connect to MCP servers over WebSocket | claude-code-mcp | P2 | M | PL-006 | TODO |
| PL-009 | MCP tool discovery | Discover tools from connected servers | claude-code-mcp | P0 | M | PL-006 | DONE |
| PL-010 | MCP resource access | Access resources from connected servers | claude-code-mcp | P1 | M | PL-006 | TODO |
| PL-011 | MCP server editor UI | Configure MCP servers in graphical editor | claude-code-viewer, 1code | P0 | M | PL-006 | DONE |
| PL-012 | Native plugin trait | Rust trait for native plugins | -- | P3 | L | -- | TODO |
| PL-013 | Native plugin loading | Load .so/.dylib at startup | -- | P3 | L | PL-012 | TODO |
| PL-014 | Plugin marketplace UI | Browse and install plugins | claude-code-templates | P2 | L | PL-001 | TODO |
| PL-015 | Plugin validation | Schema and security validation | claude-code-plugins-plus-skills | P1 | M | PL-001 | TODO |
| PL-016 | Plugin quality grading | 100-point quality scoring | claude-code-plugins-plus-skills | P2 | M | PL-015 | TODO |
| PL-017 | Plugin per-agent config | Enable/disable plugins per agent | -- | P1 | S | PL-006 | TODO |

**Subtotal**: 17 features

---

## 7. Git Integration

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| GI-001 | Git status display | Show staged, modified, untracked, conflicted files | 1code, claude-code-viewer | P1 | M | -- | TODO |
| GI-002 | Real-time status updates | File watcher triggers status refresh | 1code | P1 | M | GI-001 | TODO |
| GI-003 | Diff viewer (unified) | Unified diff with syntax highlighting | claude-code-viewer | P1 | L | GI-001 | TODO |
| GI-004 | Diff viewer (side-by-side) | Side-by-side diff view | 1code | P2 | L | GI-003 | TODO |
| GI-005 | Diff: working vs HEAD | Compare working directory to HEAD | claude-code-viewer | P1 | S | GI-003 | TODO |
| GI-006 | Diff: staged vs HEAD | Compare staged files to HEAD | claude-code-viewer | P1 | S | GI-003 | TODO |
| GI-007 | Diff: arbitrary commits | Compare between any two commits | 1code | P2 | M | GI-003 | TODO |
| GI-008 | Stage/unstage files | Checkbox-based staging | claude-code-viewer | P1 | M | GI-001 | TODO |
| GI-009 | Commit creation | Create commits with message editor | claude-code-viewer, codemcp | P1 | M | GI-008 | TODO |
| GI-010 | AI commit message | Generate commit message from diff | codemcp | P2 | M | GI-009 | TODO |
| GI-011 | Conventional commit | Type selector (feat, fix, docs, etc.) | -- | P2 | S | GI-009 | TODO |
| GI-012 | Branch list | List branches with search | claude-code-viewer | P1 | M | -- | TODO |
| GI-013 | Branch create | Create branches with naming patterns | 1code | P1 | S | GI-012 | TODO |
| GI-014 | Branch switch | Switch branches with change handling | claude-code-viewer | P1 | M | GI-012 | TODO |
| GI-015 | Branch delete | Delete branches with confirmation | 1code | P2 | S | GI-012 | TODO |
| GI-016 | Worktree create | Create git worktree for agent | 1code | P1 | L | -- | TODO |
| GI-017 | Worktree auto-create | Auto-create worktree on agent start | 1code | P1 | M | GI-016, AM-006 | TODO |
| GI-018 | Worktree merge | Merge worktree changes back | 1code | P1 | M | GI-016 | TODO |
| GI-019 | Worktree cleanup | Auto-cleanup on session end | 1code | P1 | S | GI-016 | TODO |
| GI-020 | Worktree list | Panel showing all active worktrees | 1code | P1 | M | GI-016 | TODO |
| GI-021 | PR creation (GitHub) | Create pull request via gh CLI | 1code, claude-code-action | P2 | M | GI-009 | TODO |
| GI-022 | PR creation (GitLab) | Create merge request via glab CLI | -- | P3 | M | GI-009 | TODO |
| GI-023 | PR description generation | AI-generated PR description | claude-code-action | P2 | M | GI-021 | TODO |
| GI-024 | Git-versioned edits | Per-edit commits from agents | codemcp | P2 | L | GI-009, AM-006 | TODO |
| GI-025 | Squash agent commits | Squash all agent commits on session end | codemcp | P2 | M | GI-024 | TODO |

**Subtotal**: 25 features

---

## 8. Observability

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| OB-001 | Event capture (12 types) | Capture all hook lifecycle events | hooks-observability | P0 | L | -- | DONE |
| OB-002 | Event persistence (SQLite) | Batch write events to SQLite | hooks-observability | P0 | M | OB-001 | DONE |
| OB-003 | WebSocket broadcast | Stream events to UI clients | hooks-observability | P0 | M | OB-001 | DONE |
| OB-004 | Event stream UI | Real-time event list in UI panel | hooks-observability | P0 | M | OB-003 | DONE |
| OB-005 | Event filtering | Filter by type, agent, severity | hooks-observability | P0 | S | OB-004 | WIP |
| OB-006 | Swim-lane visualization | Per-agent timeline lanes | hooks-observability | P0 | XL | OB-003 | TODO |
| OB-007 | Tool emoji system | Visual tool identification (Bash: >_, Read: eye, etc.) | hooks-observability | P1 | S | OB-006 | TODO |
| OB-008 | Dual-color coding | App color + session color | hooks-observability | P1 | S | OB-006 | TODO |
| OB-009 | Zoom and pan | Timeline zoom and pan controls | hooks-observability | P1 | M | OB-006 | TODO |
| OB-010 | Pulse chart | Real-time activity frequency bars | hooks-observability | P1 | L | OB-003 | TODO |
| OB-011 | Pulse: session colors | Color bars by session identity | hooks-observability | P1 | S | OB-010 | TODO |
| OB-012 | Pulse: configurable window | 1s, 5s, 15s, 60s time windows | hooks-observability | P2 | S | OB-010 | TODO |
| OB-013 | Token counter (per-agent) | Real-time input/output token counts | Claude-Code-Usage-Monitor | P0 | M | OB-001 | TODO |
| OB-014 | Cost calculator (real-time) | Model-specific pricing per session | Claude-Code-Usage-Monitor | P0 | M | OB-013 | TODO |
| OB-015 | Pricing tables | Configurable model pricing | Claude-Code-Usage-Monitor | P0 | S | OB-014 | TODO |
| OB-016 | Cost views: daily | Daily cost summary | Claude-Code-Usage-Monitor | P1 | M | OB-014 | TODO |
| OB-017 | Cost views: monthly | Monthly cost summary with chart | Claude-Code-Usage-Monitor | P1 | M | OB-016 | TODO |
| OB-018 | Cost views: per-project | Project-level cost rollup | Claude-Code-Usage-Monitor | P1 | M | OB-014 | TODO |
| OB-019 | P90 usage prediction | Percentile-based limit projection | Claude-Code-Usage-Monitor | P1 | L | OB-014 | TODO |
| OB-020 | Plan support | Pro, Max5, Max20, Custom plans | Claude-Code-Usage-Monitor | P1 | M | OB-019 | TODO |
| OB-021 | Auto plan switch recommendation | Suggest plan based on usage | Claude-Code-Usage-Monitor | P2 | M | OB-020 | TODO |
| OB-022 | Chat transcript viewer | Render chat with syntax highlighting | hooks-observability, claude-code-viewer | P0 | L | OB-003 | DONE |
| OB-023 | Tool call expansion | Expand tool calls to show input/output | hooks-observability | P0 | M | OB-022 | WIP |
| OB-024 | Transcript search | Search within transcript | claude-code-viewer | P1 | M | OB-022 | TODO |
| OB-025 | Auto-scroll with override | Auto-scroll that pauses on manual scroll | hooks-observability | P0 | S | OB-022 | DONE |
| OB-026 | Virtualized event list | Handle 50K+ events without lag | -- | P1 | L | OB-004 | TODO |
| OB-027 | Decision chain logging | Log model/tool/agent routing decisions | claude-code-hub | P1 | L | OB-001 | TODO |
| OB-028 | Decision chain UI | View decision chain per session | claude-code-hub | P2 | M | OB-027 | TODO |
| OB-029 | Subagent tracking | Track subagent lifecycle events | hooks-observability | P1 | M | OB-001 | TODO |
| OB-030 | Agent team view | Builder/Validator coordination display | hooks-observability | P2 | M | OB-006, AM-013 | TODO |

**Subtotal**: 30 features

---

## 9. Notifications

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| NO-001 | Webhook notifications | Send events to configurable webhook URLs | Claude-Code-Remote, claude-code-hub | P1 | M | OB-001 | TODO |
| NO-002 | Webhook signature (HMAC) | HMAC-SHA256 signature verification | -- | P1 | S | NO-001 | TODO |
| NO-003 | Webhook retry | 3 attempts with exponential backoff | -- | P1 | S | NO-001 | TODO |
| NO-004 | Telegram bot setup | BotFather integration wizard | claude-code-telegram, Claude-Code-Remote | P2 | M | -- | TODO |
| NO-005 | Telegram notifications | Send agent events to Telegram | claude-code-telegram, Claude-Code-Remote | P2 | M | NO-004 | TODO |
| NO-006 | Telegram reply-to-send | Execute commands by replying | claude-code-telegram | P2 | L | NO-005 | TODO |
| NO-007 | Telegram multi-project | Separate threads per project | claude-code-telegram | P3 | M | NO-005 | TODO |
| NO-008 | Discord webhook | Send rich embeds to Discord | Claude-Code-Remote | P2 | M | OB-001 | TODO |
| NO-009 | Discord thread creation | Create thread for long-running agents | -- | P3 | M | NO-008 | TODO |
| NO-010 | Email notifications (SMTP) | Send emails via SMTP | Claude-Code-Remote | P2 | M | OB-001 | TODO |
| NO-011 | Email HTML format | Rich HTML email templates | Claude-Code-Remote | P2 | M | NO-010 | TODO |
| NO-012 | Email attachment | Attach session export to email | -- | P3 | M | NO-010, SM-009 | TODO |
| NO-013 | Desktop notifications (macOS) | Native notification center | Claude-Code-Remote | P1 | M | OB-001 | TODO |
| NO-014 | Desktop notifications (Linux) | libnotify / D-Bus | Claude-Code-Remote | P1 | M | OB-001 | TODO |
| NO-015 | Desktop sound alerts | Configurable sound on events | Claude-Code-Remote | P2 | S | NO-013 | TODO |
| NO-016 | Notification rules | Configurable event-to-channel routing | -- | P1 | L | NO-001 | TODO |
| NO-017 | Default notification rules | Pre-configured rules for common events | -- | P1 | S | NO-016 | TODO |
| NO-018 | Notification test button | Test notification delivery | -- | P1 | S | NO-016 | TODO |
| NO-019 | Multi-channel delivery | Same event to multiple channels | Claude-Code-Remote | P1 | M | NO-016 | TODO |
| NO-020 | Output verbosity levels | Configurable detail in notifications | claude-code-telegram | P2 | S | NO-001 | TODO |

**Subtotal**: 20 features

---

## 10. Security

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| SE-001 | Permission: file read | Control agent file read access | claude-code-config | P0 | M | AM-006 | TODO |
| SE-002 | Permission: file write | Control agent file write access | claude-code-config | P0 | M | AM-006 | TODO |
| SE-003 | Permission: shell execute | Control agent shell command execution | claude-code-config | P0 | M | AM-006 | TODO |
| SE-004 | Permission: network | Control agent network access | claude-code-config | P1 | M | AM-006 | TODO |
| SE-005 | Permission: Git ops | Control agent Git operations | claude-code-config | P1 | S | AM-006 | TODO |
| SE-006 | Permission: MCP tools | Control which MCP tools agents can invoke | claude-code-config | P1 | M | PL-006 | TODO |
| SE-007 | Permission: per-agent | Set permissions per agent | claude-code-config | P0 | M | SE-001 | TODO |
| SE-008 | Permission: per-project | Set permissions per project | claude-code-config | P1 | S | SE-007 | TODO |
| SE-009 | Permission: global | Set global permission defaults | claude-code-config | P0 | S | SE-001 | TODO |
| SE-010 | Audit log (SQLite) | Immutable audit log in database | claude-code-config | P0 | L | -- | TODO |
| SE-011 | Audit: tamper detection | Checksums on audit entries | -- | P1 | M | SE-010 | TODO |
| SE-012 | Audit: export JSONL | Export for SIEM integration | -- | P1 | M | SE-010 | TODO |
| SE-013 | Audit: retention policy | Configurable retention with archive | -- | P2 | M | SE-010 | TODO |
| SE-014 | Audit: UI viewer | Browse and filter audit entries | -- | P1 | L | SE-010 | TODO |
| SE-015 | Security scanning (diff-aware) | Scan only changed lines for vulnerabilities | claude-code-security-review | P1 | XL | GI-003 | TODO |
| SE-016 | Scan: injection detection | SQL, command, XSS injection | claude-code-security-review | P1 | M | SE-015 | TODO |
| SE-017 | Scan: auth bypass detection | Authentication/authorization issues | claude-code-security-review | P1 | M | SE-015 | TODO |
| SE-018 | Scan: data exposure | Secrets, PII, sensitive data | claude-code-security-review | P1 | M | SE-015 | TODO |
| SE-019 | Scan: crypto issues | Weak cryptography, hardcoded keys | claude-code-security-review | P2 | M | SE-015 | TODO |
| SE-020 | Scan: false positive filter | Reduce noise with configurable sensitivity | claude-code-security-review | P1 | M | SE-015 | TODO |
| SE-021 | /security-review command | On-demand security scan | claude-code-security-review | P1 | M | SE-015 | TODO |
| SE-022 | Scan: inline results | Display scan results inline in diff viewer | claude-code-security-review | P2 | M | SE-015, GI-003 | TODO |

**Subtotal**: 22 features

---

## 11. Configuration

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| CF-001 | Global config (~/.claude-forge/config.toml) | Global configuration file | claude-code-config, claude-code-settings | P0 | M | -- | TODO |
| CF-002 | User config (~/.claude-forge/user.toml) | User-level overrides | claude-code-config | P1 | S | CF-001 | TODO |
| CF-003 | Project config (.claude-forge.toml) | Project-level overrides | claude-code-config | P1 | S | CF-001 | TODO |
| CF-004 | Agent-level config | Inline agent overrides | -- | P1 | S | CF-001, AM-001 | TODO |
| CF-005 | Config merge hierarchy | Global < User < Project < Agent | claude-code-config2, claude-code-settings | P0 | M | CF-001 | TODO |
| CF-006 | Schema validation (TOML) | Type and format validation | claude-code-config | P0 | M | CF-001 | TODO |
| CF-007 | Semantic validation | Value range, path existence, cron format | claude-code-config | P1 | M | CF-006 | TODO |
| CF-008 | Validation error messages | Clear errors with line numbers and suggestions | -- | P1 | M | CF-006 | TODO |
| CF-009 | Config export | Single merged TOML export | claude-code-settings | P1 | S | CF-005 | TODO |
| CF-010 | Config import | Apply TOML config with scope mapping | claude-code-settings | P1 | M | CF-005 | TODO |
| CF-011 | Selective import | Choose sections to import | -- | P2 | M | CF-010 | TODO |
| CF-012 | Settings UI | Graphical settings editor | claude-code-showcase | P1 | XL | CF-005 | TODO |
| CF-013 | Settings: scope indicator | Show which scope each setting comes from | claude-code-showcase | P1 | S | CF-012 | TODO |
| CF-014 | Settings: inline docs | Documentation for every setting | -- | P2 | M | CF-012 | TODO |
| CF-015 | Read CLAUDE.md | Import as system prompt additions | claude-code-docs | P0 | M | AM-006 | TODO |
| CF-016 | Read settings.json | Map Claude Code settings to Forge | claude-code-settings | P0 | M | CF-001 | TODO |
| CF-017 | Read .ralphrc | Import safety settings | ralph-claude-code | P1 | M | CF-001 | TODO |
| CF-018 | Read codemcp.toml | Import shell commands and project settings | codemcp | P2 | M | CF-001 | TODO |
| CF-019 | Discover SKILL.md files | Find and import skill files in project | claude-code-plugins-plus-skills | P1 | M | SK-016 | TODO |
| CF-020 | Health check | System diagnostic validator | claude-code-templates | P1 | L | CF-001 | TODO |
| CF-021 | Health: HTTP endpoint | GET /health returns status JSON | -- | P0 | S | CF-020 | TODO |
| CF-022 | Health: CLI command | forge health from terminal | -- | P1 | S | CF-020 | TODO |
| CF-023 | Health: MCP tool | health_check MCP tool | -- | P1 | S | CF-020 | TODO |
| CF-024 | Dark/light theme | Automatic detection with manual override | claude-code-viewer | P1 | M | -- | TODO |
| CF-025 | Keyboard shortcuts | All primary actions accessible via keyboard | -- | P1 | L | -- | TODO |

**Subtotal**: 25 features

---

## 12. MCP Server

| ID | Feature | Description | Source Repos | Priority | Effort | Dependencies | Status |
|----|---------|-------------|-------------|----------|--------|-------------|--------|
| MC-001 | MCP server mode | --mcp flag starts server without UI | claude-code-mcp | P0 | L | -- | TODO |
| MC-002 | Transport: stdio | Direct process invocation transport | claude-code-mcp | P0 | M | MC-001 | TODO |
| MC-003 | Transport: SSE | HTTP-based Server-Sent Events transport | claude-code-mcp | P1 | M | MC-001 | TODO |
| MC-004 | Transport: WebSocket | Persistent connection transport | claude-code-mcp | P2 | M | MC-001 | TODO |
| MC-005 | MCP handshake | Protocol negotiation and capability advertisement | claude-code-mcp | P0 | M | MC-001 | TODO |
| MC-006 | Tool: create_agent | Create agent via MCP | -- | P0 | M | MC-001, AM-001 | TODO |
| MC-007 | Tool: run_agent | Run agent via MCP | claude-code-mcp | P0 | M | MC-001, AM-006 | TODO |
| MC-008 | Tool: stop_agent | Stop agent via MCP | -- | P0 | S | MC-007 | TODO |
| MC-009 | Tool: list_agents | List agents via MCP | -- | P0 | S | MC-001 | TODO |
| MC-010 | Tool: get_agent_status | Get running status, cost, circuit breaker state | -- | P1 | M | MC-007 | TODO |
| MC-011 | Tool: search_sessions | Full-text session search via MCP | -- | P1 | M | MC-001, SM-006 | TODO |
| MC-012 | Tool: get_session | Get full session content via MCP | -- | P1 | M | MC-001, SM-005 | TODO |
| MC-013 | Tool: export_session | Export session in JSON/Markdown via MCP | -- | P1 | S | MC-012 | TODO |
| MC-014 | Tool: resume_session | Resume session via MCP | -- | P1 | M | MC-001, AM-007 | TODO |
| MC-015 | Tool: search_skills | Search skill catalog via MCP | -- | P1 | S | MC-001, SK-004 | TODO |
| MC-016 | Tool: install_skill | Install skill via MCP | -- | P1 | M | MC-001, SK-007 | TODO |
| MC-017 | Tool: activate_skill | Activate skill for agent via MCP | -- | P1 | S | MC-016 | TODO |
| MC-018 | Tool: git_status | Get Git status via MCP | codemcp | P1 | S | MC-001, GI-001 | TODO |
| MC-019 | Tool: git_diff | Get Git diff via MCP | codemcp | P1 | S | MC-001, GI-003 | TODO |
| MC-020 | Tool: git_commit | Create commit via MCP | codemcp | P1 | M | MC-001, GI-009 | TODO |
| MC-021 | Tool: create_worktree | Create git worktree via MCP | -- | P1 | M | MC-001, GI-016 | TODO |
| MC-022 | Tool: remove_worktree | Remove git worktree via MCP | -- | P1 | S | MC-021 | TODO |
| MC-023 | Tool: get_config | Read configuration via MCP | -- | P1 | S | MC-001, CF-005 | TODO |
| MC-024 | Tool: set_config | Update configuration via MCP | -- | P2 | M | MC-023 | TODO |
| MC-025 | Tool: health_check | Run health check via MCP | -- | P1 | S | MC-001, CF-020 | TODO |
| MC-026 | Tool: get_cost_summary | Get cost data via MCP | -- | P1 | S | MC-001, OB-014 | TODO |
| MC-027 | Tool: get_circuit_breaker_state | Get circuit breaker state via MCP | -- | P1 | S | MC-001, SR-001 | TODO |
| MC-028 | Resource: forge://agents | Agent list resource | -- | P1 | M | MC-001 | TODO |
| MC-029 | Resource: forge://sessions | Session list resource | -- | P1 | M | MC-001 | TODO |
| MC-030 | Resource: forge://skills | Skill catalog resource | -- | P2 | M | MC-001 | TODO |
| MC-031 | Resource: forge://config/{scope} | Configuration resource | -- | P2 | S | MC-001 | TODO |
| MC-032 | Resource: forge://git/status | Git status resource | -- | P2 | S | MC-001 | TODO |
| MC-033 | Resource: forge://cost/{period} | Cost data resource | -- | P2 | S | MC-001 | TODO |
| MC-034 | Resource: forge://events | Live event subscription | -- | P1 | L | MC-001 | TODO |
| MC-035 | Prompt: create_feature | Feature implementation prompt template | -- | P2 | S | MC-001 | TODO |
| MC-036 | Prompt: fix_bug | Bug fix prompt template | -- | P2 | S | MC-001 | TODO |
| MC-037 | Prompt: review_code | Code review prompt template | -- | P2 | S | MC-001 | TODO |
| MC-038 | Prompt: write_tests | Test writing prompt template | -- | P2 | S | MC-001 | TODO |
| MC-039 | Permission bypass mode | --dangerously-skip-permissions flag | claude-code-mcp | P1 | M | MC-001 | TODO |
| MC-040 | Client compatibility: Claude Desktop | Tested with Claude Desktop | claude-code-mcp | P0 | M | MC-001 | TODO |
| MC-041 | Client compatibility: Cursor | Tested with Cursor | claude-code-mcp | P1 | M | MC-001 | TODO |
| MC-042 | Client compatibility: Windsurf | Tested with Windsurf | claude-code-mcp | P2 | M | MC-001 | TODO |
| MC-043 | MCP spec compliance | Pass MCP conformance tests | claude-code-mcp | P0 | L | MC-001 | TODO |

**Subtotal**: 43 features (largest bounded context due to dual-mode architecture)

---

## 13. Summary Statistics

### Features by Bounded Context

| Context | Features | % of Total |
|---------|----------|-----------|
| MCP Server | 43 | 20.2% |
| Agent Management | 30 | 14.1% |
| Observability | 30 | 14.1% |
| Session Management | 25 | 11.7% |
| Skills | 25 | 11.7% |
| Configuration | 25 | 11.7% |
| Git Integration | 25 | 11.7% |
| Safety and Reliability | 23 | 10.8% |
| Security | 22 | 10.3% |
| Workflows | 20 | 9.4% |
| Notifications | 20 | 9.4% |
| Plugins | 17 | 8.0% |
| **Total** | **213** | -- |

Note: Percentages sum to > 100% due to rounding, but total is 213 unique features.

### Features by Priority

| Priority | Count | % of Total |
|----------|-------|-----------|
| P0 | 58 | 27.2% |
| P1 | 101 | 47.4% |
| P2 | 42 | 19.7% |
| P3 | 12 | 5.6% |
| **Total** | **213** | 100% |

### Features by Effort

| Effort | Count | % of Total | Estimated Days |
|--------|-------|-----------|----------------|
| S (< 1 day) | 61 | 28.6% | ~45 days |
| M (1-3 days) | 104 | 48.8% | ~210 days |
| L (3-7 days) | 37 | 17.4% | ~185 days |
| XL (1-3 weeks) | 11 | 5.2% | ~110 days |
| **Total** | **213** | 100% | **~550 dev-days** |

### Features by Status

| Status | Count | % of Total |
|--------|-------|-----------|
| DONE | 22 | 10.3% |
| WIP | 4 | 1.9% |
| TODO | 187 | 87.8% |
| **Total** | **213** | 100% |

### P0 Features by Context (Critical Path)

| Context | P0 Count | Key P0 Features |
|---------|----------|-----------------|
| Agent Management | 9 | CRUD, presets, process spawning, session resume, split-pane |
| Observability | 8 | Event capture, WebSocket, swim-lane, cost tracking, transcript |
| Session Management | 8 | Browser, FTS, resume, cost per session, filtering |
| MCP Server | 7 | Server mode, stdio transport, handshake, core tools, compliance |
| Safety and Reliability | 6 | Circuit breaker, rate limiter, exit gate, file protection, budget |
| Security | 5 | Permissions (read, write, execute), per-agent, global |
| Configuration | 4 | Global config, merge hierarchy, schema validation, CLAUDE.md |
| Plugins | 3 | MCP client (stdio), tool discovery, server editor |
| Workflows | 2 | L1 workflow, workflow abort |
| Skills | 0 | -- (all P1, catalog system) |
| Notifications | 0 | -- (all P1+, not blocking) |

---

## 14. Absorption Matrix

This matrix shows which source repositories contribute features to which bounded context. Each cell indicates the number of features absorbed.

### Primary Contributions (3+ features)

| Source Repository | AM | SR | WF | SM | SK | PL | GI | OB | NO | SE | CF | MC |
|-------------------|----|----|----|----|----|----|----|----|----|----|----|----|
| 1code | 6 | | | | | 1 | 10 | | | | | |
| claude-code-viewer | | | | 12 | | | 4 | 1 | | | 1 | |
| Claude-Code-Workflow | 3 | | 16 | | | | | | | | | |
| ralph-claude-code | 4 | 12 | | | | | | | | | 1 | |
| hooks-observability | 3 | | | | | | | 14 | | | | |
| claude-code-plugins-plus-skills | | | | | 13 | 2 | | | | | 1 | |
| Claude-Code-Usage-Monitor | | | | 1 | | | | 8 | | | | |
| claude-code-hub | | 5 | | | | | | 2 | 1 | | | |
| claude-code-mcp | | | | | | 4 | | | | | | 8 |
| claude-code-config | | | | | | | | | | 7 | 3 | |
| claude-code-security-review | | | | | | | | | | 7 | | |
| Claude-Code-Remote | | | | | | | | | 8 | | | |
| claude-code-telegram | | | | | | | | | 4 | | | |
| codemcp | | | | | | | 3 | | | | 1 | 3 |
| claude-code-templates | | | | | 1 | 1 | | | | | 1 | |
| claude-code-action | | | 2 | | | | 2 | | | | | |
| claude-code-subagents | 3 | | | | | | | | | | | |
| claude-code-skill-factory | | | | | 3 | | | | | | | |
| claude-code-skills | | | | | 3 | | | | | | | |
| claude-code-router | 3 | | | | | | | | | | | |
| claude_code_bridge | 3 | | | | | | | | | | | |

### Repos with No Direct Feature Absorption (Content/Knowledge Only)

These repositories contribute patterns, best practices, documentation, or learning content rather than directly absorbable features:

| Repository | Category | Contribution Type |
|-----------|----------|-------------------|
| claude-code-showcase | Config & Settings | Example configurations |
| claude-code-settings | Config & Settings | Settings reference |
| claude-code-config2 | Config & Settings | Configuration patterns |
| awesome-claude-code | Curated Guides | Discovery, community links |
| awesome-claude-code-subagents | Curated Guides | Agent pattern catalog |
| claude-code-guide | Curated Guides | Best practices |
| claude-code-best-practice | Curated Guides | Coding patterns |
| claude-code-tips | Curated Guides | Tips and tricks |
| claude-code-cheat-sheet | Curated Guides | Quick reference |
| claude-code-mastering | Curated Guides | Advanced techniques |
| claude-code-docs | Docs & Internals | Official documentation mirror |
| claude-code-system-prompts | Docs & Internals | System prompt archive |
| claude-code-prompt-improver | Prompts & Learning | Prompt enhancement patterns |
| claude-code-pm-course | Prompts & Learning | Project management course |
| claude-code-requirements-builder | Prompts & Learning | Requirements methodology |
| claude-code-transcripts | Transcripts & Misc | Session export patterns |
| claude-code-my-workflow | Transcripts & Misc | Workflow documentation |
| claude-code-infrastructure-showcase | Transcripts & Misc | Infrastructure patterns |
| claude-code-reverse | Transcripts & Misc | Reverse engineering docs |
| claude-code-communication | Transcripts & Misc | Communication patterns |
| edmunds-claude-code | Transcripts & Misc | Enterprise usage patterns |
| claude-coder | Transcripts & Misc | LLM visualization concepts |
| claude-code-webui | Desktop & IDEs | Web UI patterns |
| claude-code-chat | Desktop & IDEs | Chat interface patterns |
| claude-code-nvim | Desktop & IDEs | Editor integration patterns |
| claude-code-ide-el | Desktop & IDEs | Editor integration patterns |
| idea-claude-code-gui | Desktop & IDEs | IDE plugin patterns |
| codexbar | Desktop & IDEs | Menu bar agent patterns |
| claude-code-workflows | Orchestration | Alternative workflow patterns |
| claude-code-spec-workflow | Orchestration | Spec-driven workflow patterns |
| claude-code-development-kit | Orchestration | Dev kit scripts |
| claude-code-proxy | Remote & Infra | Proxy architecture patterns |
| claude-code-hub | Remote & Infra | API gateway patterns |
| claude-code-hooks-mastery | Hooks & Observability | Hook configuration guide |
| claude-code-tools | MCP & Tooling | Tool patterns |
| everything-claude-code | Templates & Skills | Comprehensive reference |
| claude-code-tresor | Templates & Skills | Curated plugin set |
| my-claude-code-setup | Templates & Skills | Personal setup reference |
| claude-code-sub-agents | Subagents | Agent patterns |
| claude-code-agents | Subagents | QA agent patterns |
| claude-code-remote | Remote & Infra | Remote access patterns |

### Feature Flow Summary

```
61 Source Repositories
    |
    v
+-- 28 repos contribute directly to features (213 features)
|   +-- Top 5 contributors: hooks-observability (14), Claude-Code-Workflow (16),
|   |   ralph-claude-code (12), claude-code-viewer (12), 1code (17)
|   +-- Features feed into 12 bounded contexts
|
+-- 33 repos contribute patterns, docs, and knowledge
    +-- Inform architecture decisions
    +-- Provide example configurations
    +-- Supply learning content for documentation
```

### Priority Distribution by Context

```
           P0 ||||||||  P1 ||||||||||||||||  P2 ||||||||  P3 |||
Agent Mgmt  ████████░░░░░░░░░░░░░░░░░░░░░░  9P0  6P1  10P2  5P3
Safety      ██████░░░░░░░░░░░░░░░░░░░       6P0  12P1  3P2   2P3
Workflows   ██░░░░░░░░░░░░░░░░░░░           2P0  11P1  6P2   1P3
Sessions    ████████░░░░░░░░░░░░░░░░░       8P0  8P1   8P2   1P3
Skills      ░░░░░░░░░░░░░░░░░░░░░           0P0  15P1  7P2   3P3
Plugins     ███░░░░░░░░░░░░░░░              3P0  6P1   5P2   3P3
Git         ░░░░░░░░░░░░░░░░░░░░░░░░░      0P0  16P1  8P2   1P3
Observe     ████████░░░░░░░░░░░░░░░░░░░░░  8P0  12P1  5P2   5P3
Notify      ░░░░░░░░░░░░░░░░░░░░           0P0  9P1   7P2   4P3
Security    █████░░░░░░░░░░░░░░░░░░        5P0  10P1  5P2   2P3
Config      ████░░░░░░░░░░░░░░░░░░░░░░░   4P0  12P1  5P2   4P3
MCP Server  ███████░░░░░░░░░░░░░░░░░░░░░░ 7P0  18P1  12P2  6P3
```

---

## Appendix: Feature ID Index

For quick lookup, features are numbered by context prefix:

| Prefix | Context | Range |
|--------|---------|-------|
| AM- | Agent Management | 001-030 |
| SR- | Safety and Reliability | 001-023 |
| WF- | Workflows | 001-020 |
| SM- | Session Management | 001-025 |
| SK- | Skills | 001-025 |
| PL- | Plugins | 001-017 |
| GI- | Git Integration | 001-025 |
| OB- | Observability | 001-030 |
| NO- | Notifications | 001-020 |
| SE- | Security | 001-022 |
| CF- | Configuration | 001-025 |
| MC- | MCP Server | 001-043 |
