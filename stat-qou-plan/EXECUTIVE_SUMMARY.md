# AgentForge HQ — Executive Summary

> Where we started, what we found, where we're going.
> Date: 2026-03-15

---

## Where We Started

A Rust + Svelte 5 single-binary platform with 13 crates, 16 frontend pages, 112 AI personas, 30 skills, and a working execution pipeline that spawns Claude Code CLI. Built across multiple agent waves with 229 tests passing.

On paper: a full AI workforce platform.

---

## What We Found

**The product is 50% real, 50% facade.**

- **8 pages work**: Run, Agents, Companies, Personas, Org Chart, Goals, Approvals, Sessions (partial)
- **8 pages are empty shells**: Skills, Memory, Hooks, Workflows, Schedules, Analytics, Settings, Sessions (detail)
- **Smart features are invisible**: 30 skills auto-inject based on task type, security scanner runs on every output — but users can't see any of this happening
- **Governance is decorative**: Budgets don't enforce, approvals don't block, goals don't influence agents
- **We duplicated Claude Code**: Our skills, memory, hooks, and loop detection replicate what Claude Code already does natively — and does better

**The honest product is: pick agent → type prompt → claude runs → see output. Everything else is scaffolding.**

---

## What We Decided

### Identity shift
AgentForge is not another agent runtime. It's the **management and governance layer** on top of Claude Code. We don't compete with the engine — we manage the workforce.

**Our moat**: 112 curated personas, organizational governance (budgets, approvals, org charts), multi-agent orchestration, and observability. Exposed as MCP tools so any AI tool in the ecosystem can use it.

### What to keep
- Persona catalog (unique, nobody else has this)
- Org structure (companies, departments, reporting chains)
- Budget enforcement (real cost tracking, not decorative)
- Approval gating (actually blocks actions)
- Event system + observability (audit trail)
- Security scanner (adds value Claude Code doesn't have)
- MCP server (the future product surface)

### What to drop
- Skills injection middleware (let Claude Code use its own skills)
- TaskTypeDetection middleware (move to on-demand MCP tool)
- Memory UI (Claude Code has native memory)
- Hooks UI (configure Claude Code's native hooks instead)
- Workflow designer (use Claude Code's Agent tool)
- LoopDetector (Claude Code handles this)

### What to build
- AgentConfigurator (generate CLAUDE.md per persona)
- HookReceiver (capture Claude Code events back)
- 15 MCP tools (workforce + intelligence + governance)
- HTTP SSE transport (remote MCP access)

---

## The Plan: 3 Waves

### Wave 1 — Make It Real
Wire governance (budgets enforce, approvals block, goals inject). Skills page (read-only). Session detail (view past output). Remove 4 shell pages from sidebar. **Result: 12 pages, all functional, zero fakes.**

### Wave 2 — Make It Observable
Analytics dashboard (run counts, costs). Settings page. Event schema locked to audit-ready format. Agent cards enriched. Health checks. **Result: observable, configurable product.**

### Wave 3 — Make It Infrastructure
Expand MCP tools to 15. Add HTTP SSE transport. Build AgentConfigurator (CLAUDE.md per persona). Build HookReceiver (Claude Code events flow back). Simplify middleware to governance-only. **Result: any AI tool can use AgentForge as its workforce layer.**

---

## End State

```
Before:  A web app with 16 pages, half empty
After:   AI workforce infrastructure accessible from any MCP client

Before:  Duplicates Claude Code features poorly
After:   Orchestrates Claude Code instances professionally

Before:  Governance is decorative
After:   Budgets enforce, approvals block, everything is auditable

Before:  Only usable in a browser
After:   Usable from Claude Code, Cursor, ADK, any MCP client
```

---

## One-Liner

**AgentForge: Rust-native AI workforce and governance layer on top of Claude Code and MCP. Not another agent runtime — the management layer that makes AI teams accountable, observable, and composable from any tool.**
