# MCP Server Design (stdio)

> Design for the Forge MCP server: stdio transport, JSON-RPC 2.0, initial tool set.
> Implements Phase B “MCP server (10 tools, stdio)” from NORTH_STAR.

---

## Transport

- **Stdio only:** read from stdin, write to stdout. No HTTP in v0.2.0.
- **Framing:** one JSON-RPC request or response per line (newline-delimited JSON).
- **Encoding:** UTF-8.

---

## Protocol

- **JSON-RPC 2.0:** `jsonrpc: "2.0"`, `id` (number or string), `method`, `params` (object or array); response has `result` or `error` (code, message, optional data).
- **Error mapping:** ForgeError variants → JSON-RPC error code (e.g. -32602 invalid params, -32001 not found, -32000 internal). Message = error display string.

---

## Initial tools (~10)

Tools are exposed as JSON-RPC methods. Params and results mirror the HTTP API types where possible.

| Method | Params | Result | Notes |
|--------|--------|--------|--------|
| `agent_list` | none | `{ "agents": Agent[] }` | Same as GET /api/v1/agents |
| `agent_get` | `{ "id": string }` | Agent | GET /api/v1/agents/:id |
| `agent_create` | NewAgent (name, model?, system_prompt?, allowed_tools?, max_turns?, use_max?, preset?, config?) | Agent | POST /api/v1/agents |
| `agent_update` | `{ "id": string }` + UpdateAgent fields | Agent | PUT /api/v1/agents/:id |
| `agent_delete` | `{ "id": string }` | `{ "ok": true }` | DELETE /api/v1/agents/:id |
| `session_list` | none | `{ "sessions": Session[] }` | GET /api/v1/sessions |
| `session_get` | `{ "id": string }` | Session | GET /api/v1/sessions/:id |
| `session_create` | `{ "agent_id": string, "directory": string, "claude_session_id"?: string }` | Session | POST /api/v1/sessions |
| `session_export` | `{ "id": string, "format"?: "json" \| "markdown" }` | JSON object (session + events) or string (markdown) | GET /api/v1/sessions/:id/export |
| `run_create` | `{ "agent_id": string, "prompt": string, "session_id"?: string, "directory"?: string }` | `{ "session_id": string, "message"?: string }` | POST /api/v1/run (async; returns session_id) |

- **Agent:** id, name, model, system_prompt?, allowed_tools?, max_turns?, use_max, preset?, config?, created_at, updated_at (ISO 8601).
- **Session:** id, agent_id, claude_session_id?, directory, status, cost_usd, created_at, updated_at.
- **NewAgent / UpdateAgent:** same shapes as API (preset as string enum: CodeWriter, Reviewer, …).

---

## Server loop (implementation)

1. Open DB: `FORGE_DB_PATH` env (default `~/.claude-forge/forge.db`). Create connection, run migrations.
2. Build repos: AgentRepo, SessionRepo, EventRepo (and for run_create: need spawn/event_bus — defer run_create to a second phase or add minimal async/spawn in MCP binary).
3. Loop: read line from stdin → parse JSON-RPC request → dispatch by `method` to repo (or run handler) → build JSON-RPC response → write line to stdout.
4. On parse error or unknown method: respond with appropriate JSON-RPC error.

---

## Defer / later

- **run_create:** Requires spawning Claude CLI and event bus; can be added in a follow-up (MCP binary with tokio + forge-process) or omitted from first cut (only CRUD + session list/get/export).
- **HTTP transport:** Optional later: HTTP server exposes MCP endpoint or spawns this binary per connection.

---

## File map

| Item | Location |
|------|----------|
| Design | this doc |
| Server loop + dispatch | `crates/forge-mcp/src/server.rs` (new) |
| Binary | `crates/forge-mcp-bin` (new) or `crates/forge-mcp` with `[[bin]]` |
