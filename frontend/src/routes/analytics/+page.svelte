<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getUsageAnalytics,
    listAgents,
    listCompanies,
    type UsageReport,
    type Agent,
    type Company,
  } from '$lib/api';

  let report = $state<UsageReport | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let startDate = $state('');
  let endDate = $state('');
  let selectedCompany = $state('');

  let companies = $state<Company[]>([]);
  let agentNames = $state<Record<string, string>>({});

  function defaultDates() {
    const now = new Date();
    const thirtyAgo = new Date(now);
    thirtyAgo.setDate(thirtyAgo.getDate() - 30);
    startDate = thirtyAgo.toISOString().slice(0, 10);
    endDate = now.toISOString().slice(0, 10);
  }

  async function loadReport() {
    loading = true;
    error = null;
    try {
      const start = startDate ? `${startDate}T00:00:00` : undefined;
      const end = endDate ? `${endDate}T23:59:59` : undefined;
      report = await getUsageAnalytics(start, end, selectedCompany || undefined);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      report = null;
    } finally {
      loading = false;
    }
  }

  function formatCost(n: number): string {
    return '$' + n.toFixed(4);
  }

  function barWidth(cost: number, max: number): string {
    if (max <= 0) return '0%';
    return Math.min(100, (cost / max) * 100).toFixed(1) + '%';
  }

  let maxDailyCost = $derived(
    report?.daily_costs.reduce((m, d) => Math.max(m, d.cost), 0) ?? 0
  );

  let maxAgentCost = $derived(
    report?.agent_breakdown.reduce((m, a) => Math.max(m, a.total_cost), 0) ?? 0
  );

  let successRate = $derived(
    report && report.stats.total > 0
      ? ((report.stats.completed / report.stats.total) * 100).toFixed(1)
      : '0.0'
  );

  onMount(async () => {
    defaultDates();
    try {
      const [companyList, agentList] = await Promise.all([
        listCompanies(),
        listAgents(),
      ]);
      companies = companyList;
      const names: Record<string, string> = {};
      for (const a of agentList) {
        names[a.id] = a.name;
      }
      agentNames = names;
    } catch {
      // non-critical — proceed without names/companies
    }
    loadReport();
  });
</script>

<svelte:head>
  <title>Analytics &middot; AgentForge</title>
</svelte:head>

<div class="analytics-page">
  <header class="page-header">
    <h1>Usage Analytics</h1>
  </header>

  <div class="filters">
    <label>
      <span>From</span>
      <input type="date" bind:value={startDate} />
    </label>
    <label>
      <span>To</span>
      <input type="date" bind:value={endDate} />
    </label>
    <label>
      <span>Company</span>
      <select bind:value={selectedCompany} onchange={loadReport}>
        <option value="">All Companies</option>
        {#each companies as company}
          <option value={company.id}>{company.name}</option>
        {/each}
      </select>
    </label>
    <button class="btn btn-primary" onclick={loadReport}>Apply</button>
  </div>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading analytics...</p>
  {:else if report}
    {#if report.stats.total === 0}
      <div class="empty-state">
        <p>No sessions recorded yet. Run an agent to see analytics here.</p>
      </div>
    {:else}
      <div class="summary-cards">
        <div class="card summary-card">
          <span class="summary-label">Total Cost</span>
          <span class="summary-value">{formatCost(report.total_cost)}</span>
        </div>
        <div class="card summary-card">
          <span class="summary-label">Sessions</span>
          <span class="summary-value">{report.stats.total}</span>
          <span class="summary-sub">{report.stats.completed} completed / {report.stats.failed} failed</span>
        </div>
        <div class="card summary-card">
          <span class="summary-label">Success Rate</span>
          <span class="summary-value success">{successRate}%</span>
        </div>
        <div class="card summary-card">
          <span class="summary-label">Projected Monthly</span>
          <span class="summary-value accent">{formatCost(report.projected_monthly_cost)}</span>
        </div>
      </div>

      <section class="section">
        <h2>Daily Costs</h2>
        {#if report.daily_costs.length === 0}
          <p class="muted">No cost data for this period.</p>
        {:else}
          <div class="daily-chart">
            {#each report.daily_costs as day}
              <div class="day-bar">
                <div class="bar-fill-v" style="height: {barWidth(day.cost, maxDailyCost)}"></div>
                <span class="day-cost">{formatCost(day.cost)}</span>
                <span class="day-label">{day.date.slice(5)}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <section class="section">
        <h2>Agent Breakdown</h2>
        {#if report.agent_breakdown.length === 0}
          <p class="muted">No agent data for this period.</p>
        {:else}
          <div class="agent-list">
            {#each report.agent_breakdown as ab}
              <div class="agent-row">
                <span class="agent-name">{agentNames[ab.agent_id] ?? ab.agent_id.slice(0, 8)}</span>
                <span class="agent-sessions">{ab.session_count} sessions</span>
                <div class="agent-bar-track">
                  <div class="agent-bar-fill" style="width: {barWidth(ab.total_cost, maxAgentCost)}"></div>
                </div>
                <span class="agent-cost">{formatCost(ab.total_cost)}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    {/if}
  {/if}
</div>

<style>
  .analytics-page { max-width: 56rem; }
  .filters { display: flex; align-items: flex-end; gap: 1rem; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .filters label { display: flex; flex-direction: column; gap: 0.25rem; }
  .filters label span { font-size: 0.8rem; color: var(--muted); }
  .filters input[type="date"],
  .filters select {
    padding: 0.4rem 0.6rem; border-radius: 6px; border: 1px solid var(--border);
    background: var(--bg); color: var(--text); font-size: 0.9rem;
  }
  .filters select { min-width: 10rem; }

  .empty-state {
    text-align: center; padding: 3rem 1rem; color: var(--muted);
    border: 1px dashed var(--border); border-radius: 8px; margin: 2rem 0;
  }

  .summary-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr)); gap: 0.75rem; margin-bottom: 2rem; }
  .summary-card { padding: 1rem; text-align: center; }
  .summary-label { display: block; font-size: 0.75rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.35rem; }
  .summary-value { display: block; font-size: 1.5rem; font-weight: 700; }
  .summary-value.success { color: #86efac; }
  .summary-value.accent { color: #a78bfa; }
  .summary-sub { display: block; font-size: 0.7rem; color: var(--muted); margin-top: 0.2rem; }

  .section { margin-bottom: 2rem; }
  .section h2 { font-size: 1.1rem; margin: 0 0 1rem 0; }

  /* Vertical bar chart */
  .daily-chart {
    display: flex; gap: 2px; align-items: flex-end;
    height: 10rem; padding: 0.5rem 0;
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
  }
  .day-bar {
    flex: 1; min-width: 2rem; display: flex; flex-direction: column;
    align-items: center; justify-content: flex-end; height: 100%;
    position: relative;
  }
  .bar-fill-v {
    width: 80%; background: var(--accent); border-radius: 3px 3px 0 0;
    min-height: 2px; transition: height 0.3s;
  }
  .day-cost {
    font-size: 0.6rem; color: var(--muted); margin-bottom: 0.15rem;
    white-space: nowrap;
  }
  .day-label { font-size: 0.65rem; color: var(--muted); margin-top: 0.3rem; }

  /* Agent breakdown */
  .agent-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .agent-row {
    display: grid; grid-template-columns: 10rem 6rem 1fr 5rem;
    gap: 0.5rem; align-items: center;
  }
  .agent-name { font-size: 0.9rem; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .agent-sessions { font-size: 0.75rem; color: var(--muted); }
  .agent-bar-track { background: var(--surface); border-radius: 4px; height: 1.2rem; overflow: hidden; }
  .agent-bar-fill { background: var(--accent); height: 100%; border-radius: 4px; min-width: 2px; transition: width 0.3s; }
  .agent-cost { font-size: 0.85rem; text-align: right; }
</style>
