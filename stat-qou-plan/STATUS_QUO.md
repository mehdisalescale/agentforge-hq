# AgentForge HQ — Status Quo

> Honest assessment of what exists, what works, and what's a shell.
> Date: 2026-03-15

---

## What Actually Works (End-to-End)

### 1. Agent Execution Pipeline
The core loop is real and functional:
- User picks an agent, types a prompt, hits Run
- 8-middleware chain processes the request (rate limit, circuit breaker, cost check, skill injection, task type detection, security scan, persist, spawn)
- `claude` CLI spawns as a subprocess with streaming JSON output
- Output flows back to browser via WebSocket in real time
- Sub-agent spawning shows swim-lane view
- Sessions are persisted to SQLite

### 2. Skills System
- 30 skills loaded at startup (10 core + 14 superpowers + 6 plugins)
- TaskTypeDetector classifies prompts into 6 categories (NewFeature, BugFix, CodeReview, Refactor, Research, General)
- SkillRouter auto-injects relevant methodology skills into agent system prompts
- Example: prompt "fix the login bug" → injects `systematic-debugging` + `test-driven-development` skills

### 3. Security Scanning
- SecurityScanner with 9 OWASP regex patterns (command injection, XSS, eval, SQL injection, path traversal, pickle, hardcoded secrets, insecure random, open redirect)
- Runs as post-execution middleware, scans output code blocks
- Emits SecurityScanPassed/Failed events

### 4. Persona Catalog + Hire Flow
- 112 personas across 11 divisions (engineering, product, design, marketing, etc.)
- Browse, search, filter by division
- Hire flow: pick persona → select company/department → creates agent + org position automatically
- Agent gets system prompt derived from persona description

### 5. Organization CRUD
- Companies: create, edit, delete (name, mission, budget)
- Departments: create, edit, delete (scoped to company)
- Org Chart: tree visualization of positions per company
- All fully functional CRUD with SQLite persistence

### 6. Governance CRUD
- Goals: create with parent/child hierarchy, update status (planned, in_progress, completed)
- Approvals: create requests, approve/reject with approver tracking
- Both scoped to company

### 7. Demo Seed
- First launch creates: Acme AI Corp, 2 departments, 4 agents, 3 goals, 1 pending approval
- Every page shows real data immediately

---

## What's a Shell (UI exists, nothing behind it)

| Page | What exists | What's missing |
|------|-------------|----------------|
| **Workflows** | Empty page with "no workflows" | No workflow engine, no DAG execution, no step chaining |
| **Schedules** | Empty page | Cron scheduler exists in backend but no UI to create/manage schedules |
| **Memory** | Empty page | MemoryRepo exists but agents can't read/write memory during runs |
| **Hooks** | Empty page | HookRepo exists but no event triggers wired |
| **Analytics** | Empty page | AnalyticsRepo collects some data but no dashboard renders it |
| **Settings** | Empty page | No configuration UI for env vars, safety thresholds, etc. |
| **Skills** | Empty page | 30 skills loaded but no UI to browse, edit, or assign them |
| **Sessions** | Lists past sessions | Can't replay or inspect output (just metadata) |

---

## What's Connected vs Disconnected

### Connected (affects agent behavior)
```
Skills ──→ SkillInjection MW ──→ Agent system prompt (auto)
TaskType ──→ SkillRouter ──→ Methodology injection (auto)
SecurityScan ──→ Output scanning ──→ Events (auto)
RateLimit ──→ Blocks runs when exceeded
CircuitBreaker ──→ Blocks runs after failures
CostTracker ──→ Warns/blocks at budget thresholds (env vars only)
```

### Disconnected (exists but doesn't affect anything)
```
Company budget ──✗── CostTracker (not linked)
Goals ──✗── Agent behavior (no scoping)
Approvals ──✗── Agent runs (no blocking)
Org hierarchy ──✗── Permissions (no scoping)
Memory ──✗── Agent context (not injected)
Hooks ──✗── Events (not triggered)
Schedules ──✗── Agent runs (backend exists, no UI)
Workflows ──✗── Anything (no engine)
```

---

## Architecture Inventory

### Crates (13)
| Crate | LOC (approx) | Maturity |
|-------|-------------|----------|
| forge-core | ~600 | Solid — 35 event types, error hierarchy, typed IDs |
| forge-agent | ~400 | Solid — 10 presets, validation, CRUD models |
| forge-db | ~2000 | Solid — 16 repos, 12 migrations, BatchWriter |
| forge-process | ~800 | Solid — spawn, stream parser, ConcurrentRunner, LoopDetector |
| forge-safety | ~500 | Solid — CircuitBreaker, RateLimiter, CostTracker, SecurityScanner |
| forge-api | ~1500 | Solid — full HTTP API, WebSocket, 8-middleware chain |
| forge-app | ~300 | Solid — binary wiring, startup, shutdown |
| forge-git | ~200 | Solid — worktree create/remove/list |
| forge-org | ~200 | Solid — Company, Department, OrgPosition models |
| forge-persona | ~300 | Solid — parser, catalog, hire models |
| forge-governance | ~100 | Thin — Goal, Approval models only |
| forge-mcp | ~50 | Stub |
| forge-mcp-bin | ~300 | Working — rmcp, 10 tools |

### Frontend (16 pages)
| Page | Status |
|------|--------|
| Run (Dashboard) | Fully functional |
| Agents | Fully functional |
| Sessions | Partial (list only) |
| Workflows | Shell |
| Companies | Fully functional |
| Personas | Fully functional |
| Org Chart | Fully functional |
| Goals | Fully functional |
| Approvals | Fully functional |
| Skills | Shell |
| Memory | Shell |
| Hooks | Shell |
| Schedules | Shell |
| Analytics | Shell |
| Settings | Shell |

### Tests
- **229 tests passing**, zero warnings
- Good coverage on core, db, process, safety, api
- Thin coverage on org, persona, governance

---

## Key Metrics
- **Crates**: 13
- **Frontend pages**: 16 (6 functional, 4 partial, 6 shell)
- **DB migrations**: 12
- **DB repos**: 16
- **Middleware chain**: 8 deep
- **ForgeEvent variants**: 37 (35 original + 2 security scan)
- **Personas**: 112 across 11 divisions
- **Skills**: 30 loaded
- **Tests**: 229 passing
- **Binary size**: single binary, zero runtime deps
