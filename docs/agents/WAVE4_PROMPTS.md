# Wave 4 Agent Prompts

> 4 frontend agents. All create/modify frontend files. No backend changes. No file conflicts between agents.
> **Prerequisite**: Wave 3 gate passed (118 tests, zero warnings, clippy clean).

---

## Agent J — Worktree UI + Integration Test (WT3+T1)

**Files (exclusive):** `frontend/src/routes/sessions/+page.svelte` (modify), `tests/` (NEW)

```
You are Agent J. Your task: add worktree info to the Sessions page and write an integration test.

## Context

This is a Rust/Axum + Svelte 5 project. The frontend lives in `frontend/`.
Svelte 5 runes: use `$state`, `$derived`, `$effect` (NOT `let` for reactive state).
CSS: use CSS variables from `frontend/src/app.css`: `--bg`, `--surface`, `--border`, `--text`, `--muted`, `--accent`.

The backend has a forge-git crate with worktree support. Sessions have a `directory` field.
The git worktree API is not exposed via HTTP yet, so for now:
- Show the session's `directory` field prominently in the detail panel
- Add a "Worktree" badge if the directory contains `.claude/worktrees/`
- Add placeholder buttons for "Merge" and "Cleanup" (disabled, tooltip "Coming soon — requires worktree API")

## Your files

MODIFY: `frontend/src/routes/sessions/+page.svelte`

Current state: 265 lines, two-pane layout. Left pane lists sessions. Right pane shows detail with ID, Agent ID, Directory, Status, Cost, Created, and Resume/Export buttons.

Changes:
1. Add a "Worktree" badge next to the status badge in the session list items IF `s.directory` contains `.claude/worktrees/`
2. In the detail panel, make the Directory field more prominent (larger font, full path visible)
3. Add "Merge" and "Cleanup" buttons in `detail-actions` div, disabled with title="Coming soon"
4. Convert remaining `let` vars to `$state` runes for consistency (sessions, sessionsError, selectedId, detail, detailError, agents)

CREATE: `tests/integration_test.sh`

A bash script that:
1. Starts the forge binary in background (`cargo run --release &`)
2. Waits for health endpoint (`curl -s http://127.0.0.1:4173/api/v1/health`)
3. Creates an agent via API (`curl -X POST .../agents`)
4. Lists agents (verify it appears)
5. Creates a session via the run endpoint
6. Lists sessions (verify it appears)
7. Cleans up (kill server, exit with status)

Make the script executable, with proper error handling and cleanup on exit (trap).

## Style guide

- Match existing CSS patterns (card, badge, btn classes from app.css)
- Status badge colors: running=#60a5fa, completed=#86efac, failed=#f87171
- Worktree badge: use a subtle blue/purple background like `rgba(96, 165, 250, 0.15)` with `#60a5fa` text

## Verify

```bash
cd frontend && pnpm build
# Integration test:
chmod +x tests/integration_test.sh
```

## Report

When done, output a summary of what you changed and created.
```

---

## Agent K — Memory UI + Hook UI (ME4+HK3)

**Files (exclusive):** `frontend/src/routes/memory/+page.svelte` (NEW), `frontend/src/routes/hooks/+page.svelte` (NEW), `frontend/src/lib/api.ts` (add memory + hook API functions)

```
You are Agent K. Your task: build the Memory and Hooks management pages.

## Context

This is a Rust/Axum + Svelte 5 project. The frontend lives in `frontend/`.
Svelte 5 runes: use `$state`, `$derived`, `$effect` (NOT `let` for reactive state).
CSS: use CSS variables from `frontend/src/app.css`: `--bg`, `--surface`, `--border`, `--text`, `--muted`, `--accent`.
Use existing CSS classes: `btn`, `btn-primary`, `btn-ghost`, `badge`, `card`, `modal-backdrop`, `modal`, `message`, `error`, `muted`, `page-header`, `empty-state`.

## Backend API (already implemented)

### Memory API
- `GET    /api/v1/memory?limit=50&offset=0`  → `Memory[]`
- `GET    /api/v1/memory?q=search_term`       → `Memory[]` (FTS search)
- `POST   /api/v1/memory`                     → `Memory` (body: `NewMemory`)
- `GET    /api/v1/memory/:id`                 → `Memory`
- `PUT    /api/v1/memory/:id`                 → `Memory` (body: `UpdateMemory`)
- `DELETE /api/v1/memory/:id`                 → 204

Types:
```ts
interface Memory {
  id: string;
  category: string;
  content: string;
  confidence: number;       // 0.0–1.0
  source_session_id: string | null;
  created_at: string;
  updated_at: string;
}
interface NewMemory {
  category?: string;
  content: string;
  confidence?: number;
  source_session_id?: string;
}
interface UpdateMemory {
  content?: string;
  category?: string;
  confidence?: number;
}
```

### Hooks API
- `GET    /api/v1/hooks`          → `Hook[]`
- `POST   /api/v1/hooks`         → `Hook` (body: `NewHook`)
- `GET    /api/v1/hooks/:id`     → `Hook`
- `PUT    /api/v1/hooks/:id`     → `Hook` (body: `UpdateHook`)
- `DELETE /api/v1/hooks/:id`     → 204

Types:
```ts
interface Hook {
  id: string;
  name: string;
  event_type: string;     // e.g. "ProcessStarted", "ProcessCompleted"
  timing: string;         // "pre" | "post"
  command: string;
  enabled: boolean;
  created_at: string;
}
interface NewHook {
  name: string;
  event_type: string;
  timing: string;
  command: string;
}
interface UpdateHook {
  name?: string;
  command?: string;
  enabled?: boolean;
}
```

## Step 1: Add API functions to `frontend/src/lib/api.ts`

Add these functions at the bottom of the file (after the existing Workflows section):

```ts
// --- Memory (Wave 4) ---
export interface Memory { ... }
export interface NewMemory { ... }
export interface UpdateMemory { ... }

export async function listMemories(params?: { q?: string; limit?: number; offset?: number }): Promise<Memory[]> { ... }
export async function getMemory(id: string): Promise<Memory> { ... }
export async function createMemory(data: NewMemory): Promise<Memory> { ... }
export async function updateMemory(id: string, data: UpdateMemory): Promise<Memory> { ... }
export async function deleteMemory(id: string): Promise<void> { ... }

// --- Hooks (Wave 4) ---
export interface Hook { ... }
export interface NewHook { ... }
export interface UpdateHook { ... }

export async function listHooks(): Promise<Hook[]> { ... }
export async function createHook(data: NewHook): Promise<Hook> { ... }
export async function updateHook(id: string, data: UpdateHook): Promise<Hook> { ... }
export async function deleteHook(id: string): Promise<void> { ... }
```

Follow the exact same pattern as the existing agent CRUD functions. Use `handleResponse<T>`.

## Step 2: Create `frontend/src/routes/memory/+page.svelte`

Full CRUD page with:
- Header: "Memory" + "Add Memory" button
- Search bar (input field, queries `?q=...`)
- Card grid showing memories (category badge, content, confidence bar, timestamp)
- Click card → expand or modal with full content + edit form
- Edit: inline content textarea, category input, confidence slider (0–100%)
- Delete: confirmation dialog (like agents page pattern)
- Empty state: "No memories yet. Memories are extracted from session transcripts."
- Loading state: "Loading memories…"

## Step 3: Create `frontend/src/routes/hooks/+page.svelte`

Full CRUD page with:
- Header: "Hooks" + "New Hook" button
- Table/card list of hooks (name, event_type, timing badge, command preview, enabled toggle)
- Create modal: name input, event_type select (options: ProcessStarted, ProcessCompleted, ProcessFailed, SessionCreated, SessionUpdated, HookStarted, HookCompleted, SubAgentRequested, SubAgentStarted, SubAgentCompleted, SubAgentFailed), timing select (pre/post), command textarea
- Edit: same modal, pre-filled
- Enable/disable: toggle switch, calls `updateHook(id, { enabled: !hook.enabled })`
- Delete: confirmation dialog
- Empty state: "No hooks yet. Hooks run shell commands when events occur."

## Style guide

- Use `$state` runes for ALL reactive state
- Use `onMount` for initial data loading
- Match the Agents page patterns (modal, cards, form layout, delete confirmation)
- Confidence as a colored bar: red (<0.3), yellow (0.3–0.7), green (>0.7)
- Timing badge colors: pre=blue (#60a5fa), post=green (#86efac)
- Enabled toggle: use a styled checkbox or switch

## Verify

```bash
cd frontend && pnpm build
```

No TypeScript errors, no build warnings.

## Report

When done, output a summary of what you created.
```

---

## Agent L — Multi-Agent Dashboard (SA4+SA5)

**Files (exclusive):** `frontend/src/routes/+page.svelte` (modify), `frontend/src/routes/+layout.svelte` (minor update)

```
You are Agent L. Your task: add sub-agent progress tracking to the Dashboard and update the layout statusbar.

## Context

This is a Rust/Axum + Svelte 5 project. The frontend lives in `frontend/`.
Svelte 5 runes: use `$state`, `$derived`, `$effect` (NOT `let` for reactive state).
CSS: use CSS variables from `frontend/src/app.css`.

The backend emits SubAgent* events via WebSocket:
- `SubAgentRequested` — { parent_session_id, sub_agent_id, prompt, timestamp }
- `SubAgentStarted` — { parent_session_id, sub_agent_id, session_id, timestamp }
- `SubAgentCompleted` — { parent_session_id, sub_agent_id, session_id, timestamp }
- `SubAgentFailed` — { parent_session_id, sub_agent_id, error, timestamp }

The `AgentPreset` type now includes `'Coordinator'` (10th preset).

## Your files

MODIFY: `frontend/src/routes/+page.svelte`

Current state: 430 lines. Dashboard with Run form (agent select, prompt, directory) + streaming output panel. Uses `let` vars (NOT $state runes) for most state. WebSocket handles ProcessOutput and ProcessLifecycle events.

Changes:

### 1. Convert to Svelte 5 runes
Replace all `let` state variables with `$state()`:
- `agents`, `agentsError`, `selectedAgentId`, `prompt`, `directory`
- `running`, `runError`, `outputBlocks`, `streamStatus`, `streamStatusDetail`
- `currentSessionId`, `ws`, `wsReconnectTimer`, `wsReconnectDelay`

Replace `$:` reactive statements with `$derived()`.

### 2. Add sub-agent tracking
Add a new state section:
```ts
interface SubAgentStatus {
  agentId: string;
  status: 'requested' | 'running' | 'completed' | 'failed';
  sessionId?: string;
  prompt?: string;
  error?: string;
  timestamp: string;
}
let subAgents = $state<SubAgentStatus[]>([]);
```

In the WebSocket `onmessage` handler, add cases for SubAgent events:
- `SubAgentRequested` → push to subAgents with status 'requested'
- `SubAgentStarted` → update status to 'running', store session_id
- `SubAgentCompleted` → update status to 'completed'
- `SubAgentFailed` → update status to 'failed', store error

### 3. Add sub-agent progress panel
Below the stream output section, add a "Sub-agents" section that only shows when `subAgents.length > 0`:
```html
<section class="subagents-section">
  <h2>Sub-agents</h2>
  <div class="subagent-grid">
    {#each subAgents as sa}
      <div class="subagent-card" class:requested={sa.status === 'requested'} ...>
        <span class="subagent-id">{sa.agentId.slice(0,8)}…</span>
        <span class="status-badge {sa.status}">{sa.status}</span>
        {#if sa.prompt}<p class="subagent-prompt">{sa.prompt.slice(0,80)}</p>{/if}
        {#if sa.error}<p class="subagent-error">{sa.error}</p>{/if}
      </div>
    {/each}
  </div>
  <p class="subagent-summary">
    {subAgents.filter(s => s.status === 'completed').length}/{subAgents.length} completed
  </p>
</section>
```

### 4. Clear sub-agents on new run
In the `clearStream()` function, also clear `subAgents = []`.

MODIFY: `frontend/src/routes/+layout.svelte`

Update the statusbar text from "Phase 1" to "v0.4.0-dev" and from "Run + Sessions UI (Agent F)" to "Multi-agent orchestrator".

## Style guide

- Sub-agent card: compact, uses --surface background
- Status colors: requested=#71717a (muted), running=#60a5fa (blue), completed=#86efac (green), failed=#f87171 (red)
- Summary text: small, muted, right-aligned
- Grid: `grid-template-columns: repeat(auto-fill, minmax(200px, 1fr))`
- Keep the `on:click` / `on:*` event handlers (don't change to onclick unless the rest of the file uses it)

## Important

Do NOT modify `frontend/src/lib/api.ts` — Agent K is modifying that file.
The `AgentPreset` type in api.ts currently doesn't include 'Coordinator' but that's fine — it's just a display type.

## Verify

```bash
cd frontend && pnpm build
```

## Report

When done, output a summary of what you changed.
```

---

## Agent M — Polish (SA6+P1+P2+P3)

**Files (exclusive):** `frontend/src/routes/skills/+page.svelte` (modify), `frontend/src/routes/workflows/+page.svelte` (modify), `frontend/src/routes/settings/+page.svelte` (modify), `frontend/src/routes/agents/+page.svelte` (modify styles only)

```
You are Agent M. Your task: polish the frontend — improve skills/workflows/settings/agents pages.

## Context

This is a Rust/Axum + Svelte 5 project. The frontend lives in `frontend/`.
Svelte 5 runes: use `$state`, `$derived`, `$effect` (NOT `let` for reactive state).
CSS: use CSS variables from `frontend/src/app.css`.

## Your files

### 1. MODIFY: `frontend/src/routes/skills/+page.svelte`

Current state: 95 lines. Basic list of skills. Already uses $state runes.

Improvements:
- Add tag pills for each skill (parse `skill.parameters_json` — it contains a JSON string with `tags` array)
- Add usage count display per skill
- Add expandable content preview (show first 2 lines of `skill.content`, click to expand full)
- Add category filter dropdown (extract unique categories from loaded skills)
- Better card layout matching agents page style

### 2. MODIFY: `frontend/src/routes/workflows/+page.svelte`

Current state: 87 lines. Basic list. Already uses $state runes.

Improvements:
- Better empty state messaging: "No workflows yet. Workflows are sequences of agent tasks."
- Add a visual placeholder showing what a workflow will look like (a simple diagram in CSS)
- Match card styling from agents page

### 3. MODIFY: `frontend/src/routes/settings/+page.svelte`

Current state: 14 lines. Just says "Coming soon."

Replace with a useful settings display page that shows current configuration:
- Read-only config display (all FORGE_* env vars with their descriptions)
- Show current values from the health endpoint (`GET /api/v1/health` returns uptime)
- System info section: version (hardcode "0.4.0-dev"), database path (show FORGE_DB_PATH default)
- Use the `$state` pattern for loading state
- Add a "Reload" button that re-fetches health

### 4. MODIFY: `frontend/src/routes/agents/+page.svelte`

Current state: 258 lines. Full CRUD with $state runes. Already well-structured.

Minor improvements only:
- Add `Coordinator` to the PRESETS list in the form's select dropdown (it's the 10th preset)
- Add a domain badge concept: map presets to domains (CodeWriter/Refactorer → "code", Reviewer/Tester/SecurityAuditor → "quality", Architect/Documenter/Explorer → "ops", Coordinator → "orchestration")
- Show domain badge in agent cards next to the preset badge

## Important

Do NOT modify:
- `frontend/src/routes/+page.svelte` — Agent L is modifying that
- `frontend/src/routes/sessions/+page.svelte` — Agent J is modifying that
- `frontend/src/routes/memory/` or `frontend/src/routes/hooks/` — Agent K is creating those
- `frontend/src/lib/api.ts` — Agent K is modifying that

## Style guide

- Tag pills: small rounded badges, use subtle accent colors
- Domain badges: code=#60a5fa, quality=#86efac, ops=#fbbf24, orchestration=#c084fc
- Keep all pages consistent with existing card/badge/form patterns
- Loading states: show "Loading…" text with muted color

## Verify

```bash
cd frontend && pnpm build
```

## Report

When done, output a summary of what you changed.
```

---

## Verification Gate (after all 4 agents complete)

```bash
cd frontend && pnpm build           # Frontend compiles
cargo build --workspace              # Backend still compiles
cargo test --workspace               # All tests pass
cargo clippy --workspace             # Clean
```
