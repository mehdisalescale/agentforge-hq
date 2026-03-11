# ADR-005: Company as Tenant Boundary

## Status
Accepted

## Context
Users need to manage multiple independent AI workforces (per client, per project). We need data isolation without adding PostgreSQL or complex multi-tenancy middleware. The existing codebase has no tenant concept — all agents and sessions are global.

## Decision
Introduce `Company` as the tenant boundary. All major entities (agents, sessions, goals, KB documents, budgets) are scoped to a company_id. Backward compatibility is preserved via a "Default" company auto-created on first run.

**Key rules:**
1. Every agent belongs to exactly one company (nullable FK for migration path)
2. API queries filter by company_id (middleware extracts from auth context or header)
3. Budget enforcement is per-company
4. Knowledge base is per-company
5. A "Default" company is auto-created if none exists (backward compat)
6. Existing agents (pre-v0.8.0) are assigned to Default company on migration

**Migration strategy:**
```sql
-- Safe: nullable FK, no data loss
ALTER TABLE agents ADD COLUMN company_id TEXT REFERENCES companies(id);

-- On startup: assign orphan agents to default company
UPDATE agents SET company_id = (SELECT id FROM companies WHERE name = 'Default')
WHERE company_id IS NULL;
```

## Consequences

### Positive
- Clean data isolation without PostgreSQL
- Backward compatible (existing installs keep working)
- Budget enforcement naturally scoped
- Future: multi-user with company-based RBAC

### Negative
- Every query gains a `WHERE company_id = ?` clause
- Single SQLite file contains all companies (not true isolation)
- Cross-company reporting requires explicit join

### Risks
- SQLite file corruption affects all companies. Mitigated by WAL mode + regular backups.
- Company deletion cascades widely. Mitigated by soft-delete pattern.
