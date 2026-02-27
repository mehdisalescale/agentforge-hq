# What to Do Next — Deep Repo Recommendation

> Generated from a full pass over NORTH_STAR, AUDIT_REPORT, SESSION_LOG, planning docs, and code.
> **Single source for "what should we do next."**

---

## 1. Sync NORTH_STAR and docs (quick wins)

**NORTH_STAR.md is out of date** after Batch 2:

- **"What's Missing for v0.2.0"** still lists: circuit breaker, rate limiter, cost tracking, markdown rendering, GitHub Release binaries. **All of these are now implemented.** Update to:
  - **Done for v0.2.0:** Circuit breaker, rate limiter, cost tracking, markdown in stream, tool-use collapsible panels, GitHub Release workflow, configurable host/port, E2E smoke script, README.
  - **Still missing for v0.2.0:** MCP server (stdio + ~10 tools); optionally "ship v0.2.0" = tag + verify release.
- **Phase B table:** Mark "Circuit breaker", "Rate limiter" as Done.

**README.md:** Add rate-limit env vars so operators can tune:

- `FORGE_RATE_LIMIT_MAX` (default 10)
- `FORGE_RATE_LIMIT_REFILL_MS` (default 1000)

**SESSION_LOG.md:** Append one entry for the Batch 2 handoff (HANDOFF_BATCH_2: TASK_11–TASK_20, all waves, SafetyState refactor).

---

## 2. Release workflow: one release, three binaries

**Current behavior:** `.github/workflows/release.yml` runs a **matrix** (macos-arm64, macos-x64, linux-x64). Each job calls `softprops/action-gh-release@v2` with a single `files:` entry. That can create/update the same release three times (race) or fail on duplicate tag.

**Recommendation:** Use a **single release job** that depends on all build jobs and uploads all artifacts:

- **Option A:** One job that builds all three targets sequentially and passes all three binaries to `action-gh-release` (simpler, slower).
- **Option B:** Keep matrix for build; add a second job `release` with `needs: [build]` that uses `actions/upload-artifact` in the build job and `actions/download-artifact` in the release job, then uploads all to one release. (Requires storing artifacts per matrix cell and downloading them in the release job.)

Implement Option A or B so one tag produces **one GitHub Release** with `forge-macos-arm64`, `forge-macos-x64`, `forge-linux-x64`.

---

## 3. Fix known code issues (AUDIT_REPORT / AUDIT_REMEDIATION)

From `docs/AUDIT_REPORT.md` and `docs/planning/AUDIT_REMEDIATION.md` section 6 (Rust + workspace):

| Priority | Item | Where | Action |
|----------|------|--------|--------|
| **P1** | rusqlite FTS5 | `Cargo.toml` | rusqlite 0.32 has no separate `fts5` feature; bundled SQLite already includes FTS5. Tests (e.g. `fts5_tables_exist`) pass. No change needed. |
| **P2** | BatchWriter: event timestamp | `forge-db/src/batch_writer.rs` | Persist the event’s embedded timestamp instead of `Utc::now()` at flush. |
| **P2** | BatchWriter: transaction | `forge-db/src/batch_writer.rs` | Use `transaction()` instead of `unchecked_transaction()` unless there’s a documented reason. |
| **P2** | Preset serialization | `forge-db/src/repos/agents.rs` | Use stable format (e.g. serde or fixed string), not `format!("{:?}", p)`. |
| **P2** | UUID/timestamp parse errors | `forge-db/src/repos/agents.rs` | Return a proper rusqlite/ForgeError variant instead of `InvalidParameterName`. |
| **P3** | Agent name validation | `forge-agent/src/validation.rs` | Add character-set rule (e.g. alphanumeric, hyphen, underscore) per plan. |
| **P3** | Re-export `validate_update_agent` | `forge-agent/src/lib.rs` | Re-export if callers need it. |

Do P1 first (FTS5 is used by migration). Then P2/P3 in any order.

---

## 4. MCP server (Phase B — main feature gap)

**NORTH_STAR** and Phase B say: MCP server (stdio transport + ~10 tools) is the next big deliverable.

**Current state:** `crates/forge-mcp` is a **stub** (McpRequest, McpResponse, McpTool, McpResource; no JSON-RPC, no transport).

**Suggested next steps:**

1. **Design:** Pick stdio transport and a minimal JSON-RPC 2.0 flow (read stdin, write stdout). Define the initial tool set (e.g. `agent_create`, `agent_run`, `session_list`, `session_export`, …) and their params/results.
2. **Implement:** In `forge-mcp` (or a new binary crate that links forge-mcp + forge-api/forge-db): run a loop that reads requests, dispatches to forge-api/forge-db, returns responses. No need to integrate into the HTTP server first; a separate `forge-mcp` binary that speaks stdio is enough for v0.2.0.
3. **Integrate later:** Optional: HTTP server exposes an MCP endpoint or spawns the MCP binary per connection.

Reference: NORTH_STAR and NEXT_PHASE_AGENT_PROMPTS mention claude-flow, deer-flow for MCP/plugin design.

---

## 5. Optional: budget enforcement (cost tracking stretch)

TASK_20 added **cost tracking** (session cost_usd from stream, DB, API, UI). The task card listed **optional stretch:** FORGE_BUDGET_WARN / FORGE_BUDGET_LIMIT and BudgetWarning / BudgetExceeded events.

If you want budget enforcement next:

- Read `FORGE_BUDGET_WARN` and `FORGE_BUDGET_LIMIT` in forge-app (or forge-safety).
- In the path that updates session cost (e.g. after `session_repo.update_cost`), compare cumulative cost (e.g. per agent or global) to the limit; emit an event or block further runs when over.
- Document the env vars in README.

---

## 6. What not to do (per NORTH_STAR)

- **Don’t** expand scope to WASM plugins, multi-CLI, 1,500 skills, Kanban, etc., until users ask.
- **Don’t** spend time rewriting frozen planning docs (00–08); only NORTH_STAR, SESSION_LOG, and code-facing docs (README, handoffs) need to stay current.
- **Don’t** block v0.2.0 on MCP if you want to ship first: you can tag v0.2.0 as “current without MCP” and do MCP in v0.3.0.

---

## Suggested order of execution

1. **Today:** Update NORTH_STAR (“What’s Missing”), Phase B table, README (rate limit vars), SESSION_LOG (Batch 2 entry). Optional: fix rusqlite `fts5` (P1).
2. **Before tagging v0.2.0:** Fix release workflow so one tag → one release with three binaries (Section 2).
3. **Next sprint:** P2/P3 code fixes (Section 3); then either MCP server (Section 4) or budget enforcement (Section 5), depending on product priority.
4. **When ready to ship:** Tag `v0.2.0`, push, run E2E smoke test against the release binary, and get 5 people to try it (Phase A).

---

## File map (where to edit)

| Goal | Files |
|------|--------|
| NORTH_STAR sync | `NORTH_STAR.md` |
| README env table | `README.md` |
| SESSION_LOG entry | `docs/SESSION_LOG.md` |
| Release workflow | `.github/workflows/release.yml` |
| rusqlite fts5 | `Cargo.toml` (workspace.dependencies) |
| BatchWriter / agents / validation | `crates/forge-db/src/`, `crates/forge-agent/src/` |
| MCP server | `crates/forge-mcp/` (and possibly new binary) |
