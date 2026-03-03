# Phase 1 Design Notes

> **When implementing forge-process, MCP, and skills:** use the references below so Phase 1 (Agent Engine) aligns with proven patterns and the 26-feb enhancement map.

---

## forge-process (process spawn, stream-json)

- **Use the `claude` CLI** (official Claude Code CLI) for behavior and contract: how processes are spawned, how stream-json is produced and consumed, and how to map output to `ForgeEvent` (e.g. `ProcessOutput`, `ProcessCompleted`).
- **Use claude-flow** as a **design reference** (not code reuse): its ADRs and plugin/MCP design inform coordination and process lifecycle. See **08-reference/TREND_26FEB_ENHANCEMENT_MAP.md** → claude-flow (15-agent mesh, MCP-first API, event sourcing).
- Keep **forge-process** stubs (`ProcessHandle`, `StreamJsonEvent`) and replace with real process spawning and stream-json parsing when implementing Phase 1.

---

## MCP / server behavior

- **Use claude-flow** ADRs and **plugin/MCP design** as reference for MCP server shape, tool/resource exposure, and lifecycle.
- Full map and priorities: **08-reference/TREND_26FEB_ENHANCEMENT_MAP.md** (claude-flow, Scrapling for MCP tools, etc.).

---

## Skills and workflows

- **Agent-Skills-for-Context-Engineering** — use for **skill content** and context-optimization reference (evaluation, LLM-as-judge, multi-agent patterns).
- **superpowers** — use for **spec → design → plan → subagent TDD** workflow and composable skills; supports quality gates and workflow templates.
- Ingest as skill catalog content and reference; implement engine in Rust.

---

## Full map

**08-reference/TREND_26FEB_ENHANCEMENT_MAP.md** — full 26-feb enhancement map: repos, impact tiers, absorption types, and suggested order (claude-flow + Agent-Skills + superpowers first; then ruvector, Scrapling in Phase 2).
