# Launch Instructions

Open 4 Zellij tabs. In each tab, cd to the project root and launch claude with the agent's brief.

## Tab 1 — Agent A: Skills Importer (no code changes, just .md files)
```bash
cd /Users/bm/cod/trend/10-march/agentforge-hq
cat docs/agents/AGENT_A_SKILLS.md | claude
```

## Tab 2 — Agent B: Task Detector (forge-process only)
```bash
cd /Users/bm/cod/trend/10-march/agentforge-hq
cat docs/agents/AGENT_B_TASKTYPE.md | claude
```

## Tab 3 — Agent C: Security Scanner (forge-safety only)
```bash
cd /Users/bm/cod/trend/10-march/agentforge-hq
cat docs/agents/AGENT_C_SCANNER.md | claude
```

## Tab 4 — Agent D: E1 Backend Polish (forge-api + forge-db)
```bash
cd /Users/bm/cod/trend/10-march/agentforge-hq
cat docs/agents/AGENT_D_POLISH.md | claude
```

## Conflict Risk

| Agent | Files Modified | Risk |
|-------|---------------|------|
| A | `skills/superpowers/*.md`, `skills/plugins/*.md` | None — new dirs only |
| B | `forge-process/src/task_type.rs` (new), `forge-process/src/lib.rs` (+1 line) | None |
| C | `forge-safety/src/scanner.rs` (new), `forge-safety/src/lib.rs` (+1 line), `forge-safety/Cargo.toml` (+regex) | None |
| D | `forge-db/src/repos/*.rs`, `forge-api/src/routes/*.rs`, `forge-api/src/lib.rs` (tests) | None — different crates from B/C |

**Zero overlap.** All 4 agents can work simultaneously without conflicts.

## After All Agents Report Done

Come back to the coordinator session (this one) and say "agents done" — I will:
1. Check each agent's report
2. Run full `cargo check` and `cargo test`
3. Resolve any issues
4. Merge/commit if needed
5. Update NORTH_STAR.md
