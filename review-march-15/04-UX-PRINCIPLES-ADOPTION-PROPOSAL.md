# UX Principles Adoption Proposal

> **Date:** 2026-03-15
> **Scope:** AgentForge HQ SvelteKit frontend
> **Goal:** Transform from a functional CRUD interface into a product-grade experience
> **Framework:** Nielsen's 10 Heuristics + specific findings from codebase audit

---

## Current State Assessment

The frontend works. Users can create companies, hire personas, view org charts, run agents, and see streaming output. But it operates at the level of a developer tool prototype, not a product. The gap isn't about features — it's about the 15 small things that make users feel confident vs. confused.

Score against Nielsen's heuristics (1-5):

| Heuristic | Score | Key Gap |
|-----------|-------|---------|
| Visibility of system status | 2/5 | No loading skeletons, no progress indicators during hires |
| Match between system and real world | 3/5 | Good domain language, but UUID exposure in goals/approvals |
| User control and freedom | 2/5 | No undo for deletes, no "back" from detail views |
| Consistency and standards | 3/5 | Consistent dark theme, but button styles vary per page |
| Error prevention | 1/5 | No confirmation for destructive actions beyond delete |
| Recognition over recall | 2/5 | Org chart doesn't show agent names, goals show raw UUIDs for parents |
| Flexibility and efficiency | 2/5 | No keyboard shortcuts, no bulk actions |
| Aesthetic and minimalist design | 4/5 | Clean dark theme, but empty states need work |
| Help users recognize/recover from errors | 1/5 | Raw error strings, no retry buttons, no guidance |
| Help and documentation | 2/5 | Onboarding modal exists but is dismiss-once |

**Overall: 2.2/5** — Functional but not delightful.

---

## Proposal 1: Loading States & System Status Feedback

### Problem
When a user clicks "Hire Persona" or "Create Company", there's a `submitting` flag that disables the button but no visual indication of progress. For API calls that take >200ms, this feels broken.

### Implementation

**Skeleton loading for lists:**
```svelte
{#if loading}
  <div class="skeleton-grid">
    {#each Array(6) as _}
      <div class="skeleton-card">
        <div class="skeleton-line w-60"></div>
        <div class="skeleton-line w-80"></div>
        <div class="skeleton-line w-40"></div>
      </div>
    {/each}
  </div>
{:else}
  <!-- actual content -->
{/if}
```

**Button loading states:**
```svelte
<button disabled={submitting} class="btn btn-primary">
  {#if submitting}
    <span class="spinner"></span> Hiring...
  {:else}
    Hire Persona
  {/if}
</button>
```

**Toast notifications for async results:**
```svelte
<!-- After successful hire -->
<div class="toast toast-success" role="alert" aria-live="polite">
  ✓ {persona.name} hired as {title} — <a href="/org-chart">View in Org Chart</a>
</div>
```

### CSS additions needed:
```css
.skeleton-card { background: var(--surface); border-radius: var(--radius); }
.skeleton-line { height: 0.875rem; background: var(--border); border-radius: 4px; animation: pulse 1.5s infinite; }
.spinner { width: 1rem; height: 1rem; border: 2px solid transparent; border-top-color: currentColor; border-radius: 50%; animation: spin 0.6s linear infinite; }

@keyframes pulse { 50% { opacity: 0.5; } }
@keyframes spin { to { transform: rotate(360deg); } }
```

**Effort:** 1 day across all pages
**Impact:** Immediately feels more responsive

---

## Proposal 2: Error Recovery UX

### Problem
Errors display as raw strings like `"Failed to fetch"` or `"404: Not Found"`. No guidance on what to do next. No retry mechanism.

### Implementation

Create a shared `ErrorMessage.svelte` component:

```svelte
<script lang="ts">
  interface Props {
    error: string;
    retryFn?: () => void;
    context?: string;
  }
  let { error, retryFn, context }: Props = $props();

  const friendlyMessages: Record<string, string> = {
    'Failed to fetch': 'Unable to reach the server. Check that AgentForge is running.',
    'rate limited': 'Too many requests. Please wait a moment and try again.',
    'budget exceeded': 'This action would exceed the company budget. Adjust the budget in Settings.',
    'circuit open': 'The CLI is temporarily unavailable. The system will retry automatically.',
  };

  let friendly = $derived(
    Object.entries(friendlyMessages).find(([key]) =>
      error.toLowerCase().includes(key.toLowerCase())
    )?.[1] || error
  );
</script>

<div class="error-message" role="alert">
  <p>{friendly}</p>
  {#if retryFn}
    <button class="btn btn-ghost" onclick={retryFn}>Try Again</button>
  {/if}
</div>
```

Usage across pages:
```svelte
{#if error}
  <ErrorMessage {error} retryFn={loadCompanies} context="loading companies" />
{/if}
```

**Effort:** 0.5 days
**Impact:** Users know what to do when things fail

---

## Proposal 3: Recognition Over Recall — Kill the UUIDs

### Problem
Goals page shows parent goal as raw UUID: `"Parent: 7a3f2b1e-..."`. Approvals show `data_json` as truncated code. Org chart positions show IDs instead of human names.

### Implementation

**Goals page — resolve parent to title:**
```svelte
<!-- Before -->
<td>{goal.parent_id || '—'}</td>

<!-- After -->
<td>{goals.find(g => g.id === goal.parent_id)?.title || '—'}</td>
```

**Approvals page — structured data display:**
```svelte
<!-- Before -->
<code>{approval.data_json?.substring(0, 80)}</code>

<!-- After -->
{#if approval.data_json}
  {@const data = JSON.parse(approval.data_json)}
  <dl class="approval-data">
    {#each Object.entries(data) as [key, value]}
      <dt>{key}</dt><dd>{value}</dd>
    {/each}
  </dl>
{/if}
```

**Org chart — show agent and persona names:**
```svelte
<div class="org-card">
  <div class="org-title">{position.title}</div>
  <div class="org-agent">{position.agent_name || 'Vacant'}</div>
  <div class="org-dept">{departmentName(position.department_id)}</div>
</div>
```

**Effort:** 0.5 days
**Impact:** Massive cognitive load reduction

---

## Proposal 4: Responsive Design — Mobile-First Sidebar

### Problem
The sidebar is fixed at 13.5rem. On screens < 768px, it consumes 40%+ of viewport, making the app unusable on tablets and phones.

### Implementation

```css
/* Mobile: collapsible sidebar */
@media (max-width: 768px) {
  .app {
    grid-template-columns: 1fr;
    grid-template-areas: "main" "statusbar";
  }

  .sidebar {
    position: fixed;
    left: -100%;
    width: 80vw;
    max-width: 18rem;
    height: 100vh;
    z-index: 50;
    transition: left 0.25s ease;
  }

  .sidebar.open { left: 0; }

  .sidebar-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.5);
    z-index: 49;
  }

  .mobile-header {
    display: flex;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  .hamburger {
    display: block;
    background: none;
    border: none;
    color: var(--text);
    cursor: pointer;
  }
}

@media (min-width: 769px) {
  .mobile-header { display: none; }
  .hamburger { display: none; }
  .sidebar-overlay { display: none; }
}
```

Add to `+layout.svelte`:
```svelte
let sidebarOpen = $state(false);

<!-- Mobile header -->
<div class="mobile-header">
  <button class="hamburger" onclick={() => sidebarOpen = !sidebarOpen} aria-label="Toggle navigation">
    <Menu size={24} />
  </button>
  <span class="brand">AgentForge</span>
</div>
```

**Effort:** 1 day
**Impact:** App becomes usable on tablets and phones

---

## Proposal 5: Accessibility Compliance (WCAG 2.1 AA)

### Critical Fixes

**1. Skip-to-content link:**
```html
<!-- In app.html, first child of <body> -->
<a href="#main-content" class="skip-link">Skip to main content</a>

<style>
  .skip-link {
    position: absolute;
    top: -100%;
    left: 0.5rem;
    z-index: 100;
    padding: 0.5rem 1rem;
    background: var(--accent);
    color: white;
    border-radius: var(--radius);
  }
  .skip-link:focus { top: 0.5rem; }
</style>
```

**2. Focus trap for modals:**
```svelte
<script>
  import { tick } from 'svelte';

  function trapFocus(node: HTMLElement) {
    const focusable = node.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    const first = focusable[0] as HTMLElement;
    const last = focusable[focusable.length - 1] as HTMLElement;

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Tab') {
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
      if (e.key === 'Escape') {
        node.dispatchEvent(new CustomEvent('close'));
      }
    }

    node.addEventListener('keydown', handleKeydown);
    tick().then(() => first?.focus());

    return {
      destroy() { node.removeEventListener('keydown', handleKeydown); }
    };
  }
</script>

<!-- Usage -->
<div class="modal" use:trapFocus on:close={closeModal} role="dialog" aria-modal="true" aria-labelledby="modal-title">
  <h2 id="modal-title">Create Company</h2>
  ...
</div>
```

**3. ARIA attributes for dynamic content:**
```svelte
<!-- Loading state -->
<div aria-busy={loading} aria-live="polite">
  {#if loading}
    <p class="sr-only">Loading companies...</p>
  {/if}
</div>

<!-- Error state -->
{#if error}
  <div role="alert">{error}</div>
{/if}

<!-- Active nav -->
<a href="/companies" aria-current={isActive('/companies') ? 'page' : undefined}>
  Companies
</a>
```

**4. Screen-reader-only utility class:**
```css
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
```

**Effort:** 2 days total
**Impact:** Legal compliance, inclusive design, keyboard power users

---

## Proposal 6: Consistent Component Library

### Problem
Button styles, card layouts, form inputs, badges, and modals are all defined in `app.css` but with overlapping and sometimes conflicting page-level overrides. The `agents` page defines its own badge colors. The `approvals` page defines its own status colors. There's no shared component contract.

### Implementation

Create a minimal shared component set in `frontend/src/lib/components/`:

```
lib/components/
  Button.svelte        — Primary, ghost, danger variants with loading state
  Card.svelte          — Consistent card with header, body, footer slots
  Modal.svelte         — Focus-trapped, accessible modal with close on Escape
  Badge.svelte         — Status badges (success, warning, danger, info, neutral)
  EmptyState.svelte    — Icon + message + action button
  ErrorMessage.svelte  — Friendly error with retry
  Skeleton.svelte      — Loading placeholder
  Select.svelte        — Wrapped <select> with label and error state
  Toast.svelte         — Notification system
```

**Badge component example:**
```svelte
<script lang="ts">
  interface Props {
    variant?: 'success' | 'warning' | 'danger' | 'info' | 'neutral';
    children: import('svelte').Snippet;
  }
  let { variant = 'neutral', children }: Props = $props();
</script>

<span class="badge badge-{variant}">{@render children()}</span>
```

This eliminates the pattern of 12 pages each re-implementing badges with slightly different padding and colors.

**Effort:** 2-3 days
**Impact:** Consistent look, faster page development, easier theming

---

## Proposal 7: Org Chart — From Flat List to Visual Tree

### Problem
The org chart renders a maximum 3-level hardcoded nested div. No tree connectors, no visual hierarchy. This is the signature feature of a "workforce platform" and it currently looks like a bullet list.

### Implementation

Use a recursive Svelte component with CSS tree connectors:

```svelte
<!-- OrgNode.svelte -->
<script lang="ts">
  import type { OrgNode } from '$lib/api';
  interface Props { node: OrgNode; depth?: number; }
  let { node, depth = 0 }: Props = $props();
</script>

<div class="tree-node" style="--depth: {depth}">
  <div class="tree-card">
    <div class="tree-avatar">{node.title?.[0] || '?'}</div>
    <div class="tree-info">
      <div class="tree-title">{node.title}</div>
      <div class="tree-agent">{node.agent_name || 'Vacant'}</div>
      <div class="tree-dept">{node.department_name}</div>
    </div>
  </div>

  {#if node.children?.length}
    <div class="tree-children">
      {#each node.children as child}
        <svelte:self node={child} depth={depth + 1} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .tree-node { position: relative; padding-left: 2rem; }
  .tree-node::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--border);
  }
  .tree-node::after {
    content: '';
    position: absolute;
    left: 0;
    top: 1.5rem;
    width: 1.5rem;
    height: 1px;
    background: var(--border);
  }
  .tree-card {
    display: flex;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    margin-bottom: 0.5rem;
  }
  .tree-avatar {
    width: 2.5rem;
    height: 2.5rem;
    background: var(--accent-muted);
    color: var(--accent);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
  }
  .tree-children { margin-top: 0.25rem; }
</style>
```

**Effort:** 1-2 days
**Impact:** The org chart becomes the hero feature it should be

---

## Proposal 8: Keyboard-First Navigation

### Problem
No keyboard shortcuts exist. Power users (the target audience for an AI workforce platform) expect to navigate quickly.

### Implementation

Global keyboard handler in `+layout.svelte`:

```svelte
<svelte:window on:keydown={handleGlobalKeydown} />

<script>
  import { goto } from '$app/navigation';

  function handleGlobalKeydown(e: KeyboardEvent) {
    // Don't fire when typing in inputs
    if (['INPUT', 'TEXTAREA', 'SELECT'].includes(
      (e.target as HTMLElement)?.tagName)) return;

    // Cmd/Ctrl + K: Quick navigation
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      openCommandPalette();
    }

    // G then letter for "Go to" navigation (vim-style)
    if (e.key === 'g' && !e.metaKey && !e.ctrlKey) {
      waitForSecondKey((key) => {
        const routes: Record<string, string> = {
          'r': '/',          // Run
          'a': '/agents',
          's': '/sessions',
          'c': '/companies',
          'p': '/personas',
          'o': '/org-chart',
        };
        if (routes[key]) goto(routes[key]);
      });
    }
  }
</script>
```

**Effort:** 1 day
**Impact:** Power users get fast navigation

---

## Proposal 9: Empty States That Guide

### Problem
When a page has no data (no companies, no agents, no goals), it shows blank space or a generic "No items found" message. This is the worst possible first impression for a new user.

### Implementation

```svelte
<!-- EmptyState.svelte -->
<script lang="ts">
  import type { Component } from 'svelte';
  interface Props {
    icon: Component;
    title: string;
    description: string;
    actionLabel?: string;
    actionHref?: string;
    onAction?: () => void;
  }
  let { icon: Icon, title, description, actionLabel, actionHref, onAction }: Props = $props();
</script>

<div class="empty-state">
  <div class="empty-icon"><Icon size={48} strokeWidth={1} /></div>
  <h3>{title}</h3>
  <p>{description}</p>
  {#if actionLabel}
    {#if actionHref}
      <a href={actionHref} class="btn btn-primary">{actionLabel}</a>
    {:else if onAction}
      <button class="btn btn-primary" onclick={onAction}>{actionLabel}</button>
    {/if}
  {/if}
</div>
```

**Per-page empty states:**

| Page | Title | Description | Action |
|------|-------|-------------|--------|
| Companies | "No companies yet" | "Create a company to start building your AI workforce" | "Create Company" |
| Personas | "Browse AI Personas" | "100+ pre-built personas ready to hire into your organization" | Link to division filter |
| Org Chart | "Select a company" | "Choose a company to see its organizational hierarchy" | Company selector |
| Goals | "No goals defined" | "Goals give your AI workforce direction and purpose" | "Create Goal" |
| Approvals | "All clear" | "No pending approvals. Your workforce is running smoothly." | — |
| Sessions | "No sessions yet" | "Run an agent from the dashboard to see session history here" | Link to "/" |

**Effort:** 0.5 days
**Impact:** New users aren't lost; the product tells them what to do next

---

## Proposal 10: WebSocket Connection Status Indicator

### Problem
The dashboard auto-reconnects to the WebSocket with exponential backoff, but the user has no idea whether they're connected. If the server is down, the stream just... shows nothing.

### Implementation

Add a connection status dot to the status bar:

```svelte
<div class="connection-status" title={wsStatus}>
  <span class="status-dot {wsStatus}"></span>
  {wsStatus === 'connected' ? 'Live' : wsStatus === 'connecting' ? 'Reconnecting...' : 'Disconnected'}
</div>

<style>
  .status-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
  .status-dot.connected { background: var(--success); }
  .status-dot.connecting { background: var(--warning); animation: pulse 1s infinite; }
  .status-dot.disconnected { background: var(--danger); }
</style>
```

**Effort:** 2 hours
**Impact:** Users always know if they're seeing live data

---

## Implementation Roadmap

### Phase 1 — Quick Wins (Week 1, ~3 days)

| Item | Effort | Impact |
|------|--------|--------|
| Error recovery component | 0.5d | High |
| UUID removal (goals, approvals) | 0.5d | High |
| Loading skeletons | 1d | Medium |
| WebSocket status indicator | 2h | Medium |
| Empty states | 0.5d | Medium |

### Phase 2 — Structural (Week 2-3, ~5 days)

| Item | Effort | Impact |
|------|--------|--------|
| Shared component library | 2-3d | High (long-term) |
| Responsive sidebar | 1d | Medium |
| Org chart tree view | 1-2d | High (brand feature) |

### Phase 3 — Polish (Week 4, ~3 days)

| Item | Effort | Impact |
|------|--------|--------|
| Accessibility compliance | 2d | High (compliance) |
| Keyboard navigation | 1d | Medium |

### Total: ~11 days of focused frontend work

---

## Success Metrics

After implementation, re-score against Nielsen's heuristics:

| Heuristic | Before | Target |
|-----------|--------|--------|
| Visibility of system status | 2/5 | 4/5 |
| Match between system and real world | 3/5 | 4/5 |
| User control and freedom | 2/5 | 3/5 |
| Consistency and standards | 3/5 | 4/5 |
| Error prevention | 1/5 | 3/5 |
| Recognition over recall | 2/5 | 4/5 |
| Flexibility and efficiency | 2/5 | 3/5 |
| Aesthetic and minimalist design | 4/5 | 4/5 |
| Help users recognize/recover from errors | 1/5 | 4/5 |
| Help and documentation | 2/5 | 3/5 |

**Target overall: 3.6/5** (from 2.2/5) — a 64% improvement in UX quality.
