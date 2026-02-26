# TASK 04 — CORS hardening

**Status:** done
**Priority:** medium
**Track:** security

---

## Context

In `crates/forge-api/src/lib.rs`, CORS is set to allow any origin:

```rust
let cors = CorsLayer::new()
    .allow_origin(tower_http::cors::Any)
    .allow_methods(tower_http::cors::Any)
    .allow_headers(tower_http::cors::Any);
```

This is fine for local dev but should be configurable for production.

## Task

1. Read `FORGE_CORS_ORIGIN` from environment (default: `*` for dev).
2. If `*`, keep `Any`. Otherwise parse as a specific origin.
3. Always allow methods: GET, POST, PUT, DELETE, OPTIONS.
4. Always allow headers: Content-Type, Authorization.

```rust
use std::env;
use tower_http::cors::{CorsLayer, AllowOrigin};
use http::HeaderValue;

let cors_origin = env::var("FORGE_CORS_ORIGIN").unwrap_or_else(|_| "*".into());
let cors = if cors_origin == "*" {
    CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
} else {
    let origin: HeaderValue = cors_origin.parse().expect("invalid CORS origin");
    CorsLayer::new()
        .allow_origin(AllowOrigin::exact(origin))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
};
```

## Files to read first

- `crates/forge-api/src/lib.rs` — current CORS setup

## Files to edit

- `crates/forge-api/src/lib.rs`

## Verify

```bash
cargo test --workspace
cargo clippy --workspace
# Default (no env var) should still work for local dev
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: In `crates/forge-api/src/lib.rs`: read `FORGE_CORS_ORIGIN` from env (default `*`). If `*`, use `Any` for origin; else parse as `HeaderValue` and use `AllowOrigin::exact(origin)`. Always allow methods GET, POST, PUT, DELETE, OPTIONS and headers Content-Type, Authorization. Added short doc on `app()` for the env var.
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [x] Notes: Default (no env) keeps permissive CORS for local dev. Production: set `FORGE_CORS_ORIGIN=https://your-frontend.example.com` (or the exact origin). Invalid value panics at startup via `.expect("FORGE_CORS_ORIGIN must be a valid HTTP header value")`.
