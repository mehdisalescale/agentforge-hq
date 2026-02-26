# TASK 17 — Tool use / tool result collapsible panels

**Status:** pending
**Priority:** medium
**Track:** Phase B — UX

**Depends on:** TASK_16 (Markdown rendering)

---

## Context

When Claude uses tools, the WebSocket sends `ProcessOutput` events with `kind: "ToolUse"`, `kind: "ToolResult"`, and `kind: "Thinking"`. Currently all output is concatenated into one string. It should be structured into blocks with collapsible panels.

## Task

1. In `+page.svelte`, replace the single `streamContent` string with an array:
   ```typescript
   interface OutputBlock {
     kind: 'assistant' | 'tool_use' | 'tool_result' | 'thinking' | 'result';
     content: string;
   }
   let outputBlocks: OutputBlock[] = [];
   ```

2. In the WebSocket handler, instead of `streamContent += ev.data.content`, push to the array:
   ```typescript
   const kind = (ev.data?.kind ?? 'assistant').toLowerCase();
   // Append to last block if same kind, otherwise create new block
   const last = outputBlocks[outputBlocks.length - 1];
   if (last && last.kind === kind) {
     last.content += ev.data.content;
     outputBlocks = outputBlocks; // trigger reactivity
   } else {
     outputBlocks = [...outputBlocks, { kind, content: ev.data.content }];
   }
   ```

3. Render each block type differently:
   ```svelte
   {#each outputBlocks as block}
     {#if block.kind === 'assistant' || block.kind === 'result'}
       <div class="block-assistant">{@html renderMarkdown(block.content)}</div>
     {:else if block.kind === 'tool_use'}
       <details class="block-tool">
         <summary>Tool Call</summary>
         <pre><code>{block.content}</code></pre>
       </details>
     {:else if block.kind === 'tool_result'}
       <details class="block-tool result">
         <summary>Tool Result</summary>
         <pre><code>{block.content}</code></pre>
       </details>
     {:else if block.kind === 'thinking'}
       <details class="block-thinking">
         <summary>Thinking...</summary>
         <pre class="dimmed">{block.content}</pre>
       </details>
     {/if}
   {/each}
   ```

4. Add CSS for the block types:
   - `.block-tool`: border-left accent color, collapsible
   - `.block-thinking`: dimmed/muted text, collapsed by default
   - `.block-tool.result`: slightly different border color

## Files to edit

- `frontend/src/routes/+page.svelte`

## Verify

```bash
cd frontend && pnpm build
```
Manual: run an agent that uses tools. Tool calls appear as collapsible panels.

---

## Report

*Agent: fill this in when done.*

- [ ] What was changed:
- [ ] Build passes: yes/no
- [ ] Notes:
