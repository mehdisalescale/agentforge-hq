# UI/UX Design

> Design document for Claude Forge's embedded web interface.

---

## Design Principles

### 1. Information Density Over Simplicity

Forge is a professional tool for developers. Developers prefer information-dense interfaces that show more data with less clicking. We optimize for power users, not first-time visitors.

- Show summaries inline, expand on demand
- Use multi-column layouts by default
- Avoid modals for information display (modals block the background)
- Tables over cards when comparing items
- Cards over tables when items need visual hierarchy

### 2. Progressive Disclosure

Information density does not mean information overload. Start with the most important 20% of information and reveal the rest on demand.

- Primary: visible immediately (agent name, status, current action)
- Secondary: visible on hover or focus (metrics, timestamps, config summary)
- Tertiary: visible on click or expand (full configuration, event history, raw data)

### 3. Keyboard-First

Every action in the UI must be achievable without a mouse. Many developers live in keyboards-only workflows.

- Global shortcuts for navigation (e.g., `Ctrl+1` through `Ctrl+9` for pages)
- `Ctrl+K` / `Cmd+K` for command palette (search everything)
- `?` for keyboard shortcut overlay
- `Esc` to close any overlay, dismiss any modal
- `Tab` / `Shift+Tab` for sequential focus navigation
- `Enter` to confirm, `Esc` to cancel in dialogs

### 4. Real-Time by Default

Forge manages live agent processes. The UI must reflect current state, not stale snapshots.

- WebSocket-powered live updates for agent status, events, and metrics
- No manual refresh buttons (data streams automatically)
- Optimistic UI updates (show change immediately, reconcile if server disagrees)
- Visual indicators when real-time connection is interrupted

### 5. Consistent Visual Language

Every component should feel like it belongs to the same system.

- One typeface family (system font stack: -apple-system, system-ui, sans-serif)
- One spacing scale (TailwindCSS 4 default: 4px base unit)
- One color system (slate/neutral palette with accent colors per semantic meaning)
- One animation timing (150ms for micro-interactions, 300ms for layout changes)

### 6. Context Preservation

When the user navigates away and comes back, everything should be where they left it.

- Tab state persists across page navigations
- Scroll positions preserved within panes
- Filter and sort settings remembered per page
- Split view configurations saved

---

## Page Architecture

### Navigation Structure

```
Sidebar (persistent, collapsible)
 |
 +-- Dashboard                 /
 +-- Agents                    /agents
 |    +-- Agent Detail         /agents/:id
 |    +-- Agent Edit           /agents/:id/edit
 |    +-- New Agent            /agents/new
 +-- Sessions                  /sessions
 |    +-- Session Detail       /sessions/:id
 +-- Workflows                 /workflows
 |    +-- Workflow Detail      /workflows/:id
 |    +-- Workflow Editor      /workflows/:id/edit
 |    +-- New Workflow         /workflows/new
 +-- Skills                    /skills
 |    +-- Skill Detail         /skills/:id
 +-- Observability             /observe
 +-- Git                       /git
 +-- Code                      /code
 +-- Terminal                  /terminal
 +-- Settings                  /settings
     +-- General               /settings/general
     +-- MCP Servers           /settings/mcp
     +-- Hooks                 /settings/hooks
     +-- Notifications         /settings/notifications
     +-- Security              /settings/security
```

### Page Descriptions

#### Dashboard (`/`)

The landing page. At-a-glance system health and active work.

**Content**:
- Active agents with current status (running, idle, errored, paused)
- Recent sessions (last 10, with resume buttons)
- System health: CPU, memory, active WebSocket connections, SQLite size
- Active workflows with progress indicators
- Usage metrics: tokens consumed today, estimated cost, rate limit status
- Quick actions: create agent, start session, run workflow

**Layout**: Grid of cards. Top row: active agents (scrollable horizontal). Bottom: 2-column layout with recent sessions (left) and system health + metrics (right).

#### Agent Manager (`/agents`)

CRUD interface for agent configurations.

**Content**:
- Agent list with: name, model, status, last active, session count, circuit breaker state
- Filter by: status (all, running, idle, errored), model, tag
- Sort by: name, last active, session count
- Bulk actions: start, stop, duplicate, delete
- Preset browser: 9 built-in presets + imported presets
- Each agent row expands to show: system prompt preview, MCP servers, hooks, permissions

**Layout**: Table view (default) or card grid view (toggle). Filter bar at top. Create button in header.

#### Agent Detail (`/agents/:id`)

Deep view of a single agent.

**Content**:
- Agent configuration (editable inline)
- Session history for this agent
- Event stream (live if agent is running)
- Performance metrics (response times, token usage, error rate)
- Circuit breaker status with reset button
- CLAUDE.md editor (agent-specific context)
- MCP server configuration
- Hook configuration

**Layout**: Tabbed interface. Tabs: Overview | Sessions | Events | Config | CLAUDE.md

#### Session Browser (`/sessions`)

Search, browse, and manage all sessions across all agents.

**Content**:
- Session list with: agent name, start time, duration, message count, status, project directory
- Full-text search (FTS5) across session content
- Filter by: agent, project, date range, status (active, completed, errored)
- Sort by: start time, duration, message count
- Grouped by: project (default), agent, date
- Export options: JSON, Markdown, HTML
- Todo extraction: show TodoWrite items across sessions
- Resume button for continuable sessions

**Layout**: Two-column. Left: session list with search/filter. Right: session preview (messages, events).

#### Workflow Designer (`/workflows`)

Visual editor for multi-agent workflow DAGs.

**Content**:
- Workflow list with: name, status (draft, running, completed, failed), last run, step count
- Visual DAG editor:
  - Nodes represent agent steps (drag to position)
  - Edges represent dependencies (draw to connect)
  - Node inspector panel (click node to configure)
  - Step types: agent invocation, condition, parallel group, human approval
- Workflow templates (pre-built patterns):
  - Code Review: write -> lint -> test -> review
  - Feature Implementation: spec -> design -> implement -> test -> review
  - Bug Fix: reproduce -> diagnose -> fix -> verify
  - Security Audit: scan -> analyze -> report -> remediate
- Run history with step-by-step status
- Real-time execution view (nodes light up as they execute)

**Layout**: Full-width canvas with collapsible side panel for node inspector. Toolbar at top.

#### Skill Browser (`/skills`)

Catalog of available skills with search and install.

**Content**:
- Skill catalog: name, category, description, auto-activation triggers, quality score
- Search by: name, category, keyword
- Filter by: category (20 categories from plugins-plus-skills), quality score, installed/available
- Skill detail: full description, activation rules, dependencies, usage examples
- Install/uninstall toggle
- Auto-activation configuration (which contexts trigger which skills)
- Skill creator: generate new skills from templates

**Layout**: Grid of skill cards. Click to expand detail panel on right side.

#### Observability (`/observe`)

Real-time multi-agent monitoring dashboard.

**Content**:
- Swim-lane view: one column per active agent, events flow vertically in time
- Pulse chart: horizontal bars showing activity per agent over time
- Tool usage breakdown: which tools each agent is calling (with emoji indicators)
- Cost dashboard: real-time token usage, projected costs, budget remaining
- Event filter: by agent, event type, time range
- Chat transcript viewer: rendered messages with syntax highlighting
- Alert indicators: circuit breaker trips, rate limit approaches, error spikes

**Layout**: Full-width. Swim lanes take primary space. Pulse chart at top (collapsible). Side panel for event detail.

**Tool emoji system** (from hooks-observability):
| Tool | Emoji | Tool | Emoji |
|------|-------|------|-------|
| Bash | `>_` | Read | eye |
| Write | pencil | Edit | scissors |
| Glob | magnifying glass | Grep | search |
| TodoWrite | checkbox | WebSearch | globe |
| MCP tool | plug | Think | brain |

#### Git Panel (`/git`)

Git operations interface.

**Content**:
- Repository status: current branch, clean/dirty, ahead/behind
- Changed files list with diff viewer (inline and side-by-side)
- Branch list with search and switch capability
- Worktree management: list, create, switch, delete
- Staging area: stage/unstage individual files or hunks
- Commit interface: message editor, commit, push
- PR creation (via `gh` CLI integration)

**Layout**: Three-column when space allows. Left: file tree with change indicators. Center: diff viewer. Right: branch/worktree panel.

#### Code Viewer (`/code`)

File browser and viewer for the active project.

**Content**:
- File tree (collapsible sidebar)
- Code viewer with syntax highlighting (read-only)
- File search (fuzzy match, like Cmd+P)
- Preview panel for Markdown, images, PDFs
- File metadata: size, last modified, language, line count

**Layout**: Two-column. Left: file tree. Right: code/preview panel.

#### Terminal (`/terminal`)

Browser-based terminal access.

**Content**:
- PTY-backed terminal emulator via WebSocket
- Multiple terminal tabs
- Copy/paste support
- Scrollback buffer
- Terminal size auto-adjustment to viewport

**Layout**: Full-width terminal. Tab bar at top for multiple terminals.

#### Settings (`/settings`)

Application configuration.

**Sub-pages**:
- **General**: Default model, theme (dark/light/system), language, default project directory, startup behavior
- **MCP Servers**: List of configured MCP servers, add/edit/remove, connection status, test button
- **Hooks**: Hook configurations per event type, enable/disable, script editor
- **Notifications**: Notification preferences (desktop, sound, in-app), webhook URLs for remote notifications
- **Security**: Permission model defaults, allowed/blocked commands, file access rules, rate limit configuration

**Layout**: Left sidebar within settings for sub-page navigation. Content area on right.

---

## Layout System

### Multi-Pane Tabs with Split View

The primary layout paradigm is tabbed panes that can be split horizontally or vertically.

```
+--+-------------------------------------------+
|  |  Tab A  |  Tab B  |  Tab C  |             |
|  +-------------------------------------------+
|  |                    |                       |
|S |   Left Pane        |   Right Pane          |
|I |   (e.g., agent     |   (e.g., event        |
|D |    list)            |    stream)            |
|E |                    |                       |
|B |                    |                       |
|A |                    |                       |
|R +-------------------------------------------+
|  |  Status Bar (connection, active agents)    |
+--+-------------------------------------------+
```

**Split behaviors**:
- Drag the divider to resize panes
- Double-click divider to reset to 50/50
- Keyboard shortcut to toggle split: `Ctrl+\`
- Pane content is independent (different pages in each pane)

### Responsive Breakpoints

| Breakpoint | Width | Layout Adaptation |
|-----------|-------|-------------------|
| Desktop (large) | >= 1440px | Full multi-pane layout, sidebar expanded |
| Desktop (standard) | >= 1024px | Multi-pane layout, sidebar collapsed to icons |
| Tablet | >= 768px | Single pane, sidebar as overlay |
| Mobile | < 768px | Single pane, bottom navigation, simplified views |

Note: Forge is primarily a desktop tool. Mobile layouts are functional but not optimized for extended use.

---

## Component Patterns

### Cards

Used for: agent summaries, session previews, skill entries, workflow nodes.

```
+-----------------------------------------------+
|  Icon  Agent Name                   Status ●   |
|        Model: sonnet-4              Running     |
|                                                 |
|  Sessions: 42    Last: 5 min ago   Tokens: 12K |
+-----------------------------------------------+
```

- Fixed height within a grid, scroll content if overflow
- Status indicator as colored dot (green = running, yellow = idle, red = errored, gray = stopped)
- Click to navigate to detail
- Right-click or `...` menu for actions (edit, duplicate, delete)

### Tables

Used for: session lists, event logs, file lists, configuration entries.

- Sortable columns (click header to sort)
- Resizable columns (drag column border)
- Sticky header (remains visible on scroll)
- Row selection (click to select, Shift+click for range, Ctrl+click for multi)
- Virtual scrolling for large datasets (render only visible rows)

### Modals

Used sparingly: confirmation dialogs, destructive action warnings, quick create forms.

Rules:
- Never use modals for displaying information (use panels or inline expansion)
- Always closeable with `Esc`
- Always have a visible close button
- Focus trapped within modal while open
- Backdrop click closes (unless form has unsaved changes)

### Forms

Used for: agent configuration, workflow node settings, MCP server setup.

- Inline validation (validate on blur, show error immediately)
- Labels above inputs (not placeholder text as labels)
- Required fields marked with asterisk
- Default values pre-populated
- Save/Cancel buttons at bottom (sticky if form scrolls)
- Unsaved changes indicator in page title

### Charts

Used for: pulse charts, cost graphs, usage meters, performance timelines.

- Dark-theme-first color palette
- Tooltips on hover with exact values
- Responsive to container size
- Legend toggleable (click to show/hide series)
- Time range selector for time-series data

---

## Theming

### Dark Theme (Default)

```
Background (primary):    #0f172a (slate-900)
Background (secondary):  #1e293b (slate-800)
Background (tertiary):   #334155 (slate-700)
Surface:                 #1e293b (slate-800)
Border:                  #475569 (slate-600)
Text (primary):          #f8fafc (slate-50)
Text (secondary):        #94a3b8 (slate-400)
Text (muted):            #64748b (slate-500)
Accent (primary):        #3b82f6 (blue-500)
Accent (success):        #22c55e (green-500)
Accent (warning):        #eab308 (yellow-500)
Accent (error):          #ef4444 (red-500)
```

### Light Theme

```
Background (primary):    #ffffff (white)
Background (secondary):  #f8fafc (slate-50)
Background (tertiary):   #f1f5f9 (slate-100)
Surface:                 #ffffff (white)
Border:                  #e2e8f0 (slate-200)
Text (primary):          #0f172a (slate-900)
Text (secondary):        #475569 (slate-600)
Text (muted):            #94a3b8 (slate-400)
Accent (primary):        #2563eb (blue-600)
Accent (success):        #16a34a (green-600)
Accent (warning):        #ca8a04 (yellow-600)
Accent (error):          #dc2626 (red-600)
```

### Theme Switching

- System preference detection (`prefers-color-scheme`)
- Manual override in Settings
- Persisted to localStorage
- Applied via CSS custom properties for instant switching (no page reload)

---

## Keyboard Shortcuts Map

### Global

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + K` | Open command palette |
| `?` | Show keyboard shortcuts overlay |
| `Esc` | Close overlay / dismiss modal / deselect |
| `Ctrl/Cmd + 1` | Go to Dashboard |
| `Ctrl/Cmd + 2` | Go to Agents |
| `Ctrl/Cmd + 3` | Go to Sessions |
| `Ctrl/Cmd + 4` | Go to Workflows |
| `Ctrl/Cmd + 5` | Go to Skills |
| `Ctrl/Cmd + 6` | Go to Observability |
| `Ctrl/Cmd + 7` | Go to Git |
| `Ctrl/Cmd + 8` | Go to Code |
| `Ctrl/Cmd + 9` | Go to Terminal |
| `Ctrl/Cmd + ,` | Go to Settings |
| `Ctrl/Cmd + \` | Toggle split view |
| `Ctrl/Cmd + B` | Toggle sidebar |

### Agent Manager

| Shortcut | Action |
|----------|--------|
| `N` | Create new agent |
| `Enter` | Open selected agent |
| `E` | Edit selected agent |
| `D` | Duplicate selected agent |
| `Delete` / `Backspace` | Delete selected agent (with confirmation) |
| `/` | Focus search |
| `J` / `K` | Navigate list down / up |

### Session Browser

| Shortcut | Action |
|----------|--------|
| `/` | Focus search |
| `R` | Resume selected session |
| `X` | Export selected session |
| `J` / `K` | Navigate list down / up |
| `Enter` | Open selected session detail |

### Workflow Designer

| Shortcut | Action |
|----------|--------|
| `A` | Add node |
| `C` | Connect nodes (enter edge drawing mode) |
| `Delete` | Delete selected node or edge |
| `Ctrl/Cmd + Z` | Undo |
| `Ctrl/Cmd + Shift + Z` | Redo |
| `Ctrl/Cmd + S` | Save workflow |
| `F5` / `Ctrl/Cmd + Enter` | Run workflow |
| `Space` | Toggle node expand/collapse |
| `+` / `-` | Zoom in / out |
| `0` | Fit to screen |

### Observability

| Shortcut | Action |
|----------|--------|
| `Space` | Pause / resume auto-scroll |
| `F` | Toggle fullscreen swim-lane view |
| `1-9` | Focus on agent N |
| `C` | Clear events |

### Git Panel

| Shortcut | Action |
|----------|--------|
| `S` | Stage selected file |
| `U` | Unstage selected file |
| `D` | Show diff for selected file |
| `Enter` | Open file in code viewer |
| `Ctrl/Cmd + Enter` | Commit staged changes |

### Terminal

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + T` | New terminal tab |
| `Ctrl/Cmd + W` | Close current terminal tab |
| `Ctrl/Cmd + Shift + [` | Previous terminal tab |
| `Ctrl/Cmd + Shift + ]` | Next terminal tab |

---

## Mobile and Responsive Considerations

Forge is a desktop-first application, but provides functional mobile access for monitoring.

### Mobile-Optimized Views

- **Dashboard**: Stack cards vertically, show only active agents and last 5 sessions
- **Observability**: Simplified pulse chart, single-agent event stream (no swim lanes)
- **Agent list**: Card view only (no table), simplified actions

### Not Available on Mobile

- Workflow visual editor (requires canvas interaction)
- Terminal (requires keyboard)
- Code viewer (not useful on small screens)
- Split view

### Responsive Patterns

- Sidebar becomes bottom tab bar on mobile
- Tables become card lists
- Multi-pane layouts collapse to single pane with navigation
- Charts simplify (fewer data points, no zoom interaction)
- Touch targets minimum 44x44 pixels on mobile
