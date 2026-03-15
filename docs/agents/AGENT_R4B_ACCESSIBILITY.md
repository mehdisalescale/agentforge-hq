# Agent R4-B: Accessibility (WCAG 2.1 AA Subset)

> Add skip-to-content link, focus management for modals, aria attributes, screen-reader utilities, and visible focus styles.

**Run AFTER R3-B and R3-C complete (they create/modify the components this agent enhances).**

## Step 1: Read Context

- `frontend/src/app.html` — HTML shell
- `frontend/src/app.css` — design system
- `frontend/src/routes/+layout.svelte` — layout with sidebar + statusbar
- `frontend/src/lib/components/` — list all components (Skeleton, ErrorMessage, EmptyState, OrgNode, Markdown, Onboarding)
- Read 2-3 page files with modals: `routes/companies/+page.svelte`, `routes/personas/+page.svelte`

## Step 2: Add Skip-to-Content Link

In `frontend/src/app.html`, add before `<div id="svelte">` (or whatever the mount point is):

If `app.html` has a `<body>` tag, add right after it:
```html
<a href="#main-content" class="sr-only sr-only-focusable">Skip to main content</a>
```

In `frontend/src/routes/+layout.svelte`, add `id="main-content"` to the `<main>` element:
```svelte
<main class="main" id="main-content">
```

## Step 3: Add Screen-Reader + Focus Utilities to CSS

In `frontend/src/app.css`, add at the end (before any responsive media queries):

```css
/* ─── Accessibility ─── */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.sr-only-focusable:focus {
  position: fixed;
  top: 0;
  left: 0;
  z-index: 999;
  width: auto;
  height: auto;
  padding: 0.75rem 1.5rem;
  margin: 0;
  overflow: visible;
  clip: auto;
  white-space: normal;
  background: var(--accent);
  color: #09090b;
  font-weight: 600;
  font-size: 0.9rem;
  border-radius: 0 0 var(--radius) 0;
}

/* Visible focus ring for keyboard navigation */
:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

/* Remove focus ring for mouse clicks */
:focus:not(:focus-visible) {
  outline: none;
}
```

## Step 4: Add aria Attributes to Layout

In `frontend/src/routes/+layout.svelte`:

1. Add `role="navigation"` and `aria-label="Main"` to sidebar nav:
```svelte
<nav class="nav" role="navigation" aria-label="Main navigation">
```

2. Add `aria-current="page"` to active links:
```svelte
<a
  class="link"
  class:active={isActive(link.href, $page.url.pathname)}
  href={link.href}
  aria-current={isActive(link.href, $page.url.pathname) ? 'page' : undefined}
>
```

3. Add `role="status"` to statusbar:
```svelte
<footer class="statusbar" role="status">
```

## Step 5: Add Focus Trap to Modals

For pages with modals (companies, personas, goals, approvals), add focus management.

Create a small focus trap action in `frontend/src/lib/actions/focusTrap.ts`:

```typescript
/**
 * Svelte action that traps focus within an element (for modals).
 * Usage: <div use:focusTrap>
 */
export function focusTrap(node: HTMLElement) {
  const focusable = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== 'Tab') return;

    const elements = Array.from(node.querySelectorAll(focusable)) as HTMLElement[];
    if (elements.length === 0) return;

    const first = elements[0];
    const last = elements[elements.length - 1];

    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  // Focus first focusable element on mount
  requestAnimationFrame(() => {
    const elements = node.querySelectorAll(focusable) as NodeListOf<HTMLElement>;
    if (elements.length > 0) elements[0].focus();
  });

  node.addEventListener('keydown', handleKeydown);

  return {
    destroy() {
      node.removeEventListener('keydown', handleKeydown);
    }
  };
}
```

Then in modal pages, add `use:focusTrap` to the modal element:

```svelte
<script>
  import { focusTrap } from '$lib/actions/focusTrap';
</script>

<div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Create company">
  <div class="modal" use:focusTrap>
    <!-- modal content -->
  </div>
</div>
```

Also add Escape key to close modals (if not already):
```svelte
<svelte:window onkeydown={(e) => { if (e.key === 'Escape' && formOpen) closeForm(); }} />
```

Apply to: `companies/+page.svelte`, `personas/+page.svelte`, `goals/+page.svelte`, `approvals/+page.svelte`

## Step 6: Add aria to Status Badges

In pages with status badges (approvals), add text that screen readers can understand:

```svelte
<span class="badge badge-{status}" role="status" aria-label="Status: {status}">
  {status}
</span>
```

## Step 7: Add aria-busy to Loading States

In pages that show loading skeletons, add `aria-busy="true"` to the container:

```svelte
<div aria-busy={loading}>
  {#if loading}
    <Skeleton ... />
  {:else}
    <!-- content -->
  {/if}
</div>
```

## Step 8: Verify

```bash
cd frontend && pnpm check
cd frontend && pnpm build
```

## Rules

- Touch ONLY files under `frontend/`
- Create new files only for `focusTrap.ts` action
- Do NOT restructure sidebar or change responsive behavior (R3-C already did that)
- Do NOT change visual styles — only add accessibility enhancements
- Do NOT touch any Rust code, `site-docs/`, `CLAUDE.md`, `.github/workflows/`

## Report

When done, create `docs/agents/REPORT_R4B.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
FILES_MODIFIED: [list]
FILES_CREATED: [list]
SKIP_TO_CONTENT: added (yes/no)
FOCUS_TRAP: added to [N] modals
ARIA_ATTRIBUTES: [count] added
SR_ONLY_UTILITY: added (yes/no)
FOCUS_VISIBLE: added (yes/no)
PNPM_CHECK: pass/fail
PNPM_BUILD: pass/fail
```
