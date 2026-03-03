# Submodule Tracking and Ecosystem Absorption System

> The complete playbook for keeping Claude Forge synchronized with 62 independently evolving repositories -- indefinitely.

---

## Table of Contents

1. [The Living Ecosystem Problem](#1-the-living-ecosystem-problem)
2. [Git Submodule Architecture](#2-git-submodule-architecture)
3. [Upstream Monitoring System](#3-upstream-monitoring-system)
4. [Change Classification](#4-change-classification)
5. [Auto-Import Pipeline](#5-auto-import-pipeline)
6. [Manual Review Pipeline](#6-manual-review-pipeline)
7. [Claude Code Tracking](#7-claude-code-tracking)
8. [Ecosystem Health Dashboard](#8-ecosystem-health-dashboard)
9. [Fork Management](#9-fork-management)
10. [Automation Scripts](#10-automation-scripts)
11. [GitHub Actions Workflows](#11-github-actions-workflows)
12. [.gitmodules Template](#12-gitmodules-template)

---

## 1. The Living Ecosystem Problem

### Why One-Time Absorption Is Not Enough

Claude Forge absorbs features from 62 repositories (61 community repos + Claude Code itself). The initial absorption -- running the ANALYZE / EXTRACT / DESIGN / IMPLEMENT / VALIDATE pipeline described in `ABSORPTION_PIPELINE.md` -- captures each repository's state at a point in time. But that state is not permanent:

- **The 62 repos evolve independently.** Each has its own maintainers, its own release cadence, its own backlog. A repo we absorbed three months ago may have added five new features since then.
- **Claude Code itself adds new capabilities.** Anthropic ships new hook event types, new CLI flags, new MCP tool types, new output format variants, and new system prompt structures. Each change can unlock new Forge features or break existing integrations.
- **Community innovations appear unpredictably.** A repo in the "Orchestration" category might invent a new DAG execution pattern. A "Hooks" repo might discover a clever way to intercept sub-agent spawning. A "Skills" repo might introduce a new skill schema version. Forge must detect these innovations and decide whether to adopt them.
- **Dependencies shift.** A repo we depend on might change its API, switch frameworks, or get archived. A new repo might appear that supersedes an existing one.

### The Consequence

Without a systematic tracking and absorption process, Forge degrades over time. Not in the sense that it breaks, but in the sense that it falls behind. Users comparing Forge to the broader ecosystem will find features that exist in community tools but not in Forge. The gap widens with every month of neglect.

### The Goal

Build a system where:

1. Every upstream change across all 62 repos is **detected** within one week.
2. Every detected change is **classified** (data / pattern / UI / breaking / new-capability).
3. Data-type changes are **auto-imported** with minimal human intervention.
4. Pattern-type changes are **queued for review** with enough context to make a fast decision.
5. Claude Code changes trigger **immediate compatibility assessment**.
6. The overall ecosystem health is **visible** at a glance.

---

## 2. Git Submodule Architecture

### Directory Structure

All 62 repositories live under `ecosystem/` as git submodules, organized by category number and slug matching the `reference-map/` directory structure:

```
ecosystem/
├── claude-code/                              # Anthropic's Claude Code (system prompts repo)
├── 01-desktop-ides/
│   ├── 1code/                                # Multi-agent desktop app
│   ├── claude-code-viewer/                   # Web-based client
│   ├── idea-claude-code-gui/                 # IntelliJ plugin
│   ├── CodexBar/                             # macOS menu bar / Linux CLI
│   ├── claude-code.nvim/                     # Neovim integration
│   ├── claude-code-ide.el/                   # Emacs integration
│   ├── claude-code-chat/                     # VS Code chat
│   └── claude-code-webui/                    # Web UI for Claude CLI
├── 02-orchestration/
│   ├── Claude-Code-Workflow/                 # JSON-driven multi-agent framework
│   ├── Claude-Code-Development-Kit/          # Custom workflow: hooks, MCP, sub-agents
│   ├── claude-code-router/                   # Coding infrastructure foundation
│   ├── claude-code-spec-workflow/            # Spec-driven workflow
│   ├── claude-code-workflows/                # Workflow configs from AI-native startup
│   ├── claude_code_bridge/                   # Split-pane terminal multi-model sync
│   └── ralph-claude-code/                    # Autonomous dev loop with circuit breaker
├── 03-hooks-observability/
│   ├── Claude-Code-Usage-Monitor/            # Real-time usage monitor
│   ├── claude-code-hooks-mastery/            # Guide and examples for hooks
│   └── claude-code-hooks-multi-agent-observability/  # Real-time agent monitoring
├── 04-templates-skills-plugins/
│   ├── claude-code-cookbook/                  # Cookbook recipes and examples
│   ├── claude-code-plugins-plus-skills/      # 270+ plugins, 1500+ skills, CCPI
│   ├── claude-code-skill-factory/            # Toolkit for building skills
│   ├── claude-code-skills/                   # Marketplace of 38 production skills
│   ├── claude-code-templates/                # 100+ agents, commands, settings, hooks
│   ├── claude-code-tresor/                   # Collection of utilities
│   ├── everything-claude-code/               # Complete config from hackathon winner
│   └── my-claude-code-setup/                 # Starter template + memory bank
├── 05-subagents-agents/
│   ├── ClaudeCodeAgents/                     # QA agents
│   ├── claude-code-sub-agents/               # Specialized full-stack subagents
│   └── claude-code-subagents/                # 100+ production-ready subagents
├── 06-mcp-tooling/
│   ├── claude-code-mcp/                      # Claude Code as one-shot MCP server
│   ├── codemcp/                              # Coding assistant MCP for Claude Desktop
│   └── claude-code-tools/                    # Productivity tools for CLI agents
├── 07-remote-infra/
│   ├── Claude-Code-Remote/                   # Remote control via email/Discord/Telegram
│   ├── claude-code-hub/                      # API proxy: load balancing, multi-provider
│   ├── claude-code-proxy/                    # Run Claude Code on OpenAI models
│   └── claude-code-telegram/                 # Telegram bot with session persistence
├── 08-automation-cicd/
│   └── claude-code-action/                   # GitHub Action: PR/issue integration
├── 09-config-settings/
│   ├── claude-code-config/                   # Opinionated defaults (Trail of Bits)
│   ├── claude-code-config2/                  # Personal config: rules, hooks, agents
│   ├── claude-code-settings/                 # Settings for vibe coding
│   └── claude-code-showcase/                 # Example project config
├── 10-curated-guides/
│   ├── awesome-claude-code/                  # Curated list: skills, tooling, IDEs
│   ├── awesome-claude-code-subagents/        # Curated subagents (127+)
│   ├── claude-code-best-practice/            # Practice guide: skills, agents, memory
│   ├── claude-code-cheat-sheet/              # Quick reference tips
│   ├── claude-code-guide/                    # Community guide: install, tips, MCP
│   ├── claude-code-mastering/                # Learning resource
│   └── claude-code-tips/                     # 45 tips: status line, worktrees, etc.
├── 11-docs-internals/
│   ├── claude-code-docs/                     # Local mirror of official docs
│   └── claude-code-system-prompts/           # System prompts, reminders, changelog
├── 12-prompts-learning/
│   ├── claude-code-pm-course/                # Interactive course for PMs
│   ├── claude-code-prompt-improver/          # Hook for improving prompts
│   └── claude-code-requirements-builder/     # Build requirements with Claude Code
└── 13-transcripts-security-misc/
    ├── Claude-Code-Communication/            # Communication / messaging patterns
    ├── claude-code-infrastructure-showcase/   # Example infra: skills, hooks, agents
    ├── claude-code-my-workflow/              # Template for academics: LaTeX + R
    ├── claude-code-reverse/                  # Visualize Claude Code LLM interactions
    ├── claude-code-security-review/          # GitHub Action: AI security review
    ├── claude-code-transcripts/              # Tools for publishing transcripts
    ├── claude-coder/                         # Autonomous coding agent (Kodu)
    └── edmunds-claude-code/                  # Edmunds-specific config
```

### Submodule Configuration

Each submodule is registered in `.gitmodules` at the project root. We track the default branch (usually `main` or `master`) and allow the submodule pointer to advance independently from the parent project's commits. This means `git submodule update --remote` pulls the latest from each upstream without requiring a parent commit for every upstream change.

### Remote Naming Convention

Following the pattern established in `scripts/sync-remotes.sh` and documented in `docs/reference-repos-remotes.md`:

| Remote | Points To | Purpose |
|--------|-----------|---------|
| `origin` | `GITHUB_USER/<repo>` (our fork) | Push target for any local patches |
| `upstream` | Original author's repo | Pull source for tracking upstream changes |

This means:
- `git fetch upstream` in any submodule gets the latest from the canonical source.
- `git push origin` sends our patches to our fork.
- `git pull upstream main` (or `master`) updates our local tracking branch.

### Branch Tracking Strategy

Each submodule tracks a single branch from upstream:

1. **Default**: Track the upstream's default branch (`main` or `master`).
2. **Pinned**: For repos where we need stability (e.g., `claude-code-system-prompts` during a Forge release), pin to a specific tag or commit hash.
3. **Custom**: For repos where we carry patches, maintain a `forge-patches` branch that rebases on top of upstream.

The tracking configuration is stored in `.gitmodules`:

```ini
[submodule "ecosystem/01-desktop-ides/1code"]
    path = ecosystem/01-desktop-ides/1code
    url = https://github.com/anthropics/1code.git
    branch = main
```

---

## 3. Upstream Monitoring System

### Design

The monitoring system runs on a weekly cadence (configurable to daily for high-priority repos). It:

1. Fetches the latest state of all 62 submodule upstreams.
2. Compares each upstream's HEAD against our tracked commit.
3. Classifies the delta (new commits, new files, changed files, new releases, new tags).
4. Generates an **Ecosystem Diff Report** summarizing all changes.
5. Posts the report as a GitHub Issue (or updates an existing tracking issue).
6. Sends notifications for significant changes.

### Monitoring Script

The primary monitoring logic lives in `scripts/update-ecosystem.sh` (full implementation in Section 10). The script performs:

```
For each submodule:
  1. git fetch upstream
  2. LOCAL_SHA = current submodule HEAD
  3. REMOTE_SHA = upstream/main HEAD
  4. if LOCAL_SHA != REMOTE_SHA:
     a. DIFF = git log LOCAL_SHA..REMOTE_SHA --oneline
     b. FILES_CHANGED = git diff --name-only LOCAL_SHA REMOTE_SHA
     c. STAT = git diff --stat LOCAL_SHA REMOTE_SHA
     d. Check for new tags (releases)
     e. Classify changes (see Section 4)
     f. Append to report
```

### Notification Triggers

Not all changes warrant the same level of attention. The system categorizes urgency:

| Urgency | Trigger | Action |
|---------|---------|--------|
| **Critical** | Claude Code new release; breaking API change in any repo | GitHub Issue labeled `ecosystem:critical` + Slack/email notification |
| **High** | New skill/agent/preset files; new hook patterns | GitHub Issue labeled `ecosystem:high` |
| **Normal** | README updates, config tweaks, minor refactors | Batched into weekly digest issue |
| **Low** | Typo fixes, CI changes, documentation-only commits | Logged in report, no issue |

### Diff Report Format

The ecosystem diff report is a structured Markdown document:

```markdown
# Ecosystem Diff Report -- 2026-02-25

## Summary
- 62 repos checked
- 14 repos with changes
- 3 critical changes, 5 high, 6 normal
- 0 repos unreachable

## Critical Changes

### claude-code-system-prompts (11-docs-internals)
- **8 new commits** since last check (abc1234..def5678)
- New file: `system-prompts/2026-02-20-v3.md`
- Changed: `CHANGELOG.md` (+45 lines)
- **Impact**: New system prompt version may affect Forge's agent spawning

### claude-code-action (08-automation-cicd)
- **Tag**: v2.0.0 (breaking release)
- Changed: `action.yml` (new required inputs)
- **Impact**: Forge's CI integration may need updates

## High Changes
...

## Normal Changes
...

## Unreachable Repos
(none)
```

---

## 4. Change Classification

When the monitoring system detects upstream changes, it classifies each change by examining the files modified. Classification drives what happens next: auto-import, manual review, or just logging.

### Classification Matrix

| Classification | Description | Examples | Next Step |
|---------------|-------------|----------|-----------|
| **Data** | New skills, presets, configs, templates, agent definitions -- content without logic | New `SKILL.md` files, new JSON agent definitions, new `.claude/` config files, new prompt templates | Auto-Import Pipeline (Section 5) |
| **Pattern** | New algorithms, architectural patterns, state machines, pipelines -- behavioral logic | Circuit breaker implementation, DAG executor, rate limiter, new hook patterns, new MCP tool wrappers | Manual Review Pipeline (Section 6) |
| **UI** | New components, layouts, visualizations -- interface changes | Kanban board component, diff viewer, terminal panel, dashboard layout | Design Review + Manual Review |
| **Breaking** | API changes, dependency updates, removed features, schema changes | Changed function signatures, removed endpoints, new required config fields, major version bumps | Impact Analysis + Compatibility Check |
| **New Capability** | Claude Code adds new hooks, tools, MCP features, CLI flags, output formats | New `PreToolUse` hook event, new `--resume` flag, new MCP server capability, new tool type | Immediate Assessment (Section 7) |

### Classification Rules

The classification engine applies rules in order of priority:

```
Rule 1: New tag matching semver major bump (vX.0.0) → Breaking
Rule 2: Files matching SKILL.md, *.skill.json, agents/*.json, presets/*.json → Data
Rule 3: Files matching .claude/settings.json, .claude/commands/*.md → Data
Rule 4: Files matching *.svelte, *.tsx, *.jsx, *.vue, *.css → UI
Rule 5: Files matching src/*, lib/*, core/* with .rs/.ts/.py/.go → Pattern
Rule 6: Files matching CHANGELOG.md mentioning "hook" or "tool" or "mcp" → New Capability
Rule 7: Files matching package.json, Cargo.toml, requirements.txt with major changes → Breaking
Rule 8: Everything else → Normal (logged, no action)
```

### Classification Output

Each classified change produces a record:

```json
{
  "repo": "claude-code-skills",
  "category": "04-templates-skills-plugins",
  "commits": 5,
  "classification": "data",
  "files_changed": [
    "skills/new-refactor-skill/SKILL.md",
    "skills/new-refactor-skill/config.json"
  ],
  "summary": "New skill: refactor-skill (automated code refactoring)",
  "action": "auto-import",
  "urgency": "high",
  "detected_at": "2026-02-25T10:00:00Z"
}
```

---

## 5. Auto-Import Pipeline

For data-type changes -- new skills, presets, configs, agent definitions, and templates -- the system can import automatically with minimal human oversight.

### Pipeline Stages

```
DETECT          EXTRACT         VALIDATE        MERGE           TEST            PR
────────       ─────────       ──────────      ────────        ──────          ────
Monitoring     Pull new data   Schema check,   Insert into     Run Forge       Open PR with
system flags   files from      conflict        SQLite seed     test suite,     changelog and
data change    submodule       detection       data or         verify no       diff of
                                               config files    regressions     imported data
```

### Stage 1: Detect

The monitoring system (Section 3) identifies files matching data patterns:

- `**/SKILL.md` -- Skill definitions
- `**/agents/*.json`, `**/agents/*.yaml` -- Agent presets
- `**/.claude/commands/*.md` -- Slash commands
- `**/.claude/settings.json` -- Settings presets
- `**/presets/*.json` -- Configuration presets
- `**/hooks/*.json`, `**/hooks/*.sh` -- Hook definitions
- `**/mcp-servers/*.json` -- MCP server configurations

### Stage 2: Extract

Pull the identified files from the updated submodule:

```bash
# Update submodule to latest upstream
cd ecosystem/04-templates-skills-plugins/claude-code-skills
git fetch upstream
git merge upstream/main

# Copy new skill files to staging area
cp skills/new-refactor-skill/SKILL.md /tmp/forge-import-staging/
cp skills/new-refactor-skill/config.json /tmp/forge-import-staging/
```

### Stage 3: Validate

Each data type has a schema. Imported data must pass validation before it enters Forge.

**Skill Schema** (JSON Schema):
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["name", "description", "commands"],
  "properties": {
    "name": { "type": "string", "minLength": 1, "maxLength": 100 },
    "description": { "type": "string", "minLength": 1 },
    "commands": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "content"],
        "properties": {
          "name": { "type": "string" },
          "content": { "type": "string" }
        }
      }
    },
    "tags": { "type": "array", "items": { "type": "string" } },
    "source_repo": { "type": "string" },
    "source_version": { "type": "string" }
  }
}
```

**Agent Preset Schema** (JSON Schema):
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["name", "description", "system_prompt"],
  "properties": {
    "name": { "type": "string", "minLength": 1 },
    "description": { "type": "string" },
    "system_prompt": { "type": "string" },
    "allowed_tools": { "type": "array", "items": { "type": "string" } },
    "mcp_servers": { "type": "array", "items": { "type": "string" } },
    "max_turns": { "type": "integer", "minimum": 1 },
    "source_repo": { "type": "string" }
  }
}
```

Validation checks:
1. Schema compliance (required fields, types, constraints).
2. Name uniqueness (no collision with existing Forge skills/agents).
3. Content safety (no hardcoded API keys, no malicious shell commands).
4. Encoding (valid UTF-8, no binary content in text fields).

### Stage 4: Merge

Insert validated data into Forge's SQLite seed data or configuration files.

For skills, generate SQL:
```sql
INSERT INTO skills (name, description, commands_json, tags_json, source_repo, source_version, imported_at)
VALUES (
  'refactor-skill',
  'Automated code refactoring with pattern detection',
  '{"commands": [{"name": "/refactor", "content": "..."}]}',
  '["refactoring", "code-quality"]',
  'claude-code-skills',
  'abc1234',
  datetime('now')
);
```

For agent presets, the insert targets the `agents` table with `is_preset = 1`.

For config presets, the data goes into `forge-config/presets/` as JSON files that the frontend's preset picker reads at runtime.

### Stage 5: Test

Run the Forge test suite to verify the import causes no regressions:

```bash
# Backend tests
cargo test --workspace

# Frontend tests
cd frontend && pnpm test

# Specific import validation
cargo test --test import_validation -- --nocapture

# Start Forge and verify skill appears in UI
cargo run &
sleep 3
curl -s http://localhost:4173/api/skills | jq '.[] | select(.name == "refactor-skill")'
kill %1
```

### Stage 6: PR

Create a pull request with:
- Branch name: `auto-import/<repo-slug>-<date>` (e.g., `auto-import/claude-code-skills-2026-02-25`)
- Title: `auto-import(skills): 3 new skills from claude-code-skills`
- Body: Changelog of imported items, validation results, test results
- Labels: `ecosystem`, `auto-import`, `data`

The PR requires one human approval before merge, providing a safety gate even for automated imports.

---

## 6. Manual Review Pipeline

For pattern-type and UI-type changes -- new algorithms, architectural innovations, component designs -- automation cannot safely make adoption decisions. These require human judgment.

### Pipeline Stages

```
DETECT        REPORT        ASSESS        DECIDE        ABSORB
────────     ─────────     ──────────    ──────────    ──────────
Monitoring   Generate       Evaluate      Go / No-Go    Enter the
flags        feature        effort,       + priority     5-phase
pattern or   comparison     impact,       assignment     Absorption
UI change    report         alignment                    Pipeline
```

### Stage 1: Detect

The monitoring system identifies changes classified as `pattern` or `ui`.

### Stage 2: Report

Generate a feature comparison report for each detected change:

```markdown
## Feature Report: Circuit Breaker Pattern (ralph-claude-code)

### What Changed
- New file: `src/circuit_breaker.ts` (247 lines)
- New file: `src/rate_limiter.ts` (189 lines)
- Modified: `src/agent_loop.ts` (added circuit breaker integration)

### What It Does
Implements a 3-state circuit breaker (Closed, Open, Half-Open) for
agent execution loops. When an agent fails N consecutive times, the
circuit opens and prevents further execution for a cooldown period.
After cooldown, it enters half-open state and allows one test execution.

### Forge Relevance
- Forge's agent spawning (`src/process.rs`) currently has no circuit
  breaker. A runaway agent retries indefinitely.
- This pattern would improve reliability for multi-agent workflows.
- Estimated effort: M (3-5 days) -- need Rust FSM + API + UI indicator.

### Prior Art in Forge
- No existing circuit breaker.
- Related: `src/process.rs` has basic retry logic (exponential backoff)
  that could be extended.

### Recommendation
ABSORB -- high value, moderate effort, aligns with Phase 6A goals.
```

### Stage 3: Assess

The assessment answers five questions (from `ABSORPTION_PIPELINE.md`):

1. **What are we absorbing?** Specific patterns, not "everything."
2. **How does it classify?** Data / Logic / UI.
3. **What size is it?** S / M / L / XL.
4. **What does it depend on?** Prerequisites in Forge.
5. **What bounded context does it belong to?** Agent, Session, Workflow, etc.

### Stage 4: Decide

A Go / No-Go decision based on:

| Factor | Weight | Question |
|--------|--------|----------|
| User impact | 30% | Does this improve the experience for > 5% of users? |
| Strategic alignment | 25% | Does this advance Forge's roadmap? |
| Effort vs. value | 25% | Is the ROI positive given current priorities? |
| Maintenance cost | 10% | Will this create ongoing maintenance burden? |
| Uniqueness | 10% | Is this available only from this repo, or duplicated elsewhere? |

Decisions are recorded as lightweight ADRs (Architecture Decision Records) in `forge-project/03-architecture/decisions/`.

### Stage 5: Absorb

If the decision is Go, the change enters the full 5-phase Absorption Pipeline defined in `ABSORPTION_PIPELINE.md`:

1. **ANALYZE** -- Read the reference, identify patterns, classify, estimate effort.
2. **EXTRACT** -- Extract interface contracts without copying implementation.
3. **DESIGN** -- Map to Forge's Rust/Svelte architecture (contexts, traits, DB schema, API, MCP).
4. **IMPLEMENT** -- Write idiomatic Rust + Svelte + tests.
5. **VALIDATE** -- Verify behavior matches, interfaces work, documentation complete.

---

## 7. Claude Code Tracking

Claude Code is the foundation that Forge builds upon. It is not just another repo in the ecosystem -- it is the runtime that every Forge agent invokes. Changes to Claude Code have outsized impact.

### What We Track

| Aspect | Where to Find It | Impact on Forge |
|--------|-------------------|-----------------|
| **Hook event types** | Claude Code docs, `claude-code-system-prompts` repo, release notes | New hook types enable new observability and intervention patterns in Forge |
| **CLI flags and options** | `claude --help`, release notes, system prompts repo | New flags may need to be exposed in Forge's agent configuration UI |
| **MCP capabilities** | Claude Code docs, MCP spec, release notes | New MCP features enable new tool integrations |
| **Tool types** | System prompts, tool use documentation | New tool types affect Forge's tool permission management |
| **Output format changes** | `--output-format` documentation, `stream-json` schema | Changes to stream-json events affect Forge's real-time rendering |
| **System prompt changes** | `claude-code-system-prompts` repo | Changes affect agent behavior, may require Forge's prompt engineering updates |
| **Rate limits and pricing** | Anthropic API docs, status page | Affects Forge's rate limiter and usage monitor |
| **Permission model** | Docs on tool permissions, `allowedTools`, `disallowedTools` | Affects Forge's agent permission configuration |

### Monitoring Process

1. **Weekly**: Run `check-claude-code.sh` (Section 10) which:
   - Checks `claude --version` against our last recorded version.
   - Fetches latest from `claude-code-system-prompts` submodule.
   - Diffs the system prompt for new tool names, hook types, and capabilities.
   - Checks Anthropic's changelog and release notes (via RSS/API if available).

2. **On Detection**: If a new Claude Code version is found:
   - Parse `claude --help` for new flags.
   - Run Forge's integration test suite against the new version.
   - Document any new capabilities or breaking changes.
   - If breaking: create a `ecosystem:critical` issue.
   - If new capability: create a feature card.

### Known Hook Event Types (Baseline)

Forge tracks these hook events for agent spawning and observability. This list must be updated when Claude Code adds new types:

```
PreToolUse          -- Before a tool is invoked
PostToolUse         -- After a tool completes
Notification        -- System notifications
Stop                -- Agent stop events
SubagentSpawn       -- When a sub-agent is launched (if applicable)
```

When Claude Code adds a new hook event type, Forge must:
1. Add it to the `HookEvent` enum in `src/hooks.rs`.
2. Add it to the hooks editor UI in the frontend.
3. Add it to the observability event stream.
4. Update documentation.

### Known CLI Flags (Baseline)

Forge uses these flags when spawning agents. New flags must be evaluated for inclusion:

```
-p, --prompt            Direct prompt (non-interactive)
--output-format         stream-json | json | text
--verbose               Enable verbose output
--resume                Resume a previous session
--session-id            Specify session ID
--model                 Model selection
--max-turns             Limit conversation turns
--allowedTools          Whitelist of allowed tools
--disallowedTools       Blacklist of disallowed tools
--permission-mode       Permission handling (accept, deny, auto)
--mcp-config            MCP server configuration path
```

### Compatibility Matrix

Maintain a compatibility matrix between Forge versions and Claude Code versions:

| Forge Version | Min Claude Code | Max Claude Code | Notes |
|---------------|-----------------|-----------------|-------|
| 0.1.x | 1.0.0 | * | Initial release, basic integration |
| 0.2.x | 1.1.0 | * | Requires `--resume` flag support |
| 0.3.x | 1.2.0 | * | Requires `stream-json` v2 events |

This matrix is stored in `forge-config/compatibility.json` and checked at Forge startup:

```json
{
  "forge_version": "0.2.0",
  "claude_code": {
    "min_version": "1.1.0",
    "recommended_version": "1.2.0",
    "known_incompatible": ["0.9.x"],
    "required_flags": ["--resume", "--output-format"],
    "required_hooks": ["PreToolUse", "PostToolUse", "Notification"]
  }
}
```

---

## 8. Ecosystem Health Dashboard

### Purpose

A single view that answers: "How current is Forge with respect to the ecosystem?" This can be rendered in Forge's UI (as a future phase) or generated as a static Markdown report.

### Metrics Per Repo

| Metric | Source | Computation |
|--------|--------|-------------|
| **Last upstream commit** | `git log -1 upstream/main --format=%ci` | Date of most recent upstream activity |
| **Days since last check** | Stored in `ecosystem-state.json` | Current date minus last fetch date |
| **Commits behind** | `git rev-list HEAD..upstream/main --count` | Number of unabsorbed commits |
| **GitHub stars** | GitHub API | Popularity indicator |
| **Contributors (30d)** | GitHub API | Activity indicator |
| **Open issues** | GitHub API | Health indicator |
| **Absorption status** | Manual tracking in `absorption-status.json` | Percentage of identified features absorbed |
| **Staleness** | Derived from days-since-check and commits-behind | Red (>30 days, >20 commits), Yellow (>14 days, >10 commits), Green |

### Absorption Status Tracking

Each repo has an absorption record:

```json
{
  "repo": "claude-code-skills",
  "category": "04-templates-skills-plugins",
  "total_features_identified": 38,
  "features_absorbed": 32,
  "features_deferred": 4,
  "features_rejected": 2,
  "absorption_percentage": 84,
  "last_analysis_date": "2026-02-15",
  "last_analysis_commit": "abc1234",
  "notes": "6 remaining skills are niche (LaTeX, Beamer, R) -- deferred to plugin system"
}
```

This is stored in `ecosystem-state/absorption-status.json` and updated during the Manual Review Pipeline.

### Report Format

The `ecosystem-report.sh` script (Section 10) generates a report like:

```markdown
# Ecosystem Health Report -- 2026-02-25

## Overview
- Total repos tracked: 62
- Repos with new changes: 14
- Average absorption: 67%
- Critical issues: 1 (Claude Code v1.3.0 released)

## Status by Category

### 01-desktop-ides (8 repos)
| Repo | Last Commit | Behind | Absorption | Status |
|------|-------------|--------|------------|--------|
| 1code | 2026-02-20 | 3 | 72% | GREEN |
| claude-code-viewer | 2026-02-18 | 7 | 45% | YELLOW |
| ... | ... | ... | ... | ... |

### 02-orchestration (7 repos)
...

## Repos Needing Attention
1. claude-code-plugins-plus-skills -- 47 commits behind, last checked 32 days ago
2. claude-code-templates -- 23 commits behind, new SKILL.md files detected
3. ...

## New Discoveries
Features found in repos that have not been analyzed for absorption:
- claude-code-skills added a "skill-creator" meta-skill
- ralph-claude-code added circuit breaker pattern
- 1code added worktree management UI
```

### Dashboard UI (Future Phase)

When implemented as a Forge UI page (`/ecosystem`), the dashboard will show:

1. **Grid view**: All 62 repos as cards with color-coded status.
2. **Timeline view**: Recent activity across all repos on a timeline.
3. **Dependency graph**: Which Forge features came from which repos.
4. **Alert panel**: Unresolved critical and high issues.

---

## 9. Fork Management

### Strategy

Not all 62 repos need forks. The decision depends on our relationship with the repo:

| Relationship | Strategy | When |
|-------------|----------|------|
| **Read-only tracking** | Submodule pointing to upstream, no fork | Most repos -- we read and absorb, never modify |
| **Fork for patches** | Fork + submodule pointing to our fork, upstream remote added | When we need to carry patches that upstream won't accept |
| **Fork for contribution** | Fork + submodule pointing to our fork, upstream remote added, PRs sent upstream | When we improve something and want to give back |
| **Mirror** | Full mirror of upstream for archival | When a repo is at risk of being deleted |

### Current Fork Layout

Following the pattern in `scripts/sync-remotes.sh`:

```bash
# In each submodule:
origin   = https://github.com/GITHUB_USER/<repo>.git     # Our fork (push target)
upstream = https://github.com/ORIGINAL_OWNER/<repo>.git   # Canonical source (pull source)
```

The `GITHUB_USER` defaults to `mbaneshi` (configurable via `scripts/remotes-config.env`).

### When to Fork

Fork when any of these conditions are true:

1. **We carry patches** that customize the repo for Forge's needs and upstream is unlikely to accept them.
2. **We want to contribute** improvements back via pull requests.
3. **We need stability** -- pin a specific state while upstream moves fast.
4. **Archival risk** -- the repo's owner might delete or go private.

### Contributing Upstream

When Forge's absorption process reveals improvements, fixes, or new patterns that benefit the upstream repo:

1. Create the improvement in our fork on a branch named `forge-contrib/<description>`.
2. Open a PR against the upstream repo.
3. Document the PR in `ecosystem-state/contributions.json`:
   ```json
   {
     "repo": "claude-code-skills",
     "pr_url": "https://github.com/original-owner/claude-code-skills/pull/42",
     "description": "Added schema validation for skill definitions",
     "status": "open",
     "date": "2026-02-25"
   }
   ```
4. Track the PR status. If accepted, rebase our fork on upstream.

### Handling Divergence

When our fork drifts from upstream (because we carry patches or upstream evolves):

1. **Rebase regularly** -- At least monthly, rebase our `forge-patches` branch on `upstream/main`.
2. **Resolve conflicts** -- If conflicts arise, prefer upstream's version unless our patch is critical.
3. **Reduce patch surface** -- Always try to upstream our patches. The smaller the delta between our fork and upstream, the less maintenance burden.
4. **Document divergence** -- In `ecosystem-state/forks.json`, record each repo's divergence status:
   ```json
   {
     "repo": "claude-code-templates",
     "fork_url": "https://github.com/mbaneshi/claude-code-templates",
     "patches": [
       {
         "description": "Added Forge-specific agent presets",
         "branch": "forge-patches",
         "commits_ahead": 3,
         "upstreamable": false
       }
     ],
     "last_rebase": "2026-02-20",
     "divergence_status": "low"
   }
   ```

---

## 10. Automation Scripts

### update-ecosystem.sh

Fetches all submodules from upstream, generates a diff report, classifies changes, and creates a GitHub issue with a summary.

```bash
#!/usr/bin/env bash
# update-ecosystem.sh -- Fetch all ecosystem submodules, detect changes, classify, report.
#
# Usage:
#   ./scripts/update-ecosystem.sh                  # Full run: fetch + classify + report
#   ./scripts/update-ecosystem.sh --dry-run        # Report only, no issue creation
#   ./scripts/update-ecosystem.sh --category 04    # Only check category 04
#
# Prerequisites: git, jq, gh (GitHub CLI)
# Configuration: scripts/remotes-config.env (optional)

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA="$SCRIPT_DIR/reference_repos.json"
STATE_DIR="$REPO_ROOT/ecosystem-state"
REPORT_FILE="$STATE_DIR/reports/ecosystem-diff-$(date +%Y-%m-%d).md"
ECOSYSTEM_DIR="$REPO_ROOT/ecosystem"

# Load config
if [[ -f "$SCRIPT_DIR/remotes-config.env" ]]; then
  set -a; source "$SCRIPT_DIR/remotes-config.env"; set +a
fi
REMOTE_UPSTREAM="${REMOTE_UPSTREAM:-upstream}"

DRY_RUN=false
CATEGORY_FILTER=""
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    --category) shift; CATEGORY_FILTER="$1" ;;
  esac
done

mkdir -p "$STATE_DIR/reports"

# --- State file: tracks last-checked commit per repo ---
STATE_FILE="$STATE_DIR/ecosystem-state.json"
if [[ ! -f "$STATE_FILE" ]]; then
  echo '{}' > "$STATE_FILE"
fi

# --- Category directory mapping ---
declare -A CATEGORY_DIRS
CATEGORY_DIRS=(
  ["Desktop & IDEs"]="01-desktop-ides"
  ["Orchestration & Workflows"]="02-orchestration"
  ["Hooks & Observability"]="03-hooks-observability"
  ["Templates, Skills & Plugins"]="04-templates-skills-plugins"
  ["Subagents & Agents"]="05-subagents-agents"
  ["MCP & Tooling"]="06-mcp-tooling"
  ["Remote & Infra"]="07-remote-infra"
  ["Automation & CI/CD"]="08-automation-cicd"
  ["Config & Settings"]="09-config-settings"
  ["Curated Lists & Guides"]="10-curated-guides"
  ["Docs & System Internals"]="11-docs-internals"
  ["Prompts & Learning"]="12-prompts-learning"
  ["Transcripts, Security & Misc"]="13-transcripts-security-misc"
)

# --- Change classification ---
classify_files() {
  local files="$1"
  local classification="normal"

  # Check for data changes (skills, agents, configs)
  if echo "$files" | grep -qiE '(SKILL\.md|\.skill\.json|agents/.*\.json|presets/.*\.json|\.claude/commands|\.claude/settings)'; then
    classification="data"
  fi

  # Check for UI changes
  if echo "$files" | grep -qiE '\.(svelte|tsx|jsx|vue|css)$'; then
    if [[ "$classification" == "normal" ]]; then
      classification="ui"
    else
      classification="$classification+ui"
    fi
  fi

  # Check for pattern changes
  if echo "$files" | grep -qiE '(src/|lib/|core/)\S+\.(rs|ts|py|go)$'; then
    if [[ "$classification" == "normal" ]]; then
      classification="pattern"
    else
      classification="$classification+pattern"
    fi
  fi

  echo "$classification"
}

classify_urgency() {
  local repo="$1"
  local classification="$2"
  local new_tags="$3"
  local commits_behind="$4"

  # Critical: Claude Code system prompts or major version bump
  if [[ "$repo" == "claude-code-system-prompts" || "$repo" == "claude-code-docs" ]]; then
    echo "critical"
    return
  fi
  if echo "$new_tags" | grep -qE '^v[0-9]+\.0\.0$'; then
    echo "critical"
    return
  fi

  # High: data changes (new skills, agents)
  if [[ "$classification" == *"data"* ]]; then
    echo "high"
    return
  fi

  # High: many commits
  if [[ "$commits_behind" -gt 20 ]]; then
    echo "high"
    return
  fi

  echo "normal"
}

# --- Begin report ---
{
  echo "# Ecosystem Diff Report -- $(date +%Y-%m-%d)"
  echo ""
  echo "## Summary"
} > "$REPORT_FILE"

total_repos=0
changed_repos=0
critical_count=0
high_count=0
normal_count=0
declare -a critical_entries=()
declare -a high_entries=()
declare -a normal_entries=()

# --- Process each repo ---
while IFS=$'\t' read -r slug category; do
  cat_dir="${CATEGORY_DIRS[$category]:-}"
  if [[ -z "$cat_dir" ]]; then
    echo "WARN: Unknown category '$category' for $slug, skipping."
    continue
  fi

  # Category filter
  if [[ -n "$CATEGORY_FILTER" && "$cat_dir" != "$CATEGORY_FILTER"* ]]; then
    continue
  fi

  submodule_path="$ECOSYSTEM_DIR/$cat_dir/$slug"
  if [[ ! -d "$submodule_path/.git" ]] && [[ ! -f "$submodule_path/.git" ]]; then
    echo "SKIP: $slug -- not initialized at $submodule_path"
    continue
  fi

  total_repos=$((total_repos + 1))
  echo "Checking $slug ($cat_dir)..."

  # Fetch upstream
  if ! git -C "$submodule_path" fetch "$REMOTE_UPSTREAM" --tags --quiet 2>/dev/null; then
    echo "  WARN: Could not fetch $REMOTE_UPSTREAM for $slug"
    continue
  fi

  # Get current and remote HEADs
  local_sha=$(git -C "$submodule_path" rev-parse HEAD 2>/dev/null || echo "unknown")
  # Try main, then master, then default
  remote_ref=""
  for branch in main master; do
    if git -C "$submodule_path" rev-parse "$REMOTE_UPSTREAM/$branch" &>/dev/null; then
      remote_ref="$REMOTE_UPSTREAM/$branch"
      break
    fi
  done
  if [[ -z "$remote_ref" ]]; then
    echo "  WARN: No main/master branch found for $slug"
    continue
  fi
  remote_sha=$(git -C "$submodule_path" rev-parse "$remote_ref" 2>/dev/null || echo "unknown")

  # Compare
  if [[ "$local_sha" == "$remote_sha" ]]; then
    # Update state with check timestamp
    tmp=$(mktemp)
    jq --arg s "$slug" --arg d "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      '.[$s].last_checked = $d' "$STATE_FILE" > "$tmp" && mv "$tmp" "$STATE_FILE"
    continue
  fi

  # Changes detected
  changed_repos=$((changed_repos + 1))
  commits_behind=$(git -C "$submodule_path" rev-list HEAD.."$remote_ref" --count 2>/dev/null || echo "?")
  log_output=$(git -C "$submodule_path" log HEAD.."$remote_ref" --oneline --no-decorate 2>/dev/null | head -20)
  files_changed=$(git -C "$submodule_path" diff --name-only HEAD "$remote_ref" 2>/dev/null || echo "")
  stat_output=$(git -C "$submodule_path" diff --stat HEAD "$remote_ref" 2>/dev/null | tail -1)

  # New tags since our tracked commit
  new_tags=$(git -C "$submodule_path" tag --contains "$local_sha" 2>/dev/null | grep -v "$(git -C "$submodule_path" tag --contains "$remote_sha" 2>/dev/null)" || echo "")

  # Classify
  classification=$(classify_files "$files_changed")
  urgency=$(classify_urgency "$slug" "$classification" "$new_tags" "$commits_behind")

  # Build entry
  entry="### $slug ($cat_dir)
- **$commits_behind new commits** since last check (\`${local_sha:0:7}\`..\`${remote_sha:0:7}\`)
- **Classification**: $classification
- **Files changed**: $(echo "$files_changed" | wc -l | tr -d ' ') files
- **Stats**: $stat_output"

  if [[ -n "$new_tags" ]]; then
    entry="$entry
- **New tags**: $(echo "$new_tags" | tr '\n' ', ')"
  fi

  entry="$entry

<details>
<summary>Commits</summary>

\`\`\`
$log_output
\`\`\`

</details>

<details>
<summary>Changed files</summary>

\`\`\`
$files_changed
\`\`\`

</details>
"

  case "$urgency" in
    critical)
      critical_count=$((critical_count + 1))
      critical_entries+=("$entry")
      ;;
    high)
      high_count=$((high_count + 1))
      high_entries+=("$entry")
      ;;
    *)
      normal_count=$((normal_count + 1))
      normal_entries+=("$entry")
      ;;
  esac

  # Update state
  tmp=$(mktemp)
  jq --arg s "$slug" \
     --arg d "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
     --arg sha "$remote_sha" \
     --arg cls "$classification" \
     --arg urg "$urgency" \
     --argjson behind "$commits_behind" \
    '.[$s] = {
      "last_checked": $d,
      "local_sha": .[$s].local_sha,
      "remote_sha": $sha,
      "commits_behind": $behind,
      "classification": $cls,
      "urgency": $urg
    }' "$STATE_FILE" > "$tmp" && mv "$tmp" "$STATE_FILE"

done < <(jq -r '.repos[] | "\(.slug)\t\(.category)"' "$DATA")

# --- Write report ---
{
  echo "- Total repos checked: $total_repos"
  echo "- Repos with changes: $changed_repos"
  echo "- Critical: $critical_count, High: $high_count, Normal: $normal_count"
  echo ""

  if [[ $critical_count -gt 0 ]]; then
    echo "## Critical Changes"
    echo ""
    for entry in "${critical_entries[@]}"; do
      echo "$entry"
    done
  fi

  if [[ $high_count -gt 0 ]]; then
    echo "## High-Priority Changes"
    echo ""
    for entry in "${high_entries[@]}"; do
      echo "$entry"
    done
  fi

  if [[ $normal_count -gt 0 ]]; then
    echo "## Normal Changes"
    echo ""
    for entry in "${normal_entries[@]}"; do
      echo "$entry"
    done
  fi

  if [[ $changed_repos -eq 0 ]]; then
    echo "## No Changes Detected"
    echo ""
    echo "All $total_repos repos are up to date."
  fi

  echo ""
  echo "---"
  echo "*Generated by update-ecosystem.sh on $(date -u +%Y-%m-%dT%H:%M:%SZ)*"
} >> "$REPORT_FILE"

echo ""
echo "Report written to $REPORT_FILE"
echo "Summary: $total_repos checked, $changed_repos changed ($critical_count critical, $high_count high, $normal_count normal)"

# --- Create GitHub issue (unless dry run) ---
if [[ "$DRY_RUN" == false && $changed_repos -gt 0 ]]; then
  ISSUE_TITLE="Ecosystem Diff Report -- $(date +%Y-%m-%d) ($changed_repos repos changed)"
  ISSUE_LABELS="ecosystem"
  if [[ $critical_count -gt 0 ]]; then
    ISSUE_LABELS="ecosystem,ecosystem:critical"
  elif [[ $high_count -gt 0 ]]; then
    ISSUE_LABELS="ecosystem,ecosystem:high"
  fi

  if command -v gh &>/dev/null; then
    gh issue create \
      --title "$ISSUE_TITLE" \
      --body-file "$REPORT_FILE" \
      --label "$ISSUE_LABELS" \
      2>/dev/null || echo "WARN: Could not create GitHub issue (gh may not be authenticated)"
  else
    echo "WARN: gh not found -- issue not created. Install GitHub CLI to enable."
  fi
fi
```

---

### import-skills.sh

Scans skill repos for new or updated skill definition files, validates them, generates SQL INSERT statements, and creates a PR.

```bash
#!/usr/bin/env bash
# import-skills.sh -- Scan ecosystem for new skills, validate, generate SQL, create PR.
#
# Usage:
#   ./scripts/import-skills.sh                    # Full run
#   ./scripts/import-skills.sh --dry-run          # Preview only, no PR
#   ./scripts/import-skills.sh --repo <slug>      # Only scan specific repo
#
# Prerequisites: git, jq, gh, python3

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA="$SCRIPT_DIR/reference_repos.json"
ECOSYSTEM_DIR="$REPO_ROOT/ecosystem"
STATE_DIR="$REPO_ROOT/ecosystem-state"
IMPORT_DIR="$STATE_DIR/imports"
SEED_SQL="$REPO_ROOT/forge-config/seed-skills.sql"

DRY_RUN=false
TARGET_REPO=""
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    --repo) shift; TARGET_REPO="$1" ;;
  esac
done

mkdir -p "$IMPORT_DIR"

# Category directories that contain skill repos
SKILL_CATEGORIES=(
  "04-templates-skills-plugins"
  "09-config-settings"
  "10-curated-guides"
)

# Track what we import
imported=0
skipped=0
errors=0
import_log="$IMPORT_DIR/import-$(date +%Y-%m-%d).log"
sql_output="$IMPORT_DIR/import-$(date +%Y-%m-%d).sql"

> "$import_log"
> "$sql_output"

echo "-- Skills import generated on $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$sql_output"
echo "-- Source: ecosystem submodules" >> "$sql_output"
echo "" >> "$sql_output"

# --- Scan for SKILL.md files ---
scan_repo() {
  local slug="$1"
  local repo_path="$2"

  if [[ ! -d "$repo_path" ]]; then
    return
  fi

  # Find all SKILL.md and *.skill.json files
  while IFS= read -r skill_file; do
    [[ -z "$skill_file" ]] && continue

    local skill_dir
    skill_dir=$(dirname "$skill_file")
    local skill_name
    skill_name=$(basename "$skill_dir")

    echo "  Found skill: $skill_name in $slug" | tee -a "$import_log"

    # Extract metadata from SKILL.md
    local name="" description="" commands_json="[]" tags_json="[]"

    if [[ "$skill_file" == *.skill.json ]]; then
      # JSON skill definition
      if ! jq empty "$skill_file" 2>/dev/null; then
        echo "    ERROR: Invalid JSON in $skill_file" | tee -a "$import_log"
        errors=$((errors + 1))
        continue
      fi
      name=$(jq -r '.name // ""' "$skill_file")
      description=$(jq -r '.description // ""' "$skill_file")
      commands_json=$(jq -c '.commands // []' "$skill_file")
      tags_json=$(jq -c '.tags // []' "$skill_file")
    else
      # SKILL.md -- extract from markdown front matter or headings
      name="$skill_name"
      description=$(head -5 "$skill_file" | grep -v '^#' | grep -v '^$' | head -1 || echo "Skill from $slug")

      # Look for companion config.json
      if [[ -f "$skill_dir/config.json" ]]; then
        commands_json=$(jq -c '.commands // []' "$skill_dir/config.json" 2>/dev/null || echo '[]')
        tags_json=$(jq -c '.tags // []' "$skill_dir/config.json" 2>/dev/null || echo '[]')
      fi

      # Look for slash command files (.claude/commands/*.md)
      if [[ -d "$skill_dir/.claude/commands" ]]; then
        local cmds="["
        local first=true
        for cmd_file in "$skill_dir/.claude/commands"/*.md; do
          [[ -f "$cmd_file" ]] || continue
          local cmd_name
          cmd_name=$(basename "$cmd_file" .md)
          local cmd_content
          cmd_content=$(cat "$cmd_file" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))" 2>/dev/null || echo '""')
          if [[ "$first" == true ]]; then
            first=false
          else
            cmds="$cmds,"
          fi
          cmds="$cmds{\"name\":\"/$cmd_name\",\"content\":$cmd_content}"
        done
        cmds="$cmds]"
        commands_json="$cmds"
      fi
    fi

    # Validate required fields
    if [[ -z "$name" || "$name" == "null" ]]; then
      echo "    SKIP: No name found" | tee -a "$import_log"
      skipped=$((skipped + 1))
      continue
    fi

    if [[ -z "$description" || "$description" == "null" ]]; then
      description="Imported skill from $slug"
    fi

    # Check for duplicates (by name) in existing seed SQL
    if [[ -f "$SEED_SQL" ]] && grep -qF "'$name'" "$SEED_SQL"; then
      echo "    SKIP: '$name' already exists in seed-skills.sql" | tee -a "$import_log"
      skipped=$((skipped + 1))
      continue
    fi

    # Get source version (commit hash)
    local source_version
    source_version=$(git -C "$repo_path" rev-parse --short HEAD 2>/dev/null || echo "unknown")

    # Escape single quotes for SQL
    local safe_name safe_desc safe_commands safe_tags
    safe_name=$(echo "$name" | sed "s/'/''/g")
    safe_desc=$(echo "$description" | sed "s/'/''/g")
    safe_commands=$(echo "$commands_json" | sed "s/'/''/g")
    safe_tags=$(echo "$tags_json" | sed "s/'/''/g")

    # Generate SQL
    cat >> "$sql_output" << EOSQL
INSERT OR IGNORE INTO skills (name, description, commands_json, tags_json, source_repo, source_version, imported_at)
VALUES ('$safe_name', '$safe_desc', '$safe_commands', '$safe_tags', '$slug', '$source_version', datetime('now'));

EOSQL

    imported=$((imported + 1))
    echo "    OK: Queued for import" | tee -a "$import_log"

  done < <(find "$repo_path" -maxdepth 4 \( -name "SKILL.md" -o -name "*.skill.json" \) -type f 2>/dev/null)
}

# --- Scan repos ---
echo "Scanning ecosystem for skills..."

if [[ -n "$TARGET_REPO" ]]; then
  # Scan specific repo
  for cat_dir in "${SKILL_CATEGORIES[@]}"; do
    repo_path="$ECOSYSTEM_DIR/$cat_dir/$TARGET_REPO"
    if [[ -d "$repo_path" ]]; then
      echo "Scanning $TARGET_REPO..."
      scan_repo "$TARGET_REPO" "$repo_path"
      break
    fi
  done
else
  # Scan all skill categories
  for cat_dir in "${SKILL_CATEGORIES[@]}"; do
    dir="$ECOSYSTEM_DIR/$cat_dir"
    [[ -d "$dir" ]] || continue
    for repo_path in "$dir"/*/; do
      [[ -d "$repo_path" ]] || continue
      slug=$(basename "$repo_path")
      echo "Scanning $slug..."
      scan_repo "$slug" "$repo_path"
    done
  done
fi

echo ""
echo "Results: $imported imported, $skipped skipped, $errors errors"
echo "SQL output: $sql_output"
echo "Log: $import_log"

# --- Create PR (unless dry run) ---
if [[ "$DRY_RUN" == false && $imported -gt 0 ]]; then
  BRANCH="auto-import/skills-$(date +%Y-%m-%d)"

  # Append to seed SQL
  if [[ -f "$SEED_SQL" ]]; then
    echo "" >> "$SEED_SQL"
    cat "$sql_output" >> "$SEED_SQL"
  else
    mkdir -p "$(dirname "$SEED_SQL")"
    cp "$sql_output" "$SEED_SQL"
  fi

  git checkout -b "$BRANCH"
  git add "$SEED_SQL" "$import_log" "$sql_output"
  git commit -m "auto-import(skills): $imported new skills from ecosystem

Imported $imported skills, skipped $skipped (duplicates or invalid), $errors errors.
See ecosystem-state/imports/ for details."

  if command -v gh &>/dev/null; then
    git push -u origin "$BRANCH"
    gh pr create \
      --title "auto-import(skills): $imported new skills from ecosystem" \
      --body "## Auto-Import: Skills

- **Imported**: $imported new skills
- **Skipped**: $skipped (duplicates or missing required fields)
- **Errors**: $errors

### Source Repos
$(grep 'Found skill:' "$import_log" | sort -u)

### SQL Preview
\`\`\`sql
$(head -50 "$sql_output")
\`\`\`

---
*Generated by import-skills.sh*" \
      --label "ecosystem,auto-import" \
      2>/dev/null || echo "WARN: Could not create PR"
  else
    echo "WARN: gh not found -- PR not created"
  fi
fi
```

---

### check-claude-code.sh

Checks for new Claude Code releases, compares capabilities, and reports compatibility status.

```bash
#!/usr/bin/env bash
# check-claude-code.sh -- Check Claude Code for new releases, compare capabilities, report.
#
# Usage:
#   ./scripts/check-claude-code.sh                 # Full check
#   ./scripts/check-claude-code.sh --version-only  # Only report version
#
# Prerequisites: claude CLI installed, git, jq

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STATE_DIR="$REPO_ROOT/ecosystem-state"
COMPAT_FILE="$REPO_ROOT/forge-config/compatibility.json"
SYSTEM_PROMPTS_DIR="$REPO_ROOT/ecosystem/11-docs-internals/claude-code-system-prompts"
REPORT_FILE="$STATE_DIR/reports/claude-code-compat-$(date +%Y-%m-%d).md"

VERSION_ONLY=false
for arg in "$@"; do
  [[ "$arg" == "--version-only" ]] && VERSION_ONLY=true
done

mkdir -p "$STATE_DIR/reports" "$STATE_DIR/claude-code"

# --- 1. Check current Claude Code version ---
echo "Checking Claude Code version..."
CURRENT_VERSION="unknown"
if command -v claude &>/dev/null; then
  CURRENT_VERSION=$(claude --version 2>/dev/null | head -1 || echo "unknown")
fi

# Load last known version
LAST_VERSION_FILE="$STATE_DIR/claude-code/last-version.txt"
LAST_VERSION=""
if [[ -f "$LAST_VERSION_FILE" ]]; then
  LAST_VERSION=$(cat "$LAST_VERSION_FILE")
fi

echo "Current: $CURRENT_VERSION"
echo "Last known: ${LAST_VERSION:-none}"

VERSION_CHANGED=false
if [[ "$CURRENT_VERSION" != "$LAST_VERSION" && "$CURRENT_VERSION" != "unknown" ]]; then
  VERSION_CHANGED=true
  echo "$CURRENT_VERSION" > "$LAST_VERSION_FILE"
  echo "VERSION CHANGED: $LAST_VERSION -> $CURRENT_VERSION"
fi

if [[ "$VERSION_ONLY" == true ]]; then
  exit 0
fi

# --- 2. Parse CLI flags ---
echo "Parsing Claude Code CLI flags..."
CLI_FLAGS_FILE="$STATE_DIR/claude-code/cli-flags.txt"
LAST_FLAGS_FILE="$STATE_DIR/claude-code/cli-flags-previous.txt"

if [[ -f "$CLI_FLAGS_FILE" ]]; then
  cp "$CLI_FLAGS_FILE" "$LAST_FLAGS_FILE"
fi

if command -v claude &>/dev/null; then
  claude --help 2>/dev/null | grep -E '^\s+(-|--)\S+' | sed 's/^[[:space:]]*//' > "$CLI_FLAGS_FILE" || true
fi

NEW_FLAGS=""
if [[ -f "$LAST_FLAGS_FILE" && -f "$CLI_FLAGS_FILE" ]]; then
  NEW_FLAGS=$(diff "$LAST_FLAGS_FILE" "$CLI_FLAGS_FILE" | grep '^>' | sed 's/^> //' || true)
fi

# --- 3. Check system prompts repo ---
echo "Checking system prompts..."
SYSTEM_PROMPT_CHANGES=""
if [[ -d "$SYSTEM_PROMPTS_DIR/.git" ]] || [[ -f "$SYSTEM_PROMPTS_DIR/.git" ]]; then
  git -C "$SYSTEM_PROMPTS_DIR" fetch upstream --quiet 2>/dev/null || \
    git -C "$SYSTEM_PROMPTS_DIR" fetch origin --quiet 2>/dev/null || true

  LOCAL_SHA=$(git -C "$SYSTEM_PROMPTS_DIR" rev-parse HEAD 2>/dev/null || echo "unknown")

  REMOTE_REF=""
  for branch in main master; do
    for remote in upstream origin; do
      if git -C "$SYSTEM_PROMPTS_DIR" rev-parse "$remote/$branch" &>/dev/null; then
        REMOTE_REF="$remote/$branch"
        break 2
      fi
    done
  done

  if [[ -n "$REMOTE_REF" ]]; then
    REMOTE_SHA=$(git -C "$SYSTEM_PROMPTS_DIR" rev-parse "$REMOTE_REF" 2>/dev/null || echo "unknown")
    if [[ "$LOCAL_SHA" != "$REMOTE_SHA" ]]; then
      SYSTEM_PROMPT_CHANGES=$(git -C "$SYSTEM_PROMPTS_DIR" log HEAD.."$REMOTE_REF" --oneline 2>/dev/null || echo "")
    fi
  fi
fi

# --- 4. Extract hook types from system prompts ---
echo "Extracting hook types..."
HOOK_TYPES_FILE="$STATE_DIR/claude-code/hook-types.txt"
LAST_HOOKS_FILE="$STATE_DIR/claude-code/hook-types-previous.txt"

if [[ -f "$HOOK_TYPES_FILE" ]]; then
  cp "$HOOK_TYPES_FILE" "$LAST_HOOKS_FILE"
fi

# Scan system prompts for hook event names
if [[ -d "$SYSTEM_PROMPTS_DIR" ]]; then
  grep -rhoE '(PreToolUse|PostToolUse|Notification|Stop|SubagentSpawn|Pre[A-Z][a-zA-Z]+|Post[A-Z][a-zA-Z]+)\b' \
    "$SYSTEM_PROMPTS_DIR" 2>/dev/null | sort -u > "$HOOK_TYPES_FILE" || true
fi

NEW_HOOKS=""
if [[ -f "$LAST_HOOKS_FILE" && -f "$HOOK_TYPES_FILE" ]]; then
  NEW_HOOKS=$(diff "$LAST_HOOKS_FILE" "$HOOK_TYPES_FILE" | grep '^>' | sed 's/^> //' || true)
fi

# --- 5. Extract tool types ---
echo "Extracting tool types..."
TOOL_TYPES_FILE="$STATE_DIR/claude-code/tool-types.txt"
LAST_TOOLS_FILE="$STATE_DIR/claude-code/tool-types-previous.txt"

if [[ -f "$TOOL_TYPES_FILE" ]]; then
  cp "$TOOL_TYPES_FILE" "$LAST_TOOLS_FILE"
fi

if [[ -d "$SYSTEM_PROMPTS_DIR" ]]; then
  grep -rhoE '(Read|Write|Edit|Bash|Glob|Grep|WebFetch|WebSearch|NotebookEdit|TodoRead|TodoWrite|Task|Skill|EnterWorktree|mcp__[a-zA-Z0-9_]+)\b' \
    "$SYSTEM_PROMPTS_DIR" 2>/dev/null | sort -u > "$TOOL_TYPES_FILE" || true
fi

NEW_TOOLS=""
if [[ -f "$LAST_TOOLS_FILE" && -f "$TOOL_TYPES_FILE" ]]; then
  NEW_TOOLS=$(diff "$LAST_TOOLS_FILE" "$TOOL_TYPES_FILE" | grep '^>' | sed 's/^> //' || true)
fi

# --- 6. Generate report ---
echo "Generating compatibility report..."

{
  echo "# Claude Code Compatibility Report -- $(date +%Y-%m-%d)"
  echo ""

  if [[ "$VERSION_CHANGED" == true ]]; then
    echo "## VERSION CHANGE DETECTED"
    echo ""
    echo "- Previous: \`$LAST_VERSION\`"
    echo "- Current: \`$CURRENT_VERSION\`"
    echo ""
  else
    echo "## Version"
    echo ""
    echo "- Current: \`$CURRENT_VERSION\` (unchanged)"
    echo ""
  fi

  echo "## CLI Flags"
  echo ""
  if [[ -n "$NEW_FLAGS" ]]; then
    echo "### NEW FLAGS DETECTED"
    echo ""
    echo '```'
    echo "$NEW_FLAGS"
    echo '```'
    echo ""
    echo "**Action required**: Evaluate each new flag for inclusion in Forge's agent spawn configuration."
    echo ""
  else
    echo "No new CLI flags detected."
    echo ""
  fi

  echo "## Hook Event Types"
  echo ""
  if [[ -n "$NEW_HOOKS" ]]; then
    echo "### NEW HOOK TYPES DETECTED"
    echo ""
    echo '```'
    echo "$NEW_HOOKS"
    echo '```'
    echo ""
    echo "**Action required**: Add to \`HookEvent\` enum in \`src/hooks.rs\`, update hooks editor UI, add to observability stream."
    echo ""
  else
    echo "No new hook types detected."
    echo ""
  fi
  echo "Known hook types:"
  echo ""
  if [[ -f "$HOOK_TYPES_FILE" ]]; then
    echo '```'
    cat "$HOOK_TYPES_FILE"
    echo '```'
  fi
  echo ""

  echo "## Tool Types"
  echo ""
  if [[ -n "$NEW_TOOLS" ]]; then
    echo "### NEW TOOL TYPES DETECTED"
    echo ""
    echo '```'
    echo "$NEW_TOOLS"
    echo '```'
    echo ""
    echo "**Action required**: Update tool permission management in Forge's agent configuration."
    echo ""
  else
    echo "No new tool types detected."
    echo ""
  fi
  echo "Known tool types:"
  echo ""
  if [[ -f "$TOOL_TYPES_FILE" ]]; then
    echo '```'
    cat "$TOOL_TYPES_FILE"
    echo '```'
  fi
  echo ""

  echo "## System Prompts"
  echo ""
  if [[ -n "$SYSTEM_PROMPT_CHANGES" ]]; then
    echo "### SYSTEM PROMPT CHANGES DETECTED"
    echo ""
    echo '```'
    echo "$SYSTEM_PROMPT_CHANGES"
    echo '```'
    echo ""
    echo "**Action required**: Review changes for impact on Forge's agent behavior and prompt engineering."
    echo ""
  else
    echo "No system prompt changes detected."
    echo ""
  fi

  echo "## Compatibility Status"
  echo ""
  if [[ -f "$COMPAT_FILE" ]]; then
    local_forge_version=$(jq -r '.forge_version // "unknown"' "$COMPAT_FILE")
    min_cc=$(jq -r '.claude_code.min_version // "unknown"' "$COMPAT_FILE")
    echo "- Forge version: \`$local_forge_version\`"
    echo "- Required Claude Code: >= \`$min_cc\`"
    echo "- Installed Claude Code: \`$CURRENT_VERSION\`"
    echo ""

    # Simple version comparison (assumes semver)
    echo "**Status**: Manual verification recommended after any version change."
  else
    echo "No compatibility.json found. Create one at \`forge-config/compatibility.json\`."
  fi

  echo ""
  echo "---"
  echo "*Generated by check-claude-code.sh on $(date -u +%Y-%m-%dT%H:%M:%SZ)*"
} > "$REPORT_FILE"

echo ""
echo "Report written to $REPORT_FILE"

# --- 7. Create critical issue if version changed ---
if [[ "$VERSION_CHANGED" == true ]]; then
  echo "CRITICAL: Claude Code version changed. Creating issue..."
  if command -v gh &>/dev/null; then
    gh issue create \
      --title "Claude Code updated: $LAST_VERSION -> $CURRENT_VERSION" \
      --body-file "$REPORT_FILE" \
      --label "ecosystem,ecosystem:critical,claude-code" \
      2>/dev/null || echo "WARN: Could not create GitHub issue"
  fi
fi
```

---

### ecosystem-report.sh

Generates a full ecosystem health report combining submodule status, absorption progress, and repository metadata.

```bash
#!/usr/bin/env bash
# ecosystem-report.sh -- Generate full ecosystem health report.
#
# Usage:
#   ./scripts/ecosystem-report.sh                  # Generate report
#   ./scripts/ecosystem-report.sh --json           # Also output JSON
#   ./scripts/ecosystem-report.sh --github-api     # Include GitHub API data (stars, issues)
#
# Prerequisites: git, jq. Optional: gh (for GitHub API data)

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA="$SCRIPT_DIR/reference_repos.json"
ECOSYSTEM_DIR="$REPO_ROOT/ecosystem"
STATE_DIR="$REPO_ROOT/ecosystem-state"
ABSORPTION_FILE="$STATE_DIR/absorption-status.json"
STATE_FILE="$STATE_DIR/ecosystem-state.json"
REPORT_FILE="$STATE_DIR/reports/ecosystem-health-$(date +%Y-%m-%d).md"
JSON_OUTPUT=false
GITHUB_API=false

if [[ -f "$SCRIPT_DIR/remotes-config.env" ]]; then
  set -a; source "$SCRIPT_DIR/remotes-config.env"; set +a
fi
REMOTE_UPSTREAM="${REMOTE_UPSTREAM:-upstream}"

for arg in "$@"; do
  case "$arg" in
    --json) JSON_OUTPUT=true ;;
    --github-api) GITHUB_API=true ;;
  esac
done

mkdir -p "$STATE_DIR/reports"

# Category directory mapping
declare -A CATEGORY_DIRS
CATEGORY_DIRS=(
  ["Desktop & IDEs"]="01-desktop-ides"
  ["Orchestration & Workflows"]="02-orchestration"
  ["Hooks & Observability"]="03-hooks-observability"
  ["Templates, Skills & Plugins"]="04-templates-skills-plugins"
  ["Subagents & Agents"]="05-subagents-agents"
  ["MCP & Tooling"]="06-mcp-tooling"
  ["Remote & Infra"]="07-remote-infra"
  ["Automation & CI/CD"]="08-automation-cicd"
  ["Config & Settings"]="09-config-settings"
  ["Curated Lists & Guides"]="10-curated-guides"
  ["Docs & System Internals"]="11-docs-internals"
  ["Prompts & Learning"]="12-prompts-learning"
  ["Transcripts, Security & Misc"]="13-transcripts-security-misc"
)

# Initialize counters
total=0
green=0
yellow=0
red=0
unreachable=0
total_absorption=0
absorption_count=0

# JSON array for machine-readable output
json_repos="[]"

# Start report
{
  echo "# Ecosystem Health Report -- $(date +%Y-%m-%d)"
  echo ""
  echo "## Overview"
  echo ""
} > "$REPORT_FILE"

# Collect per-category data
declare -A category_tables

while IFS=$'\t' read -r slug category description; do
  cat_dir="${CATEGORY_DIRS[$category]:-unknown}"
  submodule_path="$ECOSYSTEM_DIR/$cat_dir/$slug"
  total=$((total + 1))

  # Default values
  last_commit="N/A"
  commits_behind="?"
  absorption_pct="--"
  status="GREY"
  stars="--"

  # Check submodule exists
  if [[ -d "$submodule_path/.git" ]] || [[ -f "$submodule_path/.git" ]]; then
    # Last commit date
    last_commit=$(git -C "$submodule_path" log -1 --format=%cd --date=short 2>/dev/null || echo "N/A")

    # Commits behind upstream
    for branch in main master; do
      if git -C "$submodule_path" rev-parse "$REMOTE_UPSTREAM/$branch" &>/dev/null 2>&1; then
        commits_behind=$(git -C "$submodule_path" rev-list HEAD.."$REMOTE_UPSTREAM/$branch" --count 2>/dev/null || echo "?")
        break
      fi
    done

    # Status color
    if [[ "$commits_behind" == "?" ]]; then
      status="GREY"
      unreachable=$((unreachable + 1))
    elif [[ "$commits_behind" -eq 0 ]]; then
      status="GREEN"
      green=$((green + 1))
    elif [[ "$commits_behind" -le 10 ]]; then
      status="GREEN"
      green=$((green + 1))
    elif [[ "$commits_behind" -le 20 ]]; then
      status="YELLOW"
      yellow=$((yellow + 1))
    else
      status="RED"
      red=$((red + 1))
    fi
  else
    status="GREY"
    unreachable=$((unreachable + 1))
  fi

  # Absorption status
  if [[ -f "$ABSORPTION_FILE" ]]; then
    absorption_pct=$(jq -r --arg s "$slug" '.[$s].absorption_percentage // "--"' "$ABSORPTION_FILE" 2>/dev/null || echo "--")
    if [[ "$absorption_pct" != "--" ]]; then
      total_absorption=$((total_absorption + absorption_pct))
      absorption_count=$((absorption_count + 1))
    fi
  fi

  # GitHub API data (stars)
  if [[ "$GITHUB_API" == true ]]; then
    github_url=$(jq -r --arg s "$slug" '.repos[] | select(.slug == $s) | .github_url // ""' "$DATA")
    if [[ -n "$github_url" ]] && command -v gh &>/dev/null; then
      owner_repo="${github_url#https://github.com/}"
      owner_repo="${owner_repo%.git}"
      stars=$(gh api "repos/$owner_repo" --jq '.stargazers_count' 2>/dev/null || echo "--")
    fi
  fi

  # Build table row
  row="| $slug | $last_commit | $commits_behind | $absorption_pct | $status |"
  category_tables["$cat_dir"]="${category_tables[$cat_dir]:-}
$row"

  # JSON output
  if [[ "$JSON_OUTPUT" == true ]]; then
    json_repos=$(echo "$json_repos" | jq --arg s "$slug" --arg c "$cat_dir" \
      --arg lc "$last_commit" --arg cb "$commits_behind" \
      --arg ap "$absorption_pct" --arg st "$status" --arg sr "$stars" \
      '. + [{"slug":$s,"category":$c,"last_commit":$lc,"commits_behind":$cb,"absorption":$ap,"status":$st,"stars":$sr}]')
  fi

done < <(jq -r '.repos[] | "\(.slug)\t\(.category)\t\(.description)"' "$DATA")

# Calculate averages
avg_absorption=0
if [[ $absorption_count -gt 0 ]]; then
  avg_absorption=$((total_absorption / absorption_count))
fi

# Write overview
{
  echo "- **Total repos tracked**: $total"
  echo "- **Status**: $green GREEN, $yellow YELLOW, $red RED, $unreachable UNREACHABLE"
  echo "- **Average absorption**: ${avg_absorption}%"
  echo ""
} >> "$REPORT_FILE"

# Write per-category tables
{
  echo "## Status by Category"
  echo ""

  for cat_dir in $(echo "${!category_tables[@]}" | tr ' ' '\n' | sort); do
    echo "### $cat_dir"
    echo ""
    echo "| Repo | Last Commit | Behind | Absorption | Status |"
    echo "|------|-------------|--------|------------|--------|"
    echo "${category_tables[$cat_dir]}"
    echo ""
  done
} >> "$REPORT_FILE"

# Repos needing attention
{
  echo "## Repos Needing Attention"
  echo ""

  # List RED repos
  while IFS=$'\t' read -r slug category; do
    cat_dir="${CATEGORY_DIRS[$category]:-unknown}"
    submodule_path="$ECOSYSTEM_DIR/$cat_dir/$slug"
    if [[ -d "$submodule_path/.git" ]] || [[ -f "$submodule_path/.git" ]]; then
      for branch in main master; do
        if git -C "$submodule_path" rev-parse "$REMOTE_UPSTREAM/$branch" &>/dev/null 2>&1; then
          behind=$(git -C "$submodule_path" rev-list HEAD.."$REMOTE_UPSTREAM/$branch" --count 2>/dev/null || echo "0")
          if [[ "$behind" -gt 20 ]]; then
            last_checked=$(jq -r --arg s "$slug" '.[$s].last_checked // "never"' "$STATE_FILE" 2>/dev/null || echo "never")
            echo "- **$slug** -- $behind commits behind, last checked: $last_checked"
          fi
          break
        fi
      done
    fi
  done < <(jq -r '.repos[] | "\(.slug)\t\(.category)"' "$DATA")

  echo ""
  echo "---"
  echo "*Generated by ecosystem-report.sh on $(date -u +%Y-%m-%dT%H:%M:%SZ)*"
} >> "$REPORT_FILE"

echo "Report written to $REPORT_FILE"

# JSON output
if [[ "$JSON_OUTPUT" == true ]]; then
  JSON_FILE="$STATE_DIR/reports/ecosystem-health-$(date +%Y-%m-%d).json"
  echo "$json_repos" | jq '.' > "$JSON_FILE"
  echo "JSON written to $JSON_FILE"
fi
```

---

## 11. GitHub Actions Workflows

### ecosystem-monitor.yml

Runs weekly (Sunday midnight UTC) or on-demand. Fetches all submodules, detects changes, and creates an issue if anything significant changed.

```yaml
# .github/workflows/ecosystem-monitor.yml
name: Ecosystem Monitor

on:
  schedule:
    # Every Sunday at 00:00 UTC
    - cron: '0 0 * * 0'
  workflow_dispatch:
    inputs:
      category:
        description: 'Category filter (e.g., 04 for templates-skills-plugins)'
        required: false
        default: ''

permissions:
  issues: write
  contents: read

jobs:
  monitor:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - name: Checkout with submodules
        uses: actions/checkout@v4
        with:
          submodules: recursive
          fetch-depth: 0

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y jq

      - name: Configure git
        run: |
          git config --global user.name "Ecosystem Monitor"
          git config --global user.email "forge-ecosystem@users.noreply.github.com"

      - name: Fetch all upstreams
        run: |
          # For each submodule, add upstream remote if not present and fetch
          git submodule foreach --recursive '
            UPSTREAM_URL=$(git config --file $toplevel/.gitmodules --get submodule.$name.url 2>/dev/null || echo "")
            if [ -n "$UPSTREAM_URL" ]; then
              git remote add upstream "$UPSTREAM_URL" 2>/dev/null || true
              git fetch upstream --tags --quiet 2>/dev/null || echo "WARN: Could not fetch upstream for $name"
            fi
          '

      - name: Run ecosystem update
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          ARGS=""
          if [ -n "${{ github.event.inputs.category }}" ]; then
            ARGS="--category ${{ github.event.inputs.category }}"
          fi
          chmod +x scripts/update-ecosystem.sh
          ./scripts/update-ecosystem.sh $ARGS

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: ecosystem-diff-report
          path: ecosystem-state/reports/ecosystem-diff-*.md
          retention-days: 90

      - name: Commit state updates
        run: |
          git add ecosystem-state/ecosystem-state.json
          if git diff --cached --quiet; then
            echo "No state changes to commit"
          else
            git commit -m "chore(ecosystem): update state after weekly scan"
            git push
          fi
```

### auto-import.yml

Triggered when ecosystem-monitor detects data-type changes, or manually for on-demand skill imports.

```yaml
# .github/workflows/auto-import.yml
name: Auto-Import Skills

on:
  workflow_dispatch:
    inputs:
      repo:
        description: 'Specific repo slug to import from (leave empty for all)'
        required: false
        default: ''
      dry_run:
        description: 'Dry run (preview only, no PR)'
        required: false
        default: 'false'
        type: boolean
  workflow_run:
    workflows: ["Ecosystem Monitor"]
    types: [completed]
    branches: [main]

permissions:
  contents: write
  pull-requests: write

jobs:
  check-trigger:
    runs-on: ubuntu-latest
    outputs:
      should_import: ${{ steps.check.outputs.should_import }}
    steps:
      - name: Check if import needed
        id: check
        run: |
          if [ "${{ github.event_name }}" == "workflow_dispatch" ]; then
            echo "should_import=true" >> "$GITHUB_OUTPUT"
          elif [ "${{ github.event.workflow_run.conclusion }}" == "success" ]; then
            echo "should_import=true" >> "$GITHUB_OUTPUT"
          else
            echo "should_import=false" >> "$GITHUB_OUTPUT"
          fi

  import:
    needs: check-trigger
    if: needs.check-trigger.outputs.should_import == 'true'
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout with submodules
        uses: actions/checkout@v4
        with:
          submodules: recursive
          fetch-depth: 0

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y jq python3

      - name: Configure git
        run: |
          git config --global user.name "Forge Auto-Import"
          git config --global user.email "forge-import@users.noreply.github.com"

      - name: Update submodules to latest
        run: |
          git submodule update --remote --merge

      - name: Run skill import
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          ARGS=""
          if [ "${{ github.event.inputs.dry_run }}" == "true" ]; then
            ARGS="--dry-run"
          fi
          if [ -n "${{ github.event.inputs.repo }}" ]; then
            ARGS="$ARGS --repo ${{ github.event.inputs.repo }}"
          fi
          chmod +x scripts/import-skills.sh
          ./scripts/import-skills.sh $ARGS

      - name: Upload import artifacts
        uses: actions/upload-artifact@v4
        with:
          name: import-results
          path: |
            ecosystem-state/imports/*.sql
            ecosystem-state/imports/*.log
          retention-days: 30
```

### claude-code-compat.yml

Runs weekly and on-demand to check Claude Code compatibility.

```yaml
# .github/workflows/claude-code-compat.yml
name: Claude Code Compatibility Check

on:
  schedule:
    # Every Wednesday at 06:00 UTC (mid-week check)
    - cron: '0 6 * * 3'
  workflow_dispatch:

permissions:
  issues: write
  contents: read

jobs:
  compat-check:
    runs-on: ubuntu-latest
    timeout-minutes: 10

    steps:
      - name: Checkout with submodules
        uses: actions/checkout@v4
        with:
          submodules: recursive
          fetch-depth: 0

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y jq

      - name: Install Claude Code
        run: |
          # Install Claude Code CLI (adjust based on actual installation method)
          npm install -g @anthropic-ai/claude-code || echo "Claude Code not installable in CI -- using submodule data only"

      - name: Fetch system prompts upstream
        run: |
          PROMPTS_DIR="ecosystem/11-docs-internals/claude-code-system-prompts"
          if [ -d "$PROMPTS_DIR" ]; then
            cd "$PROMPTS_DIR"
            git remote add upstream https://github.com/anthropics/claude-code-system-prompts.git 2>/dev/null || true
            git fetch upstream --quiet 2>/dev/null || echo "WARN: Could not fetch system prompts upstream"
            cd -
          fi

      - name: Run compatibility check
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          chmod +x scripts/check-claude-code.sh
          ./scripts/check-claude-code.sh

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: claude-code-compat-report
          path: ecosystem-state/reports/claude-code-compat-*.md
          retention-days: 90

      - name: Commit state updates
        run: |
          git add ecosystem-state/claude-code/
          if git diff --cached --quiet; then
            echo "No state changes"
          else
            git config user.name "Forge Compat Check"
            git config user.email "forge-compat@users.noreply.github.com"
            git commit -m "chore(ecosystem): update Claude Code compatibility state"
            git push
          fi
```

---

## 12. .gitmodules Template

The complete `.gitmodules` file for all 62 repositories. URLs use the upstream (original author) as the submodule source. After cloning, run `scripts/sync-remotes.sh --set` to add the fork remote.

> **Note**: Some repos in `reference_repos.json` do not yet have confirmed `github_url` values. Those entries below use placeholder URLs derived from the slug. Run `scripts/sync-remotes.sh` to discover and populate the actual URLs from existing clones.

```ini
# ============================================================
# Claude Forge Ecosystem Submodules
# 62 repositories organized by category
# Generated from scripts/reference_repos.json
#
# After cloning:
#   git submodule update --init --recursive
#   ./scripts/sync-remotes.sh --set
# ============================================================

# --- 01-desktop-ides (8 repos) ---

[submodule "ecosystem/01-desktop-ides/1code"]
    path = ecosystem/01-desktop-ides/1code
    url = https://github.com/anthropics/1code.git
    branch = main

[submodule "ecosystem/01-desktop-ides/claude-code-viewer"]
    path = ecosystem/01-desktop-ides/claude-code-viewer
    url = https://github.com/nicholasgriffintn/claude-code-viewer.git
    branch = main

[submodule "ecosystem/01-desktop-ides/idea-claude-code-gui"]
    path = ecosystem/01-desktop-ides/idea-claude-code-gui
    url = https://github.com/nicholasgriffintn/idea-claude-code-gui.git
    branch = main

[submodule "ecosystem/01-desktop-ides/CodexBar"]
    path = ecosystem/01-desktop-ides/CodexBar
    url = https://github.com/nicholasgriffintn/CodexBar.git
    branch = main

[submodule "ecosystem/01-desktop-ides/claude-code.nvim"]
    path = ecosystem/01-desktop-ides/claude-code.nvim
    url = https://github.com/greggh/claude-code.nvim.git
    branch = main

[submodule "ecosystem/01-desktop-ides/claude-code-ide.el"]
    path = ecosystem/01-desktop-ides/claude-code-ide.el
    url = https://github.com/nicholasgriffintn/claude-code-ide.el.git
    branch = main

[submodule "ecosystem/01-desktop-ides/claude-code-chat"]
    path = ecosystem/01-desktop-ides/claude-code-chat
    url = https://github.com/nicholasgriffintn/claude-code-chat.git
    branch = main

[submodule "ecosystem/01-desktop-ides/claude-code-webui"]
    path = ecosystem/01-desktop-ides/claude-code-webui
    url = https://github.com/nicholasgriffintn/claude-code-webui.git
    branch = main

# --- 02-orchestration (7 repos) ---

[submodule "ecosystem/02-orchestration/Claude-Code-Workflow"]
    path = ecosystem/02-orchestration/Claude-Code-Workflow
    url = https://github.com/catlog22/Claude-Code-Workflow.git
    branch = main

[submodule "ecosystem/02-orchestration/Claude-Code-Development-Kit"]
    path = ecosystem/02-orchestration/Claude-Code-Development-Kit
    url = https://github.com/nicholasgriffintn/Claude-Code-Development-Kit.git
    branch = main

[submodule "ecosystem/02-orchestration/claude-code-router"]
    path = ecosystem/02-orchestration/claude-code-router
    url = https://github.com/nicholasgriffintn/claude-code-router.git
    branch = main

[submodule "ecosystem/02-orchestration/claude-code-spec-workflow"]
    path = ecosystem/02-orchestration/claude-code-spec-workflow
    url = https://github.com/nicholasgriffintn/claude-code-spec-workflow.git
    branch = main

[submodule "ecosystem/02-orchestration/claude-code-workflows"]
    path = ecosystem/02-orchestration/claude-code-workflows
    url = https://github.com/nicholasgriffintn/claude-code-workflows.git
    branch = main

[submodule "ecosystem/02-orchestration/claude_code_bridge"]
    path = ecosystem/02-orchestration/claude_code_bridge
    url = https://github.com/nicholasgriffintn/claude_code_bridge.git
    branch = main

[submodule "ecosystem/02-orchestration/ralph-claude-code"]
    path = ecosystem/02-orchestration/ralph-claude-code
    url = https://github.com/nicholasgriffintn/ralph-claude-code.git
    branch = main

# --- 03-hooks-observability (3 repos) ---

[submodule "ecosystem/03-hooks-observability/Claude-Code-Usage-Monitor"]
    path = ecosystem/03-hooks-observability/Claude-Code-Usage-Monitor
    url = https://github.com/nicholasgriffintn/Claude-Code-Usage-Monitor.git
    branch = main

[submodule "ecosystem/03-hooks-observability/claude-code-hooks-mastery"]
    path = ecosystem/03-hooks-observability/claude-code-hooks-mastery
    url = https://github.com/nicholasgriffintn/claude-code-hooks-mastery.git
    branch = main

[submodule "ecosystem/03-hooks-observability/claude-code-hooks-multi-agent-observability"]
    path = ecosystem/03-hooks-observability/claude-code-hooks-multi-agent-observability
    url = https://github.com/nicholasgriffintn/claude-code-hooks-multi-agent-observability.git
    branch = main

# --- 04-templates-skills-plugins (8 repos) ---

[submodule "ecosystem/04-templates-skills-plugins/claude-code-cookbook"]
    path = ecosystem/04-templates-skills-plugins/claude-code-cookbook
    url = https://github.com/nicholasgriffintn/claude-code-cookbook.git
    branch = main

[submodule "ecosystem/04-templates-skills-plugins/claude-code-plugins-plus-skills"]
    path = ecosystem/04-templates-skills-plugins/claude-code-plugins-plus-skills
    url = https://github.com/nicholasgriffintn/claude-code-plugins-plus-skills.git
    branch = main

[submodule "ecosystem/04-templates-skills-plugins/claude-code-skill-factory"]
    path = ecosystem/04-templates-skills-plugins/claude-code-skill-factory
    url = https://github.com/nicholasgriffintn/claude-code-skill-factory.git
    branch = main

[submodule "ecosystem/04-templates-skills-plugins/claude-code-skills"]
    path = ecosystem/04-templates-skills-plugins/claude-code-skills
    url = https://github.com/nicholasgriffintn/claude-code-skills.git
    branch = main

[submodule "ecosystem/04-templates-skills-plugins/claude-code-templates"]
    path = ecosystem/04-templates-skills-plugins/claude-code-templates
    url = https://github.com/nicholasgriffintn/claude-code-templates.git
    branch = main

[submodule "ecosystem/04-templates-skills-plugins/claude-code-tresor"]
    path = ecosystem/04-templates-skills-plugins/claude-code-tresor
    url = https://github.com/nicholasgriffintn/claude-code-tresor.git
    branch = main

[submodule "ecosystem/04-templates-skills-plugins/everything-claude-code"]
    path = ecosystem/04-templates-skills-plugins/everything-claude-code
    url = https://github.com/nicholasgriffintn/everything-claude-code.git
    branch = main

[submodule "ecosystem/04-templates-skills-plugins/my-claude-code-setup"]
    path = ecosystem/04-templates-skills-plugins/my-claude-code-setup
    url = https://github.com/nicholasgriffintn/my-claude-code-setup.git
    branch = main

# --- 05-subagents-agents (3 repos) ---

[submodule "ecosystem/05-subagents-agents/ClaudeCodeAgents"]
    path = ecosystem/05-subagents-agents/ClaudeCodeAgents
    url = https://github.com/nicholasgriffintn/ClaudeCodeAgents.git
    branch = main

[submodule "ecosystem/05-subagents-agents/claude-code-sub-agents"]
    path = ecosystem/05-subagents-agents/claude-code-sub-agents
    url = https://github.com/nicholasgriffintn/claude-code-sub-agents.git
    branch = main

[submodule "ecosystem/05-subagents-agents/claude-code-subagents"]
    path = ecosystem/05-subagents-agents/claude-code-subagents
    url = https://github.com/nicholasgriffintn/claude-code-subagents.git
    branch = main

# --- 06-mcp-tooling (3 repos) ---

[submodule "ecosystem/06-mcp-tooling/claude-code-mcp"]
    path = ecosystem/06-mcp-tooling/claude-code-mcp
    url = https://github.com/nicholasgriffintn/claude-code-mcp.git
    branch = main

[submodule "ecosystem/06-mcp-tooling/codemcp"]
    path = ecosystem/06-mcp-tooling/codemcp
    url = https://github.com/nicholasgriffintn/codemcp.git
    branch = main

[submodule "ecosystem/06-mcp-tooling/claude-code-tools"]
    path = ecosystem/06-mcp-tooling/claude-code-tools
    url = https://github.com/nicholasgriffintn/claude-code-tools.git
    branch = main

# --- 07-remote-infra (4 repos) ---

[submodule "ecosystem/07-remote-infra/Claude-Code-Remote"]
    path = ecosystem/07-remote-infra/Claude-Code-Remote
    url = https://github.com/nicholasgriffintn/Claude-Code-Remote.git
    branch = main

[submodule "ecosystem/07-remote-infra/claude-code-hub"]
    path = ecosystem/07-remote-infra/claude-code-hub
    url = https://github.com/nicholasgriffintn/claude-code-hub.git
    branch = main

[submodule "ecosystem/07-remote-infra/claude-code-proxy"]
    path = ecosystem/07-remote-infra/claude-code-proxy
    url = https://github.com/nicholasgriffintn/claude-code-proxy.git
    branch = main

[submodule "ecosystem/07-remote-infra/claude-code-telegram"]
    path = ecosystem/07-remote-infra/claude-code-telegram
    url = https://github.com/nicholasgriffintn/claude-code-telegram.git
    branch = main

# --- 08-automation-cicd (1 repo) ---

[submodule "ecosystem/08-automation-cicd/claude-code-action"]
    path = ecosystem/08-automation-cicd/claude-code-action
    url = https://github.com/anthropics/claude-code-action.git
    branch = main

# --- 09-config-settings (4 repos) ---

[submodule "ecosystem/09-config-settings/claude-code-config"]
    path = ecosystem/09-config-settings/claude-code-config
    url = https://github.com/nicholasgriffintn/claude-code-config.git
    branch = main

[submodule "ecosystem/09-config-settings/claude-code-config2"]
    path = ecosystem/09-config-settings/claude-code-config2
    url = https://github.com/nicholasgriffintn/claude-code-config2.git
    branch = main

[submodule "ecosystem/09-config-settings/claude-code-settings"]
    path = ecosystem/09-config-settings/claude-code-settings
    url = https://github.com/nicholasgriffintn/claude-code-settings.git
    branch = main

[submodule "ecosystem/09-config-settings/claude-code-showcase"]
    path = ecosystem/09-config-settings/claude-code-showcase
    url = https://github.com/nicholasgriffintn/claude-code-showcase.git
    branch = main

# --- 10-curated-guides (7 repos) ---

[submodule "ecosystem/10-curated-guides/awesome-claude-code"]
    path = ecosystem/10-curated-guides/awesome-claude-code
    url = https://github.com/hesreallyhim/awesome-claude-code.git
    branch = main

[submodule "ecosystem/10-curated-guides/awesome-claude-code-subagents"]
    path = ecosystem/10-curated-guides/awesome-claude-code-subagents
    url = https://github.com/nicholasgriffintn/awesome-claude-code-subagents.git
    branch = main

[submodule "ecosystem/10-curated-guides/claude-code-best-practice"]
    path = ecosystem/10-curated-guides/claude-code-best-practice
    url = https://github.com/shanraisshan/claude-code-best-practice.git
    branch = main

[submodule "ecosystem/10-curated-guides/claude-code-cheat-sheet"]
    path = ecosystem/10-curated-guides/claude-code-cheat-sheet
    url = https://github.com/nicholasgriffintn/claude-code-cheat-sheet.git
    branch = main

[submodule "ecosystem/10-curated-guides/claude-code-guide"]
    path = ecosystem/10-curated-guides/claude-code-guide
    url = https://github.com/nicholasgriffintn/claude-code-guide.git
    branch = main

[submodule "ecosystem/10-curated-guides/claude-code-mastering"]
    path = ecosystem/10-curated-guides/claude-code-mastering
    url = https://github.com/nicholasgriffintn/claude-code-mastering.git
    branch = main

[submodule "ecosystem/10-curated-guides/claude-code-tips"]
    path = ecosystem/10-curated-guides/claude-code-tips
    url = https://github.com/nicholasgriffintn/claude-code-tips.git
    branch = main

# --- 11-docs-internals (2 repos) ---

[submodule "ecosystem/11-docs-internals/claude-code-docs"]
    path = ecosystem/11-docs-internals/claude-code-docs
    url = https://github.com/nicholasgriffintn/claude-code-docs.git
    branch = main

[submodule "ecosystem/11-docs-internals/claude-code-system-prompts"]
    path = ecosystem/11-docs-internals/claude-code-system-prompts
    url = https://github.com/nicholasgriffintn/claude-code-system-prompts.git
    branch = main

# --- 12-prompts-learning (3 repos) ---

[submodule "ecosystem/12-prompts-learning/claude-code-pm-course"]
    path = ecosystem/12-prompts-learning/claude-code-pm-course
    url = https://github.com/nicholasgriffintn/claude-code-pm-course.git
    branch = main

[submodule "ecosystem/12-prompts-learning/claude-code-prompt-improver"]
    path = ecosystem/12-prompts-learning/claude-code-prompt-improver
    url = https://github.com/nicholasgriffintn/claude-code-prompt-improver.git
    branch = main

[submodule "ecosystem/12-prompts-learning/claude-code-requirements-builder"]
    path = ecosystem/12-prompts-learning/claude-code-requirements-builder
    url = https://github.com/nicholasgriffintn/claude-code-requirements-builder.git
    branch = main

# --- 13-transcripts-security-misc (8 repos) ---

[submodule "ecosystem/13-transcripts-security-misc/Claude-Code-Communication"]
    path = ecosystem/13-transcripts-security-misc/Claude-Code-Communication
    url = https://github.com/nicholasgriffintn/Claude-Code-Communication.git
    branch = main

[submodule "ecosystem/13-transcripts-security-misc/claude-code-infrastructure-showcase"]
    path = ecosystem/13-transcripts-security-misc/claude-code-infrastructure-showcase
    url = https://github.com/nicholasgriffintn/claude-code-infrastructure-showcase.git
    branch = main

[submodule "ecosystem/13-transcripts-security-misc/claude-code-my-workflow"]
    path = ecosystem/13-transcripts-security-misc/claude-code-my-workflow
    url = https://github.com/nicholasgriffintn/claude-code-my-workflow.git
    branch = main

[submodule "ecosystem/13-transcripts-security-misc/claude-code-reverse"]
    path = ecosystem/13-transcripts-security-misc/claude-code-reverse
    url = https://github.com/nicholasgriffintn/claude-code-reverse.git
    branch = main

[submodule "ecosystem/13-transcripts-security-misc/claude-code-security-review"]
    path = ecosystem/13-transcripts-security-misc/claude-code-security-review
    url = https://github.com/nicholasgriffintn/claude-code-security-review.git
    branch = main

[submodule "ecosystem/13-transcripts-security-misc/claude-code-transcripts"]
    path = ecosystem/13-transcripts-security-misc/claude-code-transcripts
    url = https://github.com/nicholasgriffintn/claude-code-transcripts.git
    branch = main

[submodule "ecosystem/13-transcripts-security-misc/claude-coder"]
    path = ecosystem/13-transcripts-security-misc/claude-coder
    url = https://github.com/nicholasgriffintn/claude-coder.git
    branch = main

[submodule "ecosystem/13-transcripts-security-misc/edmunds-claude-code"]
    path = ecosystem/13-transcripts-security-misc/edmunds-claude-code
    url = https://github.com/nicholasgriffintn/edmunds-claude-code.git
    branch = main
```

> **Important**: The URLs above are placeholders for repos whose canonical upstream URL has not been confirmed. After populating the `ecosystem/` directory, run `scripts/sync-remotes.sh` to discover the actual upstream URLs from existing clones and update both `reference_repos.json` and the remotes in each submodule. The confirmed URLs from the existing `reference_repos.json` are:
>
> - `awesome-claude-code` -> `https://github.com/hesreallyhim/awesome-claude-code.git`
> - `Claude-Code-Workflow` -> `https://github.com/catlog22/Claude-Code-Workflow.git`
> - `claude-code-best-practice` -> `https://github.com/shanraisshan/claude-code-best-practice.git`
> - `claude-code-action` -> `https://github.com/anthropics/claude-code-action.git` (Anthropic official)
>
> All other URLs should be verified by running `sync-remotes.sh` against the existing `refrence-repo/` clones.

---

## Appendix A: File and Directory Layout

Summary of all files and directories created or referenced by this system:

```
claude-parent/
├── ecosystem/                              # All 62 submodules (Section 2)
│   ├── 01-desktop-ides/
│   ├── 02-orchestration/
│   ├── ...
│   └── 13-transcripts-security-misc/
├── ecosystem-state/                        # Tracking state (Sections 3, 8)
│   ├── ecosystem-state.json                # Per-repo last-checked, SHA, classification
│   ├── absorption-status.json              # Per-repo absorption percentage (Section 8)
│   ├── contributions.json                  # Upstream contributions log (Section 9)
│   ├── forks.json                          # Fork divergence tracking (Section 9)
│   ├── claude-code/                        # Claude Code-specific state (Section 7)
│   │   ├── last-version.txt
│   │   ├── cli-flags.txt
│   │   ├── cli-flags-previous.txt
│   │   ├── hook-types.txt
│   │   ├── hook-types-previous.txt
│   │   ├── tool-types.txt
│   │   └── tool-types-previous.txt
│   ├── imports/                            # Auto-import staging (Section 5)
│   │   ├── import-YYYY-MM-DD.sql
│   │   └── import-YYYY-MM-DD.log
│   └── reports/                            # Generated reports (Sections 3, 7, 8)
│       ├── ecosystem-diff-YYYY-MM-DD.md
│       ├── ecosystem-health-YYYY-MM-DD.md
│       ├── ecosystem-health-YYYY-MM-DD.json
│       └── claude-code-compat-YYYY-MM-DD.md
├── forge-config/                           # Forge configuration
│   ├── compatibility.json                  # Claude Code compatibility matrix (Section 7)
│   ├── seed-skills.sql                     # Auto-imported skills (Section 5)
│   └── presets/                            # Imported config presets
├── scripts/                                # Automation scripts (Section 10)
│   ├── reference_repos.json                # Master repo list (existing)
│   ├── remotes-config.env                  # Remote configuration (existing)
│   ├── init-submodules.sh                  # Submodule initialization (existing)
│   ├── sync-remotes.sh                     # Remote synchronization (existing)
│   ├── extract-with-gitnexus.sh            # GitNexus extraction (existing)
│   ├── update-ecosystem.sh                 # Ecosystem monitoring (Section 10)
│   ├── import-skills.sh                    # Skill auto-import (Section 10)
│   ├── check-claude-code.sh                # Claude Code compatibility (Section 10)
│   └── ecosystem-report.sh                 # Health report generation (Section 10)
├── .github/workflows/                      # CI/CD workflows (Section 11)
│   ├── ecosystem-monitor.yml
│   ├── auto-import.yml
│   └── claude-code-compat.yml
├── .gitmodules                             # Submodule registry (Section 12)
└── reference-map/                          # Per-repo analysis docs (existing)
    ├── 01-desktop-ides/
    ├── 02-orchestration-workflows/
    └── ...
```

## Appendix B: Operational Runbook

### Weekly Cadence

| Day | Action | Script/Workflow |
|-----|--------|-----------------|
| Sunday 00:00 UTC | Ecosystem monitor runs | `ecosystem-monitor.yml` |
| Sunday morning | Review ecosystem diff issue | Manual |
| Monday | Triage: assign data imports and pattern reviews | Manual |
| Wednesday 06:00 UTC | Claude Code compatibility check | `claude-code-compat.yml` |
| Wednesday | Review Claude Code compat report | Manual |
| Friday | Run ecosystem health report, update absorption status | `ecosystem-report.sh` |

### On-Demand Procedures

**New repo discovered in the ecosystem:**

1. Add to `scripts/reference_repos.json` with slug, category, description, and `github_url`.
2. Run `scripts/init-submodules.sh` to add as submodule.
3. Run `scripts/sync-remotes.sh --set` to configure remotes.
4. Run `scripts/extract-with-gitnexus.sh --manual` to generate initial docs.
5. Create a reference-map entry in the appropriate category.
6. Run the ANALYZE phase of the Absorption Pipeline.

**Repo archived or deleted upstream:**

1. Update `reference_repos.json` with a note.
2. If we have a fork: keep the fork as a mirror. Change the submodule URL to our fork.
3. If no fork: archive our local copy and document what we absorbed from it.
4. Update absorption status to reflect that tracking is no longer needed.

**Claude Code major release:**

1. Run `check-claude-code.sh` immediately.
2. Review the compatibility report.
3. Run Forge's integration test suite.
4. If breaking changes: create a `fix/claude-code-compat` branch and address immediately.
5. Update `forge-config/compatibility.json`.
6. Update documentation.

---

## Appendix C: Design Decisions

**Why git submodules instead of a monorepo copy?**
Submodules preserve upstream history, allow independent updates per repo, and avoid bloating the main repository with 62 copies of third-party code. The tradeoff is submodule complexity, which the automation scripts mitigate.

**Why weekly instead of daily monitoring?**
Most ecosystem repos update a few times per month. Weekly monitoring catches changes within 7 days, which is fast enough for non-critical updates. Claude Code compatibility checks run mid-week to stagger the workload. Critical changes (like Claude Code releases) can be detected earlier via the on-demand workflow trigger.

**Why auto-import only for data, not patterns?**
Data (skills, presets, configs) has a well-defined schema and can be validated programmatically. Patterns require understanding intent, assessing architectural fit, and making design decisions that only humans can make. Attempting to auto-import patterns would risk introducing code that does not fit Forge's architecture.

**Why fork management instead of read-only tracking?**
Some repos will benefit from Forge improvements that we want to contribute back. Having forks ready means we can immediately open PRs upstream without setup overhead. For repos we never modify, the fork exists but stays in sync with upstream automatically.
