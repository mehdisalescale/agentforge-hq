#!/usr/bin/env bash
# Integration test for Claude Forge API.
# Starts the server, exercises core CRUD endpoints, then cleans up.
#
# Usage:
#   ./tests/integration_test.sh              # builds & runs in release mode
#   FORGE_BIN=./target/debug/forge ./tests/integration_test.sh  # use pre-built binary
#
# Requires: curl, jq

set -euo pipefail

###############################################################################
# Configuration
###############################################################################

PORT="${FORGE_PORT:-14173}"                       # non-default port to avoid clashes
HOST="${FORGE_HOST:-127.0.0.1}"
BASE="http://${HOST}:${PORT}/api/v1"
DB_DIR=$(mktemp -d)
DB_PATH="${DB_DIR}/integration_test.db"
FORGE_BIN="${FORGE_BIN:-}"
SERVER_PID=""
PASS=0
FAIL=0

###############################################################################
# Helpers
###############################################################################

log()  { printf "\033[1;34m==> %s\033[0m\n" "$*"; }
ok()   { printf "\033[1;32m  OK: %s\033[0m\n" "$*"; PASS=$((PASS + 1)); }
fail() { printf "\033[1;31m  FAIL: %s\033[0m\n" "$*"; FAIL=$((FAIL + 1)); }

cleanup() {
    log "Cleaning up"
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    rm -rf "$DB_DIR"
    echo ""
    echo "-------------------------------------------"
    echo "Results: $PASS passed, $FAIL failed"
    echo "-------------------------------------------"
    if [ "$FAIL" -gt 0 ]; then
        exit 1
    fi
}
trap cleanup EXIT

assert_eq() {
    local label="$1" expected="$2" actual="$3"
    if [ "$expected" = "$actual" ]; then
        ok "$label"
    else
        fail "$label (expected '$expected', got '$actual')"
    fi
}

assert_not_empty() {
    local label="$1" value="$2"
    if [ -n "$value" ] && [ "$value" != "null" ]; then
        ok "$label"
    else
        fail "$label (got empty or null)"
    fi
}

###############################################################################
# Build (if needed)
###############################################################################

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ -z "$FORGE_BIN" ]; then
    log "Building forge (release)"
    cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml" 2>&1
    FORGE_BIN="$PROJECT_DIR/target/release/forge"
fi

if [ ! -x "$FORGE_BIN" ]; then
    echo "ERROR: Binary not found or not executable: $FORGE_BIN"
    exit 1
fi

###############################################################################
# Start server
###############################################################################

log "Starting forge server on ${HOST}:${PORT} (db: $DB_PATH)"
FORGE_PORT="$PORT" FORGE_HOST="$HOST" FORGE_DB_PATH="$DB_PATH" \
    "$FORGE_BIN" > /dev/null 2>&1 &
SERVER_PID=$!

# Wait for health endpoint (up to 15 seconds)
log "Waiting for server to be ready"
READY=0
for i in $(seq 1 30); do
    if curl -sf "${BASE}/health" > /dev/null 2>&1; then
        READY=1
        break
    fi
    sleep 0.5
done

if [ "$READY" -eq 0 ]; then
    fail "Server did not become ready within 15 seconds"
    exit 1
fi
ok "Server is healthy"

###############################################################################
# 1. Health check
###############################################################################

log "Health check"
HEALTH=$(curl -sf "${BASE}/health")
HEALTH_STATUS=$(echo "$HEALTH" | jq -r '.status // empty')
assert_eq "Health status is ok" "ok" "$HEALTH_STATUS"

###############################################################################
# 2. Agents CRUD
###############################################################################

log "Create agent"
AGENT=$(curl -sf -X POST "${BASE}/agents" \
    -H 'Content-Type: application/json' \
    -d '{"name":"IntegrationTestAgent","model":"claude-sonnet-4-20250514"}')
AGENT_ID=$(echo "$AGENT" | jq -r '.id')
AGENT_NAME=$(echo "$AGENT" | jq -r '.name')
assert_not_empty "Agent ID returned" "$AGENT_ID"
assert_eq "Agent name matches" "IntegrationTestAgent" "$AGENT_NAME"

log "List agents"
AGENTS_LIST=$(curl -sf "${BASE}/agents")
AGENTS_COUNT=$(echo "$AGENTS_LIST" | jq 'length')
assert_eq "Agents list has 1 entry" "1" "$AGENTS_COUNT"
LISTED_ID=$(echo "$AGENTS_LIST" | jq -r '.[0].id')
assert_eq "Listed agent ID matches" "$AGENT_ID" "$LISTED_ID"

log "Get agent by ID"
FETCHED=$(curl -sf "${BASE}/agents/${AGENT_ID}")
FETCHED_NAME=$(echo "$FETCHED" | jq -r '.name')
assert_eq "Fetched agent name matches" "IntegrationTestAgent" "$FETCHED_NAME"

log "Update agent"
UPDATED=$(curl -sf -X PUT "${BASE}/agents/${AGENT_ID}" \
    -H 'Content-Type: application/json' \
    -d '{"name":"UpdatedAgent"}')
UPDATED_NAME=$(echo "$UPDATED" | jq -r '.name')
assert_eq "Updated agent name" "UpdatedAgent" "$UPDATED_NAME"

###############################################################################
# 3. Sessions CRUD
###############################################################################

log "Create session"
SESSION=$(curl -sf -X POST "${BASE}/sessions" \
    -H 'Content-Type: application/json' \
    -d "{\"agent_id\":\"${AGENT_ID}\",\"directory\":\"/tmp/test-project\"}")
SESSION_ID=$(echo "$SESSION" | jq -r '.id')
SESSION_STATUS=$(echo "$SESSION" | jq -r '.status')
SESSION_DIR=$(echo "$SESSION" | jq -r '.directory')
assert_not_empty "Session ID returned" "$SESSION_ID"
assert_eq "Session status is pending" "pending" "$SESSION_STATUS"
assert_eq "Session directory matches" "/tmp/test-project" "$SESSION_DIR"

log "Create worktree session"
WT_SESSION=$(curl -sf -X POST "${BASE}/sessions" \
    -H 'Content-Type: application/json' \
    -d "{\"agent_id\":\"${AGENT_ID}\",\"directory\":\"/home/user/project/.claude/worktrees/feature-x\"}")
WT_SESSION_ID=$(echo "$WT_SESSION" | jq -r '.id')
WT_DIR=$(echo "$WT_SESSION" | jq -r '.directory')
assert_not_empty "Worktree session ID returned" "$WT_SESSION_ID"
assert_eq "Worktree directory matches" "/home/user/project/.claude/worktrees/feature-x" "$WT_DIR"

log "List sessions"
SESSIONS_LIST=$(curl -sf "${BASE}/sessions")
SESSIONS_COUNT=$(echo "$SESSIONS_LIST" | jq 'length')
assert_eq "Sessions list has 2 entries" "2" "$SESSIONS_COUNT"

log "Get session by ID"
FETCHED_SESSION=$(curl -sf "${BASE}/sessions/${SESSION_ID}")
FETCHED_SID=$(echo "$FETCHED_SESSION" | jq -r '.id')
assert_eq "Fetched session ID matches" "$SESSION_ID" "$FETCHED_SID"

log "Export session (JSON)"
EXPORT_JSON=$(curl -sf "${BASE}/sessions/${SESSION_ID}/export?format=json")
EXPORT_SID=$(echo "$EXPORT_JSON" | jq -r '.session.id')
assert_eq "Export contains correct session" "$SESSION_ID" "$EXPORT_SID"

log "Export session (Markdown)"
EXPORT_MD=$(curl -sf "${BASE}/sessions/${SESSION_ID}/export?format=markdown")
if echo "$EXPORT_MD" | grep -q "Session $SESSION_ID"; then
    ok "Markdown export contains session ID"
else
    fail "Markdown export missing session ID"
fi

###############################################################################
# 4. Skills & Workflows (list — may be empty, just check 200)
###############################################################################

log "List skills"
SKILLS_RESP=$(curl -sf -o /dev/null -w "%{http_code}" "${BASE}/skills")
assert_eq "Skills endpoint returns 200" "200" "$SKILLS_RESP"

log "List workflows"
WF_RESP=$(curl -sf -o /dev/null -w "%{http_code}" "${BASE}/workflows")
assert_eq "Workflows endpoint returns 200" "200" "$WF_RESP"

###############################################################################
# 5. Cleanup via API
###############################################################################

log "Delete session"
DEL_SESSION_CODE=$(curl -sf -o /dev/null -w "%{http_code}" -X DELETE "${BASE}/sessions/${SESSION_ID}")
assert_eq "Delete session returns 204" "204" "$DEL_SESSION_CODE"

log "Delete worktree session"
DEL_WT_CODE=$(curl -sf -o /dev/null -w "%{http_code}" -X DELETE "${BASE}/sessions/${WT_SESSION_ID}")
assert_eq "Delete worktree session returns 204" "204" "$DEL_WT_CODE"

log "Verify sessions deleted"
SESSIONS_AFTER=$(curl -sf "${BASE}/sessions")
SESSIONS_AFTER_COUNT=$(echo "$SESSIONS_AFTER" | jq 'length')
assert_eq "Sessions list empty after delete" "0" "$SESSIONS_AFTER_COUNT"

log "Delete agent"
DEL_AGENT_CODE=$(curl -sf -o /dev/null -w "%{http_code}" -X DELETE "${BASE}/agents/${AGENT_ID}")
assert_eq "Delete agent returns 204" "204" "$DEL_AGENT_CODE"

log "Verify agents deleted"
AGENTS_AFTER=$(curl -sf "${BASE}/agents")
AGENTS_AFTER_COUNT=$(echo "$AGENTS_AFTER" | jq 'length')
assert_eq "Agents list empty after delete" "0" "$AGENTS_AFTER_COUNT"

###############################################################################
# Done — cleanup happens via trap
###############################################################################

echo ""
log "Integration test complete"
