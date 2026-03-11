<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { listCompanies, createCompany, type Company } from '$lib/api';

  setContext('pageTitle', 'Companies');

  let companies = $state<Company[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let formOpen = $state(false);
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  let name = $state('');
  let mission = $state('');
  let budgetLimit = $state('');

  async function loadCompanies() {
    loading = true;
    error = null;
    try {
      companies = await listCompanies();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function openForm() {
    name = '';
    mission = '';
    budgetLimit = '';
    formError = null;
    formOpen = true;
  }

  function closeForm() {
    formOpen = false;
    formError = null;
  }

  async function submit() {
    if (!name.trim()) {
      formError = 'Name is required';
      return;
    }
    submitting = true;
    formError = null;
    try {
      const payload: { name: string; mission?: string; budget_limit?: number } = {
        name: name.trim(),
      };
      if (mission.trim()) payload.mission = mission.trim();
      const bl = budgetLimit.trim();
      if (bl) {
        const n = Number(bl);
        if (!Number.isNaN(n)) payload.budget_limit = n;
      }
      await createCompany(payload);
      closeForm();
      await loadCompanies();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  function formatBudget(c: Company): string {
    if (c.budget_limit == null) return `$${c.budget_used.toFixed(2)} used`;
    return `$${c.budget_used.toFixed(2)} / $${c.budget_limit.toFixed(2)}`;
  }

  onMount(() => {
    loadCompanies();
  });
</script>

<svelte:head>
  <title>Companies - Claude Forge</title>
</svelte:head>

<div class="companies-page">
  <header class="page-header">
    <h1>Companies</h1>
    <button class="btn btn-primary" onclick={openForm}>New company</button>
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading companies...</p>
  {:else if companies.length === 0}
    <div class="empty-state">
      <p class="muted">No companies defined yet. Create one to start organizing agents into org charts.</p>
      <button class="btn btn-primary" onclick={openForm}>Create company</button>
    </div>
  {:else}
    <div class="company-cards">
      {#each companies as c (c.id)}
        <article class="card">
          <header class="card-header">
            <h2 class="card-title">{c.name}</h2>
          </header>
          {#if c.mission}
            <p class="card-mission">{c.mission}</p>
          {/if}
          <p class="card-budget">
            <span class="label">Budget</span>
            <span>{formatBudget(c)}</span>
          </p>
        </article>
      {/each}
    </div>
  {/if}

  {#if formOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="company-form-title">
      <div class="modal">
        <h2 id="company-form-title">Create company</h2>
        {#if formError}
          <div class="message error">{formError}</div>
        {/if}
        <form
          class="company-form"
          onsubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          <label>
            <span>Name</span>
            <input type="text" bind:value={name} required placeholder="Company name" />
          </label>
          <label>
            <span>Mission</span>
            <textarea bind:value={mission} rows="3" placeholder="Short mission or purpose"></textarea>
          </label>
          <label>
            <span>Budget limit (USD)</span>
            <input type="number" min="0" step="0.01" bind:value={budgetLimit} placeholder="Optional budget cap" />
          </label>
          <div class="form-actions">
            <button type="button" class="btn btn-ghost" onclick={closeForm}>Cancel</button>
            <button type="submit" class="btn btn-primary" disabled={submitting}>
              {submitting ? 'Creating...' : 'Create'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .companies-page {
    max-width: 48rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .company-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
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

  .card-title {
    margin: 0;
    font-size: 1.1rem;
  }

  .card-mission {
    margin: 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .card-budget {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    margin: 0.25rem 0 0 0;
  }

  .card-budget .label {
    color: var(--muted);
  }

  .empty-state {
    padding: 2rem 1rem;
    text-align: center;
  }

  .muted {
    color: var(--muted);
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-family: inherit;
  }

  .btn-primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #0f0f12;
  }

  .btn-ghost {
    background: transparent;
    border-color: transparent;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    width: 100%;
    max-width: 28rem;
    max-height: 90vh;
    overflow: auto;
  }

  .company-form label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.75rem;
  }

  .company-form label span {
    font-size: 0.85rem;
    color: var(--muted);
  }

  .company-form input,
  .company-form textarea {
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.9rem;
    font-family: inherit;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }
</style>

