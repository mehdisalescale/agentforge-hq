# Agent UW-D: Unit of Work — Tests + Verification

## Goal

Ensure the entire workspace compiles, all tests pass, and the UnitOfWork refactor is complete. Update CLAUDE.md if needed.

## Depends On
- **All of UW-A, UW-B, UW-C must be complete**

## Tasks

### 1. Full build verification
```bash
cargo check                    # zero warnings
cargo test                     # all green
cargo clippy -- -D warnings    # zero clippy issues
```

### 2. Fix any compile errors from the refactor
- Route handlers referencing old `state.xxx_repo` paths
- Middleware constructor mismatches (Arc vs reference)
- MCP tool handlers referencing old `self.xxx_repo` paths
- Missing imports for `UnitOfWork`

### 3. Update existing tests
- `crates/forge-api/src/lib.rs` has `test_state()` helper — update to use `UnitOfWork`
- Any integration tests that construct `AppState` directly
- MCP tests if they construct `ForgeMcp` directly

### 4. Add UnitOfWork-specific tests
In `crates/forge-db/src/unit_of_work.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uow_provides_all_repos() {
        let pool = DbPool::open(":memory:").unwrap();
        // Run migrations
        let uow = UnitOfWork::new(Arc::new(pool));
        // Verify each repo is accessible and functional
        assert!(uow.agent_repo.list().unwrap().is_empty());
        assert!(uow.session_repo.list().unwrap().is_empty());
        // ... etc for all 17
    }
}
```

### 5. Update CLAUDE.md
- Update the "Workspace Crates" section if `unit_of_work` adds notable structure
- Mention `UnitOfWork` pattern in Conventions if appropriate
- Verify crate count is still accurate

### 6. Final smoke test
```bash
cargo build --release
./target/release/forge &
# Test a few endpoints
curl http://127.0.0.1:4173/api/v1/agents
curl http://127.0.0.1:4173/api/v1/companies
kill %1
```

## Verification
- `cargo check` — zero warnings
- `cargo test` — all green
- `cargo clippy -- -D warnings` — clean
- Manual endpoint test passes
