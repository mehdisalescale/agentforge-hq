## OpenClaw — External Runtime Overview

**Status**: Reference for the Runtime Adapter Agent and `forge-adapter-openclaw`.  
**Upstream repo**: `https://github.com/openclaw/openclaw`  
**Local path**: `openclaw/` (sibling to `forge-project/` in this workspace)

OpenClaw is a local-first personal AI assistant with a WebSocket/HTTP gateway, multi-channel
messaging adapters, and a single embedded agent runtime (derived from pi-mono). AgentForge
uses it as an optional webhook-based backend behind `ProcessBackend::OpenClaw`.

For the shared adapter trait and common types (`RuntimeAdapter`, `BackendTask`, `AdapterError`, etc.),
see `docs/EXTERNAL_REPOS/HERMES_AGENT.md` (“Shared Runtime Adapter Contract”).

---

### Core Concepts

- **Gateway and channels**
  - Gateway exposes:
    - WebSocket control plane (sessions, presence, config, cron, tools, events).
    - HTTP surfaces: Control UI, WebChat, webhook ingress.
  - Multi-channel inbox:
    - WhatsApp, Telegram, Slack, Discord, Google Chat, Signal, BlueBubbles/iMessage, IRC, Microsoft Teams,
      Matrix, Feishu, LINE, Mattermost, Nextcloud Talk, Nostr, Synology Chat, Tlon, Twitch, Zalo, Zalo Personal,
      WebChat, and more.
  - Channel routing:
    - Per-channel routing rules, group message handling, mention gating, chunking/streaming policies.

- **Agent runtime and workspace contract**
  - Single embedded agent runtime (pi-mono–derived) per gateway.
  - Required workspace:
    - `agents.defaults.workspace` is the agent’s **only** working directory (`cwd`) for tools and context.
    - Optional sandbox workspaces for non-main sessions when `agents.defaults.sandbox` is enabled.
  - Bootstrap files in the workspace:
    - `AGENTS.md` — operating instructions + accumulated “memory”.
    - `SOUL.md` — persona, boundaries, tone.
    - `TOOLS.md` — user-maintained notes about tools and conventions.
    - `IDENTITY.md` — agent name/vibe/emoji.
    - `USER.md` — user profile and preferred address.
    - `BOOTSTRAP.md` — one-time first-run ritual (removed after completion).
  - On first turn of a new session:
    - Contents of these files are injected into the agent context.
    - Large files are trimmed with truncation markers; missing files produce a single “missing file” line.

- **Sessions and transcripts**
  - Session transcripts:
    - Stored as JSONL at `~/.openclaw/agents/<agentId>/sessions/<SessionId>.jsonl`.
    - OpenClaw chooses the session id; legacy Pi/Tau folders are not read.
  - Queue and steering:
    - Modes: `steer`, `followup`, `collect`.
    - `steer`: queued user messages can interrupt an in-progress run after each tool call.
    - Streaming and chunking:
      - Block streaming can be enabled per channel.
      - Chunking tuned via `agents.defaults.blockStreaming*` settings.

- **Webhooks**
  - Webhook ingress is configured under `hooks:` in OpenClaw config.
  - Enabled via:
    - `hooks.enabled: true`
    - `hooks.token: "<shared-secret>"`
    - `hooks.path: "/hooks"` (default; configurable).
  - Auth:
    - `Authorization: Bearer <token>` (recommended).
    - `x-openclaw-token: <token>`.
    - Query-string tokens are rejected.
  - Built-in endpoints:
    - `POST /hooks/wake` — enqueue a system wake event.
    - `POST /hooks/agent` — trigger an isolated agent run (see below).

- **Security**
  - Hook endpoints:
    - Must include the hook token; repeated auth failures are rate-limited and return `401`/`429`.
    - Recommended to keep behind loopback, tailnet (Tailscale Serve/Funnel), or a trusted reverse proxy.
  - Hook configuration:
    - `hooks.allowedAgentIds` limits which agents can be explicitly targeted by `agentId`.
    - `hooks.defaultSessionKey` and `hooks.allowRequestSessionKey` control session key behavior.
    - `hooks.allowedSessionKeyPrefixes` restricts user-supplied `sessionKey` values.
    - Optional `allowUnsafeExternalContent: true` disables wrappers for untrusted payloads (dangerous).

---

### `/hooks/agent` Request / Response Behavior (Current Reality)

#### Request payload (OpenClaw today)

`POST /hooks/agent` accepts JSON bodies such as:

```json
{
  "message": "Run this",
  "name": "Email",
  "agentId": "hooks",
  "sessionKey": "hook:email:msg-123",
  "wakeMode": "now",
  "deliver": true,
  "channel": "last",
  "to": "+15551234567",
  "model": "openai/gpt-5.2-mini",
  "thinking": "low",
  "timeoutSeconds": 120
}
```

- **Fields**:
  - `message` (required): text prompt the agent should process.
  - `name` (optional): human-readable hook name (used for summaries).
  - `agentId` (optional): route to a specific OpenClaw agent; unknown ids fall back to default.
  - `sessionKey` (optional, gated):
    - Overridden only when `hooks.allowRequestSessionKey=true`.
    - Often constrained with `hooks.allowedSessionKeyPrefixes`, e.g. `["hook:"]`.
  - `wakeMode` (optional): `"now"` (default) or `"next-heartbeat"`.
  - `deliver` (optional): whether to send the reply to a messaging channel; defaults to `true`.
  - `channel` (optional): delivery channel (`"last"`, `"whatsapp"`, `"telegram"`, `"discord"`, `"slack"`, etc.).
  - `to` (optional): per-channel recipient identifier (phone/chat/channel id).
  - `model` (optional): override model id (must respect OpenClaw’s allowed model list).
  - `thinking` (optional): thinking level (`"low"`, `"medium"`, `"high"`, etc.).
  - `timeoutSeconds` (optional): per-run timeout at the OpenClaw layer.

#### Response behavior

- HTTP responses:
  - `200` for accepted `/hooks/agent` invocations (asynchronous run).
  - `400` for invalid payloads.
  - `401` on auth failure.
  - `429` after repeated auth failures.
  - `413` on oversized payloads.
- Execution semantics:
  - The agent run is **isolated** under its own session key.
  - A summary is always posted into the main session.
  - If `deliver=true`, the reply is also delivered to the chosen messaging channel/recipient.
  - There is **no built-in callback** mechanism to send the final result to an arbitrary
    external HTTP endpoint; OpenClaw reports to its own sessions and channels.

From AgentForge’s perspective, today’s `/hooks/agent` is a **fire-and-forget** mechanism:
Forge can tell OpenClaw to run something, but OpenClaw reports results to its own users,
not back into Forge.

---

### OpenClaw Adapter Config and Task Shape (Design for AgentForge)

This section defines the contract between AgentForge and OpenClaw as used by
`forge-adapter-openclaw`. It builds on the shared `RuntimeAdapter` trait described in
`HERMES_AGENT.md`.

#### OpenClawConfig (Agent-level configuration)

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct OpenClawConfig {
    /// Base URL for the OpenClaw gateway, e.g. "http://127.0.0.1:18789".
    pub gateway_url: String,

    /// Shared secret token configured under `hooks.token`.
    pub hooks_token: String,

    /// Base path for hooks endpoint, default "/hooks".
    pub hooks_path: Option<String>,

    /// Optional default agent id in OpenClaw (e.g. "hooks").
    pub agent_id: Option<String>,

    /// Default channel and target for deliver=true.
    pub default_channel: Option<String>,
    pub default_to: Option<String>,

    /// Default model for OpenClaw runs (must be allowed by OpenClaw config).
    pub model: Option<String>,

    /// Whether Forge expects OpenClaw to deliver user-visible messages itself.
    pub deliver: Option<bool>,

    /// Optional prefix for OpenClaw `sessionKey` values (e.g. "forge:").
    pub session_key_prefix: Option<String>,
}
```

`OpenClawConfig` is intended to live inside `Agent.config_json` once adapters are
implemented and to be editible from the AgentForge UI when selecting OpenClaw as a backend.

#### OpenClawTaskRequest (Forge → `/hooks/agent`)

When a Forge session uses `ProcessBackend::OpenClaw`, the adapter builds an
`OpenClawTaskRequest` from `BackendTask<OpenClawConfig>` and POSTs it to
`{gateway_url}{hooks_path or "/hooks"}/agent`:

```json
{
  "message": "Summarize session 8f3a6f76-… for the human in a friendly tone.",
  "name": "AgentForge",
  "agentId": "hooks",
  "sessionKey": "forge:session:8f3a6f76-1e12-4c3a-9f1a-0b0c9e62a123",
  "wakeMode": "now",
  "deliver": true,
  "channel": "last",
  "to": null,
  "model": "openai/gpt-5.2-mini",
  "thinking": "low",
  "timeoutSeconds": 120
}
```

HTTP details:

- Method: `POST`.
- URL: `{gateway_url}{hooks_path or "/hooks"}/agent`.
- Headers:
  - `Authorization: Bearer <hooks_token>` **or** `x-openclaw-token: <hooks_token>`.
  - `Content-Type: application/json`.

Mapping from Forge:

- `message` is derived from the combination of `system_prompt`, `user_prompt`, and any
  necessary agent/session metadata, compressed into a single request string appropriate
  for a high-level “do X for this Forge session” instruction.
- `name` identifies AgentForge as the hook source.
- `agentId` and `sessionKey` come from `OpenClawConfig` plus the Forge session id.

#### Adapter behavior (v1: fire-and-forget)

- If the HTTP call returns **2xx**:
  - The adapter treats the request as **accepted** and returns a `BackendCompletion` with:
    - `final_text` set to a brief status message such as:
      `"Task delegated to OpenClaw; reply will appear on configured channel."`
    - `usage = None` (Forge does not know actual token usage).
    - Optionally a `BackendEvent::Warning` noting that this is asynchronous.
- If the HTTP call returns **4xx**:
  - `401` / `403` → `AdapterError::Rejected("OpenClaw auth failed")`.
  - `400` → `AdapterError::Rejected("invalid OpenClaw webhook payload")`.
  - Other `4xx` → `AdapterError::Protocol(...)`.
- If the HTTP call returns **5xx** or fails at the transport level:
  - Map to `AdapterError::Unavailable` or `AdapterError::Timeout` depending on the failure.

The v1 adapter does **not** attempt to read or reconstruct the full agent response inside
Forge. OpenClaw remains responsible for delivering the reply to end users via its own
channels and main session.

---

### Target v2 Callback Contract (Requires OpenClaw Changes)

The expansion plan describes a richer integration:

- New crate: `forge-adapter-openclaw`.
- `OpenClawAdapter`: HTTP client to the OpenClaw gateway.
- `WebhookHandler`: Receive results via callback URL.
- `ResultParser`: map OpenClaw responses into `ForgeEvent`s.
- Forge API endpoint: `POST /api/v1/webhooks/openclaw`.
- DB table: `webhook_callbacks` capturing status and payloads.

**Important**: The following callback contract is **target design** and **does not exist**
in OpenClaw today. Implementing it will require upstream OpenClaw changes or a small
shim service.

#### Desired OpenClawCallbackPayload (OpenClaw → Forge)

We want OpenClaw (or a shim) to POST to Forge when a hook-generated run completes:

```json
{
  "version": "v1",
  "correlationId": "forge-session-8f3a6f76-1e12-4c3a-9f1a-0b0c9e62a123",
  "agentId": "hooks",
  "sessionKey": "hook:ingress",
  "replyText": "Here is your summary...",
  "media": [],
  "usage": {
    "inputTokens": 123,
    "outputTokens": 456
  }
}
```

- `correlationId`:
  - Generated by Forge when building `OpenClawTaskRequest`.
  - Allows the adapter to route callbacks back into the correct Forge session.
- `replyText`:
  - The human-facing message to inject into the Forge session.
- `usage`:
  - Optional token usage; if present, feeds into `BackendUsage` and `CostTracker`.

#### Forge callback endpoint (target)

- Endpoint:

  - `POST /api/v1/webhooks/openclaw`

- Headers:

  - `Authorization: Bearer <callbackToken>` (callback token generated/configured on Forge side).
  - `Content-Type: application/json`.

- Behavior:

  - Validate token and parse `OpenClawCallbackPayload`.
  - Record row in `webhook_callbacks` (`id`, `session_id`, `source = "openclaw"`,
    `status`, `payload`, timestamps).
  - Map `replyText` into a `BackendEvent::TextChunk` (or equivalent) and finalize a
    `BackendCompletion` for the waiting Forge session.
  - Optionally surface usage into `BackendUsage`.

#### Adapter behavior (v2: callback-aware)

Given this target design:

- `execute()` for `ProcessBackend::OpenClaw` would:
  - Generate a `correlationId`.
  - Include it (and a Forge-known `callbackUrl` and `callbackToken`) in the
    `OpenClawTaskRequest` payload or mapping.
  - Return either:
    - Immediately (fire-and-forget) while a separate webhook handler completes the session
      when the callback arrives, **or**
    - Block until the matching `OpenClawCallbackPayload` is observed in a short-lived queue,
      then return `BackendCompletion` with the actual `replyText` and `usage`.
- Error mapping:
  - Missing or delayed callbacks lead to `AdapterError::Timeout` or `AdapterError::Unavailable`,
    depending on health checks and configured expectations.

Again: none of this callback behavior exists in upstream OpenClaw as of this doc. It is a
**design target** aligned with `docs/EXPANSION_PLAN.md` (Waves 4 and 5). Any concrete
implementation must either:

- Add this functionality to OpenClaw (preferred), or
- Introduce a lightweight proxy/shim that:
  - Accepts Forge’s richer payloads.
  - Talks to OpenClaw using today’s `/hooks/agent` interface.
  - Sends `OpenClawCallbackPayload` back to Forge when results are ready.

---

### Mapping OpenClaw into AgentForge

Within AgentForge, OpenClaw acts as a concrete implementation of `ProcessBackend::OpenClaw`:

- **Forge → OpenClaw (v1)**
  - Forge constructs `BackendTask<OpenClawConfig>` when an agent selects the OpenClaw backend.
  - `forge-adapter-openclaw`:
    - Translates the task into an `OpenClawTaskRequest`.
    - Calls `/hooks/agent` with proper auth.
    - Treats 2xx as “accepted” and surfaces a short status completion back into the Forge UI.
  - Actual agent replies are visible to users in OpenClaw’s own channels and Control UI.

- **Forge → OpenClaw (v2, target)**
  - Same as v1, but with:
    - Correlation id and callback metadata in the request.
    - A Forge webhook endpoint receiving `OpenClawCallbackPayload`.
    - A completed `BackendCompletion` populated from callback data.

- **Forge does not**:
  - Manage OpenClaw’s channels, DM pairing policies, or device nodes.
  - Mirror OpenClaw’s full session logs or workspace state.
  - Control OpenClaw model selection beyond the `model` field in the webhook payload.

- **Forge does**:
  - Rely on `gateway_url` and `hooks_token` being valid and reachable.
  - Treat OpenClaw as an external system; failure to contact it surfaces as
    `AdapterError::Unavailable` / `Timeout` / `Rejected`.
  - Capture minimal metadata in `webhook_callbacks` (once migrations exist) for observability.

Where `docs/EXPANSION_PLAN.md` specifies richer behavior (e.g., synchronous waits for results,
full callback-driven event streaming, or deep cost accounting), this file is the **source of
truth for the adapter contract**. Implementation work must update this document first if
the final behavior differs from the plan.

