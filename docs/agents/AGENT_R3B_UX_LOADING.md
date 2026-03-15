# Agent R3-B: UX — Loading States, Error Recovery, Empty States, WS Indicator

> Replace "Loading..." text with skeleton components. Add structured error display with retry. Add guided empty states. Add WebSocket connection indicator in statusbar.

## Step 1: Read Context

- `CLAUDE.md`
- `frontend/src/app.css` — design system variables, existing `.message.error`, `.empty-state`, `.btn` classes
- `frontend/src/routes/+layout.svelte` — layout structure, statusbar footer
- `frontend/src/lib/api.ts` — API client, `wsUrl()` helper, error handling pattern
- `frontend/src/lib/components/` — existing components (Markdown.svelte, Onboarding.svelte)
- Read 3 page files for current patterns: `routes/companies/+page.svelte`, `routes/goals/+page.svelte`, `routes/approvals/+page.svelte`

## Step 2: Create Skeleton Component

Create `frontend/src/lib/components/Skeleton.svelte`:

```svelte
<script lang="ts">
  let { lines = 3, type = 'text' }: { lines?: number; type?: 'text' | 'card' | 'table' } = $props();
</script>

{#if type === 'card'}
  <div class="skeleton-cards">
    {#each Array(lines) as _}
      <div class="skeleton-card">
        <div class="skeleton-line w-60"></div>
        <div class="skeleton-line w-80"></div>
        <div class="skeleton-line w-40"></div>
      </div>
    {/each}
  </div>
{:else if type === 'table'}
  <div class="skeleton-table">
    {#each Array(lines) as _}
      <div class="skeleton-row">
        <div class="skeleton-line w-20"></div>
        <div class="skeleton-line w-40"></div>
        <div class="skeleton-line w-30"></div>
      </div>
    {/each}
  </div>
{:else}
  <div class="skeleton-text">
    {#each Array(lines) as _, i}
      <div class="skeleton-line" style="width: {80 - i * 15}%"></div>
    {/each}
  </div>
{/if}

<style>
  .skeleton-line {
    height: 0.875rem;
    background: var(--surface-hover);
    border-radius: 4px;
    animation: shimmer 1.5s ease-in-out infinite;
    margin-bottom: 0.5rem;
  }

  .skeleton-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .skeleton-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 1.25rem;
  }

  .skeleton-row {
    display: flex;
    gap: 1rem;
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--border);
  }

  .skeleton-table {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.5rem 1rem;
  }

  .w-20 { width: 20%; }
  .w-30 { width: 30%; }
  .w-40 { width: 40%; }
  .w-60 { width: 60%; }
  .w-80 { width: 80%; }

  @keyframes shimmer {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.8; }
  }
</style>
```

## Step 3: Create ErrorMessage Component

Create `frontend/src/lib/components/ErrorMessage.svelte`:

```svelte
<script lang="ts">
  import { AlertTriangle, RefreshCw } from 'lucide-svelte';

  let { message, onretry }: { message: string; onretry?: () => void } = $props();
</script>

<div class="error-message" role="alert">
  <div class="error-content">
    <AlertTriangle size={16} />
    <span>{message}</span>
  </div>
  {#if onretry}
    <button class="btn btn-ghost" onclick={onretry}>
      <RefreshCw size={14} />
      Retry
    </button>
  {/if}
</div>

<style>
  .error-message {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-radius: var(--radius);
    background: var(--danger-muted);
    color: #fca5a5;
    border: 1px solid rgba(248, 113, 113, 0.3);
    margin-bottom: 1rem;
  }

  .error-content {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
  }
</style>
```

## Step 4: Create EmptyState Component

Create `frontend/src/lib/components/EmptyState.svelte`:

```svelte
<script lang="ts">
  import type { Component } from 'svelte';

  let { icon, title, description, actionLabel, onaction }: {
    icon?: Component;
    title: string;
    description?: string;
    actionLabel?: string;
    onaction?: () => void;
  } = $props();
</script>

<div class="empty-state">
  {#if icon}
    <div class="empty-icon">
      <svelte:component this={icon} size={32} />
    </div>
  {/if}
  <h3>{title}</h3>
  {#if description}
    <p>{description}</p>
  {/if}
  {#if actionLabel && onaction}
    <button class="btn btn-primary" onclick={onaction}>{actionLabel}</button>
  {/if}
</div>

<style>
  .empty-state {
    padding: 3rem 2rem;
    text-align: center;
    color: var(--muted);
  }

  .empty-icon {
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  p {
    margin: 0 0 1.25rem 0;
    font-size: 0.9rem;
    max-width: 24rem;
    margin-left: auto;
    margin-right: auto;
  }
</style>
```

## Step 5: Add WebSocket Status Indicator to Layout

In `frontend/src/routes/+layout.svelte`, add a WebSocket connection indicator in the statusbar.

Add to the `<script>` section:

```typescript
import { Wifi, WifiOff } from 'lucide-svelte';

let wsConnected = $state(false);
let wsRetryCount = $state(0);

onMount(() => {
  connectWs();
});

function connectWs() {
  // Detect WebSocket URL from current origin
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws`;

  const ws = new WebSocket(wsUrl);

  ws.onopen = () => {
    wsConnected = true;
    wsRetryCount = 0;
  };

  ws.onclose = () => {
    wsConnected = false;
    // Reconnect with exponential backoff
    const delay = Math.min(1000 * Math.pow(2, wsRetryCount), 30000);
    wsRetryCount++;
    setTimeout(connectWs, delay);
  };

  ws.onerror = () => {
    ws.close();
  };
}
```

Update the statusbar in the template:

```svelte
<footer class="statusbar">
  <span>v0.6.0-dev</span>
  <span class="ws-status" class:connected={wsConnected} title={wsConnected ? 'Connected to event stream' : 'Disconnected — reconnecting...'}>
    {#if wsConnected}
      <Wifi size={12} />
    {:else}
      <WifiOff size={12} />
    {/if}
  </span>
  <span class="statusbar-note">AI workforce platform</span>
</footer>
```

Add styles:

```css
.ws-status {
  display: flex;
  align-items: center;
  color: var(--danger);
  opacity: 0.7;
}

.ws-status.connected {
  color: var(--success);
}
```

## Step 6: Integrate Components into Pages

For each page, replace loading/error patterns:

**Pattern to replace:**
```svelte
{#if loading}
  <p class="muted">Loading...</p>
{:else if error}
  <div class="message error">{error}</div>
```

**Replace with:**
```svelte
{#if loading}
  <Skeleton type="card" lines={3} />
{:else if error}
  <ErrorMessage message={error} onretry={loadData} />
```

Apply to these pages (import components at top of each):
- `routes/companies/+page.svelte` — Skeleton type="card", EmptyState with "Create your first company"
- `routes/goals/+page.svelte` — Skeleton type="table", EmptyState with "Define goals"
- `routes/approvals/+page.svelte` — Skeleton type="table", EmptyState with "No pending approvals"
- `routes/personas/+page.svelte` — Skeleton type="card"
- `routes/agents/+page.svelte` — Skeleton type="card", EmptyState with "Create or hire agents"
- `routes/sessions/+page.svelte` — Skeleton type="table", EmptyState with "Run an agent to create sessions"

Add imports at top of each page:
```svelte
import Skeleton from '$lib/components/Skeleton.svelte';
import ErrorMessage from '$lib/components/ErrorMessage.svelte';
import EmptyState from '$lib/components/EmptyState.svelte';
```

## Rules

- Touch ONLY files under `frontend/`
- Create new files ONLY in `frontend/src/lib/components/`
- Modify `frontend/src/routes/+layout.svelte` — add WS indicator to statusbar ONLY (do NOT restructure sidebar — Agent R3-C does that)
- Modify `frontend/src/app.css` — add shimmer keyframe if needed, but do NOT change existing styles
- Do NOT touch any Rust code, `site-docs/`, `CLAUDE.md`, `.github/workflows/`
- Preserve all existing functionality — only enhance with loading/error/empty states

## Report

When done, create `docs/agents/REPORT_R3B.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
COMPONENTS_CREATED: [list]
PAGES_UPDATED: [list of pages that got loading/error/empty states]
WS_INDICATOR: added to statusbar (yes/no)
NOTES: [any design decisions]
```
