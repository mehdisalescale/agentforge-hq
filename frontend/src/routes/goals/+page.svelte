<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import {
    listCompanies,
    listGoals,
    createGoal,
    updateGoalStatus,
    type Company,
    type Goal,
    type GoalStatus,
  } from '$lib/api';
  import { Target as TargetIcon } from 'lucide-svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import ErrorMessage from '$lib/components/ErrorMessage.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { focusTrap } from '$lib/actions/focusTrap';

  setContext('pageTitle', 'Goals');

  let companies = $state<Company[]>([]);
  let selectedCompanyId = $state<string>('');
  let goals = $state<Goal[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let createOpen = $state(false);
  let createTitle = $state('');
  let createDescription = $state('');
  let createParentId = $state('');
  let createSubmitting = $state(false);
  let createError = $state<string | null>(null);

  async function loadInitial() {
    loading = true;
    error = null;
    try {
      companies = await listCompanies();
      if (companies.length > 0) {
        selectedCompanyId = companies[0].id;
        goals = await listGoals(selectedCompanyId);
      } else {
        goals = [];
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function changeCompany(id: string) {
    selectedCompanyId = id;
    if (!id) {
      goals = [];
      return;
    }
    loading = true;
    error = null;
    try {
      goals = await listGoals(id);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      goals = [];
    } finally {
      loading = false;
    }
  }

  function openCreate() {
    createTitle = '';
    createDescription = '';
    createParentId = '';
    createError = null;
    createOpen = true;
  }

  function closeCreate() {
    createOpen = false;
    createError = null;
  }

  async function submitCreate() {
    if (!selectedCompanyId) {
      createError = 'Select a company first.';
      return;
    }
    if (!createTitle.trim()) {
      createError = 'Title is required.';
      return;
    }
    createSubmitting = true;
    createError = null;
    try {
      await createGoal({
        company_id: selectedCompanyId,
        parent_id: createParentId || undefined,
        title: createTitle.trim(),
        description: createDescription.trim() || undefined,
      });
      closeCreate();
      goals = await listGoals(selectedCompanyId);
    } catch (e) {
      createError = e instanceof Error ? e.message : String(e);
    } finally {
      createSubmitting = false;
    }
  }

  async function onStatusChange(goal: Goal, status: GoalStatus) {
    try {
      const updated = await updateGoalStatus(goal.id, { status });
      goals = goals.map((g) => (g.id === updated.id ? updated : g));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(() => {
    loadInitial();
  });
</script>

<svelte:head>
  <title>Goals - AgentForge</title>
</svelte:head>

<div class="goals-page" aria-busy={loading}>
  <header class="page-header">
    <h1>Company goals</h1>
    <div class="toolbar">
      <label class="company-select">
        <span>Company</span>
        <select
          bind:value={selectedCompanyId}
          onchange={(e) => changeCompany((e.target as HTMLSelectElement).value)}
        >
          {#if companies.length === 0}
            <option value="">No companies yet</option>
          {:else}
            {#each companies as c}
              <option value={c.id}>{c.name}</option>
            {/each}
          {/if}
        </select>
      </label>
      <button class="btn" type="button" onclick={openCreate} disabled={!selectedCompanyId}>
        New goal
      </button>
    </div>
  </header>

  {#if error}
    <ErrorMessage message={error} onretry={loadInitial} />
  {/if}

  {#if loading}
    <Skeleton type="table" lines={4} />
  {:else if !selectedCompanyId}
    <p class="muted">Select a company to see its goals.</p>
  {:else if goals.length === 0}
    <EmptyState
      icon={TargetIcon}
      title="No goals yet"
      description="Define goals for this company to track progress and align your agent workforce."
      actionLabel="New goal"
      onaction={openCreate}
    />
  {:else}
    <table class="table">
      <thead>
        <tr>
          <th>Title</th>
          <th>Description</th>
          <th>Status</th>
          <th>Parent</th>
        </tr>
      </thead>
      <tbody>
        {#each goals as g}
          <tr>
            <td>{g.title}</td>
            <td class="muted">{g.description}</td>
            <td>
              <select
                value={g.status}
                onchange={(e) => onStatusChange(g, (e.target as HTMLSelectElement).value as GoalStatus)}
              >
                <option value="planned">Planned</option>
                <option value="in_progress">In progress</option>
                <option value="completed">Completed</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </td>
            <td>
              {#if g.parent_id}
                {#if goals.find((p) => p.id === g.parent_id)}
                  {goals.find((p) => p.id === g.parent_id)?.title}
                {:else}
                  {g.parent_id}
                {/if}
              {:else}
                —
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  {#if createOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeCreate()}>
      <div class="modal" use:focusTrap>
        <h2>Create goal</h2>
        {#if createError}
          <div class="message error">{createError}</div>
        {/if}
        <form
          class="goal-form"
          onsubmit={(e) => {
            e.preventDefault();
            submitCreate();
          }}
        >
          <label>
            <span>Title</span>
            <input type="text" bind:value={createTitle} required />
          </label>
          <label>
            <span>Description</span>
            <textarea rows="3" bind:value={createDescription} />
          </label>
          <label>
            <span>Parent goal (optional)</span>
            <select bind:value={createParentId}>
              <option value="">No parent</option>
              {#each goals as g}
                <option value={g.id}>{g.title}</option>
              {/each}
            </select>
          </label>
          <div class="form-actions">
            <button class="btn btn-ghost" type="button" onclick={closeCreate}>Cancel</button>
            <button class="btn" type="submit" disabled={createSubmitting}>
              {createSubmitting ? 'Creating…' : 'Create goal'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .goals-page {
    max-width: 64rem;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .toolbar {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .company-select {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .company-select select {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    font-size: 0.9rem;
  }

  .btn {
    padding: 0.45rem 0.9rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.9rem;
  }

  .btn-ghost {
    background: transparent;
    border-color: transparent;
  }

  .muted {
    color: var(--muted);
  }

  .table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .table th,
  .table td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    text-align: left;
  }

  .table th {
    font-weight: 500;
    color: var(--muted);
  }

  .table select {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    font-size: 0.85rem;
  }

  .message.error {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }

  .modal {
    background: var(--surface);
    border-radius: 8px;
    border: 1px solid var(--border);
    padding: 1.25rem;
    width: 100%;
    max-width: 28rem;
    max-height: 90vh;
    overflow: auto;
  }

  .goal-form {
    margin-top: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .goal-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .goal-form span {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .goal-form input,
  .goal-form textarea,
  .goal-form select {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.9rem;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
</style>

