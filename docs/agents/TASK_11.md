# TASK 11 — Configurable host and port

**Status:** pending
**Priority:** medium
**Track:** Phase A polish

---

## Context

The server is hardcoded to `127.0.0.1:4173` in `crates/forge-app/src/main.rs`. Users should be able to set host and port via environment variables.

## Task

1. Read `FORGE_HOST` (default `127.0.0.1`) and `FORGE_PORT` (default `4173`) from environment.
2. Parse into `SocketAddr`.
3. Log the actual address being used.

```rust
let host = env::var("FORGE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
let port = env::var("FORGE_PORT").unwrap_or_else(|_| "4173".into());
let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
```

## Files to edit

- `crates/forge-app/src/main.rs`

## Verify

```bash
cargo test --workspace
FORGE_PORT=8080 cargo run -p forge-app  # should start on 8080
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: main.rs now reads FORGE_HOST (default 127.0.0.1) and FORGE_PORT (default 4173), parses SocketAddr, logs the address; module doc updated.
- [x] Tests pass: yes
- [ ] Notes:
