### AstrBot → AgentForge Mapping

- **Goal**: Capture how AstrBot’s knowledge base, messaging, and plugin model map into AgentForge’s planned `forge-knowledge` and `forge-messaging` crates.
- **Scope**: Design-only reference for the Knowledge & Messaging Agent. No Rust or SQL migrations are defined here.

---

### 1. High-Level Overview

- **What AstrBot Is**
  - Open-source, Python-based agent chatbot platform.
  - Integrates with many IM platforms (Telegram, Slack, QQ, Wecom, Feishu, DingTalk, Discord, LINE, etc.).
  - Provides:
    - Multi-platform messaging adapters.
    - Knowledge base with hybrid retrieval.
    - Rich plugin system (“Stars”) and agent sandboxing.
    - Provider abstraction for multiple LLM / embedding / rerank services.
- **How AgentForge Will Use It**
  - As a **sidecar messaging and agent runtime service** (Wave 7a), not embedded into the Rust binary.
  - As a **reference design** for:
    - KB document lifecycle and retrieval behavior.
    - Messaging abstraction and platform adapter shape.
    - Plugin hosting and sandboxing as an external capability.

---

### 2. AstrBot Knowledge Base Model → `forge-knowledge`

#### 2.1 AstrBot KB Components

- **Storage and DB layer**
  - `KBSQLiteDatabase` (`astrbot/core/knowledge_base/kb_db_sqlite.py`):
    - Owns a dedicated SQLite file (`knowledge_base/kb.db` under the AstrBot data directory).
    - Uses SQLModel / SQLAlchemy for:
      - `KnowledgeBase`: logical KB instances with options (embedding provider, rerank provider, chunk size, etc.).
      - `KBDocument`: per-document metadata (name, type, size, stats).
      - `KBMedia`: extracted media associated with documents.
    - Configures SQLite pragmas (WAL, cache size, mmap, etc.) and creates indexes for KB and document fields.
  - **Design pattern**:
    - Metadata and stats live in SQLite tables.
    - Vector embeddings live in a separate FAISS index and doc-store managed by a VecDB layer.

- **Vector and retrieval layer**
  - `FaissVecDB` (`astrbot/core/db/vec_db/faiss_impl`):
    - Stores text chunks and their embeddings.
    - Metadata per chunk includes:
      - `kb_id`, `kb_doc_id`, `chunk_index`, plus arbitrary metadata JSON.
  - `KBHelper` (`astrbot/core/knowledge_base/kb_helper.py`):
    - Per-KB worker that:
      - Owns a `FaissVecDB` instance.
      - Manages filesystem layout under a KB-specific directory (doc db, index, media, files).
      - Handles document ingestion:
        - File/URL input → parser selection → text + media extraction.
        - Chunking via `RecursiveCharacterChunker`.
        - Embedding and optional reranking via providers (`EmbeddingProvider`, `RerankProvider`).
        - Batch insert into FAISS with progress callbacks.
      - Provides document and chunk CRUD and stats refresh.
  - `KnowledgeBaseManager` (`astrbot/core/knowledge_base/kb_mgr.py`):
    - Top-level orchestrator:
      - Initializes SQLite KB DB and runs migrations.
      - Creates and tracks per-KB `KBHelper` instances in memory.
      - Exposes APIs to:
        - Create / update / delete KBs.
        - List KBs.
        - Retrieve across multiple KBs:
          - Uses `SparseRetriever` and `RankFusion` via `RetrievalManager`.
          - Returns:
            - `context_text`: formatted preamble for prompt injection, including KB name, doc name and score.
            - `results[]`: structured per-chunk results (IDs, names, scores, metadata).
      - Supports URL-based ingestion, including optional LLM-based cleaning and translation.

#### 2.2 Principles AgentForge Will Borrow

- **Separation of concerns**
  - SQLite (relational) for KB metadata, documents, media and stats.
  - Vector index (FAISS in AstrBot) as an implementation detail behind an abstract VecDB interface.
  - Provider abstractions for embeddings and rerankers.
- **Multi-KB, multi-source design**
  - Multiple logical KBs, each configured with its own providers and retrieval tuning.
  - Support for mixed ingestion methods (files, URLs, pre-chunked text).
- **Retrieval output shape**
  - APIs return both:
    - Human-readable context text to inject into prompts.
    - Machine-readable per-chunk result lists for UI and debugging.

#### 2.3 Mapping to `forge-knowledge` (Wave 6)

AgentForge will not embed AstrBot’s KB implementation directly. Instead, `forge-knowledge` will reimplement a Rust-native KB using SQLite FTS5 while following the same high-level concepts:

- **Logical scope**
  - AstrBot: multiple named `KnowledgeBase` records.
  - AgentForge: **company-scoped knowledge**:
    - Tables:
      - `kb_documents` — document metadata, scoped by `company_id`.
      - `kb_chunks` — per-document text chunks.
      - `kb_chunks_fts` — FTS5 index over chunk content.
    - Optional later: a small “logical KB name or tag” dimension per company if needed.

- **Storage and search technology**
  - AstrBot:
    - SQLite for metadata (`KnowledgeBase`, `KBDocument`, `KBMedia`).
    - FAISS vector DB for semantic retrieval.
  - AgentForge (Wave 6 MVP):
    - Single Forge SQLite DB in WAL mode.
    - **FTS5-based keyword search** over `kb_chunks.content` (no FAISS or external vector DB required).
    - Optional embedding store can be added later as an extension, not a dependency.

- **Ingestion flow**
  - Shared ideas:
    - Parse → chunk → store chunks → record metadata → update stats.
    - Support for URLs and file uploads.
  - Differences:
    - AgentForge’s first implementation focuses on:
      - Text and markdown parsing, with simple chunking.
      - Minimal dependencies consistent with the single-binary constraint.
    - Image/media extraction is a later enhancement; AstrBot’s richer media pipeline is treated as a reference.

- **Retrieval and prompt injection**
  - AstrBot:
    - `KnowledgeBaseManager.retrieve(...)`:
      - Accepts query + KB names.
      - Produces `context_text` + `results[]` with scores.
  - AgentForge:
    - `forge-knowledge` will expose a `KnowledgeQuery` API and middleware:
      - Accepts `company_id` + query text.
      - Performs FTS5 search over `kb_chunks`.
      - Returns:
        - A formatted `context_text` (inspired by AstrBot’s pattern).
        - An array of chunk records (IDs, titles, indices, content, optional scores).
      - The **KnowledgeInjection middleware** (Wave 6) will:
        - Use this API before agent spawn.
        - Inject `context_text` into the system prompt or context window.
        - Optionally record which documents/chunks were injected for explainability.

---

### 3. AstrBot Messaging Model → `forge-messaging`

#### 3.1 AstrBot Platform Abstraction

- **Platform base class** (`astrbot/core/platform/platform.py`)
  - Holds:
    - Per-platform configuration (`config` dict).
    - A shared async event queue consumed by a central event bus.
    - A generated `client_self_id`.
    - A `PlatformStatus` state machine (`PENDING`, `RUNNING`, `ERROR`, `STOPPED`).
    - A list of `PlatformError`s (message, timestamp, traceback).
  - Key methods:
    - `run()`: coroutine that starts the adapter and pushes events into the queue.
    - `terminate()`: graceful shutdown (optional override).
    - `meta()`: returns a `PlatformMetadata` instance describing the adapter.
    - `send_by_session(session, message_chain)`: send a message via a persistent session abstraction.
    - `commit_event(event)`: push an `AstrMessageEvent` into the central queue.
    - `unified_webhook()`: indicates whether the adapter uses a shared webhook endpoint.
    - `webhook_callback(request)`: optional entrypoint for unified webhook integrations.

- **Platform metadata** (`astrbot/core/platform/platform_metadata.py`)
  - `PlatformMetadata` fields:
    - `name`: adapter type identifier (e.g. `telegram`, `slack`).
    - `description`: human-readable description.
    - `id`: unique ID for configuration.
    - Optional:
      - `default_config_tmpl`: default config template.
      - `adapter_display_name`: display name for WebUI.
      - `logo_path`.
      - `support_streaming_message`, `support_proactive_message` flags.
      - `module_path` and `config_metadata` for dynamic reload and UI generation.

- **Platform message history** (`astrbot/core/platform_message_history_mgr.py`)
  - `PlatformMessageHistoryManager` uses a `BaseDatabase` helper to:
    - `insert(platform_id, user_id, content, sender_id?, sender_name?)`.
    - `get(platform_id, user_id, page, page_size)`.
    - `delete(platform_id, user_id, offset_sec)`.
  - This provides per-platform, per-user history and supports compliance and UX features inside AstrBot.

#### 3.2 Principles AgentForge Will Borrow

- Treat each messaging platform as:
  - A **pluggable adapter** with well-defined metadata and config shape.
  - A source of normalized message events fed into a central pipeline.
  - A sink for outgoing messages, including proactive notifications.
- Provide:
  - Health and error visibility per platform.
  - Ability to run in **unified webhook** or dedicated endpoint modes.
- Keep long-term, per-platform message history in the system that owns platform adapters and permissions.

#### 3.3 Mapping to `forge-messaging` (Wave 7)

- **Role of AstrBot vs AgentForge**
  - AstrBot:
    - Owns **all platform adapters**, webhooks, platform-specific permissions and message history.
    - Supplies:
      - A list of supported platforms and their metadata.
      - A consistent API for sending messages and receiving commands.
  - AgentForge:
    - Owns organization-level configuration and preferences via:
      - `messaging_configs` — per-company messaging bridges and settings.
      - `notification_prefs` — per-recipient preferences for receiving Forge events.
    - Does **not** store raw platform message history or credentials beyond what is strictly required to talk to AstrBot or native adapters.

- **Sidecar integration (Wave 7a)**
  - `forge-messaging` will first be implemented as a **thin bridge** to AstrBot:
    - Forge keeps:
      - Which platforms are enabled for a company (e.g. “Company A uses Telegram via AstrBot instance X”).
      - How to map Forge events to recipients (e.g. Slack user/channel IDs).
    - AstrBot keeps:
      - All platform-specific tokens, secrets and adapter implementations.
      - Per-user and per-platform message history.
  - Trust boundary:
    - Forge is **authoritative** for what actions should be taken (which sessions/events should be notified, which agents should run).
    - AstrBot is authoritative for:
      - Whether a particular notification or command is allowed on a platform.
      - Rate limiting, retries and platform-specific failure semantics.
      - Storage and retention of chat history.

- **Optional native adapters (Wave 7b)**
  - Later, Forge may add native Rust adapters for a small subset of platforms (e.g. Telegram, Slack, Discord).
  - Even with native adapters:
    - The schema of `messaging_configs` and `notification_prefs` remains valid.
    - AstrBot can still be used as an alternative backend, configured via `platform = 'astrbot-sidecar'`.

---

### 4. AstrBot Plugins (“Stars”) → AgentForge

- **What Stars Are**
  - AstrBot’s plugin concept (`AstrBot Star`) is a first-class abstraction:
    - Stars are handlers (often called `star_handler`) registered with a central manager.
    - Filters (command, permissions, platform adapter type, message type) determine which stars see which events.
    - Stars can use:
      - LLM backends via providers.
      - AstrBot’s KB and platform abstractions.
      - The agent sandbox for controlled tool execution.

- **How AgentForge will treat them**
  - AgentForge will **not** re-implement the Star model or directly load Star plugins.
  - Instead:
    - AstrBot remains a **black-box plugin host**.
    - Forge interacts with one or more dedicated “Forge relay” Stars via HTTP-level contracts:
      - For outbound paths:
        - Forge sends a notification intent (session, event type, text, optional metadata).
        - A Star formats and routes the message within AstrBot, applying any plugin logic.
      - For inbound paths:
        - A Star receives platform messages and normalizes them.
        - Then calls back into Forge’s `/api/v1/messaging/inbound` endpoint with a simplified payload.
  - This keeps:
    - All plugin lifecycle, security and sandboxing **inside** AstrBot.
    - AgentForge’s core logic focused on agents, companies, goals, knowledge and orchestration.

---

### 5. Forge ↔ AstrBot Contracts (Conceptual Only)

> **Note**: This section describes intent-level contracts only. Exact HTTP paths, schemas and error codes will be finalized in Forge API docs and must stay consistent with `docs/EXPANSION_PLAN.md`. Treat everything here as a **proposal** for the Knowledge & Messaging Agent when designing `forge-messaging`.

#### 5.1 Outbound from Forge to AstrBot

- **Notification intents**
  - Forge generates events (e.g. `SessionCompleted`, `QualityGateFailed`, `GoalProgressUpdated`).
  - `NotificationSubscriber` in Forge:
    - Consults `notification_prefs` and `messaging_configs` for the company.
    - For each matching recipient:
      - Builds a message payload that includes:
        - Short human-readable text.
        - Minimal metadata (session ID, event type, company ID).
      - Sends this payload to the configured AstrBot bridge (sidecar).
  - AstrBot receives the payload and:
    - Delegates to a “Forge relay” Star to:
      - Apply rich formatting or workflows.
      - Fan out to specific platforms and channels according to AstrBot’s own config.

- **Optional management intents**
  - Forge may provide:
    - A simple health or registration ping so AstrBot can display connected Forge instances.
  - Details are intentionally left out here; they should be defined in Forge’s own API design docs.

#### 5.2 Inbound from AstrBot to Forge

- **User commands**
  - AstrBot receives messages from platforms via adapters.
  - A dedicated Star interprets them and classifies intents such as:
    - “@agent do X” → request to start or continue a Forge session.
    - “search kb: Y” → request to use Forge’s knowledge base.
    - “status” / “tasks” → request to query Forge state.
  - For recognized intents, the Star calls Forge’s inbound messaging endpoint with:
    - `platform` and `recipient` identifiers.
    - `company_id` or other organizational context where possible.
    - Normalized text command and optional structured intent.

- **Forge handling**
  - Forge:
    - Routes commands to the appropriate agent, workflow or knowledge query.
    - Produces results and triggers outbound notifications back through AstrBot as needed.
  - No raw platform payloads or history are stored in Forge beyond what is necessary to process the command.

---

### 6. Summary for the Knowledge & Messaging Agent

- **Use AstrBot as:**
  - A reference for designing Forge’s KB ingestion, retrieval and prompt-injection behavior.
  - A sidecar providing:
    - Multi-platform messaging adapters.
    - Plugin (Star) execution and sandboxing.
    - Platform-specific permissions, rate limits and history.
- **Do not:**
  - Copy AstrBot’s Python code or FAISS implementation into Forge.
  - Duplicate platform message history tables or expose AstrBot internals directly.
- **Design around:**
  - `forge-knowledge` and `forge-messaging` as Rust crates with:
    - SQLite-based schemas defined in Forge docs.
    - Clear contracts to AstrBot for messaging, defined at the HTTP and event-intent level in Forge’s own API design.

