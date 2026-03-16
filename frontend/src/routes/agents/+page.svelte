<script lang="ts">
  import { setContext } from 'svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import ErrorMessage from '$lib/components/ErrorMessage.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { focusTrap } from '$lib/actions/focusTrap';
  import { Bot } from 'lucide-svelte';
  import {
    listAgents,
    createAgent,
    updateAgent,
    deleteAgent,
    getAgent,
    getAllAgentStats,
    type Agent,
    type AgentStats,
    type NewAgent,
    type UpdateAgent,
    PRESETS,
  } from '$lib/api';

  setContext('pageTitle', 'Agents');

  // Extended presets list including Coordinator (10th preset added to backend)
  const ALL_PRESETS = [...PRESETS, 'Coordinator' as const];

  // Domain mapping: preset -> domain category
  const DOMAIN_MAP: Record<string, { label: string; color: string }> = {
    CodeWriter:     { label: 'code',          color: '#60a5fa' },
    Refactorer:     { label: 'code',          color: '#60a5fa' },
    Reviewer:       { label: 'quality',       color: '#86efac' },
    Tester:         { label: 'quality',       color: '#86efac' },
    SecurityAuditor:{ label: 'quality',       color: '#86efac' },
    Architect:      { label: 'ops',           color: '#fbbf24' },
    Documenter:     { label: 'ops',           color: '#fbbf24' },
    Explorer:       { label: 'ops',           color: '#fbbf24' },
    Debugger:       { label: 'ops',           color: '#fbbf24' },
    Coordinator:    { label: 'orchestration', color: '#c084fc' },
  };

  function getDomain(preset: string | null): { label: string; color: string } | null {
    if (!preset) return null;
    return DOMAIN_MAP[preset] ?? null;
  }

  let agents = $state<Agent[]>([]);
  let agentStats = $state<Record<string, AgentStats>>({});
  let loading = $state(true);
  let error = $state<string | null>(null);
  let formOpen = $state<'create' | 'edit' | null>(null);
  let editId = $state<string | null>(null);
  let deleteConfirmId = $state<string | null>(null);
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  // Form fields (shared for create/edit)
  let formName = $state('');
  let formModel = $state('');
  let formSystemPrompt = $state('');
  let formPreset = $state<string>('');
  let formMaxTurns = $state<string>('');
  let formUseMax = $state(false);
  let formBackendType = $state('claude');

  async function loadAgents() {
    loading = true;
    error = null;
    try {
      agents = await listAgents();
      getAllAgentStats().then((stats) => { agentStats = stats; }).catch((e) => console.warn('failed to load agent stats:', e));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function openCreate() {
    editId = null;
    formName = '';
    formModel = 'claude-sonnet-4-20250514';
    formSystemPrompt = '';
    formPreset = '';
    formMaxTurns = '';
    formUseMax = false;
    formBackendType = 'claude';
    formError = null;
    formOpen = 'create';
  }

  async function openEdit(id: string) {
    formError = null;
    try {
      const a = await getAgent(id);
      editId = id;
      formName = a.name;
      formModel = a.model;
      formSystemPrompt = a.system_prompt ?? '';
      formPreset = a.preset ?? '';
      formMaxTurns = a.max_turns != null ? String(a.max_turns) : '';
      formUseMax = a.use_max;
      formBackendType = a.backend_type || 'claude';
      formOpen = 'edit';
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    }
  }

  function closeForm() {
    formOpen = null;
    editId = null;
    formError = null;
  }

  function getPayload(): NewAgent & UpdateAgent {
    const payload: NewAgent & UpdateAgent = {
      name: formName.trim(),
      model: formModel.trim() || undefined,
      system_prompt: formSystemPrompt.trim() || undefined,
      preset: formPreset ? (formPreset as Agent['preset']) : undefined,
      use_max: formUseMax,
      backend_type: formBackendType || 'claude',
    };
    const mt = formMaxTurns.trim();
    if (mt) {
      const n = parseInt(mt, 10);
      if (!Number.isNaN(n)) payload.max_turns = n;
    }
    return payload;
  }

  async function submitCreate() {
    if (!formName.trim()) {
      formError = 'Name is required';
      return;
    }
    submitting = true;
    formError = null;
    try {
      await createAgent(getPayload());
      closeForm();
      await loadAgents();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function submitEdit() {
    if (!editId || !formName.trim()) {
      formError = editId ? 'Name is required' : 'Invalid agent';
      return;
    }
    submitting = true;
    formError = null;
    try {
      await updateAgent(editId, getPayload());
      closeForm();
      await loadAgents();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function doDelete(id: string) {
    try {
      await deleteAgent(id);
      deleteConfirmId = null;
      await loadAgents();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  loadAgents();
</script>

<svelte:head>
  <title>Agents · AgentForge</title>
</svelte:head>

<div class="agents-page" aria-busy={loading}>
  <header class="page-header">
    <h1>Agents</h1>
    <button class="btn btn-primary" onclick={openCreate}>New agent</button>
  </header>

  {#if error}
    <ErrorMessage message={error} onretry={loadAgents} />
  {/if}

  {#if loading}
    <Skeleton type="card" lines={3} />
  {:else if agents.length === 0}
    <EmptyState
      icon={Bot}
      title="No agents yet"
      description="Create or hire agents to get started with your AI workforce."
      actionLabel="Create agent"
      onaction={openCreate}
    />
  {:else}
    <div class="agent-cards">
      {#each agents as agent (agent.id)}
        {@const domain = getDomain(agent.preset)}
        <article class="card">
          <div class="card-header">
            <h2 class="card-title">{agent.name}</h2>
            <span class="card-meta">{agent.model}</span>
            <span class="backend-badge">{agent.backend_type}</span>
            {#if agent.preset}
              <span class="badge">{agent.preset}</span>
            {/if}
            {#if domain}
              <span class="domain-badge" style="--domain-color: {domain.color}">{domain.label}</span>
            {/if}
            {#if agent.persona_id}
              <span class="hired-badge">Hired from catalog</span>
            {/if}
          </div>
          {#if agent.system_prompt}
            <p class="card-prompt">{agent.system_prompt.slice(0, 120)}{agent.system_prompt.length > 120 ? '…' : ''}</p>
          {/if}
          {#if agentStats[agent.id]}
            {@const stats = agentStats[agent.id]}
            <div class="agent-stats">
              <span class="stat"><strong>{stats.run_count}</strong> runs</span>
              <span class="stat"><strong>${stats.total_cost.toFixed(4)}</strong> cost</span>
              {#if stats.success_rate > 0}
                <span class="stat"><strong>{stats.success_rate.toFixed(0)}%</strong> success</span>
              {/if}
              {#if stats.last_run}
                <span class="stat">Last: {new Date(stats.last_run).toLocaleDateString()}</span>
              {/if}
            </div>
          {/if}
          <div class="card-actions">
            <button class="btn btn-ghost" onclick={() => openEdit(agent.id)}>Edit</button>
            <button
              class="btn btn-ghost danger"
              onclick={() => (deleteConfirmId = agent.id)}
              aria-label="Delete {agent.name}"
            >
              Delete
            </button>
          </div>
          {#if deleteConfirmId === agent.id}
            <div class="delete-confirm">
              <span>Delete this agent?</span>
              <button class="btn btn-ghost" onclick={() => (deleteConfirmId = null)}>Cancel</button>
              <button class="btn danger" onclick={() => doDelete(agent.id)}>Delete</button>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}

  {#if formOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="form-title" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeForm()}>
      <div class="modal" use:focusTrap>
        <h2 id="form-title">{formOpen === 'create' ? 'Create agent' : 'Edit agent'}</h2>
        {#if formError}
          <div class="message error">{formError}</div>
        {/if}
        <form
          class="agent-form"
          onsubmit={(e) => {
            e.preventDefault();
            if (formOpen === 'create') submitCreate();
            else submitEdit();
          }}
        >
          <label>
            <span>Name</span>
            <input type="text" bind:value={formName} required placeholder="Agent name" />
          </label>
          <label>
            <span>Model</span>
            <input type="text" bind:value={formModel} placeholder="e.g. claude-sonnet-4-20250514" />
          </label>
          <label>
            <span>Backend</span>
            <select bind:value={formBackendType}>
              <option value="claude">claude</option>
              <option value="openai">openai</option>
              <option value="gemini">gemini</option>
              <option value="ollama">ollama</option>
              <option value="custom">custom</option>
            </select>
          </label>
          <label>
            <span>Preset</span>
            <select bind:value={formPreset}>
              <option value="">None</option>
              {#each ALL_PRESETS as p}
                <option value={p}>{p}</option>
              {/each}
            </select>
          </label>
          <label>
            <span>System prompt</span>
            <textarea bind:value={formSystemPrompt} rows="4" placeholder="Optional system prompt"></textarea>
          </label>
          <label class="row">
            <span>Max turns</span>
            <input type="number" bind:value={formMaxTurns} min="1" placeholder="Optional" />
          </label>
          <label class="row checkbox">
            <input type="checkbox" bind:checked={formUseMax} />
            <span>Use max turns</span>
          </label>
          <div class="form-actions">
            <button type="button" class="btn btn-ghost" onclick={closeForm}>Cancel</button>
            <button type="submit" class="btn btn-primary" disabled={submitting}>
              {submitting ? 'Saving…' : formOpen === 'create' ? 'Create' : 'Save'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .domain-badge {
    font-size: 0.65rem;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    background: color-mix(in srgb, var(--domain-color) 18%, transparent);
    color: var(--domain-color);
    border: 1px solid color-mix(in srgb, var(--domain-color) 30%, transparent);
  }
  .agent-stats {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding: 0.4rem 0;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    margin-top: 0.25rem;
  }
  .agent-stats .stat {
    font-size: 0.7rem;
    color: #94a3b8;
    background: rgba(255, 255, 255, 0.04);
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
  }
  .agent-stats .stat strong {
    color: #e2e8f0;
  }
  .backend-badge {
    font-size: 0.65rem;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    background: rgba(96, 165, 250, 0.12);
    color: #60a5fa;
    border: 1px solid rgba(96, 165, 250, 0.25);
  }
  .hired-badge {
    font-size: 0.65rem;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    background: rgba(134, 239, 172, 0.12);
    color: #86efac;
    border: 1px solid rgba(134, 239, 172, 0.25);
  }
</style>
