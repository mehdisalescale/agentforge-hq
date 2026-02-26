# Trend Repos (26-feb) → Claude Forge Enhancement Map

> How repos in `/Users/bm/cod/trend/26-feb` can enhance the app we are developing (Claude Forge).
> **Forge**: multi-agent Claude Code orchestrator — Rust/Axum + Svelte 5, single binary; absorbs 62 reference repos.

**Source**: Internal aggregation at `26-feb` (learning, pair coding, trend exploration).  
**Use**: Extract patterns, skills, and design; optionally add as reference repos or dependencies.

---

## App functionality summary

Summary of how the **26-feb** repos can enhance Claude Forge, by impact tier.

### High impact

| Repo | How it helps Forge |
|------|---------------------|
| **claude-flow** | **Patterns (no code reuse):** 15-agent mesh, plugin microkernel, MCP-first API, event sourcing, hybrid memory (SQLite + AgentDB). Use its ADRs and design to shape `forge-process`, plugins, and MCP. |
| **ruvector** | **Rust-native:** Vector/semantic search for "sessions like this" / "skills like this" (Forge has FTS5; ruvector adds embeddings). Optional memory backend; fits single-binary, offline-first. |
| **deer-flow** | **Patterns:** Sub-agent harness, skills, sandboxes, context engineering. Informs orchestration, skill design, and safety/worktree ideas. |
| **superpowers** | **Workflow + skills:** Spec → chunked design → implementation plan → subagent TDD. Directly supports your spec-driven workflow and quality gates; use as workflow templates and composable skills. |
| **Agent-Skills-for-Context-Engineering** | **Skill content:** Context fundamentals, degradation, compression, multi-agent patterns, tool design, evaluation, LLM-as-judge. Ingest as skill catalog content and reference for context optimization and quality gates. |

### Medium impact

| Repo | How it helps Forge |
|------|---------------------|
| **cc-switch** | **UX:** Multi-provider (Claude/Codex/Gemini) desktop patterns; compare with your Svelte UI and multi-provider routing. |
| **Scrapling** | **MCP:** "Fetch URL / scrape" as an MCP tool so agents can pull docs/runbooks into context. Implement in Rust or wrap as subprocess. |

### Lower / later

| Repo | How it helps Forge |
|------|---------------------|
| **SpacetimeDB** | **Later option:** Real-time DB (Rust); consider only if you add multi-client or real-time collaboration. |
| **moonshine** | **Optional:** Voice (transcription, intent) for "talk to Forge" or voice-triggered agents if you target that. |
| **learning/** | **Meta:** Minibooks per repo; use when deeply absorbing a 26-feb repo (e.g. `learning/claude-flow/MINIBOOK.md`). |

**Suggested order:** Use **claude-flow** and **Agent-Skills** (and **superpowers**) first for design and skill content; then consider **ruvector** for semantic search and **Scrapling** for an MCP scrape tool in Phase 2. The full map with absorption types, priorities, and next steps is in the sections below.

---

## Summary Table

| Repo | Stack | How it enhances Forge | Priority | Absorption type |
|------|--------|------------------------|----------|------------------|
| **claude-flow** | TypeScript | Orchestration patterns, plugin microkernel, MCP-first API, event sourcing, hybrid memory | High | Pattern / design |
| **ruvector** | Rust | Vector/semantic search for sessions & skills; optional memory backend | High | Code / crate |
| **deer-flow** | Python | Sub-agent harness, skills, sandbox, context engineering patterns | High | Pattern / skills |
| **superpowers** | Skills (multi-platform) | Spec→design→plan→TDD workflow; composable skills; RPI | High | Skills + workflow |
| **Agent-Skills-for-Context-Engineering** | Markdown/skills | Context engineering skills, evaluation, LLM-as-judge, multi-agent patterns | High | Skills (content) |
| **cc-switch** | Tauri + React | Multi-provider (Claude/Codex/Gemini) desktop UX; compare with Forge Svelte UI | Medium | UX patterns |
| **Scrapling** | Python, MCP | Web scraping as MCP tool for agent context (docs, runbooks) | Medium | MCP tool / pattern |
| **SpacetimeDB** | Rust | Real-time sync / multi-client backend option (Phase 3+) | Low | Future option |
| **moonshine** | Python / native | Voice layer (transcription, intent) for accessibility / voice-triggered agents | Low | Optional feature |
| **learning/** | Minibooks | Contributor onboarding for each repo — use when absorbing patterns | Meta | Reference |

---

## 1. claude-flow (V3)

**What it is**: Modular AI agent coordination — 15-agent hierarchical mesh, MCP-first API, plugin architecture, hybrid memory (SQLite + AgentDB), HNSW search, event sourcing, security/CVE fixes. TypeScript monorepo.

**How it enhances Forge**:

- **Orchestration**: Hierarchical mesh vs Forge’s current flat agents — inform `forge-process` and multi-agent coordination.
- **Plugin microkernel**: Plugin discovery, lifecycle, and MCP exposure — align with Forge’s planned skill/plugin system (`forge-mcp`, skill marketplace).
- **MCP-first API**: Every capability as MCP tools/resources — same goal as Forge’s dual-mode (embedded UI + MCP server).
- **Event sourcing**: Full audit trail; Forge already has `ForgeEvent` + `EventBus` + `BatchWriter` — compare contracts and replay semantics.
- **Hybrid memory**: SQLite + vector/AgentDB — pattern for adding semantic search or vector memory next to SQLite.
- **ADRs**: Use as reference for Phase 0/1 architecture decisions (e.g. single coordination engine, unified memory).

**Absorption**: Study design and ADRs; reimplement in Rust. No direct code reuse (stack mismatch). Use `learning/claude-flow/MINIBOOK.md` for contributor-level depth.

---

## 2. ruvector

**What it is**: Rust-native vector DB — HNSW, graph (Cypher), GNN, cognitive containers (RVF), local LLMs, WASM. AgenticDB-compatible; used by Claude Flow plugins.

**How it enhances Forge**:

- **Session/skill search**: Forge has FTS5 (Tantivy from claude-code-tools planned). RuVector adds *semantic* “sessions like this” or “skills similar to this” via embeddings.
- **Memory backend**: Optional long-term agent memory (e.g. important outcomes, decisions) with learning/improvement over time.
- **Rust-native**: Can be a crate dependency or embedded; fits single-binary story if desired.
- **Offline / local**: Aligns with Forge’s offline-first, no-cloud-required principle.

**Absorption**: Evaluate as optional dependency for `forge-db` or a dedicated `forge-memory` crate (Phase 2). Start with HNSW + embeddings for session/skill similarity.

---

## 3. deer-flow

**What it is**: Super agent harness — sub-agents, memory, sandboxes, extensible skills. Deep research → general “do almost anything” flow. Config-driven (YAML), LangChain-style.

**How it enhances Forge**:

- **Sub-agent orchestration**: Complements Forge’s agent presets and workflow engine; patterns for when to spawn sub-agents and how to pass context.
- **Skills & tools**: Extensible skill model and tool design — feed into Forge’s skill catalog and agent templates.
- **Sandbox & FS**: Sandbox and file-system patterns — relevant to safety and worktree isolation (Forge Git ops).
- **Context engineering**: Long-term memory and context handling — align with Forge’s session/context optimization and quality gates.

**Absorption**: Pattern study and skill/template ideas. No direct code (Python vs Rust). Use `learning/deer-flow/MINIBOOK.md` for extension points.

---

## 4. superpowers

**What it is**: Coding agent workflow — composable skills + instructions: spec → chunked design → implementation plan → subagent-driven TDD. For Claude Code / Cursor.

**How it enhances Forge**:

- **Spec-driven workflow**: Matches Forge’s “spec-to-code” from claude-code-spec-workflow (FEATURE_SOURCE_MAP) — brainstorming → design chunks → sign-off → implementation plan → subagent TDD.
- **Composable skills**: Auto-triggered skills (e.g. “help me plan”, “debug”) — model for Forge skill marketplace and auto-activation (e.g. skill-rules.json).
- **RPI / quality**: Refine–Plan–Implement and TDD emphasis — supports Forge quality gates and workflow engine.

**Absorption**: Import workflow as **skill content** and **workflow templates**; implement engine in Rust. Skills are platform-agnostic; workflow description can feed Forge’s workflow engine.

---

## 5. Agent-Skills-for-Context-Engineering

**What it is**: Open collection of skills for context engineering: context fundamentals, degradation, compression, multi-agent patterns, memory, tool design, evaluation, LLM-as-judge.

**How it enhances Forge**:

- **Skill content**: Ready-made skills (context-fundamentals, context-degradation, context-compression, multi-agent-patterns, memory-systems, tool-design, evaluation, advanced-evaluation, etc.) for Forge’s skill marketplace.
- **Context optimization**: Compression, compaction, masking — directly supports “60–80% token reduction” and session/context optimization in Forge.
- **Evaluation / LLM-as-judge**: Frameworks for quality gates and agent output evaluation — align with Forge’s QUALITY_GATES and safety/governance.
- **Multi-agent patterns**: Orchestrator, peer-to-peer, hierarchical — reference for `forge-process` and bounded contexts.

**Absorption**: Treat as **data repos** (REFERENCE_REPOS.md): ingest skill markdown/structured content into Forge’s skill catalog. Implement runtime behavior (compression, evaluation) in Rust where needed.

---

## 6. cc-switch

**What it is**: All-in-one assistant for Claude Code, Codex & Gemini CLI. Tauri desktop app (Vite + React).

**How it enhances Forge**:

- **Multi-provider UX**: Switching and configuring Claude Code / Codex / Gemini in one desktop app — compare with Forge’s multi-provider routing (claude-code-router) and future UI.
- **Desktop patterns**: Tauri + React vs Forge’s Svelte — windowing, tray, settings, CLI discovery.
- **Audience overlap**: Same users (Claude Code, Codex, Gemini) — Forge could eventually integrate or replace parts of such a switcher with “one binary” + embedded UI.

**Absorption**: UX and product patterns; no code reuse (different stack). Optional: list as Tier 2 “pattern study” in REFERENCE_REPOS if not already covered by claude_code_bridge / multi-provider.

---

## 7. Scrapling

**What it is**: Web scraping for the modern web — selection, fetchers, docs. Python; has MCP server.

**How it enhances Forge**:

- **MCP tool**: “Fetch URL / scrape” as MCP tool so Forge agents can pull docs, runbooks, or live pages into context.
- **Data ingestion**: Pipeline for ingesting external content into agent context or into Forge’s knowledge (e.g. for session preload).
- **Integration**: Forge is Rust — either call Scrapling as subprocess from MCP server or reimplement a minimal fetcher in Rust and adopt selection/API patterns from Scrapling.

**Absorption**: Define an MCP tool contract (e.g. `scrape_url`, `fetch_and_extract`); implement via subprocess or native Rust. Medium priority (Phase 2 MCP tool set).

---

## 8. SpacetimeDB

**What it is**: Real-time backend / DB — “development at the speed of light.” Rust, client SDKs.

**How it enhances Forge**:

- **Real-time sync**: Multi-client, live updates — relevant if Forge later supports collaborative or multi-user sessions (Phase 3+).
- **Rust ecosystem**: Same language as Forge — potential future option for real-time layer instead of or in addition to SQLite + WebSocket.

**Absorption**: Defer until multi-client or real-time collaboration is on the roadmap. Document as optional future backend in architecture docs.

---

## 9. moonshine

**What it is**: Voice AI toolkit — on-device, real-time transcription, diarization, intent recognition. Python / iOS / Android / Linux / Windows.

**How it enhances Forge**:

- **Voice layer**: “Talk to Forge” or voice-triggered agents — accessibility and hands-free use.
- **Intent**: Intent recognition could drive which agent or skill is invoked.

**Absorption**: Low priority unless Forge explicitly targets voice/accessibility. Could be an optional plugin or external integration (e.g. MCP client that sends transcribed + intent to Forge).

---

## 10. learning/ (minibooks)

**What it is**: Contributor minibooks per project — DDD, ADRs, core modules, data flow, extension points.

**How it enhances Forge**:

- **Onboarding**: When absorbing a 26-feb repo (e.g. claude-flow, ruvector, deer-flow), read the corresponding `learning/<repo>/MINIBOOK.md` for principled extension and design patterns.
- **Methodology**: Shared principles (DDD, ADRs, propose-before-execute) align with Forge’s methodology (e.g. QUALITY_GATES, DEVELOPMENT_PROCESS).

**Absorption**: Use as reference when doing deep dives; no code absorption.

---

## Suggested Next Steps

1. **Immediate (Phase 0/1)**  
   - Use **claude-flow** ADRs and plugin/MCP design as reference for `forge-process`, `forge-mcp`, and plugin architecture.  
   - Add **Agent-Skills-for-Context-Engineering** (and optionally **superpowers**) to REFERENCE_REPOS as Tier 2 “pattern + skill content” and ingest selected skills into the skill catalog when the catalog exists.

2. **Phase 2**  
   - Evaluate **ruvector** as optional crate for semantic session/skill search and optional memory backend.  
   - Define MCP tools for web context (inspired by **Scrapling**) and implement or wrap.  
   - Formalize spec→plan→TDD workflow (from **superpowers**) in the workflow engine and as skills.

3. **Later**  
   - **cc-switch**: UX comparison and multi-provider UI patterns.  
   - **SpacetimeDB**: Only if real-time multi-client is planned.  
   - **moonshine**: Only if voice/accessibility is a product goal.

4. **Process**  
   - When absorbing any 26-feb repo, open the matching `learning/<repo>/MINIBOOK.md` for contributor-level context and extension points.

---

## References

- Forge North Star: [NORTH_STAR.md](../NORTH_STAR.md)
- Feature sources: [FEATURE_SOURCE_MAP.md](FEATURE_SOURCE_MAP.md)
- Reference repo tiers: [REFERENCE_REPOS.md](../REFERENCE_REPOS.md)
- 26-feb root: `/Users/bm/cod/trend/26-feb/README.md`
