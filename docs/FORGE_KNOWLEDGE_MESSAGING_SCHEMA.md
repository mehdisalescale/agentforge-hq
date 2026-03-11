### Forge Knowledge & Messaging Schema

- **Goal**: Provide implementation-ready schema and API shapes for `forge-knowledge` and `forge-messaging`.
- **Status**: Design document only — no Rust or migration files yet.
- **Alignment**:
  - Wave 6 (Knowledge Base) and Wave 7 (Messaging) in `docs/EXPANSION_PLAN.md`.
  - Knowledge & Messaging Agent scope in `docs/AGENTFORGE_AGENT_ROLES.md`.

---

### 1. Constraints and Scope

- **Single-binary philosophy**
  - Core behavior must work with:
    - A single SQLite database in WAL mode.
    - No external vector DB or message broker required.
  - Optional external services (AstrBot, embeddings APIs) are **add-ons**, not hard dependencies.
- **Company-first model**
  - Knowledge and messaging are **scoped by company**:
    - Every document and notification preference is associated with a `company_id` when applicable.
    - Multi-company support builds on `forge-org` / `forge-governance`.
- **Wave boundaries**
  - Wave 6:
    - Introduces `forge-knowledge` crate and KB tables (`kb_documents`, `kb_chunks`, `kb_chunks_fts`).
    - Uses SQLite FTS5 for text search.
  - Wave 7:
    - Introduces `forge-messaging` crate and tables (`messaging_configs`, `notification_prefs`).
    - Implements AstrBot sidecar mode first (Wave 7a).
    - Leaves native adapters (Wave 7b) as a later addition on the same schema.

---

### 2. `forge-knowledge` — SQLite Schema

#### 2.1 Tables

> **Note**: These are SQL sketches for future migrations (e.g. `0014_knowledge_base.sql`). Column types and constraints should be adapted to existing Forge conventions when implemented.

- **`kb_documents`**

```sql
CREATE TABLE kb_documents (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id     TEXT NOT NULL UNIQUE,         -- Stable UUID for external references
  company_id      TEXT NULL,                    -- FK to companies.id (Wave 3), nullable for global docs
  title           TEXT NOT NULL,
  source_type     TEXT NOT NULL CHECK (source_type IN ('file', 'url', 'text')),
  source_path     TEXT,                         -- File path or URL; optional for raw text
  mime_type       TEXT,
  file_size_bytes INTEGER NOT NULL DEFAULT 0,
  chunk_count     INTEGER NOT NULL DEFAULT 0,
  created_at      DATETIME NOT NULL,
  updated_at      DATETIME NOT NULL
);

CREATE INDEX idx_kb_documents_document_id
  ON kb_documents (document_id);

CREATE INDEX idx_kb_documents_company_id
  ON kb_documents (company_id);

CREATE INDEX idx_kb_documents_created_at
  ON kb_documents (created_at);
```

- **`kb_chunks`**

```sql
CREATE TABLE kb_chunks (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  chunk_id     TEXT NOT NULL UNIQUE,       -- Stable UUID for referencing specific chunks
  document_id  TEXT NOT NULL,              -- References kb_documents.document_id
  company_id   TEXT,                       -- Denormalized for fast company scoping
  chunk_index  INTEGER NOT NULL,           -- 0-based index within the document
  content      TEXT NOT NULL,
  token_count  INTEGER,                    -- Optional; filled by tokenizer if available

  FOREIGN KEY (document_id) REFERENCES kb_documents (document_id)
    ON DELETE CASCADE
);

CREATE INDEX idx_kb_chunks_document_id
  ON kb_chunks (document_id);

CREATE INDEX idx_kb_chunks_company_id
  ON kb_chunks (company_id);

CREATE INDEX idx_kb_chunks_chunk_index
  ON kb_chunks (document_id, chunk_index);
```

- **`kb_chunks_fts`** (FTS5 virtual table)

```sql
-- FTS5 virtual table for full-text search.
-- "content" is duplicated for search speed; "chunk_id" ties results back to kb_chunks.
CREATE VIRTUAL TABLE kb_chunks_fts USING fts5 (
  chunk_id UNINDEXED,
  content,
  tokenize = 'porter'
);

-- Keep FTS table in sync with kb_chunks.
CREATE TRIGGER kb_chunks_ai AFTER INSERT ON kb_chunks BEGIN
  INSERT INTO kb_chunks_fts (rowid, chunk_id, content)
  VALUES (new.id, new.chunk_id, new.content);
END;

CREATE TRIGGER kb_chunks_ad AFTER DELETE ON kb_chunks BEGIN
  DELETE FROM kb_chunks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER kb_chunks_au AFTER UPDATE ON kb_chunks BEGIN
  UPDATE kb_chunks_fts
    SET chunk_id = new.chunk_id,
        content  = new.content
    WHERE rowid = old.id;
END;
```

#### 2.2 Optional Embedding Store (Future Extension)

> **Proposal**: Not required for Wave 6. This section is a possible future extension for semantic search.

- **`kb_chunk_embeddings`**

```sql
CREATE TABLE kb_chunk_embeddings (
  chunk_id     TEXT PRIMARY KEY,   -- References kb_chunks.chunk_id
  provider_id  TEXT NOT NULL,      -- Embedding provider identifier
  dim          INTEGER NOT NULL,   -- Embedding dimension
  embedding    BLOB NOT NULL,      -- Binary-encoded vector
  created_at   DATETIME NOT NULL,

  FOREIGN KEY (chunk_id) REFERENCES kb_chunks (chunk_id)
    ON DELETE CASCADE
);

CREATE INDEX idx_kb_chunk_embeddings_provider
  ON kb_chunk_embeddings (provider_id);
```

The Knowledge & Messaging Agent should:

- Implement FTS5-based retrieval first.
- Treat embeddings as **optional**, gated behind configuration and an external embedding API.

#### 2.3 Company Isolation

- **How isolation works**
  - Every document and chunk may be associated with a `company_id`.
  - All KB operations must respect company boundaries:
    - Upload: assign documents/chunks to the active `company_id` where available.
    - Search: always filter by `company_id` if provided.
- **Relationship to `forge-org`**
  - `company_id` is expected to match `companies.id` from `forge-org` (Wave 3).
  - Cross-company access is not supported by default; any cross-company sharing must be explicit and documented in future design changes.

#### 2.4 Knowledge API Shapes

> **Note**: These are shape sketches to guide `forge-api` work. Exact response wrappers (e.g. pagination envelopes) should follow existing Forge API conventions.

- **`GET /api/v1/knowledge`**

  - **Query parameters**
    - `company_id?` — filter by company; if omitted, returns global or user-allowed docs only.
    - `q?` — optional search query; if provided, returns documents matching query across title or content (via FTS).
    - `page?` / `page_size?` — standard pagination.
  - **Response (shape)**
    - `items: [ { document_id, company_id, title, source_type, source_path?, mime_type?, file_size_bytes, chunk_count, created_at, updated_at } ]`
    - `page`, `page_size`, `total` (if consistent with existing patterns).

- **`POST /api/v1/knowledge/upload`**

  - **Purpose**
    - Upload one or more documents into the KB for a company.
  - **Request (two possible forms)**
    - **Multipart form-data** (files):
      - Fields:
        - `company_id` (required where multi-company is enabled).
        - One or more `file` parts (with file name and mime type).
    - **JSON** (URL or raw text):
      - Example:
        ```json
        {
          "company_id": "COMPANY-UUID",
          "source_type": "url",
          "value": "https://example.com/doc.html"
        }
        ```
      - Or:
        ```json
        {
          "company_id": "COMPANY-UUID",
          "source_type": "text",
          "title": "Internal KB Note",
          "content": "Free-form text to index..."
        }
        ```
  - **Response (shape)**
    - For single upload:
      - `{ document_id, company_id, title, source_type, chunk_count }`
    - For batch upload:
      - `{ documents: [ { document_id, ... } ] }`

- **`POST /api/v1/knowledge/search`**

  - **Request JSON**

    ```json
    {
      "company_id": "COMPANY-UUID",
      "query": "search terms here",
      "limit": 10
    }
    ```

  - **Response JSON**

    ```json
    {
      "context_text": "Human-readable KB context suitable for prompt injection...",
      "results": [
        {
          "chunk_id": "CHUNK-UUID",
          "document_id": "DOC-UUID",
          "company_id": "COMPANY-UUID",
          "title": "Document Title",
          "chunk_index": 3,
          "content": "Chunk text...",
          "score": 0.93,
          "token_count": 128
        }
      ]
    }
    ```

  - **Behavior**
    - Uses FTS5 over `kb_chunks_fts` scoped by `company_id`.
    - Produces:
      - `context_text` that mimics AstrBot’s pattern (“Source: KB / Doc / Score: …”) for transparency.
      - A structured list of chunks for tooling and UI.

#### 2.5 Optional Logging of Knowledge Injection

> **Proposal**: Optional enhancement; not required for initial Wave 6 implementation.

- **`kb_injection_logs`**

```sql
CREATE TABLE kb_injection_logs (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id   TEXT NOT NULL,        -- Forge SessionId
  run_id       TEXT NOT NULL,        -- Per-run identifier if available
  query        TEXT NOT NULL,
  document_ids TEXT NOT NULL,        -- JSON array of document_id strings
  chunk_ids    TEXT NOT NULL,        -- JSON array of chunk_id strings
  created_at   DATETIME NOT NULL
);

CREATE INDEX idx_kb_injection_logs_session
  ON kb_injection_logs (session_id, created_at);
```

Agent behavior:

- For each run, the KnowledgeInjection middleware may write a log entry describing which documents and chunks were injected, making KB usage explainable and auditable.

---

### 3. `forge-messaging` — SQLite Schema

#### 3.1 Tables

> **Note**: These tables support both AstrBot sidecar mode (Wave 7a) and future native adapters (Wave 7b).

- **`messaging_configs`**

```sql
CREATE TABLE messaging_configs (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  company_id  TEXT,                 -- FK to companies.id; NULL for global configs
  platform    TEXT NOT NULL,        -- e.g. 'astrbot-sidecar', 'telegram', 'slack'
  config_json TEXT NOT NULL,        -- Opaque, validated in Rust; may contain URLs, tokens, options
  enabled     INTEGER NOT NULL DEFAULT 1,
  created_at  DATETIME NOT NULL,
  updated_at  DATETIME NOT NULL
);

CREATE INDEX idx_messaging_configs_company_platform
  ON messaging_configs (company_id, platform);
```

- **`notification_prefs`**

```sql
CREATE TABLE notification_prefs (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  company_id    TEXT,                -- FK to companies.id; allows per-company scoping
  recipient_key TEXT NOT NULL,       -- e.g. 'slack:user:U123', 'telegram:chat:123'
  platform      TEXT NOT NULL,       -- Must match a platform in messaging_configs
  channel_id    TEXT,                -- Room/channel/group ID where relevant
  event_filters TEXT NOT NULL,       -- JSON array of event types or categories
  created_at    DATETIME NOT NULL,
  updated_at    DATETIME NOT NULL
);

CREATE INDEX idx_notification_prefs_company_recipient
  ON notification_prefs (company_id, recipient_key);

CREATE INDEX idx_notification_prefs_platform
  ON notification_prefs (platform);
```

**Deliberate non-goals:**

- No `platform_message_history` tables in Forge:
  - AstrBot (or native adapters) own:
    - Raw inbound/outbound payloads.
    - Retention, compliance and replay logic.
  - Forge keeps only:
    - Minimal routing data (recipient keys, platforms, filters).
    - Pointers needed to correlate events and commands.

#### 3.2 Messaging API Shapes

> **Note**: Schemas below are sketches; actual field naming should match existing Forge API conventions.

- **`GET /api/v1/messaging/configs`**

  - **Query parameters**
    - `company_id?` — filter configs for a specific company.
  - **Response**
    - `items: [ { id, company_id, platform, config_summary, enabled, created_at, updated_at } ]`
      - `config_summary` is a redacted or summarized view of `config_json` (no secrets).

- **`POST /api/v1/messaging/configs`**

  - **Request JSON**

    ```json
    {
      "company_id": "COMPANY-UUID",
      "platform": "astrbot-sidecar",
      "config": {
        "astrbot_url": "https://astrbot.example.com",
        "default_platform": "telegram"
      }
    }
    ```

  - **Response**
    - `{ id, company_id, platform, enabled, created_at, updated_at }`

- **`PUT /api/v1/messaging/configs/:id`**

  - **Request JSON**
    - Same shape as POST; partial updates allowed depending on implementation.
  - **Response**
    - Updated config summary.

- **`POST /api/v1/messaging/test`**

  - **Request JSON**

    ```json
    {
      "config_id": 1,
      "recipient_key": "telegram:chat:123456789",
      "message": "This is a test message from AgentForge."
    }
    ```

  - **Response**
    - `{ status: "ok" }` on success, or structured error.

---

### 4. Messaging Webhook Endpoints (Future Native Adapters)

> **Wave 7b**: When Forge implements native adapters, the following endpoints will terminate platform webhooks or events directly in `forge-api`. Sidecar mode (Wave 7a) can also use a generic inbound endpoint.

- **Slack**

  - `POST /api/v1/webhooks/slack`
    - Receives Slack events (e.g., via Events API).
    - Normalizes into an internal “message received” event, including:
      - `platform = 'slack'`
      - `recipient_key` derived from user/channel IDs.
      - `text` and minimal metadata.

- **Telegram**

  - `POST /api/v1/webhooks/telegram`
    - Receives Telegram Bot API updates.
    - Normalizes into internal event as above.

- **Discord**

  - `POST /api/v1/webhooks/discord`
    - Receives Discord interaction payloads (e.g., via gateway relay or HTTP interactions).
    - Normalizes into internal event.

- **Generic inbound for AstrBot sidecar (Wave 7a)**

  - `POST /api/v1/messaging/inbound`
    - Expected payload (from an AstrBot Star or HTTP handler):

      ```json
      {
        "platform": "telegram",
        "recipient": "telegram:chat:123456789",
        "company_id": "COMPANY-UUID",
        "text": "search kb: deployment runbook",
        "intent": "kb_search",
        "raw": { "...": "optional opaque payload" }
      }
      ```

    - Forge:
      - Routes by `intent` (e.g. spawn agent, run KB search, fetch status).
      - Produces results and pushes notifications back via sidecar using `messaging_configs`.

---

### 5. Company Isolation & Sidecar vs Native Modes

#### 5.1 Company Isolation

- **KB**
  - All KB operations take an optional `company_id`.
  - The Knowledge & Messaging Agent must ensure:
    - Uploads assign `company_id` where appropriate.
    - Searches filter by `company_id` when provided.
    - Cross-company queries are rejected or explicitly guarded by authorization.

- **Messaging**
  - `messaging_configs.company_id`:
    - Determines which messaging bridges apply to which companies.
  - `notification_prefs.company_id`:
    - Allows the same human (e.g. Slack user) to have different preferences per company if required.

#### 5.2 Sidecar Mode (Wave 7a) vs Native Adapters (Wave 7b)

- **Sidecar Mode (Wave 7a)**
  - `forge-messaging` acts as:
    - A router from `ForgeEvent` → `{ platform, recipient_key, text }` payloads.
    - A consumer of inbound commands via `/api/v1/messaging/inbound`.
  - AstrBot:
    - Owns all platform adapters, credentials, rate limiting and history.
    - Hosts one or more Stars that:
      - Accept Forge-originated payloads.
      - Apply plugin logic and formatting.
      - Forward to IM platforms.

- **Native Adapter Mode (Wave 7b)**
  - `forge-messaging` and `forge-api`:
    - Implement platform-specific HTTP clients and webhook handlers.
    - Use the same `messaging_configs` and `notification_prefs` tables.
  - AstrBot becomes optional:
    - May still be used as a secondary backend (e.g. `platform = 'astrbot-sidecar'`).
  - The Knowledge & Messaging Agent should:
    - Treat AstrBot integration as the **first implementation target**.
    - Design all schemas and APIs so native adapters are additive, not a breaking change.

---

### 6. NotificationSubscriber Behavior

The `NotificationSubscriber` (in `forge-core` / `forge-messaging`) is the bridge from internal events to outbound messaging using the schema above.

- **Inputs**
  - Subscribes to `ForgeEvent` stream, including (examples):
    - `SessionCreated`, `SessionCompleted`, `SessionErrored`.
    - `GoalProgressUpdated`, `GoalCompleted`.
    - `QualityGateFailed`, `ExitGateTriggered`.
  - Has access to:
    - `company_id` (from the agent or session).
    - Event-specific payloads (session ID, goal ID, cost, diagnostics).

- **Routing logic**
  - Step 1: Determine applicable messaging configs
    - Query `messaging_configs` by `company_id`.
    - Filter to `enabled = 1`.
    - For Wave 7a, at least one config should have `platform = 'astrbot-sidecar'`.
  - Step 2: Determine recipients
    - Query `notification_prefs` for:
      - Matching `company_id` (or NULL if global).
      - Matching `platform` from the configs.
    - For each preference:
      - Parse `event_filters` (JSON array of event types or categories).
      - If the current event matches filters, mark that `recipient_key` as a target.
  - Step 3: Emit outbound payloads
    - For each `(config, recipient)` pair:
      - Build a minimal payload with:
        - `platform`, `recipient_key`, `company_id`.
        - Human-readable `text` summarizing the event.
        - Optional metadata (session/goal IDs, links).
      - Send via:
        - AstrBot sidecar bridge (Wave 7a), or
        - Native adapter client (Wave 7b).

- **Non-goals**
  - `NotificationSubscriber`:
    - Does not manage platform-specific rate limiting or retries (those belong to adapters/sidecar).
    - Does not store message history beyond what is necessary for logging and debugging.

---

### 7. Summary

- `forge-knowledge`:
  - Uses `kb_documents`, `kb_chunks`, `kb_chunks_fts` to provide company-scoped KB with FTS5 search.
  - Optionally adds embedding support and injection logging later without schema rewrites.
- `forge-messaging`:
  - Uses `messaging_configs` and `notification_prefs` to route Forge events to people on messaging platforms.
  - Supports AstrBot sidecar mode first, with a clear path to native adapters on the same schema.
- The Knowledge & Messaging Agent should:
  - Implement these schemas and APIs in Rust and migrations in future waves.
  - Keep this document as the design reference, updating it if implementation diverges (marked clearly as proposals).

