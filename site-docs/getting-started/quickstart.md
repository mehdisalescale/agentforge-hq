# Quick Start

## Download

```bash
# From GitHub Releases
curl -LO https://github.com/mehdisalescale/agentforge-hq/releases/latest/download/agentforge-macos-arm64
chmod +x agentforge-macos-arm64
./agentforge-macos-arm64
```

Open [http://127.0.0.1:4173](http://127.0.0.1:4173)

## Build from Source

```bash
# Prerequisites: Rust toolchain, Node.js 22+, pnpm
git clone https://github.com/mehdisalescale/agentforge-hq.git
cd agentforge-hq

# Frontend (must build first — rust-embed needs frontend/build/)
cd frontend && pnpm install && pnpm build && cd ..

# Backend
cargo build --release
./target/release/forge
```

## Prerequisites

- **Claude Code CLI** — must be installed and authenticated. Run `claude` in a terminal first to set up.
- **Rust toolchain** — for building from source (`rustup`, stable channel)
- **Node.js 22 + pnpm** — for building the frontend

## First Steps

1. **Create a company** — go to `/companies`, name it, set a budget
2. **Hire personas** — go to `/personas`, browse 100+ specialists, hire into your company
3. **View org chart** — go to `/org-chart` to see your team
4. **Run an agent** — go to the dashboard, select an agent, type a prompt

## Demo Data

On first launch, AgentForge seeds sample data:

- **Acme AI Corp** — a company with $500 budget
- 2 departments (Engineering, Product)
- 4 agents with org positions
- 3 goals (hierarchical)
- 1 pending approval

This lets you explore every page without setup.

## Data Storage

- **Database**: `~/.agentforge/forge.db` (SQLite WAL mode)
- **No cloud**: everything runs locally
- No data leaves your machine except Claude API calls made by agents
