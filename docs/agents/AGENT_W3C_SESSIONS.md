# Agent W3-C: Session Detail & Output Storage

> You are Agent W3-C. Your job: make session history useful. Store output blocks during runs and render them on the sessions page so users can review past agent work.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
frontend/src/routes/sessions/+page.svelte          — FULL FILE, study current sessions UI
frontend/src/routes/+page.svelte                   — study OutputBlock type and rendering
crates/forge-api/src/routes/sessions.rs            — session API endpoints
crates/forge-db/src/repos/sessions.rs              — SessionRepo, Session struct
crates/forge-db/src/repos/events.rs                — EventRepo (stores all events)
stat-qou-plan/REVISED_PLAN.md                      — what we're doing and why
```

## Step 2: Understand Current State

The sessions page already has:
- Kanban board view (created, running, completed, failed)
- List view with agent names
- Export functionality (JSON, Markdown, HTML)
- Resume session button

What's missing:
- Viewing the actual output of a past session (the conversation, tool calls, thinking)
- The output is in the events table but not rendered on the sessions page

## Step 3: Add Output Rendering to Session Detail

The sessions page likely has a detail panel or modal when you click a session. Enhance it to:

1. Fetch events for the session: `GET /api/v1/events?session_id=XXX` or similar
2. Filter for ProcessOutput events
3. Render them using the same OutputBlock format as the Run page

If no events endpoint exists for session-filtered events, check what's available in the API routes. You may need to add a simple endpoint.

### Backend: Add session events endpoint (if needed)

In `crates/forge-api/src/routes/sessions.rs`, add:

```rust
/// GET /api/v1/sessions/:id/events — fetch output events for a session
async fn get_session_events(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<EventRecord>>, StatusCode> {
    let events = state.event_repo
        .list_by_session(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}
```

Register the route in the sessions router.

Check if `EventRepo` already has `list_by_session` — if not, add it:

```rust
pub fn list_by_session(&self, session_id: &str) -> ForgeResult<Vec<EventRecord>> {
    let conn = self.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, event_type, session_id, agent_id, data_json, created_at
         FROM events WHERE session_id = ?1 ORDER BY created_at ASC"
    )?;
    // ... map rows to EventRecord
}
```

### Frontend: Render session output

In the sessions page, when a session is selected/expanded, fetch and render output:

```svelte
async function loadSessionOutput(sessionId: string) {
  const res = await fetch(`/api/v1/sessions/${sessionId}/events`);
  const events = await res.json();
  sessionOutput = events
    .filter(e => e.event_type === 'ProcessOutput')
    .map(e => ({
      kind: e.data?.kind ?? 'assistant',
      content: e.data?.content ?? '',
    }));
}
```

Render using the same block format as the Run page:
- Assistant blocks: render as Markdown
- Tool use/result: collapsible details
- Thinking: dimmed collapsible

Use `marked` + `DOMPurify` for markdown rendering (already imported in the Run page — check if available here, if not import them).

## Step 4: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-api 2>&1         # all tests pass
cd frontend && pnpm build 2>&1       # must build cleanly
```

## Rules

- Modify `frontend/src/routes/sessions/+page.svelte` — session output rendering
- Modify `crates/forge-api/src/routes/sessions.rs` — add events endpoint if needed
- Modify `crates/forge-db/src/repos/events.rs` — add list_by_session if needed
- Do NOT modify the Run page (+page.svelte at root) — Agent W3-A handles that
- Do NOT modify middleware.rs or run.rs — Agent W3-B handles those
- Do NOT modify skills, analytics, or layout files
- Do NOT modify existing tests — only add new ones
- Commit with: `feat(sessions): add output viewing for past sessions`

## Report
```
STATUS: done | blocked
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
NEW_ENDPOINTS: [list]
ISSUES: [any]
```
