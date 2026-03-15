# Fix Proposals — Prioritized Action Plan

> **Date:** 2026-03-15
> **Scope:** Fixes derived from site-vs-codebase inconsistency analysis
> **Approach:** Grouped by priority tier, each with concrete steps

---

## Tier 1 — Fix This Week (Breaks Developer Trust)

### FP-01: Canonical Documentation Generation Pipeline

**Problem:** Docs drift from code because they're manually maintained.

**Proposal:** Implement a semi-automated doc generation workflow:

1. **API routes:** Add a `cargo test` integration test that extracts all registered Axum routes and writes them to `site-docs/reference/api.md`. Use utoipa's OpenAPI spec generation (already in dependencies) to produce a machine-readable route list.

```rust
// tests/doc_routes.rs
#[test]
fn generate_api_docs() {
    let app = forge_api::router(/* test state */);
    // Walk the router tree, extract method + path
    // Compare against site-docs/reference/api.md
    // Fail test if mismatch (CI enforcement)
}
```

2. **MCP tools:** Add a similar test in `forge-mcp-bin` that lists all `#[tool]` annotated functions and asserts they match `site-docs/reference/mcp-tools.md`.

3. **Event variants:** A `build.rs` or test that counts `ForgeEvent` variants via `strum::EnumCount` and asserts the number in docs.

**Effort:** 4 hours
**Impact:** Prevents ALL future drift automatically

---

### FP-02: Fix Database Default Path

**Problem:** Code says `~/.claude-forge/forge.db`, docs say `~/.agentforge/forge.db`.

**Proposal:**
1. Change `forge-app/src/main.rs` default to `~/.agentforge/forge.db` (brand-aligned)
2. Add a migration helper: if `~/.claude-forge/forge.db` exists and `~/.agentforge/forge.db` doesn't, print a one-time message suggesting the user move their data
3. Update all docs to `~/.agentforge/forge.db`

```rust
fn default_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let new_path = home.join(".agentforge/forge.db");
    let legacy_path = home.join(".claude-forge/forge.db");

    if !new_path.exists() && legacy_path.exists() {
        eprintln!("⚠ Found database at legacy path ~/.claude-forge/forge.db");
        eprintln!("  Consider moving to ~/.agentforge/forge.db");
        return legacy_path; // Graceful fallback
    }
    new_path
}
```

**Effort:** 1 hour
**Impact:** Users find their data where docs say it is

---

### FP-03: Remove `site/` from Git, Add to .gitignore

**Problem:** 60+ generated HTML/JS/CSS files tracked in version control. Bloats repo, causes merge conflicts, gives false impression of manual edits.

**Proposal:**
```bash
echo "site/" >> .gitignore
git rm -r --cached site/
git commit -m "chore: remove generated site/ from tracking"
```

Add to CI: `mkdocs build` step that generates `site/` and deploys to GitHub Pages (or Cloudflare Pages).

**Effort:** 30 min
**Impact:** Cleaner repo, proper CI/CD

---

### FP-04: Middleware Chain — Single Source of Truth

**Problem:** Three docs say three different numbers (7, 8, 9).

**Proposal:**
1. In `forge-api/src/middleware.rs`, add a module-level doc comment that IS the canonical list:

```rust
//! # Middleware Pipeline (Canonical Reference)
//!
//! The run pipeline processes requests through these stages:
//!
//! 1. **RateLimit** — Token bucket (max tokens, refill interval)
//! 2. **CircuitBreaker** — 3-state FSM (Closed → Open → HalfOpen)
//! 3. **CostCheck** — Budget validation (warn threshold, hard limit)
//! 4. **SkillInjection** — Attach relevant skills to run context
//! 5. **Persist** — Log events to EventRepo
//! 6. **Spawn** — Execute Claude CLI process
//! 7. **ExitGate** — Quality/safety gate on output
//!
//! Total: 7 active middleware stages.
```

2. Update `site-docs/architecture/middleware.md` to match exactly.
3. Update `wave-history.md` metrics.
4. Add a test that counts middleware stages and asserts doc accuracy.

**Effort:** 1.5 hours
**Impact:** Ends the 7/8/9 confusion permanently

---

## Tier 2 — Fix This Sprint (Improves Onboarding)

### FP-05: Complete API Reference

Add the 10+ missing routes to `site-docs/reference/api.md`:

```markdown
### Companies (full CRUD)
| Method | Path | Description |
|--------|------|-------------|
| GET | /api/v1/companies | List all companies |
| POST | /api/v1/companies | Create company |
| GET | /api/v1/companies/:id | Get company by ID |
| PATCH | /api/v1/companies/:id | Update company |
| DELETE | /api/v1/companies/:id | Delete company |

### Health
| Method | Path | Description |
|--------|------|-------------|
| GET | /api/v1/health | Health check + CLI availability |
```

**Effort:** 2 hours
**Impact:** Complete API surface documented

---

### FP-06: Update Event Count and List

1. Read `ForgeEvent` enum from source, extract all 35 variants
2. Replace the list in `site-docs/architecture/events.md` with the actual variants
3. Update the count from 38 to 35
4. Group by the 7 actual categories (System, Agent, Process, Session, Workflow, Safety, Hook, SubAgent, Schedule, ExitGate, Pipeline, Optimization, ToolUse, Security, Error)

**Effort:** 45 min

---

### FP-07: Standardize Build Instructions

Replace all `npm` references with `pnpm` across:
- `site-docs/development/building.md`
- `site-docs/getting-started/quickstart.md`
- `CLAUDE.md` (already correct)
- `README.md`

**Effort:** 20 min

---

### FP-08: Update Gaps Document

Add resolution status to each gap in `site-docs/strategy/gaps.md`:

```markdown
| # | Gap | Status | Notes |
|---|-----|--------|-------|
| 1 | Budget decorative | OPEN | Budget tracked but not enforced in run pipeline |
| 2 | Approvals don't block | OPEN | No pre-run approval gate |
| 3 | Goals have no influence | OPEN | Goals stored but not referenced |
| 4 | 6 empty sidebar pages | PARTIAL | Skills, Memory functional; Hooks, Schedules stubs |
| 5 | Skills invisible | RESOLVED | Skills page has category filter + content preview |
```

**Effort:** 30 min

---

## Tier 3 — Fix Next Sprint (Nice to Have)

### FP-09: Persona Pipeline Documentation

Document the actual persona loading flow:
1. TOML files in `personas/` directory (source of truth)
2. Parsed by `forge-persona` catalog loader
3. Seeded into `personas` and `persona_divisions` tables via migration 0009
4. Served via `GET /api/v1/personas` from database
5. Hired via `POST /api/v1/personas/:id` → creates Agent + OrgPosition

Verify and document exact count.

**Effort:** 1 hour

---

### FP-10: Architecture Diagram Refresh

Redraw the Mermaid diagram in `site-docs/architecture/overview.md` to include:
- Governance layer (Companies, Goals, Approvals)
- HookReceiver endpoints
- AgentConfigurator
- Persona hire flow
- The actual middleware pipeline

Add `<!-- Last updated: 2026-03-XX -->` comment to all architecture docs.

**Effort:** 2 hours

---

### FP-11: MkDocs CI/CD Pipeline

Add a GitHub Actions workflow:

```yaml
name: Deploy Docs
on:
  push:
    branches: [main]
    paths: ['site-docs/**', 'mkdocs.yml']
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with: { python-version: '3.12' }
      - run: pip install mkdocs-material
      - run: mkdocs build --strict
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./site
```

**Effort:** 1 hour

---

## Execution Order

```
Week 1: FP-03 → FP-02 → FP-04 → FP-07   (quick wins, high trust)
Week 2: FP-05 → FP-06 → FP-08            (completeness)
Week 3: FP-01 → FP-09 → FP-10 → FP-11   (automation + polish)
```

Total estimated: ~15 hours across 3 weeks (one developer, part-time)
