# Execution Plan: Unit of Work (AR-1) + Hexagonal Backends (E3)

## Overview

Two independent workstreams that can be executed **sequentially or overlapped**:

| Workstream | Agents | Days | Risk |
|------------|--------|------|------|
| AR-1: Unit of Work | UW-A → UW-B + UW-C → UW-D | 4–5 | Medium (touches every route file) |
| E3: Hexagonal Backends | E3-A → E3-B → E3-C → E3-D | 5–6 | Low (foundation already coded) |

**Recommended order:** AR-1 first, then E3. Reason: E3 adds `backend_registry` to AppState — if UoW runs first, E3 just adds it to the slimmed AppState. If E3 runs first, UoW has one more field to fold in.

---

## AR-1: Unit of Work — Schedule

```
Day 1:     UW-A (foundation: UnitOfWork struct, slim AppState)
Day 2–3:   UW-B + UW-C in parallel (routes + MCP server update)
Day 4:     UW-D (tests + verification)
```

### Conflict Matrix

| Agent | state.rs | routes/* | middleware.rs | main.rs | mcp main.rs | unit_of_work.rs |
|-------|----------|----------|---------------|---------|-------------|-----------------|
| UW-A  | **W**    |          |               | **W**   |             | **W** (new)     |
| UW-B  |          | **W**    | **W**         |         |             |                 |
| UW-C  |          |          |               | **W**   | **W**       |                 |
| UW-D  |          |          |               |         |             | **W** (tests)   |

**Note:** UW-B and UW-C both depend on UW-A but don't overlap with each other → run in parallel.
**Note:** UW-A and UW-C both touch main.rs. UW-A does the initial rewrite, UW-C verifies/adjusts. Run sequentially.

---

## E3: Hexagonal Backends — Schedule

```
Day 5:     E3-A (DB migration + agent backend_type)
Day 6–7:   E3-B (wire BackendRegistry into SpawnMiddleware)
Day 7–8:   E3-C (health endpoint + MCP tools) — can overlap with late E3-B
Day 9:     E3-D (event normalization + verification)
```

### Conflict Matrix

| Agent | agents model | agents repo | middleware.rs | state.rs | run.rs | main.rs | routes/* | mcp main.rs | frontend |
|-------|-------------|-------------|---------------|----------|--------|---------|----------|-------------|----------|
| E3-A  | **W**       | **W**       |               |          |        |         |          |             | **W**    |
| E3-B  |             |             | **W**         | **W**    | **W**  | **W**   |          |             |          |
| E3-C  |             |             |               |          |        |         | **W**    | **W**       | **W**    |
| E3-D  |             |             | **W**         |          |        |         |          |             |          |

**Note:** E3-A and E3-B don't overlap → E3-B can start as soon as E3-A finishes.
**Note:** E3-C and E3-B overlap only if E3-C starts before E3-B finishes `state.rs`. Safe to overlap days 7–8.
**Note:** E3-C and E3-A both touch frontend but different pages (agents vs backends). Safe.

---

## Combined Schedule (9 days)

| Days | Agents | Focus |
|------|--------|-------|
| 1    | UW-A   | Create UnitOfWork, slim AppState |
| 2–3  | UW-B + UW-C (parallel) | Update all routes + MCP server |
| 4    | UW-D   | Tests, verification |
| 5    | E3-A   | Agent backend_type field |
| 6–7  | E3-B   | Wire BackendRegistry into SpawnMiddleware |
| 7–8  | E3-C   | Health endpoint + MCP tools |
| 9    | E3-D   | Event normalization + final verification |

---

## Pre-existing Issue to Fix First

Before starting either workstream, fix the trivial type error:
- `frontend/src/routes/workflows/+page.svelte` line 436 — `{{input}}` in textarea placeholder

Takes 5 minutes, avoids carrying the known issue forward.
