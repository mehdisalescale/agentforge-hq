# Claude Forge -- Tech Stack

> Single Rust binary agentic coding platform.
> Target: ~33K LOC across 12 workspace crates, embedding a Svelte 5 SPA via `rust-embed`.

---

## 1. Runtime & Language

| Layer | Choice | Version | Why |
|-------|--------|---------|-----|
| Systems language | **Rust** | 1.82+ (2024 edition) | Memory safety without GC, fearless concurrency, single-binary distribution, strong ecosystem for CLI + server workloads. |
| Async runtime | **Tokio** | 1.41 | De-facto standard. Multi-threaded work-stealing scheduler, required by Axum. Battle-tested at scale (Cloudflare, Discord, AWS). |
| Frontend language | **TypeScript** | 5.6 | Type safety for the embedded SPA. Svelte 5 has first-class TS support. |
| Plugin language | **WASM** (via Wasmtime) | -- | Sandboxed, polyglot plugin execution. Authors can write plugins in Rust, Go, Python (compiled), or any language targeting WASI. |

---

## 2. Backend Crates

### 2.1 Web Framework

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **axum** | 0.8 | HTTP/WebSocket server, routing, middleware, extractors | Actix-web is faster in some benchmarks but has a less ergonomic API and weaker tower ecosystem integration. Axum composes naturally with tower middleware and is maintained by the Tokio team. |
| **tower** | 0.5 | Middleware stack (timeout, rate-limit, compression) | Standard service abstraction for Rust async. Axum is built on it. |
| **tower-http** | 0.6 | CORS, compression, tracing, request-id, static files | Production-grade HTTP middleware. Saves hundreds of lines of boilerplate. |
| **hyper** | 1.5 (transitive) | Low-level HTTP implementation | Pulled in by Axum. We do not use it directly. |

### 2.2 Serialization & Data

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **serde** | 1.0 | Serialize/deserialize structs | Universal standard. No real alternative in Rust. |
| **serde_json** | 1.0 | JSON parsing/generation | Fastest safe JSON in Rust. Used for API payloads, MCP messages, event storage. |
| **toml** | 0.8 | Config file parsing | TOML is the Rust ecosystem standard for configuration (Cargo.toml precedent). |
| **uuid** | 1.11 | UUIDv7 generation for all entity IDs | UUIDv7 is time-sortable, which gives us chronological ordering for free in SQLite. Chosen over ULID (less ecosystem support) and UUIDv4 (not sortable). |
| **chrono** | 0.4 | Timestamp handling, duration math | More feature-complete than `time` crate for calendar arithmetic. Used for scheduling, cost tracking windows, session timestamps. |
| **base64** | 0.22 | Encoding for binary data in JSON payloads | Standard, zero-dependency. |
| **bytes** | 1.9 | Efficient byte buffer management | Required by Axum/Hyper. Also used in WebSocket message handling. |

### 2.3 Database

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **rusqlite** | 0.32 | SQLite interface with `bundled` + `bundled-full` features | Embeds SQLite directly into the binary -- no system dependency. Enables FTS5 for full-text search over skills, sessions, and logs. WAL mode for concurrent reads. Chosen over `sqlx` (needs compile-time DB connection), `diesel` (heavy ORM overhead, poor async story), and `sea-orm` (too much abstraction for our needs). |
| **r2d2** | 0.8 | Connection pooling for rusqlite | Lightweight pool. We use a pool of 4 read connections + 1 dedicated write connection (WAL pattern). |

**SQLite Configuration:**
- WAL mode (Write-Ahead Logging) for concurrent read/write
- FTS5 for full-text search (skills catalog, session search, log search)
- `journal_size_limit = 67108864` (64 MB WAL file cap)
- `cache_size = -8000` (8 MB page cache)
- `synchronous = NORMAL` (safe with WAL, 2x faster than FULL)
- `foreign_keys = ON`
- Batch writes: flush every 50 events or 2 seconds, whichever comes first
- DB location: `~/.claude-forge/forge.db`

### 2.4 Git Integration

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **git2** | 0.19 | libgit2 bindings (vendored) | Full git operations without shelling out: status, diff, log, branch, worktree management. Vendored build means no system libgit2 dependency. Chosen over `gitoxide` (gix) which is pure Rust but less mature for worktree and diff operations as of 2025. |

**Feature flags:** `vendored-libgit2`, `vendored-openssl` -- ensures the binary works on any Linux/macOS without system dependencies.

### 2.5 Plugin Runtime

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **wasmtime** | 27.0 | WASM/WASI runtime for plugin execution | Bytecode Alliance project, production-grade, supports WASI preview 2. Sandboxed execution with configurable resource limits (memory, fuel/instructions, file access). Chosen over `wasmer` (less active development, licensing concerns) and `wasm3` (interpreter only, slower). |
| **wasmtime-wasi** | 27.0 | WASI host implementation | Provides filesystem, env, clock capabilities to plugins with fine-grained permissions. |
| **wit-bindgen** | 0.36 | WIT interface generation | Generates Rust bindings for the plugin API. Plugins implement a defined WIT interface. |

**Plugin resource limits:**
- Memory: 64 MB per plugin instance (configurable)
- Fuel: 1,000,000 instructions per invocation (configurable)
- Filesystem: scoped to plugin data directory only
- Network: disabled by default, opt-in per plugin

### 2.6 CLI & Configuration

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **clap** | 4.5 | CLI argument parsing with derive macros | Industry standard. Derive macro means our CLI interface is defined by struct annotations -- less boilerplate, auto-generated help. |
| **directories** | 5.0 | XDG/platform-appropriate config paths | Cross-platform `~/.config/claude-forge`, `~/.local/share/claude-forge`, etc. |
| **dotenvy** | 0.15 | `.env` file loading | Fork of `dotenv` with active maintenance. |

### 2.7 Observability & Logging

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **tracing** | 0.1 | Structured, async-aware logging/tracing | The Rust ecosystem standard for async observability. Structured spans carry context across `.await` points. Chosen over `log` (no structured data, no spans) and `slog` (less ecosystem adoption). |
| **tracing-subscriber** | 0.3 | Log formatting, filtering, layer composition | `EnvFilter` for runtime log level control (`RUST_LOG`), JSON output for production, pretty output for development. |
| **tracing-appender** | 0.2 | Non-blocking file appender | Writes logs to `~/.claude-forge/logs/` without blocking the async runtime. Rolling daily files. |
| **metrics** | 0.24 | Lightweight metrics collection | Counters, gauges, histograms for internal dashboards. Lower overhead than full OpenTelemetry. |

### 2.8 Error Handling

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **thiserror** | 2.0 | Derive macro for library error types | Every crate defines its own error enum with `#[derive(thiserror::Error)]`. Gives us `Display`, `Error`, `From` impls for free. Used in all library/crate code. |
| **anyhow** | 1.0 | Ergonomic error handling in application code | Used only in `main.rs`, CLI entry points, and test code. Provides `context()` for error chain enrichment. Never used in library crates. |

### 2.9 Async & Concurrency

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **tokio** | 1.41 | Async runtime (multi-threaded) | Features: `full` (rt-multi-thread, io, time, sync, macros, process, signal, fs). |
| **dashmap** | 6.1 | Concurrent hash map | Lock-free reads, sharded writes. Used for in-memory agent state, active sessions, WebSocket connections. Chosen over `RwLock<HashMap>` (contention under concurrent reads) and `flurry` (less maintained). |
| **tokio::sync::broadcast** | (in tokio) | Event fan-out to WebSocket clients | Each WebSocket client subscribes to a broadcast channel. Lagging receivers get a `Lagged` error (we log + skip). |
| **tokio::sync::mpsc** | (in tokio) | Internal message passing | Bounded channels between process spawner and event processor. Backpressure via channel capacity. |
| **crossbeam-channel** | 0.5 | High-performance channels for batch writer | Used in the SQLite batch writer thread (non-async). Lower latency than tokio mpsc for this use case. |

### 2.10 Process Management

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **tokio::process** | (in tokio) | Async child process spawning | Spawns `claude` CLI processes with `--output-format stream-json --verbose`. Non-blocking stdout/stderr reading. |
| **nix** | 0.29 | Unix signal handling, process groups | `killpg` for clean process tree termination. Session/process group management for spawned agents. |
| **signal-hook** | 0.3 | Graceful shutdown signal handling | Catches SIGINT/SIGTERM for clean shutdown (flush DB, kill children, close WebSockets). |

### 2.11 Security

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **argon2** | 0.5 | Password hashing (if auth is added) | Memory-hard, resistant to GPU attacks. Future-proofing for multi-user scenarios. |
| **ring** | 0.17 | Cryptographic primitives | HMAC for webhook signatures, SHA-256 for content hashing. Audited, no-std compatible. |
| **secrecy** | 0.10 | Wrapper type that prevents secret logging | `Secret<String>` for API keys, tokens. `Debug`/`Display` impls print `[REDACTED]`. |

### 2.12 Networking & HTTP Client

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **reqwest** | 0.12 | HTTP client for MCP, webhooks, notifications | Feature flags: `json`, `rustls-tls` (no OpenSSL dependency). Used for outbound HTTP in MCP client, webhook delivery, notification channels. |
| **rustls** | 0.23 (transitive) | TLS implementation | Pure Rust TLS. No OpenSSL system dependency. Pulled in via reqwest + axum. |

### 2.13 Scheduling

| Crate | Version | Purpose | Why Not Alternatives |
|-------|---------|---------|---------------------|
| **cron** | 0.13 | Cron expression parsing | Parses standard cron expressions for scheduled tasks. Lightweight, no runtime -- just parsing + next-occurrence calculation. |
| **tokio-cron-scheduler** | 0.13 | Cron job execution on tokio runtime | Wraps cron parsing with actual job scheduling. Integrates with our tokio runtime. |

---

## 3. Frontend

### 3.1 Framework & UI

| Package | Version | Purpose | Why Not Alternatives |
|---------|---------|---------|---------------------|
| **SvelteKit** | 2.x | Application framework (adapter-static) | Generates a pure static SPA that `rust-embed` bundles into the binary. Chosen over Next.js (requires Node runtime), Remix (server-oriented), and plain Svelte (no routing). |
| **Svelte** | 5.x | UI framework with runes | Runes ($state, $derived, $effect) provide fine-grained reactivity without virtual DOM overhead. Smaller bundle than React/Vue. Compiler-based approach means less runtime JS. |
| **TailwindCSS** | 4.x | Utility-first CSS | v4 uses the Rust-based `oxide` engine -- 10x faster builds. No runtime CSS-in-JS overhead. Tree-shakes unused utilities. |
| **@tailwindcss/vite** | 4.x | Vite plugin for Tailwind | Integrates Tailwind compilation into the Vite build pipeline. |

### 3.2 Frontend Libraries

| Package | Version | Purpose |
|---------|---------|---------|
| **@sveltejs/adapter-static** | 3.x | Generates static files for rust-embed |
| **vite** | 6.x | Build tool and dev server |
| **shiki** | 1.x | Syntax highlighting (code viewer, diffs) |
| **xterm.js** | 5.x | Terminal emulator for embedded terminal (Phase 6) |
| **@xterm/addon-fit** | 0.10 | Auto-resize xterm to container |
| **chart.js** | 4.x | Cost tracking charts, token usage graphs |
| **d3-hierarchy** | 3.x | Swim lane layout for agent orchestration visualization |
| **fuse.js** | 7.x | Client-side fuzzy search for skills, commands |
| **date-fns** | 4.x | Date formatting and manipulation |

### 3.3 Frontend Patterns

- **Runes only:** No legacy `$:` reactive statements. All state uses `$state()`, computed values use `$derived()`, side effects use `$effect()`.
- **No stores:** Svelte 5 runes replace writable/readable stores. Shared state lives in `.svelte.ts` modules using rune exports.
- **Event handling:** `onclick={(e) => { ... }}` -- no `on:click|modifier` syntax (Svelte 5 change).
- **Component re-init:** `{#key value}` blocks to force component destruction/recreation when key values change.

---

## 4. Build System

### 4.1 Build Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **cargo** | 1.82+ | Rust build system, workspace management |
| **pnpm** | 9.x | Frontend package manager (strict, fast, deduped) |
| **rust-embed** | 8.5 | Embeds `frontend/build/` into the Rust binary at compile time |
| **mise** | 2024.x | Polyglot version manager (Rust, Node, pnpm) |

### 4.2 Build Pipeline

```
pnpm build (frontend/)          cargo build --release
       |                                |
       v                                |
  frontend/build/  ----rust-embed----->  |
       (static SPA)                     v
                                   target/release/claude-forge
                                       (single binary, ~35MB)
```

**Build steps:**
1. `cd frontend && pnpm install && pnpm build` -- produces static SPA in `frontend/build/`
2. `cargo build --release` -- compiles all 12 crates, embeds frontend via `rust-embed`
3. `strip target/release/claude-forge` -- removes debug symbols (~40% size reduction)
4. Optional: `upx --best target/release/claude-forge` -- further compression if needed

### 4.3 Development Tools

| Tool | Purpose |
|------|---------|
| **cargo-watch** | Auto-rebuild on file changes: `cargo watch -x 'run -- --dev'` |
| **cargo-nextest** | Faster test runner with better output |
| **cargo-llvm-cov** | Code coverage via LLVM instrumentation |
| **cargo-deny** | License and vulnerability auditing for dependencies |
| **cargo-udeps** | Detect unused dependencies |
| **cargo-bloat** | Analyze binary size by crate/function |
| **vite dev** | Frontend HMR dev server (proxies API to Rust backend) |
| **prettier** | Frontend code formatting |
| **eslint** | Frontend linting |

---

## 5. Binary Size Budget

**Target: 30-50 MB** (release, stripped)

| Component | Estimated Size | Notes |
|-----------|---------------|-------|
| Rust core (Axum, Tokio, SQLite) | ~8 MB | Tokio multi-thread + axum + rusqlite bundled |
| libgit2 (vendored) | ~4 MB | Full git implementation |
| Wasmtime | ~12 MB | Largest single dependency. WASM compiler + runtime. |
| Frontend SPA (gzip in binary) | ~2 MB | Svelte compiles to minimal JS. Tailwind purges unused CSS. |
| Other crates | ~4 MB | serde, reqwest+rustls, tracing, etc. |
| **Total (stripped)** | **~30 MB** | Before UPX compression |
| **Total (UPX)** | **~12-15 MB** | With `upx --best` |

**Size optimization strategies:**
- `codegen-units = 1` in release profile (better LTO, slower compile)
- `lto = "thin"` for release builds (good size reduction, reasonable compile time)
- `opt-level = "z"` for size-critical crates (if needed)
- `strip = true` in `Cargo.toml` release profile
- Wasmtime feature flags: only enable `cranelift` backend, disable `winch`
- `rust-embed` compresses assets with gzip

---

## 6. Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold start to HTTP ready | < 500 ms | Time from binary execution to first request served |
| API response (simple GET) | < 5 ms p99 | Agent list, session list, config read |
| API response (DB query) | < 20 ms p99 | Full-text search, session history, skill lookup |
| WebSocket event latency | < 10 ms | Time from child process stdout to WebSocket delivery |
| SQLite batch write | < 50 ms | Flushing 50 events in one transaction |
| Frontend initial load | < 1.5 s | First meaningful paint (embedded static assets, no CDN roundtrip) |
| Frontend bundle size | < 300 KB gzipped | JS + CSS total transfer |
| Memory (idle, 0 agents) | < 30 MB RSS | Base memory with no active processes |
| Memory (10 active agents) | < 200 MB RSS | 10 concurrent claude processes + WebSocket connections |
| Concurrent WebSocket clients | 100+ | Broadcast channel handles fan-out |
| Agent spawn time | < 200 ms | Time from API call to child process running |

---

## 7. Platform Support

| Platform | Tier | Notes |
|----------|------|-------|
| macOS (Apple Silicon) | Tier 1 | Primary development platform. Fully tested. |
| macOS (x86_64) | Tier 1 | CI tested. |
| Linux (x86_64, glibc) | Tier 1 | CI tested. Server deployment target. |
| Linux (aarch64, glibc) | Tier 2 | Cross-compiled, smoke tested. |
| Linux (musl) | Tier 3 | Static binary. May have issues with git2/openssl. |
| Windows | Not supported | Claude CLI is Unix-oriented. May revisit via WSL2. |

---

## 8. Dependency Policy

1. **Minimize total dependencies.** Every new crate must justify its inclusion. Prefer stdlib when reasonable.
2. **Pin major versions.** Use `crate = "1"` not `crate = "*"`. Lock file (`Cargo.lock`) is committed.
3. **Audit regularly.** `cargo deny check` runs in CI. No `unsafe` in dependencies without review.
4. **Vendor critical deps.** SQLite (bundled), libgit2 (vendored), OpenSSL (rustls instead) -- no system library dependencies.
5. **Feature-flag large deps.** Wasmtime is behind a `plugins` feature flag. Builds without plugins skip it (~12 MB savings).
6. **License allowlist.** MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib. MPL-2.0 case-by-case. No GPL/AGPL/SSPL.
7. **No `unsafe` in our code.** All 12 workspace crates use `#![forbid(unsafe_code)]`. Unsafe lives only in audited dependencies.
