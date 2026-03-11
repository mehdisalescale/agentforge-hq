# Epic E1: Persona Catalog

> **Expand from 10 hardcoded presets to 100+ rich, importable agent personas with division taxonomy.**
>
> Source repos: agency-agents (100+ personas across 11 divisions)

---

## Business Value

Users currently choose from 10 generic presets (CodeWriter, Reviewer, etc.). This epic lets them browse a rich catalog of 100+ battle-tested personas — each with personality, deliverables, success metrics, and step-by-step workflows. This transforms AgentForge from "spawn a Claude process" to "hire a specialist."

## Acceptance Gate

**The epic is DONE when:**
1. A user can browse 100+ personas organized by 11 divisions in the UI
2. A user can hire a persona (creating a fully configured agent in one click)
3. All persona markdown files parse without errors
4. Persona search by name, division, and tags works
5. All existing 10 presets still work (backward compatibility)
6. 20+ tests covering parsing, import, search, and hire flow

---

## User Stories

### E1-S1: Persona Markdown Parser

**As a** system administrator,
**I want** the system to parse agency-agents markdown files into structured data,
**So that** personas can be stored, searched, and displayed.

**Acceptance Criteria:**

```gherkin
GIVEN a persona markdown file with YAML frontmatter
WHEN the parser processes the file
THEN it extracts: name, description, division, emoji, personality,
     deliverables[], success_metrics[], workflow_steps[], tags[]

GIVEN a persona file with missing required fields (name, description)
WHEN the parser processes the file
THEN it returns a descriptive ParseError with file path and missing field

GIVEN a directory with 100+ persona files
WHEN the bulk importer runs
THEN all valid files are parsed and invalid ones are logged with reasons
AND the import count is returned
```

**Technical Notes:**
- New crate: `forge-persona`
- Struct: `Persona { id, name, division, description, emoji, personality, deliverables, success_metrics, workflow_steps, tags, source_file }`
- Parser: YAML frontmatter + markdown body splitting
- No external dependencies (use existing `serde_yaml` or manual parsing)

**Test Plan:**
- `test_parse_valid_persona_file`
- `test_parse_missing_name_returns_error`
- `test_parse_missing_description_returns_error`
- `test_parse_malformed_yaml_returns_error`
- `test_bulk_import_counts_successes_and_failures`

---

### E1-S2: Persona Database Schema & Repository

**As a** developer,
**I want** personas stored in SQLite with a repository pattern,
**So that** they can be queried, filtered, and linked to agents.

**Acceptance Criteria:**

```gherkin
GIVEN a parsed Persona struct
WHEN I call PersonaRepo.upsert(&persona)
THEN it is stored in the personas table (insert or update by source_file)

GIVEN 100 personas in the database
WHEN I call PersonaRepo.list(division: Some("engineering"))
THEN only engineering division personas are returned

GIVEN a persona ID
WHEN I call PersonaRepo.get(id)
THEN the full persona with all fields is returned

GIVEN personas in the database
WHEN I call PersonaRepo.search("frontend")
THEN personas matching "frontend" in name, description, or tags are returned
```

**Technical Notes:**
- Migration: `0009_personas.sql`
- Tables: `personas`, `persona_divisions`
- FTS5 virtual table on `personas(name, description, tags)` for search
- Repository: `PersonaRepo { conn: Arc<Mutex<Connection>> }`

**Test Plan:**
- `test_upsert_creates_new_persona`
- `test_upsert_updates_existing_persona`
- `test_list_all_returns_all`
- `test_list_by_division_filters_correctly`
- `test_search_matches_name`
- `test_search_matches_tags`
- `test_search_no_results_returns_empty`

---

### E1-S3: Division Taxonomy

**As a** user browsing the catalog,
**I want** personas organized into named divisions with descriptions and counts,
**So that** I can navigate by domain expertise.

**Acceptance Criteria:**

```gherkin
GIVEN personas have been imported
WHEN I call PersonaRepo.list_divisions()
THEN I get 11 divisions with name, description, and persona_count

GIVEN the engineering division has 16 personas
WHEN I view the division detail
THEN I see all 16 personas with their names and descriptions
```

**Technical Notes:**
- Divisions derived from directory structure: `personas/engineering/` → division "engineering"
- Division metadata (description, icon) stored in `persona_divisions` table
- Seed division descriptions in migration or import script

**Test Plan:**
- `test_list_divisions_returns_all_11`
- `test_division_persona_count_accurate`

---

### E1-S4: Persona API Endpoints

**As a** frontend developer or MCP client,
**I want** REST API endpoints for persona operations,
**So that** any client can browse and hire from the catalog.

**Acceptance Criteria:**

```gherkin
GIVEN the persona catalog is loaded
WHEN I GET /api/v1/personas
THEN I receive a paginated list of all personas with summary fields

WHEN I GET /api/v1/personas?division=marketing
THEN I receive only marketing division personas

WHEN I GET /api/v1/personas/:id
THEN I receive the full persona detail including personality and workflows

WHEN I GET /api/v1/personas/search?q=frontend
THEN I receive personas matching "frontend" in name, description, or tags

WHEN I GET /api/v1/personas/divisions
THEN I receive all divisions with counts

WHEN I POST /api/v1/personas/:id/hire
  with body { "name": "my-frontend-dev", "company_id": null }
THEN a new agent is created with persona config injected into system_prompt
AND the agent's config_json contains { "persona_id": "..." }
AND I receive the created Agent object
```

**Technical Notes:**
- Routes in `forge-api/src/routes/personas.rs`
- OpenAPI annotations on all endpoints
- Hire endpoint reuses existing `AgentRepo.create()` with persona-derived config

**Test Plan:**
- `test_list_personas_returns_200`
- `test_list_personas_by_division`
- `test_get_persona_detail`
- `test_search_personas`
- `test_list_divisions`
- `test_hire_persona_creates_agent`
- `test_hire_persona_preserves_persona_link`

---

### E1-S5: Persona Import CLI & Startup Loading

**As a** system administrator,
**I want** personas loaded from a directory on startup (like skills),
**So that** the catalog is available immediately without manual import.

**Acceptance Criteria:**

```gherkin
GIVEN FORGE_PERSONAS_DIR is set to a directory with persona files
WHEN the server starts
THEN all persona files are parsed and upserted into the database
AND the count is logged ("Loaded 107 personas from 11 divisions")

GIVEN FORGE_PERSONAS_DIR is not set
WHEN the server starts
THEN it checks ./personas/ as default
AND if not found, logs a warning and continues (no crash)

GIVEN a persona file has been updated on disk
WHEN the server restarts
THEN the updated persona is upserted (preserving ID, updating content)
```

**Technical Notes:**
- Add to `forge-app/src/main.rs` startup sequence (after skill loading)
- Env var: `FORGE_PERSONAS_DIR` (default: `./personas`)
- Reuse SkillRepo pattern: warn-and-continue on errors

**Test Plan:**
- `test_startup_loads_personas_from_directory`
- `test_startup_handles_missing_directory_gracefully`
- `test_upsert_updates_on_restart`

---

### E1-S6: Persona Catalog Frontend Page

**As a** user,
**I want** a visual catalog page to browse and hire personas,
**So that** I can discover and onboard the right specialist.

**Acceptance Criteria:**

```gherkin
GIVEN I navigate to the Personas page
WHEN the page loads
THEN I see a sidebar with 11 division categories and agent counts
AND the main area shows persona cards in a grid layout

GIVEN I click on "Engineering" in the sidebar
WHEN the filter applies
THEN only engineering personas are shown (16 cards)

GIVEN I type "frontend" in the search bar
WHEN I submit the search
THEN only personas matching "frontend" appear

GIVEN I click on a persona card
WHEN the detail modal opens
THEN I see: full personality description, deliverables list,
     success metrics, workflow steps, and tags

GIVEN I click "Hire" on a persona detail modal
WHEN I confirm the agent name
THEN a new agent is created and I'm redirected to the Agents page
AND the new agent appears with the persona's emoji and division badge
```

**Technical Notes:**
- New route: `frontend/src/routes/personas/+page.svelte`
- Components: PersonaCard, PersonaDetail, DivisionSidebar
- Use `$state` runes for filter/search state
- API calls: `GET /personas`, `GET /personas/divisions`, `POST /personas/:id/hire`

**Test Plan:**
- Manual: browse, filter, search, hire flow
- E2E: `test_persona_catalog_page_loads`
- E2E: `test_hire_from_catalog_creates_agent`

---

### E1-S7: Persona → Agent Config Mapping

**As a** system,
**I want** rich persona data to map intelligently to agent configuration,
**So that** hired agents have optimal system prompts and tool allowlists.

**Acceptance Criteria:**

```gherkin
GIVEN a persona with division "engineering" and role "Frontend Developer"
WHEN the persona is hired
THEN the agent system_prompt includes:
  - Persona personality section
  - Deliverables section
  - Success metrics section
  - Workflow steps section

GIVEN a persona with division "testing"
WHEN the persona is hired
THEN the agent allowed_tools is set to testing-appropriate tools
  [Read, Grep, Glob, Bash, Write]

GIVEN a persona with division "design"
WHEN the persona is hired
THEN the agent allowed_tools is restricted to read-only + design tools
  [Read, Grep, Glob, WebSearch]
```

**Technical Notes:**
- `PersonaMapper` in forge-persona: persona → NewAgent translation
- Division → tool allowlist mapping (configurable, not hardcoded)
- System prompt template: structured sections for LLM consumption

**Test Plan:**
- `test_engineering_persona_maps_to_full_tools`
- `test_design_persona_maps_to_read_only`
- `test_testing_persona_maps_to_testing_tools`
- `test_system_prompt_includes_personality`
- `test_system_prompt_includes_deliverables`

---

### E1-S8: MCP Server Persona Tools

**As an** MCP client (IDE, external tool),
**I want** persona-related MCP tools,
**So that** I can browse and hire from the catalog programmatically.

**Acceptance Criteria:**

```gherkin
GIVEN the MCP server is running
WHEN I call forge_persona_list with division filter
THEN I receive matching personas

WHEN I call forge_persona_hire with persona_id and agent_name
THEN a new agent is created from the persona
```

**Technical Notes:**
- Add to `forge-mcp-bin`: `forge_persona_list`, `forge_persona_hire`
- Add resource: `forge://personas`

**Test Plan:**
- `test_mcp_persona_list_tool`
- `test_mcp_persona_hire_tool`

---

## Technical Architecture

```
forge-persona (new crate)
├── src/
│   ├── lib.rs          // pub mod parser, catalog, mapper
│   ├── parser.rs       // PersonaParser: markdown → Persona
│   ├── catalog.rs      // PersonaCatalog: in-memory index
│   └── mapper.rs       // PersonaMapper: Persona → NewAgent
│
├── Cargo.toml
│   dependencies: forge-core, forge-agent, serde, serde_json, chrono, uuid

forge-db (extend)
├── src/repos/persona_repo.rs  // PersonaRepo + PersonaDivisionRepo
├── migrations/0009_personas.sql

forge-api (extend)
├── src/routes/personas.rs     // REST endpoints

forge-mcp-bin (extend)
├── src/ persona tools

personas/ (new directory at repo root)
├── engineering/     (16 files from agency-agents)
├── design/          (8 files)
├── marketing/       (17 files)
├── paid-media/      (7 files)
├── product/         (4 files)
├── project-management/ (6 files)
├── testing/         (8 files)
├── support/         (6 files)
├── spatial-computing/ (6 files)
├── specialized/     (15 files)
└── game-development/ (5+ files)
```

## Story Point Estimates

| Story | Points | Sprint |
|-------|--------|--------|
| E1-S1 Parser | 3 | S1 |
| E1-S2 DB Schema | 3 | S1 |
| E1-S3 Division Taxonomy | 2 | S1 |
| E1-S4 API Endpoints | 3 | S1 |
| E1-S5 Startup Loading | 2 | S1 |
| E1-S6 Frontend Page | 5 | S1 |
| E1-S7 Config Mapping | 3 | S1 |
| E1-S8 MCP Tools | 2 | S1 |
| **Total** | **23** | |
