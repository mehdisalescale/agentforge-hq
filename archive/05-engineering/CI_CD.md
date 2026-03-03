# Claude Forge -- CI/CD and Release Engineering

> Complete build, test, release, and distribution pipeline for the `claude-forge` single binary.
> Target: pre-built binaries on GitHub Releases for every supported platform, plus `cargo install`, Homebrew, and a one-liner install script.

---

## Table of Contents

1. [Build Pipeline (CI)](#1-build-pipeline-ci)
2. [Release Pipeline](#2-release-pipeline)
3. [GitHub Release](#3-github-release)
4. [Install Script](#4-install-script)
5. [cargo install Support](#5-cargo-install-support)
6. [Homebrew / Package Managers](#6-homebrew--package-managers)
7. [Docker Image](#7-docker-image)
8. [Version Management](#8-version-management)
9. [Quality Gates Before Release](#9-quality-gates-before-release)
10. [Complete GitHub Actions Workflows](#10-complete-github-actions-workflows)

---

## 1. Build Pipeline (CI)

Every pull request and every push to `main` triggers the CI pipeline. The goal is fast feedback: catch lint errors, test failures, and compilation problems before merge.

### 1.1 Pipeline Overview

```
PR / push to main
  |
  +-- [lint]       clippy, rustfmt, svelte-check, eslint
  |
  +-- [test]       cargo test (all crates), vitest (frontend)
  |
  +-- [build]      compile-check for primary targets (linux-x64, macOS-arm64)
  |
  +-- [audit]      cargo audit, cargo deny, npm audit
```

All four jobs run in parallel. The PR is blocked until all pass.

### 1.2 Lint Jobs

| Tool | Scope | Command | Failure Policy |
|------|-------|---------|----------------|
| `rustfmt` | All `.rs` files | `cargo fmt --all -- --check` | Hard fail |
| `clippy` | All crates | `cargo clippy --workspace --all-targets -- -D warnings` | Hard fail |
| `svelte-check` | Frontend SPA | `cd frontend && pnpm svelte-check` | Hard fail |
| `eslint` | Frontend TS/Svelte | `cd frontend && pnpm lint` | Hard fail |
| `prettier` | Frontend formatting | `cd frontend && pnpm format --check` | Hard fail |

### 1.3 Test Jobs

**Backend tests:**

```bash
# Unit + integration tests, all workspace crates
cargo test --workspace --all-targets

# Doc tests
cargo test --workspace --doc
```

**Frontend tests:**

```bash
cd frontend
pnpm test          # vitest unit tests
pnpm test:e2e      # playwright (optional, on main only)
```

### 1.4 Build Verification

CI does not produce release artifacts, but it verifies that the project compiles on the two most common development targets:

```bash
# Build frontend first (required by rust-embed)
cd frontend && pnpm install --frozen-lockfile && pnpm build

# Check compilation (faster than full build)
cargo check --workspace --all-targets
```

### 1.5 Dependency Auditing

```bash
# Rust supply-chain audit
cargo audit
cargo deny check

# Frontend audit
cd frontend && pnpm audit --audit-level=moderate
```

### 1.6 Caching Strategy

| Cache Key | What | TTL |
|-----------|------|-----|
| `rust-${{ hashFiles('Cargo.lock') }}` | `~/.cargo/registry`, `~/.cargo/git`, `target/` | Until Cargo.lock changes |
| `pnpm-${{ hashFiles('frontend/pnpm-lock.yaml') }}` | `~/.pnpm-store` | Until lockfile changes |
| `sccache-${{ runner.os }}` | Compilation cache via sccache | 7 days rolling |

We use **sccache** as a shared compilation cache to speed up incremental builds across CI runs. Particularly important because `rusqlite` (bundled), `wasmtime`, and `git2` (vendored) are expensive to compile from scratch.

---

## 2. Release Pipeline

### 2.1 Trigger

The release pipeline fires on:

1. **Tag push** matching `v*` (e.g., `v1.0.0`, `v1.2.3-beta.1`)
2. **Manual workflow dispatch** with a version input (for re-runs or hotfixes)

```yaml
on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      tag:
        description: 'Release tag (e.g., v1.0.0)'
        required: true
```

### 2.2 Target Matrix

All seven targets are built in a matrix strategy:

| Target Triple | OS | Arch | Runner | Cross? | Notes |
|---------------|----|------|--------|--------|-------|
| `x86_64-apple-darwin` | macOS | Intel | `macos-13` | No | Native build on Intel runner |
| `aarch64-apple-darwin` | macOS | Apple Silicon | `macos-14` | No | Native build on M1+ runner |
| `x86_64-unknown-linux-gnu` | Linux | x64 | `ubuntu-22.04` | No | Dynamic linking to glibc 2.35 |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 | `ubuntu-22.04` | Yes | Uses `cross-rs` |
| `x86_64-unknown-linux-musl` | Linux | x64 (static) | `ubuntu-22.04` | Yes | Fully static binary, best for containers |
| `x86_64-pc-windows-msvc` | Windows | x64 | `windows-2022` | No | Native MSVC build |
| `aarch64-pc-windows-msvc` | Windows | ARM64 | `windows-2022` | Yes | Cross-compiled with MSVC ARM64 target |

### 2.3 Cross-Compilation Setup

For targets that cannot build natively, we use **cross-rs**:

```bash
cargo install cross --git https://github.com/cross-rs/cross

# Example: Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu
```

**cross-rs requirements:**
- Docker must be available on the runner
- Custom `Cross.toml` for vendored C dependencies:

```toml
# Cross.toml
[build.env]
passthrough = ["RUSTFLAGS"]

[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"

[target.x86_64-unknown-linux-musl]
image = "ghcr.io/cross-rs/x86_64-unknown-linux-musl:main"
```

For Windows ARM64, we install the MSVC ARM64 toolchain directly on the Windows runner instead of cross-rs.

### 2.4 Build Sequence Per Target

Every matrix job follows the same steps:

```
1. Checkout code
2. Install Rust toolchain + target
3. Install Node.js + pnpm
4. Build frontend:  cd frontend && pnpm install --frozen-lockfile && pnpm build
5. Build backend:   cargo build --release --target $TARGET
6. Strip binary:    strip target/$TARGET/release/forge (or llvm-strip on cross)
7. Package:         tar.gz (Unix) or zip (Windows)
8. Generate SHA256: shasum -a 256 forge-*.tar.gz > forge-*.tar.gz.sha256
9. Upload artifact
```

### 2.5 Binary Stripping and Compression

**Stripping** (removes debug symbols, reduces size by ~40-60%):

```bash
# macOS / Linux native
strip target/$TARGET/release/forge

# Cross-compiled (use target-specific strip)
# cross handles this automatically, or:
llvm-strip target/$TARGET/release/forge

# Windows (strip is not standard, but Rust --release already excludes debug info)
# Rely on Cargo profile settings instead
```

**Cargo profile for release:**

```toml
# Cargo.toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = "fat"           # Full link-time optimization
codegen-units = 1     # Single codegen unit for maximum optimization
strip = "symbols"     # Strip symbols automatically
panic = "abort"       # Smaller binary, no unwinding tables
```

**UPX compression** (optional, may interfere with macOS notarization):

```bash
# Only for Linux builds where startup time is less critical
upx --best --lzma target/$TARGET/release/forge
```

We recommend **not** using UPX for macOS (breaks notarization) or Windows (triggers antivirus false positives). The `opt-level = "z"` + `lto = "fat"` + `strip` combination typically gets us under 30MB.

### 2.6 Binary Signing

#### macOS Notarization

macOS requires notarization for unsigned binaries to run without Gatekeeper warnings.

```bash
# 1. Sign with Developer ID
codesign --force --options runtime \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  target/aarch64-apple-darwin/release/forge

# 2. Create zip for notarization
ditto -c -k --keepParent forge forge-notarize.zip

# 3. Submit for notarization
xcrun notarytool submit forge-notarize.zip \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_APP_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" \
  --wait

# 4. Staple the ticket (only for .dmg/.pkg, not bare binaries)
# For bare binaries, Gatekeeper checks online
```

**Required GitHub Secrets:**

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE_P12` | Base64-encoded .p12 signing certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_APP_PASSWORD` | App-specific password |
| `APPLE_TEAM_ID` | Apple Developer Team ID |

#### Windows Code Signing

```powershell
# Sign with Authenticode (requires EV code signing certificate)
signtool sign /f certificate.pfx /p $env:CERT_PASSWORD /tr http://timestamp.digicert.com /td sha256 /fd sha256 forge.exe
```

**Required GitHub Secrets:**

| Secret | Description |
|--------|-------------|
| `WINDOWS_CERT_PFX` | Base64-encoded .pfx signing certificate |
| `WINDOWS_CERT_PASSWORD` | Password for the .pfx |

> **Note:** Code signing certificates cost money and require identity verification. For an open-source project, this can be deferred -- unsigned binaries work fine but show OS warnings. macOS notarization requires a $99/year Apple Developer account.

### 2.7 Checksum Generation

Every artifact gets a SHA256 checksum file:

```bash
# Generate checksums for all artifacts
cd dist/
sha256sum forge-* > SHA256SUMS.txt

# Per-file checksums too (for the install script)
sha256sum forge-v1.0.0-x86_64-apple-darwin.tar.gz > forge-v1.0.0-x86_64-apple-darwin.tar.gz.sha256
```

---

## 3. GitHub Release

### 3.1 Release Creation

The release job runs after all matrix builds complete. It:

1. Downloads all build artifacts from the matrix jobs
2. Creates a GitHub Release (draft for pre-releases, published for stable)
3. Uploads all binaries + checksums
4. Generates changelog from conventional commits

### 3.2 Artifact Naming Convention

```
forge-v{VERSION}-{TARGET}.{EXT}
forge-v{VERSION}-{TARGET}.{EXT}.sha256
```

Examples:

```
forge-v1.0.0-x86_64-apple-darwin.tar.gz
forge-v1.0.0-x86_64-apple-darwin.tar.gz.sha256
forge-v1.0.0-aarch64-apple-darwin.tar.gz
forge-v1.0.0-aarch64-unknown-linux-gnu.tar.gz
forge-v1.0.0-x86_64-unknown-linux-musl.tar.gz
forge-v1.0.0-x86_64-pc-windows-msvc.zip
forge-v1.0.0-aarch64-pc-windows-msvc.zip
SHA256SUMS.txt
```

### 3.3 Changelog Generation

We use **git-cliff** to generate changelogs from conventional commits:

```bash
# Install
cargo install git-cliff

# Generate changelog for a specific tag
git cliff --latest --strip header > RELEASE_NOTES.md

# Generate full changelog
git cliff -o CHANGELOG.md
```

**git-cliff configuration** (`cliff.toml`):

```toml
[changelog]
header = "# Changelog\n\nAll notable changes to Claude Forge.\n"
body = """
{% if version %}\
    ## [{{ version }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}\
    ## [Unreleased]
{% endif %}\
{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | striptags | trim | upper_first }}
    {% for commit in commits %}
        - {% if commit.scope %}**{{ commit.scope }}:** {% endif %}\
            {{ commit.message | upper_first }} \
            ([{{ commit.id | truncate(length=7, end="") }}](https://github.com/anthropics/claude-forge/commit/{{ commit.id }}))\
    {% endfor %}
{% endfor %}\n
"""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
split_commits = false

commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactoring" },
    { message = "^doc", group = "Documentation" },
    { message = "^style", group = "Styling" },
    { message = "^test", group = "Testing" },
    { message = "^chore\\(release\\)", skip = true },
    { message = "^chore|^ci", group = "Miscellaneous" },
]

filter_commits = false
tag_pattern = "v[0-9].*"
```

### 3.4 Release Notes Template

```markdown
# Claude Forge v{VERSION}

{CHANGELOG_BODY}

---

## Installation

**One-liner (macOS/Linux):**
```bash
curl -fsSL https://forge.dev/install.sh | sh
```

**Homebrew:**
```bash
brew install anthropics/tap/claude-forge
```

**Cargo:**
```bash
cargo install claude-forge
```

**Manual download:** Choose your platform below.

## Checksums

See `SHA256SUMS.txt` attached to this release, or verify individually:
```bash
sha256sum -c forge-v{VERSION}-{TARGET}.tar.gz.sha256
```

## Platform Binaries

| Platform | Architecture | Download |
|----------|-------------|----------|
| macOS | Apple Silicon (M1+) | `forge-v{VERSION}-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `forge-v{VERSION}-x86_64-apple-darwin.tar.gz` |
| Linux | x86_64 (glibc) | `forge-v{VERSION}-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | x86_64 (static/musl) | `forge-v{VERSION}-x86_64-unknown-linux-musl.tar.gz` |
| Linux | ARM64 | `forge-v{VERSION}-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | x86_64 | `forge-v{VERSION}-x86_64-pc-windows-msvc.zip` |
| Windows | ARM64 | `forge-v{VERSION}-aarch64-pc-windows-msvc.zip` |
```

### 3.5 Pre-release vs Stable

| Tag Pattern | Release Type | Draft? | Pre-release Flag |
|-------------|-------------|--------|-----------------|
| `v1.0.0` | Stable | No | `false` |
| `v1.0.0-beta.1` | Pre-release | Yes (manual publish) | `true` |
| `v1.0.0-rc.1` | Release Candidate | Yes (manual publish) | `true` |
| `v0.x.y` | Development | No | `true` |

Logic in workflow:

```bash
# Determine if pre-release
if [[ "$TAG" == *"-"* ]] || [[ "$TAG" == v0.* ]]; then
  echo "prerelease=true" >> $GITHUB_OUTPUT
else
  echo "prerelease=false" >> $GITHUB_OUTPUT
fi
```

---

## 4. Install Script

### 4.1 Usage

```bash
# Default install to ~/.local/bin
curl -fsSL https://forge.dev/install.sh | sh

# Custom install directory
curl -fsSL https://forge.dev/install.sh | sh -s -- --prefix /usr/local

# Specific version
curl -fsSL https://forge.dev/install.sh | sh -s -- --version v1.2.3

# Skip checksum verification (not recommended)
curl -fsSL https://forge.dev/install.sh | sh -s -- --no-verify
```

### 4.2 Full Install Script

The install script is hosted at the project website and also included in the repository at `scripts/install.sh`.

```bash
#!/bin/sh
# Claude Forge installer
# Usage: curl -fsSL https://forge.dev/install.sh | sh
#
# Options:
#   --prefix DIR     Install to DIR/bin (default: ~/.local)
#   --version VER    Install specific version (default: latest)
#   --no-verify      Skip SHA256 checksum verification
#   --help           Show this help

set -eu

# ---------- Configuration ----------

REPO="anthropics/claude-forge"
BINARY_NAME="forge"
INSTALL_PREFIX="${HOME}/.local"
VERSION=""
VERIFY_CHECKSUM=true

# ---------- Colors ----------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# ---------- Helpers ----------

info()  { printf "${BLUE}info${NC}  %s\n" "$1"; }
warn()  { printf "${YELLOW}warn${NC}  %s\n" "$1"; }
error() { printf "${RED}error${NC} %s\n" "$1" >&2; exit 1; }
ok()    { printf "${GREEN}  ok${NC}  %s\n" "$1"; }

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        error "need '$1' (command not found)"
    fi
}

# ---------- Parse Arguments ----------

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)   INSTALL_PREFIX="$2"; shift 2 ;;
        --version)  VERSION="$2"; shift 2 ;;
        --no-verify) VERIFY_CHECKSUM=false; shift ;;
        --help)
            printf "Claude Forge installer\n\n"
            printf "Options:\n"
            printf "  --prefix DIR     Install to DIR/bin (default: ~/.local)\n"
            printf "  --version VER    Install specific version (default: latest)\n"
            printf "  --no-verify      Skip SHA256 checksum verification\n"
            printf "  --help           Show this help\n"
            exit 0
            ;;
        *) error "unknown option: $1" ;;
    esac
done

# ---------- Detect Platform ----------

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) error "unsupported operating system: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) error "unsupported architecture: $(uname -m)" ;;
    esac
}

get_target() {
    local os="$1"
    local arch="$2"

    case "${os}-${arch}" in
        macos-x86_64)   echo "x86_64-apple-darwin" ;;
        macos-aarch64)  echo "aarch64-apple-darwin" ;;
        linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64)  echo "aarch64-unknown-linux-gnu" ;;
        windows-x86_64) echo "x86_64-pc-windows-msvc" ;;
        *) error "no pre-built binary for ${os} ${arch}" ;;
    esac
}

get_extension() {
    local os="$1"
    case "$os" in
        windows) echo "zip" ;;
        *)       echo "tar.gz" ;;
    esac
}

# ---------- Main ----------

main() {
    need_cmd curl
    need_cmd uname

    local os arch target ext
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="$(get_target "$os" "$arch")"
    ext="$(get_extension "$os")"

    info "detected platform: ${os} ${arch} (${target})"

    # Resolve version
    if [ -z "$VERSION" ]; then
        need_cmd grep
        info "fetching latest release version..."
        VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
        if [ -z "$VERSION" ]; then
            error "could not determine latest version"
        fi
    fi

    info "installing claude-forge ${VERSION}"

    # Construct download URL
    local filename="forge-${VERSION}-${target}.${ext}"
    local url="https://github.com/${REPO}/releases/download/${VERSION}/${filename}"
    local checksum_url="${url}.sha256"

    # Create temp directory
    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    # Download binary
    info "downloading ${filename}..."
    curl -fsSL -o "${tmpdir}/${filename}" "$url" \
        || error "download failed -- does ${VERSION} exist for ${target}?"

    # Verify checksum
    if [ "$VERIFY_CHECKSUM" = true ]; then
        info "verifying checksum..."
        curl -fsSL -o "${tmpdir}/${filename}.sha256" "$checksum_url" \
            || error "checksum download failed"

        local expected actual
        expected="$(cut -d' ' -f1 < "${tmpdir}/${filename}.sha256")"

        if command -v sha256sum > /dev/null 2>&1; then
            actual="$(sha256sum "${tmpdir}/${filename}" | cut -d' ' -f1)"
        elif command -v shasum > /dev/null 2>&1; then
            actual="$(shasum -a 256 "${tmpdir}/${filename}" | cut -d' ' -f1)"
        else
            warn "neither sha256sum nor shasum found -- skipping verification"
            actual="$expected"
        fi

        if [ "$expected" != "$actual" ]; then
            error "checksum mismatch!\n  expected: ${expected}\n  actual:   ${actual}"
        fi
        ok "checksum verified"
    fi

    # Extract
    info "extracting..."
    case "$ext" in
        tar.gz)
            tar -xzf "${tmpdir}/${filename}" -C "$tmpdir"
            ;;
        zip)
            need_cmd unzip
            unzip -q "${tmpdir}/${filename}" -d "$tmpdir"
            ;;
    esac

    # Install
    local install_dir="${INSTALL_PREFIX}/bin"
    mkdir -p "$install_dir"

    local src="${tmpdir}/${BINARY_NAME}"
    if [ "$os" = "windows" ]; then
        src="${src}.exe"
    fi

    if [ ! -f "$src" ]; then
        # Some archive formats nest in a directory
        src="$(find "$tmpdir" -name "${BINARY_NAME}" -o -name "${BINARY_NAME}.exe" | head -1)"
        if [ -z "$src" ]; then
            error "binary not found in archive"
        fi
    fi

    cp "$src" "${install_dir}/${BINARY_NAME}"
    chmod +x "${install_dir}/${BINARY_NAME}"

    ok "installed to ${install_dir}/${BINARY_NAME}"

    # Check PATH
    case ":${PATH}:" in
        *":${install_dir}:"*)
            ;;
        *)
            warn "${install_dir} is not in your PATH"
            printf "\n  Add it to your shell profile:\n"
            printf "    ${BOLD}export PATH=\"%s:\$PATH\"${NC}\n\n" "$install_dir"
            ;;
    esac

    # Verify installation
    if command -v forge > /dev/null 2>&1; then
        local installed_version
        installed_version="$(forge --version 2>/dev/null || echo "unknown")"
        ok "forge ${installed_version} is ready"
    else
        ok "installation complete -- restart your shell or update PATH"
    fi
}

main
```

---

## 5. cargo install Support

### 5.1 crates.io Publishing Workflow

Publishing to crates.io allows users to install via:

```bash
cargo install claude-forge
```

This compiles from source on the user's machine, so it works on any platform Rust supports.

### 5.2 Cargo.toml Metadata for Publishing

```toml
[package]
name = "claude-forge"
version = "1.0.0"
edition = "2024"
rust-version = "1.82"
authors = ["Anthropic <eng@anthropic.com>"]
description = "Multi-agent Claude Code orchestrator with embedded web UI"
documentation = "https://forge.dev/docs"
homepage = "https://forge.dev"
repository = "https://github.com/anthropics/claude-forge"
license = "MIT"
keywords = ["claude", "ai", "agent", "orchestrator", "coding"]
categories = ["command-line-utilities", "development-tools", "web-programming"]
readme = "README.md"
include = [
    "src/**/*",
    "frontend/build/**/*",   # Pre-built frontend assets
    "migrations/**/*",
    "Cargo.toml",
    "Cargo.lock",
    "LICENSE",
    "README.md",
    "build.rs",
]
exclude = [
    "frontend/node_modules/**",
    "frontend/src/**",
    ".github/**",
    "tests/fixtures/**",
]

[package.metadata.docs.rs]
all-features = true

[badges]
github = { repository = "anthropics/claude-forge" }
```

### 5.3 Publishing Steps

```bash
# 1. Ensure frontend is pre-built (crates.io does not run pnpm)
cd frontend && pnpm install && pnpm build && cd ..

# 2. Verify the package
cargo publish --dry-run

# 3. Check package size (crates.io has a 10MB limit by default)
cargo package --list | head -20

# 4. Publish
cargo publish
```

**Important:** The pre-built `frontend/build/` directory must be included in the crate. The `build.rs` script should detect whether it is running from crates.io (no Node.js available) and skip the frontend build step, using the pre-built assets instead.

### 5.4 build.rs Strategy

```rust
// build.rs
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");

    // Check if pre-built frontend exists (crates.io publish scenario)
    let build_dir = std::path::Path::new("frontend/build");
    if build_dir.exists() && build_dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false) {
        println!("cargo:warning=Using pre-built frontend assets");
        return;
    }

    // Build frontend (development / CI scenario)
    let status = Command::new("pnpm")
        .args(["--dir", "frontend", "install", "--frozen-lockfile"])
        .status()
        .expect("failed to run pnpm install -- is pnpm installed?");
    assert!(status.success(), "pnpm install failed");

    let status = Command::new("pnpm")
        .args(["--dir", "frontend", "build"])
        .status()
        .expect("failed to run pnpm build");
    assert!(status.success(), "pnpm build failed");
}
```

### 5.5 Build Dependencies Documentation

Users installing via `cargo install` need:

| Dependency | Required For | Install Command |
|-----------|-------------|-----------------|
| Rust 1.82+ | Compilation | `rustup update stable` |
| C compiler (cc) | rusqlite bundled SQLite, git2 vendored libgit2 | `apt install build-essential` / Xcode CLT |
| cmake | git2 vendored libgit2 | `apt install cmake` / `brew install cmake` |
| pkg-config | Locating system libs (Linux) | `apt install pkg-config` |
| perl | OpenSSL build (if needed) | Usually pre-installed |

**Note:** `pnpm` and `Node.js` are **not** required for `cargo install` because the pre-built frontend is included in the crate.

---

## 6. Homebrew / Package Managers

### 6.1 Homebrew Tap

We maintain a custom tap at `anthropics/homebrew-tap`.

**Formula** (`Formula/claude-forge.rb`):

```ruby
class ClaudeForge < Formula
  desc "Multi-agent Claude Code orchestrator with embedded web UI"
  homepage "https://forge.dev"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/anthropics/claude-forge/releases/download/v#{version}/forge-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64_MACOS"
    end

    on_intel do
      url "https://github.com/anthropics/claude-forge/releases/download/v#{version}/forge-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X64_MACOS"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/anthropics/claude-forge/releases/download/v#{version}/forge-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64_LINUX"
    end

    on_intel do
      url "https://github.com/anthropics/claude-forge/releases/download/v#{version}/forge-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X64_LINUX"
    end
  end

  def install
    bin.install "forge"
  end

  test do
    assert_match "claude-forge #{version}", shell_output("#{bin}/forge --version")
  end
end
```

**Usage:**

```bash
brew tap anthropics/tap
brew install claude-forge
```

**Automated tap updates:** The release workflow includes a job that:
1. Computes SHA256 for each platform artifact
2. Updates the formula with new version + checksums
3. Opens a PR (or pushes directly) to the tap repository

### 6.2 AUR Package (Arch Linux)

**PKGBUILD** (`aur/PKGBUILD`):

```bash
# Maintainer: Anthropic <eng@anthropic.com>
pkgname=claude-forge-bin
pkgver=1.0.0
pkgrel=1
pkgdesc="Multi-agent Claude Code orchestrator with embedded web UI"
arch=('x86_64' 'aarch64')
url="https://github.com/anthropics/claude-forge"
license=('MIT')
provides=('claude-forge')
conflicts=('claude-forge')

source_x86_64=("https://github.com/anthropics/claude-forge/releases/download/v${pkgver}/forge-v${pkgver}-x86_64-unknown-linux-gnu.tar.gz")
source_aarch64=("https://github.com/anthropics/claude-forge/releases/download/v${pkgver}/forge-v${pkgver}-aarch64-unknown-linux-gnu.tar.gz")

sha256sums_x86_64=('PLACEHOLDER')
sha256sums_aarch64=('PLACEHOLDER')

package() {
    install -Dm755 forge "$pkgdir/usr/bin/forge"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

A separate `claude-forge` (non-`-bin`) AUR package can build from source using `cargo`.

### 6.3 Scoop Manifest (Windows)

**Manifest** (`claude-forge.json`) for the Scoop bucket:

```json
{
    "version": "1.0.0",
    "description": "Multi-agent Claude Code orchestrator with embedded web UI",
    "homepage": "https://forge.dev",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/anthropics/claude-forge/releases/download/v1.0.0/forge-v1.0.0-x86_64-pc-windows-msvc.zip",
            "hash": "PLACEHOLDER_SHA256"
        },
        "arm64": {
            "url": "https://github.com/anthropics/claude-forge/releases/download/v1.0.0/forge-v1.0.0-aarch64-pc-windows-msvc.zip",
            "hash": "PLACEHOLDER_SHA256"
        }
    },
    "bin": "forge.exe",
    "checkver": {
        "github": "https://github.com/anthropics/claude-forge"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/anthropics/claude-forge/releases/download/v$version/forge-v$version-x86_64-pc-windows-msvc.zip"
            },
            "arm64": {
                "url": "https://github.com/anthropics/claude-forge/releases/download/v$version/forge-v$version-aarch64-pc-windows-msvc.zip"
            }
        }
    }
}
```

**Usage:**

```powershell
scoop bucket add anthropics https://github.com/anthropics/scoop-bucket
scoop install claude-forge
```

### 6.4 Snap / Flatpak Considerations

| Format | Recommendation | Reason |
|--------|---------------|--------|
| Snap | Skip initially | Complex sandboxing, potential issues with file system access to `~/.claude/` and spawning `claude` subprocess |
| Flatpak | Skip initially | Same sandboxing concerns, plus Flatpak portals add latency to file operations |

Forge needs unrestricted access to the filesystem (reading project directories, `~/.claude/` config) and the ability to spawn `claude` as a subprocess. Snap and Flatpak sandboxing conflicts with these requirements. Revisit if demand warrants the engineering effort to configure sandbox permissions properly.

---

## 7. Docker Image

### 7.1 Use Cases

- Running Forge in CI/CD environments
- Server deployment for shared team use
- Environments where installing a native binary is restricted

### 7.2 Multi-arch Dockerfile

```dockerfile
# ---- Build stage ----
FROM --platform=$BUILDPLATFORM rust:1.82-bookworm AS builder

ARG TARGETPLATFORM
ARG BUILDPLATFORM

# Install Node.js + pnpm for frontend build
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g pnpm@9

WORKDIR /app

# Cache Rust dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build frontend
COPY frontend/ frontend/
RUN cd frontend && pnpm install --frozen-lockfile && pnpm build

# Build backend
COPY . .
RUN touch src/main.rs && cargo build --release

# Strip binary
RUN strip target/release/forge

# ---- Runtime stage ----
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/target/release/forge /usr/local/bin/forge

# Forge default port
EXPOSE 4173

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/forge", "health"]

ENTRYPOINT ["/usr/local/bin/forge"]
CMD ["--host", "0.0.0.0", "--port", "4173"]
```

### 7.3 Building Multi-arch Images

```bash
# Build and push multi-arch image
docker buildx create --use --name forgebuilder
docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag ghcr.io/anthropics/claude-forge:latest \
    --tag ghcr.io/anthropics/claude-forge:1.0.0 \
    --push .
```

### 7.4 Image Size Target

| Stage | Size |
|-------|------|
| Build stage | ~2GB (Rust toolchain + Node.js, discarded) |
| Runtime image | ~35-50MB (distroless base ~2MB + forge binary) |

---

## 8. Version Management

### 8.1 Semantic Versioning Policy

We follow [Semantic Versioning 2.0.0](https://semver.org/):

| Bump | When |
|------|------|
| **Major** (1.0.0 -> 2.0.0) | Breaking changes to CLI flags, config file format, API endpoints, or plugin interface |
| **Minor** (1.0.0 -> 1.1.0) | New features, new agent presets, new UI views, new API endpoints (backwards-compatible) |
| **Patch** (1.0.0 -> 1.0.1) | Bug fixes, performance improvements, dependency updates, UI polish |

**Pre-release identifiers:**

- `alpha` -- Incomplete features, API may change
- `beta` -- Feature-complete, may have bugs
- `rc` -- Release candidate, final testing

### 8.2 Version Bumping Automation

We use **cargo-release** for version bumping and tag creation:

```bash
# Install
cargo install cargo-release

# Patch release
cargo release patch --execute

# Minor release
cargo release minor --execute

# Major release
cargo release major --execute

# Pre-release
cargo release --execute 1.1.0-beta.1
```

`cargo-release` handles:
1. Bumping `version` in `Cargo.toml`
2. Bumping `version` in `frontend/package.json`
3. Updating `CHANGELOG.md` via git-cliff
4. Creating a git commit
5. Creating and pushing the git tag

**Configuration** (`release.toml`):

```toml
[workspace]
allow-branch = ["main"]
sign-commit = true
sign-tag = true
push = true
publish = false  # We publish separately in CI

pre-release-commit-message = "chore(release): prepare v{{version}}"
tag-message = "v{{version}}"
tag-prefix = "v"
tag-name = "v{{version}}"

pre-release-replacements = [
    { file = "frontend/package.json", search = "\"version\": \".*\"", replace = "\"version\": \"{{version}}\"" },
]

pre-release-hook = ["git", "cliff", "-o", "CHANGELOG.md", "--tag", "v{{version}}"]
```

### 8.3 Changelog Generation

The changelog is generated automatically by `git-cliff` during the release process (see section 3.3 for configuration).

Commit message convention:

```
feat(agents): add GPT-4 agent preset
fix(ws): handle WebSocket reconnection on network change
perf(db): batch SQLite writes for 3x throughput improvement
refactor(frontend): migrate AgentForm to Svelte 5 runes
docs(api): document /api/sessions endpoint
ci: add Windows ARM64 to build matrix
chore(deps): update axum to 0.8.1
```

### 8.4 Version Embedded in Binary

The version is embedded at compile time via `Cargo.toml` and accessible at runtime:

```rust
// src/main.rs
use clap::Parser;

#[derive(Parser)]
#[command(name = "forge")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Multi-agent Claude Code orchestrator")]
struct Cli {
    // ...
}
```

For the full build metadata (includes git SHA and build date):

```rust
// build.rs addition
fn main() {
    // ... existing build.rs code ...

    // Embed git SHA
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok();
    if let Some(output) = output {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=FORGE_GIT_SHA={sha}");
    }

    // Embed build timestamp
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    println!("cargo:rustc-env=FORGE_BUILD_DATE={now}");
}
```

```rust
// Usage in version display
pub fn version_long() -> String {
    format!(
        "{} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("FORGE_GIT_SHA"),
        env!("FORGE_BUILD_DATE"),
    )
}
// Output: 1.0.0 (a1b2c3d 2026-02-25T14:30:00Z)
```

---

## 9. Quality Gates Before Release

Every release must pass all of the following before the tag is pushed:

### 9.1 Automated Gates (enforced in CI)

| Gate | Command | Requirement |
|------|---------|-------------|
| Rust formatting | `cargo fmt --all -- --check` | Zero diffs |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Zero warnings |
| Backend tests | `cargo test --workspace` | All pass |
| Frontend lint | `cd frontend && pnpm lint` | Zero errors |
| Frontend types | `cd frontend && pnpm svelte-check` | Zero errors |
| Frontend tests | `cd frontend && pnpm test` | All pass |
| Security audit | `cargo audit` | No known vulnerabilities (RUSTSEC) |
| License check | `cargo deny check licenses` | No disallowed licenses |
| Supply chain | `cargo deny check bans` | No banned crates |

### 9.2 Binary Size Budget

| Target | Maximum Size | Typical Size |
|--------|-------------|-------------|
| macOS (either arch) | 50 MB | ~28-35 MB |
| Linux glibc (either arch) | 50 MB | ~25-32 MB |
| Linux musl (x86_64) | 55 MB | ~30-38 MB |
| Windows (either arch) | 55 MB | ~30-40 MB |

> **Note:** Target: <35 MB optimized, <50 MB acceptable. See VISION for <30 MB aspirational target.

The release workflow includes a size check step:

```bash
MAX_SIZE=$((50 * 1024 * 1024))  # 50 MB
ACTUAL_SIZE=$(stat -f%z target/release/forge 2>/dev/null || stat -c%s target/release/forge)
if [ "$ACTUAL_SIZE" -gt "$MAX_SIZE" ]; then
    echo "::error::Binary too large: ${ACTUAL_SIZE} bytes (max: ${MAX_SIZE})"
    exit 1
fi
```

### 9.3 Manual Smoke Test Checklist

Before tagging a release, a maintainer should verify on at least one platform:

- [ ] `forge --version` prints correct version
- [ ] `forge` starts web UI, opens browser
- [ ] Can create an agent via the web UI
- [ ] Can run a prompt against an agent
- [ ] WebSocket streaming works (real-time output)
- [ ] Session persistence works (restart forge, sessions preserved)
- [ ] MCP server configuration works
- [ ] CLAUDE.md editor saves correctly
- [ ] Export to JSON/Markdown works
- [ ] `forge --help` output is correct and complete

---

## 10. Complete GitHub Actions Workflows

### 10.1 CI Workflow (`ci.yml`)

```yaml
# .github/workflows/ci.yml
#
# Runs on every pull request and push to main.
# Provides fast feedback on code quality and correctness.

name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"
  SCCACHE_GHA_ENABLED: true
  RUSTC_WRAPPER: sccache

jobs:
  # ============================================================
  # Rust lint and format check
  # ============================================================
  lint-rust:
    name: Lint (Rust)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.6

      - name: Cache Cargo registry and target
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: lint-rust-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}
          restore-keys: |
            lint-rust-${{ runner.os }}-

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

  # ============================================================
  # Frontend lint and type check
  # ============================================================
  lint-frontend:
    name: Lint (Frontend)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Get pnpm store directory
        id: pnpm-cache
        shell: bash
        run: echo "store=$(pnpm store path)" >> $GITHUB_OUTPUT

      - name: Cache pnpm
        uses: actions/cache@v4
        with:
          path: ${{ steps.pnpm-cache.outputs.store }}
          key: pnpm-${{ runner.os }}-${{ hashFiles('frontend/pnpm-lock.yaml') }}
          restore-keys: |
            pnpm-${{ runner.os }}-

      - name: Install dependencies
        run: cd frontend && pnpm install --frozen-lockfile

      - name: Svelte check (type checking)
        run: cd frontend && pnpm svelte-check

      - name: ESLint
        run: cd frontend && pnpm lint

      - name: Prettier (format check)
        run: cd frontend && pnpm format --check

  # ============================================================
  # Rust tests
  # ============================================================
  test-rust:
    name: Test (Rust)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.6

      - name: Cache Cargo registry and target
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: test-rust-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}
          restore-keys: |
            test-rust-${{ runner.os }}-

      # Frontend must be built for rust-embed to find the files
      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Build frontend
        run: cd frontend && pnpm install --frozen-lockfile && pnpm build

      - name: Run tests
        run: cargo test --workspace --all-targets

      - name: Run doc tests
        run: cargo test --workspace --doc

  # ============================================================
  # Frontend tests
  # ============================================================
  test-frontend:
    name: Test (Frontend)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Cache pnpm
        uses: actions/cache@v4
        with:
          path: |
            frontend/node_modules
          key: test-frontend-${{ runner.os }}-${{ hashFiles('frontend/pnpm-lock.yaml') }}
          restore-keys: |
            test-frontend-${{ runner.os }}-

      - name: Install dependencies
        run: cd frontend && pnpm install --frozen-lockfile

      - name: Run unit tests
        run: cd frontend && pnpm test

  # ============================================================
  # Security audit
  # ============================================================
  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Install cargo-deny
        run: cargo install cargo-deny

      - name: Cargo audit (RUSTSEC advisories)
        run: cargo audit

      - name: Cargo deny (licenses + bans)
        run: cargo deny check

      - name: Frontend audit
        run: |
          cd frontend
          npx pnpm install --frozen-lockfile
          npx pnpm audit --audit-level=moderate || true  # warn, don't block

  # ============================================================
  # Build check (compile verification, no artifacts)
  # ============================================================
  build-check:
    name: Build Check (${{ matrix.os }})
    needs: [lint-rust, lint-frontend]
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-14
            target: aarch64-apple-darwin
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.6

      - name: Cache Cargo registry and target
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: build-${{ matrix.target }}-${{ hashFiles('Cargo.lock') }}
          restore-keys: |
            build-${{ matrix.target }}-

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Build frontend
        run: cd frontend && pnpm install --frozen-lockfile && pnpm build

      - name: Compile check
        run: cargo check --workspace --all-targets --target ${{ matrix.target }}
```

### 10.2 Release Workflow (`release.yml`)

```yaml
# .github/workflows/release.yml
#
# Triggered by pushing a version tag (v*) or manual dispatch.
# Builds release binaries for all platforms and creates a GitHub Release.

name: Release

on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      tag:
        description: 'Release tag (e.g., v1.0.0)'
        required: true
        type: string

permissions:
  contents: write    # Create releases and upload assets
  packages: write    # Push Docker images (optional)
  id-token: write    # Needed for provenance/signing

env:
  CARGO_TERM_COLOR: always

jobs:
  # ============================================================
  # Validate and prepare
  # ============================================================
  prepare:
    name: Prepare Release
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      tag: ${{ steps.version.outputs.tag }}
      prerelease: ${{ steps.version.outputs.prerelease }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for changelog

      - name: Determine version
        id: version
        run: |
          if [ "${{ github.event_name }}" = "workflow_dispatch" ]; then
            TAG="${{ github.event.inputs.tag }}"
          else
            TAG="${GITHUB_REF#refs/tags/}"
          fi
          VERSION="${TAG#v}"

          # Determine if pre-release
          if [[ "$TAG" == *"-"* ]] || [[ "$TAG" == v0.* ]]; then
            PRERELEASE=true
          else
            PRERELEASE=false
          fi

          echo "tag=${TAG}" >> $GITHUB_OUTPUT
          echo "version=${VERSION}" >> $GITHUB_OUTPUT
          echo "prerelease=${PRERELEASE}" >> $GITHUB_OUTPUT

          echo "Release: ${TAG} (version=${VERSION}, prerelease=${PRERELEASE})"

      - name: Verify Cargo.toml version matches tag
        run: |
          CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
          TAG_VERSION="${{ steps.version.outputs.version }}"
          if [ "$CARGO_VERSION" != "$TAG_VERSION" ]; then
            echo "::error::Cargo.toml version (${CARGO_VERSION}) does not match tag version (${TAG_VERSION})"
            exit 1
          fi

      - name: Generate changelog
        run: |
          cargo install git-cliff
          git cliff --latest --strip header > RELEASE_NOTES.md
          cat RELEASE_NOTES.md

      - name: Upload changelog
        uses: actions/upload-artifact@v4
        with:
          name: release-notes
          path: RELEASE_NOTES.md

  # ============================================================
  # Build frontend (shared across all targets)
  # ============================================================
  build-frontend:
    name: Build Frontend
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Cache pnpm
        uses: actions/cache@v4
        with:
          path: frontend/node_modules
          key: frontend-${{ hashFiles('frontend/pnpm-lock.yaml') }}

      - name: Build frontend
        run: cd frontend && pnpm install --frozen-lockfile && pnpm build

      - name: Upload frontend build
        uses: actions/upload-artifact@v4
        with:
          name: frontend-build
          path: frontend/build/
          retention-days: 1

  # ============================================================
  # Build platform binaries
  # ============================================================
  build:
    name: Build (${{ matrix.name }})
    needs: [prepare, build-frontend]
    strategy:
      fail-fast: false
      matrix:
        include:
          # ---- macOS ----
          - name: macOS (Apple Silicon)
            target: aarch64-apple-darwin
            os: macos-14
            cross: false
            ext: tar.gz
            strip: strip

          - name: macOS (Intel)
            target: x86_64-apple-darwin
            os: macos-13
            cross: false
            ext: tar.gz
            strip: strip

          # ---- Linux ----
          - name: Linux x64 (glibc)
            target: x86_64-unknown-linux-gnu
            os: ubuntu-22.04
            cross: false
            ext: tar.gz
            strip: strip

          - name: Linux ARM64 (glibc)
            target: aarch64-unknown-linux-gnu
            os: ubuntu-22.04
            cross: true
            ext: tar.gz
            strip: llvm-strip

          - name: Linux x64 (musl/static)
            target: x86_64-unknown-linux-musl
            os: ubuntu-22.04
            cross: true
            ext: tar.gz
            strip: llvm-strip

          # ---- Windows ----
          - name: Windows x64
            target: x86_64-pc-windows-msvc
            os: windows-2022
            cross: false
            ext: zip
            strip: ""

          - name: Windows ARM64
            target: aarch64-pc-windows-msvc
            os: windows-2022
            cross: false
            ext: zip
            strip: ""

    runs-on: ${{ matrix.os }}
    env:
      BINARY_NAME: forge
      ARTIFACT_NAME: forge-${{ needs.prepare.outputs.tag }}-${{ matrix.target }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: release-${{ matrix.target }}-${{ hashFiles('Cargo.lock') }}
          restore-keys: |
            release-${{ matrix.target }}-

      - name: Download frontend build
        uses: actions/download-artifact@v4
        with:
          name: frontend-build
          path: frontend/build/

      # ---- Cross-compilation setup ----
      - name: Install cross-rs
        if: matrix.cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      # ---- Windows ARM64 target setup ----
      - name: Add Windows ARM64 target
        if: matrix.target == 'aarch64-pc-windows-msvc'
        run: rustup target add aarch64-pc-windows-msvc

      # ---- Build ----
      - name: Build (native)
        if: "!matrix.cross"
        run: cargo build --release --target ${{ matrix.target }}

      - name: Build (cross)
        if: matrix.cross
        run: cross build --release --target ${{ matrix.target }}

      # ---- Strip ----
      - name: Strip binary (Unix)
        if: matrix.strip != '' && runner.os != 'Windows'
        run: ${{ matrix.strip }} target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}

      # ---- Binary size check ----
      - name: Check binary size (Unix)
        if: runner.os != 'Windows'
        run: |
          MAX_SIZE=$((55 * 1024 * 1024))
          BINARY="target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}"
          ACTUAL_SIZE=$(stat -f%z "$BINARY" 2>/dev/null || stat -c%s "$BINARY")
          echo "Binary size: $(( ACTUAL_SIZE / 1024 / 1024 )) MB"
          if [ "$ACTUAL_SIZE" -gt "$MAX_SIZE" ]; then
            echo "::error::Binary too large: ${ACTUAL_SIZE} bytes (max: ${MAX_SIZE})"
            exit 1
          fi

      - name: Check binary size (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          $maxSize = 55 * 1024 * 1024
          $binary = "target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}.exe"
          $actualSize = (Get-Item $binary).Length
          $sizeMB = [math]::Round($actualSize / 1MB, 1)
          Write-Host "Binary size: ${sizeMB} MB"
          if ($actualSize -gt $maxSize) {
            Write-Error "Binary too large: ${actualSize} bytes (max: ${maxSize})"
            exit 1
          }

      # ---- Package ----
      - name: Package (tar.gz)
        if: matrix.ext == 'tar.gz'
        run: |
          mkdir -p staging
          cp target/${{ matrix.target }}/release/${{ env.BINARY_NAME }} staging/
          cp LICENSE README.md staging/ 2>/dev/null || true
          cd staging
          tar czf ../${{ env.ARTIFACT_NAME }}.tar.gz *
          cd ..

      - name: Package (zip)
        if: matrix.ext == 'zip'
        shell: pwsh
        run: |
          New-Item -ItemType Directory -Force -Path staging
          Copy-Item "target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}.exe" staging/
          Copy-Item LICENSE, README.md staging/ -ErrorAction SilentlyContinue
          Compress-Archive -Path staging/* -DestinationPath "${{ env.ARTIFACT_NAME }}.zip"

      # ---- Checksum ----
      - name: Generate checksum (Unix)
        if: runner.os != 'Windows'
        run: |
          shasum -a 256 ${{ env.ARTIFACT_NAME }}.${{ matrix.ext }} \
            > ${{ env.ARTIFACT_NAME }}.${{ matrix.ext }}.sha256

      - name: Generate checksum (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          $hash = (Get-FileHash "${{ env.ARTIFACT_NAME }}.${{ matrix.ext }}" -Algorithm SHA256).Hash.ToLower()
          "$hash  ${{ env.ARTIFACT_NAME }}.${{ matrix.ext }}" | Out-File -Encoding ascii "${{ env.ARTIFACT_NAME }}.${{ matrix.ext }}.sha256"

      # ---- macOS Signing & Notarization ----
      - name: Import Apple certificate
        if: runner.os == 'macOS' && env.APPLE_CERTIFICATE_P12 != ''
        env:
          APPLE_CERTIFICATE_P12: ${{ secrets.APPLE_CERTIFICATE_P12 }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          echo "$APPLE_CERTIFICATE_P12" | base64 --decode > certificate.p12
          security create-keychain -p "" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "" build.keychain
          security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "" build.keychain
          rm certificate.p12

      - name: Sign macOS binary
        if: runner.os == 'macOS' && env.APPLE_CERTIFICATE_P12 != ''
        env:
          APPLE_CERTIFICATE_P12: ${{ secrets.APPLE_CERTIFICATE_P12 }}
        run: |
          codesign --force --options runtime \
            --sign "Developer ID Application" \
            target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}

      - name: Notarize macOS binary
        if: runner.os == 'macOS' && env.APPLE_ID != ''
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_APP_PASSWORD: ${{ secrets.APPLE_APP_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          ditto -c -k --keepParent \
            target/${{ matrix.target }}/release/${{ env.BINARY_NAME }} \
            forge-notarize.zip
          xcrun notarytool submit forge-notarize.zip \
            --apple-id "$APPLE_ID" \
            --password "$APPLE_APP_PASSWORD" \
            --team-id "$APPLE_TEAM_ID" \
            --wait

      # ---- Windows Signing ----
      - name: Sign Windows binary
        if: runner.os == 'Windows' && env.WINDOWS_CERT_PFX != ''
        env:
          WINDOWS_CERT_PFX: ${{ secrets.WINDOWS_CERT_PFX }}
          WINDOWS_CERT_PASSWORD: ${{ secrets.WINDOWS_CERT_PASSWORD }}
        shell: pwsh
        run: |
          $certBytes = [Convert]::FromBase64String($env:WINDOWS_CERT_PFX)
          [IO.File]::WriteAllBytes("cert.pfx", $certBytes)
          & signtool sign /f cert.pfx /p $env:WINDOWS_CERT_PASSWORD `
            /tr http://timestamp.digicert.com /td sha256 /fd sha256 `
            "target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}.exe"
          Remove-Item cert.pfx

      # ---- Upload ----
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ env.ARTIFACT_NAME }}
          path: |
            ${{ env.ARTIFACT_NAME }}.${{ matrix.ext }}
            ${{ env.ARTIFACT_NAME }}.${{ matrix.ext }}.sha256
          retention-days: 3

  # ============================================================
  # Create GitHub Release
  # ============================================================
  release:
    name: Create Release
    needs: [prepare, build]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts/

      - name: Flatten artifacts
        run: |
          mkdir -p dist
          find artifacts -type f \( -name '*.tar.gz' -o -name '*.zip' -o -name '*.sha256' \) -exec cp {} dist/ \;
          ls -lh dist/

      - name: Generate combined checksums
        run: |
          cd dist
          sha256sum forge-* > SHA256SUMS.txt
          cat SHA256SUMS.txt

      - name: Download release notes
        uses: actions/download-artifact@v4
        with:
          name: release-notes
          path: .

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ needs.prepare.outputs.tag }}
          name: Claude Forge ${{ needs.prepare.outputs.tag }}
          body_path: RELEASE_NOTES.md
          draft: ${{ needs.prepare.outputs.prerelease == 'true' }}
          prerelease: ${{ needs.prepare.outputs.prerelease == 'true' }}
          files: |
            dist/*
          fail_on_unmatched_files: true
          generate_release_notes: false  # We use git-cliff instead

  # ============================================================
  # Publish to crates.io (stable releases only)
  # ============================================================
  publish-crate:
    name: Publish to crates.io
    needs: [prepare, release]
    if: needs.prepare.outputs.prerelease == 'false'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install Node.js + pnpm
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Build frontend (for inclusion in crate)
        run: cd frontend && pnpm install --frozen-lockfile && pnpm build

      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: cargo publish --allow-dirty  # frontend/build/ is not in git

  # ============================================================
  # Update Homebrew tap
  # ============================================================
  update-homebrew:
    name: Update Homebrew Tap
    needs: [prepare, release]
    if: needs.prepare.outputs.prerelease == 'false'
    runs-on: ubuntu-latest
    steps:
      - name: Download artifacts for checksums
        uses: actions/download-artifact@v4
        with:
          path: artifacts/

      - name: Compute checksums
        id: checksums
        run: |
          for target in x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
            FILE="artifacts/forge-${{ needs.prepare.outputs.tag }}-${target}/forge-${{ needs.prepare.outputs.tag }}-${target}.tar.gz"
            SHA=$(sha256sum "$FILE" | cut -d' ' -f1)
            KEY=$(echo "$target" | tr '-' '_')
            echo "sha_${KEY}=${SHA}" >> $GITHUB_OUTPUT
          done

      - name: Update Homebrew formula
        uses: peter-evans/repository-dispatch@v3
        with:
          token: ${{ secrets.HOMEBREW_TAP_TOKEN }}
          repository: anthropics/homebrew-tap
          event-type: update-formula
          client-payload: |
            {
              "formula": "claude-forge",
              "version": "${{ needs.prepare.outputs.version }}",
              "sha256_aarch64_apple_darwin": "${{ steps.checksums.outputs.sha_aarch64_apple_darwin }}",
              "sha256_x86_64_apple_darwin": "${{ steps.checksums.outputs.sha_x86_64_apple_darwin }}",
              "sha256_x86_64_unknown_linux_gnu": "${{ steps.checksums.outputs.sha_x86_64_unknown_linux_gnu }}",
              "sha256_aarch64_unknown_linux_gnu": "${{ steps.checksums.outputs.sha_aarch64_unknown_linux_gnu }}"
            }

  # ============================================================
  # Docker image (stable releases only)
  # ============================================================
  docker:
    name: Docker Image
    needs: [prepare, release]
    if: needs.prepare.outputs.prerelease == 'false'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push multi-arch image
        uses: docker/build-push-action@v6
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            ghcr.io/anthropics/claude-forge:latest
            ghcr.io/anthropics/claude-forge:${{ needs.prepare.outputs.version }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

---

## Appendix A: Required GitHub Secrets

| Secret | Purpose | Required? |
|--------|---------|-----------|
| `APPLE_CERTIFICATE_P12` | macOS code signing certificate (base64) | Optional (signing) |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password | Optional (signing) |
| `APPLE_ID` | Apple ID for notarization | Optional (notarization) |
| `APPLE_APP_PASSWORD` | App-specific password for notarization | Optional (notarization) |
| `APPLE_TEAM_ID` | Apple Developer Team ID | Optional (notarization) |
| `WINDOWS_CERT_PFX` | Windows code signing certificate (base64) | Optional (signing) |
| `WINDOWS_CERT_PASSWORD` | Certificate password | Optional (signing) |
| `CARGO_REGISTRY_TOKEN` | crates.io API token | Required (publishing) |
| `HOMEBREW_TAP_TOKEN` | PAT with repo access to homebrew-tap | Required (Homebrew) |

## Appendix B: Repository Files Checklist

Files that should exist in the repository for the full CI/CD to work:

```
.github/
  workflows/
    ci.yml                    # CI workflow (this document, section 10.1)
    release.yml               # Release workflow (this document, section 10.2)
cliff.toml                    # git-cliff changelog config (section 3.3)
release.toml                  # cargo-release config (section 8.2)
Cross.toml                    # cross-rs config (section 2.3)
deny.toml                     # cargo-deny config (licenses, bans)
Dockerfile                    # Multi-arch Docker build (section 7.2)
scripts/
  install.sh                  # Install script (section 4.2)
aur/
  PKGBUILD                    # AUR package (section 6.2)
```

## Appendix C: Estimated CI/CD Timing

| Job | Duration | Notes |
|-----|----------|-------|
| CI lint (Rust) | ~2 min | With sccache |
| CI lint (frontend) | ~1 min | Cached node_modules |
| CI test (Rust) | ~4 min | Includes frontend build for rust-embed |
| CI test (frontend) | ~1 min | Unit tests only |
| CI audit | ~2 min | cargo audit + deny |
| CI build check | ~5 min | Two targets |
| **Total CI** | **~5 min** | **Jobs run in parallel** |
| Release: build frontend | ~2 min | Shared, runs once |
| Release: build per target | ~8-15 min | Depends on target; cross builds are slower |
| Release: create release | ~1 min | Download + upload artifacts |
| **Total Release** | **~18-20 min** | **Matrix builds are parallel** |

## Appendix D: deny.toml Reference Configuration

```toml
# deny.toml -- cargo-deny configuration

[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Zlib",
    "BSL-1.0",
    "CC0-1.0",
    "OpenSSL",
]
copyleft = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "warn"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```
