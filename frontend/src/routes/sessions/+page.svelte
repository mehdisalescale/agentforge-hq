<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import ErrorMessage from '$lib/components/ErrorMessage.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { History } from 'lucide-svelte';
  import {
    listSessions,
    getSession,
    getSessionEvents,
    exportSessionUrl,
    exportSessionHtmlUrl,
    listAgents,
    type Session,
    type SessionEvent,
    type Agent,
  } from '$lib/api';

  interface OutputBlock {
    kind: 'assistant' | 'tool_use' | 'tool_result' | 'thinking' | 'result';
    content: string;
  }

  function normalizeBlockKind(k: string): OutputBlock['kind'] {
    const s = (k ?? 'assistant').toString().toLowerCase();
    if (s === 'tooluse') return 'tool_use';
    if (s === 'toolresult') return 'tool_result';
    if (s === 'thinking' || s === 'result') return s;
    return 'assistant';
  }

  function renderMarkdown(raw: string): string {
    if (!raw?.trim()) return '';
    const html = marked.parse(raw, { async: false }) as string;
    return DOMPurify.sanitize(html);
  }

  setContext('pageTitle', 'Sessions');

  let sessions: Session[] = $state([]);
  let sessionsError: string = $state('');
  let selectedId: string | null = $state(null);
  let detail: Session | null = $state(null);
  let detailError: string = $state('');
  let agents: Agent[] = $state([]);
  let viewMode: 'list' | 'kanban' = $state('list');
  let sessionOutput: OutputBlock[] = $state([]);
  let outputLoading: boolean = $state(false);

  // Kanban columns derived from sessions
  const KANBAN_STATUSES = ['created', 'running', 'completed', 'failed'] as const;
  const KANBAN_LABELS: Record<string, string> = { created: 'Pending', running: 'Running', completed: 'Completed', failed: 'Failed' };
  const KANBAN_COLORS: Record<string, string> = { created: '#71717a', running: '#3b82f6', completed: '#22c55e', failed: '#ef4444' };
  let kanbanColumns = $derived(
    KANBAN_STATUSES.map(status => ({
      status,
      label: KANBAN_LABELS[status],
      color: KANBAN_COLORS[status],
      sessions: sessions.filter(s => s.status === status),
    }))
  );

  function agentName(agentId: string): string {
    const a = agents.find(ag => ag.id === agentId);
    return a?.name ?? agentId.slice(0, 8) + '...';
  }

  function isWorktree(dir: string | null | undefined): boolean {
    return !!dir && dir.includes('.claude/worktrees/');
  }

  async function loadSessions() {
    sessionsError = '';
    try {
      sessions = await listSessions();
    } catch (e) {
      sessionsError = e instanceof Error ? e.message : String(e);
      sessions = [];
    }
  }

  async function loadDetail(id: string) {
    selectedId = id;
    detailError = '';
    detail = null;
    sessionOutput = [];
    try {
      detail = await getSession(id);
      loadSessionOutput(id);
    } catch (e) {
      detailError = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadSessionOutput(sessionId: string) {
    outputLoading = true;
    try {
      const events = await getSessionEvents(sessionId);
      const blocks: OutputBlock[] = [];
      for (const ev of events) {
        if (ev.event_type !== 'ProcessOutput') continue;
        let data: Record<string, unknown> = {};
        try { data = JSON.parse(ev.data_json); } catch { continue; }
        const kind = normalizeBlockKind((data.kind as string) ?? 'assistant');
        const content = typeof data.content === 'string' ? data.content : String(data.content ?? '');
        if (!content) continue;
        if (blocks.length > 0 && blocks[blocks.length - 1].kind === kind) {
          blocks[blocks.length - 1].content += content;
        } else {
          blocks.push({ kind, content });
        }
      }
      sessionOutput = blocks;
    } catch {
      sessionOutput = [];
    } finally {
      outputLoading = false;
    }
  }

  function resume(session: Session) {
    goto(`/?resume=${encodeURIComponent(session.id)}`);
  }

  function exportAs(sessionId: string, format: 'json' | 'markdown') {
    const url = exportSessionUrl(sessionId, format);
    window.open(url, '_blank', 'noopener,noreferrer');
  }

  function exportHtml(sessionId: string) {
    const url = exportSessionHtmlUrl(sessionId);
    window.open(url, '_blank', 'noopener,noreferrer');
  }

  onMount(() => {
    loadSessions();
    listAgents().then((a) => (agents = a)).catch(() => {});
  });
</script>

<svelte:head>
  <title>Sessions · AgentForge</title>
</svelte:head>

<div class="page sessions-page">
  <div class="page-header">
    <h1>Sessions</h1>
    <div class="view-toggle">
      <button type="button" class="toggle-btn" class:active={viewMode === 'list'} onclick={() => viewMode = 'list'}>List</button>
      <button type="button" class="toggle-btn" class:active={viewMode === 'kanban'} onclick={() => viewMode = 'kanban'}>Kanban</button>
    </div>
  </div>
  {#if sessionsError}
    <ErrorMessage message={sessionsError} onretry={loadSessions} />
  {:else if sessions.length === 0 && viewMode === 'list'}
    <EmptyState
      icon={History}
      title="No sessions yet"
      description="Run an agent from the Dashboard to create sessions."
    />
  {:else if viewMode === 'kanban'}
    <div class="kanban-board">
      {#each kanbanColumns as col (col.status)}
        <div class="kanban-col" style="--col-color: {col.color}">
          <div class="kanban-col-header">
            <span class="kanban-col-title">{col.label}</span>
            <span class="kanban-col-count">{col.sessions.length}</span>
          </div>
          <div class="kanban-col-body">
            {#if col.sessions.length === 0}
              <span class="muted kanban-empty">No sessions</span>
            {:else}
              {#each col.sessions as s (s.id)}
                <button
                  type="button"
                  class="kanban-card"
                  class:selected={selectedId === s.id}
                  onclick={() => loadDetail(s.id)}
                >
                  <span class="kanban-card-agent">{agentName(s.agent_id)}</span>
                  <span class="kanban-card-id">{s.id.slice(0, 8)}...</span>
                  <span class="kanban-card-meta">
                    {#if s.cost_usd != null && s.cost_usd !== undefined}
                      <span class="kanban-card-cost">${s.cost_usd.toFixed(4)}</span>
                    {/if}
                    {#if isWorktree(s.directory)}
                      <span class="worktree-badge">WT</span>
                    {/if}
                  </span>
                </button>
              {/each}
            {/if}
          </div>
        </div>
      {/each}
    </div>
    {#if selectedId && detail}
      <section class="kanban-detail">
        <h2>Session {detail.id.slice(0, 8)}...</h2>
        <dl class="detail-dl">
          <dt>ID</dt><dd><code>{detail.id}</code></dd>
          <dt>Agent</dt><dd>{agentName(detail.agent_id)}</dd>
          <dt>Directory</dt><dd><code>{detail.directory || '\u2014'}</code></dd>
          <dt>Status</dt><dd><span class="status-badge" class:running={detail.status === 'running'} class:completed={detail.status === 'completed'} class:failed={detail.status === 'failed'}>{detail.status}</span></dd>
          {#if detail.cost_usd != null}<dt>Cost</dt><dd>${detail.cost_usd.toFixed(4)}</dd>{/if}
          <dt>Created</dt><dd>{detail.created_at}</dd>
        </dl>
        <div class="detail-actions">
          <button type="button" class="primary" onclick={() => detail && resume(detail)}>Resume</button>
          <button type="button" class="secondary" onclick={() => detail && exportAs(detail.id, 'json')}>Export JSON</button>
          <button type="button" class="secondary" onclick={() => detail && exportHtml(detail.id)}>Export HTML</button>
        </div>
        <div class="session-output">
          <h3>Output</h3>
          {#if outputLoading}
            <p class="muted">Loading output...</p>
          {:else if sessionOutput.length === 0}
            <p class="muted">No output recorded for this session.</p>
          {:else}
            {#each sessionOutput as block}
              {#if block.kind === 'assistant' || block.kind === 'result'}
                <div class="block-assistant rendered">{@html renderMarkdown(block.content)}</div>
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
          {/if}
        </div>
      </section>
    {/if}
  {:else}
    <div class="sessions-layout">
      <section class="session-list">
        <h2>Recent</h2>
        {#if sessions.length === 0}
          <p class="muted">No sessions yet. Run an agent from the Dashboard.</p>
        {:else}
          <ul class="list">
            {#each sessions as s}
              <li>
                <button
                  type="button"
                  class="session-item"
                  class:selected={selectedId === s.id}
                  onclick={() => loadDetail(s.id)}
                >
                  <span class="session-header">
                    <span class="session-id">{s.id.slice(0, 8)}…</span>
                    <span class="session-badges">
                      {#if isWorktree(s.directory)}
                        <span class="worktree-badge">Worktree</span>
                      {/if}
                      <span class="status-badge" class:running={s.status === 'running'} class:completed={s.status === 'completed'} class:failed={s.status === 'failed'}>{s.status}</span>
                    </span>
                  </span>
                  <span class="session-meta">{s.directory || '—'}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
      <section class="session-detail">
        {#if selectedId && !detail && !detailError}
          <p class="muted">Loading…</p>
        {:else if detailError}
          <p class="error">{detailError}</p>
        {:else if detail}
          <h2>Session {detail.id.slice(0, 8)}…</h2>
          <dl class="detail-dl">
            <dt>ID</dt>
            <dd><code>{detail.id}</code></dd>
            <dt>Agent ID</dt>
            <dd><code>{detail.agent_id}</code></dd>
            <dt>Directory</dt>
            <dd>
              <span class="directory-value">
                {#if isWorktree(detail.directory)}
                  <span class="worktree-badge">Worktree</span>
                {/if}
                <code class="directory-path">{detail.directory || '—'}</code>
              </span>
            </dd>
            <dt>Status</dt>
            <dd><span class="status-badge" class:running={detail.status === 'running'} class:completed={detail.status === 'completed'} class:failed={detail.status === 'failed'}>{detail.status}</span></dd>
            {#if detail.cost_usd != null && detail.cost_usd !== undefined}
              <dt>Cost</dt>
              <dd>${detail.cost_usd.toFixed(4)}</dd>
            {/if}
            <dt>Created</dt>
            <dd>{detail.created_at}</dd>
          </dl>
          <div class="detail-actions">
            <button type="button" class="primary" onclick={() => detail && resume(detail)}>Resume</button>
            <button type="button" class="secondary" onclick={() => detail && exportAs(detail.id, 'json')}>
              Export JSON
            </button>
            <button type="button" class="secondary" onclick={() => detail && exportAs(detail.id, 'markdown')}>
              Export Markdown
            </button>
            <button type="button" class="secondary" onclick={() => detail && exportHtml(detail.id)}>
              Export HTML
            </button>
            {#if isWorktree(detail.directory)}
              <button type="button" class="secondary" disabled title="Coming soon — requires worktree API">
                Merge
              </button>
              <button type="button" class="secondary" disabled title="Coming soon — requires worktree API">
                Cleanup
              </button>
            {/if}
          </div>
          <div class="session-output">
            <h3>Output</h3>
            {#if outputLoading}
              <p class="muted">Loading output...</p>
            {:else if sessionOutput.length === 0}
              <p class="muted">No output recorded for this session.</p>
            {:else}
              {#each sessionOutput as block}
                {#if block.kind === 'assistant' || block.kind === 'result'}
                  <div class="block-assistant rendered">{@html renderMarkdown(block.content)}</div>
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
            {/if}
          </div>
        {:else}
          <p class="muted">Select a session to see details and Resume or Export.</p>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .page h1 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
  }
  .sessions-page .error {
    color: #f87171;
    margin: 0 0 0.5rem 0;
  }
  .sessions-layout {
    display: grid;
    grid-template-columns: 18rem 1fr;
    gap: 1.5rem;
  }
  .session-list h2,
  .session-detail h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    font-weight: 600;
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .session-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 0.6rem 0.75rem;
    margin-bottom: 0.25rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font-family: inherit;
  }
  .session-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .session-item.selected {
    border-color: var(--accent);
    background: rgba(167, 139, 250, 0.1);
  }
  .session-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .session-badges {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }
  .session-id {
    font-size: 0.9rem;
    font-weight: 500;
  }
  .status-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    background: var(--surface);
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .status-badge.running {
    background: rgba(96, 165, 250, 0.15);
    color: #60a5fa;
  }
  .status-badge.completed {
    background: rgba(134, 239, 172, 0.15);
    color: #86efac;
  }
  .status-badge.failed {
    background: rgba(248, 113, 113, 0.15);
    color: #f87171;
  }
  .worktree-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    background: rgba(96, 165, 250, 0.15);
    color: #60a5fa;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    font-weight: 500;
  }
  .session-meta {
    display: block;
    font-size: 0.75rem;
    color: var(--muted);
    margin-top: 0.2rem;
  }
  .detail-dl {
    margin: 0 0 1rem 0;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.25rem 0.75rem;
  }
  .detail-dl dt {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .detail-dl dd {
    margin: 0;
    font-size: 0.9rem;
  }
  .detail-dl code {
    font-size: 0.8rem;
    background: var(--bg);
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
  }
  .directory-value {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .directory-path {
    font-size: 0.9rem;
    word-break: break-all;
  }
  .detail-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .detail-actions button {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    font-family: inherit;
  }
  .detail-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .detail-actions button.primary {
    background: var(--accent);
    color: var(--bg);
  }
  .detail-actions button.secondary {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }
  .muted {
    color: var(--muted);
    font-size: 0.9rem;
    margin: 0;
  }
  /* --- Page header with view toggle --- */
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }
  .page-header h1 {
    margin: 0;
    font-size: 1.5rem;
  }
  .view-toggle {
    display: flex;
    gap: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }
  .toggle-btn {
    padding: 0.35rem 0.75rem;
    font-size: 0.8rem;
    font-weight: 500;
    background: var(--surface);
    color: var(--muted);
    border: none;
    cursor: pointer;
    font-family: inherit;
  }
  .toggle-btn.active {
    background: var(--accent);
    color: var(--bg);
  }
  /* --- Kanban board --- */
  .kanban-board {
    display: flex;
    gap: 0.75rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
  }
  .kanban-col {
    flex: 1;
    min-width: 14rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-top: 3px solid var(--col-color, var(--border));
    border-radius: 8px;
    display: flex;
    flex-direction: column;
  }
  .kanban-col-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }
  .kanban-col-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text);
  }
  .kanban-col-count {
    font-size: 0.75rem;
    padding: 0.1rem 0.4rem;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--muted);
  }
  .kanban-col-body {
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-height: 28rem;
    overflow-y: auto;
  }
  .kanban-empty {
    text-align: center;
    padding: 1rem 0;
    font-size: 0.8rem;
  }
  .kanban-card {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.5rem 0.65rem;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    color: var(--text);
  }
  .kanban-card:hover {
    border-color: var(--accent);
    background: rgba(167, 139, 250, 0.05);
  }
  .kanban-card.selected {
    border-color: var(--accent);
    background: rgba(167, 139, 250, 0.1);
  }
  .kanban-card-agent {
    font-size: 0.85rem;
    font-weight: 500;
  }
  .kanban-card-id {
    font-size: 0.75rem;
    font-family: ui-monospace, monospace;
    color: var(--muted);
  }
  .kanban-card-meta {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-top: 0.1rem;
  }
  .kanban-card-cost {
    font-size: 0.75rem;
    color: var(--muted);
  }
  .kanban-detail {
    margin-top: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1rem;
  }
  .kanban-detail h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    font-weight: 600;
  }
  /* --- Session output rendering --- */
  .session-output {
    margin-top: 1.25rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }
  .session-output h3 {
    margin: 0 0 0.75rem 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text);
  }
  .session-output .rendered {
    font-size: 0.9rem;
    line-height: 1.5;
  }
  .session-output .rendered :global(h1) { font-size: 1.15rem; margin: 0 0 0.5rem 0; }
  .session-output .rendered :global(h2) { font-size: 1rem; margin: 0.75rem 0 0.4rem 0; }
  .session-output .rendered :global(h3) { font-size: 0.95rem; margin: 0.5rem 0 0.3rem 0; }
  .session-output .rendered :global(ul), .session-output .rendered :global(ol) {
    margin: 0.25rem 0;
    padding-left: 1.5rem;
  }
  .session-output .rendered :global(pre) {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.75rem;
    overflow-x: auto;
    margin: 0.5rem 0;
    font-size: 0.82rem;
  }
  .session-output .rendered :global(code) { font-size: 0.85em; }
  .session-output .rendered :global(p) { margin: 0.5rem 0; }
  .block-tool {
    border-left: 3px solid var(--accent);
    margin: 0.5rem 0;
    padding-left: 0.75rem;
  }
  .block-tool.result {
    border-left-color: #86efac;
  }
  .block-tool summary,
  .block-thinking summary {
    cursor: pointer;
    font-size: 0.85rem;
    color: var(--muted);
  }
  .block-tool pre,
  .block-thinking pre {
    margin: 0.25rem 0 0 0;
    font-size: 0.85rem;
    overflow-x: auto;
  }
  .block-thinking {
    border-left: 3px solid var(--border);
    margin: 0.5rem 0;
    padding-left: 0.75rem;
  }
  .block-thinking .dimmed {
    color: var(--muted);
  }
</style>
