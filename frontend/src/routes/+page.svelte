<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { browser } from '$app/environment';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import {
    listAgents,
    runAgent,
    wsUrl,
    isProcessOutputEvent,
    isProcessLifecycleEvent,
    type Agent,
    type ForgeEventWire,
  } from '$lib/api';

  function renderStreamMarkdown(raw: string): string {
    if (!raw?.trim()) return '';
    const html = marked.parse(raw, { async: false }) as string;
    return DOMPurify.sanitize(html);
  }

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

  interface SubAgentStatus {
    agentId: string;
    status: 'requested' | 'running' | 'completed' | 'failed';
    sessionId?: string;
    prompt?: string;
    error?: string;
    timestamp: string;
  }

  /** Per-agent output block with icon metadata for swim lanes */
  interface SwimLaneEvent {
    kind: OutputBlock['kind'];
    content: string;
    timestamp: string;
  }

  /** Swim-lane column tracking for each sub-agent */
  interface SwimLaneColumn {
    agentId: string;
    status: 'running' | 'completed' | 'failed' | 'pending';
    sessionId?: string;
    events: SwimLaneEvent[];
  }

  const KIND_ICONS: Record<OutputBlock['kind'], string> = {
    assistant: '\u{1F4AC}',
    tool_use: '\u{1F527}',
    tool_result: '\u{1F4CB}',
    thinking: '\u{1F9E0}',
    result: '\u2705',
  };

  const STATUS_COLORS: Record<string, string> = {
    running: '#3b82f6',
    completed: '#22c55e',
    failed: '#ef4444',
    pending: '#6b7280',
  };

  setContext('pageTitle', 'Dashboard');

  // --- Svelte 5 rune state ---
  let agents = $state<Agent[]>([]);
  let agentsError = $state('');
  let selectedAgentId = $state('');
  let prompt = $state('');
  let directory = $state('');
  let running = $state(false);
  let runError = $state('');
  let outputBlocks = $state<OutputBlock[]>([]);
  let streamStatus = $state<'idle' | 'connecting' | 'streaming' | 'completed' | 'failed'>('idle');
  let streamStatusDetail = $state('');
  let currentSessionId = $state<string | null>(null);
  let ws = $state<WebSocket | null>(null);
  let wsReconnectTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let wsReconnectDelay = $state(1000);
  let subAgents = $state<SubAgentStatus[]>([]);
  let swimLaneColumns = $state<Record<string, SwimLaneColumn>>({});
  let swimLaneMode = $derived(Object.keys(swimLaneColumns).length > 0);
  let swimLanePinned = $state(true);
  const WS_MAX_RECONNECT_DELAY = 30000;

  /** Session ID from ?resume= (Sessions page "Resume"). Only in browser (prerender-safe). */
  let resumeSessionId = $derived(browser ? ($page.url.searchParams.get('resume') || null) : null);

  async function loadAgents() {
    agentsError = '';
    try {
      agents = await listAgents();
      if (agents.length > 0 && !selectedAgentId) selectedAgentId = agents[0].id;
    } catch (e) {
      agentsError = e instanceof Error ? e.message : String(e);
      agents = [];
    }
  }

  function connectWs() {
    if (ws?.readyState === WebSocket.OPEN) return;
    if (wsReconnectTimer) { clearTimeout(wsReconnectTimer); wsReconnectTimer = null; }
    const url = wsUrl();
    try {
      ws = new WebSocket(url);
      ws.onopen = () => {
        wsReconnectDelay = 1000; // reset backoff on successful connect
      };
      ws.onmessage = (event) => {
        try {
          const ev: ForgeEventWire = JSON.parse(event.data);
          if (currentSessionId && isProcessOutputEvent(ev, currentSessionId) && ev.data?.content !== undefined) {
            const kind = normalizeBlockKind(ev.data.kind ?? 'assistant');
            const content = typeof ev.data.content === 'string' ? ev.data.content : String(ev.data.content);
            if (outputBlocks.length > 0 && outputBlocks[outputBlocks.length - 1].kind === kind) {
              outputBlocks[outputBlocks.length - 1].content += content;
              outputBlocks = outputBlocks;
            } else {
              outputBlocks = [...outputBlocks, { kind, content }];
            }
            streamStatus = 'streaming';
          }
          if (currentSessionId && isProcessLifecycleEvent(ev, currentSessionId)) {
            if (ev.type === 'ProcessStarted') streamStatusDetail = 'Started...';
            if (ev.type === 'ProcessCompleted') {
              streamStatus = 'completed';
              streamStatusDetail = `Done (exit ${ev.data?.exit_code ?? 0})`;
            }
            if (ev.type === 'ProcessFailed') {
              streamStatus = 'failed';
              streamStatusDetail = ev.data?.error ?? 'Process failed';
            }
          }

          // --- Sub-agent event handling ---
          if (ev.type === 'SubAgentRequested' && ev.data) {
            const parentSid = ev.data.parent_session_id as string | undefined;
            if (parentSid === currentSessionId) {
              const subId = (ev.data.sub_agent_id as string) ?? 'unknown';
              subAgents = [...subAgents, {
                agentId: subId,
                status: 'requested',
                prompt: (ev.data.prompt as string) ?? undefined,
                timestamp: (ev.data.timestamp as string) ?? new Date().toISOString(),
              }];
              // Initialize swim-lane column
              if (!swimLaneColumns[subId]) {
                swimLaneColumns[subId] = {
                  agentId: subId,
                  status: 'pending',
                  events: [],
                };
                swimLaneColumns = swimLaneColumns;
              }
            }
          }
          if (ev.type === 'SubAgentStarted' && ev.data) {
            const parentSid = ev.data.parent_session_id as string | undefined;
            if (parentSid === currentSessionId) {
              const subId = ev.data.sub_agent_id as string;
              const subSessionId = (ev.data.session_id as string) ?? undefined;
              const idx = subAgents.findIndex(sa => sa.agentId === subId && sa.status === 'requested');
              if (idx >= 0) {
                subAgents[idx].status = 'running';
                subAgents[idx].sessionId = subSessionId;
                subAgents = subAgents;
              } else {
                subAgents = [...subAgents, {
                  agentId: subId ?? 'unknown',
                  status: 'running',
                  sessionId: subSessionId,
                  timestamp: (ev.data.timestamp as string) ?? new Date().toISOString(),
                }];
              }
              // Update swim-lane column
              if (!swimLaneColumns[subId]) {
                swimLaneColumns[subId] = { agentId: subId, status: 'running', sessionId: subSessionId, events: [] };
              } else {
                swimLaneColumns[subId].status = 'running';
                swimLaneColumns[subId].sessionId = subSessionId;
              }
              swimLaneColumns = swimLaneColumns;
            }
          }
          if (ev.type === 'SubAgentCompleted' && ev.data) {
            const parentSid = ev.data.parent_session_id as string | undefined;
            if (parentSid === currentSessionId) {
              const subId = ev.data.sub_agent_id as string;
              const idx = subAgents.findIndex(sa => sa.agentId === subId && (sa.status === 'running' || sa.status === 'requested'));
              if (idx >= 0) {
                subAgents[idx].status = 'completed';
                subAgents = subAgents;
              }
              if (swimLaneColumns[subId]) {
                swimLaneColumns[subId].status = 'completed';
                swimLaneColumns = swimLaneColumns;
              }
            }
          }
          if (ev.type === 'SubAgentFailed' && ev.data) {
            const parentSid = ev.data.parent_session_id as string | undefined;
            if (parentSid === currentSessionId) {
              const subId = ev.data.sub_agent_id as string;
              const idx = subAgents.findIndex(sa => sa.agentId === subId && (sa.status === 'running' || sa.status === 'requested'));
              if (idx >= 0) {
                subAgents[idx].status = 'failed';
                subAgents[idx].error = (ev.data.error as string) ?? undefined;
                subAgents = subAgents;
              }
              if (swimLaneColumns[subId]) {
                swimLaneColumns[subId].status = 'failed';
                swimLaneColumns = swimLaneColumns;
              }
            }
          }

          // --- Route ProcessOutput to swim-lane columns ---
          if (ev.type === 'ProcessOutput' && ev.data?.session_id && ev.data?.content !== undefined) {
            const evSessionId = ev.data.session_id as string;
            // Check if this output belongs to a sub-agent session
            for (const col of Object.values(swimLaneColumns)) {
              if (col.sessionId === evSessionId) {
                const kind = normalizeBlockKind(ev.data.kind ?? 'assistant');
                const content = typeof ev.data.content === 'string' ? ev.data.content : String(ev.data.content);
                const lastEvt = col.events.length > 0 ? col.events[col.events.length - 1] : null;
                if (lastEvt && lastEvt.kind === kind) {
                  lastEvt.content += content;
                } else {
                  col.events.push({ kind, content, timestamp: new Date().toISOString() });
                }
                swimLaneColumns = swimLaneColumns;
                break;
              }
            }
          }
        } catch {
          // ignore parse errors
        }
      };
      ws.onclose = () => {
        ws = null;
        if (streamStatus === 'streaming' || streamStatus === 'connecting') {
          streamStatusDetail = 'WebSocket closed — reconnecting...';
        }
        scheduleReconnect();
      };
      ws.onerror = () => {
        if (streamStatus === 'connecting') streamStatusDetail = 'WebSocket error';
      };
    } catch (e) {
      runError = e instanceof Error ? e.message : String(e);
      scheduleReconnect();
    }
  }

  function scheduleReconnect() {
    if (wsReconnectTimer) return;
    wsReconnectTimer = setTimeout(() => {
      wsReconnectTimer = null;
      connectWs();
    }, wsReconnectDelay);
    wsReconnectDelay = Math.min(wsReconnectDelay * 2, WS_MAX_RECONNECT_DELAY);
  }

  async function run() {
    if (!selectedAgentId?.trim() || !prompt.trim()) {
      runError = 'Select an agent and enter a prompt.';
      return;
    }
    runError = '';
    outputBlocks = [];
    subAgents = [];
    streamStatus = 'connecting';
    streamStatusDetail = resumeSessionId ? 'Resuming...' : 'Starting...';
    running = true;
    connectWs();

    try {
      const res = await runAgent({
        agent_id: selectedAgentId,
        prompt: prompt.trim(),
        session_id: resumeSessionId,
        directory: directory.trim() || undefined,
      });
      currentSessionId = res.session_id;
      streamStatus = 'streaming';
      streamStatusDetail = 'Streaming...';
    } catch (e) {
      runError = e instanceof Error ? e.message : String(e);
      streamStatus = 'failed';
      streamStatusDetail = runError;
      currentSessionId = null;
    } finally {
      running = false;
    }
  }

  function clearStream() {
    outputBlocks = [];
    subAgents = [];
    swimLaneColumns = {};
    swimLanePinned = true;
    streamStatus = 'idle';
    streamStatusDetail = '';
    currentSessionId = null;
  }

  let sortedSwimLaneColumns = $derived(Object.values(swimLaneColumns).sort((a, b) => a.agentId.localeCompare(b.agentId)));

  let swimLaneContainer: HTMLElement | undefined = $state(undefined);

  $effect(() => {
    // Auto-scroll swim-lane columns when pinned
    if (swimLanePinned && swimLaneContainer) {
      // Access swimLaneColumns to create dependency
      const _cols = swimLaneColumns;
      void _cols;
      // Scroll each column body to bottom
      const bodies = swimLaneContainer.querySelectorAll('.swim-col-body');
      for (const body of bodies) {
        body.scrollTop = body.scrollHeight;
      }
    }
  });

  onMount(() => {
    loadAgents();
    connectWs();
    return () => {
      if (wsReconnectTimer) clearTimeout(wsReconnectTimer);
      ws?.close();
    };
  });
</script>

<svelte:head>
  <title>Dashboard - Claude Forge</title>
</svelte:head>

<div class="page dashboard">
  <section class="run-section">
    <h2>Run</h2>
    {#if resumeSessionId}
      <p class="resume-badge">Resuming session <code>{resumeSessionId.slice(0, 8)}...</code></p>
    {/if}
    {#if agentsError}
      <p class="error">{agentsError}</p>
    {:else}
      <div class="form run-form">
        <label for="agent-select">Agent</label>
        <select id="agent-select" bind:value={selectedAgentId} disabled={running}>
          {#each agents as a}
            <option value={a.id}>{a.name}</option>
          {/each}
        </select>
        <label for="prompt-input">Prompt</label>
        <textarea
          id="prompt-input"
          bind:value={prompt}
          placeholder="Enter your prompt..."
          rows="4"
          disabled={running}
        ></textarea>
        <label for="directory-input">Working Directory <span class="optional">(optional)</span></label>
        <input
          id="directory-input"
          type="text"
          bind:value={directory}
          placeholder="/path/to/project"
          disabled={running}
        />
        <div class="form-actions">
          <button type="button" class="primary" onclick={run} disabled={running || agents.length === 0}>
            {running ? 'Running...' : 'Run'}
          </button>
          {#if outputBlocks.length > 0 || streamStatus !== 'idle'}
            <button type="button" class="secondary" onclick={clearStream}>Clear</button>
          {/if}
        </div>
      </div>
    {/if}
    {#if runError}
      <p class="error">{runError}</p>
    {/if}
  </section>

  <section class="stream-section">
    <h2>Output</h2>
    {#if streamStatusDetail && streamStatus !== 'idle'}
      <p class="stream-status" class:completed={streamStatus === 'completed'} class:failed={streamStatus === 'failed'}>
        {streamStatusDetail}
      </p>
    {/if}

    {#if swimLaneMode}
      <!-- Swim-lane view: one column per sub-agent -->
      <div class="swim-lane-controls">
        <span class="swim-lane-label">Swim Lanes ({sortedSwimLaneColumns.length} agents)</span>
        <label class="pin-toggle">
          <input type="checkbox" bind:checked={swimLanePinned} />
          <span>Pin to bottom</span>
        </label>
      </div>
      <div class="swim-lane-container" bind:this={swimLaneContainer}>
        {#each sortedSwimLaneColumns as col (col.agentId)}
          <div class="swim-col" style="--lane-color: {STATUS_COLORS[col.status] ?? '#6b7280'}">
            <div class="swim-col-header">
              <span class="swim-col-name" title={col.agentId}>{col.agentId.slice(0, 12)}{col.agentId.length > 12 ? '...' : ''}</span>
              <span class="swim-col-badge" style="background: {STATUS_COLORS[col.status] ?? '#6b7280'}; color: #fff;">{col.status}</span>
            </div>
            <div class="swim-col-body">
              {#if col.events.length === 0}
                <span class="muted swim-col-empty">Waiting for output...</span>
              {:else}
                {#each col.events as evt}
                  <div class="swim-evt" class:swim-evt-tool={evt.kind === 'tool_use' || evt.kind === 'tool_result'} class:swim-evt-thinking={evt.kind === 'thinking'}>
                    <span class="swim-evt-icon">{KIND_ICONS[evt.kind]}</span>
                    <span class="swim-evt-content">{evt.content.slice(0, 200)}{evt.content.length > 200 ? '...' : ''}</span>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        {/each}
      </div>

      <!-- Also show orchestrator output below the swim lanes -->
      {#if outputBlocks.length > 0}
        <div class="stream-output" style="margin-top: 1rem;">
          <h3 style="margin: 0 0 0.5rem 0; font-size: 0.9rem; color: var(--muted);">Orchestrator Output</h3>
          {#each outputBlocks as block}
            {#if block.kind === 'assistant' || block.kind === 'result'}
              <div class="block-assistant stream-rendered">{@html renderStreamMarkdown(block.content)}</div>
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
        </div>
      {/if}
    {:else}
      <!-- Flat output log (no sub-agents) -->
      <div class="stream-output" class:empty={outputBlocks.length === 0}>
        {#if outputBlocks.length > 0}
          {#each outputBlocks as block}
            {#if block.kind === 'assistant' || block.kind === 'result'}
              <div class="block-assistant stream-rendered">{@html renderStreamMarkdown(block.content)}</div>
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
        {:else}
          <span class="muted">Run an agent to see streaming output here.</span>
        {/if}
      </div>
    {/if}
  </section>

  {#if subAgents.length > 0 && !swimLaneMode}
    <section class="subagents-section">
      <h2>Sub-agents</h2>
      <div class="subagent-grid">
        {#each subAgents as sa}
          <div class="subagent-card" class:requested={sa.status === 'requested'} class:running={sa.status === 'running'} class:completed={sa.status === 'completed'} class:failed={sa.status === 'failed'}>
            <div class="subagent-header">
              <span class="subagent-id">{sa.agentId.slice(0, 8)}...</span>
              <span class="status-badge {sa.status}">{sa.status}</span>
            </div>
            {#if sa.prompt}<p class="subagent-prompt">{sa.prompt.slice(0, 80)}{sa.prompt.length > 80 ? '...' : ''}</p>{/if}
            {#if sa.error}<p class="subagent-error">{sa.error}</p>{/if}
          </div>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .page {
    max-width: 52rem;
  }
  .dashboard h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1.1rem;
    font-weight: 600;
  }
  .run-section {
    margin-bottom: 1.5rem;
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .form label {
    font-size: 0.85rem;
    color: var(--muted);
  }
  .form select,
  .form textarea,
  .form input[type='text'] {
    padding: 0.5rem 0.75rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-family: inherit;
  }
  .optional {
    font-weight: 400;
    color: var(--muted);
    font-size: 0.8rem;
  }
  .form textarea {
    resize: vertical;
    min-height: 4rem;
  }
  .form-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .form-actions button {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    border: none;
  }
  .form-actions button.primary {
    background: var(--accent);
    color: var(--bg);
  }
  .form-actions button.primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .form-actions button.secondary {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }
  .error {
    color: #f87171;
    font-size: 0.9rem;
    margin: 0.5rem 0 0 0;
  }
  .stream-section {
    margin-top: 1rem;
  }
  .stream-status {
    font-size: 0.85rem;
    color: var(--muted);
    margin: 0 0 0.5rem 0;
  }
  .stream-status.completed {
    color: #86efac;
  }
  .stream-status.failed {
    color: #f87171;
  }
  .stream-output {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1rem;
    min-height: 8rem;
  }
  .stream-output.empty {
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .stream-rendered {
    font-size: 0.9rem;
    line-height: 1.5;
  }
  .stream-rendered :global(h1) { font-size: 1.25rem; margin: 0 0 0.5rem 0; }
  .stream-rendered :global(h2) { font-size: 1.1rem; margin: 0.75rem 0 0.4rem 0; }
  .stream-rendered :global(h3) { font-size: 1rem; margin: 0.5rem 0 0.3rem 0; }
  .stream-rendered :global(ul), .stream-rendered :global(ol) {
    margin: 0.25rem 0;
    padding-left: 1.5rem;
  }
  .stream-rendered :global(pre) {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.75rem;
    overflow-x: auto;
    margin: 0.5rem 0;
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
  }
  .stream-rendered :global(code) {
    font-family: ui-monospace, monospace;
  }
  .stream-rendered :global(p) { margin: 0.5rem 0; }
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
  .muted {
    color: var(--muted);
  }
  .resume-badge {
    font-size: 0.85rem;
    color: var(--accent);
    margin: 0 0 0.5rem 0;
  }
  .resume-badge code {
    background: var(--surface);
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
  }

  /* --- Sub-agent progress panel --- */
  .subagents-section {
    margin-top: 1.5rem;
  }
  .subagent-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
  }
  .subagent-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0.75rem;
  }
  .subagent-card.requested {
    border-color: #71717a;
  }
  .subagent-card.running {
    border-color: #60a5fa;
  }
  .subagent-card.completed {
    border-color: #86efac;
  }
  .subagent-card.failed {
    border-color: #f87171;
  }
  .subagent-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.35rem;
  }
  .subagent-id {
    font-family: ui-monospace, monospace;
    font-size: 0.8rem;
    color: var(--text);
  }
  .status-badge {
    font-size: 0.7rem;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .status-badge.requested {
    background: rgba(113, 113, 122, 0.2);
    color: #71717a;
  }
  .status-badge.running {
    background: rgba(96, 165, 250, 0.2);
    color: #60a5fa;
  }
  .status-badge.completed {
    background: rgba(134, 239, 172, 0.2);
    color: #86efac;
  }
  .status-badge.failed {
    background: rgba(248, 113, 113, 0.2);
    color: #f87171;
  }
  .subagent-prompt {
    margin: 0.25rem 0 0 0;
    font-size: 0.8rem;
    color: var(--muted);
    line-height: 1.3;
  }
  .subagent-error {
    margin: 0.25rem 0 0 0;
    font-size: 0.8rem;
    color: #f87171;
    line-height: 1.3;
  }

  /* --- Swim-lane view --- */
  .swim-lane-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }
  .swim-lane-label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--muted);
  }
  .pin-toggle {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.8rem;
    color: var(--muted);
    cursor: pointer;
  }
  .pin-toggle input {
    cursor: pointer;
  }
  .swim-lane-container {
    display: flex;
    gap: 0.75rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
  }
  .swim-col {
    flex: 0 0 16rem;
    min-width: 16rem;
    max-width: 20rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-top: 3px solid var(--lane-color, var(--border));
    border-radius: 8px;
    display: flex;
    flex-direction: column;
  }
  .swim-col-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }
  .swim-col-name {
    font-size: 0.8rem;
    font-family: ui-monospace, monospace;
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .swim-col-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    flex-shrink: 0;
  }
  .swim-col-body {
    flex: 1;
    max-height: 24rem;
    overflow-y: auto;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .swim-col-empty {
    font-size: 0.8rem;
    text-align: center;
    padding: 1rem 0;
  }
  .swim-evt {
    display: flex;
    gap: 0.35rem;
    align-items: flex-start;
    padding: 0.35rem 0.5rem;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
  }
  .swim-evt-tool {
    border-left: 2px solid var(--accent);
    background: rgba(167, 139, 250, 0.04);
  }
  .swim-evt-thinking {
    border-left: 2px solid var(--border);
    opacity: 0.7;
  }
  .swim-evt-icon {
    flex-shrink: 0;
    font-size: 0.8rem;
    line-height: 1.3;
  }
  .swim-evt-content {
    font-size: 0.78rem;
    color: var(--text);
    line-height: 1.35;
    word-break: break-word;
    overflow: hidden;
  }
</style>
