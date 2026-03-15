# Agent W4-A: AgentConfigurator + ConfiguredSpawn

> You are Agent W4-A. Your job: build the AgentConfigurator that generates CLAUDE.md and hooks.json per persona, and modify SpawnMiddleware to use it. This replaces SkillInjection and TaskTypeDetection middlewares with file-based configuration.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
crates/forge-api/src/middleware.rs                  — FULL FILE (current middlewares)
crates/forge-api/src/routes/run.rs                 — FULL FILE (chain assembly)
crates/forge-api/src/state.rs                      — AppState struct
crates/forge-db/src/repos/skills.rs                — SkillRepo
crates/forge-db/src/repos/companies.rs             — CompanyRepo, Company struct
crates/forge-db/src/repos/goals.rs                 — GoalRepo
crates/forge-db/src/repos/org_positions.rs         — OrgPositionRepo
crates/forge-persona/src/catalog.rs                — persona data structures
crates/forge-process/src/spawn.rs                  — SpawnConfig, spawn()
crates/forge-process/src/skill_router.rs           — SkillRouter (current skill matching logic)
stat-qou-plan/REVISED_PLAN.md                      — what we're doing and why
stat-qou-plan/ARCHITECTURE_RETHINK.md              — the vision: configure, don't inject
```

## Step 2: Create AgentConfigurator

Create `crates/forge-api/src/configurator.rs`:

This struct generates per-agent configuration files. It replaces what SkillInjection and TaskTypeDetection middlewares do at runtime with static files generated before launch.

### generate_claude_md(agent_id, prompt) → String

Builds a CLAUDE.md containing:

1. **Persona identity** — role name, personality, deliverables (from persona catalog via agent's persona_id)
2. **Company context** — company name, mission, budget remaining (from CompanyRepo via OrgPosition lookup)
3. **Active goals** — company goals with status planned/in_progress (from GoalRepo)
4. **Relevant skills** — matched using the SAME logic as SkillInjectionMiddleware + TaskTypeDetectionMiddleware:
   - Extract keywords from the prompt
   - Match against skill tags
   - Also use SkillRouter to match by task type
   - Write the matched skill content directly into the CLAUDE.md
5. **Rules** — stay in role scope, budget constraints, approval thresholds

Important: reuse the existing keyword extraction and skill matching logic. Don't reinvent it — extract it into a shared function or call it from here.

```rust
pub struct AgentConfigurator {
    pub skill_repo: Arc<SkillRepo>,
    pub company_repo: Arc<CompanyRepo>,
    pub org_position_repo: Arc<OrgPositionRepo>,
    pub goal_repo: Arc<GoalRepo>,
    pub persona_repo: Arc<PersonaRepo>,
    pub agent_repo: Arc<AgentRepo>,
}

impl AgentConfigurator {
    /// Generate CLAUDE.md content for an agent about to execute a prompt
    pub fn generate_claude_md(&self, agent_id: &str, prompt: &str) -> String { ... }

    /// Generate hooks.json content that reports back to AgentForge
    pub fn generate_hooks_json(&self, session_id: &str, port: u16) -> String { ... }

    /// Write both files into a directory (worktree or temp dir)
    pub fn configure_workspace(&self, agent_id: &str, prompt: &str,
                                session_id: &str, dir: &str) -> std::io::Result<()> {
        let claude_md = self.generate_claude_md(agent_id, prompt);
        let hooks_json = self.generate_hooks_json(session_id, 4173);

        // Write {dir}/CLAUDE.md
        std::fs::write(format!("{}/CLAUDE.md", dir), &claude_md)?;

        // Write {dir}/.claude/hooks.json
        let claude_dir = format!("{}/.claude", dir);
        std::fs::create_dir_all(&claude_dir)?;
        std::fs::write(format!("{}/hooks.json", claude_dir), &hooks_json)?;

        Ok(())
    }
}
```

### generate_hooks_json(session_id, port) → String

Generates Claude Code hooks that POST events back to AgentForge:

```json
{
  "hooks": {
    "PreToolUse": [{
      "type": "command",
      "command": "curl -sf http://127.0.0.1:{port}/api/v1/hooks/pre-tool -H 'Content-Type: application/json' -d '{\"session_id\":\"{session_id}\",\"tool_name\":\"$TOOL_NAME\"}'",
      "timeout": 3000
    }],
    "PostToolUse": [{
      "type": "command",
      "command": "curl -sf http://127.0.0.1:{port}/api/v1/hooks/post-tool -H 'Content-Type: application/json' -d '{\"session_id\":\"{session_id}\",\"tool_name\":\"$TOOL_NAME\"}'",
      "timeout": 3000
    }],
    "Stop": [{
      "type": "command",
      "command": "curl -sf http://127.0.0.1:{port}/api/v1/hooks/stop -H 'Content-Type: application/json' -d '{\"session_id\":\"{session_id}\"}'",
      "timeout": 5000
    }]
  }
}
```

## Step 3: Modify SpawnMiddleware

In `crates/forge-api/src/middleware.rs`, update `SpawnMiddleware`:

1. Add `configurator: Arc<AgentConfigurator>` field
2. Before spawning the CLI, call `configurator.configure_workspace(...)` to write CLAUDE.md + hooks.json into the working directory
3. The spawned Claude Code will read those files automatically

```rust
pub struct SpawnMiddleware {
    pub event_bus: Arc<EventBus>,
    pub session_repo: Arc<SessionRepo>,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub cost_tracker: Arc<CostTracker>,
    pub configurator: Arc<AgentConfigurator>,  // NEW
}
```

In the `process` method, before calling `spawn()`:
```rust
// Configure workspace with persona identity, skills, goals, hooks
if let Err(e) = self.configurator.configure_workspace(
    &ctx.agent_id, &ctx.prompt, &ctx.session_id, &ctx.directory
) {
    tracing::warn!("Failed to configure workspace: {}", e);
    // Non-fatal — agent runs without persona config
}
```

## Step 4: Update run.rs Chain Assembly

In `crates/forge-api/src/routes/run.rs`:

1. Create the `AgentConfigurator` from AppState repos
2. Pass it to `SpawnMiddleware`
3. **Remove** `SkillInjectionMiddleware` and `TaskTypeDetectionMiddleware` from the chain

The chain becomes:
```
RateLimit → CircuitBreaker → CostCheck → Governance → SecurityScan → Persist → Spawn
```

(6 middlewares, down from 9)

## Step 5: Register the configurator module

In `crates/forge-api/src/lib.rs`, add `pub mod configurator;`

## Step 6: Write Tests

In `crates/forge-api/src/configurator.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_claude_md_includes_persona_role() {
        // Setup repos with test data
        // Generate CLAUDE.md
        // Assert it contains the persona name and personality
    }

    #[test]
    fn generate_claude_md_includes_matched_skills() {
        // Setup skills with tags
        // Generate with prompt containing matching keywords
        // Assert skill content appears in output
    }

    #[test]
    fn generate_hooks_json_is_valid_json() {
        // Generate hooks.json
        // Parse it as serde_json::Value
        // Assert it has PreToolUse, PostToolUse, Stop keys
    }

    #[test]
    fn configure_workspace_writes_files() {
        // Use tempdir
        // Call configure_workspace
        // Assert CLAUDE.md and .claude/hooks.json exist
    }
}
```

## Step 7: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-api 2>&1         # all tests pass
cd frontend && pnpm build 2>&1       # must build cleanly
```

## Rules

- Create: `crates/forge-api/src/configurator.rs`
- Modify: `crates/forge-api/src/middleware.rs` — add configurator field to SpawnMiddleware
- Modify: `crates/forge-api/src/routes/run.rs` — remove SkillInjection + TaskTypeDetection, wire configurator
- Modify: `crates/forge-api/src/lib.rs` — add `pub mod configurator`
- Do NOT modify forge-db, forge-process, forge-core, forge-safety, forge-persona
- Do NOT touch frontend files
- Do NOT touch main.rs
- Do NOT delete SkillInjectionMiddleware or TaskTypeDetectionMiddleware structs yet — just remove them from the chain. Agent W4-C handles cleanup.
- Do NOT modify existing tests — only add new ones
- Commit with: `feat(api): add AgentConfigurator for per-persona CLAUDE.md and hooks.json generation`

## Report

When done, append your report here:

```
STATUS: complete
FILES_CREATED: [crates/forge-api/src/configurator.rs]
FILES_MODIFIED: [crates/forge-api/src/lib.rs, crates/forge-api/src/middleware.rs, crates/forge-api/src/routes/run.rs]
TESTS_ADDED: 4
CONFIGURATOR_GENERATES: [CLAUDE.md (persona identity, company context, active goals, matched skills, rules), .claude/hooks.json (PreToolUse, PostToolUse, Stop)]
CHAIN_BEFORE: [RateLimit, CircuitBreaker, CostCheck, Governance, SkillInjection, TaskTypeDetection, SecurityScan, Persist, Spawn]
CHAIN_AFTER: [RateLimit, CircuitBreaker, CostCheck, Governance, SecurityScan, Persist, Spawn]
ISSUES: [Agent struct does not expose persona_id field from DB; configurator uses agent system_prompt (which already contains persona info from hire flow) instead of direct persona lookup. Agent W4-C can address this when cleaning up.]
```
