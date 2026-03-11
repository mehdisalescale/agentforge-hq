# Epic 1 — Foundation & Personas (v0.7.0)

> **Purpose**: Concrete story-level instructions and result slots for parallel agents.
> All agents working on Epic 1 should:
> - Read `CLAUDE.md`, `NORTH_STAR.md`, `docs/EXPANSION_PLAN.md`, and `docs/AGENTFORGE_AGENT_ROLES.md`.
> - Then read the specific story section below before coding.
> - After completing a story, add a short **Agent Result** under that story.

---

## Story 1.1.1 – Create and Manage Companies

**Role**: Org & Governance Agent  
**Goal**: Make Claude Forge multi-company aware with basic company CRUD and budget fields.

**Context**

- See `docs/EXPANSION_PLAN.md` (Wave 3: Org Charts & Governance).
- DB migration `0011_org_charts.sql` already exists and defines `companies`.

**What to implement**

- **Backend**
  - Ensure `migrations/0011_org_charts.sql` is wired via `crates/forge-db/src/migrations.rs` (already done).
  - Implement `CompanyRepo` in `crates/forge-db/src/repos/companies.rs` with:
    - `create(NewCompany) -> Company`
    - `list() -> Vec<Company>`
    - `get(id: &str) -> Company`
    - Idempotent behavior for DB setup (no duplicate schema changes).
  - Wire `CompanyRepo` into:
    - `crates/forge-db/src/repos/mod.rs` and `lib.rs` (re-export).
    - `AppState` in `crates/forge-api/src/state.rs` (add `company_repo` field and constructor wiring).
- **API**
  - Add Axum handlers in a new `crates/forge-api/src/routes/org.rs` module:
    - `POST /api/v1/companies` → create company.
    - `GET /api/v1/companies` → list companies.
  - Merge the new routes in `crates/forge-api/src/routes/mod.rs`.
- **Tests**
  - `forge-db`: in-memory DB + migrations → CompanyRepo round-trip (create/list/get).
  - `forge-api`: integration tests for `POST/GET /api/v1/companies` using the existing test harness.

**Verify**

- `cargo test --workspace`
- `cargo check --workspace`
- `cargo clippy --workspace -D warnings`

**Agent Result (to fill in when done)**

- Implementing agent: Org & Governance Agent (Cursor AI)  
- Summary of changes (files, key types/routes added): Implemented `CompanyRepo` in `crates/forge-db/src/repos/companies.rs` and re-exported it via `forge-db::lib.rs`; wired `CompanyRepo` into `AppState` in `crates/forge-api/src/state.rs` and into `forge-app` in `crates/forge-app/src/main.rs`; added `crates/forge-api/src/routes/org.rs` with `POST/GET /api/v1/companies` and merged it in `crates/forge-api/src/routes/mod.rs`; ensured `migrations/0011_org_charts.sql` is included in `crates/forge-db/src/migrations.rs`.  
- Tests executed and their status: `cargo check --workspace` (clean), `cargo clippy --workspace -- -D warnings` (clean), `cargo test --workspace` (all existing tests pass except a pre-existing failure in `forge-persona::parser::tests::parses_basic_markdown`, which was not modified in this work).  
- Any deviations from this spec or follow-ups needed: No functional deviations; HTTP integration tests for `/api/v1/companies` can be expanded later to cover additional validation/edge cases once company-level budgets and governance features are fleshed out.

---

## Story 1.1.2 – Departments & Org Chart

**Role**: Org & Governance Agent  
**Goal**: Model departments and reporting chains and expose a basic org-chart API + UI.

**Context**

- `0011_org_charts.sql` already defines `departments` and `org_positions`.
- `crates/forge-org` crate exists; use it for Org domain types and chart assembly.

**What to implement**

- **Backend**
  - Implement `DepartmentRepo` and `OrgPositionRepo` in `crates/forge-db/src/repos/` with:
    - `DepartmentRepo::create(NewDepartment)`, `list_by_company(company_id)`.
    - `OrgPositionRepo::create(NewOrgPosition)`, `list_by_company(company_id)`.
  - Re-export these repos via `forge-db` `lib.rs` and wire into `AppState`.
  - In `crates/forge-org`, implement a pure `build_org_chart` function that:
    - Accepts `Company`, `Vec<Department>`, and `Vec<OrgPosition>`.
    - Returns a `CompanyOrgChart` tree (roots are positions with `reports_to IS NULL`).
- **API**
  - In `routes/org.rs`, add:
    - `POST /api/v1/departments` → create department.
    - `POST /api/v1/org-positions` → create org position.
    - `GET /api/v1/org-chart` → return `CompanyOrgChart` for a given `company_id` (or first company if none specified).
- **Frontend**
  - Add `frontend/src/routes/companies/+page.svelte`:
    - Use `listCompanies()` to show a simple list and a minimal “create company” form.
  - Add `frontend/src/routes/org-chart/+page.svelte`:
    - Company selector (dropdown built from `listCompanies()`).
    - Fetch `GET /api/v1/org-chart?company_id=...`.
    - Render a basic tree view of positions (nested list is enough for now).
  - Extend `frontend/src/lib/api.ts` with:
    - Types and helpers for `Company`, `Department`, `OrgPosition`, `CompanyOrgChart`.

**Verify**

- `cargo test/check/clippy` all green.
- `cd frontend && pnpm build` succeeds.

**Agent Result (to fill in when done)**

- Implementing agent: Org & Governance Agent (Cursor AI)  
- Summary of changes (files, key types/routes/pages added): Implemented `DepartmentRepo` and `OrgPositionRepo` in `crates/forge-db/src/repos/{departments,org_positions}.rs` and re-exported them via `forge-db::lib.rs`, wiring them into `AppState` (`crates/forge-api/src/state.rs`) and `forge-app` (`crates/forge-app/src/main.rs`); added `forge-org::service::build_org_chart` to assemble `CompanyOrgChart` from `Company`, `Department`, and `OrgPosition`; extended `crates/forge-api/src/routes/org.rs` with `POST /api/v1/departments`, `POST /api/v1/org-positions`, and `GET /api/v1/org-chart`; and added Svelte pages `frontend/src/routes/companies/+page.svelte` and `frontend/src/routes/org-chart/+page.svelte` plus corresponding types and helpers in `frontend/src/lib/api.ts` and navigation links in `frontend/src/routes/+layout.svelte`.  
- Tests executed and their status: `cargo check --workspace` (clean), `cargo clippy --workspace -- -D warnings` (clean), `cargo test --workspace` (all existing tests pass except the pre-existing `forge-persona::parser::tests::parses_basic_markdown` failure, which is out of scope for this story), and `cd frontend && pnpm build` (succeeds).  
- Any deviations from this spec or follow-ups needed: Org chart tree rendering in `/org-chart` is implemented as a nested list up to a few levels deep rather than a fully generic infinite-depth component; this is sufficient for current usage but can be generalized later if deeper hierarchies are needed. Additional HTTP-level tests specifically for `/api/v1/departments`, `/api/v1/org-positions`, and `/api/v1/org-chart` can be added in a follow-up story to complement the existing repo and domain tests.

---

## Story 1.2.1 – Import Persona Catalog

**Role**: Persona & Methodology Agent  
**Goal**: Expand from 10 presets to 100+ personas via a structured importer.

**Context**

- See `docs/EXPANSION_PLAN.md` (Wave 1: Persona Catalog).
- `migrations/0009_personas.sql` exists and defines `persona_divisions` and `personas`.
- Upstream markdown source: local `agency-agents/` repo (curated subset copied into `forge-project/personas/`).

**What to implement**

- **Backend**
  - Ensure `0009_personas.sql` is wired in `forge-db` migrator (already done).
  - Implement `PersonaRepo` in `crates/forge-db/src/repos/personas.rs` with:
    - `upsert_divisions(&[PersonaDivision])`.
    - `upsert_personas(&[Persona])`.
    - `list(division_slug: Option<&str>, search: Option<&str>)`.
    - `get(id: &PersonaId)`.
  - Create `crates/forge-persona` with:
    - `model.rs` (`Persona`, `PersonaDivision`, IDs).
    - `parser.rs` (`PersonaParser` that reads markdown under `FORGE_PERSONAS_DIR`).
    - `catalog.rs` (in-memory indexing/filtering/search).
    - `mapper.rs` (`PersonaMapper` that converts a Persona into `forge_agent::NewAgent`).
- **Importer**
  - Implement a one-shot importer function (in `forge-persona` or `forge-app`) that:
    - Walks `FORGE_PERSONAS_DIR` (default `./personas`).
    - Parses personas.
    - Builds divisions/personas and calls `PersonaRepo` upserts.
    - Is **idempotent** (re-running does not duplicate rows).

**Verify**

- `forge-persona` unit tests for parsing and mapping.
- `forge-db` in-memory tests for `PersonaRepo` upsert and list/get behavior.

**Agent Result (to fill in when done)**

- Implementing agent:  
- Summary of changes (files, key types/modules added):  
- Tests executed and their status:  
- Any deviations from this spec or follow-ups needed:  

---

## Story 1.2.2 – Browse and Hire Personas

**Role**: Persona & Methodology Agent  
**Goal**: Expose personas via REST + Svelte UI and support “Hire from Persona”.

**Context**

- Builds on Story 1.2.1 (`forge-persona` + `PersonaRepo`).

**What to implement**

- **API**
  - New Axum routes in `crates/forge-api/src/routes/personas.rs`:
    - `GET /api/v1/personas` with `division?` and `q?` filters → returns persona summaries.
    - `GET /api/v1/personas/:id` → returns full persona details.
    - `POST /api/v1/personas/import` → runs the importer and returns a summary.
    - `POST /api/v1/personas/:id/hire` → maps persona to `NewAgent` via `PersonaMapper`, creates agent via `AgentRepo`, and returns the created agent.
  - Merge routes in `routes/mod.rs` and wire `PersonaRepo` + mapper into `AppState`.
- **Frontend**
  - Extend `frontend/src/lib/api.ts` with:
    - `listPersonas`, `getPersona`, `importPersonas`, `hirePersona`.
    - `PersonaSummary` and `PersonaDetail` types.
  - New Svelte page `frontend/src/routes/personas/+page.svelte`:
    - Division sidebar filters (derived from data).
    - Search input.
    - Card grid of persona summaries.
    - Detail modal showing personality, deliverables, metrics, workflow.
    - “Hire” button that calls `hirePersona` then navigates to `/agents`.

**Verify**

- `cargo test/check/clippy` all green.
- `cd frontend && pnpm build` succeeds.

**Agent Result (to fill in when done)**

- Implementing agent:  
- Summary of changes (files, key APIs/pages added):  
- Tests executed and their status:  
- Any deviations from this spec or follow-ups needed:  

---

## Story 1.3.x – Goals, Approvals, Methodology, Security (Preview)

> Detailed task breakdown for Stories 1.3.1–1.3.4 will be added once Stories 1.1.x and 1.2.x are implemented and stable.
> For now, follow `docs/EXPANSION_PLAN.md` for high-level intent and coordinate via `AGENTFORGE_AGENT_ROLES.md` before starting work.

