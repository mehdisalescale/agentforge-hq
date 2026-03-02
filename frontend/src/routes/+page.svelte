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

  setContext('pageTitle', 'Dashboard');

  let agents: Agent[] = [];
  let agentsError = '';
  let selectedAgentId = '';
  let prompt = '';
  let directory = '';
  let running = false;
  let runError = '';
  let outputBlocks: OutputBlock[] = [];
  let streamStatus: 'idle' | 'connecting' | 'streaming' | 'completed' | 'failed' = 'idle';
  let streamStatusDetail = '';
  let currentSessionId: string | null = null;
  let ws: WebSocket | null = null;
  let wsReconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let wsReconnectDelay = 1000;
  const WS_MAX_RECONNECT_DELAY = 30000;

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
            if (ev.type === 'ProcessStarted') streamStatusDetail = 'Started…';
            if (ev.type === 'ProcessCompleted') {
              streamStatus = 'completed';
              streamStatusDetail = `Done (exit ${ev.data?.exit_code ?? 0})`;
            }
            if (ev.type === 'ProcessFailed') {
              streamStatus = 'failed';
              streamStatusDetail = ev.data?.error ?? 'Process failed';
            }
          }
        } catch {
          // ignore parse errors
        }
      };
      ws.onclose = () => {
        ws = null;
        if (streamStatus === 'streaming' || streamStatus === 'connecting') {
          streamStatusDetail = 'WebSocket closed — reconnecting…';
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

  /** Session ID from ?resume= (Sessions page "Resume"). Only in browser (prerender-safe). */
  $: resumeSessionId = browser ? ($page.url.searchParams.get('resume') || null) : null;

  async function run() {
    if (!selectedAgentId?.trim() || !prompt.trim()) {
      runError = 'Select an agent and enter a prompt.';
      return;
    }
    runError = '';
      outputBlocks = [];
    streamStatus = 'connecting';
    streamStatusDetail = resumeSessionId ? 'Resuming…' : 'Starting…';
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
      streamStatusDetail = 'Streaming…';
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
    streamStatus = 'idle';
    streamStatusDetail = '';
    currentSessionId = null;
  }

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
  <title>Dashboard · Claude Forge</title>
</svelte:head>

<div class="page dashboard">
  <section class="run-section">
    <h2>Run</h2>
    {#if resumeSessionId}
      <p class="resume-badge">Resuming session <code>{resumeSessionId.slice(0, 8)}…</code></p>
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
          placeholder="Enter your prompt…"
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
          <button type="button" class="primary" on:click={run} disabled={running || agents.length === 0}>
            {running ? 'Running…' : 'Run'}
          </button>
          {#if outputBlocks.length > 0 || streamStatus !== 'idle'}
            <button type="button" class="secondary" on:click={clearStream}>Clear</button>
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
              <summary>Thinking…</summary>
              <pre class="dimmed">{block.content}</pre>
            </details>
          {/if}
        {/each}
      {:else}
        <span class="muted">Run an agent to see streaming output here.</span>
      {/if}
    </div>
  </section>
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
</style>
