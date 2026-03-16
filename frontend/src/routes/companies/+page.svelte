<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { listCompanies, createCompany, updateCompany, deleteCompany, type Company } from '$lib/api';
  import { Building2, Plus, DollarSign, Pencil, Trash2, X } from 'lucide-svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import ErrorMessage from '$lib/components/ErrorMessage.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { focusTrap } from '$lib/actions/focusTrap';

  setContext('pageTitle', 'Companies');

  let companies = $state<Company[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let formOpen = $state(false);
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  let name = $state('');
  let mission = $state('');
  let budgetLimit = $state<string | number>('');

  // Detail / edit state
  let selectedId = $state<string | null>(null);
  let editing = $state(false);
  let editName = $state('');
  let editMission = $state('');
  let editBudget = $state<string | number>('');
  let deleting = $state(false);

  let selectedCompany = $derived(companies.find(c => c.id === selectedId) ?? null);

  function selectCompany(id: string) {
    if (selectedId === id) {
      selectedId = null;
      editing = false;
    } else {
      selectedId = id;
      editing = false;
    }
  }

  function startEdit() {
    if (!selectedCompany) return;
    editName = selectedCompany.name;
    editMission = selectedCompany.mission || '';
    editBudget = selectedCompany.budget_limit ?? '';
    editing = true;
  }

  async function saveEdit() {
    if (!selectedCompany || !editName.trim()) return;
    submitting = true;
    try {
      const payload: Record<string, unknown> = { name: editName.trim() };
      if (editMission.trim()) payload.mission = editMission.trim();
      const bl = String(editBudget).trim();
      if (bl) {
        const n = Number(bl);
        if (!Number.isNaN(n) && n > 0) payload.budget_limit = n;
      }
      await updateCompany(selectedCompany.id, payload);
      editing = false;
      await loadCompanies();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function confirmDelete() {
    if (!selectedCompany) return;
    deleting = true;
    try {
      await deleteCompany(selectedCompany.id);
      selectedId = null;
      await loadCompanies();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      deleting = false;
    }
  }

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
      const bl = String(budgetLimit).trim();
      if (bl) {
        const n = Number(bl);
        if (!Number.isNaN(n) && n > 0) payload.budget_limit = n;
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
  <title>Companies - AgentForge</title>
</svelte:head>

<div class="companies-page" aria-busy={loading}>
  <header class="page-header">
    <div>
      <h1>Companies</h1>
      <p class="page-desc">Create and manage your organizations. Each company has its own org chart, budget, and team of agents.</p>
    </div>
    <button class="btn btn-primary" onclick={openForm}><Plus size={16} /> New company</button>
  </header>

  {#if error}
    <ErrorMessage message={error} onretry={loadCompanies} />
  {/if}

  {#if loading}
    <Skeleton type="card" lines={3} />
  {:else if companies.length === 0}
    <EmptyState
      icon={Building2}
      title="No companies yet"
      description="Create your first company to start organizing agents into org charts."
      actionLabel="Create company"
      onaction={openForm}
    />
  {:else}
    <div class="company-cards">
      {#each companies as c (c.id)}
        <article
          class="card"
          class:selected={selectedId === c.id}
          role="button"
          tabindex="0"
          onclick={() => selectCompany(c.id)}
          onkeydown={(e) => e.key === 'Enter' && selectCompany(c.id)}
        >
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

    {#if selectedCompany && !editing}
      <div class="detail-panel">
        <header class="detail-header">
          <h2>{selectedCompany.name}</h2>
          <div class="detail-actions">
            <button class="btn btn-ghost" onclick={startEdit} title="Edit"><Pencil size={14} /> Edit</button>
            <button class="btn btn-danger" onclick={confirmDelete} disabled={deleting} title="Delete"><Trash2 size={14} /> {deleting ? 'Deleting...' : 'Delete'}</button>
            <button class="btn btn-ghost" onclick={() => { selectedId = null; }} title="Close"><X size={14} /></button>
          </div>
        </header>
        <dl class="detail-fields">
          <dt>Mission</dt>
          <dd>{selectedCompany.mission || '—'}</dd>
          <dt>Budget Used</dt>
          <dd>${selectedCompany.budget_used.toFixed(2)}</dd>
          <dt>Budget Limit</dt>
          <dd>{selectedCompany.budget_limit != null ? `$${selectedCompany.budget_limit.toFixed(2)}` : 'No limit'}</dd>
          <dt>Created</dt>
          <dd>{new Date(selectedCompany.created_at).toLocaleDateString()}</dd>
        </dl>
      </div>
    {/if}

    {#if editing && selectedCompany}
      <div class="detail-panel">
        <h2>Edit {selectedCompany.name}</h2>
        {#if formError}
          <div class="message error">{formError}</div>
        {/if}
        <form class="company-form" onsubmit={(e) => { e.preventDefault(); saveEdit(); }}>
          <label>
            <span>Name</span>
            <input type="text" bind:value={editName} required />
          </label>
          <label>
            <span>Mission</span>
            <textarea bind:value={editMission} rows="3"></textarea>
          </label>
          <label>
            <span>Budget limit (USD)</span>
            <input type="number" min="0" step="0.01" bind:value={editBudget} />
          </label>
          <div class="form-actions">
            <button type="button" class="btn btn-ghost" onclick={() => { editing = false; }}>Cancel</button>
            <button type="submit" class="btn btn-primary" disabled={submitting}>{submitting ? 'Saving...' : 'Save'}</button>
          </div>
        </form>
      </div>
    {/if}
  {/if}

  {#if formOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="company-form-title" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeForm()}>
      <div class="modal" use:focusTrap>
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
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.75rem;
  }

  .page-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .page-desc {
    margin: 0.25rem 0 0 0;
    color: var(--muted);
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .company-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    transition: border-color var(--transition), box-shadow var(--transition);
  }

  .card:hover,
  .card:focus-visible {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-sm);
    cursor: pointer;
  }

  .card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-muted);
  }

  .card-title {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
  }

  .card-mission {
    margin: 0;
    font-size: 0.875rem;
    color: var(--muted);
    line-height: 1.5;
  }

  .card-budget {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    margin: 0.25rem 0 0 0;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border);
  }

  .card-budget .label {
    color: var(--muted);
  }

  .detail-panel {
    margin-top: 1.5rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 1.25rem;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .detail-header h2 {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
  }

  .detail-actions {
    display: flex;
    gap: 0.5rem;
  }

  .detail-fields {
    display: grid;
    grid-template-columns: 8rem 1fr;
    gap: 0.5rem 1rem;
    font-size: 0.875rem;
  }

  .detail-fields dt {
    color: var(--muted);
    font-weight: 500;
  }

  .detail-fields dd {
    margin: 0;
  }

  .btn-danger {
    background: transparent;
    border-color: transparent;
    color: #f87171;
  }

  .btn-danger:hover {
    background: rgba(248, 113, 113, 0.1);
  }

  .empty-state {
    padding: 3rem 1rem;
    text-align: center;
  }

  .muted {
    color: var(--muted);
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.875rem;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    transition: all var(--transition);
  }

  .btn:hover {
    background: var(--surface-hover);
    border-color: var(--border-hover);
  }

  .btn-primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #09090b;
    font-weight: 600;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
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
    border-radius: var(--radius-sm);
    margin-bottom: 1rem;
    background: var(--danger-muted);
    color: #fca5a5;
    border: 1px solid rgba(248, 113, 113, 0.3);
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }

  .modal {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 1.75rem;
    width: 100%;
    max-width: 28rem;
    max-height: 90vh;
    overflow: auto;
    box-shadow: var(--shadow-lg);
  }

  .modal h2 {
    margin: 0 0 1rem 0;
    font-size: 1.2rem;
    font-weight: 600;
    letter-spacing: -0.01em;
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
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.875rem;
    font-family: inherit;
    transition: border-color var(--transition);
  }

  .company-form input:focus,
  .company-form textarea:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-muted);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }
</style>

