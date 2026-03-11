# AgentForge — Parallel Agent Roles & Protocol

> **Purpose**: Shared instructions, prompts, and context for multiple development agents
> working in parallel on the AgentForge expansion (Claude Forge → multi-company AI workforce).
> All agents MUST read `CLAUDE.md`, `NORTH_STAR.md`, `docs/EXPANSION_PLAN.md`,
> and this file before starting work in a new session.

---

## 1. Shared Context (All Agents)

- **Product**: Claude Forge evolving into **AgentForge**, an AI workforce platform on a Rust/Axum backend
  and SvelteKit 5 frontend, shipped as a single binary. Agents, companies, personas, goals, and runs
  are all first-class domain concepts.
- **Source of truth docs** (read-only except when explicitly updating):
  - `CLAUDE.md`: high-level project context, tech stack, and conventions.
  - `NORTH_STAR.md`: current vision, state, and sprint plan for Claude Forge.
  - `docs/EXPANSION_PLAN.md`: “Absorbing 8 Repos” plan — Paperclip, Agency-Agents, Hermes-Agent,
    AstrBot, Superpowers, Claude-Code Plugins, OpenClaw, Open-Claude-Cowork.
  - `MASTER_TASK_LIST.md`: existing sprint tasks for v0.5–0.6 (reference only).
- **New expansion plan** (this session):
  - Treat `docs/EXPANSION_PLAN.md` as the core design for **v0.7+**.
  - Implement new crates and routes as specified there, in waves and epics.
- **Rules**:
  - Keep `cargo check`, `cargo test`, and `cargo clippy` clean before committing.
  - Do not edit frozen planning docs under `archive/` or numbered 00–08 folders.
  - Prefer code + tests over speculative planning, except where this file or the expansion plan
    explicitly asks for documentation.

---

## 2. Development Agent Roles

This section defines four primary development “personas” for parallel work. A single human or AI
may play multiple roles, but at any moment each role should have a focused scope to minimize conflicts.

### 2.1 Org & Governance Agent

**Scope**

- Implement and maintain:
  - `crates/forge-org`: companies, departments, org positions, budget enforcement.
  - `crates/forge-governance`: goals, approvals, mission/goal/task lineage.
  - Migrations for `companies`, `departments`, `org_positions`, `goals`, `approvals`.
  - API routes:
    - `POST /api/v1/companies`, `GET /api/v1/companies`
    - `POST /api/v1/departments`
    - `POST /api/v1/org-positions`, `GET /api/v1/org-chart`
    - `POST /api/v1/goals`, `GET /api/v1/goals/tree`
    - `POST /api/v1/approvals`, `POST /api/v1/approvals/:id/decide`
  - Frontend routes:
    - `/companies`, `/org-chart`, `/goals`, `/approvals`.

**Required reading before edits**

- `CLAUDE.md`
- `NORTH_STAR.md`
- `docs/EXPANSION_PLAN.md` (Wave 3: Org Charts & Governance)

**Working protocol**

- Start each story by writing or updating tests:
  - Rust unit tests for domain types and services.
  - Integration tests for API endpoints (`forge-api`).
  - Svelte/Playwright tests for the new pages where appropriate.
- Ensure all new concepts are reflected in:
  - `docs/EXPANSION_PLAN.md` (if behavior diverges from the plan).
  - Future domain docs as they are added (e.g., glossary).
- Coordinate with the Persona & Methodology Agent when mapping departments to persona divisions
  or when companies/goals need persona-aware behavior.

**Prompt summary for this role**

> “You are the Org & Governance Agent. Your job is to implement and evolve multi-company support,
> org charts, budgets, goals, and approvals in Claude Forge / AgentForge. Work in Rust and Svelte,
> strictly following the expansion plan and existing project conventions. For each task, write tests
> first, keep middleware and repos cohesive, and update docs when public behavior changes.”

---

### 2.2 Persona & Methodology Agent

**Scope**

- Implement and maintain:
  - `crates/forge-persona`: persona parsing/importing/catalog/search/mapping.
  - `migrations/0009_personas.sql`: `personas`, `persona_divisions`.
  - API routes for personas:
    - `GET /api/v1/personas`, `GET /api/v1/personas/:id`
    - `POST /api/v1/personas/import`
    - `POST /api/v1/personas/:id/hire`
  - Frontend route `/personas`:
    - Division filters, search, detail view, “Hire” flow.
  - Wave 2 methodology:
    - Skills imported from Superpowers and Claude-Code Plugins into `skills/`.
    - Task-type detection and skill routing.
    - Security scanning patterns wired into `forge-safety`.

**Required reading before edits**

- `docs/EXPANSION_PLAN.md` (Wave 1: Persona Catalog, Wave 2: Dev Methodology).
- `docs/RESEARCH_FINDINGS_2026_03_05.md` (for patterns that may inform task-type detection).
- Upstream repos (on demand), documented into:
  - `docs/EXTERNAL_REPOS/AGENCY_AGENTS.md` (to be created).
  - `docs/EXTERNAL_REPOS/SUPERPOWERS.md`.
  - `docs/EXTERNAL_REPOS/CLAUDE_CODE_PLUGINS.md`.

**Working protocol**

- Treat upstream persona markdown as **data**, not code:
  - Preserve structure and key semantics (division, personality, workflows).
  - Record any lossy transformations in the corresponding external repo doc.
- For methodology:
  - Start with minimal, robust task-type detection using simple heuristics + tests.
  - Only then consider ML-style classifiers or heavy logic, and document the trade-offs.

**Prompt summary for this role**

> “You are the Persona & Methodology Agent. Your job is to extend Claude Forge into AgentForge with
> a rich persona catalog and best-practice engineering workflows. Parse and index personas from
> Agency-Agents, expose them via Rust APIs and Svelte UI, and later wire Superpowers/Claude-Code
> skills into a task-type → methodology routing system. Always preserve upstream intent and keep
> tests and docs in sync.”

---

### 2.3 Knowledge & Messaging Agent

**Scope**

- Implement and maintain:
  - `crates/forge-knowledge`: KB documents, chunking, FTS5 search, context injection.
  - `crates/forge-messaging`: AstrBot bridge first (sidecar), optional native adapters later.
  - Migrations for `kb_documents`, `kb_chunks`, `kb_chunks_fts`, `messaging_configs`,
    `notification_prefs`.
  - API routes:
    - `/api/v1/knowledge`, `/api/v1/knowledge/upload`, `/api/v1/knowledge/search`.
    - `/api/v1/messaging/configs`, `/api/v1/messaging/test`, platform webhooks.
  - Frontend routes:
    - `/knowledge`: upload/search/browse.
    - `/messaging`: platform connections and notification preferences.

**Required reading before edits**

- `docs/EXPANSION_PLAN.md` (Wave 6: Knowledge Base, Wave 7: Messaging).
- Upstream repo docs:
  - `docs/EXTERNAL_REPOS/ASTRBOT.md` (once created).

**Working protocol**

- Default to SQLite FTS5-based search to preserve the “single binary, zero deps” philosophy.
- Treat AstrBot as a sidecar:
  - All platform-specific integrations handled in AstrBot.
  - Forge focuses on clean HTTP contracts and intent routing.
- Make knowledge injection transparent and explainable:
  - For each run, it should be clear which KB snippets were injected and why.

**Prompt summary for this role**

> “You are the Knowledge & Messaging Agent. Your job is to give AgentForge a first-class knowledge
> base and multi-platform messaging via AstrBot. Implement document ingestion and search in Rust
> with SQLite FTS5, wire knowledge into agent runs via middleware, and expose safe messaging
> endpoints and UI. Keep sidecar interactions simple, explicit, and well-documented.”

---

### 2.4 Runtime Adapter Agent

**Scope**

- Implement and maintain:
  - Backend abstraction in `forge-process` (`ProcessBackend` enum, routing).
  - `crates/forge-adapter-hermes`: Hermes runtime adapter and memory sync.
  - `crates/forge-adapter-openclaw`: OpenClaw webhook adapter.
  - Migrations for `agent.backend_type` and any backend-specific metadata.
  - API routes:
    - `PUT /api/v1/agents/:id/backend` to switch backends.
    - `GET /api/v1/backends` for backend availability/health.
  - Frontend:
    - Agent create/edit UI for backend selection and backend-specific config.
    - Backend badges in session views.

**Required reading before edits**

- `docs/EXPANSION_PLAN.md` (Wave 4: Hermes Adapter, Wave 5: OpenClaw Adapter).
- Upstream repo docs:
  - `docs/EXTERNAL_REPOS/HERMES_AGENT.md`.
  - `docs/EXTERNAL_REPOS/OPENCLAW.md`.

**Working protocol**

- Always design adapters so the core Forge process logic remains backend-agnostic.
- Favor clear, schema-checked JSON contracts between Forge and sidecars.
- Ensure resilience:
  - Health endpoints, retries with backoff, and clear error messages when a backend is down.

**Prompt summary for this role**

> “You are the Runtime Adapter Agent. Your job is to make AgentForge multi-backend by integrating
> Hermes and OpenClaw behind a clean `ProcessBackend` abstraction. Implement robust adapters that
> translate Forge tasks into backend-specific calls and map results back into Forge events and
> sessions. Prioritize clarity, resiliency, and testability.”

---

## 3. Multi-Agent Workflow & Gates

- **Before starting work in a session**:
  - Read `NORTH_STAR.md`, `docs/EXPANSION_PLAN.md`, and this file.
  - Choose a role and corresponding Epic/Sprint story.
- **During work**:
  - Work in the smallest possible slices (one story or sub-story at a time).
  - For each slice:
    - Add or update tests for the desired behavior.
    - Implement Rust + Svelte changes until tests pass.
    - Run `cargo test --workspace && cargo clippy --workspace` and `pnpm test` (if applicable).
  - Avoid editing files outside your role’s scope unless strictly necessary; if you must, note it
    in commit messages and, when appropriate, docs.
- **Gates**:
  - No story is “Done” until:
    - Tests match acceptance criteria in `docs/EXPANSION_PLAN.md` / Epic docs.
    - Lints are clean.
    - Relevant docs are updated.

This file is intentionally verbose to make multi-agent collaboration predictable. Treat it as the
live playbook for how different agents work together on AgentForge. Update it when roles or scopes
change, keeping the structure but refining the details as the project evolves.

