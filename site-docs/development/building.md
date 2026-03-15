# Building from Source

## Prerequisites

- Rust toolchain (stable channel via [rustup](https://rustup.rs/))
- Node.js 22+ and pnpm
- Claude Code CLI (for running agents)

## Build Steps

```bash
# Clone
git clone https://github.com/mehdisalescale/agentforge-hq.git
cd agentforge-hq

# Frontend (must build first — rust-embed needs frontend/build/)
cd frontend && pnpm install && pnpm build && cd ..

# Backend
cargo build --release

# Run
./target/release/forge
```

## Development Mode

```bash
# Backend with hot reload
cargo watch -x run

# Frontend dev server (separate terminal)
cd frontend && pnpm dev
```

## Testing

```bash
# All tests
cargo test

# Specific crate
cargo test -p forge-api
cargo test -p forge-db

# Zero warnings policy
cargo check  # must produce zero warnings
```

## Project Structure

```
agentforge-hq/
├── crates/              13 workspace crates
│   ├── forge-app/       binary entry point
│   ├── forge-api/       HTTP API + middleware
│   ├── forge-core/      events, errors, IDs
│   ├── forge-db/        SQLite, migrations, repos
│   ├── forge-process/   CLI spawn, stream parsing
│   ├── forge-safety/    circuit breaker, rate limiter
│   ├── forge-git/       worktree isolation
│   ├── forge-org/       company, department models
│   ├── forge-persona/   persona catalog
│   ├── forge-governance/ goals, approvals
│   ├── forge-agent/     agent model, presets
│   ├── forge-mcp/       MCP stubs
│   └── forge-mcp-bin/   MCP stdio server
├── frontend/            SvelteKit 5 + TailwindCSS 4
├── migrations/          SQLite migrations (0001-0012)
├── personas/            112 persona markdown files
├── skills/              10 skill markdown files
└── mkdocs.yml           this documentation site
```
