# Agent UW-C: Update MCP Server + main.rs for UnitOfWork

## Goal

Update `crates/forge-mcp-bin/src/main.rs` to use `UnitOfWork` instead of 17 individual repo fields, and ensure `crates/forge-app/src/main.rs` wiring is correct.

## Context

The MCP server (`ForgeMcp` struct) currently has the same pattern as AppState — individual repo fields:
```rust
struct ForgeMcp {
    agent_repo: Arc<AgentRepo>,
    session_repo: Arc<SessionRepo>,
    // ... more repos ...
}
```

After UW-A creates `UnitOfWork`, this should become:
```rust
struct ForgeMcp {
    uow: Arc<UnitOfWork>,
    event_bus: Arc<EventBus>,
    // safety if needed
}
```

## Files to Modify

### `crates/forge-mcp-bin/src/main.rs`

1. **ForgeMcp struct**: Replace individual repo fields with `uow: Arc<UnitOfWork>`
2. **ForgeMcp::new()**: Simplify from 9+ params to `new(uow: Arc<UnitOfWork>, event_bus: Arc<EventBus>)`
3. **All tool implementations**: Change `self.agent_repo.xxx()` → `self.uow.agent_repo.xxx()`
4. **main() function**: Construct `UnitOfWork` from `DbPool`, pass to `ForgeMcp::new()`
5. **Remove** `#[allow(clippy::too_many_arguments)]` — no longer needed with 2-3 params

### `crates/forge-mcp-bin/Cargo.toml`
- Ensure `forge-db` dependency is present (for `UnitOfWork` import)

### `crates/forge-app/src/main.rs`
- Verify the wiring that UW-A did:
  - `UnitOfWork::new(pool)` is constructed
  - `AppState::new(uow, event_bus, safety)` is called
  - `BatchWriter` still gets its own connection from pool
  - Seed operations use `uow.persona_repo`, `uow.company_repo`, etc.
  - Schedule repo for cron still works (check if it was `Arc<ScheduleRepo>` separately)

## Depends On
- **Agent UW-A must complete first**
- Can run in parallel with UW-B (different crates, no file overlap)

## Verification
```bash
cargo check -p forge-mcp-bin  # compiles
cargo check -p forge-app      # compiles
cargo build                    # full workspace builds
```

## Zero Warnings Policy
All modified files must produce zero warnings.
