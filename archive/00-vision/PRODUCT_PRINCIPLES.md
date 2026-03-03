# Claude Forge: Product Principles

These principles govern every design decision, feature prioritization, and code review in Forge. When two goals conflict, these principles determine which wins. They are ordered by priority: earlier principles take precedence over later ones.

---

## Principle 1: Single Binary, Zero Config

**Statement:** Forge ships as one compiled binary with no external dependencies. `cargo install claude-forge` must produce a working system with embedded UI, SQLite database, MCP server, 100+ agent presets, and sane defaults -- requiring zero configuration files, no Docker, no Node.js runtime, no database server, and no reverse proxy.

**Why it matters:** The #1 reason developers abandon tools is installation friction. Every dependency is a potential failure point, a version conflict, a platform incompatibility. The 62 reference repos collectively require Node.js, Python, Docker, PostgreSQL, Redis, and various system libraries. Forge requires `cargo install` or a downloaded binary.

**In practice:**
- The Svelte frontend is compiled to static assets and embedded via `rust-embed` at build time
- SQLite with WAL mode provides persistence without a database server
- Default configuration is compiled into the binary; user overrides are optional
- Agent presets are embedded as static data, not downloaded at runtime
- The server starts on `localhost:4173` with no configuration required

**Anti-pattern:** "We need Redis for pub/sub." No. We use Tokio broadcast channels. "We need PostgreSQL for complex queries." No. We use SQLite with well-designed schemas. "Users need to create a config file before first run." Never. Defaults must work. Any feature that requires configuration before it provides value is a design failure.

---

## Principle 2: Safety Before Speed

**Statement:** Every feature that gives agents power must have a corresponding safety mechanism, and that safety mechanism must be enabled by default. Users may loosen safety controls; they should never have to tighten them.

**Why it matters:** AI agents that can execute shell commands, modify files, and interact with APIs are powerful but dangerous. A single runaway agent can delete a repository, push broken code to production, or generate a $2,000 API bill in minutes. The market's trust deficit in AI tooling is our biggest adoption barrier. We solve it structurally, not with warnings.

**In practice:**
- Circuit breakers halt agent execution after configurable thresholds (token spend, time, file modifications)
- Permission boundaries restrict which directories agents can modify, which commands they can run
- Cost tracking is always on, with per-agent and per-session budgets
- Rate limiting prevents token abuse and API exhaustion
- Approval gates pause execution for human review on destructive operations
- All agent actions are logged to an immutable audit trail in SQLite
- The `--dangerously-skip-safety` flag exists but is never the default and emits warnings

**Anti-pattern:** "Let's ship the feature now and add safety later." No. Safety is not a follow-up PR. If a feature cannot be made safe, it does not ship. "Users know what they're doing; they don't need guardrails." Users are humans who make mistakes at 2 AM. Our guardrails save them from costly errors, and they can always be loosened explicitly.

---

## Principle 3: MCP-First Integration

**Statement:** Every capability Forge exposes must be available as an MCP tool. The UI and CLI are clients of the same MCP surface. Forge operates both as a standalone application (Direction A) and as an MCP server that other tools consume (Direction B).

**Why it matters:** MCP is becoming the universal integration standard for AI tools. By making MCP the architecture rather than an add-on, Forge automatically integrates with Claude Code, VS Code Copilot, CI/CD pipelines, and any future MCP client. This dual-mode design doubles our addressable market without doubling our code.

**In practice:**
- Internal Rust functions expose their capabilities as MCP tool definitions
- The Svelte UI calls the same API endpoints that MCP clients call
- Adding a new feature means implementing the logic once and registering it as both an HTTP endpoint and an MCP tool
- MCP tool schemas are auto-generated from Rust types where possible
- Every MCP tool has a human-readable description suitable for LLM consumption

**Anti-pattern:** "This feature only works in the UI." Every UI feature must have an MCP equivalent. "We'll add MCP support for this later." MCP support is designed in from the start, not retrofitted. "The MCP interface is a simplified version of the real API." No. MCP tools have full capability; the UI is a convenience layer on top of the same surface.

---

## Principle 4: Every Feature Serves Three Interfaces

**Statement:** Features are designed to work across three interfaces: the embedded web UI (for interactive use), the MCP server (for programmatic integration), and the CLI (for scripting and automation). If a feature cannot serve all three, its design needs rethinking.

**Why it matters:** Different users consume Forge differently. A solo developer uses the UI for real-time monitoring. A CI/CD pipeline calls MCP tools. A power user writes shell scripts with CLI commands. Serving all three from the same backend ensures consistency and maximizes the investment in each feature.

**In practice:**
- Agent creation: UI form, MCP `forge_create_agent` tool, `forge agent create --name "reviewer"` CLI
- Session monitoring: UI WebSocket dashboard, MCP `forge_get_session_events` tool, `forge session tail <id>` CLI
- Safety configuration: UI settings panel, MCP `forge_set_circuit_breaker` tool, `forge safety set --max-cost 5.00` CLI
- Every HTTP endpoint returns JSON suitable for both UI consumption and CLI parsing

**Anti-pattern:** "This is a UI-only feature." There are no UI-only features. "CLI users don't need this." CLI users are often the most demanding power users. "The MCP version can be simplified." Simplified means incomplete; incomplete means users hit walls.

---

## Principle 5: Absorb, Don't Reinvent

**Statement:** When a reference repository has solved a problem well, absorb its pattern into Forge. Do not reinvent from scratch. Give credit. Preserve the insight while improving the implementation.

**Why it matters:** The 62 reference repos represent years of collective problem-solving. Reinventing their solutions wastes time and ignores hard-won knowledge. Our value is unification and polish, not novelty for its own sake. The best code is code someone else already debugged.

**In practice:**
- Before building a feature, audit the reference repos for existing implementations
- Document which repo inspired which feature in code comments and changelogs
- Preserve the architectural insight even when rewriting in Rust (a Python pattern's logic transfers even if the syntax does not)
- Maintain a traceability matrix: feature -> source repo(s) -> Forge implementation
- When a reference repo has multiple approaches, choose the one with the best safety/simplicity tradeoff

**Anti-pattern:** "I have a better idea for how to do this." Maybe, but first demonstrate that you understand why the reference repo did it their way. "We should build our own X from scratch." Only if no reference repo has solved it, or all existing solutions have fundamental flaws documented in our analysis. "The reference implementation is in Python so we can't use it." The language is irrelevant; the pattern is the value.

---

## Principle 6: Observability Is the Product

**Statement:** The ability to see what agents are doing, in real time, with structured data, is not a monitoring add-on -- it is the core value proposition that makes multi-agent systems usable by humans.

**Why it matters:** Multi-agent systems are opaque by default. Without observability, developers cannot tell which agent is stuck, which is consuming excessive tokens, which modified unexpected files, or why a workflow failed. The swim-lane timeline view is Forge's signature interaction: it turns the chaos of concurrent agent activity into a comprehensible visual narrative.

**In practice:**
- Every agent action emits a structured event to the event bus
- Events are persisted to SQLite in batches (50 events or 2-second intervals) for post-hoc analysis
- The WebSocket stream delivers events to the UI in real time (< 100ms latency)
- The swim-lane view shows concurrent agents as parallel tracks with time-aligned events
- Cost accumulation is visible in real time, per agent and per session
- File modifications are tracked with diffs, not just "file changed" notifications
- Token usage breakdowns (input/output/cache) are always visible

**Anti-pattern:** "Users can check the logs if something goes wrong." Logs are for debugging after failure; observability is for understanding during operation. "We'll add metrics later." If we cannot observe a feature in operation, we cannot know if it works correctly. "The event stream is too noisy." Then we need better filtering and summarization, not less data.

---

## Principle 7: Rust's Guarantees Are Product Guarantees

**Statement:** We chose Rust not for performance vanity but because its type system, ownership model, and error handling produce reliability guarantees that directly benefit users. These guarantees are product features, not implementation details.

**Why it matters:** A tool that orchestrates AI agents executing arbitrary code must itself be rock-solid. Memory safety prevents the orchestrator from crashing. Type safety prevents misconfigured agent definitions. Exhaustive error handling prevents silent failures. When Forge says an agent completed successfully, users can trust that claim because the compiler enforced the invariants.

**In practice:**
- All agent configuration types are validated at compile time via Rust's type system
- `Option<Option<T>>` patterns for nullable PATCH fields prevent partial-update bugs
- `DashMap` provides concurrent agent state without data races
- Error types are exhaustive; `unwrap()` is used only where panic is genuinely impossible
- SQLite operations use transactions; partial writes cannot corrupt the database
- WebSocket message types are strongly typed; malformed messages are rejected, not silently ignored

**Anti-pattern:** "Let's use `unwrap()` here, it'll never fail." Famous last words. Use `?` or handle the error. "We can use `unsafe` for performance." Only with a documented safety proof and a comment explaining why safe alternatives are insufficient. "The type system is too strict; let's use `serde_json::Value` everywhere." Dynamic typing in Rust defeats the purpose; model the domain with proper types.

---

## Principle 8: Progressive Disclosure of Complexity

**Statement:** Simple things must be simple. Complex things must be possible. The path from simple to complex must be a smooth ramp, not a cliff.

**Why it matters:** Forge's target audience ranges from solo developers running their first AI agent to enterprise teams orchestrating dozens of agents across repositories. A product that requires understanding workflow engines, circuit breaker patterns, and MCP schemas before "hello world" will fail. A product that cannot scale to complex workflows will be abandoned by power users.

**In practice:**
- First experience: install, launch, pick a preset, type a prompt, see results. Under 60 seconds.
- Second session: customize an agent, adjust safety settings, export a session.
- Third session: create a multi-agent workflow, set up approval gates, integrate with git.
- Advanced: write custom plugins, define workflow templates, connect to CI/CD pipelines.
- The UI shows essential controls by default; advanced options are accessible but not prominent.
- Error messages guide users toward solutions, not just describe problems.
- Documentation follows the same progression: quickstart, guides, reference, architecture.

**Anti-pattern:** "Let's put all the options on the main screen." Overwhelming new users drives them away. "Advanced users can figure it out." No, advanced users have higher standards, not more patience. "The getting-started guide is 15 pages." If it takes more than one page to get started, the product has failed, not the documentation.

---

## Principle 9: Local-First, Cloud-Optional

**Statement:** Forge runs entirely on the developer's machine by default. Code, prompts, sessions, and configuration never leave the local environment unless the user explicitly configures remote integrations. Cloud features are additive enhancements, never requirements.

**Why it matters:** Developers have legitimate concerns about sending proprietary code to third-party services. Many enterprises have strict data residency requirements. Local-first operation means Forge works offline (with local models), behind corporate firewalls, and in air-gapped environments. It also means zero latency for UI interactions and no monthly hosting costs.

**In practice:**
- SQLite database lives at `~/.claude-forge/forge.db` on the local filesystem
- The embedded UI is served from the binary; no CDN or remote assets
- Agent presets and plugin definitions are embedded in the binary
- Model API calls go directly to the provider (Anthropic, OpenAI); Forge never proxies through its own servers
- Optional cloud features (team dashboards, shared presets, remote MCP) are clearly labeled and require explicit opt-in
- All data export formats (JSON, Markdown) work offline

**Anti-pattern:** "Let's add telemetry to understand usage." Only with explicit opt-in, clear disclosure, and local-only as default. "The preset library should be fetched from our server." No. Presets are embedded. Optional remote preset repositories can supplement but never replace. "Users need to create an account." Never for core functionality. Accounts are for optional cloud features only.

---

## Principle 10: Composability Over Completeness

**Statement:** It is better to provide 20 composable primitives that can be combined into 1,000 workflows than to provide 1,000 pre-built workflows that cannot be modified. Forge is a platform for building agentic workflows, not just a collection of pre-built ones.

**Why it matters:** No matter how many presets and templates we ship, users will have workflows we did not anticipate. If Forge's features are composable -- agents can be chained, events can be filtered, safety rules can be combined, MCP tools can be composed -- users can build what they need. If features are monolithic, users hit walls.

**In practice:**
- Agents are composable: an agent's output can be another agent's input
- Events are filterable: any event property can be used as a trigger or filter condition
- Safety rules are combinable: circuit breakers, rate limits, and approval gates compose
- MCP tools can call other MCP tools, enabling meta-workflows
- Workflow definitions are data (TOML/JSON), not code, making them shareable and version-controllable
- The plugin system allows users to add new primitives, not just new configurations

**Anti-pattern:** "Let's build a dedicated 'code review workflow' feature." Better: build composable agents (writer, reviewer, tester) and a workflow engine that lets users compose them. "This feature works great but can't be extended." Unextendable features become technical debt when users need variations. "The workflow engine only supports linear pipelines." Real workflows branch, loop, fan out, and converge. The engine must handle DAGs, not just sequences.

---

## Applying the Principles

When making a decision, ask:

1. **Does this add a dependency?** (Principle 1: Single binary)
2. **Is this safe by default?** (Principle 2: Safety first)
3. **Does this work via MCP?** (Principle 3: MCP-first)
4. **Does this serve all three interfaces?** (Principle 4: Three interfaces)
5. **Has someone already solved this?** (Principle 5: Absorb)
6. **Can users see what's happening?** (Principle 6: Observability)
7. **Are we using Rust's type system to prevent bugs?** (Principle 7: Rust guarantees)
8. **Is the simple case still simple?** (Principle 8: Progressive disclosure)
9. **Does this work offline?** (Principle 9: Local-first)
10. **Can users compose this with other features?** (Principle 10: Composability)

If a proposed feature violates any principle, document the tradeoff explicitly and get team consensus before proceeding. Principles 1-3 (single binary, safety, MCP) are near-inviolable. Principles 4-10 allow more judgment in edge cases.
