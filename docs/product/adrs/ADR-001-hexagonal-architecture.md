# ADR-001: Hexagonal Architecture with Ports & Adapters

## Status
Accepted

## Context
AgentForge needs to support multiple execution backends (Claude CLI, Hermes, OpenClaw, future backends). The current codebase has Claude CLI spawning hardwired into forge-process. We need an abstraction that allows:
- Adding new backends without modifying orchestration code
- Testing with mock backends
- Runtime backend selection per agent
- Graceful degradation when a backend is unavailable

## Decision
Adopt hexagonal architecture (ports & adapters) for the process execution layer:

```
          ┌─────────────────────────────┐
          │     Application Core        │
          │  (forge-core, forge-agent,  │
          │   forge-db, forge-safety)   │
          └─────────────┬───────────────┘
                        │
            ┌───────────┴───────────────┐
            │    Port: ProcessBackend   │
            │    trait                   │
            └───────────┬───────────────┘
          ┌─────────────┼─────────────────┐
          │             │                 │
   ┌──────┴──────┐ ┌───┴────────┐ ┌──────┴───────┐
   │ClaudeBackend│ │HermesBackend│ │OpenClawBackend│
   │  (adapter)  │ │  (adapter)  │ │   (adapter)   │
   └─────────────┘ └────────────┘ └──────────────┘
```

**Port (trait):**
```rust
#[async_trait]
pub trait ProcessBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn health_check(&self) -> BackendHealth;
    async fn spawn(&self, config: &SpawnConfig, prompt: &str, session_id: Option<&str>) -> Result<ProcessHandle, SpawnError>;
    fn capabilities(&self) -> BackendCapabilities;
}
```

**Registry:**
```rust
pub struct BackendRegistry {
    backends: HashMap<String, Arc<dyn ProcessBackend>>,
}
```

## Consequences

### Positive
- New backends added by implementing one trait (no core changes)
- Mock backends enable reliable testing
- Backend health monitoring is standardized
- Runtime backend switching per agent

### Negative
- Slight indirection cost (dynamic dispatch via trait object)
- Each backend must normalize its output to ForgeEvent (adapter responsibility)
- Backend-specific features (Hermes tools, OpenClaw Docker) must be surfaced through generic capabilities

### Risks
- Over-abstraction: the trait might not capture all backend-specific behaviors. Mitigated by `capabilities()` and `config: serde_json::Value` escape hatch.

## Alternatives Considered
1. **Enum-based dispatch**: `match backend { Claude => ..., Hermes => ... }`. Simpler but violates open-closed principle. Every new backend requires modifying the match.
2. **Plugin system (dylib)**: Maximum extensibility but adds complexity (ABI stability, loading). Overkill for 3-5 backends.
3. **Microservices**: Each backend as a separate service. Contradicts single-binary principle.
