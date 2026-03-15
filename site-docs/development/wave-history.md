# Wave History

Detailed log of all development waves.

## Wave 1-2: Foundation

Built the skill system, task type detection, security scanner, process backend trait, and code review engine.

**Commits:**
- Skills system + 30 loaded skills
- SkillRouter and TaskTypeDetection middleware
- SecurityScan middleware with OWASP patterns
- ProcessBackend trait and ClaudeBackend adapter
- Code review engine with 6 specialist aspects

## Wave 3: Make It Real

Wired governance, cleaned UI, verified pages. 4 parallel agents.

| Agent | What |
|-------|------|
| W3-A | Sidebar cleanup (removed 4 dead links), run metadata panel, agent card badges |
| W3-B | Budget enforcement, goal injection, approval visibility via GovernanceMiddleware |
| W3-C | Session events endpoint, output viewing for past sessions |
| W3-D | Verified Skills, Analytics, Settings pages work end-to-end |

## Wave 4: Configure, Don't Inject

Transitioned from middleware injection to file-based configuration. 3 parallel agents.

| Agent | What |
|-------|------|
| W4-A | AgentConfigurator — generates CLAUDE.md + hooks.json per persona. Removed SkillInjection and TaskTypeDetection from chain (9 → 7 middlewares) |
| W4-B | HookReceiver — 3 endpoints (pre-tool, post-tool, stop). 3 new ForgeEvent variants. SecurityScan migrated to hook handler |
| W4-C | 3 MCP tools (classify_task, list_personas, get_budget). Updated NORTH_STAR.md and CLAUDE.md |

## Wave 5: Observable + MCP-Complete

Made the product observable and expanded MCP surface. 3 parallel agents.

| Agent | What |
|-------|------|
| W5-A | 6 more MCP tools → 19 total (governance, analytics, workforce) |
| W5-B | Analytics dashboard: summary cards, agent name resolution, company filter, cost trends |
| W5-C | Agent card stats (run count, cost, last run), CLI health check with UI banner |

## Metrics

- **Total waves**: 5
- **Total parallel agents**: ~17
- **MCP tools**: 0 → 19
- **Middleware chain**: 9 → 7
- **Frontend pages**: 16 → 12 (removed 4 shells, all remaining functional)
