# Research Findings — Community Repos & Recent Projects

> **Date:** 2026-03-05
> **Sources:** `/Users/bm/claude-parent/reference-map/` (61 repos, 13 categories) + `/Users/bm/cod/trend/4-march/` (6 repos)
> **Purpose:** Identify patterns worth adopting in forge v0.5.0+

---

## Source Inventory

### reference-map (61 repos across 13 categories)

Claude Code community repos catalogued in claude-parent. Categories: GUIs, Workflows, Hooks/Observability, Skills/Plugins, Agents, Tools, Remote/Infrastructure, CI/CD, Config, Templates, Extensions, Prompts, Showcases.

### 4-march repos (6 projects)

| Repo | What | Lang | Size |
|------|------|------|------|
| **codebuff** | AI coding agent with best-of-N, generator steps, context pruning | TypeScript | Large |
| **ReMe** | Advanced memory system — compaction, 3-type vector, hybrid search | Python | Medium |
| **agentscope** | Multi-agent orchestration framework — pipelines, MsgHub, plans | Python | Large |
| **agency-agents** | 70+ rich agent personas with frontmatter schemas | Markdown | Collection |
| **airi** | Plugin SDK with typed channel protocols | TypeScript | Large |
| **LMCache** | KV cache engine with Prometheus observability | Python | Medium |

---

## Tier 1 — High Impact (adopt in v0.5.0)

### 1. Best-of-N Parallel Implementation Selection
**Source:** codebuff (`agents/editor/best-of-n/`)

Spawn N agents with different strategy prompts for the same task. A selector agent reviews all N outputs as unified diffs, picks the best one, and extracts improvements from runners-up. Turns ConcurrentRunner from "parallel independent work" into "parallel competing approaches → best result."

**Key files:**
- `editor-multi-prompt.ts` — orchestrator spawning N implementors
- `best-of-n-selector2.ts` — selector comparing diffs, outputting `{implementationId, reason, suggestedImprovements}`
- `editor-implementor.ts` — individual implementor with dry-run edit tools

**Forge gap:** ConcurrentRunner treats sub-agent outputs as independent. No competition/selection mechanism.

---

### 2. Quality Gates (Critic-Fixer Loops)
**Source:** reference-map — `claude-code-my-workflow`, `ClaudeCodeAgents`

After an agent completes, a separate critic agent scores output (0-100). Below threshold → fixer agent re-runs with critic feedback. Repeats until threshold met or max iterations.

**Forge gap:** No post-execution quality validation. Agents can produce substandard output that gets accepted.

---

### 3. Context Pruner + Memory Compaction
**Source:** codebuff (`agents/context-pruner.ts`) + ReMe (`reme/reme_copaw.py`)

Two complementary patterns:
- **Context pruner** (codebuff): Summarize tool calls into one-liners ("Read files: src/main.rs, src/lib.rs"), 80/20 head/tail truncation for long text, ~3 chars/token estimation
- **Memory compaction** (ReMe): When tokens exceed threshold, auto-summarize old messages into structured checkpoint, write to daily log, replace with `<previous-summary>` block. Background async summarization. Tool result compaction for oversized outputs.

**Key files:**
- codebuff: `agents/context-pruner.ts` — `truncateLongText()`, `estimateTokens()`, `summarizeToolCall()`
- ReMe: `reme/reme_copaw.py` — `compact_memory()`, `summary_memory()`, `compact_tool_result()`
- ReMe: `reme/memory/file_based_copaw/compactor.py` — structured context checkpoints

**Forge gap:** No context window management. Long sessions with many sub-agents will hit limits.

---

### 4. Swim-Lane Observability Dashboard
**Source:** reference-map — `claude-code-hooks-multi-agent-observability`

Each agent gets a vertical column. Events appear as colored blocks with timestamps. Features: live pulse chart, tool emoji system, dual-color coding (app vs session identity), auto-scroll with manual override.

**Forge gap:** Dashboard shows flat event log. With 5+ parallel sub-agents, flat logs are unreadable. WebSocket already carries `agent_id` — just needs frontend grouping.

---

### 5. Cron Scheduler for Agent Runs
**Source:** reference-map — `claude-code-viewer`

Cron expression + agent_id + prompt → automatic execution. Enables nightly audits, periodic security scans, morning standup summaries.

**Forge gap:** Every run is manual. No concept of scheduled/deferred execution. Low implementation effort: new `schedules` table + tokio background task checking every 60s.

---

### 6. Sequential + Fanout Pipeline Primitives
**Source:** agentscope (`pipeline/_functional.py`)

Two composable primitives:
- `sequential_pipeline(agents, input)` — chain A → B → C, each output feeds next input
- `fanout_pipeline(agents, input)` — send same input to N agents concurrently, collect all

**Forge gap:** Only ConcurrentRunner (parallel independent) exists. No formal sequential chaining. These fill the empty Workflows page with real functionality.

---

### 7. Three-Type Memory System
**Source:** ReMe (`memory/vector_based/`)

Three categories with type-specific storage and retrieval:
- **Personal memory** — user preferences, habits (keyed by user_name)
- **Task memory** — execution patterns, success/failure (keyed by task_name)
- **Tool memory** — tool usage patterns, parameter tuning (keyed by tool_name)

**Key files:**
- `memory/vector_based/personal/`, `procedural/`, `tool_call/` subdirectories
- `reme_summarizer.py` — orchestrates extraction across types
- `reme_retriever.py` — orchestrates retrieval across types

**Forge gap:** All memories treated uniformly. Simple schema change (`memory_type` column) enables smarter retrieval.

---

## Tier 2 — Medium Impact (adopt in v0.5.0 or v0.6.0)

### 8. Typed Agent I/O Schemas
**Source:** codebuff (`agents/types/agent-definition.ts`)

Each agent declares: `inputSchema`, `outputMode` (last_message/all_messages/structured), `outputSchema`, `spawnableAgents`, `toolNames`, `spawnerPrompt`, `inheritParentSystemPrompt`.

**Forge gap:** Agent presets have name/description/model only. No typed I/O, no spawn hints, no tool restrictions.

---

### 9. Plan/SubTask Model with State Tracking
**Source:** agentscope (`plan/_plan_model.py`)

Formal `Plan` containing `SubTask` objects with states (todo/in_progress/done/abandoned), expected outcomes, actual outcomes, timestamps. Plans auto-refresh from subtask progress.

**Forge gap:** Workflows lack plan decomposition. Agents can't self-manage complex tasks by creating/tracking subtasks.

---

### 10. MsgHub Broadcast Communication
**Source:** agentscope (`pipeline/_msghub.py`)

Context manager for automatic message broadcasting among a group of agents. Any agent's output is forwarded to all other participants. Dynamic add/remove.

**Forge gap:** Agents are isolated. MsgHub enables debate, peer review, brainstorming patterns. Built on forge's existing broadcast channel infrastructure.

---

### 11. Predictive Usage Budgeting
**Source:** reference-map — `Claude-Code-Usage-Monitor`

Rolling-window P90 analysis to predict when user hits limit. Estimated spend by billing period end. Plan-tier awareness (Pro/Max5/Max20).

**Forge gap:** CostTracker tracks per-run but can't forecast. Analytics query over existing events table + `/api/usage/forecast` endpoint.

---

### 12. Auto-Activating Skills
**Source:** reference-map — `claude-code-infrastructure-showcase`, `claude-code-plugins-plus-skills`

Analyze prompt + working directory → auto-activate matching skills. `skill_rules` table with trigger conditions (file patterns, keywords, tech stack detection).

**Forge gap:** Skill assignment is static. Scanning for Cargo.toml → Rust skills, package.json → JS skills makes the system contextual.

---

### 13. Diff-Aware Security Scanning
**Source:** reference-map — `claude-code-security-review`

Post-execution: `git diff` → send changed lines to security review prompt → structured findings → block auto-commit if severity >= HIGH. Semantic analysis, not pattern matching.

**Forge gap:** `forge-safety` focuses on process-level controls, not code-level security analysis of agent output.

---

### 14. AI-as-Judge Eval Framework
**Source:** codebuff (`evals/`)

Replay real git commits as agent tasks. Multi-judge scoring (GPT, Gemini, Claude independently score on completion, code quality, overall). Median judge analysis for reporting.

**Key files:**
- `evals/buffbench/judge.ts` — multi-judge with `JudgingResultSchema`
- `evals/subagents/eval-planner.ts` — prompting agent that guides the coding agent

**Forge gap:** No way to measure agent quality. No benchmarks, no quality tracking over time.

---

### 15. OpenAPI Auto-Generated Docs
**Source:** reference-map — `claude-code-hub`

`utoipa` crate + Scalar UI at `/docs`. Self-documenting API from Axum route definitions.

**Forge gap:** No API documentation. External integrations must read source code. Near-zero implementation effort.

---

## Tier 3 — Lower Priority (v1.0+ or as needed)

| # | Pattern | Source | Notes |
|---|---------|--------|-------|
| 16 | Generator-based agent step control | codebuff | Deterministic agent behavior (force read-before-edit). High effort — needs Rust async state machine trait. |
| 17 | Conversation rewind/branching | reference-map | Git-branch semantics for conversations. Needs schema changes. |
| 18 | Multi-provider model routing | reference-map | Route tasks to different LLMs. Breaks Claude-only assumption. |
| 19 | Rich agent personas | agency-agents | Personality traits, success metrics, workflow steps in presets. Nice but cosmetic. |
| 20 | OpenTelemetry tracing | agentscope | Distributed tracing spans for agent/tool/LLM calls. Production ops. |
| 21 | Hybrid search (vector + BM25) | ReMe | 0.7 vector + 0.3 BM25. Needs SQLite FTS5 + vector extension. |
| 22 | Plugin SDK with protocols | airi | General plugin architecture beyond MCP. Deferred — MCP covers extension. |
| 23 | Prometheus metrics | LMCache | Runtime observability: hit rates, latency, error rates. Pairs with OpenTelemetry. |
| 24 | Session transcript export | reference-map | HTML/Markdown/Gist export. Data already in SQLite. |
| 25 | Kanban session view | reference-map | Drag-and-drop columns for Queued/Running/Done. Frontend-only. |
| 26 | Fine-grained git commits | codemcp | Per-edit commits instead of per-session. Extends forge-git. |
| 27 | Dual-condition exit gate | reference-map | Validate agent actually completed vs just exited. Small process runner addition. |
| 28 | Credential vault | reference-map | Secure storage for API keys agents need. Beyond env vars. |

---

## Key Insight

**codebuff** and **ReMe** are the two most relevant projects. codebuff shows how to make agents produce better output (best-of-N, quality gates, context management). ReMe shows how to make agents remember better (compaction, typed memory, hybrid search). Together they address forge's two biggest gaps: output quality and context/memory management.

**agentscope** contributes orchestration primitives (pipelines, MsgHub, plans) that make the Workflows page functional instead of a placeholder.

The **reference-map** repos contribute operational features (scheduling, observability, security scanning, auto-docs) that make forge production-ready.

---

## Relationship to Existing Borrowed Ideas

`docs/BORROWED_IDEAS.md` documents patterns adopted from DeerFlow and Claude-Flow for v0.2.0–v0.4.0. This document is the next layer: patterns from 67 additional sources for v0.5.0+.
