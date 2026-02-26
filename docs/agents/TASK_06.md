# TASK 06 — Graceful shutdown

**Status:** done
**Priority:** high
**Track:** Phase A — ship

---

## Context

Currently `forge-app/src/main.rs` calls `serve(addr, state).await?` which blocks until the server stops. When the process is killed (Ctrl+C / SIGTERM), the BatchWriter thread may not flush its remaining events. We need graceful shutdown: catch the signal, shut down the server, then flush and stop the BatchWriter.

## Task

1. In `crates/forge-app/src/main.rs`, use `tokio::signal::ctrl_c()` for shutdown notification.
2. Change the `serve` call to use Axum's `with_graceful_shutdown`:
   ```rust
   let listener = tokio::net::TcpListener::bind(addr).await?;
   axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
   ```
3. Add a `shutdown_signal()` async function:
   ```rust
   async fn shutdown_signal() {
       tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
       tracing::info!("shutdown signal received");
   }
   ```
4. After `serve` returns, shut down the BatchWriter properly. Currently it's wrapped in `Arc` — you'll need to use `Arc::try_unwrap` to get ownership, then call `.shutdown()`. If try_unwrap fails (another reference exists), log a warning and drop it (the writer thread will still flush on channel close).
5. Update `crates/forge-api/src/lib.rs`: the `serve` function currently creates the listener internally. Refactor so the caller in `main.rs` can use `with_graceful_shutdown`. Either:
   - Change `serve()` to return the `Router` (let main.rs bind + serve), or
   - Accept a shutdown future parameter

## Files to read first

- `crates/forge-app/src/main.rs` — current startup
- `crates/forge-api/src/lib.rs` — current `serve()` function
- `crates/forge-db/src/batch_writer.rs` — `shutdown(self)` method

## Files to edit

- `crates/forge-app/src/main.rs`
- `crates/forge-api/src/lib.rs` (refactor `serve`)

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
# Manual: cargo run -p forge-app, then Ctrl+C — should see "shutdown signal received" log
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed:
  - **forge-api/lib.rs**: Added `serve_until_signal<F>(addr, state, shutdown: F)` that binds a `TcpListener`, then runs `axum::serve(listener, app(state)).with_graceful_shutdown(shutdown).await`. Kept `serve()` and `serve_with_listener()` unchanged for tests.
  - **forge-app/main.rs**: Added `shutdown_signal()` async fn that awaits `tokio::signal::ctrl_c()` and logs "shutdown signal received". Replaced `serve(addr, state)` with `serve_until_signal(addr, state, shutdown_signal())`. After serve returns, `Arc::try_unwrap(batch_writer)` — on success call `bw.shutdown()`; on failure (extra refs) log warning and drop the Arc so the writer thread flushes on channel close.
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [ ] Notes: Manual check: `cargo run -p forge-app`, then Ctrl+C — should see "shutdown signal received" in logs.
