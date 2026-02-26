# Reference Code Methodology

> How we use the 62 reference repos and the 26-feb enhancement repos: approach, rules, and where it’s documented.

---

## Approach in one sentence

We **absorb interface and behavior** from reference code, then **implement idiomatically in Rust (and Svelte)**. We do **not** port line-by-line or copy code; we treat reference repos as **design and pattern sources**, not as a codebase to fork.

---

## Core principles

| Principle | Meaning |
|-----------|--------|
| **Reference, not source** | 62 repos and the old Forge prototype are **reference material**. We build from scratch; we don’t refactor or copy their code. |
| **Use, don’t copy** | Use reference code for: interfaces, behaviors, data shapes, event flows, ADRs. Do **not** copy: implementation details, language-specific patterns, variable names, or architecture that doesn’t fit our stack. |
| **Implement in our stack** | All production logic is written in Rust (backend) and Svelte (frontend). We map reference patterns to our bounded contexts, traits, DB schema, and API/MCP. |
| **Classify before absorbing** | For each pattern we decide: **Data** (presets, skills, configs), **Logic** (algorithms, state machines), or **UI** (components, layouts). Data we can ingest; Logic/UI we reimplement from the contract. |

---

## Absorption pipeline (5 phases)

The full process for turning a reference repo’s patterns into Forge features is in **07-methodology/ABSORPTION_PIPELINE.md**. Summary:

| Phase | Goal | Key output |
|-------|------|------------|
| **1. ANALYZE** | Understand what’s worth absorbing and how it classifies. | Analysis: what we absorb, Data/Logic/UI, size (S/M/L/XL), dependencies. |
| **2. EXTRACT** | Capture **interface** only: inputs, outputs, behaviors, data shapes, events. | Extraction doc: contracts, types, event flows, integration points. **No** code copy. |
| **3. DESIGN** | Map to Forge: bounded context, Rust traits/types, DB, API, MCP. | Design doc or PR: traits, schema, endpoints, MCP tools; ADR if needed. |
| **4. IMPLEMENT** | Write Rust + Svelte + tests. | Code that satisfies the design and passes tests. |
| **5. VALIDATE** | Verify behavior, interfaces, and docs. | Green tests, updated docs, acceptance criteria met. |

**Extraction rules (from pipeline):**
- **DO** extract: interfaces, behaviors, data shapes, event names, error conditions.
- **DO NOT** extract: implementation details tied to the source language.
- **DO NOT** copy: code, comments, or architecture that is an artifact of the source stack.
- **ALWAYS** note: bugs, limitations, or design mistakes in the reference that we should avoid.

---

## Tier model (62 reference repos)

From **REFERENCE_REPOS.md**:

| Tier | How we use the repo |
|------|----------------------|
| **Tier 1** | Extract code, patterns, or architecture **directly** into Forge (after going through the pipeline: interface → design → implement in Rust). |
| **Tier 2** | **Study** for design patterns; **adapt** concepts. No direct code reuse; inform our design. |
| **Tier 3** | Reference only; consult when implementing a related feature. |

So “extract” in Tier 1 still means: extract the **interface and behavior**, then implement in our stack — not paste their code.

---

## 26-feb enhancement repos

From **08-reference/TREND_26FEB_ENHANCEMENT_MAP.md** and **PHASE1_DESIGN_NOTES.md**:

| Use | How |
|-----|-----|
| **Design reference** | e.g. **claude-flow** ADRs and plugin/MCP design — inform our forge-process and MCP design; we don’t reuse their TypeScript. |
| **Skill content** | e.g. **Agent-Skills-for-Context-Engineering**, **superpowers** — ingest as skill catalog content; implement runtime (e.g. evaluation, TDD workflow) in Rust where needed. |
| **Pattern study** | e.g. **deer-flow** (sub-agent, sandbox), **ruvector** (vector search) — study then implement our own version in Rust. |

Suggested order: claude-flow + Agent-Skills (+ superpowers) first for design and skills; then ruvector, Scrapling in Phase 2.

---

## How this shows up in agent prompts

- We tell agents: **“Use X as design reference (not code reuse)”** — e.g. claude-flow for process/MCP.
- We point to **TREND_26FEB_ENHANCEMENT_MAP** and **PHASE1_DESIGN_NOTES** so implementation aligns with chosen references.
- We say **“follow existing patterns in the codebase”** (e.g. AgentRepo, EventBus) so new code matches Forge’s architecture, not the reference’s.

---

## Where it’s documented

| Topic | Document |
|-------|----------|
| Full 5-phase pipeline | **07-methodology/ABSORPTION_PIPELINE.md** |
| 62-repo registry and tiers | **REFERENCE_REPOS.md** |
| 26-feb repos and impact | **08-reference/TREND_26FEB_ENHANCEMENT_MAP.md** |
| Phase 1 design refs | **docs/PHASE1_DESIGN_NOTES.md** |
| North star (reference = material, not starting point) | **NORTH_STAR.md** |
