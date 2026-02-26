<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    listSessions,
    getSession,
    exportSessionUrl,
    listAgents,
    type Session,
    type Agent,
  } from '$lib/api';

  setContext('pageTitle', 'Sessions');

  let sessions: Session[] = [];
  let sessionsError = '';
  let selectedId: string | null = null;
  let detail: Session | null = null;
  let detailError = '';
  let agents: Agent[] = [];

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
    try {
      detail = await getSession(id);
    } catch (e) {
      detailError = e instanceof Error ? e.message : String(e);
    }
  }

  function resume(session: Session) {
    goto(`/?resume=${encodeURIComponent(session.id)}`);
  }

  function exportAs(sessionId: string, format: 'json' | 'markdown') {
    const url = exportSessionUrl(sessionId, format);
    window.open(url, '_blank', 'noopener,noreferrer');
  }

  onMount(() => {
    loadSessions();
    listAgents().then((a) => (agents = a)).catch(() => {});
  });
</script>

<svelte:head>
  <title>Sessions · Claude Forge</title>
</svelte:head>

<div class="page sessions-page">
  <h1>Sessions</h1>
  {#if sessionsError}
    <p class="error">{sessionsError}</p>
    <p class="muted">Session API may not be available yet (Agent C). You can still use Run on the Dashboard.</p>
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
                  on:click={() => loadDetail(s.id)}
                >
                  <span class="session-header">
                    <span class="session-id">{s.id.slice(0, 8)}…</span>
                    <span class="status-badge" class:running={s.status === 'running'} class:completed={s.status === 'completed'} class:failed={s.status === 'failed'}>{s.status}</span>
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
            <dd><code>{detail.directory || '—'}</code></dd>
            <dt>Status</dt>
            <dd><span class="status-badge" class:running={detail.status === 'running'} class:completed={detail.status === 'completed'} class:failed={detail.status === 'failed'}>{detail.status}</span></dd>
            <dt>Created</dt>
            <dd>{detail.created_at}</dd>
          </dl>
          <div class="detail-actions">
            <button type="button" class="primary" on:click={() => detail && resume(detail)}>Resume</button>
            <button type="button" class="secondary" on:click={() => detail && exportAs(detail.id, 'json')}>
              Export JSON
            </button>
            <button type="button" class="secondary" on:click={() => detail && exportAs(detail.id, 'markdown')}>
              Export Markdown
            </button>
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
</style>
