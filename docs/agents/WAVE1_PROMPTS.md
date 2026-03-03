# Wave 1 — Copy-Paste Agent Prompts

> **For the coordinator.** Copy each section into its respective Claude Code session.
> Each prompt is self-contained — the agent reads its own task card from the handoff doc.

---

## Agent A — forge-git Crate

```
You are Agent A in a 5-agent parallel wave. Read these files IN ORDER before doing anything:

1. forge-project/CLAUDE.md (project context)
2. forge-project/docs/agents/AGENT_PROTOCOL.md (coordination rules)
3. forge-project/docs/agents/HANDOFF_SPRINT_2_3.md — find section "TASK_W1A: forge-git Crate"
4. forge-project/docs/agents/WAVE1_STATUS.md (shared coordination)

FIRST ACTION: Update your section in WAVE1_STATUS.md — set status to "in_progress", set Last Update to current time.

YOUR TASK: Create the forge-git crate that wraps git worktree commands.

FILES YOU OWN (create/edit these ONLY):
- crates/forge-git/Cargo.toml (NEW)
- crates/forge-git/src/lib.rs (NEW)
- Cargo.toml (root) — add "crates/forge-git" to workspace members ONLY

DO NOT TOUCH any other files. Follow the task card exactly.

WHEN DONE:
1. Run: cargo test -p forge-git && cargo clippy -p forge-git -- -D warnings
2. Update WAVE1_STATUS.md: set status to "done", fill in files created, tests added, any issues
3. Do NOT commit or push. Coordinator handles that.
```

---

## Agent B — Middleware Trait + Chain

```
You are Agent B in a 5-agent parallel wave. Read these files IN ORDER before doing anything:

1. forge-project/CLAUDE.md (project context)
2. forge-project/docs/agents/AGENT_PROTOCOL.md (coordination rules)
3. forge-project/docs/agents/HANDOFF_SPRINT_2_3.md — find section "TASK_W1B: Middleware Trait + Chain"
4. forge-project/docs/agents/WAVE1_STATUS.md (shared coordination)

FIRST ACTION: Update your section in WAVE1_STATUS.md — set status to "in_progress", set Last Update to current time.

YOUR TASK: Create the middleware trait, chain, and supporting types in a NEW file.

FILES YOU OWN (create/edit these ONLY):
- crates/forge-api/src/middleware.rs (NEW file)

DO NOT TOUCH: routes/run.rs, lib.rs, mod.rs, or any other file. Those are Wave 2 work.

IMPORTANT: Do NOT add `mod middleware;` to lib.rs — Agent F does that in Wave 2.

The file must compile on its own but won't be wired into the module tree yet.

WHEN DONE:
1. Run: cargo check -p forge-api (middleware.rs won't be in module tree, so just check syntax by reading the file for correctness)
2. Update WAVE1_STATUS.md: set status to "done", fill in files created, tests added, any issues
3. Do NOT commit or push. Coordinator handles that.

NOTE: Since the file isn't added to mod.rs yet, cargo test won't find it. That's expected. Verify the code is correct by inspection. The tests will run after Wave 2 wires it in.
```

---

## Agent C — Skill Loader + Seed Files

```
You are Agent C in a 5-agent parallel wave. Read these files IN ORDER before doing anything:

1. forge-project/CLAUDE.md (project context)
2. forge-project/docs/agents/AGENT_PROTOCOL.md (coordination rules)
3. forge-project/docs/agents/HANDOFF_SPRINT_2_3.md — find section "TASK_W1C: Skill Loader + 10 Seed Files"
4. forge-project/docs/agents/WAVE1_STATUS.md (shared coordination)

FIRST ACTION: Update your section in WAVE1_STATUS.md — set status to "in_progress", set Last Update to current time.

YOUR TASK: Create 10 Markdown skill files and add upsert/loader methods to SkillRepo.

FILES YOU OWN (create/edit these ONLY):
- skills/*.md (NEW directory, 10 files)
- crates/forge-db/src/repos/skills.rs (existing file — read it first, then add upsert + load_from_dir methods)

DO NOT TOUCH: routes/skills.rs, mod.rs, lib.rs, or any other file.

READ the existing skills.rs first to understand the current SkillRepo struct, connection patterns, and types. Add your methods following the same patterns.

WHEN DONE:
1. Run: cargo test -p forge-db && cargo clippy -p forge-db -- -D warnings
2. Run: ls skills/*.md | wc -l (should be 10+)
3. Update WAVE1_STATUS.md: set status to "done", fill in files created, tests added, any issues
4. Do NOT commit or push. Coordinator handles that.
```

---

## Agent D — Memory Table + Repo + Routes

```
You are Agent D in a 5-agent parallel wave. Read these files IN ORDER before doing anything:

1. forge-project/CLAUDE.md (project context)
2. forge-project/docs/agents/AGENT_PROTOCOL.md (coordination rules)
3. forge-project/docs/agents/HANDOFF_SPRINT_2_3.md — find section "TASK_W1D: Memory Table + Repo + Routes"
4. forge-project/docs/agents/WAVE1_STATUS.md (shared coordination)
5. crates/forge-db/src/repos/agents.rs (reference for repo patterns — Connection type, error handling, query style)
6. crates/forge-api/src/routes/agents.rs (reference for route patterns — handler signatures, JSON responses, state extraction)

FIRST ACTION: Update your section in WAVE1_STATUS.md — set status to "in_progress", set Last Update to current time.

YOUR TASK: Create the memory data layer — migration SQL, repo, and API routes.

FILES YOU OWN (create/edit these ONLY):
- migrations/0003_add_memory.sql (NEW)
- crates/forge-db/src/repos/memory.rs (NEW)
- crates/forge-api/src/routes/memory.rs (NEW)

DO NOT TOUCH: migrations.rs, repos/mod.rs, lib.rs, routes/mod.rs, state.rs — those are Wave 2 (Agent F).

IMPORTANT: Your repo file should define the struct and impl but will NOT be wired into the module tree until Wave 2. It should compile in isolation (reference the right types from forge-core). The routes file defines the handler functions and a `pub fn routes() -> Router<AppState>` but won't be nested into the app until Wave 2.

WHEN DONE:
1. Verify the files are syntactically correct (they won't compile in the module tree yet — that's expected)
2. Update WAVE1_STATUS.md: set status to "done", fill in files created, tests added, any issues
3. Do NOT commit or push. Coordinator handles that.
```

---

## Agent E — Hook Table + Repo + Routes

```
You are Agent E in a 5-agent parallel wave. Read these files IN ORDER before doing anything:

1. forge-project/CLAUDE.md (project context)
2. forge-project/docs/agents/AGENT_PROTOCOL.md (coordination rules)
3. forge-project/docs/agents/HANDOFF_SPRINT_2_3.md — find section "TASK_W1E: Hook Table + Repo + Routes"
4. forge-project/docs/agents/WAVE1_STATUS.md (shared coordination)
5. crates/forge-db/src/repos/agents.rs (reference for repo patterns)
6. crates/forge-api/src/routes/agents.rs (reference for route patterns)

FIRST ACTION: Update your section in WAVE1_STATUS.md — set status to "in_progress", set Last Update to current time.

YOUR TASK: Create the hook system data layer — migration SQL, repo with HookRunner, and API routes.

FILES YOU OWN (create/edit these ONLY):
- migrations/0004_add_hooks.sql (NEW)
- crates/forge-db/src/repos/hooks.rs (NEW)
- crates/forge-api/src/routes/hooks.rs (NEW)

DO NOT TOUCH: migrations.rs, repos/mod.rs, lib.rs, routes/mod.rs, state.rs, events.rs — those are Wave 2 (Agent F).

IMPORTANT: Your repo file should define the struct and impl but will NOT be wired into the module tree until Wave 2. Follow the same patterns as AgentRepo (read agents.rs for reference). Include a HookRunner struct with async shell execution.

WHEN DONE:
1. Verify the files are syntactically correct (they won't compile in the module tree yet — that's expected)
2. Update WAVE1_STATUS.md: set status to "done", fill in files created, tests added, any issues
3. Do NOT commit or push. Coordinator handles that.
```
