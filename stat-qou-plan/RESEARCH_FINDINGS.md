# Research Findings — Perplexity Pro Analysis

> External research validating and refining AgentForge direction.
> Date: 2026-03-15
> Source: Perplexity Pro deep research against all 7 planning docs

---

## Verdict: Direction is Right

AgentForge's overall direction (Rust-native orchestrator on top of Claude Code, governance-first middleware, future MCP server) is **well aligned with where the ecosystem is going**. The biggest leverage in v1.0 comes from observability, budget/approval wiring, and MCP exposure — not more "agent magic" inside our own runtime.

---

## Key Findings by Topic

### 1. MCP Server Architecture
- **Pattern**: "Shared core, pluggable transports" — one JSON-RPC handler, wrapped by separate stdio and HTTP/SSE frontends
- **Recommendation**: Keep Axum server as HTTP/SSE host, run rmcp stdio server as thin adapter forwarding into same internal EventBus
- **Reference crates**: `async-mcp`, `mcpr` show this pattern
- **Each MCP session** = internal struct tied to EventBus, so both transports see identical behavior

### 2. Claude Code Hooks for Orchestration
- Hooks are explicitly designed for event-driven orchestration
- **Available events**: `PreToolUse`, `PostToolUse`, `SessionStart`, `UserPromptSubmit`, `Notification`, `Stop`
- **Use PreToolUse** for governance: budget checks, approval gates, org scoping
- **Use PostToolUse** for observability: log file edits, commands, test runs → EventBus
- **Use Stop** for post-run approvals and session state updates
- Hooks run synchronously, HTTP hooks have clear timeout/retry — suitable for production
- **Key insight**: "Claude Code is the engine, EventBus + hooks are the nervous system"

### 3. Multi-Agent Coordination Patterns
Three patterns emerging across CrewAI, LangGraph:
1. **Hierarchical manager-worker**: Manager assigns, workers execute, only talk via orchestrator
2. **Graph-based with shared state**: LangGraph's approach — structured state object, not raw prompts
3. **Shared-state for codebases**: Worktrees per agent (our approach), plus "merge agent" concept
- **Suggestion**: Add explicit "manager persona + merge persona" concept and typed Task/GraphState model

### 4. Competitive Landscape
| Competitor | Approach | Our differentiator |
|-----------|----------|-------------------|
| CrewAI | Python framework, hierarchical crews | Rust single binary, org model, governance |
| LangGraph | Graph-based orchestration over LangChain | Not a Python lib, native org semantics |
| OpenHands/SWE-agent | Autonomous coding, tight model coupling | Multi-tenant governance, personas |
| Devin | Full "AI employee" SaaS | Open source, composable MCP surface |

**Position**: "Rust-native AI workforce and governance layer on top of Claude Code and MCP, not another agent runtime."

### 5. MCP Ecosystem (2026)
- MCP now under Linux Foundation, adopted by OpenAI and Google
- Claude Code, Cursor, VS Code extensions all act as MCP clients
- AI gateways treat MCP as first-class integration surface
- **An MCP-first workforce platform is still a gap in the market**

### 6. Budget & Cost Tracking
- Attribution dimensions: user, team/project, environment, model/provider, workflow
- **Enforcement**: hard caps with pre-run checks + soft caps with approval-for-overage
- **Recommendation**: Wire CostTracker to company/department budgets, pre-authorize every run, record by agent/persona/company/model

### 7. Agent Persona Best Practices
Effective personas are **concrete and operational**, not just vibes:
- Clear role + scope + explicit out-of-scope behaviors
- Constraints and tool permissions
- Step-by-step procedures/methodologies
- Alignment with real org structure
- **Recommendation**: Make persona definitions visible/editable (the CLAUDE.md), measure per-persona performance

### 8. Approval Workflows
Approvals are most effective when tied to **business risk**, not raw LLM calls:
- Per-action: high-risk tools, large diffs, budget thresholds, regulated data
- Per-workflow: approve a "run plan", then agent executes within that envelope
- Multi-level: manager + finance/security for sensitive flows
- **Recommendation**: Pre-run (authorize scope) + post-run (accept/revert changes) approvals

### 9. Event-Driven Observability
- Core entities: session, run/step, tool call, model invocation, user interaction, approval
- Each event: timestamps, actor, resource, cost, outcome
- Feed both real-time dashboards AND long-term audit trails
- **Recommendation**: Lock down event schema as long-term audit format, retrofit Analytics/Sessions/MCP onto it

### 10. Deployment
- Single static binary + minimal Docker image
- Stdio: launched by client directly
- Hosted: HTTPS with SSE/WebSocket, API key auth
- Per-company API keys, org scoping in every MCP handler
- Rate limiting at HTTP layer wrapping MCP handlers

---

## Structural Tension Identified

Perplexity flagged the sequencing dependency between our three threads:

> Thread 3 (architecture rethink) **changes what you keep** — you'd drop SkillInjection and TaskTypeDetection middleware, which are currently working features.
> Thread 1 (finish first) says "wire goals, approvals, budgets" — but Thread 3 says some of that wiring should be replaced by CLAUDE.md generation.
> Thread 2 (MCP server) depends on having a coherent product to expose.

**Implied correct order: 1 → then 2 and 3 in parallel.**

But with a refinement: during Thread 1, **build things that survive the Thread 3 transition**:
- Budget wiring → survives (governance stays)
- Approval gating → survives (governance stays)
- Goal injection → survives but changes form (goes into CLAUDE.md instead of middleware)
- Skills page (read-only) → survives (visibility)
- Session detail → survives (observability)
- Analytics → survives (observability)

What to **NOT** invest heavily in during Thread 1:
- Don't build a skills editor UI (Claude Code manages its own skills)
- Don't build a memory editor UI (Claude Code has native memory)
- Don't build a hooks config UI (use Claude Code's hooks.json)
- Don't build a workflow designer (use Claude Code's Agent tool)

---

## Revised v1.0 Priority (from research)

1. **Visibility + Cost** (Wave A/B): Skills UI, session detail, run metadata, Analytics wired to EventBus, company budgets integrated with CostTracker
2. **Governance** (Wave C): Goals + org data + approvals integrated with Claude Code hooks (Pre/PostToolUse) for high-risk action gating
3. **MCP surface** (Wave D): Minimal coherent MCP tools (`forge_list_personas`, `forge_run_agent`, `forge_get_session`, `forge_request_approval`) over stdio AND HTTP SSE from same Rust core
4. **Event schema** (throughout): Lock down event schema as long-term audit format

---

## Next Research Questions (for follow-up)

1. How does AgentForge stack up against CrewAI and LangGraph in multi-agent orchestration?
2. What are the specific JSON schemas for MCP tools that map to our Rust types?
3. Concrete Claude Code hooks.json examples for budget enforcement
4. Event schema standards emerging in the AI agent observability space
