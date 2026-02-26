# TASK 09 — TraceLayer request logging middleware

**Status:** done
**Priority:** low
**Track:** observability

---

## Context

The Axum router in `crates/forge-api/src/lib.rs` has no request logging. Every HTTP request should log method, path, status code, and latency.

## Task

1. Add `tower-http` `trace` feature to `crates/forge-api/Cargo.toml` (it may already be there for CORS — check and add `"trace"` to features).
2. In `crates/forge-api/src/lib.rs`, add TraceLayer to the router:
   ```rust
   use tower_http::trace::TraceLayer;

   let app = Router::new()
       // ... existing routes ...
       .layer(TraceLayer::new_for_http())
       .layer(cors);
   ```
3. TraceLayer should be added BEFORE the cors layer (layers wrap in reverse order, so TraceLayer added after cors in code means it runs before cors in the request lifecycle — actually just put it right after routes, before cors).

## Files to edit

- `crates/forge-api/Cargo.toml` (add "trace" feature if missing)
- `crates/forge-api/src/lib.rs` (add TraceLayer)

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
# Manual: cargo run -p forge-app, then curl http://127.0.0.1:4173/api/v1/health
# Should see a log line with method=GET path=/api/v1/health status=200 latency=Xms
```

---

## Report

*Agent: filled when done.*

- [x] What was changed:
  - **Cargo.toml** (workspace): added `"trace"` to `tower-http` features (`["cors", "trace"]`).
  - **crates/forge-api/src/lib.rs**: `use tower_http::trace::TraceLayer`, and `.layer(TraceLayer::new_for_http())` after `.nest("/api/v1", routes::router())`, before `.layer(cors)`.
- [ ] Tests pass: `cargo test -p forge-api` fails in this environment (missing `frontend/build` for RustEmbed). TraceLayer change does not affect test logic.
- [x] Clippy clean: yes (`cargo clippy --workspace` passed).
- [ ] Notes: Manual check: run `cargo run -p forge-app`, then `curl http://127.0.0.1:4173/api/v1/health` — log line should show method, path, status, latency.
