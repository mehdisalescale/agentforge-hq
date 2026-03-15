# AgentForge as Downstream MCP Server

> The shift from standalone web app to AI workforce infrastructure.
> Date: 2026-03-15

---

## The Idea

AgentForge stops being just a browser app. It becomes a **service layer** that any MCP-compatible AI tool can call. Claude Code, Cursor, ADK apps, Antigravity, Cowork — they all get access to AgentForge's personas, skills, security scanning, org structure, and governance.

The web UI remains, but it becomes **one client** among many. The real product is the API.

---

## Architecture Shift

### Before (standalone app)
```
Browser → Axum API → SQLite
                   → claude CLI
```

### After (MCP infrastructure)
```
┌─────────────────────────────────────────┐
│           UPSTREAM CLIENTS              │
│                                         │
│  Claude Code     (stdio MCP)            │
│  Cursor          (stdio MCP)            │
│  ADK apps        (HTTP SSE MCP)         │
│  Antigravity     (HTTP SSE MCP)         │
│  Cowork          (HTTP SSE MCP)         │
│  Custom scripts  (HTTP SSE MCP)         │
│  Web UI          (HTTP REST + WS)       │
└────────────┬────────────────────────────┘
             │
┌────────────▼────────────────────────────┐
│         AGENTFORGE SERVER               │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │  MCP Tool Layer (rmcp)          │    │
│  │  25+ tools exposed              │    │
│  └──────────────┬──────────────────┘    │
│                 │                        │
│  ┌──────────────▼──────────────────┐    │
│  │  Core Services                  │    │
│  │                                 │    │
│  │  PersonaCatalog (112 personas)  │    │
│  │  SkillLibrary (30+ skills)     │    │
│  │  TaskClassifier                │    │
│  │  SecurityScanner               │    │
│  │  ReviewEngine                  │    │
│  │  OrgManager                    │    │
│  │  GovernanceGate                │    │
│  │  BudgetTracker                 │    │
│  │  MemoryStore                   │    │
│  │  SessionManager                │    │
│  └──────────────┬──────────────────┘    │
│                 │                        │
│  ┌──────────────▼──────────────────┐    │
│  │  Persistence (SQLite WAL)       │    │
│  │  EventBus (broadcast)           │    │
│  │  BatchWriter (durability)       │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

---

## MCP Tools to Expose

### Persona & Workforce
| Tool | Input | Output | Value |
|------|-------|--------|-------|
| `forge_list_personas` | `division?`, `search?` | Persona list with descriptions | Browse 112 specialists from any tool |
| `forge_get_persona` | `id` or `slug` | Full persona detail | Read skills, deliverables, workflow |
| `forge_hire_persona` | `persona_id`, `company_id`, `dept?` | `{ agent_id, position_id }` | One call to create configured agent |
| `forge_list_agents` | `company_id?` | Agent list with presets | See available workforce |

### Intelligence
| Tool | Input | Output | Value |
|------|-------|--------|-------|
| `forge_classify_task` | `prompt` | `{ task_type, confidence }` | Know what kind of work this is |
| `forge_get_skills_for_task` | `prompt` or `task_type` | `{ skills: [...] }` | Get methodology recommendations |
| `forge_security_scan` | `code` | `{ findings: [...], passed }` | OWASP scan from any tool |
| `forge_code_review` | `code` or `diff` | `{ aspects: [...], confidence }` | 6-aspect review from any tool |

### Execution
| Tool | Input | Output | Value |
|------|-------|--------|-------|
| `forge_run_agent` | `agent_id`, `prompt`, `dir?` | `{ session_id }` | Execute agent from any context |
| `forge_get_session` | `session_id` | Output blocks, status, metadata | Inspect results |
| `forge_list_sessions` | `agent_id?`, `status?` | Session list | Browse history |

### Organization & Governance
| Tool | Input | Output | Value |
|------|-------|--------|-------|
| `forge_create_company` | `name`, `mission?`, `budget?` | Company | Set up org from CLI |
| `forge_get_org_chart` | `company_id` | Tree structure | Understand workforce |
| `forge_create_goal` | `company_id`, `title`, `desc?` | Goal | Set objectives |
| `forge_request_approval` | `company_id`, `type`, `data` | Approval (pending) | Governance from any tool |
| `forge_check_approval` | `approval_id` | Status, approver | Poll for decisions |

### Skills & Memory
| Tool | Input | Output | Value |
|------|-------|--------|-------|
| `forge_list_skills` | `category?` | Skill names + descriptions | Browse skill library |
| `forge_get_skill` | `name` | Full skill content | Read methodology |
| `forge_store_memory` | `key`, `content`, `type?` | Memory ID | Persist learnings |
| `forge_recall_memory` | `query` | Relevant memories | Retrieve context |

---

## Transport Options

### Currently: stdio only (forge-mcp-bin)
- Works for Claude Code (spawns subprocess)
- Works for Cursor (same model)
- **Limitation**: Only local, one client at a time

### Needed: HTTP SSE transport
- Remote clients can connect
- Multiple concurrent clients
- Shared state (all clients see same personas, sessions, etc.)
- rmcp supports this — needs wiring

### Needed: Streamable HTTP (MCP 2025-03 spec)
- Latest MCP spec uses streamable HTTP instead of SSE
- Single HTTP endpoint, bidirectional
- Better for serverless/cloud deployment

---

## Use Cases

### 1. Claude Code + AgentForge
```bash
# In ~/.claude.json, add AgentForge as MCP server
# Then from Claude Code:

"Use forge to hire a security auditor and scan auth.rs"
→ Claude Code calls forge_hire_persona + forge_security_scan
→ Gets findings back in the conversation

"What kind of task is 'fix the race condition in the connection pool'?"
→ Claude Code calls forge_classify_task
→ Returns BugFix + recommended skills

"Run the code reviewer on my last commit"
→ Claude Code calls forge_code_review with git diff
→ Gets 6-aspect review with confidence scores
```

### 2. Cursor + AgentForge
```
# Configure as MCP server in Cursor settings
# Right-click → "AgentForge: Review this file"
→ Calls forge_code_review
→ Shows findings inline

# "AgentForge: What persona should work on this?"
→ Calls forge_classify_task + forge_list_personas
→ Recommends matching persona
```

### 3. ADK App + AgentForge
```python
# An ADK orchestrator app uses AgentForge as workforce backend
client = MCPClient("http://localhost:4173/mcp")

# Build a team
company = client.call("forge_create_company", name="Project X")
client.call("forge_hire_persona", persona="backend-engineer", company_id=company.id)
client.call("forge_hire_persona", persona="qa-engineer", company_id=company.id)

# Set objectives
client.call("forge_create_goal", company_id=company.id, title="Ship v2.0 by April")

# Execute work
session = client.call("forge_run_agent", agent_id=backend.id, prompt="implement the new API")
```

### 4. CI/CD Pipeline
```yaml
# GitHub Actions step
- name: Security scan
  run: |
    echo '{"tool": "forge_security_scan", "code": "$(cat src/main.rs)"}' | \
    forge-mcp-bin --stdio
```

---

## Implementation Priority

### Phase 1: Expand MCP tools (use existing forge-mcp-bin)
- Add 15 new tools wrapping existing repos/services
- Keep stdio transport
- Test with Claude Code locally
- **Effort**: Medium — mostly wiring, services exist

### Phase 2: HTTP SSE transport
- Add HTTP endpoint to forge-app alongside Axum REST API
- rmcp supports this with feature flags
- Multiple clients can connect simultaneously
- **Effort**: Medium — rmcp has the transport, need wiring

### Phase 3: Web UI becomes MCP client
- Frontend calls same tools as external clients
- Dogfood the API — if it's good enough for Claude Code, it's good enough for the UI
- Remove duplicate REST routes where MCP tools cover the same ground
- **Effort**: High — architectural shift

### Phase 4: Remote deployment
- AgentForge runs as a service (Docker, cloud)
- Teams share one instance
- Auth layer (API keys, team scoping)
- **Effort**: High — needs auth, multi-tenancy

---

## What This Changes

| Aspect | Before | After |
|--------|--------|-------|
| **Identity** | Web app with 16 pages | AI workforce infrastructure |
| **Users** | Humans in browser | Humans + AI tools + scripts |
| **Reach** | One user, one browser | Every MCP-compatible tool |
| **Moat** | 112 personas + Rust speed | Composable workforce API that other tools can't replicate |
| **Revenue** | None (open source) | SaaS potential (hosted AgentForge, per-seat/per-run pricing) |
| **Shell pages** | Empty, misleading | Backed by real services that external clients also use |

---

## Existing MCP Tools (forge-mcp-bin, 10 tools)

The current MCP server already has tools but they're basic process management:
- `run_agent`, `list_agents`, `create_agent`
- `list_sessions`, `get_session`
- `list_skills`
- `list_workflows`
- `system_status`
- `list_memories`
- `search_memories`

These need to be expanded with the persona, intelligence, governance, and review tools listed above.

---

## Key Insight

AgentForge's real value isn't the web UI — it's the **curated workforce** (112 personas), the **intelligence layer** (task classification, skill routing, security scanning, code review), and the **governance model** (companies, budgets, approvals, org charts).

Exposing these as MCP tools means every AI tool in the ecosystem gets instant access to a managed, governed, intelligent workforce. That's the product.
