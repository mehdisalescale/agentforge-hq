# Agent W3-D: Verify & Fix Existing Pages

> You are Agent W3-D. Your job: verify that Skills, Analytics, and Settings pages actually work end-to-end. Fix any broken API calls, missing data, or rendering issues. These pages may already have UI code — your job is to make sure they function correctly with real data.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
frontend/src/routes/skills/+page.svelte            — FULL FILE
frontend/src/routes/analytics/+page.svelte         — FULL FILE
frontend/src/routes/settings/+page.svelte          — FULL FILE
frontend/src/lib/api.ts                            — API client functions
crates/forge-api/src/routes/skills.rs              — skills API
crates/forge-api/src/routes/analytics.rs           — analytics API
crates/forge-db/src/repos/skills.rs                — SkillRepo
crates/forge-db/src/repos/analytics.rs             — AnalyticsRepo
stat-qou-plan/REVISED_PLAN.md                      — what we're doing and why
```

## Step 2: Verify Skills Page

1. Read the skills page frontend code
2. Check what API calls it makes (likely `GET /api/v1/skills`)
3. Check the backend route handler
4. Verify the response format matches what the frontend expects
5. Test: does the page actually render the 30 loaded skills?

If broken, fix the mismatch. Common issues:
- Frontend expects a field the backend doesn't return
- API endpoint returns wrong format
- Skills are loaded into DB but the list endpoint doesn't return content

**Goal**: Skills page shows all 30 loaded skills with names, descriptions, and expandable content.

## Step 3: Verify Analytics Page

1. Read the analytics page frontend code
2. Check what API calls it makes (likely `GET /api/v1/analytics/usage`)
3. Check the backend route handler and what `AnalyticsRepo` provides
4. Verify with real data: after running an agent, does analytics show the run?

If the analytics endpoint returns empty data because nothing writes to it:
- Check if `BatchWriter` or `EventRepo` stores the data analytics needs
- If the data exists in events table but analytics queries a different table, fix the query

**Goal**: Analytics page shows run counts, cost data (even if zero), and session statistics.

## Step 4: Verify Settings Page

1. Read the settings page frontend code
2. Check if it reads/writes any API endpoint
3. If it's just a static page, make it read current configuration

At minimum, the settings page should show (read-only is fine):
- Server host and port
- CLI command (FORGE_CLI_COMMAND)
- Rate limit settings
- Budget thresholds
- Database path

If no settings API exists, add a simple one:

```rust
/// GET /api/v1/settings — return current runtime configuration
async fn get_settings() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "host": std::env::var("FORGE_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
        "port": std::env::var("FORGE_PORT").unwrap_or_else(|_| "4173".into()),
        "cli_command": std::env::var("FORGE_CLI_COMMAND").unwrap_or_else(|_| "claude".into()),
        "db_path": std::env::var("FORGE_DB_PATH").unwrap_or_else(|_| "~/.claude-forge/forge.db".into()),
        "rate_limit_max": std::env::var("FORGE_RATE_LIMIT_MAX").unwrap_or_else(|_| "10".into()),
        "rate_limit_refill_ms": std::env::var("FORGE_RATE_LIMIT_REFILL_MS").unwrap_or_else(|_| "1000".into()),
        "budget_warn": std::env::var("FORGE_BUDGET_WARN").ok(),
        "budget_limit": std::env::var("FORGE_BUDGET_LIMIT").ok(),
        "cors_origin": std::env::var("FORGE_CORS_ORIGIN").unwrap_or_else(|_| "*".into()),
    }))
}
```

## Step 5: Fix API Client

If the frontend `api.ts` is missing functions for skills, analytics, or settings, add them:

```typescript
export async function listSkills() {
  const res = await fetch(`${BASE}/skills`);
  if (!res.ok) throw new Error(`Failed to list skills: ${res.status}`);
  return res.json();
}

export async function getUsageReport(start: string, end: string) {
  const res = await fetch(`${BASE}/analytics/usage?start=${start}&end=${end}`);
  if (!res.ok) throw new Error(`Failed to get analytics: ${res.status}`);
  return res.json();
}

export async function getSettings() {
  const res = await fetch(`${BASE}/settings`);
  if (!res.ok) throw new Error(`Failed to get settings: ${res.status}`);
  return res.json();
}
```

## Step 6: Verify Build

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test 2>&1 | grep "FAILED"     # no failures
cd frontend && pnpm build 2>&1       # must build cleanly
```

## Rules

- You may modify: skills page, analytics page, settings page (frontend)
- You may modify: skills.rs, analytics.rs routes (backend) — add/fix endpoints
- You may modify: frontend/src/lib/api.ts — add missing API functions
- You may add a settings route if none exists
- Do NOT modify middleware.rs, run.rs — Agent W3-B handles those
- Do NOT modify sessions page — Agent W3-C handles that
- Do NOT modify +layout.svelte or the run page — Agent W3-A handles those
- Do NOT modify main.rs
- Do NOT modify existing tests — only add new ones
- Commit with: `fix: verify and fix Skills, Analytics, and Settings pages`

## Report
```
STATUS: done | blocked
FILES_MODIFIED: [list]
PAGES_VERIFIED: [skills: pass/fail, analytics: pass/fail, settings: pass/fail]
FIXES_APPLIED: [list what was broken and how you fixed it]
ISSUES: [any]
```
