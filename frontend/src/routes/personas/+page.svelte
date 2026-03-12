<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { listPersonas, type Persona } from '$lib/api';

  setContext('pageTitle', 'Personas');

  let personas = $state<Persona[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let divisionFilter = $state<string>('');
  let query = $state<string>('');

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

  onMount(() => {
    load();
  });
</script>

<svelte:head>
  <title>Personas - Claude Forge</title>
</svelte:head>

<div class="personas-page">
  <header class="page-header">
    <h1>Persona catalog</h1>
    <div class="filters">
      <label>
        <span>Division</span>
        <input
          type="text"
          bind:value={divisionFilter}
          placeholder="e.g. engineering, product, marketing"
          onkeydown={(e) => e.key === 'Enter' && load()}
        />
      </label>
      <label>
        <span>Search</span>
        <input
          type="search"
          bind:value={query}
          placeholder="Name, summary, tags…"
          onkeydown={(e) => e.key === 'Enter' && load()}
        />
      </label>
      <div class="filter-actions">
        <button class="btn" type="button" onclick={load}>Apply</button>
        <button class="btn btn-ghost" type="button" onclick={clearFilters}>Reset</button>
      </div>
    </div>
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading personas…</p>
  {:else if personas.length === 0}
    <div class="empty-state">
      <p class="muted">
        No personas found. Once the persona catalog is imported, they will appear here for browsing
        and assignment.
      </p>
    </div>
  {:else}
    <section class="grid">
      {#each personas as p (p.id)}
        <article class="card">
          <header class="card-header">
            <h2 class="card-title">{p.name}</h2>
            <p class="card-division">{p.division_slug}</p>
          </header>
          <p class="card-summary">{p.short_description}</p>
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
</div>

<style>
  .personas-page {
    max-width: 64rem;
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: flex-end;
  }

  .filters label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 12rem;
  }

  .filters span {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .filters input {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.9rem;
  }

  .filter-actions {
    display: flex;
    gap: 0.5rem;
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
    border-radius: 8px;
    border: 1px solid var(--border);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
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
    background: rgba(148, 163, 184, 0.15);
    color: #e5e7eb;
    font-size: 0.75rem;
  }

  .tags .more {
    background: transparent;
    border: 1px dashed rgba(148, 163, 184, 0.5);
  }

  .message.error {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }
</style>

