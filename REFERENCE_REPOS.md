# Claude Forge -- Reference Repository Registry

> Canonical registry of all 61 repos. Tracks what to extract and absorption status.
> Source: `/Users/bm/claude-parent/refrence-repo/`
> All repos: `origin` = `git@github.com:zixelfreelance/<name>.git`, `upstream` = original author

---

## Absorption Tiers

- **Tier 1**: Extract code, patterns, or architecture directly into Forge
- **Tier 2**: Study for design patterns, adapt concepts
- **Tier 3**: Reference only, consult during implementation

---

## Tier 1: Direct Extraction (10 repos)

| # | Repo | Upstream | Extract What | Target Phase | Status |
|---|------|----------|-------------|-------------|--------|
| 1 | `claude-code-tools` | severity1 | Session search (Tantivy FTS), safety hooks, tmux-cli, vault, env-safe | Phase 0, 1 | Pending |
| 2 | `claude-code-router` | claude-code-router | Multi-provider routing, preset system, SSE stream, transformer pattern | Phase 1 | Pending |
| 3 | `claude-code-hooks-mastery` | claude-code-hooks-mastery | All 13 hook types, UV single-file scripts, builder/validator agents, status lines | Phase 1, 2 | Pending |
| 4 | `Claude-Code-Development-Kit` | Claude-Code-Development-Kit | 3-tier documentation system, MCP integration, sub-agent context injection | Phase 1 | Pending |
| 5 | `claude-code-infrastructure-showcase` | claude-code-infrastructure-showcase | Auto-skill activation hooks, skill-rules.json, 500-line modular pattern | Phase 2 | Pending |
| 6 | `claude-code-skills` | claude-code-skills | 38 production skills, marketplace manifest.json, YAML frontmatter, validation | Phase 2 | Pending |
| 7 | `awesome-claude-code-subagents` | awesome-claude-code-subagents | 127+ agent definitions, model routing (haiku/sonnet/opus), smart tool selection | Phase 2 | Pending |
| 8 | `claude-code-spec-workflow` | claude-code-spec-workflow | Spec-driven workflow (req->design->task->exec), steering docs, real-time dashboard | Phase 2 | Pending |
| 9 | `1code` | 1code | Multi-agent desktop patterns, worktree UI, Kanban board, Git UI, MCP, automations | Phase 3, 6 | Pending |
| 10 | `claude-code-action` | claude-code-action | GitHub Action YAML, PR/issue automation, @claude integration, code review flow | CI/CD | Pending |

---

## Tier 2: Pattern Study (10 repos)

| # | Repo | Upstream | Learn What | Relevant Phase |
|---|------|----------|-----------|----------------|
| 11 | `claude-code-plugins-plus-skills` | claude-code-plugins-plus-skills | 270+ plugins, 1500+ skills, CCPI package manager, Jupyter tutorials | Phase 2, 5 |
| 12 | `ralph-claude-code` | ralph-claude-code | Autonomous dev loop, exit detection, rate limiting, circuit breaker pattern | Phase 1 |
| 13 | `claude-code-templates` | claude-code-templates | 100+ agent templates, npx installer, web dashboard patterns | Phase 2 |
| 14 | `Claude-Code-Workflow` | Claude-Code-Workflow | JSON-driven multi-agent framework, 4-level workflows, CLI orchestration | Phase 2 |
| 15 | `claude_code_bridge` | claude_code_bridge | Split-pane terminal, multi-CLI (Claude+Codex+Gemini), low-token sync | Phase 6 |
| 16 | `claude-code-hooks-multi-agent-observability` | claude-code-hooks-multi-agent-observability | Real-time agent monitoring via hooks, event correlation | Phase 3 |
| 17 | `Claude-Code-Usage-Monitor` | Claude-Code-Usage-Monitor | Usage tracking with predictions, warnings, real-time display | Phase 3 |
| 18 | `claude-code-webui` | claude-code-webui | Web UI for CLI with streaming chat, compare with our approach | All |
| 19 | `claude-code-viewer` | claude-code-viewer | Web-based client, interactive features, session management | All |
| 20 | `claude-code-best-practice` | claude-code-best-practice | Practice patterns: skills, agents, memory, hooks, MCP, workflows | All |

---

## Tier 3: Reference (41 repos)

### Desktop / Web Clients & IDEs
| Repo | What's There |
|------|-------------|
| `idea-claude-code-gui` | IntelliJ plugin: Claude + Codex, diff, agents, skills, MCP |
| `CodexBar` | macOS menu bar: usage limits for Codex, Claude, Cursor, Gemini |
| `claude-code.nvim` | Neovim integration |
| `claude-code-ide.el` | Emacs integration |
| `claude-code-chat` | VS Code chat interface |

### Curated Lists & Guides
| Repo | What's There |
|------|-------------|
| `awesome-claude-code` | Master index of 350+ projects (use as discovery tool) |
| `claude-code-guide` | Community guide: install, tips, MCP, troubleshooting |
| `claude-code-tips` | 45 tips: status line, prompt slimming, worktrees |
| `claude-code-cheat-sheet` | Quick reference |
| `claude-code-mastering` | Learning resource |

### Config & Settings
| Repo | What's There |
|------|-------------|
| `claude-code-config` | Trail of Bits opinionated defaults |
| `claude-code-config2` | Personal config: rules, hooks, agents |
| `claude-code-settings` | Settings for vibe coding |
| `claude-code-showcase` | Example project config |
| `everything-claude-code` | Complete config collection (hackathon winner) |
| `my-claude-code-setup` | Starter template, memory bank system |

### Docs & Internals
| Repo | What's There |
|------|-------------|
| `claude-code-docs` | Local mirror of official docs |
| `claude-code-system-prompts` | All system prompts, token counts, changelog |
| `claude-code-reverse` | Visualize LLM interactions |
| `Claude-Code-Communication` | Messaging patterns |

### Remote, Proxy & Infra
| Repo | What's There |
|------|-------------|
| `claude-code-telegram` | Telegram bot with session persistence |
| `claude-code-hub` | API proxy: load balancing, monitoring |
| `Claude-Code-Remote` | Remote control via email, Discord, Telegram |
| `claude-code-proxy` | Run on OpenAI models (proxy layer) |
| `claude-code-mcp` | Claude Code as one-shot MCP server |

### Agents & Subagents
| Repo | What's There |
|------|-------------|
| `claude-code-subagents` | 100+ production-ready subagents |
| `claude-code-sub-agents` | Specialized full-stack subagents |
| `ClaudeCodeAgents` | QA agents |

### Skills & Plugins
| Repo | What's There |
|------|-------------|
| `claude-code-skill-factory` | Toolkit for building/deploying skills |
| `claude-code-tresor` | Utilities: skills, agents, commands, prompts |

### Workflows & Process
| Repo | What's There |
|------|-------------|
| `claude-code-workflows` | Workflow configs from AI-native startup |
| `claude-code-my-workflow` | Academic template: LaTeX/Beamer + R |

### Learning & Prompts
| Repo | What's There |
|------|-------------|
| `claude-code-prompt-improver` | Hook for improving prompts |
| `claude-code-pm-course` | PM course using Claude Code |
| `claude-code-requirements-builder` | Requirements building |

### Security & Misc
| Repo | What's There |
|------|-------------|
| `claude-code-security-review` | GitHub Action for AI security review |
| `claude-code-transcripts` | Transcript publishing tools |
| `edmunds-claude-code` | Edmunds-specific config |
| `claude-coder` | Autonomous coding agent (Kodu), VSCode extension |
| `claude-code-cookbook` | Recipes and examples |
| `codemcp` | Coding assistant MCP for Claude Desktop |

---

## Absorption Tracking

When a repo's patterns are absorbed into Forge:
1. Change its status from `Pending` to `Absorbed`
2. Note which Forge files/crates received the absorbed code
3. Record the session number where absorption happened
4. Keep the reference repo for future upstream updates

### Absorbed So Far
_None yet. Absorption begins with Phase 0._
