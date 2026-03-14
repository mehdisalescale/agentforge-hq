<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import {
    listCompanies,
    getOrgChart,
    type Company,
    type CompanyOrgChart,
    type OrgChartNode,
    type Department,
  } from '$lib/api';

  setContext('pageTitle', 'Org Chart');

  let companies = $state<Company[]>([]);
  let selectedCompanyId = $state<string | null>(null);
  let chart = $state<CompanyOrgChart | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  function departmentName(departments: Department[], id: string | null | undefined): string {
    if (!id) return '—';
    const d = departments.find((x) => x.id === id);
    return d ? d.name : id.slice(0, 8) + '...';
  }

  async function loadInitial() {
    loading = true;
    error = null;
    try {
      companies = await listCompanies();
      if (companies.length > 0) {
        selectedCompanyId = companies[0].id;
        chart = await getOrgChart(selectedCompanyId);
      } else {
        chart = null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function changeCompany(id: string) {
    selectedCompanyId = id;
    loading = true;
    error = null;
    try {
      chart = await getOrgChart(id);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      chart = null;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadInitial();
  });
</script>

<svelte:head>
  <title>Org Chart - AgentForge</title>
</svelte:head>

<div class="org-page">
  <header class="page-header">
    <h1>Org chart</h1>
    {#if companies.length > 0}
      <label class="company-select">
        <span>Company</span>
        <select bind:value={selectedCompanyId} onchange={(e) => changeCompany((e.target as HTMLSelectElement).value)}>
          {#each companies as c}
            <option value={c.id}>{c.name}</option>
          {/each}
        </select>
      </label>
    {/if}
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading org chart...</p>
  {:else if !chart}
    <p class="muted">No org chart available yet. Create a company and positions to see the hierarchy.</p>
  {:else}
    <section class="org-header">
      <h2>{chart.company.name}</h2>
      {#if chart.company.mission}
        <p class="mission">{chart.company.mission}</p>
      {/if}
    </section>

    <section class="org-layout">
      <aside class="departments">
        <h3>Departments</h3>
        {#if chart.departments.length === 0}
          <p class="muted small">No departments yet.</p>
        {:else}
          <ul>
            {#each chart.departments as d}
              <li>{d.name}</li>
            {/each}
          </ul>
        {/if}
      </aside>

      <div class="tree">
        {#each chart.roots as node (node.position.id)}
          <div class="org-node">
            <div class="org-card">
              <div class="title">{node.position.title ?? node.position.role}</div>
              <div class="meta">
                <span class="meta-label">Department</span>
                <span>{departmentName(chart.departments, node.position.department_id ?? undefined)}</span>
              </div>
            </div>
            {#if node.children && node.children.length > 0}
              <div class="children">
                {#each node.children as child (child.position.id)}
                  <div class="org-node">
                    <div class="org-card">
                      <div class="title">{child.position.title ?? child.position.role}</div>
                      <div class="meta">
                        <span class="meta-label">Department</span>
                        <span>{departmentName(chart.departments, child.position.department_id ?? undefined)}</span>
                      </div>
                    </div>
                    {#if child.children && child.children.length > 0}
                      <div class="children">
                        {#each child.children as grand (grand.position.id)}
                          <div class="org-card org-node">
                            <div class="title">{grand.position.title ?? grand.position.role}</div>
                            <div class="meta">
                              <span class="meta-label">Department</span>
                              <span
                                >{departmentName(chart.departments, grand.position.department_id ?? undefined)}</span
                              >
                            </div>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .org-page {
    max-width: 60rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
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

  .org-header h2 {
    margin: 0 0 0.25rem 0;
  }

  .org-header .mission {
    margin: 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .org-layout {
    display: grid;
    grid-template-columns: 200px minmax(0, 1fr);
    gap: 1rem;
    margin-top: 1rem;
  }

  .departments {
    border-right: 1px solid var(--border);
    padding-right: 1rem;
  }

  .departments h3 {
    margin: 0 0 0.5rem 0;
    font-size: 0.95rem;
  }

  .departments ul {
    list-style: none;
    padding: 0;
    margin: 0;
    font-size: 0.9rem;
  }

  .departments li {
    padding: 0.2rem 0;
  }

  .tree {
    padding-left: 0.5rem;
  }

  .org-node {
    position: relative;
    padding-left: 1.25rem;
    margin: 0.5rem 0;
  }

  .org-node::before {
    content: '';
    position: absolute;
    left: 0.5rem;
    top: 0;
    bottom: 0;
    border-left: 1px solid var(--border);
  }

  .org-card {
    position: relative;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    min-width: 10rem;
  }

  .org-card::before {
    content: '';
    position: absolute;
    left: -0.75rem;
    top: 50%;
    width: 0.75rem;
    border-top: 1px solid var(--border);
  }

  .org-card .title {
    font-size: 0.9rem;
    font-weight: 600;
    margin-bottom: 0.15rem;
  }

  .org-card .meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--muted);
  }

  .meta-label {
    margin-right: 0.25rem;
  }

  .children {
    margin-left: 1.25rem;
    margin-top: 0.25rem;
  }

  .muted {
    color: var(--muted);
  }

  .small {
    font-size: 0.8rem;
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

