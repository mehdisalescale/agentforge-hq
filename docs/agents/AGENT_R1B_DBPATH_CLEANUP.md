# Agent R1-B: DB Path Fix + Git Cleanup + Root Docs

> Fix the database default path to match docs, remove site/ from git tracking, and update root-level documentation.

## Step 1: Read Context

- `CLAUDE.md` — current project docs
- `README.md` — current README
- `crates/forge-app/src/main.rs` — find `default_db_path()` function (~line 32)
- `crates/forge-mcp-bin/src/main.rs` — find any DB path references
- `crates/forge-core/src/events.rs` — count ForgeEvent variants for CLAUDE.md update

## Step 2: Fix DB Default Path

In `crates/forge-app/src/main.rs`, replace the `default_db_path()` function:

**Current:**
```rust
fn default_db_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/.claude-forge/forge.db", home)
}
```

**Replace with:**
```rust
fn default_db_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let new_path = format!("{}/.agentforge/forge.db", home);
    let legacy_path = format!("{}/.claude-forge/forge.db", home);

    // Graceful migration: use legacy path if it exists and new path doesn't
    if !std::path::Path::new(&new_path).exists() && std::path::Path::new(&legacy_path).exists() {
        eprintln!("Note: Found database at legacy path ~/.claude-forge/forge.db");
        eprintln!("      Consider moving to ~/.agentforge/forge.db");
        return legacy_path;
    }
    new_path
}
```

Also ensure the parent directory is created before opening the DB. Check if there's a `std::fs::create_dir_all` call for the DB path directory. If not, add one.

Check `crates/forge-mcp-bin/src/main.rs` for any hardcoded DB path references and fix those too.

## Step 3: Remove site/ from Git Tracking

The `site/` directory contains MkDocs build output (~60+ files) that should not be tracked. `.gitignore` already has `site/` but the files were committed before that.

Run:
```bash
git rm -r --cached site/
```

Do NOT delete the actual `site/` directory — just untrack it.

## Step 4: Update CLAUDE.md

Read `crates/forge-core/src/events.rs` and count the exact ForgeEvent variants.

In `CLAUDE.md`, update:
1. Event count — change "35 variants" to the actual count
2. DB path reference — ensure it says `~/.agentforge/forge.db`
3. MCP tool count — currently says 10, should be 19 (tools were added in Waves 4-5)
4. Workspace crate descriptions — update `forge-mcp-bin` to say "MCP stdio server (rmcp, 19 tools)"
5. Any other stale numbers you find

## Step 5: Update README.md

- Ensure build instructions use `pnpm` (not `npm`)
- Fix DB path if mentioned
- Update any stale feature counts

## Step 6: Verify

```bash
cargo check 2>&1 | head -20   # zero warnings
cargo test 2>&1 | tail -5      # all pass
git status                      # site/ files show as deleted from index
```

## Rules

- Touch ONLY: `crates/forge-app/src/main.rs`, `crates/forge-mcp-bin/src/main.rs`, `CLAUDE.md`, `README.md`
- Do NOT touch files under `site-docs/` (Agent R1-A handles those)
- Do NOT touch `.github/workflows/` or `Cargo.toml` (Agent R1-C handles those)
- Do NOT touch `crates/forge-api/` or `crates/forge-core/`
- Run `cargo check` and `cargo test` before reporting done

## Report

When done, create `docs/agents/REPORT_R1B.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
FILES_MODIFIED: [list]
DB_PATH_CHANGE: old_path → new_path
SITE_UNTRACKED: yes/no (file count removed from index)
CLAUDE_MD_UPDATES: [list of changes]
CARGO_CHECK: pass/fail
CARGO_TEST: pass/fail
```
