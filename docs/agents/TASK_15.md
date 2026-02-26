# TASK 15 — Update NORTH_STAR.md honestly

**Status:** pending
**Priority:** medium
**Track:** Phase A — polish

---

## Context

NORTH_STAR.md claims features that don't exist: "MCP server editor", "Hooks editor", "Multi-pane tab layout", "Directory picker", "CLAUDE.md editor", "split view". These are from the old prototype (`claude-forge/`), not the current codebase.

## Task

In `NORTH_STAR.md`, replace the "What Works" section under "Current State" with what actually exists NOW:

**What Works (verified in code):**
- 8 Rust crates: forge-core, forge-agent, forge-db, forge-api, forge-process, forge-safety, forge-mcp, forge-app
- Agent CRUD + 9 presets (API + frontend)
- Process spawning with `--output-format stream-json` + `--resume`
- Real-time WebSocket event streaming
- Event persistence via BatchWriter (50 events / 2s flush to SQLite)
- Session CRUD + export (JSON / Markdown) with status lifecycle
- Run endpoint with real Claude CLI spawn and directory support
- Embedded frontend via rust-embed (single binary)
- Graceful shutdown with signal handling
- TraceLayer request logging
- Configurable CORS
- Skills and Workflows list API (Phase 2 seed)
- CI: GitHub Actions

**What's Missing for v0.2.0:**
- MCP server (stdio transport + 10 tools)
- Circuit breaker, rate limiter, cost tracking
- Markdown rendering in stream output
- GitHub Release binaries

**Remove** all references to: MCP server editor, Hooks editor, Multi-pane tab layout, Directory picker, CLAUDE.md editor, split view, agent edit page, AgentForm component.

Also update "Phase 1 sprint status" to reflect completion.

## Files to edit

- `NORTH_STAR.md`

## Verify

Read it. Every claim should be verifiable by looking at the code.

---

## Report

*Agent: fill this in when done.*

- [x] What was changed: Replaced "What Works" with verified-in-code list; "What's Missing" now v0.2.0 (MCP, circuit breaker, rate limiter, cost tracking, markdown rendering, GitHub Release binaries). Removed all references to MCP server editor, Hooks editor, multi-pane/split view, directory picker, CLAUDE.md editor, agent edit page, AgentForm. Updated Phase 1 sprint status and Phase A table.
- [ ] Notes:
