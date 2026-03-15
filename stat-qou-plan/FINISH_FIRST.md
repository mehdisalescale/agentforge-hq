# Finish First: Wire Everything Before Evolving

> Make every page real. Remove what we won't finish. Then evolve.
> Date: 2026-03-15

---

## Principle

Don't add new capabilities. Make existing ones actually work. Every sidebar link should do something real or be removed. No shells, no stubs, no decorative pages.

---

## Current Reality: 16 pages, 6 work

| Page | Status | Action needed |
|------|--------|---------------|
| Run | Works | Minor: show what middleware did |
| Agents | Works | Minor: show persona link, run count |
| Companies | Works | Minor: wire budget to actual spend |
| Personas | Works | None |
| Org Chart | Works | None |
| Goals | Works | Wire: inject into agent context |
| Approvals | Works | Wire: actually gate actions |
| Sessions | Partial | Wire: show past output, not just metadata |
| Skills | Shell | Wire: show loaded skills, read-only |
| Memory | Shell | Decision: build or remove |
| Hooks | Shell | Decision: build or remove |
| Workflows | Shell | Decision: build or remove |
| Schedules | Shell | Decision: build or remove |
| Analytics | Shell | Wire: render existing data |
| Settings | Shell | Wire: show/edit env config |

---

## Decision: What Stays, What Goes

### Keep and wire (aligned with future direction)
These exist in backend AND are useful in the orchestrator future:

| Page | Why keep | Work needed |
|------|----------|-------------|
| **Skills** | Shows what intelligence we have. Future: configurable per persona. | Build read-only page listing 30 loaded skills with content |
| **Sessions** | Audit trail. Core to observation. | Store output blocks, render on detail page |
| **Analytics** | Cost visibility. Core to governance. | Aggregate run counts, costs, render dashboard |
| **Settings** | Runtime config. Needed for deployment. | Show env vars, allow safe edits |
| **Approvals** | Governance. Wire to actually block. | Add approval check before spawn |

### Remove from sidebar (not ready, misleading)
These need engines that don't exist yet. Showing empty pages hurts trust:

| Page | Why remove | When to bring back |
|------|------------|-------------------|
| **Workflows** | No execution engine. Can't fake it. | When we build DAG orchestration |
| **Memory** | Claude Code has native memory. Don't duplicate. | When we decide our memory strategy |
| **Hooks** | Claude Code has native hooks. Our hook system adds nothing yet. | When we build the HookReceiver for Claude Code events |
| **Schedules** | Backend scheduler exists but no UI. Low value right now. | When someone actually needs cron agents |

**Result: 12 pages, all functional. Zero shells.**

---

## Wiring Plan (4 waves, each shippable)

### Wave A: Visibility (make hidden things visible)
No backend changes. Just show what already exists.

**A1. Skills page** — list all 30 loaded skills with name, content (markdown), category
- Read from existing `skill_repo.list()`
- Read-only grid with expandable cards
- Shows users what intelligence the system has

**A2. Run metadata panel** — after a run, show what happened
- Task type detected (BugFix, NewFeature, etc.)
- Skills injected (which ones)
- Security scan result (passed/failed)
- Read from `ctx.metadata` already set by middleware

**A3. Agent cards enrichment** — show persona source, run count
- Display `persona_id` link if agent was hired from catalog
- Show session count per agent
- Show last run timestamp

**A4. Sessions detail page** — view past output
- Store output blocks in session metadata (already partially there)
- Render output on click (same component as Run page)
- Search/filter sessions

### Wave B: Budget & Cost (make governance real)

**B1. Wire company budget to CostTracker**
- On run, look up company budget from agent's company
- Set CostTracker warn/limit from company budget_limit
- Deduct estimated cost from budget_used after run
- Budget bar on company card shows real spend

**B2. Analytics dashboard**
- Total runs per agent (query sessions table)
- Total cost per company (query cost events)
- Run success/failure rate
- Simple bar charts (no charting library — CSS bars)

**B3. Settings page**
- Show current env config (host, port, CLI command, rate limits)
- Allow editing safe values (rate limit, budget thresholds)
- Persist to a config table or .env file

### Wave C: Governance (make approvals and goals real)

**C1. Goal injection**
- When agent runs, fetch active goals for its company
- Inject into system prompt: "Active company goals: [list]"
- Agent now knows what the org is trying to achieve

**C2. Approval gating**
- Add `requires_approval` flag to certain action types
- Before spawn, check for pending approvals on this agent/company
- If approval required and not granted, return error with link to approvals page
- Start simple: budget increases require approval

**C3. Post-run approval**
- After agent produces output with file changes > N files
- Auto-create approval request: "Agent X wants to modify 15 files"
- Block next run until approved

### Wave D: Cleanup & Polish

**D1. Remove shell pages from sidebar**
- Remove Workflows, Memory, Hooks, Schedules links
- Keep the route files (so direct URLs don't 404) but show "Coming soon" with context

**D2. Onboarding update**
- Update quickstart to reflect real capabilities
- Remove references to features that aren't ready

**D3. Error handling**
- Health check on startup: is `claude` CLI in PATH?
- Show banner if not configured
- Better error messages on run failure

**D4. Test coverage**
- Add tests for new wiring (budget, goals injection, approval gate)
- Target: 250+ tests

---

## Wave Execution Model

Same as before: write agent briefs, launch in parallel tabs, verify.

| Wave | Agents | Estimated effort | Dependencies |
|------|--------|-----------------|--------------|
| **A** | 4 agents (A1-A4) in parallel | 1 session | None |
| **B** | 3 agents (B1-B3) in parallel | 1 session | A complete |
| **C** | 3 agents (C1-C3), C2 after C1 | 1 session | B complete |
| **D** | 4 agents (D1-D4) in parallel | 1 session | C complete |

**4 sessions to go from "6 real pages" to "12 real pages, all wired."**

---

## After "Finish First": The Evolution

Once every page works and the product is coherent:

1. **Expand MCP tools** (expose finished features to upstream clients)
2. **Claude Code native integration** (CLAUDE.md generation per persona, hook-based event capture)
3. **HTTP transport** (remote MCP access)
4. **Bring back removed pages** one at a time, each backed by real functionality

The key: **evolve from a working product, not from scaffolding.**

---

## Success Criteria

Before declaring "finished":
- [ ] Every sidebar link leads to a functional page
- [ ] Skills page shows all 30 loaded skills
- [ ] Sessions page shows past output (not just metadata)
- [ ] Run page shows task type, injected skills, security result
- [ ] Company budget reflects actual agent spend
- [ ] Analytics shows run counts and costs
- [ ] Settings shows current config
- [ ] Goals are injected into agent context
- [ ] At least one approval type actually blocks an action
- [ ] Shell pages removed from sidebar
- [ ] 250+ tests passing
- [ ] Zero warnings
