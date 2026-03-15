# Middleware Pipeline

Every agent run passes through an ordered middleware chain. Each middleware can inspect, modify, or reject the request.

## Current Chain (7 middlewares)

```
Request → RateLimit → CircuitBreaker → CostCheck → Governance → SecurityScan → Persist → Spawn → Response
```

| # | Middleware | Purpose |
|---|-----------|---------|
| 1 | **RateLimit** | Token-bucket rate limiting. Rejects if no tokens available. |
| 2 | **CircuitBreaker** | 3-state FSM (Closed → Open → HalfOpen). Prevents cascading failures. |
| 3 | **CostCheck** | Pre-flight budget check against session cost and configured limits. |
| 4 | **Governance** | Company budget enforcement, goal injection into context, approval visibility. |
| 5 | **SecurityScan** | OWASP pattern matching on code output. Emits scan events. |
| 6 | **Persist** | Sets session to "running", updates to "completed"/"failed" after, emits lifecycle events. |
| 7 | **Spawn** | Configures workspace (CLAUDE.md + hooks.json via AgentConfigurator), spawns Claude CLI. |

## Evolution

The chain was simplified from 9 middlewares to 7:

- **Removed**: SkillInjection — replaced by AgentConfigurator writing skills into CLAUDE.md
- **Removed**: TaskTypeDetection — moved to on-demand MCP tool (`forge_classify_task`)

## Configure → Execute → Observe

The middleware chain is now focused on **governance**. Feature injection moved to file-based configuration:

```
AgentConfigurator                    Claude Code Instance
generates per persona:               reads at launch:
├── CLAUDE.md (identity, skills,  →  system instructions
│   goals, constraints)
├── .claude/hooks.json            →  event reporting hooks
└── worktree isolation            →  git worktree

HookReceiver                         Claude Code hooks:
captures events:                     POST back to AgentForge:
├── POST /hooks/pre-tool          ←  PreToolUse (budget check)
├── POST /hooks/post-tool         ←  PostToolUse (security scan, cost)
└── POST /hooks/stop              ←  Stop (session complete)
```

## RunContext

Data passed through the chain:

```rust
pub struct RunContext {
    pub agent_id: String,
    pub prompt: String,
    pub session_id: String,
    pub working_dir: Option<String>,
    pub metadata: HashMap<String, String>,
    pub agent_id_typed: AgentId,
    pub session_id_typed: SessionId,
    pub resume_session_id: Option<String>,
    pub directory: String,
}
```

Middlewares can add to `metadata`:

- `budget_warning` — company budget 90%+ used
- `company_goals` — active goals formatted as list
- `pending_approvals` — count of pending approvals
- `security_scan` — "passed" or "failed"
- `security_findings` — detailed findings if failed
