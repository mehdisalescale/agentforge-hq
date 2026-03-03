# Information Architecture

> How information is organized, navigated, searched, routed, and kept in sync across Claude Forge's UI.

---

## Content Hierarchy

### Primary Content (Always Visible)

The most important information is visible without any interaction.

| Context | Primary Content |
|---------|----------------|
| Dashboard | Active agent count, running workflow count, system health status |
| Agent Manager | Agent names, statuses, models |
| Session Browser | Session list with agent, timestamp, status |
| Workflow Designer | Workflow name, status, step count |
| Observability | Swim-lane events, pulse chart |
| Git Panel | Current branch, changed file count |
| Settings | Setting name, current value |

### Secondary Content (Visible on Hover or Focus)

Information revealed with minimal interaction -- hover a row, focus a card, or glance at a tooltip.

| Context | Secondary Content |
|---------|------------------|
| Dashboard | Token usage numbers, last active timestamps, session counts |
| Agent Manager | System prompt preview, MCP server count, hook count, circuit breaker state |
| Session Browser | Message count, duration, project directory, last message preview |
| Workflow Designer | Step names, dependency edges, last run duration |
| Observability | Tool names, token counts per event, event timestamps |
| Git Panel | File paths, change type (added/modified/deleted), diff stats |
| Settings | Setting description, default value, valid range |

### Tertiary Content (Visible on Click or Expand)

Full detail that requires deliberate navigation.

| Context | Tertiary Content |
|---------|-----------------|
| Dashboard | Full agent configuration, detailed metrics history |
| Agent Manager | Full system prompt, all settings, session history, event log |
| Session Browser | Full chat transcript, raw event stream, extracted todos |
| Workflow Designer | Node configuration, execution logs, step-by-step trace |
| Observability | Full event detail (JSON), agent configuration at time of event |
| Git Panel | Full file diff, commit history, worktree details |
| Settings | Extended help text, environment variable reference, examples |

---

## Navigation Model

### Primary Navigation: Sidebar

The sidebar is the persistent navigation element visible on all pages.

```
+--+
|  |  [Forge Logo]
|  |
|  |  Dashboard      (icon: grid)
|  |  Agents         (icon: users)
|  |  Sessions       (icon: messages)
|  |  Workflows      (icon: git-branch)
|  |  Skills         (icon: puzzle)
|  |  Observability  (icon: activity)
|  |  Git            (icon: git-merge)
|  |  Code           (icon: file-code)
|  |  Terminal        (icon: terminal)
|  |
|  |  ────────────
|  |
|  |  Settings       (icon: settings)
|  |  [Collapse btn]
+--+
```

**Sidebar states**:
- **Expanded**: Icon + label visible. Width: 220px.
- **Collapsed**: Icon only. Width: 56px. Labels appear as tooltips on hover.
- **Hidden**: On mobile, sidebar is replaced by bottom tab bar.

Toggle: `Ctrl/Cmd + B` or click the collapse button.

Active page is indicated by: background highlight + left border accent.

### Secondary Navigation: Tabs

Within a page, tabs separate content categories.

Examples:
- Agent detail: `Overview | Sessions | Events | Config | CLAUDE.md`
- Settings: `General | MCP Servers | Hooks | Notifications | Security`
- Session detail: `Transcript | Events | Todos | Export`

Tab behavior:
- Active tab persists when navigating away and back (stored in URL hash)
- Keyboard navigation: `Tab` moves focus between tabs, `Arrow Left/Right` selects adjacent tab
- Tabs scroll horizontally if there are more than fit the viewport

### Tertiary Navigation: Breadcrumbs

Breadcrumbs appear on detail pages to show location in the hierarchy.

```
Agents > reviewer-agent > Sessions > session-abc123
```

Each breadcrumb segment is a link. The current page (last segment) is not linked.

### Contextual Navigation: Command Palette

`Ctrl/Cmd + K` opens a fuzzy-search command palette that searches across:
1. Pages (Dashboard, Agents, Sessions, ...)
2. Agents (by name)
3. Sessions (by content, via FTS5)
4. Skills (by name and category)
5. Workflows (by name)
6. Settings (by name)
7. Actions (Create Agent, Start Session, Run Workflow, ...)

Results are grouped by type and ranked by relevance. The palette supports:
- Typing a `/` prefix to filter by type: `/agent reviewer`, `/session postgres`, `/setting model`
- Arrow keys to navigate, Enter to select, Esc to close
- Recent items shown when palette opens with empty query

---

## Search Architecture

### What Is Searchable

| Content Type | Search Method | Index |
|-------------|--------------|-------|
| Sessions (content) | Full-text search | SQLite FTS5 |
| Sessions (metadata) | Structured query | SQLite indexed columns |
| Agents (name, tags) | Prefix/fuzzy match | In-memory (DashMap) |
| Skills (name, category, description) | Full-text search | SQLite FTS5 |
| Workflows (name) | Prefix match | In-memory |
| Events (content) | Full-text search | SQLite FTS5 |
| Files (path) | Fuzzy match | In-memory file tree |
| Settings (name) | Prefix match | Static list |
| Commands (name, description) | Fuzzy match | Static list |

### Search Implementation

**SQLite FTS5** powers all full-text search. This provides:
- Tokenized full-text search with ranking
- Prefix queries (`prefix*`)
- Phrase queries (`"exact phrase"`)
- Boolean operators (`AND`, `OR`, `NOT`)
- Snippet extraction for result previews
- Highlight markers for matched terms

**FTS5 Tables**:
```sql
CREATE VIRTUAL TABLE session_fts USING fts5(
    session_id,
    agent_name,
    project_dir,
    content,
    tokenize='porter unicode61'
);

CREATE VIRTUAL TABLE skill_fts USING fts5(
    skill_id,
    name,
    category,
    description,
    tokenize='porter unicode61'
);

CREATE VIRTUAL TABLE event_fts USING fts5(
    event_id,
    session_id,
    agent_id,
    content,
    tokenize='porter unicode61'
);
```

**Fuzzy matching** (for command palette, file search) uses a JavaScript implementation of the Smith-Waterman algorithm or a simpler character-subsequence matcher, running client-side on pre-fetched data.

### Search UX

| Location | Trigger | Scope | Results |
|----------|---------|-------|---------|
| Command palette | `Ctrl/Cmd + K` | Everything | Grouped by type, max 10 per group |
| Session browser | `/` or search box | Sessions only | Paginated list with snippets |
| Skill browser | `/` or search box | Skills only | Filtered grid |
| Agent list | `/` or search box | Agents only | Filtered table/grid |
| Code viewer | `Ctrl/Cmd + P` | Files only | Ranked file list |
| Git panel | `/` in branch list | Branches only | Filtered branch list |

---

## URL Structure and Routing

### Route Map

```
/                                   Dashboard
/agents                             Agent list
/agents/new                         Create agent
/agents/:id                         Agent detail (Overview tab)
/agents/:id/edit                    Agent edit form
/agents/:id#sessions                Agent detail (Sessions tab)
/agents/:id#events                  Agent detail (Events tab)
/agents/:id#config                  Agent detail (Config tab)
/agents/:id#claudemd                Agent detail (CLAUDE.md tab)
/sessions                           Session browser
/sessions/:id                       Session detail (Transcript tab)
/sessions/:id#events                Session detail (Events tab)
/sessions/:id#todos                 Session detail (Todos tab)
/sessions/:id#export                Session detail (Export tab)
/workflows                          Workflow list
/workflows/new                      Create workflow
/workflows/:id                      Workflow detail (view mode)
/workflows/:id/edit                 Workflow editor (edit mode)
/workflows/:id/runs                 Workflow run history
/workflows/:id/runs/:runId          Specific run detail
/skills                             Skill browser
/skills/:id                         Skill detail
/observe                            Observability dashboard
/observe?agents=a,b,c               Filtered to specific agents
/observe?range=1h                   Filtered to time range
/git                                Git panel
/git/diff/:path                     Diff view for specific file
/git/worktrees                      Worktree management
/code                               Code viewer (root)
/code/*path                         Code viewer (specific file/directory)
/terminal                           Terminal
/settings                           Settings (General tab)
/settings/mcp                       MCP server configuration
/settings/hooks                     Hook configuration
/settings/notifications             Notification preferences
/settings/security                  Security settings
```

### URL State Encoding

Filters, sort orders, and view modes are encoded in URL query parameters so that URLs are shareable and bookmarkable.

```
/sessions?q=postgres&agent=reviewer&sort=date&order=desc&view=list
/agents?status=running&sort=name&view=grid
/observe?agents=reviewer,tester&range=1h&scroll=live
```

### SvelteKit Routing

Routes map to SvelteKit file-based routing:

```
src/routes/
  +layout.svelte              (sidebar, global state)
  +page.svelte                (dashboard)
  agents/
    +page.svelte              (agent list)
    new/+page.svelte          (create agent)
    [id]/
      +page.svelte            (agent detail)
      edit/+page.svelte       (agent edit)
  sessions/
    +page.svelte              (session browser)
    [id]/+page.svelte         (session detail)
  workflows/
    +page.svelte              (workflow list)
    new/+page.svelte          (create workflow)
    [id]/
      +page.svelte            (workflow detail)
      edit/+page.svelte       (workflow editor)
      runs/
        +page.svelte          (run history)
        [runId]/+page.svelte  (run detail)
  skills/
    +page.svelte              (skill browser)
    [id]/+page.svelte         (skill detail)
  observe/+page.svelte        (observability)
  git/+page.svelte            (git panel)
  code/
    +page.svelte              (code viewer root)
    [...path]/+page.svelte    (code viewer file/dir)
  terminal/+page.svelte       (terminal)
  settings/
    +page.svelte              (general settings)
    mcp/+page.svelte
    hooks/+page.svelte
    notifications/+page.svelte
    security/+page.svelte
```

---

## State Management Patterns

### Svelte 5 Runes Architecture

Claude Forge uses Svelte 5's rune system for reactive state management.

#### Layer 1: Component State (`$state`)

Local state that lives and dies with a component instance.

```svelte
<script>
  let searchQuery = $state('');
  let isExpanded = $state(false);
  let selectedIndex = $state(0);
</script>
```

Use for: form inputs, UI toggles, hover states, scroll positions.

#### Layer 2: Derived State (`$derived`)

Computed values that automatically update when dependencies change.

```svelte
<script>
  let agents = $state([]);
  let filter = $state('running');

  let filteredAgents = $derived(
    agents.filter(a => filter === 'all' || a.status === filter)
  );

  let agentCount = $derived(filteredAgents.length);
</script>
```

Use for: filtered lists, computed metrics, conditional display logic.

#### Layer 3: Shared State (Store Modules)

State shared across components, managed in dedicated `.svelte.ts` store files.

```typescript
// stores/agents.svelte.ts
import type { Agent } from '$lib/types';

let agents = $state<Agent[]>([]);
let loading = $state(false);

export function getAgents() {
  return agents;
}

export async function fetchAgents() {
  loading = true;
  const response = await fetch('/api/agents');
  agents = await response.json();
  loading = false;
}

export function updateAgent(id: string, updates: Partial<Agent>) {
  agents = agents.map(a => a.id === id ? { ...a, ...updates } : a);
}
```

Use for: data shared across pages (agent list, session list, settings), WebSocket connection state.

#### Layer 4: Effects (`$effect`)

Side effects that run when reactive dependencies change.

```svelte
<script>
  let agentId = $state('');

  $effect(() => {
    if (agentId) {
      // Subscribe to WebSocket events for this agent
      const unsub = subscribeToAgent(agentId);
      return () => unsub(); // Cleanup on change or destroy
    }
  });
</script>
```

Use for: WebSocket subscriptions, localStorage persistence, DOM measurements.

### State Ownership Rules

| State | Owner | Access Pattern |
|-------|-------|---------------|
| Agent list | `stores/agents.svelte.ts` | Imported by any component that needs agents |
| Session list | `stores/sessions.svelte.ts` | Imported by session-related components |
| WebSocket connection | `stores/ws.svelte.ts` | Single connection, shared event stream |
| Current route params | SvelteKit `$page` store | Read from `$page.params` |
| User preferences | `stores/preferences.svelte.ts` | Persisted to localStorage |
| Theme | `stores/theme.svelte.ts` | Applied to `<html>` class |
| Command palette | `stores/palette.svelte.ts` | Global, toggled from anywhere |

---

## Real-Time Update Patterns

### WebSocket to Store to Component Pipeline

```
Server Event                 WebSocket              Store               Component
──────────                   ─────────              ─────                ─────────
Agent status changed ──> WS message ──> updateAgent(id, {status}) ──> UI re-renders
New event emitted    ──> WS message ──> appendEvent(event)        ──> Event list updates
Metric updated       ──> WS message ──> updateMetric(metric)      ──> Chart updates
```

### WebSocket Message Format

```typescript
interface WSMessage {
  type: 'agent_update' | 'event' | 'metric' | 'workflow_update' | 'system';
  payload: unknown;
  timestamp: string;  // ISO 8601
  agent_id?: string;
  session_id?: string;
}
```

### Connection Management

```typescript
// stores/ws.svelte.ts

let socket = $state<WebSocket | null>(null);
let connected = $state(false);
let reconnectAttempts = $state(0);

const MAX_RECONNECT_DELAY = 30_000; // 30 seconds

function connect() {
  socket = new WebSocket(`ws://${location.host}/ws`);

  socket.onopen = () => {
    connected = true;
    reconnectAttempts = 0;
  };

  socket.onmessage = (event) => {
    const msg: WSMessage = JSON.parse(event.data);
    dispatch(msg);  // Route to appropriate store
  };

  socket.onclose = () => {
    connected = false;
    scheduleReconnect();
  };
}

function scheduleReconnect() {
  const delay = Math.min(1000 * 2 ** reconnectAttempts, MAX_RECONNECT_DELAY);
  reconnectAttempts++;
  setTimeout(connect, delay);
}

function dispatch(msg: WSMessage) {
  switch (msg.type) {
    case 'agent_update':
      updateAgent(msg.agent_id!, msg.payload);
      break;
    case 'event':
      appendEvent(msg.payload);
      break;
    case 'metric':
      updateMetric(msg.payload);
      break;
    case 'workflow_update':
      updateWorkflow(msg.payload);
      break;
    case 'system':
      handleSystemMessage(msg.payload);
      break;
  }
}
```

### Reconnection Strategy

| Attempt | Delay | Total Elapsed |
|---------|-------|---------------|
| 1 | 1 second | 1 second |
| 2 | 2 seconds | 3 seconds |
| 3 | 4 seconds | 7 seconds |
| 4 | 8 seconds | 15 seconds |
| 5 | 16 seconds | 31 seconds |
| 6+ | 30 seconds (capped) | 61+ seconds |

Visual indicator: when `connected` is false, a persistent banner shows "Reconnecting..." with the retry countdown.

### Optimistic Updates

For user-initiated actions (create agent, update config, toggle setting), update the store immediately and reconcile with the server response.

```typescript
// Optimistic: update immediately
updateAgent(id, { status: 'running' });

// Then confirm with server
const result = await fetch(`/api/agents/${id}/start`, { method: 'POST' });
if (!result.ok) {
  // Rollback on failure
  updateAgent(id, { status: previousStatus });
  showError('Failed to start agent');
}
```

### Event Batching

For high-frequency events (agent event streams during active sessions), batch UI updates to avoid rendering bottlenecks.

```typescript
let eventBuffer: ForgeEvent[] = [];
let flushScheduled = false;

function bufferEvent(event: ForgeEvent) {
  eventBuffer.push(event);
  if (!flushScheduled) {
    flushScheduled = true;
    requestAnimationFrame(() => {
      appendEvents(eventBuffer);  // Single store update for entire batch
      eventBuffer = [];
      flushScheduled = false;
    });
  }
}
```

This ensures the UI never renders more than once per animation frame (16.67ms at 60fps), regardless of how many events arrive.
