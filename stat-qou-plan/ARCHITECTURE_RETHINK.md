# Architecture Rethink: Claude Code IS the Engine

> Stop building what Claude Code already does. Orchestrate, govern, observe.
> Date: 2026-03-15

---

## The Realization

AgentForge spawns `claude` CLI as its execution backend. But we've been **duplicating** things Claude Code already does and does better:

| We built | Claude Code already has |
|----------|----------------------|
| 30 skills (markdown injection) | Native skills system (slash commands, skill files) |
| Memory repo (empty) | Built-in memory (`/memory`, persistent across sessions) |
| Hooks repo (empty) | Native hooks (pre/post tool use, notifications) |
| LoopDetector | Built-in loop detection |
| Sub-agent tracking | Native subagent spawning with Agent tool |
| Session management | Session resume, conversation history |

**We're reinventing wheels that are evolving faster than we can copy them.**

Claude Code ships updates weekly — plugins, MCP servers, new agent capabilities, better tool use. Every feature we duplicate is a feature we have to maintain while the original gets better without us.

---

## The Shift: Orchestrate, Don't Replicate

### What AgentForge should NOT do
- Build its own skill system (Claude Code has one)
- Build its own memory system (Claude Code has one)
- Build its own hook system (Claude Code has one)
- Build its own workflow engine (use Claude Code's Agent tool for sub-tasks)
- Parse Claude Code's output format (it will change)

### What AgentForge SHOULD do
- **Workforce management** — who works on what (personas, org charts, assignments)
- **Governance** — budgets, approvals, permission scoping
- **Orchestration** — spawn, configure, and coordinate multiple Claude Code instances
- **Observation** — capture events, track costs, audit decisions
- **Configuration** — set up Claude Code instances with the right CLAUDE.md, skills, MCP servers
- **Exposure** — make all of this available as MCP tools to upstream clients

---

## New Architecture

```
┌─────────────────────────────────────────────┐
│  UPSTREAM (any MCP client)                  │
│  Claude Code · Cursor · ADK · scripts       │
└──────────────┬──────────────────────────────┘
               │ MCP tools
┌──────────────▼──────────────────────────────┐
│  AGENTFORGE (Orchestrator + Governor)       │
│                                             │
│  ┌─────────┐ ┌──────────┐ ┌─────────────┐  │
│  │Workforce│ │Governance│ │ Observation  │  │
│  │Manager  │ │  Gate    │ │   Engine     │  │
│  │         │ │          │ │              │  │
│  │Personas │ │Budgets   │ │Event capture │  │
│  │Org chart│ │Approvals │ │Cost tracking │  │
│  │Assign   │ │Scoping   │ │Audit log     │  │
│  └────┬────┘ └────┬─────┘ └──────┬──────┘  │
│       │           │              │          │
│  ┌────▼───────────▼──────────────▼──────┐   │
│  │  Orchestration Layer                 │   │
│  │                                      │   │
│  │  Spawn Claude Code instances with:   │   │
│  │  - Custom CLAUDE.md per persona      │   │
│  │  - Scoped allowed tools              │   │
│  │  - Budget limits as env vars         │   │
│  │  - Project-specific MCP servers      │   │
│  │  - Git worktree isolation            │   │
│  │  - Hooks for event reporting         │   │
│  └──────────────┬───────────────────────┘   │
└─────────────────┼───────────────────────────┘
                  │ spawns + configures
┌─────────────────▼───────────────────────────┐
│  CLAUDE CODE INSTANCES (the actual workers) │
│                                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ Agent A  │ │ Agent B  │ │ Agent C  │    │
│  │ (Arch)   │ │ (Dev)    │ │ (Review) │    │
│  │          │ │          │ │          │    │
│  │ Its own  │ │ Its own  │ │ Its own  │    │
│  │ skills   │ │ skills   │ │ skills   │    │
│  │ memory   │ │ memory   │ │ memory   │    │
│  │ hooks    │ │ hooks    │ │ hooks    │    │
│  │ tools    │ │ tools    │ │ tools    │    │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘    │
│       │            │            │           │
│       └────────────┼────────────┘           │
│                    │ hooks / events          │
└────────────────────┼────────────────────────┘
                     │ flow back
┌────────────────────▼────────────────────────┐
│  AGENTFORGE captures:                       │
│  - What tools were used                     │
│  - How much it cost                         │
│  - What files were changed                  │
│  - Whether it succeeded or failed           │
│  - Security scan results                    │
│  - Session output for audit                 │
└─────────────────────────────────────────────┘
```

---

## How Claude Code Becomes Configurable Per-Agent

When AgentForge spawns a Claude Code instance for a persona, it sets up the environment:

### 1. Custom CLAUDE.md (persona → system instructions)
```markdown
# You are: Senior Security Auditor
# Company: Acme AI Corp
# Department: Engineering
# Reports to: Lead Architect
# Active goals: Launch v1.0, Security audit pass
# Budget remaining: $340 of $500

## Your role
{persona.personality}

## Your deliverables
{persona.deliverables}

## Rules
- Stay within your role scope
- Report costs after each task
- Request approval for changes > 50 files
```

### 2. Scoped tools via --allowedTools
```bash
claude --allowedTools "Read,Grep,Glob,Bash(git diff),Bash(cargo test)" \
       --model claude-sonnet-4-6 \
       --max-turns 20 \
       -p "Review the auth module for vulnerabilities"
```

### 3. Budget enforcement via hooks
```json
// .claude/hooks.json injected per agent
{
  "afterToolUse": [{
    "command": "curl -s http://localhost:4173/api/v1/hook/cost-check?session=$SESSION_ID",
    "timeout": 2000
  }]
}
```

### 4. Event reporting via hooks
```json
{
  "afterResponse": [{
    "command": "curl -X POST http://localhost:4173/api/v1/events -d '{\"type\":\"agent_response\",\"session\":\"$SESSION_ID\"}'",
    "timeout": 2000
  }]
}
```

### 5. Git worktree isolation (already built)
```bash
# Each agent gets its own worktree
forge-git worktree create agent-security-audit
cd /path/to/worktree
claude -p "..."
```

---

## What We Keep vs What We Drop

### Keep and enhance
| Component | Why |
|-----------|-----|
| **Persona catalog** | Claude Code doesn't have this. 112 role definitions is our unique value. |
| **Org structure** | Companies, departments, reporting chains — governance layer. |
| **Budget tracking** | Track real costs across agents, enforce limits. |
| **Approvals** | Gate certain actions (deploy, large changes) behind human approval. |
| **Orchestration** | Coordinate multiple Claude Code instances on one project. |
| **Event capture** | Observe what every agent does, build audit trail. |
| **MCP exposure** | Let upstream tools access all of the above. |
| **Web UI** | Dashboard for humans to monitor, configure, approve. |
| **SecurityScanner** | Adds a layer Claude Code doesn't have — OWASP pattern matching on output. |

### Drop or deprecate
| Component | Why |
|-----------|-----|
| **Skills injection middleware** | Let Claude Code use its own native skills. AgentForge writes the CLAUDE.md instead. |
| **TaskTypeDetector middleware** | Move to MCP tool (upstream calls it if needed). Don't inject silently. |
| **Memory repo** | Let Claude Code use its native memory. AgentForge reads it for observation. |
| **Hooks repo** | Configure Claude Code's native hooks instead of building our own. |
| **LoopDetector** | Claude Code handles this natively. |
| **Workflow engine** | Use Claude Code's Agent tool for sub-task orchestration. |

### Transform
| Component | From | To |
|-----------|------|-----|
| **SkillRouter** | Injects skills into prompt | Generates CLAUDE.md with relevant skills section |
| **Middleware chain** | 8-deep processing pipeline | Thinner: governance checks (budget, approval, rate limit) only |
| **Spawn** | Raw CLI spawn + JSON parsing | Configurable Claude Code launch with hooks for event reporting |
| **forge-mcp-bin** | 10 basic tools | 25+ tools covering full workforce API |

---

## Event-Driven Control Loop

The key pattern: AgentForge **configures** Claude Code instances, then **observes** them via hooks.

```
1. CONFIGURE
   AgentForge writes:
   - CLAUDE.md (persona + goals + constraints)
   - .claude/hooks.json (event reporting)
   - .claude/settings.json (allowed tools, model, limits)
   - Git worktree (isolation)

2. LAUNCH
   AgentForge spawns: claude -p "..." --resume SESSION_ID

3. OBSERVE (via hooks)
   Claude Code fires hooks → AgentForge receives:
   - Tool uses (what files read/written)
   - Cost updates (token usage)
   - Completion status
   - Errors

4. REACT
   AgentForge can:
   - Kill agent if over budget
   - Queue approval if change is large
   - Spawn follow-up agent (reviewer after writer)
   - Update dashboard in real time
   - Log everything for audit

5. REPORT
   AgentForge provides to upstream (via MCP):
   - Session results
   - Cost summary
   - Security scan findings
   - Approval status
```

---

## What This Means for the Codebase

### Middleware chain simplification
```
Current (8 deep):
  RateLimit → CircuitBreaker → CostCheck → SkillInjection
  → TaskTypeDetection → SecurityScan → Persist → Spawn

Future (4 deep):
  RateLimit → BudgetGate → Persist → ConfiguredSpawn

SkillInjection → moves to CLAUDE.md generation
TaskTypeDetection → moves to MCP tool (on-demand)
SecurityScan → moves to post-execution hook
```

### New components needed
| Component | Purpose |
|-----------|---------|
| **AgentConfigurator** | Generates CLAUDE.md, hooks.json, settings per persona |
| **HookReceiver** | HTTP endpoint that Claude Code hooks POST to |
| **CostAggregator** | Receives cost events, enforces budgets, updates company spend |
| **ApprovalGate** | Checks if pending approvals block this agent's action |
| **EventStream** | Real-time feed of all agent activity to Web UI and MCP clients |

---

## The Bottom Line

Claude Code is getting smarter every week. Instead of chasing its feature set, we build what it **can't do**:

- **Workforce identity** (who is this agent, what's their role)
- **Organizational governance** (budgets, approvals, scoping)
- **Multi-agent orchestration** (coordinate a team, not just one agent)
- **Observability** (what did every agent do, how much did it cost)
- **Composable access** (MCP tools for any upstream client)

AgentForge becomes the **management layer** on top of Claude Code, not a wrapper around it.
