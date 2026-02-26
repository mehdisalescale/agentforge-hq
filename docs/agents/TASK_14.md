# TASK 14 — README.md

**Status:** pending
**Priority:** high
**Track:** Phase A — ship

---

## Context

No README. GitHub landing page is empty. Users need to know what this is and how to use it.

## Task

Create root `README.md` with these sections:

1. **Title + one-liner**: "Claude Forge — Multi-agent Claude Code orchestrator. Rust + Svelte, single binary."
2. **What it does** (3-4 bullets): spawn agents, stream output, manage sessions, export results
3. **Quick start**:
   ```
   # Download from GitHub Releases (or build from source)
   ./forge
   # Open http://127.0.0.1:4173
   ```
4. **Build from source**:
   ```bash
   cd frontend && pnpm install && pnpm build && cd ..
   cargo build --release
   ./target/release/forge
   ```
5. **Configuration** (env vars table):
   | Var | Default | Description |
   |-----|---------|-------------|
   | FORGE_DB_PATH | ~/.claude-forge/forge.db | SQLite database |
   | FORGE_PORT | 4173 | Server port |
   | FORGE_HOST | 127.0.0.1 | Bind address |
   | FORGE_CORS_ORIGIN | * | CORS allowed origin |
   | FORGE_CLI_COMMAND | claude | CLI executable |
6. **Architecture** (text diagram showing the 8 crates)
7. **License** (placeholder — check with user)

Keep it under 100 lines. No emojis.

## Files to create

- `README.md` (project root)

## Verify

Looks good when viewed on GitHub.

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: Created root README.md with title, one-liner, what it does, quick start, build from source, configuration table, architecture (8 crates), license placeholder. Under 100 lines, no emojis.
- [ ] Notes:
