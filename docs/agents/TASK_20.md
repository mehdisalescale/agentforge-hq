# TASK 20 — Cost tracking

**Status:** pending
**Priority:** medium
**Track:** Phase B — safety

---

## Context

No visibility into how much agent runs cost. The stream-json `result` event from Claude CLI includes a `cost_usd` field. We need to capture it, store it, and expose it.

## Task

1. **Parse cost from stream output** — In `crates/forge-process/src/stream_event.rs`, check if `ResultPayload` has a `cost_usd` field. If not, add it:
   ```rust
   pub struct ResultPayload {
       pub result: String,
       pub cost_usd: Option<f64>,
       // ... other fields
   }
   ```

2. **Store cost on session** — Add a DB migration (`migrations/0002_add_cost.sql`):
   ```sql
   ALTER TABLE sessions ADD COLUMN cost_usd REAL DEFAULT 0.0;
   ```
   Add `update_cost` method to `SessionRepo`.

3. **Update cost on ProcessCompleted** — In the run handler's spawned task, after receiving a Result event with cost, call `session_repo.update_cost(&sid, cost)`.

4. **Expose in API** — `Session` struct already has `Serialize`; the new `cost_usd` field will appear automatically.

5. **Budget enforcement** (optional stretch):
   - Read `FORGE_BUDGET_WARN` and `FORGE_BUDGET_LIMIT` from env
   - On cost update, check against limits
   - Emit `BudgetWarning` or `BudgetExceeded` events

6. **Frontend** — Show cost in Sessions detail view.

## Files to edit

- `migrations/0002_add_cost.sql` (new)
- `crates/forge-db/src/migrations.rs` (register new migration)
- `crates/forge-db/src/repos/sessions.rs` (add cost_usd field, update_cost method)
- `crates/forge-process/src/stream_event.rs` (check cost_usd field)
- `crates/forge-process/src/runner.rs` (capture cost from result)
- `crates/forge-api/src/routes/run.rs` (pass cost to session)
- `frontend/src/routes/sessions/+page.svelte` (show cost)
- `frontend/src/lib/api.ts` (add cost_usd to Session type)

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
cd frontend && pnpm build
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: ResultPayload already had cost_usd. Added migration 0002_add_cost.sql (ALTER sessions ADD cost_usd REAL DEFAULT 0.0); Migrator applies 0002 when current < 2; Session struct + cost_usd, SessionRepo update_cost, SELECTs/row_to_session updated; run handler on StreamJsonEvent::Result(payload) with cost_usd calls session_repo.update_cost; Session API exposes cost_usd; frontend Session type + cost_usd, sessions detail shows Cost when present. Budget enforcement (FORGE_BUDGET_*) not implemented (optional stretch).
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [ ] Notes:
