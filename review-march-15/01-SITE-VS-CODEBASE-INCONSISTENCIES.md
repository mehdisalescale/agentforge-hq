# Site Documentation vs Codebase — Inconsistency Report

> **Date:** 2026-03-15
> **Reviewer:** Senior Architecture Review
> **Scope:** All 22 site-docs markdown files compared against actual Rust + Svelte source

---

## Severity Legend

| Level | Meaning |
|-------|---------|
| **CRITICAL** | Documentation actively misleads developers; will cause bugs or wrong assumptions |
| **HIGH** | Significant gap between docs and reality; wastes developer time |
| **MEDIUM** | Inaccurate but not dangerous; cosmetic or stale information |
| **LOW** | Minor nitpick or wording that could be clearer |

---

## 1. MCP Tool Count Mismatch — HIGH

**Site claims:** 19 MCP tools (repeated in index.md, mcp.md, mcp-tools.md, CLAUDE.md)
**Actual code:** 20 tools in `forge-mcp-bin/src/main.rs`

The missing tool in documentation is `forge_list_goals`, which is implemented and functional but never appears in the reference docs. Additionally, the MCP tools reference page (`reference/mcp-tools.md`) lists 7 workforce tools but actual workforce category has only 2 (`forge_list_personas`, `forge_hire_persona`). The doc conflates agent CRUD (5 tools) and session CRUD (5 tools) under "workforce" umbrella inconsistently.

**Fix:** Update all references to 20 tools. Add `forge_list_goals` to reference/mcp-tools.md under Governance section.

---

## 2. Middleware Chain Count Contradiction — HIGH

**Site says (architecture/middleware.md):** 7-middleware pipeline (RateLimit → CircuitBreaker → CostCheck → Governance → SecurityScan → Persist → Spawn)
**Actual code (forge-api/src/middleware.rs docstring):** 8-middleware pattern (RateLimit → CircuitBreaker → CostCheck → SkillInjection → Persist → Spawn → ExitGate + domain-specific)
**Strategy docs (architecture-rethink.md):** Says "simplify from 9 to 7"
**Wave history (wave-history.md):** Says "9→7 middleware"

Three different numbers in three different docs. The actual code has an 8-middleware pattern with ExitGate as a quality gate. The site's middleware.md omits SkillInjection and ExitGate but includes Governance and SecurityScan which don't appear in the actual middleware chain enum.

**Fix:** Audit the actual `MiddlewareError` variants and the run pipeline in `forge-api`. Write ONE canonical middleware list. Update middleware.md, architecture-rethink.md, and wave-history.md to match.

---

## 3. Event Count Mismatch — MEDIUM

**Site claims (architecture/events.md):** 38 ForgeEvent variants across 7 categories
**CLAUDE.md claims:** 35 variants
**Actual code (forge-core/src/events.rs):** 35 variants

The events.md doc inflates the count to 38. This likely happened when the doc was written speculatively before some events were consolidated. The actual enum has exactly 35 variants.

**Fix:** Update events.md to 35. Enumerate them accurately from source. Remove phantom variants.

---

## 4. Database Default Path Inconsistency — MEDIUM

**Site (configuration.md, env-vars.md):** `FORGE_DB_PATH` defaults to `~/.agentforge/forge.db`
**CLAUDE.md:** `~/.agentforge/forge.db`
**Actual code (forge-app/main.rs):** `~/.claude-forge/forge.db`

Two different default paths. The code uses `.claude-forge` but all docs say `.agentforge`. A user reading the docs would look in the wrong directory.

**Fix:** Decide on one canonical path. If `.claude-forge` is intended (historical), update all docs. If `.agentforge` is the new branding, update the code. Recommend `.agentforge` for brand consistency.

---

## 5. API Routes Missing from Documentation — HIGH

**Site (reference/api.md) documents these routes but actual codebase has additional ones not documented:**

| Route | Exists in Code | In api.md |
|-------|---------------|-----------|
| `GET /api/v1/companies/:id` | Yes | No |
| `PATCH /api/v1/companies/:id` | Yes | No |
| `DELETE /api/v1/companies/:id` | Yes | No |
| `GET /api/v1/departments/:id` | Yes | No |
| `PATCH /api/v1/departments/:id` | Yes | No |
| `DELETE /api/v1/departments/:id` | Yes | No |
| `GET /api/v1/agents/stats` | Yes | No |
| `GET /api/v1/agents/:id/stats` | Yes | No |
| `GET /api/v1/personas/divisions` | Yes | No |
| `GET /api/v1/health` | Yes | No |

At least 10 implemented routes are undocumented. The API reference gives users an incomplete picture.

**Fix:** Generate route documentation from actual Axum router registration. Consider auto-generating from utoipa/OpenAPI annotations.

---

## 6. Persona Count Unverifiable — MEDIUM

**Site claims:** "112 personas across 11 divisions" (reference/personas.md)
**CLAUDE.md claims:** "100+ pre-built agent personas"
**Actual code:** Personas loaded from database via migrations/seed data. No TOML catalog files found in repo.

The persona catalog is claimed to exist as TOML files in the `personas/` directory, but the actual persona loading happens through database seeding. The `personas/` directory at repo root contains files, but the `forge-persona` crate loads from DB, not from filesystem at runtime.

**Fix:** Clarify the persona loading pipeline in docs. If TOML files are the source of truth that get seeded into DB, document that flow. Verify the actual count matches 112.

---

## 7. Frontend Page Count Drift — MEDIUM

**Wave history claims:** "16→12 pages"
**Actual frontend routes found:** 15 pages (/, agents, sessions, skills, workflows, schedules, memory, hooks, analytics, settings, companies, org-chart, personas, goals, approvals)

The "12 pages" claim is stale. After Epic 1, 5 new org/governance pages were added but the wave history wasn't updated.

**Fix:** Update wave-history.md page count. Consider listing pages explicitly.

---

## 8. Strategy Docs Reference "Empty Sidebar Pages" — LOW but Stale

**gaps.md says:** "6 empty sidebar pages" (Skills, Workflows, Memory, Hooks, Schedules, Settings)
**Actual state:** Skills and Memory pages have functional UIs. Workflows has partial UI. Hooks and Schedules are stubs. Settings displays runtime config.

The gap analysis is partially resolved but the doc still lists all 6 as empty. This misleads anyone reading the gaps doc into thinking no progress was made.

**Fix:** Update gaps.md with current status per page. Mark resolved gaps as closed.

---

## 9. Architecture Overview Diagram Drift — MEDIUM

**overview.md Mermaid diagram** shows a clean 3-layer architecture (Clients → Orchestrator → Core Services) but doesn't include:
- HookReceiver endpoints (Wave 4)
- AgentConfigurator (Wave 4)
- The persona hire flow
- The org-chart/governance layer

The diagram represents Wave 2 architecture, not the current state.

**Fix:** Redraw the Mermaid diagram to include governance layer, hook receiver, and configurator. Add a "last updated" date to architecture docs.

---

## 10. MCP Server Transport Documentation — LOW

**mcp.md says:** "stdio transport" only
**CLAUDE.md (Wave 4 section):** Mentions HTTP SSE as additional transport
**Actual code:** Only stdio transport implemented (`rmcp` stdio server)

HTTP SSE transport is aspirational, not implemented. Docs should not present it as current.

**Fix:** Remove HTTP SSE references until implemented. Add a "Planned" section for future transports.

---

## 11. Build Instructions Inconsistency — LOW

**building.md says:** `cd frontend && npm install && npm run build`
**CLAUDE.md says:** `cd frontend && pnpm install && pnpm build`
**Actual package manager:** pnpm (pnpm-lock.yaml exists, no package-lock.json)

Using `npm` instead of `pnpm` would create a parallel lock file and potentially different dependency resolution.

**Fix:** Standardize all build docs to use `pnpm`.

---

## 12. `.gitignore` Missing `site/` Directory — MEDIUM

The `site/` directory (MkDocs build output) is committed to the repo. It contains ~60+ generated HTML files, JavaScript bundles, CSS, and search indexes. This is a build artifact that should not be in version control.

**Fix:** Add `site/` to `.gitignore`. Remove from tracking with `git rm -r --cached site/`.

---

## Summary Table

| # | Issue | Severity | Effort |
|---|-------|----------|--------|
| 1 | MCP tool count (19 vs 20) | HIGH | 30 min |
| 2 | Middleware chain contradictions | HIGH | 1 hour |
| 3 | Event count (38 vs 35) | MEDIUM | 20 min |
| 4 | DB default path mismatch | MEDIUM | 15 min |
| 5 | 10+ undocumented API routes | HIGH | 2 hours |
| 6 | Persona count unverifiable | MEDIUM | 1 hour |
| 7 | Frontend page count drift | MEDIUM | 15 min |
| 8 | Stale gap analysis | LOW | 30 min |
| 9 | Architecture diagram outdated | MEDIUM | 1 hour |
| 10 | MCP transport claim | LOW | 10 min |
| 11 | npm vs pnpm in build docs | LOW | 10 min |
| 12 | site/ committed to git | MEDIUM | 15 min |

**Total estimated fix time:** ~7 hours for a single developer
