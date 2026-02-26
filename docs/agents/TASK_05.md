# TASK 05 — rust-embed: serve frontend from single binary

**Status:** done
**Priority:** critical (v0.1.0 blocker)
**Track:** Phase A — ship

---

## Context

The frontend builds to `frontend/build/` via `pnpm build` (SvelteKit adapter-static). Currently `forge-app` is API-only — the frontend must be served separately. For a single-binary distribution, we need to embed the built frontend assets into the Rust binary using `rust-embed` and serve them from the Axum router.

## Task

1. Add `rust-embed = "8"` to `crates/forge-api/Cargo.toml` dependencies.
2. In `crates/forge-api/src/lib.rs`, create an embed struct:
   ```rust
   #[derive(rust_embed::Embed)]
   #[folder = "../frontend/build"]
   struct FrontendAssets;
   ```
3. Add a fallback handler that serves embedded files. For any GET request not matched by `/api/v1/*` or `/api/v1/ws`:
   - Try to serve the file at the request path (e.g. `/skills` → look for `skills/index.html` or `skills.html`)
   - Serve with correct Content-Type (use `mime_guess` — rust-embed re-exports it)
   - If file not found, serve `index.html` (SPA fallback for client-side routing)
4. Mount the fallback AFTER the API routes so API routes take priority.
5. Verify: `cargo run -p forge-app` serves both `http://127.0.0.1:4173/api/v1/health` (API) and `http://127.0.0.1:4173/` (frontend).

**Important:** The `#[folder]` path is relative to the crate's `Cargo.toml`, so `../frontend/build` is correct for `crates/forge-api/`.

## Files to read first

- `crates/forge-api/src/lib.rs` — current router setup
- `crates/forge-api/Cargo.toml` — current dependencies
- `frontend/build/` — check it exists (run `cd frontend && pnpm build` if not)

## Files to edit

- `crates/forge-api/Cargo.toml` (add rust-embed)
- `crates/forge-api/src/lib.rs` (embed struct + fallback handler)

## Verify

```bash
cd frontend && pnpm build && cd ..
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
# Then manually:
cargo run -p forge-app &
curl -s http://127.0.0.1:4173/api/v1/health  # should return JSON
curl -s http://127.0.0.1:4173/ | head -5       # should return HTML
kill %1
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: **forge-api:** Added `rust-embed` (workspace) and `mime_guess = "2"` to `Cargo.toml`. In `lib.rs`: `FrontendAssets` embed struct with `#[folder = "../frontend/build"]` and `#[allow_missing = true]` (so crate builds when frontend not built, e.g. CI). Fallback handler `serve_embedded_fallback`: GET only (405 otherwise); tries path, path/index.html, path.html, then serves index.html (SPA fallback); sets Content-Type via `mime_guess::from_path`; 404 if index.html missing. Router: `.nest("/api/v1", ...)` then `.fallback(serve_embedded_fallback)` so API takes priority.
- [x] Tests pass: yes
- [x] Clippy clean: yes
- [x] Manual verification: yes — `cargo run -p forge-app` then `curl .../api/v1/health` → 200 JSON, `curl .../` → 200 HTML (index.html).
- [ ] Notes: Run `cd frontend && pnpm build` before release so the binary embeds the frontend. With `allow_missing`, debug build reads from `../frontend/build` at runtime when present.
