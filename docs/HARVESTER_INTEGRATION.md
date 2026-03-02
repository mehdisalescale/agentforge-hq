# Harvester ↔ Forge Integration Assessment

> **Date:** 2026-03-02
> **Status:** Assessment complete — integration deferred to post-Sprint 2
> **Harvester repo:** `/Users/bm/smart-standalone-harvestor`
> **Forge repo:** `/Users/bm/claude-parent/forge-project`

---

## What the Harvester Is

Smart Standalone Harvester is a production-ready freelance job intelligence platform.

**Stack:** Python FastAPI + TypeScript (Chrome CDP scraper) + React dashboard + PostgreSQL

**Pipeline:** Discover → Score → Bid → Build MVP

### Core Components

| Component | Detail |
|-----------|--------|
| Chrome CDP scraper | Passively captures jobs from Upwork & Freelancer.com |
| 4-agent AI pipeline | Analyst → Architect → Strategist → Writer (Gemini/Ollama, not Claude) |
| 11 MCP tools | stdio server via FastMCP (search, score, bid, artifacts, settings) |
| MVP builder | Creates `~/mvp-projects/{slug}/CLAUDE.md` for Claude Code sessions |
| 30+ REST endpoints | Full CRUD for jobs, bids, builds, artifacts, settings, activity |
| 119 unit tests | Agents, scoring, artifacts, API, DB, MCP serialization |
| 8 PostgreSQL tables | jobs, job_scores, bids, builds, competitor_bids, similar_jobs, activity, settings |

### MCP Tools (11)

| Tool | Purpose |
|------|---------|
| `search_jobs` | Full-text search by query, platform, score, status |
| `get_job` | Fetch job + latest score + latest bid |
| `score_job` | Trigger scoring pipeline |
| `generate_bid` | Run full 4-agent bid pipeline |
| `approve_bid` | Mark bid as approved |
| `reject_bid` | Mark bid as rejected |
| `get_job_artifact` | Generate/retrieve 6 markdown artifact types |
| `list_job_artifacts` | List available artifacts for a job |
| `get_stats` | Total/scored/high-score/open counts |
| `get_settings` | Read user profile, model prefs, thresholds |
| `update_settings` | Update auto-bid threshold, keywords, search frequency |

### AI Agents

| Agent | Input | Output |
|-------|-------|--------|
| JobAnalystAgent | Job posting | problem_summary, core_tasks, hidden_risks, skill_match_score, is_good_fit |
| MVPArchitectAgent | Analysis | mvp_scope, deliverables, out_of_scope, estimated_days, tech_stack |
| BidStrategistAgent | Analysis + MVP + budget | pricing_model, recommended_rate, bid_range, confidence |
| ProposalWriterAgent | All prior outputs | proposal_text (150-250 words), timeline, differentiators |

**Scoring formula:** 30% skill match + 20% budget + 20% client quality + 15% competition + 15% AI leverage

**LLM providers:** Gemini Flash (scoring), Gemini Pro (proposals), Ollama as local fallback

### MVP Builder Flow

```
Approved bid → Create ~/mvp-projects/{slug}/ → Write CLAUDE.md (job context,
  problem summary, tasks, MVP scope, tech stack, deliverables, strategy) → git init
```

The `CLAUDE.md` file is the integration point with Claude Code — it provides structured briefing for interactive development sessions.

---

## Forge Integration Surface

### Current State (as of Sprint 0)

| Surface | What Exists |
|---------|-------------|
| REST API | `/api/v1/agents` CRUD, `/api/v1/run` (spawn + stream), `/api/v1/sessions`, `/api/v1/skills` (read-only stub), `/api/v1/workflows` (read-only stub) |
| MCP server | stdio JSON-RPC, 10 methods (agent/session CRUD + export), hand-rolled (Grade D) |
| Process spawn | `SpawnConfig` → `claude -p "prompt" --output-format stream-json`, stdout parsing, ProcessHandle lifecycle |
| Events | 20 ForgeEvent variants, EventBus broadcast, EventRepo persistence |
| Safety | RateLimiter (token bucket), CircuitBreaker (3-state FSM), budget tracking |
| Skills | Table exists with FTS5, SkillRepo has list/get, zero content or loader |
| Workflows | Table exists, WorkflowRepo has list/get, no execution logic |

### 9 Agent Presets

CodeWriter, Reviewer, Tester, Debugger, Architect, Documenter, SecurityAuditor, Refactorer, Explorer

---

## Previously Proposed Integration (Phase 3C)

From an earlier conversation, three layers were proposed:

1. **Multi-Agent Builder** — Use forge to orchestrate MVP builds with multiple specialized Claude agents
2. **Deeper Scoring** — Replace Gemini scoring with Claude via forge
3. **MCP Bridge** — Connect harvester's 11 MCP tools through forge as meta-orchestrator

---

## Assessment

### Why It's Premature

| Blocker | Detail |
|---------|--------|
| MCP is Grade D | Forge's MCP server is hand-rolled JSON-RPC, needs full rewrite with rmcp (Sprint 1) |
| No middleware chain | Run handler is a monolith, can't plug harvester into a pipeline (Sprint 2) |
| No skill system | Skills table empty, no loader or injection (Sprint 2) |
| No multi-agent coordination | Single process spawning only (Sprint 3) |
| Sprint 1 not started | 3 bugs unfixed, MCP rewrite not begun |

The harvester is ahead of forge in maturity for its domain. It has a working MCP server (FastMCP), working agent pipeline, working artifact system. Routing it through forge today would be a downgrade.

### What to Cut from Phase 3C

| Proposed | Verdict | Reason |
|----------|---------|--------|
| Replace Gemini scoring with Claude | **Cut** | Gemini Flash is cheaper, pipeline already tuned, no benefit |
| Deep MCP bridge before Sprint 1 | **Cut** | Forge's MCP is broken, fix it first |
| Custom harvester UI in forge | **Cut** | Harvester has its own React dashboard that works |

### What to Keep / Add

| Item | When | Detail |
|------|------|--------|
| External MCP tool provider | After Sprint 1 | Forge proxies to harvester's 11 tools, unified 21-tool surface |
| Freelance skill files | After Sprint 2 | `skills/freelance-bid.md`, `skills/mvp-build.md` encoding harvester workflows |
| HarvesterMiddleware | After Sprint 2 | Checks if run is freelance task, injects job context from harvester API |
| Worktree-based MVP builds | After Sprint 2 | Replace harvester's bare `git init` with forge's worktree isolation |
| Multi-agent MVP builder | After Sprint 3 | Coordinator decomposes build, sub-agents (CodeWriter, Tester, Reviewer) work in parallel |
| `ForgeEvent::ExternalToolInvoked` | Sprint 2-3 | New event type for external MCP proxy calls |
| Harvester health in forge | Sprint 2-3 | Forge's `/health` pings harvester API for unified status |
| Session ↔ job linking | Sprint 2-3 | Link forge session_id to harvester job_id for traceability |

---

## Integration Roadmap

```
Sprint 1 (v0.2.0)  — No harvester work. Fix bugs, rewrite MCP with rmcp.
Sprint 2 (v0.3.0)  — Register harvester as external MCP tool provider.
                      Create freelance skill files using harvester tools.
                      Add HarvesterMiddleware to pipeline.
Sprint 3 (v0.4.0)  — Multi-agent MVP builder using forge coordinator.
                      Parallel sub-agents in worktrees for builds.
                      Session ↔ job_id linking.
```

### Integration Architecture (Target: Sprint 3)

```
                    ┌─────────────────────────┐
                    │    Forge Dashboard UI    │
                    │  (real-time streaming)   │
                    └────────────┬────────────┘
                                 │ WebSocket
                    ┌────────────▼────────────┐
                    │      Forge API (Axum)    │
                    │  Middleware Chain:        │
                    │  Rate → CB → Skill →     │
                    │  Harvester → Spawn →     │
                    │  Persist → Cost          │
                    └──────┬──────────┬───────┘
                           │          │
              ┌────────────▼──┐  ┌────▼────────────┐
              │  Claude CLI   │  │  Harvester API   │
              │  (spawn per   │  │  (FastAPI)       │
              │   worktree)   │  │  11 MCP tools    │
              └───────────────┘  │  4 AI agents     │
                                 │  PostgreSQL      │
                                 └─────────────────┘
```

### Data Flow (Target: Sprint 3)

```
1. User prompt: "Build MVP for job #42"
2. Forge skill matcher → hits freelance-bid.md
3. HarvesterMiddleware → GET harvester/api/jobs/42 → inject context
4. Coordinator agent decomposes:
   a. CodeWriter → implement core features (worktree A)
   b. Tester → write tests (worktree B)
   c. Reviewer → review code (worktree C)
5. Each sub-agent reads CLAUDE.md briefing from harvester
6. Results aggregated, merged to main branch
7. Forge session linked to harvester job #42
```

---

## Bottom Line

The integration is valuable but premature. Forge needs Sprints 1-2 complete before it can be a proper orchestrator. The harvester works standalone today — there is no urgency to merge them. When forge reaches Sprint 2-3, the integration becomes natural because forge will have the middleware chain, skill system, and multi-agent coordination needed.

**Do not start integration work until forge Sprint 1 is shipped (v0.2.0).**
