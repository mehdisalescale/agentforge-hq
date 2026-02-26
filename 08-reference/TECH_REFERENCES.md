# Technology References

> Links to all key technologies, reference repositories, specifications, and resources that inform Claude Forge.

---

## Core Technology Stack

### Rust Ecosystem

| Technology | Purpose in Forge | Documentation |
|-----------|-----------------|---------------|
| Rust | Primary backend language | https://doc.rust-lang.org/book/ |
| Cargo | Build system and package manager | https://doc.rust-lang.org/cargo/ |
| Axum 0.8 | HTTP/WebSocket server framework | https://docs.rs/axum/latest/axum/ |
| Tokio | Async runtime | https://tokio.rs/ |
| rusqlite | SQLite bindings (bundled) | https://docs.rs/rusqlite/latest/rusqlite/ |
| DashMap | Concurrent hash map | https://docs.rs/dashmap/latest/dashmap/ |
| rust-embed | Static file embedding | https://docs.rs/rust-embed/latest/rust_embed/ |
| serde / serde_json | Serialization framework | https://serde.rs/ |
| thiserror | Error type derivation | https://docs.rs/thiserror/latest/thiserror/ |
| tokio-tungstenite | WebSocket implementation | https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/ |
| tower | HTTP middleware framework | https://docs.rs/tower/latest/tower/ |
| tower-http | HTTP-specific middleware (CORS, compression) | https://docs.rs/tower-http/latest/tower_http/ |
| uuid | UUID generation | https://docs.rs/uuid/latest/uuid/ |
| chrono | Date/time handling | https://docs.rs/chrono/latest/chrono/ |
| tracing | Structured logging | https://docs.rs/tracing/latest/tracing/ |

### Frontend Ecosystem

| Technology | Purpose in Forge | Documentation |
|-----------|-----------------|---------------|
| Svelte 5 | Frontend framework (runes) | https://svelte.dev/docs/svelte |
| SvelteKit | Application framework + routing | https://svelte.dev/docs/kit |
| adapter-static | Static site generation for embedding | https://svelte.dev/docs/kit/adapter-static |
| TailwindCSS 4 | Utility-first CSS framework | https://tailwindcss.com/docs |
| Vite | Frontend build tool | https://vite.dev/guide/ |
| TypeScript | Frontend type system | https://www.typescriptlang.org/docs/ |

### Database

| Technology | Purpose in Forge | Documentation |
|-----------|-----------------|---------------|
| SQLite | Embedded database (all persistence) | https://www.sqlite.org/docs.html |
| SQLite FTS5 | Full-text search extension | https://www.sqlite.org/fts5.html |
| SQLite WAL mode | Concurrent read/write access | https://www.sqlite.org/wal.html |

### Build and Tooling

| Technology | Purpose in Forge | Documentation |
|-----------|-----------------|---------------|
| pnpm | Frontend package manager | https://pnpm.io/motivation |
| mise | Tool version manager | https://mise.jdx.dev/ |
| Node.js 22 | Frontend build runtime | https://nodejs.org/docs/latest-v22.x/api/ |

---

## Protocols and Specifications

### Model Context Protocol (MCP)

| Resource | URL |
|----------|-----|
| MCP Specification | https://spec.modelcontextprotocol.io/ |
| MCP Documentation | https://modelcontextprotocol.io/docs |
| MCP TypeScript SDK | https://github.com/modelcontextprotocol/typescript-sdk |
| MCP Rust SDK | https://github.com/modelcontextprotocol/rust-sdk |
| MCP Servers Repository | https://github.com/modelcontextprotocol/servers |
| JSON-RPC 2.0 Specification | https://www.jsonrpc.org/specification |

### Claude Code

| Resource | URL |
|----------|-----|
| Claude Code Documentation | https://docs.anthropic.com/en/docs/claude-code |
| Claude Code Hooks | https://docs.anthropic.com/en/docs/claude-code/hooks |
| Anthropic API Reference | https://docs.anthropic.com/en/api |
| Claude Models | https://docs.anthropic.com/en/docs/about-claude/models |

### Web Standards

| Resource | URL |
|----------|-----|
| WebSocket Protocol (RFC 6455) | https://datatracker.ietf.org/doc/html/rfc6455 |
| Server-Sent Events | https://html.spec.whatwg.org/multipage/server-sent-events.html |
| JSON Schema | https://json-schema.org/specification |
| OpenAPI 3.1 | https://spec.openapis.org/oas/v3.1.0 |

---

## All 61 Reference Repositories

### Category 01: Desktop and IDEs (8 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| 1code | https://github.com/1anthropic/1code | Electron, React 19, SQLite | -- |
| claude-code-viewer | https://github.com/nicekid1/claude-code-viewer | Hono, Vue 3, SQLite | -- |
| idea-claude-code-gui | https://github.com/nicekid1/idea-claude-code-gui | Java/Kotlin, IntelliJ SDK | -- |
| CodexBar | https://github.com/nicekid1/codexbar | Swift, WidgetKit | -- |
| claude-code.nvim | https://github.com/coder/claude-code.nvim | Lua, Neovim API | -- |
| claude-code-ide.el | https://github.com/nicekid1/claude-code-ide-el | Emacs Lisp, MCP | -- |
| claude-code-chat | https://github.com/nicekid1/claude-code-chat | TypeScript, VS Code | -- |
| claude-code-webui | https://github.com/nicekid1/claude-code-webui | TypeScript, React | -- |

### Category 02: Orchestration and Workflows (7 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| Claude-Code-Workflow | https://github.com/nicekid1/Claude-Code-Workflow | Node.js, Bun, SQLite | -- |
| ralph-claude-code | https://github.com/nicekid1/ralph-claude-code | Bash, BATS | -- |
| claude_code_bridge | https://github.com/nicekid1/claude_code_bridge | Python, tmux | -- |
| claude-code-workflows | https://github.com/nicekid1/claude-code-workflows | GitHub Actions | -- |
| claude-code-spec-workflow | https://github.com/nicekid1/claude-code-spec-workflow | Node.js, TypeScript | -- |
| claude-code-router | https://github.com/nicekid1/claude-code-router | TypeScript, pnpm | -- |
| Claude-Code-Development-Kit | https://github.com/nicekid1/Claude-Code-Development-Kit | Shell, Markdown | -- |

### Category 03: Hooks and Observability (3 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-hooks-multi-agent-observability | https://github.com/nicekid1/claude-code-hooks-multi-agent-observability | Bun, Vue 3, SQLite | -- |
| claude-code-hooks-mastery | https://github.com/nicekid1/claude-code-hooks-mastery | Python, Emacs Lisp | -- |
| Claude-Code-Usage-Monitor | https://github.com/nicekid1/Claude-Code-Usage-Monitor | Python, Rich | -- |

### Category 04: Templates, Skills, and Plugins (7 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-templates | https://github.com/nicekid1/claude-code-templates | Astro 5, Node.js | -- |
| claude-code-plugins-plus-skills | https://github.com/nicekid1/claude-code-plugins-plus-skills | pnpm, Astro 5 | -- |
| claude-code-skills | https://github.com/nicekid1/claude-code-skills | Skill.md, Python | -- |
| everything-claude-code | https://github.com/nicekid1/everything-claude-code | Markdown, Node.js | -- |
| claude-code-skill-factory | https://github.com/nicekid1/claude-code-skill-factory | Prompt templates | -- |
| claude-code-tresor | https://github.com/nicekid1/claude-code-tresor | Markdown, Python | -- |
| my-claude-code-setup | https://github.com/nicekid1/my-claude-code-setup | Markdown | -- |

### Category 05: Subagents and Agents (3 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-subagents | https://github.com/nicekid1/claude-code-subagents | Markdown | -- |
| claude-code-sub-agents | https://github.com/nicekid1/claude-code-sub-agents | Markdown, MCP | -- |
| ClaudeCodeAgents | https://github.com/nicekid1/ClaudeCodeAgents | Markdown | -- |

### Category 06: MCP and Tooling (3 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-mcp | https://github.com/nicekid1/claude-code-mcp | TypeScript, Node.js | -- |
| codemcp | https://github.com/nicekid1/codemcp | Python, Git | -- |
| claude-code-tools | https://github.com/nicekid1/claude-code-tools | Python, Rust | -- |

### Category 07: Remote and Infrastructure (4 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-hub | https://github.com/nicekid1/claude-code-hub | Next.js, PostgreSQL, Redis | -- |
| claude-code-telegram | https://github.com/nicekid1/claude-code-telegram | Python, SQLite | -- |
| Claude-Code-Remote | https://github.com/nicekid1/Claude-Code-Remote | Node.js, Express | -- |
| claude-code-proxy | https://github.com/nicekid1/claude-code-proxy | Python, LiteLLM | -- |

### Category 08: Automation and CI/CD (1 repo)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-action | https://github.com/anthropics/claude-code-action | TypeScript, Bun | -- |

### Category 09: Config and Settings (4 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-config | https://github.com/nicekid1/claude-code-config | JSON | -- |
| claude-code-config2 | https://github.com/nicekid1/claude-code-config2 | JSON, YAML | -- |
| claude-code-settings | https://github.com/nicekid1/claude-code-settings | Documentation | -- |
| claude-code-showcase | https://github.com/nicekid1/claude-code-showcase | Markdown, JSON | -- |

### Category 10: Curated Guides (7 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| awesome-claude-code | https://github.com/nicekid1/awesome-claude-code | Markdown | -- |
| awesome-claude-code-subagents | https://github.com/nicekid1/awesome-claude-code-subagents | Markdown | -- |
| claude-code-guide | https://github.com/nicekid1/claude-code-guide | Markdown | -- |
| claude-code-best-practice | https://github.com/nicekid1/claude-code-best-practice | Markdown | -- |
| claude-code-tips | https://github.com/nicekid1/claude-code-tips | Markdown | -- |
| claude-code-cheat-sheet | https://github.com/nicekid1/claude-code-cheat-sheet | Markdown | -- |
| claude-code-mastering | https://github.com/nicekid1/claude-code-mastering | Markdown | -- |

### Category 11: Docs and Internals (2 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-docs | https://github.com/nicekid1/claude-code-docs | Git, Bash | -- |
| claude-code-system-prompts | https://github.com/nicekid1/claude-code-system-prompts | Markdown | -- |

### Category 12: Prompts and Learning (3 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-prompt-improver | https://github.com/nicekid1/claude-code-prompt-improver | Python hooks | -- |
| claude-code-pm-course | https://github.com/nicekid1/claude-code-pm-course | Next.js | -- |
| claude-code-requirements-builder | https://github.com/nicekid1/claude-code-requirements-builder | Slash commands | -- |

### Category 13: Transcripts, Security, and Misc (8 repos)

| Repo | URL | Stack | Stars |
|------|-----|-------|-------|
| claude-code-transcripts | https://github.com/nicekid1/claude-code-transcripts | Python | -- |
| claude-code-security-review | https://github.com/nicekid1/claude-code-security-review | GitHub Actions, Python | -- |
| claude-code-my-workflow | https://github.com/nicekid1/claude-code-my-workflow | LaTeX, R | -- |
| claude-code-infrastructure-showcase | https://github.com/nicekid1/claude-code-infrastructure-showcase | Node.js, React | -- |
| claude-code-reverse | https://github.com/nicekid1/claude-code-reverse | JavaScript, Python | -- |
| Claude-Code-Communication | https://github.com/nicekid1/Claude-Code-Communication | Bash, tmux | -- |
| edmunds-claude-code | https://github.com/nicekid1/edmunds-claude-code | Next.js, Supabase | -- |
| claude-coder | https://github.com/nicekid1/claude-coder | VS Code Extension | -- |

Note: GitHub URLs are placeholders. Replace with actual repository URLs from the `refrence-repo/` directory remotes before publishing.

---

## Research and Articles

### Multi-Agent Systems

| Resource | Relevance |
|----------|-----------|
| [Gartner: Multi-Agent AI](https://www.gartner.com/en/articles/intelligent-agent-in-ai) | Market validation for multi-agent approach |
| [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) | Reference architecture for agent orchestration (Python-based) |
| [CrewAI Documentation](https://docs.crewai.com/) | Multi-agent framework comparison |
| [AutoGen Documentation](https://microsoft.github.io/autogen/) | Microsoft's multi-agent framework |

### Rust Web Development

| Resource | Relevance |
|----------|-----------|
| [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples) | Official Axum patterns and idioms |
| [Tokio Tutorial](https://tokio.rs/tokio/tutorial) | Async Rust patterns |
| [SQLite in Rust](https://docs.rs/rusqlite/latest/rusqlite/) | Database access patterns |
| [Rust Performance Book](https://nnethercote.github.io/perf-book/) | Performance optimization |

### Svelte 5

| Resource | Relevance |
|----------|-----------|
| [Svelte 5 Runes](https://svelte.dev/blog/runes) | Reactivity model documentation |
| [SvelteKit Docs](https://svelte.dev/docs/kit) | Application framework |
| [Svelte 5 Migration Guide](https://svelte.dev/docs/svelte/v5-migration-guide) | Svelte 4 to 5 changes |

### Design and UX

| Resource | Relevance |
|----------|-----------|
| [Refactoring UI](https://www.refactoringui.com/) | Design principles for developers |
| [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/) | Accessibility standards reference |
| [Information Architecture (O'Reilly)](https://www.oreilly.com/library/view/information-architecture-4th/9781491911679/) | IA principles |

### Circuit Breaker and Reliability

| Resource | Relevance |
|----------|-----------|
| [Martin Fowler: Circuit Breaker](https://martinfowler.com/bliki/CircuitBreaker.html) | Pattern definition |
| [Microsoft: Circuit Breaker Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker) | Implementation guidance |
| [Release It! (Nygard)](https://pragprog.com/titles/mnee2/release-it-second-edition/) | Stability patterns |

### SQLite and FTS

| Resource | Relevance |
|----------|-----------|
| [SQLite FTS5 Documentation](https://www.sqlite.org/fts5.html) | Full-text search implementation |
| [SQLite WAL Mode](https://www.sqlite.org/wal.html) | Concurrent access mode |
| [SQLite Best Practices](https://www.sqlite.org/np1queryprob.html) | Query optimization |

---

## Competitor Products

Understanding the competitive landscape informs Forge's differentiation.

| Product | Description | How Forge Differs |
|---------|-------------|-------------------|
| [Cursor](https://cursor.sh/) | AI-powered code editor | Forge is an orchestration platform, not an editor. MCP-compatible, so Forge can work alongside Cursor. |
| [Windsurf (Codeium)](https://codeium.com/windsurf) | AI IDE | Editor-focused; Forge focuses on multi-agent coordination and safety. |
| [GitHub Copilot Workspace](https://githubnext.com/projects/copilot-workspace) | Cloud-based AI development | Cloud-only, closed source. Forge is local-first, open source. |
| [Devin (Cognition)](https://devin.ai/) | Autonomous AI developer | Cloud-only, closed source, opaque safety model. Forge is transparent and locally auditable. |
| [Aider](https://aider.chat/) | Terminal AI pair programmer | Single-agent, terminal-only. Forge adds multi-agent orchestration, UI, and safety. |
| [Continue](https://continue.dev/) | Open-source AI code assistant | IDE plugin, not standalone. Forge is a complete platform. |
| [LangGraph Platform](https://www.langchain.com/langgraph-platform) | Multi-agent orchestration | General-purpose, Python-based. Forge is coding-specialized, Rust-based, single binary. |
| [CrewAI](https://www.crewai.com/) | Multi-agent framework | General-purpose, Python-based. Forge is coding-specialized with built-in UI. |
| [OpenHands](https://www.all-hands.dev/) | Open-source AI developer | Docker-based, browser-only. Forge is a native binary with embedded UI. |

---

## Standards and Specifications

| Standard | URL | Usage in Forge |
|----------|-----|---------------|
| Semantic Versioning 2.0 | https://semver.org/ | Release versioning |
| Conventional Commits | https://www.conventionalcommits.org/ | Commit message format |
| JSON Schema Draft 2020-12 | https://json-schema.org/draft/2020-12 | MCP tool input validation |
| TOML 1.0 | https://toml.io/ | Cargo.toml configuration |
| YAML 1.2 | https://yaml.org/spec/1.2.2/ | Configuration files |
| HTTP/1.1 (RFC 9110) | https://www.rfc-editor.org/rfc/rfc9110 | API protocol |
| WebSocket (RFC 6455) | https://www.rfc-editor.org/rfc/rfc6455 | Real-time streaming protocol |
