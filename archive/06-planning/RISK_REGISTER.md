# Claude Forge -- Risk Register

> 20 risks across technical, strategic, operational, and resource categories.
> Reviewed and updated at the start of each phase.

---

## Risk Scoring

**Likelihood:** 1 (Rare) -- 2 (Unlikely) -- 3 (Possible) -- 4 (Likely) -- 5 (Almost Certain)
**Impact:** 1 (Negligible) -- 2 (Minor) -- 3 (Moderate) -- 4 (Major) -- 5 (Critical)
**Risk Score:** Likelihood x Impact (range 1-25)

---

## Risk Matrix

```
                         I M P A C T
                  1       2       3       4       5
              +-------+-------+-------+-------+-------+
          5   |       |       |       |  R17  |       |
              +-------+-------+-------+-------+-------+
 L    4   |       |  R14  |  R07  |  R04  |       |
 I        +-------+-------+-------+-------+-------+
 K    3   |       |  R16  |R09,R12|R01,R03|  R02  |
 E        |       |       |R13,R20|  R08  |       |
 L        +-------+-------+-------+-------+-------+
 I    2   |       |R15,R19|  R10  |  R05  |  R06  |
 H        |       |       |  R18  |       |       |
 O        +-------+-------+-------+-------+-------+
 O    1   |       |       |  R11  |       |       |
 D        +-------+-------+-------+-------+-------+

Legend:
  Score 1-4:   Low risk (green zone)
  Score 5-9:   Medium risk (yellow zone)
  Score 10-15: High risk (orange zone)
  Score 16-25: Critical risk (red zone)
```

---

## Risk Register

### Technical Risks

#### R01: Wasmtime Binary Size Bloat
| Field | Value |
|-------|-------|
| **ID** | R01 |
| **Category** | Technical |
| **Description** | Wasmtime adds ~12 MB to the binary. Combined with other dependencies, the binary may exceed the 50 MB target, making distribution and updates slower. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 4 (Major) |
| **Risk Score** | 12 |
| **Mitigation** | Feature-flag the `plugins` feature. Offer a "slim" build without WASM support (~20 MB). Use Wasmtime feature flags to include only the Cranelift backend. Apply `strip`, thin LTO, and `opt-level = "z"` for the plugins crate. Consider UPX compression for distribution. |
| **Owner** | Lead Developer |
| **Status** | Open -- monitor during Phase 0 build |

#### R02: Plugin Sandbox Escape
| Field | Value |
|-------|-------|
| **ID** | R02 |
| **Category** | Technical / Security |
| **Description** | A vulnerability in the WASM sandbox could allow a malicious plugin to access the host filesystem, network, or memory outside its allocated limits, compromising user data or system integrity. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 5 (Critical) |
| **Risk Score** | 15 |
| **Mitigation** | Use Wasmtime's WASI preview 2 with minimal capabilities granted. Filesystem access scoped to plugin-specific directory via capability-based security. Network disabled by default. Fuel limits prevent infinite loops. Memory limits prevent OOM. Fuzz-test the plugin API boundary. Run security review before Phase 5 release. Keep Wasmtime updated for security patches. |
| **Owner** | Lead Developer |
| **Status** | Open -- address in Phase 5 |

#### R03: libgit2 Edge Cases
| Field | Value |
|-------|-------|
| **ID** | R03 |
| **Category** | Technical |
| **Description** | libgit2 may handle edge cases differently from the git CLI: submodules, LFS, sparse checkouts, large binary files, unusual branch structures, or corrupted repos could cause crashes or incorrect results. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 4 (Major) |
| **Risk Score** | 12 |
| **Mitigation** | Start with core operations only (status, diff, log, branch). Test against 20+ real-world repos including monorepos. Add graceful error handling for unsupported features (return "not supported" rather than crashing). Consider falling back to `git` CLI for edge cases. Document known limitations. |
| **Owner** | Lead Developer |
| **Status** | Open -- address in Phase 3 |

#### R04: MCP Protocol Instability
| Field | Value |
|-------|-------|
| **ID** | R04 |
| **Category** | Technical |
| **Description** | The MCP specification is evolving. Breaking changes to the protocol could require significant rework of the `forge-mcp` crate, invalidating client integrations. |
| **Likelihood** | 4 (Likely) |
| **Impact** | 4 (Major) |
| **Risk Score** | 16 |
| **Mitigation** | Pin to a specific protocol version (2024-11-05). Abstract the protocol layer behind an internal trait so transport/message changes do not affect tool implementations. Monitor the MCP spec repo for changes. Maintain protocol version negotiation in the handshake. Budget 1 week per phase for protocol updates if needed. |
| **Owner** | Lead Developer |
| **Status** | Open -- critical risk, active monitoring |

#### R05: SQLite Performance Under Concurrent Load
| Field | Value |
|-------|-------|
| **ID** | R05 |
| **Category** | Technical |
| **Description** | Despite WAL mode, SQLite may become a bottleneck under high concurrent agent activity (50+ events/second from 10+ agents). Write contention or WAL file growth could degrade performance. |
| **Likelihood** | 2 (Unlikely) |
| **Impact** | 4 (Major) |
| **Risk Score** | 8 |
| **Mitigation** | Batch writer absorbs write spikes (50 events or 2 seconds). Single write connection eliminates write contention. 4 read connections handle concurrent queries. WAL checkpoint controlled (not during batch writes). Monitor WAL file size with `journal_size_limit`. If SQLite becomes a bottleneck, the batch writer can be tuned (larger batches, longer intervals) before considering a different database. |
| **Owner** | Lead Developer |
| **Status** | Open -- monitor during Phase 2+ |

#### R06: Process Management Reliability (Agent Spawning)
| Field | Value |
|-------|-------|
| **ID** | R06 |
| **Category** | Technical |
| **Description** | Spawned `claude` processes may become zombies, orphans, or leak resources if the parent Forge process crashes or the child process misbehaves. This could exhaust system resources over time. |
| **Likelihood** | 2 (Unlikely) |
| **Impact** | 5 (Critical) |
| **Risk Score** | 10 |
| **Mitigation** | Use process groups (`setsid`) so killing the group kills all children. Register all child PIDs in an in-memory set. On graceful shutdown, kill all children. On startup, scan for orphaned processes from previous runs (PID file). Implement a reaper task that checks for zombie children every 30 seconds. Use `tokio::process` with `kill_on_drop`. Set process timeouts (kill after configurable max duration). |
| **Owner** | Lead Developer |
| **Status** | Open -- address in Phase 0 |

#### R07: Workflow State Machine Bugs
| Field | Value |
|-------|-------|
| **ID** | R07 |
| **Category** | Technical |
| **Description** | The workflow engine's state machine (5 step types, error strategies, parallel execution) is complex. Bugs in state transitions could cause workflows to hang, skip steps, or produce incorrect results. |
| **Likelihood** | 4 (Likely) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 12 |
| **Mitigation** | Exhaustive unit tests for every state transition. Property-based testing with proptest for random workflow shapes. Timeout on every state (no infinite hangs). Workflow run logging at debug level for every transition. Manual testing with 10+ workflow scenarios before Phase 2 sign-off. Consider formal state machine verification tools. |
| **Owner** | Lead Developer |
| **Status** | Open -- address in Phase 2 |

### Strategic Risks

#### R08: Claude CLI Breaking Changes
| Field | Value |
|-------|-------|
| **ID** | R08 |
| **Category** | Strategic |
| **Description** | Claude Forge depends on the `claude` CLI's `--output-format stream-json` interface. If Anthropic changes the CLI output format, command-line flags, or deprecates the CLI, Forge's core functionality breaks. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 4 (Major) |
| **Risk Score** | 12 |
| **Mitigation** | Abstract the CLI interface behind a `ProcessSpawner` trait. Parse output through a versioned parser that can handle multiple output formats. Pin to a known CLI version in documentation. Monitor Claude CLI releases and changelog. Maintain a compatibility test suite that runs against the actual CLI. If the CLI is deprecated, migrate to direct API calls (reqwest + Anthropic API). |
| **Owner** | Lead Developer |
| **Status** | Open -- active monitoring |

#### R09: Scope Creep from Reference Repo Absorption
| Field | Value |
|-------|-------|
| **ID** | R09 |
| **Category** | Strategic |
| **Description** | Absorbing 62 reference repos into ~33K LOC creates pressure to implement every feature from every repo. This could balloon the scope far beyond 32 weeks, resulting in a perpetually unfinished project. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 9 |
| **Mitigation** | Strict phase boundaries with exit criteria (MILESTONES.md). Each phase ships a usable product. Features not in the current phase are deferred, not cut. Weekly scope review: if a task was not planned for this sprint, it goes to the backlog. The 1,537-skill catalog is the absorber -- reference repo features become skills, not code. "Good enough" over "perfect." |
| **Owner** | Lead Developer |
| **Status** | Open -- continuous vigilance |

#### R10: Low Adoption / No Users
| Field | Value |
|-------|-------|
| **ID** | R10 |
| **Category** | Strategic |
| **Description** | Claude Forge may not attract users if the value proposition is unclear, setup is too complex, or alternatives (Claude Desktop, Cursor, Windsurf) are good enough. Development effort would be wasted. |
| **Likelihood** | 2 (Unlikely) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 6 |
| **Mitigation** | Ship after every phase -- get real user feedback early. Focus Phase 1-2 on unique differentiators (multi-agent orchestration, workflow automation, skill catalog). One-command install (`brew install`). Zero-config default experience. Document clear use cases that cannot be done with alternatives. Engage community during development (blog posts, demos). |
| **Owner** | Lead Developer |
| **Status** | Open |

### Operational Risks

#### R11: Data Loss from Unplanned Shutdown
| Field | Value |
|-------|-------|
| **ID** | R11 |
| **Category** | Operational |
| **Description** | If the Forge binary crashes (panic, OOM, SIGKILL) while the batch writer has unflushed events, those events are lost. Agent output that has not been persisted is gone. |
| **Likelihood** | 1 (Rare) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 3 |
| **Mitigation** | SQLite WAL mode provides crash recovery for committed transactions. Batch writer flushes every 2 seconds (max data loss window). Critical events (agent start/stop) are flushed immediately, not batched. Graceful shutdown handler flushes all pending events before exit. WebSocket events are the source of truth during active sessions -- the frontend has the data even if the backend has not flushed. Consider write-ahead to a journal file as an additional safety net. |
| **Owner** | Lead Developer |
| **Status** | Open -- mitigated by design |

#### R12: Development Environment Complexity
| Field | Value |
|-------|-------|
| **ID** | R12 |
| **Category** | Operational |
| **Description** | The project requires Rust, Node.js, pnpm, and multiple system libraries. Setting up the development environment may be painful, especially for contributors, slowing development velocity. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 9 |
| **Mitigation** | Use `mise` for version management (Rust, Node, pnpm in one `mise.toml`). Vendor all native dependencies (SQLite bundled, libgit2 vendored, rustls instead of OpenSSL). Provide a `scripts/setup.sh` that installs everything. Document setup in CONTRIBUTING.md with copy-paste commands. Consider a devcontainer for consistent environments. |
| **Owner** | Lead Developer |
| **Status** | Open -- address in Phase 0 |

#### R13: CI/CD Pipeline Fragility
| Field | Value |
|-------|-------|
| **ID** | R13 |
| **Category** | Operational |
| **Description** | A complex CI pipeline (Rust compilation, frontend build, 12 crate tests, E2E tests) may be slow and flaky, blocking development velocity and eroding trust in the test suite. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 9 |
| **Mitigation** | Cache aggressively: Rust target directory, pnpm node_modules, sccache. Parallelize: unit tests and build run concurrently. E2E tests run nightly, not per-PR. Fail fast: lint and format check first (1 minute). Flaky test policy: fix within 24 hours or delete. Monitor CI duration weekly -- target < 10 minutes for PR checks. |
| **Owner** | Lead Developer |
| **Status** | Open -- address in Phase 0 |

### Resource Risks

#### R14: Single Developer Bus Factor
| Field | Value |
|-------|-------|
| **ID** | R14 |
| **Category** | Resource |
| **Description** | As a single-developer project, all knowledge, context, and momentum depend on one person. Illness, burnout, or other commitments could halt the project entirely. |
| **Likelihood** | 4 (Likely over 32 weeks) |
| **Impact** | 2 (Minor per incident, Major if prolonged) |
| **Risk Score** | 8 |
| **Mitigation** | Extensive documentation: architecture decisions, coding standards, roadmap. Clean code with doc comments on all public items. Automated tests as living documentation. Each phase produces a stable, shippable state (no half-finished branches). Regular commits with descriptive messages. Take breaks -- sustainable pace over sprint. If a contributor joins, the documentation enables them to ramp up quickly. |
| **Owner** | Lead Developer |
| **Status** | Open -- accepted risk |

#### R15: Rust/Svelte 5 Learning Curve for Contributors
| Field | Value |
|-------|-------|
| **ID** | R15 |
| **Category** | Resource |
| **Description** | The combination of Rust (ownership, lifetimes, async) and Svelte 5 (runes, new event handling) creates a high barrier for potential contributors. Most developers know one or the other, not both. |
| **Likelihood** | 2 (Unlikely to matter near-term) |
| **Impact** | 2 (Minor) |
| **Risk Score** | 4 |
| **Mitigation** | Backend and frontend are cleanly separated -- a contributor can work on one without the other. Coding standards document covers patterns specific to this project. Example code in every crate's tests. "Good first issue" labels on isolated, well-documented tasks. Svelte 5 runes are documented with before/after examples in CODING_STANDARDS.md. |
| **Owner** | Lead Developer |
| **Status** | Open -- low priority |

#### R16: Dependency Maintenance Burden
| Field | Value |
|-------|-------|
| **ID** | R16 |
| **Category** | Resource |
| **Description** | With 30+ Rust crates and 15+ npm packages, keeping dependencies updated (security patches, breaking changes, deprecations) consumes ongoing effort. A security vulnerability in a transitive dependency could force emergency updates. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 2 (Minor for routine, Moderate for security) |
| **Risk Score** | 6 |
| **Mitigation** | `cargo deny` in CI for vulnerability scanning. Dependabot or Renovate for automated update PRs. Pin major versions, accept minor/patch updates. Quarterly "dependency day" for major version updates. Minimize total dependency count (see dependency policy in TECH_STACK.md). Use `cargo audit` weekly. |
| **Owner** | Lead Developer |
| **Status** | Open |

#### R17: Burnout from 32-Week Solo Sprint
| Field | Value |
|-------|-------|
| **ID** | R17 |
| **Category** | Resource |
| **Description** | A 32-week development plan executed solo is grueling. Maintaining motivation, code quality, and health over 8 months is the single biggest risk to project completion. |
| **Likelihood** | 5 (Almost Certain without mitigation) |
| **Impact** | 4 (Major) |
| **Risk Score** | 20 |
| **Mitigation** | **Phase-based shipping:** each phase produces something usable. Celebrate milestones. **Sustainable pace:** 6-hour focused days, not 12-hour grinds. **Buffer time:** 10% of timeline is buffer, not committed work. **Variety:** alternate between backend and frontend tasks within each sprint. **External accountability:** share progress publicly (blog, demos). **Hard stops:** no work on weekends. Take a 3-day break between phases. **Scope flexibility:** Phases 5-6 can be reduced or deferred if burnout is approaching. |
| **Owner** | Lead Developer |
| **Status** | Open -- highest priority risk |

#### R18: Estimation Accuracy
| Field | Value |
|-------|-------|
| **ID** | R18 |
| **Category** | Resource |
| **Description** | The sprint plan estimates may be wildly inaccurate. Complex features (workflow engine, WASM plugins, swim lane visualization) could take 2-3x longer than estimated, blowing the timeline. |
| **Likelihood** | 2 (Unlikely for small tasks, Possible for complex ones) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 6 |
| **Mitigation** | Track actual vs. estimated hours every sprint. Use rolling 3-sprint velocity to adjust future estimates. If a task exceeds 2x its estimate, stop and re-scope. Each phase has buffer days for overruns. Cut scope from later phases rather than extending the timeline. The sprint plan is a guide, not a contract -- adjust based on reality. |
| **Owner** | Lead Developer |
| **Status** | Open |

#### R19: Anthropic API/Pricing Changes
| Field | Value |
|-------|-------|
| **ID** | R19 |
| **Category** | Strategic |
| **Description** | Anthropic could change API pricing, rate limits, or terms of service in ways that affect Forge's cost tracking accuracy or make heavy multi-agent usage prohibitively expensive. |
| **Likelihood** | 2 (Unlikely to be breaking) |
| **Impact** | 2 (Minor) |
| **Risk Score** | 4 |
| **Mitigation** | Cost tracking uses a configurable pricing table (not hardcoded). Users can update model prices in settings. Budget enforcement prevents surprise bills. Cost alerts warn before limits are hit. If pricing changes dramatically, the workflow/skill system helps users be more efficient (fewer tokens per task). |
| **Owner** | Lead Developer |
| **Status** | Open -- low priority |

#### R20: PTY/Terminal Cross-Platform Issues
| Field | Value |
|-------|-------|
| **ID** | R20 |
| **Category** | Technical |
| **Description** | Embedded terminal (Phase 6) requires PTY allocation, which differs between macOS and Linux. Signal handling, terminal resizing, and shell integration may behave inconsistently across platforms. |
| **Likelihood** | 3 (Possible) |
| **Impact** | 3 (Moderate) |
| **Risk Score** | 9 |
| **Mitigation** | Use the `portable-pty` crate for cross-platform PTY abstraction. Test on both macOS and Linux in CI. Start with basic terminal functionality (shell, input, output) before adding advanced features (resize, 256-color). Fall back gracefully if PTY allocation fails (show error message, suggest using external terminal). Terminal is Phase 6 -- by then, platform issues from Phases 0-5 will have informed the approach. |
| **Owner** | Lead Developer |
| **Status** | Open -- address in Phase 6 |

---

## Top 5 Risks -- Detailed Mitigation Plans

### 1. R17: Burnout (Score: 20)

**Why it is the top risk:** A solo 32-week project has a ~70% chance of burnout without active mitigation. Once burnout hits, code quality drops, bugs increase, motivation disappears, and the project stalls indefinitely.

**Detailed mitigation:**

| Strategy | Implementation |
|----------|---------------|
| Sustainable pace | Max 6 focused hours/day, 5 days/week. No weekends. |
| Phase celebrations | After each milestone, take 1-2 days off. Share the achievement publicly. |
| Progress visibility | Maintain a public changelog or blog. External validation sustains motivation. |
| Task variety | Within each sprint, alternate: backend morning, frontend afternoon. Never grind on one thing for a full day. |
| Scope safety valve | Phases 5 (Plugins) and 6 (Dev Environment) are deferrable. If burnout approaches in Phase 3-4, reduce scope of later phases. Ship what is done as 1.0. |
| Physical health | Exercise before coding. Stand/walk hourly. No late-night sessions. |
| Mental health | If motivation drops for 3+ consecutive days, take a full week off. The project will survive a pause. |

**Indicators to watch:**
- Commit frequency dropping below 3/week
- Test coverage declining
- Skipping code review (committing without review)
- Dreading opening the project
- Working past 7pm regularly

### 2. R04: MCP Protocol Instability (Score: 16)

**Why it is critical:** MCP is a core integration point. If the protocol changes fundamentally, all MCP tools and client connections break. Rework could cost 2-4 weeks.

**Detailed mitigation:**

| Strategy | Implementation |
|----------|---------------|
| Protocol abstraction | `McpMessage` and `McpResponse` types wrap raw JSON-RPC. Tool implementations work with domain types, not wire format. Protocol changes only affect the `forge-mcp` crate's message layer. |
| Version negotiation | Initialize handshake includes protocol version. Server supports the pinned version and optionally newer versions. |
| Monitoring | Subscribe to the MCP specification repository for releases. Review changes within 48 hours of publication. |
| Compatibility tests | Maintain a test suite that validates protocol compliance against the spec. Run monthly against latest spec. |
| Fallback plan | If MCP is abandoned, the tool registry and execution engine remain. Only the transport layer changes (MCP -> HTTP API or gRPC). |

### 3. R01: Wasmtime Binary Size (Score: 12)

**Why it matters:** A 50+ MB binary feels heavy for a developer tool. Downloads are slow, disk usage is high, and it signals bloat.

**Detailed mitigation:**

| Strategy | Implementation |
|----------|---------------|
| Feature flag | `forge-plugins` crate is behind `features = ["plugins"]`. Default build includes it; `--no-default-features` excludes it. |
| Wasmtime flags | Only enable `cranelift` backend (disable `winch`). Disable debug info in Wasmtime. |
| Build profile | `codegen-units = 1`, `lto = "thin"`, `strip = true`, `panic = "abort"` in release profile. |
| Measurement | Add binary size check to CI: `ls -la target/release/claude-forge`. Fail if > 50 MB. Track size over time. |
| Two distributions | Offer `claude-forge` (full, ~35 MB) and `claude-forge-lite` (no plugins, ~23 MB). |

### 4. R03: libgit2 Edge Cases (Score: 12) and R08: Claude CLI Changes (Score: 12)

**Combined mitigation (both are external dependency risks):**

| Strategy | Implementation |
|----------|---------------|
| Abstraction layers | Both `GitRepo` and `ProcessSpawner` are traits. Implementations can be swapped without changing business logic. |
| Graceful degradation | If a git operation fails on an edge case, return a structured error with a message like "Unsupported repository configuration: [details]. Try using git CLI directly." |
| CLI output parsing | Versioned parser modules (`v1_parser`, `v2_parser`) for Claude CLI output. New format detected -> log warning, attempt parse with latest parser, fall back to raw text. |
| Test repos | Maintain 20+ test repos covering edge cases: empty repo, detached HEAD, submodules, bare repo, shallow clone, LFS files, large history, merge conflicts. |
| Version pinning | Document minimum/maximum supported versions of `claude` CLI and `libgit2`. |

### 5. R07: Workflow State Machine Bugs (Score: 12)

**Why it matters:** Workflow bugs are user-visible and trust-destroying. A workflow that skips steps, hangs, or produces wrong results makes the system unreliable.

**Detailed mitigation:**

| Strategy | Implementation |
|----------|---------------|
| Formal state machine | Define all states and valid transitions in a table. Code enforces: transition not in table -> error. |
| Exhaustive tests | One test per (State x Event) combination. Invalid transitions tested too (verify rejection). |
| Property testing | Use `proptest` to generate random workflow shapes (varying step counts, nesting depths, error injection). Verify invariants: every workflow terminates, output count equals completed step count. |
| Timeouts | Every state has a maximum duration. If exceeded, transition to Failed with timeout error. No infinite hangs. |
| Debug logging | Every state transition emits a `tracing::debug!` event with from-state, to-state, and trigger. Workflow debugging is possible by reading the log. |
| Canary testing | Before Phase 2 sign-off, run 100 random workflows and verify all terminate correctly with expected outputs. |

---

## Risk Review Cadence

| When | Activity |
|------|----------|
| **Sprint start** | Review top 5 risks. Update likelihood/impact if new information available. |
| **Phase boundary** | Full register review. Close resolved risks. Add new risks discovered during the phase. |
| **After incident** | Post-mortem. If caused by a known risk, update mitigation. If new risk, add to register. |
| **Monthly** | Review dependency vulnerabilities (`cargo deny`, `npm audit`). Update R16 status. |

---

## Risk Status Definitions

| Status | Meaning |
|--------|---------|
| **Open** | Risk is identified and being monitored. Mitigation not yet implemented. |
| **Mitigating** | Mitigation actions are in progress. |
| **Mitigated** | Mitigation is in place. Risk remains but impact/likelihood is reduced. |
| **Closed** | Risk is no longer relevant (resolved, avoided, or accepted). |
| **Occurred** | Risk materialized. Tracking the response. |
