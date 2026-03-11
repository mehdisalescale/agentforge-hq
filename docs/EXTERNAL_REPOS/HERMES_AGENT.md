## Hermes-Agent — External Runtime Overview

**Status**: Reference for the Runtime Adapter Agent and `forge-adapter-hermes`.  
**Upstream repo**: `https://github.com/NousResearch/hermes-agent`  
**Local path**: `hermes-agent/` (sibling to `forge-project/` in this workspace)

Hermes-Agent is a Python-based, tool-heavy agent runtime. AgentForge uses it as an optional
execution backend behind a clean `ProcessBackend::Hermes` abstraction, without taking over
Hermes’ config, providers, or skills.

---

### Core Concepts

- **AIAgent runtime**
  - Class: `AIAgent` in `run_agent.py`.
  - Key constructor fields (simplified):
    - `model: str` (e.g. `anthropic/claude-opus-4.6`).
    - `max_iterations: int` (iteration budget).
    - `enabled_toolsets: list[str]` / `disabled_toolsets: list[str]`.
    - `quiet_mode: bool` (suppress verbose logs).
    - `platform: str | None` (`"cli"`, `"telegram"`, etc.).
    - `session_id: str | None` (ties into SQLite session DB).
    - Flags for skipping context files or memory.
  - Call patterns:
    - `chat(message: str) -> str`: simple “just give me a reply” interface.
    - `run_conversation(user_message, system_message=None, conversation_history=None, task_id=None) -> dict`:
      full loop with messages, tool calls, and final response in a structured dict.
  - Loop structure (conceptual):
    - Build OpenAI-style `messages` list (`role: system/user/assistant/tool`).
    - Call OpenAI-compatible `chat.completions.create(model, messages, tools=...)`.
    - If tool calls are present, dispatch via `handle_function_call()` and append tool results.
    - Stop when no tool calls remain, or `max_iterations` / budget is hit.

- **Tools and toolsets**
  - Tool registry: `tools/registry.py`.
    - Each tool registers:
      - `name`, `toolset`, JSON schema, handler function, `check_fn`, `requires_env`.
      - **Contract**: handlers return a JSON string.
  - Discovery:
    - `_discover_tools()` in `model_tools.py` imports all tool modules.
    - `toolsets.py` defines built-in toolsets and `_HERMES_CORE_TOOLS`.
  - At runtime:
    - Registry exposes tool schemas for the LLM.
    - Tool availability is filtered via `check_fn` and environment.

- **Config and providers**
  - Config directory: `~/.hermes/`.
    - `config.yaml`: models, providers, terminal backend, memory, compression, display, etc.
    - `.env`: API keys and secrets (`OPENROUTER_API_KEY`, `OPENAI_API_KEY`, etc.).
    - `auth.json`: OAuth credentials.
    - `memories/`, `skills/`, `cron/`, `sessions/`, `logs/`.
  - Precedence:
    1. CLI arguments.
    2. `config.yaml`.
    3. `.env`.
    4. Built-in defaults.
  - Providers:
    - Built-in support for Nous Portal, OpenRouter, z.ai/GLM, Kimi/Moonshot, MiniMax, custom OpenAI-compatible endpoints.
    - Optional auxiliary models (vision, web_extract) configured under `auxiliary:` in `config.yaml`.

- **Terminal backends and isolation**
  - Controlled via `terminal:` in `config.yaml`:
    - `backend: "local" | "docker" | "ssh" | "singularity" | "modal" | "daytona"`.
    - `cwd`, `timeout`, container resource limits, and `docker_volumes`.
  - Git worktree isolation:
    - `worktree: true` creates per-session worktrees under `.worktrees/`.
    - `.worktreeinclude` controls which ignored files are copied.

- **Memory, skills, and context files**
  - Global state in `~/.hermes/`:
    - `memories/` directory (e.g. `MEMORY.md`, `USER.md`).
    - Optional `SOUL.md` defining the global persona.
  - Project-specific context:
    - `AGENTS.md`, `SOUL.md`, `.cursorrules`, `.cursor/rules/*.mdc` in the working directory.
    - Hermes automatically discovers and truncates these to fit within context limits.
  - Skills:
    - `~/.hermes/skills/` plus any workspace-level skills.
    - Each skill is exposed as a slash command (`/skill-name`).

- **MCP (Model Context Protocol)**
  - Optional extra (`pip install hermes-agent[mcp]`).
  - `mcp_servers` in `config.yaml` configures stdio or HTTP MCP servers.
  - Tools are registered as `mcp_{server_name}_{tool_name}` plus 4 utility tools per server.
  - Sampling support (`sampling/createMessage`) lets MCP servers ask Hermes to call LLMs.

---

### Current Task / Result Behavior (What Exists Today)

Hermes currently exposes its runtime via:

- **CLI / TUI**
  - `hermes` (TUI).
  - `hermes chat -q "Hello"` or `hermes chat --model <model> --toolsets "web,terminal,skills"`.
  - `hermes --continue` / `hermes --resume <session_id>` for resuming conversations.
- **Python API**
  - Direct import and use of `AIAgent` in Python code, calling `chat()` or `run_conversation()`.
- **Gateway and messaging**
  - A separate gateway component that wires Hermes into Telegram/Discord/Slack/etc.
  - That gateway is outside Forge’s direct concern; Forge only needs a process-level contract.

Hermes **does not** today expose a Forge-specific JSON protocol. For AgentForge, we will
interact with a **thin wrapper process** that embeds Hermes and speaks a stable JSON
contract over stdin/stdout, described below.

---

### Shared Runtime Adapter Contract (Trait + Types)

To keep Forge backend-agnostic, all backends (Claude, Hermes, OpenClaw, others) share a
common adapter trait and associated types. This section defines the *design contract*;
actual Rust code will be added later.

#### Trait and types

```rust
pub enum BackendKind {
    Claude,
    Hermes,
    OpenClaw,
}

pub enum BackendMessageRole {
    System,
    User,
    Assistant,
    Tool,
}

pub struct BackendMessage {
    pub role: BackendMessageRole,
    pub content: String,
    pub tool_name: Option<String>,
}

pub struct BackendTask<BackendConfig> {
    pub session_id: SessionId,
    pub agent_id: AgentId,
    pub backend: BackendKind,
    pub config: BackendConfig,
    pub system_prompt: String,
    pub user_prompt: String,
    pub conversation_snippet: Vec<BackendMessage>,
    pub max_iterations: u32,
    pub soft_token_budget: u32,
    pub hard_timeout: Duration,
}

pub enum BackendEvent {
    TextChunk { content: String },
    ToolStarted { name: String, args_preview: String },
    ToolFinished { name: String, ok: bool },
    CostUpdate { input_tokens: u32, output_tokens: u32, dollars: f32 },
    Warning { code: String, message: String },
}

pub struct BackendUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub cost_usd: f32,
}

pub struct BackendCompletion {
    pub final_text: String,
    pub usage: Option<BackendUsage>,
    pub tool_summaries: Vec<String>,
}

pub trait RuntimeAdapter {
    type Config;

    fn kind(&self) -> BackendKind;

    async fn health(&self) -> Result<(), AdapterError>;

    async fn execute(
        &self,
        task: BackendTask<Self::Config>,
        mut event_sink: impl FnMut(BackendEvent) + Send,
    ) -> Result<BackendCompletion, AdapterError>;
}

#[derive(thiserror::Error, Debug)]
pub enum AdapterError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("backend timeout after {0:?}")]
    Timeout(Duration),
    #[error("backend rejected task: {0}")]
    Rejected(String),
    #[error("backend protocol error: {0}")]
    Protocol(String),
    #[error("backend internal error: {0}")]
    Internal(String),
}
```

**Error semantics**:

- **Unavailable**: process cannot be spawned, HTTP unreachable, health check fails.
- **Timeout**: exceeded `hard_timeout` or backend-specific timeout.
- **Rejected**: validation / authorization failure; safe to show directly to user; not retried.
- **Protocol**: malformed/mismatched JSON, unexpected stdout, HTTP with invalid body.
- **Internal**: all other backend-side failures; may be eligible for limited retries.

---

### Hermes Adapter Contract (HermesBackendRequest / Response)

The Hermes adapter (`forge-adapter-hermes`) is defined in terms of:

- A Forge-side **config struct** (`HermesConfig`).
- A **JSON request/response contract** with a thin Hermes wrapper process.

#### HermesConfig (Agent-level configuration)

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct HermesConfig {
    /// Path to `hermes` executable or wrapper; default from FORGE_HERMES_COMMAND or "hermes".
    pub command: Option<String>,

    /// Working directory for the spawned process (optional).
    pub cwd: Option<PathBuf>,

    /// Hermes model name, e.g. "anthropic/claude-sonnet-4".
    pub model: Option<String>,

    /// Enabled Hermes toolsets.
    pub enabled_toolsets: Option<Vec<String>>,

    /// Disabled Hermes toolsets.
    pub disabled_toolsets: Option<Vec<String>>,

    /// Terminal backend ("local", "docker", "ssh", "modal", "daytona", "singularity").
    pub terminal_backend: Option<String>,

    /// Optional fixed Hermes session id to reuse across runs.
    pub hermes_session_id: Option<String>,

    /// When true, skip Hermes memory/context files for this run (stateless mode).
    pub stateless: Option<bool>,
}
```

This struct is intended to live inside `Agent.config_json` once adapters are implemented,
and to be patched via the AgentForge UI when selecting Hermes as a backend.

#### HermesBackendRequest (Forge → Hermes wrapper)

The thin wrapper process embeds Hermes and reads a single JSON request from stdin:

```json
{
  "version": "v1",
  "session_id": "8f3a6f76-1e12-4c3a-9f1a-0b0c9e62a123",
  "agent_id": "d3f1a911-4321-4e5f-9c0d-aabbccddeeff",
  "model": "anthropic/claude-sonnet-4",
  "max_iterations": 40,
  "soft_token_budget": 8000,
  "system_prompt": "System prompt assembled by Forge (persona, company context, skills).",
  "user_prompt": "User message for this turn.",
  "messages": [
    { "role": "system", "content": "Previous system context..." },
    { "role": "user", "content": "Previous user question..." },
    { "role": "assistant", "content": "Previous reply..." }
  ],
  "enabled_toolsets": ["terminal", "file", "web", "mcp"],
  "disabled_toolsets": ["dangerous"],
  "terminal_backend": "docker",
  "workspace": "/path/to/git/worktree",
  "quiet_mode": true
}
```

Notes:

- `messages` mirrors what Forge passes into `BackendTask.conversation_snippet`, translated to
  Hermes’ OpenAI-style message format.
- The wrapper is free to ignore fields it does not need, but must tolerate their presence
  for forward-compatibility.

#### HermesBackendResponse (Hermes wrapper → Forge)

The wrapper writes exactly one JSON object to stdout:

```json
{
  "version": "v1",
  "ok": true,
  "final_text": "Here is the answer or patch proposal...",
  "events": [
    { "type": "tool_started", "name": "terminal.run", "args_preview": "pytest -q" },
    { "type": "text_chunk", "content": "Running tests...\n" },
    { "type": "tool_finished", "name": "terminal.run", "ok": true }
  ],
  "usage": {
    "input_tokens": 1234,
    "output_tokens": 456,
    "total_tokens": 1690,
    "cost_usd": 0.12
  },
  "error": null
}
```

On error:

```json
{
  "version": "v1",
  "ok": false,
  "error": {
    "kind": "unavailable",
    "message": "hermes process failed to start"
  }
}
```

Mapping to `RuntimeAdapter`:

- `events[*]`:
  - `type = "text_chunk"` → `BackendEvent::TextChunk`.
  - `type = "tool_started"` → `BackendEvent::ToolStarted`.
  - `type = "tool_finished"` → `BackendEvent::ToolFinished`.
  - Optionally `type = "cost_update"` or `"warning"` for cost / warnings.
- `usage` → `BackendUsage`.
- `error.kind` → `AdapterError`:
  - `"unavailable"` → `Unavailable`.
  - `"timeout"` → `Timeout`.
  - `"rejected"` → `Rejected`.
  - `"protocol"` → `Protocol`.
  - `"internal"` → `Internal`.

Wrapper requirements:

- All logs go to **stderr**, not stdout.
- Stdout must contain exactly one valid JSON object per run.
- The `version` field must be preserved and bumped if the schema changes.

---

### Mapping Hermes into AgentForge

Hermes is the concrete implementation of `ProcessBackend::Hermes` in the expansion plan:

- **Forge → Hermes**
  - Forge constructs `BackendTask<HermesConfig>` from:
    - Agent configuration (`backend_type = "hermes"`, `HermesConfig`).
    - Session state (id, snippet of recent messages).
    - System prompt: persona, company/org context, skills/methodology, knowledge base snippets.
    - User prompt: current user message or pipeline step.
  - The Hermes adapter:
    - Translates `BackendTask` into `HermesBackendRequest`.
    - Spawns the Hermes wrapper process.
    - Streams `BackendEvent`s back into Forge’s `ForgeEvent` → WebSocket.
    - Returns `BackendCompletion` with final text and usage.

- **Hermes → Forge**
  - Final text becomes the assistant’s response in the current Forge session.
  - Events map to:
    - Tool indicators in the UI.
    - Cost tracking via `forge-safety` CostTracker.
  - Optionally, in later waves:
    - `MemorySync` exports Forge memories into a temporary `MEMORY.md` for Hermes.
    - After the run, new Hermes memories are summarized and imported back into Forge’s memory table.

**Forge assumptions and non-goals**:

- Forge **does not**:
  - Manage Hermes provider configuration, API keys, or model routing.
  - Install or curate Hermes skills or MCP servers.
  - Depend on Hermes’ CLI UX or spinner output.
- Forge **does**:
  - Require that a compatible `hermes` (or wrapper) binary is on `PATH` or configured via `FORGE_HERMES_COMMAND`.
  - Treat Hermes memory as eventually consistent, via opt-in summary sync rather than raw file copying.
  - Isolate its own git worktrees; Hermes’ terminal backend sees those as its `cwd`.

Where `docs/EXPANSION_PLAN.md` assumes richer behavior (e.g., bidirectional, lossless
MEMORY.md ↔ Forge memory syncing, or deep toolset filtering based on personas), this
document should be treated as the **current target contract** for the adapter. Any
divergence discovered during implementation must be recorded here before changing code.

