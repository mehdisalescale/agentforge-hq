# Revised Plan: Finish Smart, Then Evolve

> Incorporating Perplexity research. Resolves the sequencing tension.
> Date: 2026-03-15

---

## The Key Insight from Research

Don't just "finish what exists" — **finish only what survives the architecture transition**. Every hour spent wiring a feature that gets replaced by Claude Code native capabilities is wasted.

### What survives (invest here)
- Budget enforcement (governance stays, form doesn't change)
- Approval gating (governance stays, form doesn't change)
- Event schema + observability (core to everything)
- Skills visibility (read-only — don't build an editor)
- Session output storage (audit trail, always needed)
- Analytics dashboard (cost/run tracking, always needed)
- MCP tool surface (the future product IS the MCP layer)

### What gets replaced (don't invest here)
- Skills injection middleware → CLAUDE.md generation
- TaskTypeDetection middleware → MCP tool (on-demand, not auto)
- Memory repo UI → Claude Code native memory
- Hooks config UI → Claude Code native hooks
- Workflow designer → Claude Code Agent tool
- Schedule manager → lower priority

---

## Revised Waves (3 waves, not 4)

### Wave 1: Visibility + Governance Wiring
Make existing data visible and governance real. No new engines.

| Task | What | Effort | Survives transition? |
|------|------|--------|---------------------|
| **Skills page** | Read-only list of 30 loaded skills with markdown content | Medium | Yes — becomes "what intelligence we offer" |
| **Run metadata panel** | Show task type, injected skills, security scan result after run | Low | Partially — task type stays, skill injection changes form |
| **Session detail** | Store + render past output blocks on session page | Medium | Yes — audit trail |
| **Budget wiring** | Connect company budget → CostTracker → deduct per run | Medium | Yes — core governance |
| **Goal injection** | Inject active company goals into agent system prompt | Low | Yes — moves to CLAUDE.md later, same data |
| **Approval gating** | Block runs when approval required (budget threshold) | Medium | Yes — core governance |
| **Sidebar cleanup** | Remove Workflows, Memory, Hooks, Schedules from nav | Low | Yes |

**Agent count**: 4-5 parallel agents, 1 session
**Result**: 12 pages, all functional. Governance is real.

### Wave 2: Analytics + Settings + Event Schema
Make the system observable and configurable. Lock down the event model.

| Task | What | Effort | Survives transition? |
|------|------|--------|---------------------|
| **Event schema revision** | Align 37 event variants to audit-ready schema (actor, resource, cost, outcome) | Medium | Yes — foundation for everything |
| **Analytics dashboard** | Run counts, cost per agent/company, success rate, CSS bar charts | Medium | Yes — core observability |
| **Settings page** | Show/edit runtime config (rate limits, budget thresholds, CLI path) | Medium | Yes |
| **Agent cards enrichment** | Persona link, run count, last run, cost total | Low | Yes |
| **Health check** | Startup check for claude CLI, banner in UI if missing | Low | Yes |

**Agent count**: 4-5 parallel agents, 1 session
**Result**: Observable, configurable product. Event schema locked.

### Wave 3: MCP Expansion + Claude Code Integration
The transition wave. Start becoming infrastructure.

| Task | What | Effort | Survives transition? |
|------|------|--------|---------------------|
| **MCP tool expansion** | Add 15 tools to forge-mcp-bin (personas, governance, intelligence) | High | Yes — this IS the future product |
| **HTTP SSE transport** | Add HTTP endpoint alongside stdio in forge-app | Medium | Yes |
| **AgentConfigurator** | Generate CLAUDE.md + hooks.json per persona for Claude Code | High | Yes — replaces middleware injection |
| **HookReceiver** | HTTP endpoint for Claude Code hooks to POST events back | Medium | Yes — the nervous system |
| **Middleware simplification** | Drop SkillInjection, TaskTypeDetection from chain. Keep governance only. | Medium | Yes — the cleanup |

**Agent count**: 5 parallel agents, 1-2 sessions
**Result**: AgentForge is now infrastructure, not just a web app.

---

## What Gets Removed from Sidebar (Wave 1)

| Page | Action | Bring back when |
|------|--------|----------------|
| Workflows | Remove link, keep route with "Coming soon" | We build DAG orchestration or Claude Code's Agent tool integration |
| Memory | Remove link | We decide: expose Claude Code's memory via MCP, or build our own |
| Hooks | Remove link | HookReceiver is built and Claude Code hooks flow in |
| Schedules | Remove link | Someone needs cron agents and we build the UI |

**Sidebar after Wave 1: 12 links, all functional.**

---

## Event Schema (Lock Down in Wave 2)

Based on research, every event should carry:

```rust
pub struct ForgeEvent {
    pub id: EventId,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub actor: Actor,          // User, Agent(AgentId), System
    pub resource: Resource,    // Session, Company, File, Tool
    pub company_id: Option<CompanyId>,
    pub session_id: Option<SessionId>,
    pub cost: Option<CostInfo>,
    pub outcome: Outcome,      // Success, Failure(reason), Pending
    pub data: serde_json::Value,
}
```

This schema serves:
- Real-time dashboard (Analytics page)
- Audit trail (Sessions detail)
- MCP tools (forge_get_events, forge_get_session)
- Budget enforcement (cost attribution)
- Approval decisions (who did what, when)

---

## MCP Tool Surface (Wave 3)

Minimal coherent set for v1.0:

```
Workforce:
  forge_list_personas(division?, search?)     → persona list
  forge_get_persona(id)                       → full detail
  forge_hire_persona(persona_id, company_id)  → agent_id, position_id
  forge_list_agents(company_id?)              → agent list

Intelligence:
  forge_classify_task(prompt)                 → task_type, confidence
  forge_security_scan(code)                   → findings, passed
  forge_list_skills()                         → skill names + descriptions

Execution:
  forge_run_agent(agent_id, prompt, dir?)     → session_id
  forge_get_session(session_id)               → output, status, cost

Governance:
  forge_request_approval(company_id, type, data) → approval_id
  forge_check_approval(approval_id)              → status
  forge_get_budget(company_id)                   → remaining, used, limit

Observability:
  forge_get_events(session_id?, company_id?)  → event stream
  forge_get_analytics(company_id)             → run counts, costs
```

**15 tools. Both stdio and HTTP SSE transport. Same Rust handler.**

---

## Success Criteria for v1.0

- [ ] Every sidebar link leads to a functional page (12 pages)
- [ ] Budget enforcement is real (runs blocked when over limit)
- [ ] At least one approval type gates an action
- [ ] Goals are visible to agents during runs
- [ ] Skills page shows all loaded skills
- [ ] Session detail shows past output
- [ ] Analytics shows run counts and costs
- [ ] Settings shows runtime config
- [ ] Event schema is locked and auditable
- [ ] 15 MCP tools work over stdio
- [ ] HTTP SSE transport works for remote MCP clients
- [ ] AgentConfigurator generates CLAUDE.md per persona
- [ ] HookReceiver captures Claude Code events
- [ ] Middleware chain simplified to 4 (governance only)
- [ ] 280+ tests passing
- [ ] Zero warnings
- [ ] Can configure AgentForge as MCP server in Claude Code and use it

---

## Timeline Estimate

| Wave | Sessions | What you get |
|------|----------|-------------|
| Wave 1 | 1-2 | Working product, no fakes, governance real |
| Wave 2 | 1-2 | Observable, configurable, event schema locked |
| Wave 3 | 2-3 | Infrastructure product, MCP server, Claude Code integration |

**5-7 sessions from "demo with shells" to "infrastructure product."**

---

## Positioning Statement

> AgentForge: Rust-native AI workforce and governance layer on top of Claude Code and MCP. Not another agent runtime — the management layer that makes AI teams accountable, observable, and composable from any tool.
