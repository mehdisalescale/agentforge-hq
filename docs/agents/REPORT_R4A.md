STATUS: COMPLETE
FILES_MODIFIED:
  - site-docs/architecture/overview.md (full rewrite: new Mermaid diagram, request flow, hook receiver flow)
  - site-docs/architecture/crates.md (updated all crate descriptions with verified numbers)
  - site-docs/reference/personas.md (documented full 5-stage pipeline, file structure, API endpoints, AgentConfigurator)
DIAGRAM_UPDATED: yes
  - Added: HookReceiver endpoints, AgentConfigurator, EventBus fan-out (mpsc/broadcast), SecurityScanner, ConcurrentRunner, MCP binary
  - Updated connections: Hook→EventBus, EventBus→BatchWriter (mpsc), EventBus→WebSocket (broadcast), Persona pipeline→Repos
CRATE_DESCRIPTIONS_UPDATED: 5
  - forge-core: EventBus fan-out (mpsc + broadcast), 43 ForgeEvent variants
  - forge-db: r2d2 pool (write:1, read:N), 17 repos, SafetyRepo
  - forge-safety: CircuitBreaker with persistence, SecurityScanner
  - forge-process: ConcurrentRunner (semaphore-based), max_concurrent + max_output_bytes
  - forge-api: 14 route modules, AgentConfigurator, HookReceiver
PERSONA_PIPELINE_DOCUMENTED: yes
  - 5-stage pipeline: Walk → Filter → Parse → Seed divisions → Seed personas
  - File structure with example markdown
  - API endpoints table (4 routes + MCP equivalents)
  - Hiring flow details
  - AgentConfigurator workspace file generation
NUMBERS_VERIFIED:
  - ForgeEvent variants: 43 (from forge-core/src/events.rs)
  - Persona files: 98 markdown (from personas/ directory)
  - Divisions: 11 (from parser whitelist)
  - Repos: 17 (from forge-db/src/repos/mod.rs)
  - Migrations: 12 (from forge-db/migrations/)
  - Middleware stages: 7 (from forge-api/src/routes/run.rs)
  - MCP tools: 19 (from forge-mcp-bin)
MKDOCS_BUILD: pass
