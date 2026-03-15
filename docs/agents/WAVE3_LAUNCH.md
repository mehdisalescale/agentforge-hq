# Wave 3 Launch Instructions (Finish First)

## Context
Many "shell" pages actually have working UI + backend already. The real gap is governance wiring and visibility. This wave makes governance real and cleans up the product.

## Agents A-D can run in parallel — they touch separate files.

### Recommended: All 4 in parallel
```
Phase 1 (parallel): W3-A + W3-B + W3-C + W3-D
```

**IMPORTANT**: Agent W3-B is the only one touching middleware.rs and run.rs. No other agent may modify those files.

## Tab 1 — Agent W3-A: Frontend Polish (frontend only)
```
Read docs/agents/AGENT_W3A_FRONTEND.md and execute all instructions in it. You are Agent W3-A: Frontend Polish. Start by reading CLAUDE.md and NORTH_STAR.md, then do your work. Commit when done and output your report.
```

## Tab 2 — Agent W3-B: Governance Wiring (backend, middleware)
```
Read docs/agents/AGENT_W3B_GOVERNANCE.md and execute all instructions in it. You are Agent W3-B: Governance Wiring. Start by reading CLAUDE.md and NORTH_STAR.md, then do your work. Commit when done and output your report.
```

## Tab 3 — Agent W3-C: Session Detail (isolated)
```
Read docs/agents/AGENT_W3C_SESSIONS.md and execute all instructions in it. You are Agent W3-C: Session Detail. Start by reading CLAUDE.md and NORTH_STAR.md, then do your work. Commit when done and output your report.
```

## Tab 4 — Agent W3-D: Verify & Fix Existing Pages
```
Read docs/agents/AGENT_W3D_VERIFY.md and execute all instructions in it. You are Agent W3-D: Verify & Fix. Start by reading CLAUDE.md and NORTH_STAR.md, then do your work. Commit when done and output your report.
```

## Conflict Matrix

| Agent | middleware.rs | run.rs | +layout.svelte | +page.svelte (/) | sessions/ | skills/ | analytics/ |
|-------|-------------|--------|----------------|-------------------|-----------|---------|------------|
| W3-A  | —           | —      | MODIFY         | MODIFY            | —         | —       | —          |
| W3-B  | MODIFY      | MODIFY | —              | —                 | —         | —       | —          |
| W3-C  | —           | —      | —              | —                 | MODIFY    | —       | —          |
| W3-D  | —           | —      | —              | —                 | —         | VERIFY  | VERIFY     |

**Zero conflicts. All 4 safe in parallel.**
