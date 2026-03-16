#!/usr/bin/env bash
# Generate TypeScript types from the OpenAPI spec served by forge-app.
#
# Usage:
#   1. Start forge-app: cargo run --bin forge-app
#   2. Run this script: ./scripts/generate-api-types.sh
#
# This fetches /api/openapi.json from the running server and generates
# TypeScript interfaces into frontend/src/lib/api-types.generated.ts.
#
# Requires: npx (comes with npm/pnpm), and a running forge-app server.

set -euo pipefail

API_URL="${FORGE_API_URL:-http://127.0.0.1:3000}"
OPENAPI_URL="${API_URL}/api/openapi.json"
OUTPUT_DIR="$(cd "$(dirname "$0")/../frontend/src/lib" && pwd)"
OUTPUT_FILE="${OUTPUT_DIR}/api-types.generated.ts"

echo "Fetching OpenAPI spec from ${OPENAPI_URL}..."
if ! curl -sf "${OPENAPI_URL}" -o /tmp/forge-openapi.json; then
  echo "ERROR: Could not fetch OpenAPI spec. Is forge-app running?" >&2
  exit 1
fi

echo "Generating TypeScript types..."
npx --yes openapi-typescript /tmp/forge-openapi.json -o "${OUTPUT_FILE}"

echo "Generated: ${OUTPUT_FILE}"
echo ""
echo "NOTE: The generated types are the source of truth for API contracts."
echo "Update frontend/src/lib/api.ts to import from api-types.generated.ts"
echo "instead of manually maintaining duplicate interfaces."
