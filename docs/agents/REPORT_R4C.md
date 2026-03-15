STATUS: COMPLETE
VARIANTS_BEFORE: 10 (Database, Serialization, AgentNotFound, SessionNotFound, SkillNotFound, WorkflowNotFound, Validation, EventBus, Io, Internal)
VARIANTS_AFTER: 12 (Validation, NotFound, Conflict, BudgetExceeded, ApprovalRequired, RateLimited, CliUnavailable, CircuitOpen, Timeout, Database, Internal, Process)
FILES_MODIFIED:
  - crates/forge-core/Cargo.toml (added http dependency)
  - crates/forge-core/src/error.rs (restructured ForgeError enum, added methods)
  - crates/forge-api/src/error.rs (use new methods, structured JSON response with code+retriable)
  - crates/forge-api/src/routes/run.rs (map MiddlewareError to specific ForgeError variants)
  - crates/forge-db/src/repos/agents.rs (AgentNotFound → NotFound)
  - crates/forge-db/src/repos/sessions.rs (SessionNotFound → NotFound)
  - crates/forge-db/src/repos/skills.rs (SkillNotFound → NotFound, Io → Internal)
  - crates/forge-db/src/repos/workflows.rs (WorkflowNotFound → NotFound, updated test)
METHODS_ADDED: is_retriable(), http_status(), error_code()
API_RESPONSE_FORMAT: { error, code, retriable }
CARGO_CHECK: pass
CARGO_TEST: pass
