<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import {
    listPersonas,
    listCompanies,
    listDepartmentsByCompany,
    listOrgPositionsByCompany,
    hirePersona,
    type Persona,
    type Company,
    type Department,
    type OrgPosition,
  } from '$lib/api';
  import Markdown from '$lib/components/Markdown.svelte';
  import { Search, Filter, UserPlus, Users as UsersIcon, X } from 'lucide-svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import ErrorMessage from '$lib/components/ErrorMessage.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { focusTrap } from '$lib/actions/focusTrap';

  setContext('pageTitle', 'Personas');

  let personas = $state<Persona[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let divisionFilter = $state<string>('');
  let query = $state<string>('');

  // Hire modal state
  let hireOpen = $state(false);
  let hireTarget: Persona | null = $state(null);
  let companies = $state<Company[]>([]);
  let departments = $state<Department[]>([]);
  let positions = $state<OrgPosition[]>([]);
  let hireCompanyId = $state<string>('');
  let hireDepartmentId = $state<string>('');
  let hireReportsTo = $state<string>('');
  let hireTitle = $state<string>('');
  let hireSubmitting = $state(false);
  let hireError = $state<string | null>(null);
  let hireSuccess = $state<string | null>(null);

  async function load() {
    loading = true;
    error = null;
    try {
      const opts: { division_slug?: string; q?: string } = {};
      if (divisionFilter.trim()) opts.division_slug = divisionFilter.trim();
      if (query.trim()) opts.q = query.trim();
      personas = await listPersonas(opts);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function clearFilters() {
    divisionFilter = '';
    query = '';
    load();
  }

  async function ensureOrgData() {
    if (companies.length === 0) {
      companies = await listCompanies();
    }
    if (hireCompanyId && hireCompanyId.trim()) {
      departments = await listDepartmentsByCompany(hireCompanyId);
      positions = await listOrgPositionsByCompany(hireCompanyId);
    } else {
      departments = [];
      positions = [];
    }
  }

  async function openHireModal(p: Persona) {
    hireTarget = p;
    hireCompanyId = '';
    hireDepartmentId = '';
    hireReportsTo = '';
    hireTitle = p.name;
    hireError = null;
    hireSuccess = null;
    await ensureOrgData();
    hireOpen = true;
  }

  function closeHireModal() {
    hireOpen = false;
    hireTarget = null;
    hireError = null;
    hireSuccess = null;
  }

  async function onCompanyChange(id: string) {
    hireCompanyId = id;
    hireDepartmentId = '';
    hireReportsTo = '';
    await ensureOrgData();
  }

  async function submitHire() {
    if (!hireTarget) return;
    if (!hireCompanyId.trim()) {
      hireError = 'Company is required to hire a persona.';
      return;
    }
    hireSubmitting = true;
    hireError = null;
    hireSuccess = null;
    try {
      await hirePersona(hireTarget.id, {
        company_id: hireCompanyId,
        department_id: hireDepartmentId || undefined,
        reports_to: hireReportsTo || undefined,
        title_override: hireTitle.trim() || undefined,
      });
      hireSuccess = 'Persona hired into the org chart. Check the Org Chart view to see the new position.';
    } catch (e) {
      hireError = e instanceof Error ? e.message : String(e);
    } finally {
      hireSubmitting = false;
    }
  }

  onMount(() => {
    load();
  });
</script>

<svelte:head>
  <title>Personas - AgentForge</title>
</svelte:head>

<div class="personas-page" aria-busy={loading}>
  <header class="page-header">
    <h1>Persona Catalog</h1>
    <p class="page-desc">Browse 100+ pre-built AI agent personas. Filter by division or search to find the right fit, then hire them into your company.</p>
    <div class="filters">
      <label>
        <span><Filter size={12} /> Division</span>
        <input
          type="text"
          bind:value={divisionFilter}
          placeholder="e.g. engineering, product, marketing"
          onkeydown={(e) => e.key === 'Enter' && load()}
        />
      </label>
      <label>
        <span><Search size={12} /> Search</span>
        <input
          type="search"
          bind:value={query}
          placeholder="Name, summary, tags…"
          onkeydown={(e) => e.key === 'Enter' && load()}
        />
      </label>
      <div class="filter-actions">
        <button class="btn" type="button" onclick={load}>Apply</button>
        <button class="btn btn-ghost" type="button" onclick={clearFilters}><X size={14} /> Reset</button>
      </div>
    </div>
  </header>

  {#if error}
    <ErrorMessage message={error} onretry={load} />
  {/if}

  {#if loading}
    <Skeleton type="card" lines={4} />
  {:else if personas.length === 0}
    <EmptyState
      icon={UsersIcon}
      title="No personas found"
      description="Once the persona catalog is imported, they will appear here for browsing and assignment."
    />
  {:else}
    <section class="grid">
      {#each personas as p (p.id)}
        <article class="card">
          <header class="card-header">
            <div>
              <h2 class="card-title">{p.name}</h2>
              <p class="card-division">{p.division_slug}</p>
            </div>
            <button class="btn btn-small" type="button" onclick={() => openHireModal(p)}><UserPlus size={14} /> Hire</button>
          </header>
          <div class="card-summary"><Markdown content={p.short_description} /></div>
          {#if p.tags?.length}
            <ul class="tags">
              {#each p.tags.slice(0, 6) as tag}
                <li>{tag}</li>
              {/each}
              {#if p.tags.length > 6}
                <li class="more">+{p.tags.length - 6} more</li>
              {/if}
            </ul>
          {/if}
        </article>
      {/each}
    </section>
  {/if}

  {#if hireOpen && hireTarget}
    <div class="modal-backdrop" role="dialog" aria-modal="true" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeHireModal()}>
      <div class="modal" use:focusTrap>
        <header class="modal-header">
          <h2>Hire persona</h2>
          <button class="btn btn-ghost btn-small" type="button" onclick={closeHireModal}>Close</button>
        </header>
        <p class="muted small">Create an agent and org position for <strong>{hireTarget.name}</strong>.</p>

        {#if hireError}
          <div class="message error">{hireError}</div>
        {/if}
        {#if hireSuccess}
          <div class="message success">{hireSuccess}</div>
        {/if}

        <form
          class="hire-form"
          onsubmit={(e) => {
            e.preventDefault();
            submitHire();
          }}
        >
          <label>
            <span>Company</span>
            <select bind:value={hireCompanyId} onchange={(e) => onCompanyChange((e.target as HTMLSelectElement).value)}>
              <option value="">Select company…</option>
              {#each companies as c}
                <option value={c.id}>{c.name}</option>
              {/each}
            </select>
          </label>

          <label>
            <span>Department (optional)</span>
            <select bind:value={hireDepartmentId} disabled={!hireCompanyId}>
              <option value="">No specific department</option>
              {#each departments as d}
                <option value={d.id}>{d.name}</option>
              {/each}
            </select>
          </label>

          <label>
            <span>Reports to (optional)</span>
            <select bind:value={hireReportsTo} disabled={!hireCompanyId}>
              <option value="">Top-level (no manager)</option>
              {#each positions as pos}
                <option value={pos.id}>{pos.title ?? pos.role}</option>
              {/each}
            </select>
          </label>

          <label>
            <span>Title</span>
            <input type="text" bind:value={hireTitle} />
          </label>

          <div class="form-actions">
            <button class="btn btn-ghost" type="button" onclick={closeHireModal}>Cancel</button>
            <button class="btn" type="submit" disabled={hireSubmitting}>
              {hireSubmitting ? 'Hiring…' : 'Hire persona'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .personas-page {
    max-width: 64rem;
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1.75rem;
  }

  .page-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .page-desc {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
    line-height: 1.5;
    max-width: 40rem;
  }

  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: flex-end;
    margin-top: 0.25rem;
  }

  .filters label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    min-width: 12rem;
  }

  .filters span {
    font-size: 0.8rem;
    color: var(--muted);
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .filters input {
    padding: 0.45rem 0.7rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.875rem;
    font-family: inherit;
    transition: border-color var(--transition);
  }

  .filters input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-muted);
  }

  .filter-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn {
    padding: 0.45rem 0.9rem;
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

  .btn-ghost {
    background: transparent;
    border-color: transparent;
  }

  .btn-ghost:hover {
    background: var(--surface-hover);
  }

  .btn-small {
    padding: 0.3rem 0.65rem;
    font-size: 0.8rem;
  }

  .muted {
    color: var(--muted);
  }

  .empty-state {
    padding: 2rem 1rem;
    text-align: center;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }

  .card {
    background: var(--surface);
    border-radius: var(--radius);
    border: 1px solid var(--border);
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    transition: border-color var(--transition), box-shadow var(--transition);
  }

  .card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-sm);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 0.75rem;
  }

  .card-title {
    margin: 0;
    font-size: 1rem;
  }

  .card-division {
    margin: 0;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
  }

  .card-summary {
    margin: 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin: 0.25rem 0 0 0;
    padding: 0;
    list-style: none;
  }

  .tags li {
    padding: 0.15rem 0.5rem;
    border-radius: 999px;
    background: var(--accent-muted);
    color: var(--text-secondary);
    font-size: 0.72rem;
    font-weight: 500;
  }

  .tags .more {
    background: transparent;
    border: 1px dashed var(--border-hover);
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

  .message.success {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(34, 197, 94, 0.15);
    color: #bbf7d0;
    border: 1px solid rgba(34, 197, 94, 0.4);
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
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    padding: 1.5rem;
    width: 100%;
    max-width: 30rem;
    max-height: 90vh;
    overflow: auto;
    box-shadow: var(--shadow-lg);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .hire-form {
    margin-top: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .hire-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .hire-form span {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .hire-form select,
  .hire-form input {
    padding: 0.45rem 0.7rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.875rem;
    font-family: inherit;
    transition: border-color var(--transition);
  }

  .hire-form select:focus,
  .hire-form input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-muted);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .small {
    font-size: 0.85rem;
  }
</style>

