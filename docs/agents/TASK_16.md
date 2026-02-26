# TASK 16 — Markdown rendering in output stream

**Status:** pending
**Priority:** high
**Track:** Phase B — UX

---

## Context

The Dashboard output area renders everything as raw text inside `<pre><code>`. When Claude produces Markdown (code blocks, headings, lists), it should render formatted.

## Task

1. Install `marked` in the frontend:
   ```bash
   cd frontend && pnpm add marked
   ```
2. In `frontend/src/routes/+page.svelte`:
   - Import `marked`:
     ```typescript
     import { marked } from 'marked';
     ```
   - Replace the raw output block:
     ```svelte
     <!-- Before -->
     <pre class="stream-pre"><code>{streamContent}</code></pre>

     <!-- After -->
     <div class="stream-rendered">{@html marked.parse(streamContent)}</div>
     ```
3. Add CSS for `.stream-rendered`:
   - Code blocks: dark background, monospace, scrollable
   - Headings: proper sizing
   - Lists: proper indentation
   - Keep the overall styling consistent with the existing dark theme
4. **XSS safety**: `marked` doesn't sanitize by default. Add DOMPurify or use marked's `sanitize` option:
   ```bash
   pnpm add dompurify
   pnpm add -D @types/dompurify
   ```
   ```typescript
   import DOMPurify from 'dompurify';
   const rendered = DOMPurify.sanitize(marked.parse(streamContent));
   ```

## Files to edit

- `frontend/package.json` (add marked, dompurify)
- `frontend/src/routes/+page.svelte`

## Verify

```bash
cd frontend && pnpm build
```
Manual: run an agent that produces code blocks and lists. Output should render formatted.

---

## Report

*Agent: fill this in when done.*

- [ ] What was changed:
- [ ] Build passes: yes/no
- [ ] Notes:
