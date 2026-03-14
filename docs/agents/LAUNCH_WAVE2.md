# Wave 2 Launch Instructions

## IMPORTANT: Execution Order

**Agents C2 and D2** can run in parallel — they only touch `forge-process` with separate files.

**Agents A2 and B2** both modify `middleware.rs` and `run.rs` — they MUST run sequentially:
1. Launch A2 first, wait for it to finish
2. Then launch B2 (it will see A2's changes)

**OR** launch all 4 in parallel but expect the coordinator to resolve middleware.rs merge manually.

### Recommended: 3 parallel + 1 sequential

```
Phase 1 (parallel): A2 + C2 + D2
Phase 2 (after A2 done): B2
```

## Tab 1 — Agent A2: Skill Router (runs first, touches middleware)
```
Read docs/agents/AGENT_A2_SKILL_ROUTER.md and execute all instructions in it. You are Agent A2: Skill Router. Start by reading CLAUDE.md and NORTH_STAR.md, then study the middleware chain and TaskTypeDetector. Create the SkillRouter, TaskTypeDetection middleware, and wire it into the chain. Commit when done and output your report.
```

## Tab 2 — Agent C2: ProcessBackend Trait (safe to run in parallel)
```
Read docs/agents/AGENT_C2_BACKEND_TRAIT.md and execute all instructions in it. You are Agent C2: ProcessBackend Trait. Start by reading CLAUDE.md and NORTH_STAR.md, then study spawn.rs. Create the ProcessBackend trait and ClaudeBackend adapter. Commit when done and output your report.
```

## Tab 3 — Agent D2: Code Review Engine (safe to run in parallel)
```
Read docs/agents/AGENT_D2_REVIEW_ENGINE.md and execute all instructions in it. You are Agent D2: Code Review Engine. Start by reading CLAUDE.md and NORTH_STAR.md, then study ConcurrentRunner. Build the review engine with 6 specialist aspects and confidence scoring. Commit when done and output your report.
```

## Tab 4 — Agent B2: SecurityScan Middleware (LAUNCH AFTER A2 FINISHES)
```
Read docs/agents/AGENT_B2_SECURITY_MW.md and execute all instructions in it. You are Agent B2: SecurityScan Middleware. Start by reading CLAUDE.md and NORTH_STAR.md, then study the middleware chain (which now includes TaskTypeDetection from Agent A2). Add SecurityScanMiddleware and wire it in. Commit when done and output your report.
```

## Conflict Matrix

| Agent | forge-process/lib.rs | middleware.rs | run.rs | events.rs |
|-------|---------------------|---------------|--------|-----------|
| A2    | +1 line (skill_router) | +TaskTypeDetectionMW | +1 chain.add | — |
| B2    | — | +SecurityScanMW +extract_code_blocks | +1 chain.add | +2 variants |
| C2    | +2 lines (backend, claude_backend) | — | — | — |
| D2    | +1 line (review) | — | — | — |

**C2 + D2 conflict on `forge-process/src/lib.rs`**: Both add `pub mod` lines, but at different positions. Auto-mergeable.

**A2 + B2 conflict on `middleware.rs` and `run.rs`**: Sequential execution avoids this.
