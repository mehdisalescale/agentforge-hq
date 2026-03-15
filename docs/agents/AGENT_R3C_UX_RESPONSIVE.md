# Agent R3-C: UX — Responsive Sidebar, Org Chart Recursion, Component Library

> Sidebar is fixed at 13.5rem on all screen sizes — add hamburger menu for mobile. Org chart is hardcoded to 3 levels — make recursive. Extract reusable components.

**IMPORTANT: Run this AFTER Agent R3-B completes. R3-B modifies +layout.svelte (statusbar) and app.css (shimmer animation). This agent restructures the sidebar in the same files.**

## Step 1: Read Context

- `CLAUDE.md`
- `frontend/src/app.css` — full design system
- `frontend/src/routes/+layout.svelte` — current layout with fixed sidebar
- `frontend/src/routes/org-chart/+page.svelte` — hardcoded 3-level tree
- Skim 2-3 pages to see modal/card/badge patterns: `routes/companies/+page.svelte`, `routes/approvals/+page.svelte`

## Step 2: Add Responsive Sidebar

In `frontend/src/routes/+layout.svelte`, add a hamburger toggle for mobile:

Add to `<script>`:
```typescript
import { Menu, X } from 'lucide-svelte';

let sidebarOpen = $state(false);

function toggleSidebar() {
  sidebarOpen = !sidebarOpen;
}

// Close sidebar on navigation
$effect(() => {
  // Reading $page.url.pathname triggers on route changes
  const _ = $page.url.pathname;
  sidebarOpen = false;
});
```

Update the template — add hamburger button before sidebar:
```svelte
<button class="hamburger" onclick={toggleSidebar} aria-label="Toggle navigation">
  {#if sidebarOpen}
    <X size={20} />
  {:else}
    <Menu size={20} />
  {/if}
</button>

{#if sidebarOpen}
  <div class="sidebar-overlay" onclick={() => sidebarOpen = false}></div>
{/if}

<aside class="sidebar" class:open={sidebarOpen}>
  <!-- existing nav content -->
</aside>
```

## Step 3: Add Responsive CSS

In `frontend/src/app.css`, add responsive breakpoints:

```css
/* ─── Responsive ─── */
.hamburger {
  display: none;
  position: fixed;
  top: 0.75rem;
  left: 0.75rem;
  z-index: 200;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 0.4rem;
  color: var(--text);
  cursor: pointer;
}

.sidebar-overlay {
  display: none;
}

@media (max-width: 768px) {
  .hamburger {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .app {
    grid-template-columns: 1fr;
    grid-template-areas:
      "main"
      "statusbar";
  }

  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 13.5rem;
    z-index: 150;
    transform: translateX(-100%);
    transition: transform 200ms ease;
    box-shadow: none;
  }

  .sidebar.open {
    transform: translateX(0);
    box-shadow: var(--shadow-lg);
  }

  .sidebar-overlay {
    display: block;
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 140;
  }

  .main {
    padding: 1rem;
    padding-top: 3rem; /* space for hamburger */
  }

  .statusbar {
    grid-column: 1;
  }
}

@media (max-width: 480px) {
  .main {
    padding: 0.75rem;
    padding-top: 3rem;
  }

  .modal {
    max-width: 100%;
    margin: 0.5rem;
    padding: 1.25rem;
  }

  .agent-cards,
  .skeleton-cards {
    grid-template-columns: 1fr;
  }
}
```

## Step 4: Create Recursive OrgNode Component

Create `frontend/src/lib/components/OrgNode.svelte`:

```svelte
<script lang="ts">
  import type { OrgChartNode, Department } from '$lib/api';

  let { node, departments, depth = 0 }: {
    node: OrgChartNode;
    departments: Department[];
    depth?: number;
  } = $props();

  function deptName(id: string | null | undefined): string {
    if (!id) return '—';
    const d = departments.find(x => x.id === id);
    return d ? d.name : id.slice(0, 8) + '...';
  }

  let collapsed = $state(depth > 2);
  let hasChildren = $derived(node.children && node.children.length > 0);
</script>

<div class="org-node" style="--depth: {depth}">
  <div class="org-card">
    <div class="card-top">
      {#if hasChildren}
        <button class="toggle" onclick={() => collapsed = !collapsed} aria-label={collapsed ? 'Expand' : 'Collapse'}>
          {collapsed ? '+' : '−'}
        </button>
      {/if}
      <div class="title">{node.position.title ?? node.position.role}</div>
    </div>
    <div class="meta">
      <span class="meta-label">Dept:</span>
      <span>{deptName(node.position.department_id ?? undefined)}</span>
    </div>
  </div>

  {#if hasChildren && !collapsed}
    <div class="children">
      {#each node.children as child (child.position.id)}
        <svelte:self node={child} {departments} depth={depth + 1} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .org-node {
    position: relative;
    padding-left: 1.25rem;
    margin: 0.5rem 0;
  }

  .org-node::before {
    content: '';
    position: absolute;
    left: 0.5rem;
    top: 0;
    bottom: 0;
    border-left: 1px solid var(--border);
  }

  .org-card {
    position: relative;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    min-width: 10rem;
  }

  .org-card::before {
    content: '';
    position: absolute;
    left: -0.75rem;
    top: 50%;
    width: 0.75rem;
    border-top: 1px solid var(--border);
  }

  .card-top {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .toggle {
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--muted);
    width: 1.2rem;
    height: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 0.8rem;
    flex-shrink: 0;
  }

  .toggle:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .title {
    font-size: 0.9rem;
    font-weight: 600;
  }

  .meta {
    display: flex;
    gap: 0.25rem;
    font-size: 0.75rem;
    color: var(--muted);
    margin-top: 0.15rem;
  }

  .meta-label {
    opacity: 0.7;
  }

  .children {
    margin-left: 1.25rem;
    margin-top: 0.25rem;
  }
</style>
```

## Step 5: Update Org Chart Page to Use Recursive Component

Replace the hardcoded 3-level nesting in `frontend/src/routes/org-chart/+page.svelte`:

Replace the `<div class="tree">` section with:
```svelte
<div class="tree">
  {#each chart.roots as node (node.position.id)}
    <OrgNode {node} departments={chart.departments} />
  {/each}
</div>
```

Add import at top:
```svelte
import OrgNode from '$lib/components/OrgNode.svelte';
```

Remove the old hardcoded tree rendering (the nested `{#each node.children}` with `{#each child.children}` with `{#each grand}`).

Also remove the `.org-node`, `.org-card`, `.children`, `.meta-label` styles from the page's `<style>` block since they're now in the OrgNode component.

## Step 6: Verify

```bash
cd frontend && pnpm check   # type checking
cd frontend && pnpm build    # builds successfully
```

## Rules

- Touch ONLY files under `frontend/`
- Create new components in `frontend/src/lib/components/`
- Modify `frontend/src/routes/+layout.svelte` — add hamburger + overlay (do NOT change statusbar — R3-B already added WS indicator there)
- Modify `frontend/src/app.css` — add responsive breakpoints at the END of the file
- Modify `frontend/src/routes/org-chart/+page.svelte` — replace hardcoded tree with recursive OrgNode
- Do NOT touch any Rust code, `site-docs/`, `CLAUDE.md`, `.github/workflows/`
- Do NOT remove or change any existing styles — only ADD responsive overrides

## Report

When done, create `docs/agents/REPORT_R3C.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
COMPONENTS_CREATED: [list]
PAGES_MODIFIED: [list]
RESPONSIVE_BREAKPOINTS: 768px (tablet), 480px (mobile)
ORG_CHART_RECURSIVE: yes/no
PNPM_CHECK: pass/fail
PNPM_BUILD: pass/fail
```
