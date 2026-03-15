# Architecture Rethink: Claude Code IS the Engine

> Stop building what Claude Code already does. Orchestrate, govern, observe.

## The Realization

AgentForge spawns `claude` CLI as its execution backend. But we were **duplicating** things Claude Code already does and does better:

| We Built | Claude Code Already Has |
|----------|----------------------|
| 30 skills (markdown injection) | Native skills system |
| Memory repo (empty) | Built-in memory |
| Hooks repo (empty) | Native hooks (pre/post tool use) |
| LoopDetector | Built-in loop detection |
| Sub-agent tracking | Native subagent spawning |
| Session management | Session resume, conversation history |

**We were reinventing wheels that evolve faster than we can copy them.**

## The Shift: Orchestrate, Don't Replicate

### What AgentForge Does NOT Do

- Build its own skill system
- Build its own memory system
- Build its own hook system
- Build its own workflow engine
- Parse Claude Code's output format

### What AgentForge DOES Do

- **Workforce management** — who works on what
- **Governance** — budgets, approvals, permission scoping
- **Orchestration** — spawn, configure, coordinate Claude Code instances
- **Observation** — capture events, track costs, audit decisions
- **Configuration** — set up Claude Code instances with the right CLAUDE.md, tools, hooks
- **Exposure** — make all of this available as MCP tools

## Configure → Execute → Observe

```
1. CONFIGURE
   AgentForge writes: CLAUDE.md, hooks.json, settings
   Per persona: identity, skills, goals, constraints

2. LAUNCH
   AgentForge spawns: claude -p "..." in isolated worktree

3. OBSERVE (via hooks)
   Claude Code fires hooks → AgentForge receives:
   - Tool uses, cost updates, completion status, errors

4. REACT
   AgentForge can: kill over-budget agents, queue approvals,
   spawn follow-up agents, update dashboard, log for audit

5. REPORT
   AgentForge provides to upstream (via MCP):
   - Session results, cost summary, security findings
```

## Middleware Simplification

```
Before (9 middlewares):
  RateLimit → CircuitBreaker → CostCheck → Governance
  → SkillInjection → TaskTypeDetection → SecurityScan → Persist → Spawn

After (7 middlewares):
  RateLimit → CircuitBreaker → CostCheck → Governance
  → SecurityScan → Persist → Spawn

SkillInjection    → moved to CLAUDE.md generation
TaskTypeDetection → moved to MCP tool (on-demand)
```

## The Bottom Line

Claude Code gets smarter every week. Instead of chasing its feature set, we build what it **can't do**: workforce identity, organizational governance, multi-agent orchestration, observability, and composable MCP access.
