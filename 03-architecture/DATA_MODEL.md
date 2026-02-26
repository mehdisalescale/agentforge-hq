# Claude Forge -- Data Model

> Complete SQLite schema, entity relationships, migration strategy, query patterns, and FTS5 configuration.
> Database location: `~/.claude-forge/forge.db` | Journal mode: WAL

**Phase 0 / current implementation:** The Phase 0 build uses a single migration with denormalized `agents` columns (e.g. `name`, `system_prompt`, `model` as columns) and **three** FTS5 virtual tables: `skills_fts`, `sessions_fts`, `events_fts`. See `PHASE0_IMPLEMENTATION_PLAN.md` and `migrations/0001_init.sql` for the canonical schema. The "Current Schema (v1)" and "Target Schema (v5)" sections describe alternative/legacy shapes.

---

## Table of Contents

1. [Schema Overview](#schema-overview)
2. [Current Schema (v1)](#current-schema-v1)
3. [Target Schema (v5)](#target-schema-v5)
4. [Entity Relationship Diagram](#entity-relationship-diagram)
5. [Migration Strategy](#migration-strategy)
6. [Data Lifecycle](#data-lifecycle)
7. [Query Patterns](#query-patterns)
8. [FTS5 Configuration](#fts5-configuration)
9. [Batch Write Strategy](#batch-write-strategy)
10. [Storage Estimates](#storage-estimates)

---

## Schema Overview

Claude Forge uses a single SQLite database in WAL (Write-Ahead Logging) mode. This provides:
- Concurrent reads while writing (readers never block)
- Crash-safe writes (WAL survives process kill)
- Single-file portability (copy `forge.db` to migrate)
- Bundled with the binary via `rusqlite` with `bundled` feature (no system SQLite dependency)

Connection model: `Arc<Mutex<Connection>>` -- single writer, serialized access. This is acceptable because:
- Hot-path reads are served from in-memory `DashMap` (never hit SQLite)
- Writes are batched (50 events or 2-second timer)
- Agent CRUD writes are infrequent (human-speed operations)

---

## Current Schema (v1)

This is the schema as implemented in the codebase today.

```sql
-- Migration tracking
CREATE TABLE schema_version (
    version INTEGER NOT NULL
);

-- Agent records
CREATE TABLE agents (
    id          TEXT PRIMARY KEY,           -- UUID v4 as string
    name        TEXT NOT NULL,              -- Human-readable agent name
    config      TEXT NOT NULL,              -- Full AgentConfig as JSON
    session_id  TEXT,                       -- Claude Code session ID (for --resume)
    status      TEXT DEFAULT 'idle',        -- idle | running | stopped | error
    usage       TEXT DEFAULT '{}',          -- TokenUsage as JSON
    created_at  TEXT NOT NULL,              -- RFC 3339 timestamp
    updated_at  TEXT NOT NULL               -- RFC 3339 timestamp
);

-- Event log (append-only)
CREATE TABLE events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id    TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    event_type  TEXT NOT NULL,              -- system | assistant | user | tool_use | tool_result | result
    event       TEXT NOT NULL,              -- Raw Claude stream-json event as JSON
    timestamp   TEXT NOT NULL               -- RFC 3339 timestamp
);

CREATE INDEX idx_events_agent ON events(agent_id, timestamp);
```

### Data Types in JSON Columns

**agents.config** stores `AgentConfig`:
```json
{
    "name": "Planner",
    "model": "opus",
    "system_prompt": "You are a software architect...",
    "append_system_prompt": null,
    "permission_mode": "plan",
    "allowed_tools": [],
    "disallowed_tools": [],
    "mcp_servers": {
        "gitnexus": {
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "gitnexus@latest", "mcp"],
            "url": null,
            "env": {},
            "headers": {}
        }
    },
    "hooks": null,
    "max_budget_usd": null,
    "max_turns": null,
    "working_directory": "/Users/bm/project",
    "additional_dirs": [],
    "chrome_enabled": false,
    "use_max": false,
    "use_gitnexus": true,
    "worktree": null,
    "json_schema": null,
    "subagents": null
}
```

**agents.usage** stores `TokenUsage`:
```json
{
    "input_tokens": 15420,
    "output_tokens": 3210,
    "cache_creation_tokens": 5000,
    "cache_read_tokens": 12000,
    "estimated_cost_usd": 0.1234
}
```

**events.event** stores raw Claude stream-json:
```json
{
    "type": "assistant",
    "message": {
        "id": "msg_...",
        "role": "assistant",
        "content": [
            { "type": "text", "text": "Here is the plan..." }
        ],
        "usage": {
            "input_tokens": 5140,
            "output_tokens": 1070,
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 4800
        }
    }
}
```

---

## Target Schema (v5)

The complete schema including all planned features. Migrations will be incremental (v1->v2->v3->v4->v5).

```sql
-- ===================================================================
-- MIGRATION TRACKING
-- ===================================================================

CREATE TABLE schema_version (
    version     INTEGER NOT NULL,
    applied_at  TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT
);

-- ===================================================================
-- CORE: AGENTS
-- ===================================================================

CREATE TABLE agents (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    config          TEXT NOT NULL,              -- AgentConfig JSON
    session_id      TEXT,
    status          TEXT NOT NULL DEFAULT 'idle'
                    CHECK (status IN ('idle','running','stopped','error')),
    usage           TEXT NOT NULL DEFAULT '{}', -- TokenUsage JSON
    preset_id       TEXT,                       -- Source preset (if created from one)
    group_id        TEXT,                       -- Agent group (for coordination)
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    archived_at     TEXT                        -- Soft-delete timestamp
);

CREATE INDEX idx_agents_status ON agents(status) WHERE archived_at IS NULL;
CREATE INDEX idx_agents_group ON agents(group_id) WHERE group_id IS NOT NULL;
CREATE INDEX idx_agents_preset ON agents(preset_id) WHERE preset_id IS NOT NULL;

-- ===================================================================
-- CORE: EVENTS (append-only log)
-- ===================================================================

CREATE TABLE events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id    TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    event_type  TEXT NOT NULL,
    event       TEXT NOT NULL,                 -- Raw JSON
    timestamp   TEXT NOT NULL,
    session_id  TEXT,                          -- Denormalized for faster queries
    prompt_id   TEXT                           -- Groups events to a single prompt/response cycle
);

CREATE INDEX idx_events_agent_time ON events(agent_id, timestamp);
CREATE INDEX idx_events_type ON events(event_type, timestamp);
CREATE INDEX idx_events_session ON events(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX idx_events_prompt ON events(prompt_id) WHERE prompt_id IS NOT NULL;

-- ===================================================================
-- WORKFLOWS
-- ===================================================================

CREATE TABLE workflows (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    definition      TEXT NOT NULL,             -- DAG definition as JSON
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    archived_at     TEXT
);

CREATE TABLE workflow_runs (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','running','paused','completed','failed','cancelled')),
    started_at      TEXT,
    completed_at    TEXT,
    total_cost_usd  REAL DEFAULT 0.0,
    error           TEXT,
    context         TEXT                       -- Shared context JSON passed between steps
);

CREATE INDEX idx_wf_runs_workflow ON workflow_runs(workflow_id, started_at DESC);
CREATE INDEX idx_wf_runs_status ON workflow_runs(status);

CREATE TABLE workflow_steps (
    id              TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    step_name       TEXT NOT NULL,
    agent_id        TEXT REFERENCES agents(id),
    status          TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','running','completed','failed','skipped')),
    depends_on      TEXT,                      -- JSON array of step IDs this depends on
    input           TEXT,                      -- Input context JSON
    output          TEXT,                      -- Output/result JSON
    started_at      TEXT,
    completed_at    TEXT,
    cost_usd        REAL DEFAULT 0.0
);

CREATE INDEX idx_wf_steps_run ON workflow_steps(run_id, step_name);
CREATE INDEX idx_wf_steps_agent ON workflow_steps(agent_id) WHERE agent_id IS NOT NULL;

-- ===================================================================
-- SKILLS
-- ===================================================================

CREATE TABLE skills (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT NOT NULL,
    category        TEXT NOT NULL
                    CHECK (category IN ('coding','review','testing','docs','security','workflow','custom')),
    prompt_template TEXT NOT NULL,             -- Template with {{arg}} placeholders
    source          TEXT NOT NULL DEFAULT 'builtin'
                    CHECK (source IN ('builtin','preset','user','repo')),
    source_ref      TEXT,                      -- Reference repo slug or preset ID
    arguments       TEXT,                      -- JSON schema for arguments
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX idx_skills_category ON skills(category);
CREATE INDEX idx_skills_name ON skills(name);

-- ===================================================================
-- CUSTOM PRESETS
-- ===================================================================

CREATE TABLE custom_presets (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    icon            TEXT,
    config          TEXT NOT NULL,             -- AgentConfig JSON (template)
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- ===================================================================
-- AGENT GROUPS (for coordination)
-- ===================================================================

CREATE TABLE agent_groups (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    coordination    TEXT NOT NULL DEFAULT 'independent'
                    CHECK (coordination IN ('independent','sequential','parallel','supervisor')),
    supervisor_id   TEXT REFERENCES agents(id),
    created_at      TEXT NOT NULL
);

CREATE INDEX idx_groups_coord ON agent_groups(coordination);

-- ===================================================================
-- COST TRACKING (daily aggregates for historical analysis)
-- ===================================================================

CREATE TABLE cost_daily (
    date            TEXT NOT NULL,             -- YYYY-MM-DD
    agent_id        TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    model           TEXT NOT NULL,
    input_tokens    INTEGER NOT NULL DEFAULT 0,
    output_tokens   INTEGER NOT NULL DEFAULT 0,
    cache_create    INTEGER NOT NULL DEFAULT 0,
    cache_read      INTEGER NOT NULL DEFAULT 0,
    cost_usd        REAL NOT NULL DEFAULT 0.0,
    prompt_count    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, agent_id)
);

CREATE INDEX idx_cost_date ON cost_daily(date);

-- ===================================================================
-- FTS5: FULL-TEXT SEARCH ON EVENTS
-- ===================================================================

CREATE VIRTUAL TABLE fts_events USING fts5(
    agent_id UNINDEXED,
    event_type UNINDEXED,
    content,                                   -- Extracted text from events
    timestamp UNINDEXED,
    content='events',
    content_rowid='id',
    tokenize='porter unicode61'
);

-- Triggers to keep FTS in sync with events table
CREATE TRIGGER events_ai AFTER INSERT ON events BEGIN
    INSERT INTO fts_events(rowid, agent_id, event_type, content, timestamp)
    VALUES (
        new.id,
        new.agent_id,
        new.event_type,
        CASE
            WHEN new.event_type IN ('assistant', 'user', 'result')
            THEN json_extract(new.event, '$.message.content[0].text')
            WHEN new.event_type = 'tool_use'
            THEN json_extract(new.event, '$.name') || ' ' ||
                 COALESCE(json_extract(new.event, '$.input.command'), '')
            ELSE ''
        END,
        new.timestamp
    );
END;

CREATE TRIGGER events_ad AFTER DELETE ON events BEGIN
    INSERT INTO fts_events(fts_events, rowid, agent_id, event_type, content, timestamp)
    VALUES ('delete', old.id, old.agent_id, old.event_type, '', old.timestamp);
END;

-- ===================================================================
-- CONFIGURATION (key-value store for runtime settings)
-- ===================================================================

CREATE TABLE config (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- Seed with defaults
INSERT OR IGNORE INTO config (key, value, updated_at) VALUES
    ('default_model', '"sonnet"', datetime('now')),
    ('global_max_budget_usd', 'null', datetime('now')),
    ('event_retention_days', '90', datetime('now'));
```

---

## Entity Relationship Diagram

```
+------------------+        +------------------+        +-------------------+
|  agent_groups    |        |     agents       |        |  custom_presets   |
+------------------+        +------------------+        +-------------------+
| id (PK)         |<----+  | id (PK)          |------->| id (PK)           |
| name             |    |  | name              |  preset| name              |
| description      |    |  | config (JSON)     |  _id   | description       |
| coordination     |    +--| group_id (FK)     |        | icon              |
| supervisor_id(FK)|------>| preset_id         |        | config (JSON)     |
| created_at       |       | session_id        |        | created_at        |
+------------------+       | status            |        | updated_at        |
                           | usage (JSON)      |        +-------------------+
                           | created_at        |
                           | updated_at        |
                           | archived_at       |
                           +--------+---------+
                                    |
                   +----------------+----------------+
                   |                                  |
          +--------v---------+              +---------v---------+
          |     events       |              |   cost_daily      |
          +------------------+              +-------------------+
          | id (PK, auto)    |              | date (PK)         |
          | agent_id (FK)    |              | agent_id (PK, FK) |
          | event_type       |              | model             |
          | event (JSON)     |              | input_tokens      |
          | timestamp        |              | output_tokens     |
          | session_id       |              | cache_create      |
          | prompt_id        |              | cache_read        |
          +------------------+              | cost_usd          |
                   |                        | prompt_count      |
          +--------v---------+              +-------------------+
          |   fts_events     |
          | (FTS5 virtual)   |
          +------------------+
          | content          |
          | (synced via      |
          |  triggers)       |
          +------------------+


+------------------+        +------------------+        +-------------------+
|    workflows     |        |  workflow_runs   |        | workflow_steps    |
+------------------+        +------------------+        +-------------------+
| id (PK)          |<-------| id (PK)          |<------| id (PK)           |
| name             |  wf_id | workflow_id (FK)  |  run  | run_id (FK)       |
| description      |        | status            |  _id  | step_name         |
| definition (JSON)|        | started_at        |       | agent_id (FK)---->agents
| created_at       |        | completed_at      |       | status            |
| updated_at       |        | total_cost_usd    |       | depends_on (JSON) |
| archived_at      |        | error             |       | input (JSON)      |
+------------------+        | context (JSON)    |       | output (JSON)     |
                            +------------------+        | started_at        |
                                                        | completed_at      |
                                                        | cost_usd          |
+------------------+        +------------------+        +-------------------+
|     skills       |        |     config       |
+------------------+        +------------------+
| id (PK)          |        | key (PK)         |
| name (UNIQUE)    |        | value            |
| description      |        | updated_at       |
| category         |        +------------------+
| prompt_template  |
| source           |
| source_ref       |
| arguments (JSON) |
| created_at       |
| updated_at       |
+------------------+
```

---

## Migration Strategy

### Principles

1. **Forward-only migrations** -- no rollbacks. Each migration is a numbered function.
2. **Automatic on startup** -- the `Db::open()` method checks `schema_version` and applies pending migrations.
3. **Idempotent DDL** -- all `CREATE TABLE` use `IF NOT EXISTS`, all `CREATE INDEX` use `IF NOT EXISTS`.
4. **Data preservation** -- migrations never drop columns. Schema changes use `ALTER TABLE ADD COLUMN` or create new tables.
5. **SQLite limitations** -- SQLite does not support `DROP COLUMN` (before 3.35) or `ALTER COLUMN`. For type changes, create new table, copy data, drop old, rename.

### Migration Implementation

```rust
// In db.rs
const SCHEMA_VERSION: u32 = 5;  // Target version

fn migrate(&self) -> Result<(), String> {
    let conn = self.conn.lock().unwrap();

    // Ensure version table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now')),
            description TEXT
        );"
    )?;

    let current: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [], |r| r.get(0)
    ).unwrap_or(0);

    if current < 1 { self.migrate_v1(&conn)?; }
    if current < 2 { self.migrate_v2(&conn)?; }
    if current < 3 { self.migrate_v3(&conn)?; }
    if current < 4 { self.migrate_v4(&conn)?; }
    if current < 5 { self.migrate_v5(&conn)?; }

    Ok(())
}
```

### Migration Plan

| Version | Description | Tables Affected |
|---------|-------------|----------------|
| v1 | Initial schema (current) | schema_version, agents, events |
| v2 | Add workflows support | workflows, workflow_runs, workflow_steps |
| v3 | Add skills and custom presets | skills, custom_presets |
| v4 | Add agent groups, cost tracking, FTS5 | agent_groups, cost_daily, fts_events + triggers; ALTER agents ADD group_id, preset_id, archived_at |
| v5 | Add config table, enhance events | config; ALTER events ADD session_id, prompt_id; add new indexes |

### v2 Migration (Workflows)

```sql
-- Migration v2: Workflow support
CREATE TABLE IF NOT EXISTS workflows (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    definition      TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    archived_at     TEXT
);

CREATE TABLE IF NOT EXISTS workflow_runs (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'pending',
    started_at      TEXT,
    completed_at    TEXT,
    total_cost_usd  REAL DEFAULT 0.0,
    error           TEXT,
    context         TEXT
);

CREATE TABLE IF NOT EXISTS workflow_steps (
    id              TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    step_name       TEXT NOT NULL,
    agent_id        TEXT REFERENCES agents(id),
    status          TEXT NOT NULL DEFAULT 'pending',
    depends_on      TEXT,
    input           TEXT,
    output          TEXT,
    started_at      TEXT,
    completed_at    TEXT,
    cost_usd        REAL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_wf_runs_workflow ON workflow_runs(workflow_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_wf_steps_run ON workflow_steps(run_id, step_name);

INSERT INTO schema_version (version, description) VALUES (2, 'Add workflow support');
```

### v3 Migration (Skills, Presets)

```sql
-- Migration v3: Skills and custom presets
CREATE TABLE IF NOT EXISTS skills (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT NOT NULL,
    category        TEXT NOT NULL,
    prompt_template TEXT NOT NULL,
    source          TEXT NOT NULL DEFAULT 'builtin',
    source_ref      TEXT,
    arguments       TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS custom_presets (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    icon            TEXT,
    config          TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);

INSERT INTO schema_version (version, description) VALUES (3, 'Add skills and custom presets');
```

### v4 Migration (Groups, Cost, FTS)

```sql
-- Migration v4: Agent groups, cost tracking, full-text search
CREATE TABLE IF NOT EXISTS agent_groups (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    coordination    TEXT NOT NULL DEFAULT 'independent',
    supervisor_id   TEXT REFERENCES agents(id),
    created_at      TEXT NOT NULL
);

ALTER TABLE agents ADD COLUMN group_id TEXT;
ALTER TABLE agents ADD COLUMN preset_id TEXT;
ALTER TABLE agents ADD COLUMN archived_at TEXT;

CREATE TABLE IF NOT EXISTS cost_daily (
    date            TEXT NOT NULL,
    agent_id        TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    model           TEXT NOT NULL,
    input_tokens    INTEGER NOT NULL DEFAULT 0,
    output_tokens   INTEGER NOT NULL DEFAULT 0,
    cache_create    INTEGER NOT NULL DEFAULT 0,
    cache_read      INTEGER NOT NULL DEFAULT 0,
    cost_usd        REAL NOT NULL DEFAULT 0.0,
    prompt_count    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, agent_id)
);

-- FTS5 virtual table
CREATE VIRTUAL TABLE IF NOT EXISTS fts_events USING fts5(
    agent_id UNINDEXED,
    event_type UNINDEXED,
    content,
    timestamp UNINDEXED,
    content='events',
    content_rowid='id',
    tokenize='porter unicode61'
);

-- Backfill FTS from existing events
INSERT INTO fts_events(rowid, agent_id, event_type, content, timestamp)
SELECT
    id,
    agent_id,
    event_type,
    CASE
        WHEN event_type IN ('assistant', 'user', 'result')
        THEN json_extract(event, '$.message.content[0].text')
        WHEN event_type = 'tool_use'
        THEN json_extract(event, '$.name')
        ELSE ''
    END,
    timestamp
FROM events;

-- Triggers for FTS sync
CREATE TRIGGER IF NOT EXISTS events_ai AFTER INSERT ON events BEGIN
    INSERT INTO fts_events(rowid, agent_id, event_type, content, timestamp)
    VALUES (
        new.id, new.agent_id, new.event_type,
        CASE
            WHEN new.event_type IN ('assistant', 'user', 'result')
            THEN json_extract(new.event, '$.message.content[0].text')
            WHEN new.event_type = 'tool_use'
            THEN json_extract(new.event, '$.name') || ' ' ||
                 COALESCE(json_extract(new.event, '$.input.command'), '')
            ELSE ''
        END,
        new.timestamp
    );
END;

CREATE TRIGGER IF NOT EXISTS events_ad AFTER DELETE ON events BEGIN
    INSERT INTO fts_events(fts_events, rowid, agent_id, event_type, content, timestamp)
    VALUES ('delete', old.id, old.agent_id, old.event_type, '', old.timestamp);
END;

INSERT INTO schema_version (version, description) VALUES (4, 'Add groups, cost, FTS');
```

### v5 Migration (Config, Enhanced Events)

```sql
-- Migration v5: Config store, enhanced event indexing
CREATE TABLE IF NOT EXISTS config (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

INSERT OR IGNORE INTO config (key, value, updated_at) VALUES
    ('default_model', '"sonnet"', datetime('now')),
    ('global_max_budget_usd', 'null', datetime('now')),
    ('event_retention_days', '90', datetime('now'));

ALTER TABLE events ADD COLUMN session_id TEXT;
ALTER TABLE events ADD COLUMN prompt_id TEXT;

CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_prompt ON events(prompt_id) WHERE prompt_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status) WHERE archived_at IS NULL;

INSERT INTO schema_version (version, description) VALUES (5, 'Add config store and enhanced events');
```

---

## Data Lifecycle

### Agent Lifecycle

```
Create                  Update              Archive              Delete
  |                       |                    |                    |
  v                       v                    v                    v
INSERT INTO agents   UPDATE agents SET   UPDATE agents SET     DELETE FROM events
  (immediate)        config=?, name=?    archived_at=NOW()     WHERE agent_id=?;
                     (immediate)          (soft delete)         DELETE FROM agents
                                                                WHERE id=?;
                                                                (cascade events)
```

**Retention policy:**
- Active agents: kept indefinitely
- Archived agents: kept for `event_retention_days` (default 90), then hard-deleted
- Archival is soft-delete: `archived_at IS NOT NULL` filters them from default queries

### Event Lifecycle

```
Claude stdout line
       |
       v
  Stream Reader (parse JSON, tag with agent_id)
       |
       v
  Broadcast channel (in-memory, ephemeral)
       |
       +---> WebSocket clients (real-time display, not stored)
       +---> Event accumulator (buffers in Vec)
       |          |
       |     50 events OR 2 seconds
       |          |
       |          v
       |     Batch INSERT into events table
       |     Update agents.usage and agents.session_id
       |
       +---> Agent ring buffer (VecDeque, max 10K events, in-memory)
                  |
             On startup: populated from DB (load_events)
```

**Retention policy:**
- Events older than `event_retention_days` are eligible for deletion
- Planned: nightly vacuum job deletes old events and runs `PRAGMA incremental_vacuum`
- FTS entries auto-deleted via trigger when event rows are deleted

### Cost Data Lifecycle

```
  assistant event with usage
       |
       v
  Extract UsageDelta (input/output/cache tokens)
       |
       v
  Accumulate on AgentHandle.usage (in-memory)
       |
       v
  Persist to agents.usage (on batch flush)
       |
       v
  Roll up to cost_daily (planned: daily aggregation job)
       |
       v
  cost_daily kept indefinitely (small: 1 row per agent per day)
```

---

## Query Patterns

### Hot Queries (served from memory)

These queries NEVER hit SQLite during normal operation:

| Query | Source | Access Pattern |
|-------|--------|---------------|
| List all agents | `DashMap.iter()` | O(n) scan, n = agent count |
| Get single agent | `DashMap.get(&id)` | O(1) hash lookup |
| Get recent events | `AgentHandle.events` | VecDeque slice, last 100 |
| Get cost summary | `DashMap.iter()` | Sum over agent usages |

### Warm Queries (hit SQLite, should be fast)

| Query | SQL | Index Used | Expected Latency |
|-------|-----|-----------|-----------------|
| Agent events by time range | `SELECT * FROM events WHERE agent_id=? AND timestamp BETWEEN ? AND ? ORDER BY id ASC LIMIT ? OFFSET ?` | `idx_events_agent_time` | <5ms for 1000 rows |
| Events by type | `SELECT * FROM events WHERE event_type=? AND timestamp > ? ORDER BY timestamp DESC LIMIT ?` | `idx_events_type` | <5ms |
| Full-text search | `SELECT e.* FROM events e JOIN fts_events f ON e.id=f.rowid WHERE fts_events MATCH ? ORDER BY rank LIMIT ?` | FTS5 index | <20ms |
| Workflow runs | `SELECT * FROM workflow_runs WHERE workflow_id=? ORDER BY started_at DESC LIMIT ?` | `idx_wf_runs_workflow` | <1ms |
| Cost by date range | `SELECT date, SUM(cost_usd) FROM cost_daily WHERE date BETWEEN ? AND ? GROUP BY date` | `idx_cost_date` | <1ms |

### Cold Queries (acceptable to be slower)

| Query | SQL | Expected Latency |
|-------|-----|-----------------|
| Export all events for agent | `SELECT * FROM events WHERE agent_id=? ORDER BY id ASC` | <100ms for 100K events |
| Rebuild FTS index | `INSERT INTO fts_events(fts_events) VALUES('rebuild')` | Seconds (rare, maintenance only) |
| Total cost across all time | `SELECT SUM(cost_usd) FROM cost_daily` | <10ms |

### Query Optimization Notes

1. **Covering indexes**: The `idx_events_agent_time` index covers the most common query pattern (events for an agent sorted by time).

2. **Partial indexes**: `WHERE archived_at IS NULL` and `WHERE session_id IS NOT NULL` reduce index size by excluding irrelevant rows.

3. **JSON extraction**: `json_extract()` is used in triggers (write-time) not queries (read-time). Read queries never parse JSON in SQLite -- they return raw JSON to Rust for deserialization.

4. **LIMIT/OFFSET pagination**: Used for event loading. For large result sets, cursor-based pagination (using `id > last_seen_id`) would be more efficient and is planned for v2 API.

---

## FTS5 Configuration

### What Is Searchable

| Event Type | Indexed Content | Example |
|-----------|----------------|---------|
| `assistant` | Text blocks from `message.content[0].text` | "Here is the implementation of the sort function..." |
| `user` | Text blocks from `message.content[0].text` | "Please refactor the auth module" |
| `result` | Result text from `message.content[0].text` | "Completed. Modified 3 files." |
| `tool_use` | Tool name + command input | "Bash npm test" |
| `tool_result` | (not indexed -- too noisy, contains full file contents) | |
| `system` | (not indexed -- contains only session metadata) | |

### Tokenizer

```
tokenize='porter unicode61'
```

- **porter**: English stemming (search for "running" finds "run", "runs", "runner")
- **unicode61**: Unicode-aware tokenization, case-insensitive, handles non-ASCII

### Search API

```sql
-- Basic search
SELECT e.agent_id, e.event_type, e.event, e.timestamp
FROM events e
JOIN fts_events f ON e.id = f.rowid
WHERE fts_events MATCH 'refactor auth'
ORDER BY f.rank
LIMIT 20;

-- Search within specific agent
SELECT e.*
FROM events e
JOIN fts_events f ON e.id = f.rowid
WHERE fts_events MATCH 'refactor auth'
  AND f.agent_id = ?
ORDER BY f.rank
LIMIT 20;

-- Search by event type
SELECT e.*
FROM events e
JOIN fts_events f ON e.id = f.rowid
WHERE fts_events MATCH 'error handling'
  AND f.event_type = 'assistant'
ORDER BY e.timestamp DESC
LIMIT 20;

-- Phrase search
WHERE fts_events MATCH '"sort function"'

-- Boolean search
WHERE fts_events MATCH 'refactor AND NOT test'

-- Prefix search
WHERE fts_events MATCH 'auth*'
```

### FTS Maintenance

```sql
-- Optimize FTS index (merge segments, run periodically)
INSERT INTO fts_events(fts_events) VALUES('optimize');

-- Rebuild entire FTS index (if corrupted or after bulk import)
INSERT INTO fts_events(fts_events) VALUES('rebuild');

-- Check integrity
INSERT INTO fts_events(fts_events) VALUES('integrity-check');
```

---

## Batch Write Strategy

The current implementation uses a dual-trigger batch write strategy for events:

```
                    Events arrive via broadcast channel
                              |
                    +---------v---------+
                    | Event Accumulator |
                    | (tokio task)      |
                    +---------+---------+
                              |
                    +---------v---------+
                    | pending_events:   |
                    | Vec<TaggedEvent>  |
                    +---+----------+----+
                        |          |
               +--------+          +--------+
               |                            |
        Batch size >= 50?           Timer >= 2 seconds?
        (throughput trigger)        (latency trigger)
               |                            |
               +--------+   +--------------+
                        |   |
                   +----v---v----+
                   | FLUSH       |
                   |             |
                   | 1. INSERT events (batch)
                   | 2. UPDATE agents (usage + session_id)
                   | 3. Clear pending buffer
                   | 4. Reset timer
                   +-------------+
```

### Design Rationale

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Batch size | 50 events | Amortizes SQLite transaction overhead. 50 INSERTs in one transaction is ~10x faster than 50 individual transactions |
| Timer | 2 seconds | Ensures events are persisted within 2 seconds even under low throughput. Acceptable for crash recovery |
| Buffer type | `Vec<TaggedEvent>` | Simple, grows as needed, cleared on flush |

### Throughput Analysis

- A typical Claude response produces 5-20 events (system, assistant chunks, tool calls, result)
- A busy session with 3 concurrent agents might produce ~50 events/second
- Batch write at 50 events: ~1 transaction/second under load
- Each transaction: ~1-5ms (WAL mode, SSD)
- Maximum sustainable throughput: ~10,000 events/second (well above any real usage)

### Crash Recovery

If Forge crashes between flushes:
- Events in the pending buffer (max 50 events or 2 seconds worth) are lost
- Agent state (usage counters) may be slightly behind
- On restart, agents are loaded from DB with their last-persisted state
- Lost events are acceptable: Claude Code's own session files in `~/.claude/projects/` serve as the authoritative log

---

## Storage Estimates

### Per-Event Storage

| Component | Typical Size |
|-----------|-------------|
| events row (small event, e.g., system) | ~200 bytes |
| events row (assistant with text) | ~2-5 KB |
| events row (tool_result with file content) | ~5-50 KB |
| FTS entry | ~50-200 bytes |
| Average across event types | ~3 KB |

### Per-Session Storage

| Metric | Typical Value |
|--------|--------------|
| Events per prompt/response cycle | 5-20 |
| Average session length | 10-50 prompt cycles |
| Events per session | 50-1000 |
| Storage per session | 150 KB - 3 MB |

### Growth Projections

| Usage Level | Sessions/Day | Events/Day | Storage/Day | Storage/Month |
|------------|-------------|-----------|------------|--------------|
| Light (solo dev) | 5-10 | 500-5,000 | 1.5-15 MB | 45-450 MB |
| Moderate (team) | 20-50 | 5,000-25,000 | 15-75 MB | 450 MB - 2.2 GB |
| Heavy (CI + team) | 100+ | 50,000+ | 150+ MB | 4.5+ GB |

### Database Maintenance

```sql
-- Check database size
SELECT page_count * page_size AS size_bytes FROM pragma_page_count(), pragma_page_size();

-- Compact database (reclaim space after deletions)
PRAGMA incremental_vacuum(1000);  -- Free up to 1000 pages

-- Full vacuum (rewrites entire DB, requires 2x disk space temporarily)
VACUUM;

-- Analyze for query optimizer
ANALYZE;

-- Check integrity
PRAGMA integrity_check;
PRAGMA foreign_key_check;
```

### Recommended Maintenance Schedule

| Task | Frequency | Automated |
|------|-----------|-----------|
| Delete events older than retention | Daily (planned) | Yes, background task |
| `PRAGMA incremental_vacuum` | After deletion | Yes |
| `ANALYZE` | Weekly | Yes (planned) |
| `VACUUM` | Monthly (if needed) | Manual |
| FTS optimize | Weekly | Yes (planned) |
| Backup (`cp forge.db forge.db.bak`) | Before upgrades | Manual |

---

*Next: [API_DESIGN.md](API_DESIGN.md) for the complete REST and WebSocket API specification.*
