# AgentForge HQ — App Inspection Notes

> Walk through every page, document what a user sees.
> Date: 2026-03-15

---

## Onboarding (first visit)

**What user sees**: Compact card with 3 steps (Create company, Hire personas, Run agent) + "Also available" chip grid with all 12 other features. Dismiss button.

**Verdict**: Clear, actionable, not bloated. Good.

---

## Sidebar Navigation (always visible)

4 groups, 16 links:
- **Workspace**: Run, Agents, Sessions, Workflows
- **Organization**: Companies, Personas, Org Chart, Goals, Approvals
- **Configuration**: Skills, Memory, Hooks, Schedules
- **Insights**: Analytics, Settings

**Issue**: 6 of these lead to empty/non-functional pages. User explores, finds nothing, loses confidence.

---

## Page-by-Page Inspection

### Run (Dashboard) — `/`
- Agent dropdown (populated from DB)
- Prompt textarea
- Working directory input (optional)
- Run button → spawns claude CLI
- Output section: streaming markdown, tool calls in collapsible details, thinking in dimmed blocks
- Swim-lane view when sub-agents spawn
- **Works**: Yes, fully end-to-end
- **Missing**: No visibility into what middleware did (skills injected? task type? security scan result?)

### Agents — `/agents`
- Card grid of agents (name, model, preset badge, domain color)
- Create form: name, model selector, system prompt, preset dropdown
- Edit/delete
- **Works**: Yes, full CRUD
- **Missing**: No link back to persona source. No skill assignment. No run history per agent.

### Sessions — `/sessions`
- List of past session records (ID, agent, status, timestamps)
- Resume button (re-opens in Run page with session ID)
- **Works**: Partial — lists sessions, can resume
- **Missing**: Can't view past output. No search. No filtering by agent/status.

### Workflows — `/workflows`
- **Shows**: Empty state, "No workflows found"
- **Reality**: No workflow engine exists. This is a placeholder page.
- **User feeling**: Confused — what's a workflow? How do I create one?

### Companies — `/companies`
- Card grid of companies (name, mission, budget bar)
- Create form: name, mission, budget limit
- Edit/delete
- **Works**: Yes, full CRUD
- **Pre-seeded**: "Acme AI Corp" with $500 budget
- **Missing**: Budget bar doesn't reflect actual spend (always 0)

### Personas — `/personas`
- Card grid of 112 personas with markdown descriptions
- Filter by division slug, search by text
- Hire modal: select company, department, reports-to, title override
- **Works**: Yes, fully functional
- **Good**: Rich descriptions, clear division taxonomy

### Org Chart — `/org-chart`
- Company selector dropdown
- Tree visualization of positions
- Department grouping
- **Works**: Yes, renders hierarchy
- **Pre-seeded**: 4 agents under Acme (Lead-Architect at root, others reporting to it)
- **Missing**: Can't drag-and-drop. Can't hire from this view.

### Goals — `/goals`
- Company selector
- Create goal form (title, description, parent goal)
- Status buttons (planned → in_progress → completed)
- **Works**: Yes, full CRUD with hierarchy
- **Pre-seeded**: 3 goals (Launch v1.0 + 2 sub-goals)
- **Missing**: Goals are standalone — nothing references or enforces them

### Approvals — `/approvals`
- Company selector, status filter
- Approval cards with approve/reject buttons
- **Works**: Yes, full CRUD
- **Pre-seeded**: 1 pending budget increase request
- **Missing**: Approvals don't gate any action. Purely informational.

### Skills — `/skills`
- **Shows**: Empty or minimal page
- **Reality**: 30 skills are loaded in backend. Zero UI to see them.
- **User feeling**: The sidebar says "Skills" but there's nothing here.

### Memory — `/memory`
- **Shows**: Empty page
- **Reality**: MemoryRepo exists with full CRUD but nothing writes to it
- **User feeling**: What is memory? Why is it empty?

### Hooks — `/hooks`
- **Shows**: Empty page
- **Reality**: HookRepo exists but no event triggers
- **User feeling**: What's a hook?

### Schedules — `/schedules`
- **Shows**: Empty page
- **Reality**: Cron scheduler runs in background but no UI to create schedules
- **User feeling**: Schedules for what?

### Analytics — `/analytics`
- **Shows**: Empty page
- **Reality**: AnalyticsRepo exists, BatchWriter logs events. No aggregation or visualization.
- **User feeling**: Where's the data?

### Settings — `/settings`
- **Shows**: Empty page
- **Reality**: All config is env vars. No runtime config UI.
- **User feeling**: Can't configure anything.

---

## Summary

| Category | Count | Notes |
|----------|-------|-------|
| Fully functional pages | 6 | Run, Agents, Companies, Personas, Org Chart, Goals |
| Partially functional | 2 | Sessions (list only), Approvals (CRUD but no enforcement) |
| Shell pages | 6 | Skills, Memory, Hooks, Schedules, Analytics, Settings |
| Onboarding | 1 | Clean quickstart, good |

**The honest product surface is 8 pages, not 16.**
