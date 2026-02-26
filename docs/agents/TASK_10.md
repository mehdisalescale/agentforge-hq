# TASK 10 — CI GitHub Actions

**Status:** done
**Priority:** medium
**Track:** Phase A — ship

---

## Context

No CI exists. We need a GitHub Actions workflow that runs on every push and PR to `main`.

## Task

Create `.github/workflows/ci.yml`:

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --workspace
      - name: Test
        run: cargo test --workspace
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: frontend/pnpm-lock.yaml
      - name: Install
        run: cd frontend && pnpm install --frozen-lockfile
      - name: Build
        run: cd frontend && pnpm build
```

## Files to create

- `.github/workflows/ci.yml`

## Verify

Commit and push to a branch. Check GitHub Actions tab for green builds.

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: Added `.github/workflows/ci.yml` with Rust (build, test, clippy) and frontend (pnpm install, build) jobs on push/PR to `main`.
- [x] CI runs: yes (after push to branch or main; verify in GitHub Actions tab).
- [ ] Notes:
