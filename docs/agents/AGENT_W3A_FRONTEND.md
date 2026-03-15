# Agent W3-A: Frontend Polish

> You are Agent W3-A. Your job: add run metadata panel to dashboard, clean up sidebar (remove shell pages), and enrich agent cards.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
frontend/src/routes/+layout.svelte                 — sidebar navigation
frontend/src/routes/+page.svelte                   — dashboard / run page
frontend/src/routes/agents/+page.svelte            — agent cards
stat-qou-plan/REVISED_PLAN.md                      — what we're doing and why
```

## Step 2: Sidebar Cleanup

In `frontend/src/routes/+layout.svelte`, remove these links from the sidebar navigation:
- Workflows (from Workspace group)
- Memory (from Configuration group)
- Hooks (from Configuration group)
- Schedules (from Configuration group)

Keep the route files so direct URLs don't 404 — only remove the sidebar links.

The sidebar should end up with:
```
WORKSPACE:      Run, Agents, Sessions
ORGANIZATION:   Companies, Personas, Org Chart, Goals, Approvals
CONFIGURATION:  Skills
INSIGHTS:       Analytics, Settings
```

That's 12 links, all pointing to functional pages.

## Step 3: Run Metadata Panel

In `frontend/src/routes/+page.svelte`, add a metadata panel that shows AFTER a run completes (when `streamStatus === 'completed'`). Display the metadata that the middleware chain already sets in `ctx.metadata`:

Add a section between the run form and the output section:

```svelte
{#if streamStatus === 'completed' || streamStatus === 'failed'}
<div class="run-meta">
  {#if runMeta.taskType}
    <span class="meta-chip">
      <strong>Task:</strong> {runMeta.taskType}
    </span>
  {/if}
  {#if runMeta.skillsInjected}
    <span class="meta-chip">
      <strong>Skills:</strong> {runMeta.skillsInjected}
    </span>
  {/if}
  {#if runMeta.securityScan}
    <span class="meta-chip" class:passed={runMeta.securityScan === 'passed'} class:failed={runMeta.securityScan === 'failed'}>
      <strong>Security:</strong> {runMeta.securityScan}
    </span>
  {/if}
</div>
{/if}
```

To get the metadata, after a run completes, fetch session details:
```typescript
// After streamStatus changes to 'completed', fetch session metadata
if (currentSessionId) {
  const session = await fetch(`/api/v1/sessions/${currentSessionId}`).then(r => r.json());
  runMeta = {
    taskType: session.metadata?.task_type,
    skillsInjected: session.metadata?.injected_skills ? 'Yes' : 'None',
    securityScan: session.metadata?.security_scan,
  };
}
```

Style the metadata panel:
```css
.run-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin: 0.75rem 0;
  padding: 0.6rem 0.8rem;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  font-size: 0.8rem;
}
.meta-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  background: var(--bg);
  color: var(--text-secondary);
}
.meta-chip strong {
  color: var(--muted);
  font-weight: 500;
}
.meta-chip.passed { color: #86efac; }
.meta-chip.failed { color: #f87171; }
```

## Step 4: Agent Cards Enrichment

In `frontend/src/routes/agents/+page.svelte`:
- If an agent has a `persona_id`, show a small "Hired from catalog" badge on the card
- Show the agent's `preset` as a colored badge (already partially there — verify it works)

## Step 5: Update Onboarding

In `frontend/src/lib/components/Onboarding.svelte`, remove the "also available" chips that correspond to removed sidebar items (Workflows, Memory, Hooks, Schedules). Keep the remaining 8 chips.

## Step 6: Verify

```bash
cd frontend && pnpm build 2>&1   # must build cleanly
```

## Rules

- ONLY modify frontend files
- Do NOT touch any Rust code
- Do NOT modify middleware.rs or run.rs
- Do NOT modify sessions page (Agent W3-C handles that)
- Do NOT modify skills or analytics pages (Agent W3-D handles those)
- Commit with: `feat(frontend): clean sidebar, add run metadata panel, enrich agent cards`

## Report
```
STATUS: done | blocked
FILES_MODIFIED: [list]
SIDEBAR_LINKS_REMOVED: [list]
SIDEBAR_LINKS_REMAINING: [count]
ISSUES: [any]
```
