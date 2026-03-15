# Personas

AgentForge includes 113 pre-built AI personas across 11 divisions.

## Loading Pipeline

Personas are markdown files in `personas/<division>/` directories. They are:

1. **Parsed** by `forge-persona` at startup (frontmatter + body)
2. **Seeded** into SQLite via database migration
3. **Served** via the REST API (`GET /api/v1/personas`) and MCP (`forge_list_personas`)

## Divisions

| Division | Examples |
|----------|---------|
| Engineering | AI Engineer, Backend Architect, Frontend Engineer, Systems Architect |
| Security | Security Auditor, Penetration Tester, Security Architect |
| Testing | QA Engineer, Test Automation Engineer, Performance Tester |
| DevOps | DevOps Engineer, SRE, Cloud Architect, CI/CD Specialist |
| Product | Product Manager, Product Owner, Business Analyst |
| Design | UX Architect, UX Researcher, Whimsy Injector |
| Marketing | Content Strategist, SEO Specialist, Growth Engineer |
| Data | Data Engineer, Data Scientist, ML Engineer, Analytics Engineer |
| Game Development | Game designers and developers |
| Paid Media | Paid media specialists |
| Support | Technical Support Engineer, Customer Success, Documentation |

## Persona Structure

Each persona is a markdown file in `personas/<division>/` with:

- **Name** and **title**
- **Division** classification
- **Personality** — how the agent behaves
- **Deliverables** — what it produces
- **Skills** — areas of expertise
- **Constraints** — what it should NOT do

## Hiring Flow

1. Browse personas at `/personas` or via MCP (`forge_list_personas`)
2. Click **Hire** (or call `forge_hire_persona`)
3. AgentForge creates:
   - An **agent** with the persona's system prompt
   - An **org position** linking agent to company hierarchy
4. The agent appears in `/agents` and `/org-chart`

## AgentConfigurator

When an agent runs, AgentConfigurator generates a CLAUDE.md containing:

- Persona identity (name, role, personality)
- Company context (name, mission, budget)
- Active goals
- Matched skills from the skill library
- Behavioral constraints

This CLAUDE.md is written to the agent's worktree before Claude Code launches.
