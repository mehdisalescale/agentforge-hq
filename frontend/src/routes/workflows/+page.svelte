<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { listWorkflows, type Workflow } from '$lib/api';

  setContext('pageTitle', 'Workflows');

  let workflows = $state<Workflow[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  function parseDefinition(w: Workflow): { steps: { name: string; agent?: string }[] } {
    try {
      const def = JSON.parse(w.definition_json);
      if (Array.isArray(def?.steps)) return { steps: def.steps };
    } catch {
      // ignore
    }
    return { steps: [] };
  }

  async function loadWorkflows() {
    loading = true;
    error = null;
    try {
      workflows = await listWorkflows();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      workflows = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadWorkflows();
  });
</script>

<svelte:head>
  <title>Workflows · Claude Forge</title>
</svelte:head>

<div class="workflows-page">
  <header class="page-header">
    <h1>Workflows</h1>
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
    <p class="muted">Workflows API may not be available. Ensure the backend is running.</p>
  {:else if loading}
    <p class="muted">Loading workflows...</p>
  {:else if workflows.length === 0}
    <div class="empty-state">
      <div class="workflow-placeholder">
        <div class="wf-diagram">
          <div class="wf-node wf-start">Start</div>
          <div class="wf-arrow"></div>
          <div class="wf-node wf-step">Agent A</div>
          <div class="wf-arrow"></div>
          <div class="wf-node wf-step">Agent B</div>
          <div class="wf-arrow"></div>
          <div class="wf-node wf-end">Done</div>
        </div>
      </div>
      <h2 class="empty-title">No workflows yet</h2>
      <p class="muted">Workflows are sequences of agent tasks that run in order. Define steps, assign agents, and let Forge orchestrate the pipeline.</p>
      <p class="muted hint">Create workflows via the API or define them in your project configuration.</p>
    </div>
  {:else}
    <div class="workflow-cards">
      {#each workflows as w (w.id)}
        {@const def = parseDefinition(w)}
        <article class="card">
          <div class="card-header">
            <h2 class="card-title">{w.name}</h2>
            {#if def.steps.length > 0}
              <span class="step-count">{def.steps.length} step{def.steps.length !== 1 ? 's' : ''}</span>
            {/if}
          </div>
          {#if w.description}
            <p class="card-desc">{w.description}</p>
          {/if}
          {#if def.steps.length > 0}
            <div class="wf-steps-inline">
              {#each def.steps as step, i}
                {#if i > 0}
                  <span class="wf-arrow-inline"></span>
                {/if}
                <span class="wf-step-chip">{step.name || step.agent || `Step ${i + 1}`}</span>
              {/each}
            </div>
          {/if}
          <div class="card-footer">
            <span class="card-meta">Created {new Date(w.created_at).toLocaleDateString()}</span>
            {#if w.updated_at !== w.created_at}
              <span class="card-meta">Updated {new Date(w.updated_at).toLocaleDateString()}</span>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .workflows-page {
    max-width: 56rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .page-header h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
  }

  .empty-title {
    margin: 1.5rem 0 0.5rem 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .hint {
    font-size: 0.85rem;
    margin-top: 0.5rem;
  }

  /* Placeholder workflow diagram */
  .workflow-placeholder {
    padding: 1.5rem;
    border: 1px dashed var(--border);
    border-radius: 8px;
    background: rgba(167, 139, 250, 0.03);
  }

  .wf-diagram {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    flex-wrap: wrap;
  }

  .wf-node {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 500;
    white-space: nowrap;
  }

  .wf-start {
    background: rgba(134, 239, 172, 0.15);
    color: #86efac;
    border: 1px solid rgba(134, 239, 172, 0.3);
  }

  .wf-step {
    background: rgba(167, 139, 250, 0.15);
    color: var(--accent);
    border: 1px solid rgba(167, 139, 250, 0.3);
  }

  .wf-end {
    background: rgba(251, 191, 36, 0.15);
    color: #fbbf24;
    border: 1px solid rgba(251, 191, 36, 0.3);
  }

  .wf-arrow {
    width: 2rem;
    height: 2px;
    background: var(--border);
    position: relative;
    margin: 0 0.25rem;
  }

  .wf-arrow::after {
    content: '';
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    border-left: 6px solid var(--border);
    border-top: 4px solid transparent;
    border-bottom: 4px solid transparent;
  }

  /* Workflow cards when populated */
  .workflow-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .card-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  .card-title {
    margin: 0;
    font-size: 1.1rem;
    flex: 1 1 auto;
  }

  .step-count {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .card-desc {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
    line-height: 1.4;
  }

  /* Inline step visualization */
  .wf-steps-inline {
    display: flex;
    align-items: center;
    gap: 0;
    flex-wrap: wrap;
    padding: 0.5rem 0;
  }

  .wf-step-chip {
    font-size: 0.75rem;
    padding: 0.2rem 0.6rem;
    border-radius: 4px;
    background: rgba(167, 139, 250, 0.15);
    color: var(--accent);
    border: 1px solid rgba(167, 139, 250, 0.25);
    white-space: nowrap;
  }

  .wf-arrow-inline {
    display: inline-block;
    width: 1.25rem;
    height: 2px;
    background: var(--border);
    position: relative;
    margin: 0 0.15rem;
    flex-shrink: 0;
  }

  .wf-arrow-inline::after {
    content: '';
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    border-left: 5px solid var(--border);
    border-top: 3px solid transparent;
    border-bottom: 3px solid transparent;
  }

  .card-footer {
    border-top: 1px solid var(--border);
    padding-top: 0.5rem;
    display: flex;
    gap: 1rem;
  }

  .card-meta {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .message.error {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .muted {
    color: var(--muted);
  }
</style>
