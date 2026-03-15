# Personas

<!-- Last updated: 2026-03-15 -->

AgentForge includes 98 pre-built AI personas across 11 divisions.

## Loading Pipeline

Personas are markdown files in `personas/<division>/` directories. On startup they are loaded through a five-stage pipeline:

1. **Walk** — `PersonaParser::parse_all()` recursively walks the `personas/` directory for `.md` files
2. **Filter** — Files outside the division whitelist (11 known divisions) are skipped
3. **Parse** — Each markdown file is parsed into sections: name (from `# heading`), short description (first paragraph), and `## sections` (Personality, Deliverables, Success Metrics, Workflow, Tags)
4. **Seed divisions** — Unique `division_slug` values are collected, auto-capitalized, and upserted via `persona_repo.upsert_divisions()`
5. **Seed personas** — All parsed personas are converted to `Persona` models and upserted via `persona_repo.upsert_personas()`

This runs in `seed_personas()` in `forge-app/src/main.rs` on every startup. Upsert ensures idempotent reloads.

## Divisions

| Division | Slug | Example Personas |
|----------|------|-----------------|
| Engineering | `engineering` | AI Engineer, Backend Architect, Frontend Engineer, Systems Architect |
| Security | `specialized` | Security Auditor, Penetration Tester, Security Architect |
| Testing | `testing` | QA Engineer, Test Automation Engineer, Performance Tester |
| Design | `design` | UX Architect, UX Researcher, Whimsy Injector |
| Marketing | `marketing` | Content Strategist, SEO Specialist, Growth Engineer |
| Product | `product` | Product Manager, Product Owner, Business Analyst |
| Project Management | `project-management` | Scrum Master, Technical Program Manager |
| Support | `support` | Technical Support Engineer, Customer Success, Documentation |
| Spatial Computing | `spatial-computing` | AR/VR specialists |
| Game Development | `game-development` | Game designers and developers |
| Paid Media | `paid-media` | Paid media specialists |

## Persona File Structure

Each persona is a markdown file in `personas/<division>/` with:

```markdown
# Persona Name

Short description paragraph.

## Personality

- Trait 1
- Trait 2

## Deliverables

- Output 1
- Output 2

## Success Metrics

- Metric 1

## Workflow

1. Step 1
2. Step 2

## Tags

- tag1
- tag2
```

The parser extracts:

- **Name** from the `# heading`
- **Short description** from the first paragraph after the heading
- **Sections** from `## headings` (personality, deliverables, success metrics, workflow, tags)
- **Division** from the parent directory name
- **Slug** from the filename (underscores replaced with hyphens)

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/personas` | List all personas (optional `?division=` filter) |
| `GET` | `/api/v1/personas/:id` | Get single persona |
| `GET` | `/api/v1/personas/divisions` | List all divisions with persona counts |
| `POST` | `/api/v1/personas/:id` | Hire persona into a company |

MCP equivalent: `forge_list_personas` (list with filter), `forge_hire_persona` (hire).

## Hiring Flow

1. Browse personas at `/personas` or via MCP (`forge_list_personas`)
2. Click **Hire** (or call `forge_hire_persona` / `POST /api/v1/personas/:id`)
3. AgentForge creates:
   - An **agent** with the persona's system prompt and metadata
   - An **org position** linking the agent to the company hierarchy
4. The agent appears in `/agents` and `/org-chart`

## AgentConfigurator

When an agent runs via `POST /api/v1/run`, AgentConfigurator generates workspace files:

### CLAUDE.md (written to working directory)

Contains:

- Agent identity (name, system prompt)
- Company context (name, mission, remaining budget)
- Active goals (planned + in_progress)
- Matched skills from the skill library (keyword matching + task type routing)
- Behavioral rules (scope, budget, approval, reporting)

### hooks.json (written to `.claude/hooks.json`)

Configures Claude Code to report back via HTTP:

- `PreToolUse` → `POST /api/v1/hooks/pre-tool`
- `PostToolUse` → `POST /api/v1/hooks/post-tool`
- `Stop` → `POST /api/v1/hooks/stop`
