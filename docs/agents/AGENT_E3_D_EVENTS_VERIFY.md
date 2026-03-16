# Agent E3-D: Event Normalization + Final Verification

## Goal

Ensure all backends emit normalized ForgeEvent variants and verify the full E3 integration works end-to-end. Update docs and CLAUDE.md.

## Context

Corresponds to **E3-S6** (Event Normalization) and final verification from the epic doc.

Currently only `ClaudeBackend` exists. The event normalization layer ensures that when Hermes/OpenClaw backends are added later, they'll emit the same `ForgeEvent` variants as Claude — so the UI, BatchWriter, and analytics all work without changes.

## Files to Modify

### 1. `crates/forge-process/src/stream_event.rs`

Add a normalization function that maps backend-specific events to `ForgeEvent`:
```rust
/// Normalize any backend's output events into ForgeEvent variants.
/// Each backend may produce different raw event formats, but they all
/// get normalized here before emission to EventBus.
pub fn normalize_to_forge_event(
    backend: &str,
    raw: &StreamJsonEvent,
    session_id: &SessionId,
    agent_id: &AgentId,
) -> Option<ForgeEvent> {
    // For now, Claude events map 1:1 to ForgeEvent
    // Future backends will have their own mapping logic
    match raw {
        StreamJsonEvent::Assistant(payload) => Some(ForgeEvent::AssistantMessage { ... }),
        StreamJsonEvent::Result(payload) => Some(ForgeEvent::SessionCompleted { ... }),
        StreamJsonEvent::Error(payload) => Some(ForgeEvent::SessionFailed { ... }),
        // etc.
    }
}
```

### 2. `crates/forge-api/src/middleware.rs` — SpawnMiddleware

Update the event emission loop to use `normalize_to_forge_event()` instead of inline event construction. This centralizes the mapping.

### 3. `crates/forge-process/src/backend.rs`

Add documentation comments on the `ProcessBackend` trait explaining the event normalization contract:
```rust
/// All backends MUST produce ProcessHandle instances whose stdout emits
/// newline-delimited JSON parseable by `parse_line()`. Events are then
/// normalized to ForgeEvent via `normalize_to_forge_event()`.
```

### 4. Integration tests

Add to `crates/forge-api/` tests:
- Test that `BackendRegistry` correctly dispatches to `ClaudeBackend`
- Test that `SpawnMiddleware` with registry produces correct RunResponse
- Test health endpoint returns expected format
- Test backends endpoint lists capabilities

Add to `crates/forge-process/` tests:
- Test `normalize_to_forge_event` maps all StreamJsonEvent variants
- Test ClaudeBackend capabilities match expected values

### 5. Update docs

**`site-docs/architecture/events.md`:**
- Add section on event normalization for multi-backend support

**`site-docs/reference/api.md`:**
- Add `/api/v1/backends` and `/api/v1/backends/health` endpoints

**`site-docs/reference/mcp-tools.md`:**
- Add `forge_list_backends` and `forge_backend_health` tools (total now 21)

### 6. Update CLAUDE.md

- MCP tools count: 19 → 21
- Add "Hexagonal Backends" to Wave 4 Architecture section
- Mention `BackendRegistry` and `ProcessBackend` trait in Conventions
- Update crate description for `forge-process` to mention multi-backend support

## Depends On
- **All of E3-A, E3-B, E3-C must be complete**

## Verification
```bash
cargo check                    # zero warnings
cargo test                     # all green
cargo clippy -- -D warnings    # clean
cargo build --release          # release build succeeds
```

Manual:
```bash
./target/release/forge &
curl http://127.0.0.1:4173/api/v1/backends
curl http://127.0.0.1:4173/api/v1/backends/health
curl http://127.0.0.1:4173/api/v1/agents  # verify backend_type field appears
kill %1
```

## Zero Warnings Policy
All modified files must produce zero warnings.
