# Claude Forge -- User Personas

**Version**: 1.0
**Date**: 2026-02-25

---

## Table of Contents

1. [Persona 1: Maya Chen -- Solo AI Developer](#persona-1-maya-chen----solo-ai-developer)
2. [Persona 2: David Okonkwo -- Team Lead](#persona-2-david-okonkwo----team-lead)
3. [Persona 3: Rina Vasquez -- Tool Builder](#persona-3-rina-vasquez----tool-builder)
4. [Persona 4: James Aldridge -- DevOps / CI Engineer](#persona-4-james-aldridge----devops--ci-engineer)
5. [Persona 5: Sven Holmberg -- Open Source Contributor](#persona-5-sven-holmberg----open-source-contributor)
6. [Persona Summary Matrix](#persona-summary-matrix)

---

## Persona 1: Maya Chen -- Solo AI Developer

### Background

Maya is a 28-year-old full-stack developer working as an independent contractor. She builds web applications and internal tools for small to mid-size companies. She works alone, managing 3-4 client projects simultaneously. She has been using Claude Code for 8 months and considers it her primary development tool. She works from a MacBook Pro and spends 10+ hours daily in her terminal and editor.

### Demographics

- **Role**: Independent software contractor
- **Experience**: 5 years professional development
- **Team size**: Solo
- **Projects**: 3-4 concurrent client projects
- **Platform**: macOS (Apple Silicon)
- **LLM Plan**: Claude Max20

### Goals

1. **Maximize throughput**: Complete more work per day by running multiple agents in parallel across different projects.
2. **Minimize context switching**: Keep all projects, agents, and sessions accessible from one interface rather than juggling terminals.
3. **Control costs**: Stay within her Max20 plan budget while maximizing the value of each API call.
4. **Maintain quality**: Ensure agents do not introduce bugs or security vulnerabilities, even when she is not actively watching.
5. **Build a personal library**: Accumulate skills and workflows that make her faster on every subsequent project.

### Frustrations

- **Tool sprawl**: Currently uses 5 separate tools (Claude Code CLI, a session viewer, a usage monitor, a skill collection, and manual Git worktree management). None talk to each other.
- **Lost sessions**: Has lost important session context when resuming after interruptions. Searching for a past session where she solved a specific problem takes too long.
- **Cost surprises**: Has exceeded her plan limits twice because she had no visibility into token usage until after the fact.
- **Agent runaway**: An agent once spent 45 minutes in a loop rewriting the same file. She did not notice until she checked back.
- **Skill discovery**: Knows there are useful skills somewhere in GitHub, but finding, evaluating, and installing them is a multi-step manual process.

### How Maya Uses Forge

Maya starts her day by opening Forge in her browser. She sees her Kanban board with sessions from yesterday -- two completed, one paused mid-way. She resumes the paused session with one click. While that agent works on a React component, she starts a new agent for a different client project, using a worktree so the two do not interfere.

She glances at the pulse chart to see both agents are making progress. The cost panel shows she has used 35% of her daily budget. She searches for "pagination" across all past sessions and finds the solution she wrote two months ago, copies the approach into her current prompt.

When one agent finishes, a desktop notification pops up. She reviews the diff in the Git panel, stages the files, and commits -- all without leaving Forge. She schedules a nightly cron job to run her test suite agent and email her the results.

### Key Features Maya Cares About

| Priority | Feature | Why |
|----------|---------|-----|
| P0 | Multi-agent parallel execution | Runs agents for different projects simultaneously |
| P0 | Session search (FTS) | Finds past solutions quickly |
| P0 | Cost tracking and predictions | Prevents budget overruns |
| P0 | Circuit breaker | Prevents runaway agents she cannot watch |
| P1 | Worktree-per-agent | Keeps project files isolated |
| P1 | Skill catalog and search | Discovers tools to work faster |
| P1 | Desktop notifications | Knows when agents finish without watching |
| P1 | Cron scheduler | Runs nightly tasks unattended |
| P2 | Session export (Markdown) | Shares solutions with clients |
| P2 | Git-versioned edits | Rolls back individual agent changes |

### Success Scenario

Maya completes a two-week client project in five days. She ran three agents in parallel for the frontend, backend, and tests. The circuit breaker caught a loop on day 2, saving two hours. She found and installed a "Next.js API Routes" skill that generated her entire API layer from the OpenAPI spec. Her final cost was 15% under budget because usage predictions let her schedule heavy work during off-peak hours.

---

## Persona 2: David Okonkwo -- Team Lead

### Background

David is a 36-year-old engineering manager at a 200-person SaaS company. He leads a team of 8 developers who have adopted Claude Code as their standard AI coding tool. His primary concern is not writing code himself (though he does occasionally), but ensuring his team uses AI agents productively and safely. He needs visibility into what agents are doing across the team, control over budgets, and confidence that agents are not introducing security issues.

### Demographics

- **Role**: Engineering Manager / Tech Lead
- **Experience**: 12 years, 4 in management
- **Team size**: 8 developers
- **Projects**: 1 main product, 3 microservices
- **Platform**: macOS, with team on Linux and macOS
- **LLM Plan**: Enterprise (team account)

### Goals

1. **Team visibility**: See what every agent on the team is doing, what it costs, and whether it is stuck.
2. **Budget enforcement**: Set per-developer and per-project budgets that cannot be exceeded.
3. **Security compliance**: Ensure agents do not expose secrets, modify protected files, or introduce vulnerabilities.
4. **Workflow standardization**: Define team-wide workflows that developers can instantiate for common tasks.
5. **Metrics and reporting**: Generate weekly reports on agent usage, costs, and productivity for leadership.

### Frustrations

- **No team visibility**: Each developer runs Claude Code independently. David has no way to see aggregate usage or costs.
- **Inconsistent practices**: Some developers use agents with no safety guardrails, while others are overly cautious and underutilize the tools.
- **Security incidents**: A developer's agent once committed an `.env` file to a public branch. The hook to prevent this was not configured consistently.
- **Budget overruns**: The team exceeded their monthly API budget twice in Q4 because there was no enforcement mechanism.
- **No shared knowledge**: When one developer creates a useful workflow or skill, there is no systematic way to share it with the team.

### How David Uses Forge

David opens Forge and navigates to the team dashboard. He sees 12 active agents across 5 projects. The swim-lane view shows Agent-7 has been stuck for 15 minutes -- he clicks to investigate and sees it is looping on a test failure. He kills it and notifies the developer via Slack.

He reviews the weekly cost report: the team is at 72% of budget with 8 days remaining, tracking to finish under. He notices one developer is using 40% of the budget and drills down to see they are running brainstorm-level workflows for simple tasks. He sends a note suggesting they use lite-lite level instead.

He opens the security audit log and confirms that file protection rules caught 3 attempts to modify `.env` files this week -- all handled correctly. He exports the log for the compliance review.

He creates a new workflow template for "Feature Implementation" that includes a security scan step and a test step, then pushes it to the team's shared configuration.

### Key Features David Cares About

| Priority | Feature | Why |
|----------|---------|-----|
| P0 | Multi-agent swim-lane view | Sees all team agents at a glance |
| P0 | Budget controls (per-user, per-project) | Prevents overspending |
| P0 | File protection rules | Prevents secrets from leaking |
| P0 | Audit log | Compliance and incident investigation |
| P1 | Cost analytics (team rollup) | Reports to leadership |
| P1 | Workflow templates | Standardizes team practices |
| P1 | Security scanning | Catches vulnerabilities before merge |
| P1 | Permission system | Controls what agents can do |
| P2 | Usage predictions | Plans budget for next quarter |
| P2 | Notification rules | Gets alerted on anomalies |

### Success Scenario

David's team completes a major feature release with zero security incidents, 8% under budget, and 30% faster than the previous release. The workflow template he created was used by all 8 developers, ensuring consistent quality. The audit log showed 100% compliance with the company's security policy. Leadership asks him to roll out Forge to two more teams.

---

## Persona 3: Rina Vasquez -- Tool Builder

### Background

Rina is a 32-year-old platform engineer at a developer tools company. She builds internal tooling and automation systems. Her team is creating a custom AI-powered development platform for their company, and she wants to integrate Claude Code's capabilities into it without requiring developers to use the Claude Code CLI directly. She sees Forge's MCP server as the integration point.

### Demographics

- **Role**: Platform Engineer
- **Experience**: 8 years, 3 in platform engineering
- **Team size**: 4 (platform team), 50 (internal users)
- **Projects**: Internal developer platform
- **Platform**: Linux (servers), macOS (development)
- **LLM Plan**: Enterprise API

### Goals

1. **Programmatic access**: Invoke all Forge capabilities from her platform's backend code, not from a browser.
2. **Composability**: Chain Forge operations together in custom workflows that her platform orchestrates.
3. **Abstraction**: Hide the complexity of agent management, safety, and sessions behind a clean API.
4. **Reliability**: Integrate Forge into a production system that must be available 99.9% of the time.
5. **Extensibility**: Write custom plugins to bridge Forge with her company's internal tools (Jira, Confluence, internal code review).

### Frustrations

- **No stable API**: Most Claude Code ecosystem tools are designed for human interaction, not programmatic use. Scraping CLIs and parsing stdout is fragile.
- **No MCP standard compliance**: Several tools claim MCP support but implement it partially or incorrectly.
- **Integration overhead**: Connecting each ecosystem tool separately requires maintaining multiple integrations.
- **Plugin limitations**: Existing plugin systems are JavaScript-only or require running additional servers.
- **Error handling**: Tools designed for humans give human-readable errors that are hard to parse programmatically.

### How Rina Uses Forge

Rina runs Forge as an MCP server on a Linux server in her company's infrastructure. Her platform's backend connects to it via SSE transport. When a developer submits a task through the internal portal, Rina's platform:

1. Calls `create_agent` with the appropriate preset and project directory.
2. Calls `run_agent` with the task description.
3. Subscribes to the `forge://events` resource for real-time progress.
4. On completion, calls `git_diff` and `export_session` to package the results.
5. Posts the results to the internal code review system via a WASM plugin Rina wrote.

Rina also wrote a custom MCP-to-REST bridge that lets non-MCP-aware systems use Forge over plain HTTP. She contributes bug fixes and feature requests to the Forge repository.

### Key Features Rina Cares About

| Priority | Feature | Why |
|----------|---------|-----|
| P0 | MCP server with full tool exposure | Programmatic access to everything |
| P0 | MCP transport support (stdio, SSE, WS) | Flexible integration options |
| P0 | Structured error responses | Parseable errors for automation |
| P0 | WASM plugin host | Custom integrations in a sandbox |
| P1 | MCP resource exposure | Query state without tool calls |
| P1 | Health check endpoint | Monitoring in production |
| P1 | Circuit breaker and rate limiter | Reliability for unattended operation |
| P1 | Event stream subscription | Real-time progress for her UI |
| P2 | MCP prompt templates | Standardized task descriptions |
| P2 | Native plugin interface | High-performance integrations |

### Success Scenario

Rina's internal platform processes 200+ AI-assisted tasks per day, all routed through Forge's MCP server. Average task completion time dropped from 45 minutes (manual) to 12 minutes (automated). Her WASM plugin for Jira integration was adopted by two other teams. Forge's health check endpoint integrates with their PagerDuty alerting, and the circuit breaker has prevented 15 runaway agent incidents without human intervention.

---

## Persona 4: James Aldridge -- DevOps / CI Engineer

### Background

James is a 41-year-old DevOps engineer at a fintech company. He manages CI/CD pipelines for 20+ microservices. He wants to integrate AI-powered code review, automated PR implementation, and security scanning into the existing pipeline. He is cautious about AI tools because they must meet the company's security and compliance requirements. He does not use AI for his own coding but is responsible for making it available to the development teams safely.

### Demographics

- **Role**: Senior DevOps Engineer
- **Experience**: 15 years, 7 in DevOps/SRE
- **Team size**: 3 (DevOps team), 60 (developers served)
- **Projects**: 20+ microservices, 5 CI/CD pipelines
- **Platform**: Linux (CI runners), macOS (personal)
- **LLM Plan**: Enterprise API with budget controls

### Goals

1. **Automated code review**: Every PR gets an AI review before human reviewers see it.
2. **Security gate**: AI-powered security scanning blocks PRs with critical vulnerabilities.
3. **Compliance**: Full audit trail of every AI action in the pipeline.
4. **Cost control**: Predictable CI costs with hard budget limits.
5. **Minimal disruption**: Integrate into existing GitHub Actions workflows without rewriting pipelines.

### Frustrations

- **Security concerns**: AI tools that auto-approve file writes or shell commands are unacceptable in a fintech environment.
- **Non-deterministic costs**: AI-powered CI steps have unpredictable costs. A 10-line PR and a 1,000-line PR can cost the same or wildly different amounts.
- **No audit trail**: Existing AI CI integrations do not provide the level of audit logging required by the compliance team.
- **Pipeline complexity**: Each AI tool adds its own step, configuration, and failure mode to the pipeline.
- **False positives**: Security scanning tools flag too many false positives, causing developers to ignore the results.

### How James Uses Forge

James adds Forge to the CI pipeline as a GitHub Action step. The action runs Forge in headless mode (no UI) with the MCP interface.

For every PR:
1. Forge runs a `security-review` agent that scans the diff for vulnerabilities. Results are posted as PR comments with severity ratings.
2. Forge runs a `code-review` agent that provides general code quality feedback.
3. If the PR was created from an issue, Forge verifies the implementation addresses the issue requirements.
4. All actions are logged to the audit trail, which is exported to the company's SIEM.

James configures hard budget limits per repository. The circuit breaker prevents long-running reviews on massive PRs. File protection rules prevent the AI from modifying CI configuration, Dockerfiles, or infrastructure-as-code files.

James monitors aggregate CI costs and review quality through Forge's observability dashboard, which he accesses from a monitoring screen in the DevOps war room.

### Key Features James Cares About

| Priority | Feature | Why |
|----------|---------|-----|
| P0 | Headless / CLI mode | Runs in CI without a browser |
| P0 | Audit log with export | Compliance requirement |
| P0 | Security scanning (diff-aware) | Blocks vulnerable code |
| P0 | File protection rules | Prevents AI from modifying infra |
| P0 | Budget controls (hard limits) | Predictable CI costs |
| P1 | GitHub Action integration | Drops into existing pipelines |
| P1 | Permission system | Least-privilege for CI agents |
| P1 | Circuit breaker | Prevents hung CI jobs |
| P1 | Structured JSON output | Machine-parseable results |
| P2 | False positive filtering | Reduces developer alert fatigue |
| P2 | Cost analytics per repository | Chargeback to teams |

### Success Scenario

James deploys Forge to all 20 microservice repositories. AI code review catches an average of 2.3 issues per PR that human reviewers would have missed. Security scanning has a 4% false positive rate (down from 25% with the previous tool). CI costs are predictable within 5% month over month. The compliance team signs off on the audit trail format. Developer satisfaction with AI reviews is 4.2/5 in the quarterly survey.

---

## Persona 5: Sven Holmberg -- Open Source Contributor

### Background

Sven is a 25-year-old computer science graduate student in Stockholm. He is passionate about open-source tooling and AI. He has contributed to several Claude Code ecosystem repositories and wants to build extensions for Forge. He is equally interested in writing Rust plugins, creating WASM modules, and publishing skill collections. He sees Forge as a platform he can build a reputation on.

### Demographics

- **Role**: Graduate student, open-source contributor
- **Experience**: 3 years (academic + OSS)
- **Team size**: Solo (with OSS community)
- **Projects**: Personal projects, OSS contributions
- **Platform**: Linux (NixOS)
- **LLM Plan**: Claude Pro (personal budget)

### Goals

1. **Learn the platform**: Understand Forge's architecture deeply enough to contribute meaningfully.
2. **Build plugins**: Create WASM plugins that solve problems he cares about (ML workflow integration, Jupyter notebook support).
3. **Publish skills**: Write and publish high-quality skills that rank well in the catalog.
4. **Contribute to core**: Fix bugs and implement features in the Forge codebase itself.
5. **Build reputation**: Become a recognized contributor with published plugins and skills that others use.

### Frustrations

- **High barrier to entry**: Rust + Svelte + SQLite + MCP is a complex stack. Getting the development environment set up takes effort.
- **Incomplete documentation**: Internal APIs and extension points are not always well-documented.
- **Plugin API instability**: If the plugin API changes, his extensions break.
- **Skill quality bar**: Without clear grading criteria, he does not know what makes a skill "good enough" to publish.
- **Limited feedback loop**: After publishing a skill, he has no way to know if anyone is using it or finding issues.

### How Sven Uses Forge

Sven clones the Forge repository and sets up the development environment. He reads the architecture documentation and runs the test suite to understand the codebase. He starts by fixing a small bug in the session browser -- this gives him confidence with the codebase.

Next, he builds a WASM plugin that integrates Forge with Jupyter notebooks: the plugin can read `.ipynb` files, extract code cells, and format them as agent context. He tests it locally, writes documentation, and publishes it to the plugin marketplace.

He also creates a collection of 15 ML-focused skills (PyTorch debugging, model training workflow, dataset preparation) and publishes them to the skill catalog. He monitors the quality grades and download counts to see how they perform.

He joins the Forge Discord, answers questions from new users, and submits feature requests based on his own needs. Over six months, he becomes a recognized contributor.

### Key Features Sven Cares About

| Priority | Feature | Why |
|----------|---------|-----|
| P0 | WASM plugin host with clear API | Building extensions safely |
| P0 | Skill quality grading (100-point) | Knows what to aim for |
| P0 | Skill/plugin publishing workflow | Shares his work with the community |
| P1 | Development environment setup docs | Gets started without friction |
| P1 | Plugin marketplace with metrics | Sees download counts and ratings |
| P1 | Skill factory / builder wizard | Creates skills faster |
| P1 | SKILL.md standard documentation | Follows the correct format |
| P2 | Native plugin interface (Rust trait) | High-performance extensions |
| P2 | Contributing guide | Knows how to submit PRs |
| P2 | API stability guarantees | Confidence his plugins will not break |

### Success Scenario

In six months, Sven has published 2 WASM plugins and 30 skills. His Jupyter plugin has 500+ installs and a 92/100 quality grade. His ML skill collection averages 85/100 and appears in the "Top Skills" section of the catalog. He has merged 8 PRs to Forge core, including a feature that adds Jupyter cell extraction to the code viewer. He is invited to be a maintainer of the skills catalog. His contributions become part of his portfolio, and he cites them in job applications.

---

## Persona Summary Matrix

| Dimension | Maya (Solo Dev) | David (Team Lead) | Rina (Tool Builder) | James (DevOps) | Sven (Contributor) |
|-----------|----------------|-------------------|---------------------|----------------|-------------------|
| **Primary mode** | UI (browser) | UI (dashboard) | MCP (programmatic) | CLI (headless) | UI + code |
| **Primary concern** | Productivity | Visibility, Safety | Integration | Compliance | Extensibility |
| **Budget sensitivity** | High | Medium | Low (enterprise) | Medium | High |
| **Technical depth** | Medium | Low-Medium | High | Medium-High | High |
| **Concurrent agents** | 2-4 | Monitors 10-20 | 50-200 (automated) | 1-5 per pipeline | 1-2 |
| **Top P0 feature** | Multi-agent parallel | Swim-lane view | MCP tools | Audit log | WASM plugin host |
| **Risk tolerance** | Medium | Low | Medium | Very Low | High |
| **Session count** | 100s/month | Views 1000s | 1000s/day | 100s/day | 10s/month |
| **Skills usage** | Consumer | Curator | Builder | Ignores | Creator |
| **Notification pref** | Desktop | Slack/email | Webhook | Pipeline logs | Discord |

### Feature Priority by Persona

The table below maps which features each persona considers P0 (must have), P1 (should have), or P2 (nice to have). Features not listed are P3 (not a priority) for that persona.

| Feature | Maya | David | Rina | James | Sven |
|---------|------|-------|------|-------|------|
| Agent CRUD | P0 | P1 | P0 | P1 | P1 |
| Multi-agent parallel | P0 | P0 | P0 | P2 | P2 |
| Circuit breaker | P0 | P0 | P1 | P1 | P2 |
| Rate limiter | P1 | P0 | P1 | P0 | P2 |
| Budget controls | P0 | P0 | P2 | P0 | P1 |
| Session browser | P0 | P1 | P2 | P2 | P1 |
| Session FTS search | P0 | P1 | P2 | P2 | P1 |
| Cron scheduler | P1 | P2 | P1 | P1 | P2 |
| Skill catalog | P1 | P1 | P2 | -- | P0 |
| WASM plugins | P2 | P2 | P0 | P2 | P0 |
| MCP server | P2 | P2 | P0 | P1 | P1 |
| Git integration | P1 | P1 | P1 | P1 | P1 |
| Worktree-per-agent | P1 | P1 | P1 | P2 | P2 |
| Swim-lane view | P1 | P0 | P2 | P2 | P1 |
| Cost analytics | P0 | P0 | P2 | P1 | P1 |
| Audit log | P2 | P0 | P1 | P0 | P2 |
| File protection | P1 | P0 | P1 | P0 | P2 |
| Security scanning | P2 | P1 | P2 | P0 | P2 |
| Workflow templates | P1 | P1 | P1 | P1 | P2 |
| Notifications | P1 | P1 | P1 | P2 | P2 |
| Health check | P2 | P1 | P0 | P1 | P1 |
| Config scopes | P1 | P0 | P1 | P1 | P1 |
| Plugin marketplace | P1 | P2 | P1 | -- | P0 |
