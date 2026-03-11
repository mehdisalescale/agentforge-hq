# Epic E7: Messaging & Communications

> **Multi-platform messaging (Telegram, Slack, Discord) + notification routing.**
>
> Source: AstrBot (16+ platforms), Hermes (gateway pattern)

---

## Business Value

Users shouldn't need a browser open to manage their AI workforce. This epic lets them send commands, receive reports, and approve actions from Telegram, Slack, or Discord. "Fix the login bug" on Slack → routed to the right engineering agent → results posted back to Slack.

## Acceptance Gate

1. Telegram bot receives messages and routes to Forge agents
2. Slack integration receives commands and posts results
3. Discord bot works with slash commands
4. Users configure notification preferences (which events, which platform)
5. Intent router classifies messages into actions (run agent, check status, search KB)
6. 20+ tests

---

## User Stories (8)

### E7-S1: Message Bridge Trait
```rust
#[async_trait]
pub trait MessageBridge: Send + Sync {
    fn platform_name(&self) -> &str;
    async fn start(&self) -> ForgeResult<()>;
    async fn send(&self, channel: &str, message: &str) -> ForgeResult<()>;
    async fn health(&self) -> BridgeHealth;
}
```

### E7-S2: Telegram Adapter (Native Rust)
- Bot API via reqwest (no external SDK)
- Webhook mode for production, polling for development
- Message → IntentRouter → Forge action → Response

### E7-S3: Slack Adapter (Native Rust)
- Events API webhook endpoint
- Slash commands: `/forge run`, `/forge status`, `/forge search`
- Block Kit formatted responses

### E7-S4: Discord Adapter (Native Rust)
- Gateway WebSocket via tungstenite
- Slash command registration
- Embed-formatted responses

### E7-S5: Intent Router
```gherkin
GIVEN a message "@backend-architect fix the auth bug"
WHEN IntentRouter.classify() runs
THEN it returns Intent::RunAgent { agent_name: "backend-architect", prompt: "fix the auth bug" }

GIVEN a message "status"
WHEN IntentRouter.classify() runs
THEN it returns Intent::GetStatus

GIVEN a message "search: how does payment work"
WHEN IntentRouter.classify() runs
THEN it returns Intent::SearchKB { query: "how does payment work" }
```

### E7-S6: Notification Router
```gherkin
GIVEN a user has notification prefs: { telegram: [ProcessCompleted, BudgetWarning] }
WHEN a ProcessCompleted event occurs
THEN a formatted message is sent to their Telegram

GIVEN a user has no notification prefs for Slack
WHEN events occur
THEN nothing is sent to Slack
```

### E7-S7: Messaging Config API & Frontend
- Platform connection CRUD
- Test message button
- Notification preference editor

### E7-S8: Sidecar AstrBot Bridge (Alternative)
- For users wanting 16+ platforms immediately
- Thin HTTP bridge to AstrBot REST API
- Forge sends/receives via AstrBot proxy

---

## Strategy

**Phase 1 (this epic):** Native Rust for top 3 (Telegram, Slack, Discord) — stays in single binary
**Phase 2 (future):** Optional AstrBot sidecar for WeChat, QQ, DingTalk, etc.

## Story Point Estimates: **32 total** across S6-S7
