# Agent E3-A: Backend Config — DB Migration + Agent Model

## Goal

Add `backend_type` field to agents so each agent can specify which execution backend to use (claude, hermes, openclaw). Create DB migration, update Agent model, and wire into API.

## Context

**Already done:** `ProcessBackend` trait, `ClaudeBackend` adapter, and `BackendRegistry` exist in:
- `crates/forge-process/src/backend.rs` — trait + registry
- `crates/forge-process/src/claude_backend.rs` — Claude adapter

**This agent adds:** The data model + API layer so users can configure backends per agent.

Corresponds to **E3-S4** (Agent Backend Configuration) from `/docs/product/epics/E3_HEXAGONAL_BACKENDS.md`.

## Files to Modify

### 1. New migration: `crates/forge-db/src/migrations.rs`

Add migration (next number after existing ones):
```sql
ALTER TABLE agents ADD COLUMN backend_type TEXT NOT NULL DEFAULT 'claude';
```

This is backward-compatible — all existing agents default to 'claude'.

### 2. Update Agent model: `crates/forge-agent/src/model.rs`

Add to `Agent` struct:
```rust
pub backend_type: String,  // "claude", "hermes", "openclaw"
```

Add to `NewAgent`:
```rust
pub backend_type: Option<String>,  // defaults to "claude" if None
```

Add to `UpdateAgent`:
```rust
pub backend_type: Option<String>,
```

### 3. Update AgentRepo: `crates/forge-db/src/repos/agents.rs`

- `create()`: INSERT now includes `backend_type` column, default to "claude" if None
- `update()`: SET `backend_type` if provided
- `get()` / `list()`: SELECT includes `backend_type`
- Row mapping: extract `backend_type` from row

### 4. Update API routes: `crates/forge-api/src/routes/agents.rs`

No route changes needed — `NewAgent` / `UpdateAgent` are already `Json<T>` deserialized, so adding the field flows through automatically.

### 5. Update persona hire flow: `crates/forge-api/src/routes/personas.rs`

When hiring a persona, set `backend_type: Some("claude".to_string())` in the `NewAgent` construction (or leave as None to use default).

### 6. Frontend: `frontend/src/routes/agents/+page.svelte`

Add a dropdown to the agent create/edit form:
```svelte
<select bind:value={newAgent.backend_type}>
    <option value="claude">Claude CLI</option>
    <option value="hermes" disabled>Hermes (coming soon)</option>
    <option value="openclaw" disabled>OpenClaw (coming soon)</option>
</select>
```

Show `backend_type` in the agent list table as a badge.

## Do NOT Modify
- `crates/forge-process/src/backend.rs` — already done
- `crates/forge-process/src/claude_backend.rs` — already done
- `crates/forge-api/src/middleware.rs` — Agent E3-B handles SpawnMiddleware
- `crates/forge-api/src/routes/run.rs` — Agent E3-B handles that

## Verification
```bash
cargo check                # compiles
cargo test -p forge-db     # migration runs, agent CRUD with backend_type works
cargo test -p forge-agent  # model tests pass
```

## Zero Warnings Policy
All modified files must produce zero warnings.
