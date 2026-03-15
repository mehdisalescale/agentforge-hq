# Wave 4 Launch: Configure → Execute → Observe

> 3 agents, zero file conflicts. Each agent gets its own Claude Code tab.

## What This Wave Does

Transitions AgentForge from "middleware injection" to "file-based configuration + hook observation":

- **W4-A**: Builds AgentConfigurator (generates CLAUDE.md + hooks.json per persona), removes SkillInjection and TaskTypeDetection from chain
- **W4-B**: Builds HookReceiver endpoints (pre-tool, post-tool, stop) that Claude Code hooks POST to, migrates SecurityScan to hook handler
- **W4-C**: Adds 3 MCP tools (classify_task, list_personas, get_budget), updates docs

## Conflict Matrix

| File | W4-A | W4-B | W4-C |
|------|------|------|------|
| `crates/forge-api/src/configurator.rs` | CREATE | — | — |
| `crates/forge-api/src/middleware.rs` | MODIFY (SpawnMiddleware) | — | — |
| `crates/forge-api/src/routes/run.rs` | MODIFY (chain assembly) | — | — |
| `crates/forge-api/src/lib.rs` | MODIFY (add mod) | — | — |
| `crates/forge-api/src/routes/hooks.rs` | — | REWRITE | — |
| `crates/forge-core/src/events.rs` | — | MAY MODIFY | — |
| `crates/forge-mcp-bin/src/main.rs` | — | — | MODIFY |
| `crates/forge-mcp-bin/Cargo.toml` | — | — | MODIFY |
| `NORTH_STAR.md` | — | — | MODIFY |
| `CLAUDE.md` | — | — | MODIFY |

**Zero conflicts. All three can run in parallel.**

## How to Launch

Open 3 Claude Code tabs. In each:

### Tab 1 — Agent W4-A (AgentConfigurator)
```
Read docs/agents/AGENT_W4A_CONFIGURATOR.md and execute all steps. You are Agent W4-A. When done, append your report to the bottom of that file and commit.
```

### Tab 2 — Agent W4-B (HookReceiver)
```
Read docs/agents/AGENT_W4B_HOOKRECEIVER.md and execute all steps. You are Agent W4-B. When done, append your report to the bottom of that file and commit.
```

### Tab 3 — Agent W4-C (MCP + Docs)
```
Read docs/agents/AGENT_W4C_CLEANUP.md and execute all steps. You are Agent W4-C. When done, append your report to the bottom of that file and commit.
```

## After All Agents Complete

```bash
# Verify everything together
cargo check 2>&1 | grep -c warning   # must be 0
cargo test 2>&1 | grep "FAILED"      # no failures
cd frontend && pnpm build 2>&1       # must build cleanly

# Integration test: check new endpoints respond
cargo build --release
./target/release/forge &
sleep 2
curl -s http://127.0.0.1:4173/api/v1/hooks/pre-tool -X POST -H 'Content-Type: application/json' -d '{"session_id":"test","tool_name":"Read"}'
curl -s http://127.0.0.1:4173/api/v1/settings
kill %1
```

## Result After Wave 4

- Middleware chain: 7 deep (down from 9) — SkillInjection and TaskTypeDetection removed
- AgentConfigurator generates CLAUDE.md + hooks.json per persona before spawn
- Claude Code hooks POST events back to AgentForge (pre-tool, post-tool, stop)
- SecurityScan runs per-tool-use (more granular) instead of post-session
- 3 new MCP tools: classify_task, list_personas, get_budget (13 total)
- Docs updated to reflect new architecture
