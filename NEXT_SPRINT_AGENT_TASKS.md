# Next Sprint — Agent Tasks A–F

> **Purpose:** Plan ahead after Phase 1. Six tracks (A–F) with copy-paste prompts so agents can work in parallel.
> **When to use:** After Phase 1 is done (run endpoint, real spawn, sessions, Agents CRUD, Run/Sessions UI). This sprint = Phase 1 polish + Phase 2 seed.

---

## 1. Context for all agents

**Phase 1 status:** Done. Tracks A–F implemented: spawn + stream-json, runner + EventBus, SessionRepo + session API, POST /api/v1/run (real spawn), Agents CRUD, Run + Sessions UI. See [PHASE1_6_AGENT_SPRINT.md](PHASE1_6_AGENT_SPRINT.md).

**This sprint:** Harden Phase 1 and seed Phase 2 (Workflows + Skills per [SPRINT_PLAN.md](06-planning/SPRINT_PLAN.md) S4–S5, [MILESTONES.md](06-planning/MILESTONES.md) M1→M3).

**Codebase:** `forge-project` (or runnable `claude-forge` if you use it). Work in the repo that has the runnable workspace.

**Definition of done (this sprint):**
- [ ] Run uses session directory when spawning (or optional directory in request).
- [ ] Process/session events are persisted (BatchWriter wired to EventBus in app).
- [ ] E2E smoke test documented or scripted (create agent → run → stream → list session → export).
- [ ] At least one Phase 2 seed in place (workflow or skill stub in DB/API or frontend).
- [ ] `cargo test --workspace` and `cargo clippy --workspace` pass.

---

## 2. Six tracks (assign one per agent)

| Agent | Track | Deliverable | Deps |
|-------|--------|-------------|------|
| **A** | forge-process: config + working dir | SpawnConfig overrides from env or agent; run handler passes session.directory (or request directory) as working_dir to spawn. | forge-process, forge-api |
| **B** | Event persistence | Wire BatchWriter to EventBus in forge-app: subscribe to EventBus, write process/session events to DB so export and history are complete. | forge-db, forge-app, forge-core |
| **C** | forge-db: Phase 2 seed | WorkflowRepo or SkillRepo stub: list/get (and create if needed) using existing workflows/skills tables; minimal API route (e.g. GET /api/v1/workflows or GET /api/v1/skills) returning empty or seeded data. | forge-db, forge-api |
| **D** | forge-api: run + E2E | Run request body: optional `directory`; pass to session create and spawn. Add E2E smoke test script (bash) or step-by-step doc (create agent, POST run, GET sessions, GET export). Optional: readiness probe. | forge-api, docs |
| **E** | Frontend: Run form + Phase 2 seed | Run form: optional directory field (sent in runAgent payload). Or Skills page stub: route /skills, list placeholder or call GET /api/v1/skills when available. | frontend, api.ts |
| **F** | Frontend: Sessions polish + Phase 2 seed | Sessions: show directory and status clearly; improve empty state. Or Workflows page stub: route /workflows, list placeholder or call GET /api/v1/workflows when available. | frontend, api.ts |

**Merge order:** A and D (config + directory) can merge together. B (BatchWriter) next. C (Phase 2 API) then E and F (frontend polish + stubs).

---

## 3. Copy-paste prompts for each agent

Give each agent **section 1 (Context)** above plus **only** the “Your task” block for their letter.

---

### Agent A — forge-process: SpawnConfig + working directory

**Your task**
- In **forge-process**, support **SpawnConfig** overrides: from environment (e.g. `FORGE_CLI_COMMAND`, `FORGE_CLI_ARGS`) or from a future agent config; keep defaults for backward compatibility.
- Ensure **working_dir** is set from the caller: run handler in forge-api should pass the session’s directory (or an optional directory from the run request) into `SpawnConfig.working_dir` when calling `spawn`.
- Add a test that spawn uses the configured working_dir when set.
- Commit when: `cargo test -p forge-process` passes and run handler can pass directory through to spawn.

---

### Agent B — Event persistence (BatchWriter ↔ EventBus)

**Your task**
- In **forge-app**, wire **BatchWriter** to **EventBus**: on startup, subscribe to the EventBus and forward received events to BatchWriter so process and session events are persisted to the `events` table.
- Ensure the existing EventRepo/BatchWriter in forge-db is used; avoid duplicating event logic. If BatchWriter already exists, spawn a task that subscribes to EventBus and calls the writer for each event.
- Verify: after a run, events appear in the DB and session export includes them.
- Commit when: `cargo test --workspace` passes and a manual run produces persisted events.

---

### Agent C — Phase 2 seed: WorkflowRepo or SkillRepo

**Your task**
- In **forge-db**, add **WorkflowRepo** or **SkillRepo** (choose one): implement `list()` and `get(id)` using the existing `workflows` or `skills` table from the schema. Return empty list if no rows.
- In **forge-api**, add one route: **GET /api/v1/workflows** or **GET /api/v1/skills** that returns the list from the repo (JSON array, possibly empty).
- Add a minimal test for the repo and for the HTTP route.
- Commit when: `cargo test --workspace` passes and the new endpoint returns 200 with a JSON array.

---

### Agent D — Run request directory + E2E smoke test

**Your task**
- In **forge-api** run handler, extend **RunRequest** with an optional **directory** field. When creating a new session, use this directory if provided; otherwise keep current default (e.g. "."). Pass the session’s directory into spawn (via SpawnConfig.working_dir) when calling forge_process::spawn.
- Add an **E2E smoke test** either as a bash script or a step-by-step doc (e.g. `docs/E2E_SMOKE_TEST.md`): create an agent (POST /api/v1/agents), run a prompt (POST /api/v1/run), list sessions (GET /api/v1/sessions), export one (GET /api/v1/sessions/:id/export?format=json). Use curl or similar.
- Optional: add a readiness probe route (e.g. GET /api/v1/ready) that returns 200 when DB and event bus are ready.
- Commit when: run accepts optional directory, tests pass, and E2E steps are documented or scripted.

---

### Agent E — Frontend: Run form directory + Skills stub

**Your task**
- In the **Run** form (Dashboard), add an optional **directory** field and include it in the `runAgent` request body when provided.
- **Or** (if directory is already covered by another agent): add a **Skills** page stub: route **/skills**, page that lists skills (call GET /api/v1/skills when available; show placeholder or “No skills yet” otherwise). Add nav link in the layout.
- Ensure `pnpm build` passes.
- Commit when: Run form sends directory and/or Skills page is present and build succeeds.

---

### Agent F — Frontend: Sessions polish + Workflows stub

**Your task**
- In the **Sessions** page, ensure **directory** and **status** are clearly shown for each session (they may already be in the API response; surface them in the UI).
- **Or** (if Sessions are already polished): add a **Workflows** page stub: route **/workflows**, page that lists workflows (call GET /api/v1/workflows when available; show placeholder or “No workflows yet” otherwise). Add nav link in the layout.
- Improve empty state copy if needed (e.g. “Run an agent from the Dashboard to create a session”).
- Commit when: Sessions show directory/status and/or Workflows stub is present; `pnpm build` passes.

---

## 4. Where to find things

| What | Where |
|------|--------|
| Phase 1 sprint (done) | [PHASE1_6_AGENT_SPRINT.md](PHASE1_6_AGENT_SPRINT.md) |
| Sprint plan (S4–S5) | [06-planning/SPRINT_PLAN.md](06-planning/SPRINT_PLAN.md) |
| Milestones (M1–M3) | [06-planning/MILESTONES.md](06-planning/MILESTONES.md) |
| North star | [NORTH_STAR.md](NORTH_STAR.md) |
| Phase 2 design (workflows/skills) | [02-requirements/FEATURE_CATALOG.md](02-requirements/FEATURE_CATALOG.md), [03-architecture/API_DESIGN.md](03-architecture/API_DESIGN.md) |

---

## 5. After this sprint

- **Phase 1** is polished: run uses directory, events persisted, E2E documented.
- **Phase 2** has a seed: one of workflows/skills has a list endpoint and optional frontend stub.
- **Next:** Full workflow engine (S5), skill catalog, or Safety/MCP per ROADMAP.
