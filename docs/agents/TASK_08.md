# TASK 08 — Frontend: /skills and /workflows call real API

**Status:** done
**Priority:** medium
**Track:** frontend

---

## Context

The frontend has `/skills` and `/workflows` pages that show "Coming soon" placeholders. The backend now has `GET /api/v1/skills` (TASK_02) and will have `GET /api/v1/workflows` (TASK_07). Wire the frontend pages to call the real API.

**Depends on:** TASK_07 (for workflows API). Can do skills part immediately.

## Task

### Skills page (`frontend/src/routes/skills/+page.svelte`)

1. Import `listSkills` (you'll add this to `api.ts` — see below)
2. On mount, call the API. Show results in a list or table.
3. If empty, show "No skills yet."
4. If API errors (e.g. backend not running), show error message gracefully.

### Workflows page (`frontend/src/routes/workflows/+page.svelte`)

Same pattern as skills but for workflows.

### API client (`frontend/src/lib/api.ts`)

Add types and functions:

```typescript
export interface Skill {
  id: string;
  name: string;
  description: string | null;
  category: string | null;
  subcategory: string | null;
  content: string;
  source_repo: string | null;
  usage_count: number;
  created_at: string;
}

export async function listSkills(): Promise<Skill[]> {
  const res = await fetch(`${API_BASE}/api/v1/skills`);
  return handleResponse<Skill[]>(res);
}

// Same pattern for Workflow — check the backend Workflow struct for fields
```

## Files to read first

- `frontend/src/routes/skills/+page.svelte` — current placeholder
- `frontend/src/routes/workflows/+page.svelte` — current placeholder
- `frontend/src/lib/api.ts` — existing patterns
- `frontend/src/routes/sessions/+page.svelte` — good pattern to follow for list + detail layout

## Files to edit

- `frontend/src/lib/api.ts`
- `frontend/src/routes/skills/+page.svelte`
- `frontend/src/routes/workflows/+page.svelte`

## Verify

```bash
cd frontend && pnpm build
```

---

## Report

*Agent: fill this in when done.*

- [x] What was changed:
  - **api.ts**: Added `Skill` and `Workflow` interfaces (matching forge-db repos: Skill includes parameters_json, examples_json; Workflow has definition_json, created_at, updated_at). Added `listSkills()` and `listWorkflows()` calling GET `/api/v1/skills` and GET `/api/v1/workflows` with existing `handleResponse`.
  - **skills/+page.svelte**: Replaced placeholder with onMount → `listSkills()`; loading/error/empty states; list of skills (name, category/subcategory, description) with scoped styles.
  - **workflows/+page.svelte**: Same pattern for workflows (name, description); list layout and error handling.
- [x] Tests pass (pnpm build): yes
- [ ] Notes: Backend Skill has `parameters_json` and `examples_json` (included in frontend type). If GET /api/v1/workflows is not yet implemented (TASK_07), workflows page will show error until that endpoint exists.
