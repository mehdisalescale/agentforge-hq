<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { listWorkflows, type Workflow } from '$lib/api';

  setContext('pageTitle', 'Workflows');

  let workflows = $state<Workflow[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

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

<div class="page workflows-page">
  <h1>Workflows</h1>
  {#if error}
    <p class="error">{error}</p>
    <p class="muted">Workflows API may not be available. Ensure the backend is running.</p>
  {:else if loading}
    <p class="muted">Loading workflows…</p>
  {:else if workflows.length === 0}
    <p class="muted">No workflows yet.</p>
  {:else}
    <ul class="workflows-list">
      {#each workflows as w (w.id)}
        <li class="workflow-item">
          <span class="workflow-name">{w.name}</span>
          {#if w.description}
            <p class="workflow-desc">{w.description}</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .page h1 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
  }
  .workflows-page .error {
    color: #f87171;
    margin: 0 0 0.5rem 0;
  }
  .workflows-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .workflow-item {
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .workflow-name {
    font-weight: 600;
  }
  .workflow-desc {
    margin: 0.35rem 0 0 0;
    font-size: 0.9rem;
    color: var(--muted);
  }
</style>
