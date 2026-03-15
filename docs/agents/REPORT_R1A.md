STATUS: COMPLETE
FILES_MODIFIED:
  - site-docs/reference/mcp-tools.md (fixed Sessions count 4→5, added session_delete, fixed Observability count 1→2)
  - site-docs/architecture/mcp.md (fixed Sessions count 4→5, Observability count 1→2, removed HTTP SSE transport claim)
  - site-docs/architecture/events.md (rewrote: 38→43 variants, listed all variants grouped by category from actual code)
  - site-docs/reference/api.md (added 30+ missing routes: companies CRUD, departments CRUD, sessions export, skills/:id, all workflow routes, all memory routes, all hook CRUD routes, all schedule routes, personas/divisions)
  - site-docs/reference/personas.md (clarified loading pipeline: markdown files → forge-persona parser → DB seed → API; updated count to 113; fixed division list to match actual 11 directories)
  - site-docs/development/wave-history.md (fixed frontend page count to 15)
  - site-docs/strategy/gaps.md (updated sidebar page resolution status: Skills RESOLVED, Memory RESOLVED, Workflows PARTIAL, Hooks STUB, Schedules STUB, Settings RESOLVED, Analytics RESOLVED)
ISSUES_FOUND:
  - MCP doc listed Sessions as 4 tools, actual is 5 (session_delete was missing)
  - MCP doc listed Observability as 1 tool, actual is 2 (forge_get_session_events + forge_get_analytics)
  - Events doc said 38 variants, actual is 43; variant names in doc didn't match code (e.g. SessionStarted→ProcessStarted, ProcessSpawned→ProcessStarted, RateLimitHit doesn't exist, SubAgentSpawned→SubAgentRequested+SubAgentStarted)
  - API doc was missing ~30 routes (companies/:id CRUD, departments/:id CRUD, session export, skills/:id, all workflow/memory/hook/schedule CRUD)
  - Personas doc said 112, actual file count is 113; said "markdown file" but didn't explain the loading pipeline
  - Wave history said "16 → 12" frontend pages, actual is 15 route directories
  - building.md and quickstart.md already used pnpm — no changes needed
  - configuration.md and env-vars.md already referenced ~/.agentforge/forge.db — no changes needed
  - middleware.md was already accurate — no changes needed
COUNTS_VERIFIED:
  - MCP tools: 19
  - ForgeEvent variants: 43
  - Middleware stages: 7
  - API routes documented: 58
  - Frontend pages: 15
  - Persona files: 113
  - Persona divisions: 11
VALIDATION: mkdocs build --strict passed cleanly
