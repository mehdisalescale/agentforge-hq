# TASK 12 — E2E smoke test script

**Status:** pending
**Priority:** medium
**Track:** Phase A polish

---

## Context

There's a `docs/E2E_SMOKE_TEST.md` with curl examples, but no runnable script. We need an automated script that exercises the full flow.

## Task

Create `scripts/e2e-smoke.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

BASE="${FORGE_BASE_URL:-http://127.0.0.1:4173}/api/v1"

echo "=== Health check ==="
curl -sf "$BASE/health" | jq .

echo "=== Create agent ==="
AGENT=$(curl -sf -X POST "$BASE/agents" \
  -H 'Content-Type: application/json' \
  -d '{"name":"SmokeTest"}')
AGENT_ID=$(echo "$AGENT" | jq -r .id)
echo "Agent ID: $AGENT_ID"

echo "=== Run agent ==="
RUN=$(curl -sf -X POST "$BASE/run" \
  -H 'Content-Type: application/json' \
  -d "{\"agent_id\":\"$AGENT_ID\",\"prompt\":\"Say hello in one word.\"}")
SESSION_ID=$(echo "$RUN" | jq -r .session_id)
echo "Session ID: $SESSION_ID"

echo "=== Wait for process ==="
sleep 10

echo "=== List sessions ==="
curl -sf "$BASE/sessions" | jq '.[0]'

echo "=== Export session (JSON) ==="
EVENTS=$(curl -sf "$BASE/sessions/$SESSION_ID/export?format=json" | jq '.events | length')
echo "Events: $EVENTS"

echo "=== List skills ==="
curl -sf "$BASE/skills" | jq 'length'

echo "=== List workflows ==="
curl -sf "$BASE/workflows" | jq 'length'

echo "=== Cleanup ==="
curl -sf -X DELETE "$BASE/agents/$AGENT_ID"

echo ""
echo "=== SMOKE TEST PASSED ==="
```

Make it executable: `chmod +x scripts/e2e-smoke.sh`

## Files to create

- `scripts/e2e-smoke.sh`

## Verify

Start the server in one terminal (`cargo run -p forge-app`), run `bash scripts/e2e-smoke.sh` in another. Should print "SMOKE TEST PASSED".

---

## Report

*Agent: fill this in when done.*

- [ ] What was changed:
- [ ] Script runs: yes/no
- [ ] Notes:
