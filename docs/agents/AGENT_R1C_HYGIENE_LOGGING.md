# Agent R1-C: Dependency Hygiene + Structured Logging

> Add cargo-deny for license/vulnerability auditing. Add JSON structured logging for release builds. Update CI.

## Step 1: Read Context

- `CLAUDE.md` — conventions
- `Cargo.toml` (workspace root) — current dependencies
- `crates/forge-app/src/main.rs` — find tracing subscriber init (~lines 39-41)
- `crates/forge-app/Cargo.toml` — current forge-app dependencies
- `.github/workflows/ci.yml` — current CI pipeline

## Step 2: Add Structured JSON Logging

In `crates/forge-app/Cargo.toml`, ensure `tracing-subscriber` has the `json` feature:

Check the workspace `Cargo.toml` for how `tracing-subscriber` is defined. Add `json` to its features if not present. Example:
```toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

In `crates/forge-app/src/main.rs`, replace the tracing init section:

**Current (~lines 39-41):**
```rust
let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
tracing_subscriber::fmt().with_env_filter(filter).init();
```

**Replace with:**
```rust
let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

if cfg!(not(debug_assertions)) {
    // Production: JSON structured logging for machine parsing
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
} else {
    // Development: human-readable format
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
```

You'll need to add `use tracing_subscriber::fmt::format::FmtSpan;` or use the full path as shown.

## Step 3: Create cargo-deny Configuration

Create `deny.toml` at the workspace root:

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Zlib",
    "MPL-2.0",
    "OpenSSL",
    "Unicode-3.0",
    "BSL-1.0",
]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "allow"

[sources]
unknown-registry = "warn"
unknown-git = "warn"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

## Step 4: Update CI Pipeline

In `.github/workflows/ci.yml`, add cargo-deny and cargo-audit steps to the Rust job.

Read the current ci.yml first to understand its structure. Then add steps AFTER the existing clippy step:

```yaml
      - name: Install cargo-deny
        run: cargo install cargo-deny --locked

      - name: Check licenses and advisories
        run: cargo deny check advisories licenses

      - name: Install cargo-audit
        run: cargo install cargo-audit --locked

      - name: Audit dependencies
        run: cargo audit
```

**Important:** These should be `continue-on-error: true` initially so they don't block the CI if there are existing issues. We can make them strict later.

```yaml
      - name: Check licenses and advisories
        run: cargo deny check advisories licenses
        continue-on-error: true

      - name: Audit dependencies
        run: cargo audit
        continue-on-error: true
```

## Step 5: Verify

```bash
cargo check 2>&1 | head -20    # zero warnings
cargo test 2>&1 | tail -5       # all pass

# Test JSON logging works in release mode
cargo build --release 2>&1 | tail -5
```

If cargo-deny is installed locally, also run:
```bash
cargo deny check advisories licenses
```

## Rules

- Touch ONLY: `crates/forge-app/src/main.rs` (tracing init section ONLY — do NOT touch default_db_path or event bus wiring), `Cargo.toml` (workspace), `crates/forge-app/Cargo.toml`, `.github/workflows/ci.yml`, `deny.toml` (new)
- Do NOT touch `crates/forge-app/src/main.rs` default_db_path function (Agent R1-B handles that)
- Do NOT touch files under `site-docs/` (Agent R1-A handles those)
- Do NOT touch `CLAUDE.md` or `README.md` (Agent R1-B handles those)
- Run `cargo check` and `cargo test` before reporting done

## Report

When done, create `docs/agents/REPORT_R1C.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
FILES_MODIFIED: [list]
FILES_CREATED: [list]
TRACING_JSON: enabled for release builds (yes/no)
DENY_TOML: created (yes/no)
CI_UPDATED: cargo-deny + cargo-audit added (yes/no)
CARGO_CHECK: pass/fail
CARGO_TEST: pass/fail
```
