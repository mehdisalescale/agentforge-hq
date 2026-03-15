# Personas

AgentForge includes 112 pre-built AI personas across 11 divisions.

## Divisions

| Division | Count | Examples |
|----------|-------|---------|
| Engineering | ~20 | Senior Software Engineer, Backend Developer, Frontend Engineer, Systems Architect |
| Security | ~10 | Security Auditor, Penetration Tester, Security Architect |
| Testing | ~10 | QA Engineer, Test Automation Engineer, Performance Tester |
| DevOps | ~10 | DevOps Engineer, SRE, Cloud Architect, CI/CD Specialist |
| Product | ~10 | Product Manager, Product Owner, Business Analyst |
| Design | ~8 | UX Designer, UI Designer, Design System Engineer |
| Marketing | ~8 | Content Strategist, SEO Specialist, Growth Engineer |
| Data | ~10 | Data Engineer, Data Scientist, ML Engineer, Analytics Engineer |
| Support | ~6 | Technical Support Engineer, Customer Success, Documentation |
| Legal | ~5 | Compliance Officer, Legal Analyst, Privacy Engineer |
| Executive | ~5 | CTO, VP Engineering, Engineering Manager |

## Persona Structure

Each persona is a markdown file in `personas/` with:

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
