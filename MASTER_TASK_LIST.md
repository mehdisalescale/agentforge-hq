# Claude Forge — Master Task List

> **Updated:** 2026-03-02 (merged from audit task list + enhancement proposal)
> **Source:** `docs/FORGE_AUDIT_2026_03_02.md` (audit), `docs/BORROWED_IDEAS.md` (patterns)
> **Rule:** Complete tasks in order within each sprint. Don't skip sprints.

---

## How To Use This File

1. Pick a task from the current sprint. Read its section.
2. Mark it `[x]` when done.
3. Run `cargo test --workspace && cargo clippy --workspace` after every task.
4. Commit after each task.

---

## Principles

1. **Ship small, ship often.** Three releases, each one usable.
2. **Code over docs.** The 50+ doc era is over. Build features, not plans.
3. **Borrow proven patterns.** Middleware, skills, sub-agents — all verified in DeerFlow (~10K real LOC).
4. **Honest scope.** 3,400 LOC that works beats 30,000 LOC that's half stubs.

---

## Completed Work (for reference)

Everything below was verified working as of 2026-03-02 audit.

<details>
<summary>Phase 0: Foundation — ALL DONE</summary>

- [x] S1: Wire BatchWriter to EventBus
- [x] S2: Increase EventBus capacity (16 → 1024)
- [x] S3: Add `directory` field to frontend Run form
- [x] S4: Show `status` in Sessions UI
- [x] S5: Replace `.expect()` with error propagation
- [x] S6: Add prompt length validation (100KB limit)
- [x] S7: Fix clippy warnings
- [x] S8: Fix CORS for production (FORGE_CORS_ORIGIN env var)

</details>

<details>
<summary>Phase A: Ship v0.1.0 — ALL DONE</summary>

- [x] A1: Embed frontend in binary (rust-embed)
- [x] A2: Graceful shutdown (Ctrl+C → flush BatchWriter)
- [x] A3: TraceLayer request logging
- [x] A4: Configurable host and port (FORGE_HOST, FORGE_PORT)
- [x] A5: E2E smoke test script
- [x] A6: GitHub Actions CI
- [x] A7: GitHub Release workflow
- [x] A8: README.md
- [x] A9: Update NORTH_STAR.md

</details>

<details>
<summary>Phase B: Safety — ALL DONE</summary>

- [x] B1: Auto-update session status on process events
- [x] B2: Markdown rendering in output stream (marked + DOMPurify)
- [x] B3: Tool use/result collapsible panels (OutputBlock array)
- [x] B4: Circuit breaker (3-state FSM, 10 tests)
- [x] B5: Rate limiter (token bucket, configurable via env)
- [x] B6: Cost tracking (parse cost_usd, budget warn/limit, migration 0002)

</details>

---

## SPRINT 1 → v0.2.0 — Fix + MCP + Ship

> **Goal:** Fix known bugs, rewrite MCP server with official SDK, consolidate docs, ship tagged release.
> **Estimated effort:** 3-5 days

### Fixes

#### F1: Dashboard null-safety bug
- [ ] **Done**

**What:** `outputBlocks` can be empty when first WebSocket event arrives. `last.content += content` crashes.

**Where:** `frontend/src/routes/+page.svelte` ~line 80

**How:** Add guard before accessing last element:
```typescript
if (outputBlocks.length === 0) {
    outputBlocks.push({ kind: ev.data.kind, content: '' });
}
const last = outputBlocks[outputBlocks.length - 1];
```

**Verify:** Start a run. First streaming event doesn't crash the page.

---

#### F2: Budget warning logic
- [ ] **Done**

**What:** Current logic: `cost >= warn AND cost < limit` — confusing and may miss edge cases.

**Where:** `crates/forge-api/src/routes/run.rs`

**How:** Simplify to:
```rust
if let Some(limit) = budget_limit {
    if cost >= limit {
        // emit BudgetExceeded, stop
    }
}
if let Some(warn) = budget_warn {
    if cost >= warn {
        // emit BudgetWarning (only once per session)
    }
}
```
Check limit first (takes priority), then warn. Track `warning_emitted` bool to avoid spam.

**Verify:** Set FORGE_BUDGET_WARN=0.01, run an agent. Warning emitted once. Set FORGE_BUDGET_LIMIT=0.01, agent stops.

---

#### F3: Preset serialization
- [ ] **Done**

**What:** `parse_preset()` in agents.rs uses `Debug` format output as fallback — breaks if enum format changes.

**Where:** `crates/forge-db/src/repos/agents.rs`

**How:** Replace Debug fallback with explicit match:
```rust
fn parse_preset(s: &str) -> Option<AgentPreset> {
    match s {
        "CodeWriter" => Some(AgentPreset::CodeWriter),
        "Reviewer" => Some(AgentPreset::Reviewer),
        "Tester" => Some(AgentPreset::Tester),
        "Debugger" => Some(AgentPreset::Debugger),
        "Architect" => Some(AgentPreset::Architect),
        "Documenter" => Some(AgentPreset::Documenter),
        "SecurityAuditor" => Some(AgentPreset::SecurityAuditor),
        "Refactorer" => Some(AgentPreset::Refactorer),
        "Explorer" => Some(AgentPreset::Explorer),
        _ => None,
    }
}
```
Or better: `#[derive(Deserialize)]` and use `serde_json::from_str`.

**Verify:** `cargo test --workspace` passes. Create agent with preset via API, read back, preset field matches.

---

### MCP Rewrite

#### M1: Add rmcp dependency
- [ ] **Done**

**What:** Replace hand-rolled JSON-RPC with official Rust MCP SDK.

**Where:** `Cargo.toml` (workspace), `crates/forge-mcp/Cargo.toml`

**How:**
1. Add `rmcp` to workspace dependencies
2. Research rmcp API: `#[tool]` macro, server builder, stdio transport
3. Decide: rewrite forge-mcp or replace forge-mcp-bin (likely replace forge-mcp-bin, keep forge-mcp as thin types)

**Verify:** `cargo check` passes with rmcp in dependency tree.

---

#### M2: Implement MCP tools with rmcp
- [ ] **Done**

**What:** 10 tools using `#[tool]` macro.

**Tools:**
| Tool | Description |
|------|-------------|
| `forge_agent_create` | Create agent (name, model, preset) |
| `forge_agent_list` | List all agents |
| `forge_agent_get` | Get agent by ID |
| `forge_agent_delete` | Delete agent |
| `forge_run` | Run agent (agent_id, prompt, directory) |
| `forge_session_list` | List sessions (optional agent_id filter) |
| `forge_session_get` | Get session details |
| `forge_session_export` | Export session (session_id, format) |
| `forge_config_get` | Get configuration |
| `forge_health` | Health check + uptime |

**Verify:** Each tool callable via MCP protocol over stdio. Responses match HTTP API format.

---

#### M3: Implement MCP resources
- [ ] **Done**

**What:** 5 resources.

| URI | Description |
|-----|-------------|
| `forge://agents` | List of all agents |
| `forge://sessions` | Recent sessions |
| `forge://config` | Current configuration |
| `forge://health` | System health |
| `forge://skills` | Skill catalog (empty for now) |

**Verify:** `resources/list` returns all 5. `resources/read` returns valid JSON.

---

#### M4: MCP stdio server entry point
- [ ] **Done**

**What:** `forge --mcp` flag starts MCP mode (stdio) instead of HTTP.

**Where:** `crates/forge-app/src/main.rs`

**How:** Add CLI arg parsing (clap or manual). If `--mcp`, run MCP server over stdin/stdout instead of Axum.

**Verify:** `echo '{"jsonrpc":"2.0","method":"initialize","id":1}' | ./forge --mcp` returns valid response.

---

#### M5: MCP protocol compliance tests
- [ ] **Done**

**What:** Tests covering handshake, all tools, all resources, error handling.

**Verify:** `cargo test -p forge-mcp` passes with comprehensive coverage.

---

### Housekeeping

#### D1: Create CLAUDE.md
- [ ] **Done**

**What:** Project context file for every AI/human session.

**Where:** `CLAUDE.md` (project root)

**Verify:** File exists, under 200 lines, covers build commands, architecture, current sprint.

---

#### D2: Doc consolidation
- [ ] **Done**

**What:** Cut doc count from 50+ to ~15.

| Action | Files Affected |
|--------|---------------|
| Merge 35 frozen `00-08/` files into `docs/ORIGINAL_DESIGN_REFERENCE.md` | 35 files → 1 |
| Merge 14 `docs/planning/` files into `docs/PLANNING_ARCHIVE.md` | 14 files → 1 |
| Delete 10 superseded docs already marked in DOC_INDEX | WHAT_TO_DO_NEXT, REMAINING_APP_PLAN, PROPOSAL_2_3_4, STRATEGIC_ASSESSMENT, EXECUTIVE_SUMMARY, AUDIT_REPORT, PRODUCT_JOURNEY, REFERENCE_REPOS, PHASE1_DESIGN_NOTES |
| Update DOC_INDEX.md | Reflect new structure |

**Verify:** `find docs/ -name '*.md' | wc -l` shows ~15 or fewer.

---

### Release

#### R1: Tag and ship v0.2.0
- [ ] **Done**

**What:** Tag, push, verify release binaries work.

**Verify:** Download binary from GitHub Releases, run it, MCP mode works, HTTP mode works.

---

## SPRINT 2 → v0.3.0 — Worktrees + Middleware + Skills

> **Goal:** Git worktree isolation, structured run pipeline, skill system with content.
> **Prerequisite:** Sprint 1 complete.
> **Estimated effort:** 8-12 days

### Worktrees

#### WT1: Git worktree creation
- [ ] **Done**

**What:** Create git worktree per session for agent isolation.

**Where:** New crate `crates/forge-git/` (wraps git2 or shell commands)

**How:**
```rust
pub fn create_worktree(repo_dir: &Path, session_id: &str) -> Result<PathBuf> {
    let worktree_dir = repo_dir.join(".worktrees").join(session_id);
    let branch = format!("forge/{}", session_id);
    Command::new("git")
        .args(["worktree", "add", &worktree_dir.to_string_lossy(), "-b", &branch])
        .current_dir(repo_dir)
        .status()?;
    Ok(worktree_dir)
}
```

**Why worktrees first:** Industry-converged pattern (Claude official `--worktree`, ccswarm). Prerequisite for sub-agent parallelism in Sprint 3. Estimated ~300-500 LOC.

**Verify:** Run creates `.worktrees/{session_id}/` with its own branch. Agent operates in isolation.

---

#### WT2: Worktree cleanup
- [ ] **Done**

**What:** Remove worktree when session is deleted or completed.

**How:** `git worktree remove .worktrees/{session_id}` + `git branch -d forge/{session_id}`

**Verify:** Delete session → worktree directory removed, branch deleted.

---

#### WT3: Worktree cleanup / merge UI
- [ ] **Done**

**What:** Frontend controls for worktree branch status, merge, and delete.

**Where:** `frontend/src/routes/sessions/`

**Verify:** Session detail page shows branch name, merge button works.

---

### Middleware

#### MW1: Middleware trait and chain
- [ ] **Done**

**What:** Define `Middleware` trait, build `MiddlewareChain`, refactor `handle_run`.

**Where:** New file `crates/forge-api/src/middleware.rs`

**How:**
```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process(&self, ctx: &mut RunContext, next: Next<'_>) -> Result<RunResponse, ApiError>;
}

pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
}
```

**Pattern from:** DeerFlow — 8 real middlewares, 1,089 LOC total. See `docs/BORROWED_IDEAS.md` §1.

**Verify:** `handle_run` uses chain instead of inline logic. All existing tests pass.

---

#### MW2: Extract existing logic into middlewares
- [ ] **Done**

**What:** Move rate limit, circuit breaker, spawn, persist, cost into middleware components.

**Chain order:**
1. `RateLimitMiddleware` — token bucket check
2. `CircuitBreakerMiddleware` — failure threshold check
3. `SkillInjectionMiddleware` — (wired in SK3)
4. `SpawnMiddleware` — run claude process
5. `PersistMiddleware` — save events via BatchWriter
6. `CostMiddleware` — track cost, check budget

**Verify:** E2E smoke test passes. Same behavior, better structure.

---

### Skills

#### SK1: Skill file format and loader
- [ ] **Done**

**What:** Define skill Markdown format, write loader, populate skills table at startup.

**Where:** New directory `skills/`, new code in `crates/forge-db/src/repos/skills.rs`

**Skill format:**
```markdown
---
name: deep-research
description: Systematic multi-angle research methodology
tags: [research, analysis]
tools: [Read, WebSearch, Grep]
---

# Deep Research

## When to Use
Use this skill when the user asks for thorough research...

## Methodology
1. Identify key questions
2. Search multiple angles
...
```

**Loader:** At startup, scan `skills/` directory, parse YAML frontmatter + body, upsert into `skills` table.

**Pattern from:** DeerFlow — 15 SKILL.md files + 208-line loader. See `docs/BORROWED_IDEAS.md` §2.

**Verify:** `cargo test`. Start app, `GET /api/v1/skills` returns populated list.

---

#### SK2: Seed 10-15 skills
- [ ] **Done**

**What:** Create Markdown skill files for the most useful workflows.

**Skills to create:**
1. `deep-research` — multi-angle research
2. `code-review` — thorough code review
3. `refactor` — systematic refactoring
4. `test-writer` — comprehensive test generation
5. `debug` — systematic debugging
6. `security-audit` — security vulnerability analysis
7. `document` — documentation generation
8. `architect` — system design and architecture
9. `explore` — codebase exploration and understanding
10. `fix-bug` — bug diagnosis and fix

**Verify:** `skills/` directory has 10+ `.md` files. All load correctly at startup.

---

#### SK3: Skill injection middleware
- [ ] **Done**

**What:** Match user prompt against skills, inject matched skill content into agent system prompt.

**Where:** `SkillInjectionMiddleware` (created in MW2)

**How:** Keyword match: scan prompt for skill tags/names, if match found, append skill body to system prompt before spawn.

**Verify:** Run with prompt "review this code" → Reviewer agent gets `code-review` skill injected.

---

### Testing

#### T1: Integration test: happy path E2E
- [ ] **Done**

**What:** Automated integration test covering the full flow.

**Where:** `tests/`

**How:** Start server → create agent → run → stream → verify session created → verify events persisted.

**Verify:** `cargo test --test integration` passes.

---

## SPRINT 3 → v0.4.0 — Multi-Agent + Memory + Hooks

> **Goal:** Parallel agent spawning, cross-session learning, lifecycle hooks.
> **Prerequisite:** Sprint 2 complete (middleware chain + worktrees).
> **Estimated effort:** 10-14 days

### Sub-Agent Parallelism

#### SA1: Sub-agent events
- [ ] **Done**

**What:** Add sub-agent event types to forge-core.

**Events:** `SubAgentRequested`, `SubAgentStarted`, `SubAgentCompleted`, `SubAgentFailed`

---

#### SA2: Concurrent process manager
- [ ] **Done**

**What:** Extend ProcessRunner to manage N concurrent spawns.

**How:** `ConcurrentRunner` wraps N `ProcessRunner` instances, each in its own worktree. Configurable limit (default 3). Uses `tokio::JoinSet` for concurrent execution.

**Pattern from:** DeerFlow SubagentExecutor — 414 lines, real ThreadPool + timeout. See `docs/BORROWED_IDEAS.md` §4.

---

#### SA3: Coordinator agent
- [ ] **Done**

**What:** A meta-agent that analyzes a task, picks sub-agents, spawns them, aggregates results.

**How:** Coordinator receives prompt → asks Claude to decompose task → spawns sub-agents in parallel → collects results → synthesizes final output.

---

#### SA4: Agent domains
- [ ] **Done**

**What:** Add `domain` field to Agent model. Group 9 presets into domains.

**Domains:**
- `code`: CodeWriter, Refactorer
- `quality`: Reviewer, Tester, Debugger
- `ops`: SecurityAuditor, Architect, Documenter, Explorer

---

#### SA5: WebSocket sub-agent events + multi-agent dashboard UI
- [ ] **Done**

**What:** Frontend shows per-sub-agent progress in real time. Per-agent progress panels, status indicators.

---

### Memory

#### ME1: Memory table and repo
- [ ] **Done**

**What:** Add `memory` table: id, category, content, confidence, created_at, updated_at. MemoryRepo with CRUD.

---

#### ME2: Post-session memory extraction
- [ ] **Done**

**What:** After session completes, send transcript to Claude with extraction prompt. Store facts with confidence scores.

**Pattern from:** DeerFlow MemoryUpdater — 319 lines, LLM-powered, atomic file writes. See `docs/BORROWED_IDEAS.md` §5.

---

#### ME3: Memory injection middleware
- [ ] **Done**

**What:** On new run, query relevant memories, prepend to system prompt.

---

#### ME4: Memory management UI
- [ ] **Done**

**What:** Frontend page to view, edit, delete memory facts.

---

### Hooks

#### HK1: Hook table and runner
- [ ] **Done**

**What:** `hooks` table: id, event_type, timing (pre/post), command, enabled. HookRunner executes shell commands around process spawn.

**Pattern from:** Reference repos — hooks-mastery (13 types), hooks-observability (event interception). See `docs/BORROWED_IDEAS.md` §6.

---

#### HK2: Hook events
- [ ] **Done**

**What:** `HookStarted`, `HookCompleted`, `HookFailed` events in forge-core.

---

#### HK3: Hook management UI
- [ ] **Done**

**What:** Frontend page to create, enable/disable, delete hooks.

---

### Polish

#### SA6: Frontend pagination
- [ ] **Done**

**What:** Add limit/offset to agents, sessions, skills list endpoints and UI.

---

#### P1: Shutdown timeout
- [ ] **Done**

**What:** Add timeout to graceful shutdown (default 10s) to prevent hanging.

---

#### P2: Svelte 5 rune normalization
- [ ] **Done**

**What:** Use `$state` consistently across all pages (Dashboard and Sessions still use `let`).

---

#### P3: Loading states
- [ ] **Done**

**What:** Add loading spinners during API calls on all forms.

---

## PARKED (Not in scope)

| Feature | Why Parked |
|---------|------------|
| WASM plugin runtime | MCP is the extension mechanism |
| Multi-LLM routing | Claude-only by design |
| Consensus protocols | Agents are independent |
| RL/learning layer | No usage data yet |
| Plugin marketplace | Need users first |
| Cron scheduler | Manual for now |
| Dev environment | Post-1.0 if ever |
| Authentication | Add when deploying remotely |
| Audit log + permissions | Post-1.0 |
| Harvester integration | Deferred to post-Sprint 2. See `docs/HARVESTER_INTEGRATION.md` |

---

## Success Criteria

| Release | Key Metric |
|---------|------------|
| v0.2.0 | MCP server passes compliance test, all 3 bugs fixed, docs consolidated |
| v0.3.0 | Agent run creates worktree in isolation, skills injected into prompts, middleware chain active |
| v0.4.0 | 3 sub-agents run in parallel worktrees, memory persists across sessions, hooks fire on events |

---

## Dependencies

```
Sprint 1 (v0.2.0) — bugs + MCP + docs
  └── Sprint 2 (v0.3.0) — worktrees + middleware + skills
        └── Sprint 3 (v0.4.0) — sub-agents use worktrees, memory uses middleware, hooks use events
```

---

## Post-Task Verification

Run after every task:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
# If frontend was touched:
cd frontend && pnpm build && cd ..
```
