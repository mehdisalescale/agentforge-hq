# Remaining App Plan

> Single roadmap for all work left after v0.2.0. Order is suggested; dependencies noted.

---

## 0. Run downloaded binary (one-time fix)

**Problem:** After downloading a release binary (e.g. `forge-macos-arm64`) from GitHub in a browser, the file has no execute bit → `zsh: permission denied`.

**Fix:** `chmod +x forge-macos-arm64` then `./forge-macos-arm64`.

README Quick start updated so new users see this. No workflow change (browser downloads don’t preserve execute bit).

---

## 1. Phase A wrap-up (ship v0.2.0)

| Task | Effort | Notes |
|------|--------|--------|
| E2E smoke against release binary | 1 session | Run `scripts/e2e-smoke.sh` against a downloaded binary (or `gh release download` then chmod +x). |
| Get 5 people to try it | Ongoing | Real users, real feedback; no code. |
| Fix user-reported issues | As needed | Whatever breaks from Phase A. |

**Outcome:** v0.2.0 validated; input for Phase C and MCP priority.

---

## 2. P2/P3 code fixes (quality, no new features)

From `docs/WHAT_TO_DO_NEXT.md` Section 3 and AUDIT_REPORT.

### P2 — forge-db

| Item | File | Action |
|------|------|--------|
| BatchWriter: event timestamp | `forge-db/src/batch_writer.rs` | Persist event’s embedded timestamp instead of `Utc::now()` at flush. |
| BatchWriter: transaction | `forge-db/src/batch_writer.rs` | Use `transaction()` instead of `unchecked_transaction()` unless there’s a documented reason. |
| Preset serialization | `forge-db/src/repos/agents.rs` | Stable format (e.g. serde or fixed string), not `format!("{:?}", p)`. |
| UUID/timestamp parse errors | `forge-db/src/repos/agents.rs` | Return proper rusqlite/ForgeError variant, not `InvalidParameterName`. |

### P3 — forge-agent

| Item | File | Action |
|------|------|--------|
| Agent name validation | `forge-agent/src/validation.rs` | Character-set rule (e.g. alphanumeric, hyphen, underscore) per plan. |
| Re-export | `forge-agent/src/lib.rs` | Re-export `validate_update_agent` if callers need it. |

**Order:** P2 first (data correctness), then P3. One commit per item or one per priority.

---

## 3. MCP server (Phase B — main feature)

**Goal:** Stdio MCP server with ~10 tools so IDEs/CLIs can drive Forge (agent_create, agent_run, session_list, etc.).

### 3.1 Design

- **Transport:** stdio only (read stdin, write stdout).
- **Protocol:** JSON-RPC 2.0 (or MCP spec if already JSON-RPC-aligned).
- **Initial tools (candidates):**  
  `agent_list`, `agent_create`, `agent_get`, `agent_update`, `agent_delete`,  
  `agent_run` (or `run_create`), `session_list`, `session_get`, `session_export`,  
  plus one or two more (e.g. `run_status`, `run_cancel`) to reach ~10.
- **Params/results:** Define JSON schema per tool; mirror API types where possible.

**Deliverable:** Short design doc (e.g. `docs/MCP_DESIGN.md`) with tool list, request/response shapes, and error handling.

### 3.2 Implement

- **Location:** `crates/forge-mcp` (extend stub) + optional small binary crate (e.g. `forge-mcp-bin`) that links forge-mcp, forge-db, forge-agent; no HTTP server dependency for v0.2.0.
- **Loop:** Read JSON-RPC from stdin → dispatch by method to forge-db/forge-agent (and later forge-api for run) → write JSON-RPC response to stdout.
- **Dependencies:** Minimal (serde_json, tokio for async if needed; or sync stdin/stdout for simplicity).

**Deliverable:** `forge-mcp` (or `forge-mcp-bin`) binary that responds to a fixed set of tools over stdio; manual or script test.

### 3.3 Integrate later (optional)

- HTTP server exposes MCP endpoint or spawns MCP binary per connection (post–v0.2.0).

**Outcome:** MCP is the one big remaining feature; P2/P3 can be done before or in parallel (different files).

---

## 4. Optional: budget enforcement

**Scope:** Stretch on existing cost tracking (TASK_20).

- **Env:** `FORGE_BUDGET_WARN`, `FORGE_BUDGET_LIMIT` (e.g. USD per session or global).
- **Behavior:** After `session_repo.update_cost`, compare to limit; emit BudgetWarning / BudgetExceeded event; optionally block further runs when over limit.
- **Docs:** README env table.

**Effort:** ~1 session. Can slot after P2/P3 or after MCP.

---

## 5. Phase C: one differentiator (deferred)

Pick one after Phase A feedback:

- **Option 1:** Multi-agent swim-lane visualization (observability).
- **Option 2:** Worktree-per-agent isolation (safety).
- **Option 3:** Workflow DAG execution (automation).

No detailed plan here; decision when users have tried v0.2.0.

---

## Suggested execution order

| Phase | Items | Rationale |
|-------|--------|-----------|
| Now | README chmod note (done), run binary with chmod +x | Unblock running the release. |
| Next | Phase A: E2E on release binary, get 5 users | Validate v0.2.0. |
| Then | P2 fixes (BatchWriter, agents) | Data correctness before new features. |
| Then | MCP design + implement | Main remaining feature. |
| Parallel or after | P3 (validation, re-export) | Small, independent. |
| Optional | Budget enforcement | When product priority says so. |
| Later | Phase C one differentiator | After user feedback. |

---

## File map (remaining work)

| Goal | Files / docs |
|------|------------------|
| Run binary | README (Quick start) — updated |
| P2 | `crates/forge-db/src/batch_writer.rs`, `crates/forge-db/src/repos/agents.rs` |
| P3 | `crates/forge-agent/src/validation.rs`, `crates/forge-agent/src/lib.rs` |
| MCP design | `docs/MCP_DESIGN.md` (new) |
| MCP impl | `crates/forge-mcp/`, optional `forge-mcp-bin` |
| Budget | forge-app or forge-safety, README |
