# Multi-Agent Development

AgentForge is built using parallel AI agents. Here's how the process works.

## The Pattern

1. Write **agent briefs** — markdown files with full instructions, file ownership, and report templates
2. Design a **conflict matrix** — ensure zero file overlaps between agents
3. Launch agents in **parallel Claude Code tabs** — each reads its brief and executes
4. Agents **write reports** back to their brief files and commit
5. **Verify** — cargo check, cargo test, pnpm build

## Conflict Matrix Example

From Wave 5:

| File | W5-A | W5-B | W5-C |
|------|------|------|------|
| `forge-mcp-bin/src/main.rs` | MODIFY | — | — |
| `routes/analytics.rs` | — | MODIFY | — |
| `routes/agents.rs` | — | — | MODIFY |
| `routes/health.rs` | — | — | MODIFY |
| `analytics/+page.svelte` | — | MODIFY | — |
| `agents/+page.svelte` | — | — | MODIFY |

**Zero conflicts = safe parallel execution.**

## Agent Brief Structure

```markdown
# Agent W5-A: [Title]

> One-line role description

## Step 1: Read Context
[List of files to read first]

## Step 2-N: Implementation Steps
[Concrete instructions with code examples]

## Rules
- Files this agent MAY modify
- Files this agent must NOT touch
- Commit message format

## Report
STATUS: pending
FILES_MODIFIED: []
ISSUES: []
```

## Wave History

| Wave | Agents | What |
|------|--------|------|
| 1-2 | 4 each | Skills, task detection, security scanner, backend traits |
| 3 | 4 | Sidebar cleanup, governance wiring, session detail, page verification |
| 4 | 3 | AgentConfigurator, HookReceiver, MCP tool expansion |
| 5 | 3 | Analytics enrichment, agent stats, health check |

## Launch Prompt Template

```
You are Agent W[N]-[X]. Read docs/agents/AGENT_W[N][X]_[NAME].md — it contains
your full instructions. Execute every step. Verify with cargo check. When done,
append your report to the file and commit.
```
