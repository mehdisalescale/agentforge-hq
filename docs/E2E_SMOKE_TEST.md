# E2E Smoke Test — Claude Forge API

**Purpose:** Verify core flow: create agent → run (with optional directory) → observe sessions/events.  
**When:** After Phase 1 polish (run uses directory, events persisted).  
**Agent D deliverable:** step-by-step E2E using curl.

---

## Prerequisites

- **Server running:** from `claude-forge` repo, `cargo run -p forge-app` (default: `http://127.0.0.1:4173`).
- **Base URL:** set for convenience (e.g. `BASE=http://127.0.0.1:4173`).

```bash
export BASE=http://127.0.0.1:4173
```

---

## 1. Create an agent

**Request:** `POST /api/v1/agents`

```bash
curl -s -X POST "$BASE/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d '{"name": "SmokeTestAgent"}' | jq .
```

**Expected:** `201` with JSON body containing `id`, `name`, `model`, etc. Save the agent `id` for the run step.

Example (save ID for next step):

```bash
AGENT_ID=$(curl -s -X POST "$BASE/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d '{"name": "SmokeTestAgent"}' | jq -r '.id')
echo "Agent ID: $AGENT_ID"
```

---

## 2. Run a prompt (optional directory)

**Request:** `POST /api/v1/run`

Body: `agent_id` (required), `prompt` (required), `session_id` (optional, for resume), `directory` (optional working directory for the process).

```bash
# Run with default working directory
curl -s -X POST "$BASE/api/v1/run" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\", \"prompt\": \"Say hello in one sentence.\"}" | jq .

# Or with an optional working directory
curl -s -X POST "$BASE/api/v1/run" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\", \"prompt\": \"List files here.\", \"directory\": \"/tmp\"}" | jq .
```

**Expected:** `202 Accepted` with `session_id` and a message. Process runs in background; events stream to WebSocket at `ws://127.0.0.1:4173/api/v1/ws`.

Example (save session ID for later):

```bash
SESSION_ID=$(curl -s -X POST "$BASE/api/v1/run" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\", \"prompt\": \"Say hello.\"}" | jq -r '.session_id')
echo "Session ID: $SESSION_ID"
```

---

## 3. List sessions

**Request:** `GET /api/v1/sessions`

```bash
curl -s "$BASE/api/v1/sessions" | jq .
```

**Expected:** `200` with a JSON array of sessions. Each session has `id`, `agent_id`, `directory`, `status`, `created_at`, etc.

**Stream live events** via WebSocket: `ws://127.0.0.1:4173/api/v1/ws`.

---

## 4. Export a session

**Request:** `GET /api/v1/sessions/:id/export?format=json`

```bash
curl -s "$BASE/api/v1/sessions/$SESSION_ID/export?format=json" | jq .
```

**Expected:** `200` with session metadata + events array in JSON. Also supports `format=markdown`.

---

## 5. Health check

**Request:** `GET /api/v1/health`

```bash
curl -s "$BASE/api/v1/health" | jq .
```

**Expected:** `200` with a health payload (e.g. `ok`, version).

---

## One-liner smoke script (create + run)

Assumes `jq` is installed and server is up:

```bash
BASE=${BASE:-http://127.0.0.1:4173}
AGENT_ID=$(curl -s -X POST "$BASE/api/v1/agents" -H "Content-Type: application/json" -d '{"name":"E2EAgent"}' | jq -r '.id')
echo "Agent: $AGENT_ID"
SESSION_ID=$(curl -s -X POST "$BASE/api/v1/run" -H "Content-Type: application/json" \
  -d "{\"agent_id\":\"$AGENT_ID\",\"prompt\":\"Hello\"}" | jq -r '.session_id')
echo "Session: $SESSION_ID"
sleep 2
curl -s "$BASE/api/v1/sessions" | jq .
curl -s "$BASE/api/v1/sessions/$SESSION_ID/export?format=json" | jq .
```

Success: create returns `id`; run returns `202` with `session_id`; sessions list includes the new session; export returns events.
