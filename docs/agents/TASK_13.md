# TASK 13 — GitHub Release workflow

**Status:** pending
**Priority:** high
**Track:** Phase A — ship

---

## Context

CI exists (`.github/workflows/ci.yml`). We need a release workflow that builds binaries when a tag is pushed.

## Task

Create `.github/workflows/release.yml`:

```yaml
name: Release
on:
  push:
    tags: ['v*']

permissions:
  contents: write

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-latest
            name: forge-macos-arm64
          - target: x86_64-apple-darwin
            os: macos-latest
            name: forge-macos-x64
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: forge-linux-x64

    runs-on: ${{ matrix.os }}
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
      - run: cd frontend && pnpm install --frozen-lockfile && pnpm build

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Rename binary
        run: cp target/${{ matrix.target }}/release/forge ${{ matrix.name }}

      - uses: softprops/action-gh-release@v2
        with:
          files: ${{ matrix.name }}
```

## Files to create

- `.github/workflows/release.yml`

## Verify

After committing, push a tag like `v0.1.1-test` to trigger the workflow. Check GitHub Actions tab and Releases page. Delete the test tag/release after verifying.

---

## Report

*Agent: fill this in when done.*

- [ ] What was changed:
- [ ] Workflow runs: yes/no
- [ ] Notes:
