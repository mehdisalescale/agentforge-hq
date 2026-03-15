# AgentForge HQ — Gap Analysis

> What's broken, disconnected, or misleading.
> Date: 2026-03-15

---

## Critical Gaps (The app promises things it can't do)

### 1. Budget is decorative
- Companies have a `budget_limit` field
- CostTracker exists with warn/limit thresholds
- **But**: CostTracker reads from env vars, not from company budget
- **Result**: A user sets $500 budget on a company, agents ignore it completely
- **Fix**: Wire CostTracker to read company budget, deduct from it per run

### 2. Approvals don't block anything
- Users create approval requests
- Other users can approve/reject
- **But**: Agents run regardless of pending approvals
- **Result**: The approvals page feels like a todo list with no consequence
- **Fix**: Add approval-required flag on certain actions (budget changes, agent creation, etc.)

### 3. Goals have no influence
- Goals exist with status tracking
- **But**: No agent reads goals, no output references them
- **Result**: Goals page is a standalone notepad
- **Fix**: Inject active goals into agent context so they know what the company is trying to achieve

### 4. Sidebar page status
Resolution status as of Wave 5:

- **Skills**: RESOLVED — has category filter + content display
- **Memory**: RESOLVED — functional CRUD UI
- **Workflows**: PARTIAL — has UI, execution works
- **Hooks**: STUB — CRUD exists, HookReceiver functional
- **Schedules**: STUB — CRUD exists, scheduler runs
- **Settings**: RESOLVED — displays runtime configuration
- **Analytics**: RESOLVED — summary cards, agent breakdown, company filter

---

## Connectivity Gaps (Pieces exist but aren't wired together)

### 5. Skills are invisible to users
- 30 skills loaded, auto-injected based on task type
- **But**: No UI to see what skills exist, what they do, or which ones were injected
- **Result**: The smartest feature in the app is completely hidden
- **Fix**: Build Skills page showing all loaded skills with content, and show "injected skills" badge on run output

### 6. Memory doesn't persist across sessions
- MemoryRepo has full CRUD
- **But**: Memory is never read during agent runs, never written by agents
- **Result**: The Memory page will always be empty
- **Fix**: Inject relevant memories into agent context, let agents write learnings back

### 7. Sessions can't be inspected
- Sessions page lists past runs with metadata
- **But**: Can't view the actual output, tool calls, or thinking
- **Result**: Session history is useless for learning from past runs
- **Fix**: Store output blocks, render them on session detail page

### 8. Org hierarchy is visual only
- Org chart shows who reports to whom
- **But**: No permission scoping (an engineer agent can do anything a CEO agent can)
- **Result**: Org chart is a pretty picture with no functional meaning
- **Fix**: (E4) Department-scoped permissions, reporting chain for approvals

---

## UX Gaps

### 9. No feedback after hiring
- User hires a persona → redirect to personas list
- **But**: No confirmation of what was created, no link to the new agent
- **Fix**: Show success toast with "View agent" link

### 10. Agent names are cryptic after hire
- Hired persona becomes "AI-Engineer-Agent" — the hyphenated slug
- **But**: User doesn't see the persona name or description on the Agents page
- **Fix**: Show persona name/description on agent cards, or use display name

### 11. Run page doesn't explain what happens
- User hits Run, output appears (or doesn't)
- **But**: No indication of which skills were injected, what task type was detected, whether security scan passed
- **Fix**: Show a small metadata panel: "Detected: BugFix | Skills injected: systematic-debugging, TDD | Security: passed"

### 12. No error guidance
- If claude CLI isn't in PATH, run silently fails
- **Fix**: Health check on startup, show banner if CLI not found

---

## Technical Debt

### 13. Duplicate skill names
- `code-review` exists in both plugins/ and superpowers/ — both get loaded
- Should deduplicate or namespace

### 14. No auth whatsoever
- Anyone on the network can access everything
- Fine for local dev, not for any shared deployment

### 15. Hardcoded model list
- ClaudeBackend hardcodes model names
- Should come from config or discovery

---

## Priority Matrix

| Gap | Impact | Effort | Priority |
|-----|--------|--------|----------|
| Empty sidebar pages (show "coming soon") | High UX | Low | Do first |
| Skills page (show what's loaded) | High UX | Medium | Do first |
| Run metadata panel (show injections) | High UX | Low | Do first |
| Budget wiring | High trust | Medium | Do second |
| Session output viewing | High utility | Medium | Do second |
| Memory injection | High value | High | Do third |
| Approval blocking | Medium trust | Medium | Do third |
| Goal injection | Medium value | Low | Do third |
| Workflows engine | High value | Very high | Later |
