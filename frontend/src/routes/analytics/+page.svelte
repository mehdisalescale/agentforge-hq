<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getUsageAnalytics,
    type UsageReport,
  } from '$lib/api';

  let report = $state<UsageReport | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let startDate = $state('');
  let endDate = $state('');

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
      report = await getUsageAnalytics(start, end);
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

  /** Simple bar chart using CSS. */
  function barWidth(cost: number, max: number): string {
    if (max <= 0) return '0%';
    return Math.min(100, (cost / max) * 100).toFixed(1) + '%';
  }

  let maxDailyCost = $derived(
    report?.daily_costs.reduce((m, d) => Math.max(m, d.cost), 0) ?? 0
  );

  onMount(() => {
    defaultDates();
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

  <div class="date-picker">
    <label>
      <span>From</span>
      <input type="date" bind:value={startDate} />
    </label>
    <label>
      <span>To</span>
      <input type="date" bind:value={endDate} />
    </label>
    <button class="btn btn-primary" onclick={loadReport}>Apply</button>
  </div>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading analytics...</p>
  {:else if report}
    <div class="summary-cards">
      <div class="card summary-card">
        <span class="summary-label">Total Cost</span>
        <span class="summary-value">{formatCost(report.total_cost)}</span>
      </div>
      <div class="card summary-card">
        <span class="summary-label">Sessions</span>
        <span class="summary-value">{report.stats.total}</span>
      </div>
      <div class="card summary-card">
        <span class="summary-label">Completed</span>
        <span class="summary-value success">{report.stats.completed}</span>
      </div>
      <div class="card summary-card">
        <span class="summary-label">Failed</span>
        <span class="summary-value danger">{report.stats.failed}</span>
      </div>
      <div class="card summary-card">
        <span class="summary-label">Avg Cost</span>
        <span class="summary-value">{formatCost(report.stats.avg_cost)}</span>
      </div>
      <div class="card summary-card">
        <span class="summary-label">P90 Cost</span>
        <span class="summary-value">{formatCost(report.stats.p90_cost)}</span>
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
        <div class="bar-chart">
          {#each report.daily_costs as day}
            <div class="bar-row">
              <span class="bar-label">{day.date}</span>
              <div class="bar-track">
                <div class="bar-fill" style="width: {barWidth(day.cost, maxDailyCost)}"></div>
              </div>
              <span class="bar-value">{formatCost(day.cost)}</span>
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
        <table class="data-table">
          <thead>
            <tr><th>Agent ID</th><th>Sessions</th><th>Total Cost</th></tr>
          </thead>
          <tbody>
            {#each report.agent_breakdown as ab}
              <tr>
                <td><code>{ab.agent_id.slice(0, 12)}...</code></td>
                <td>{ab.session_count}</td>
                <td>{formatCost(ab.total_cost)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  {/if}
</div>

<style>
  .analytics-page { max-width: 56rem; }
  .date-picker { display: flex; align-items: flex-end; gap: 1rem; margin-bottom: 1.5rem; }
  .date-picker label { display: flex; flex-direction: column; gap: 0.25rem; }
  .date-picker label span { font-size: 0.8rem; color: var(--muted); }
  .date-picker input[type="date"] {
    padding: 0.4rem 0.6rem; border-radius: 6px; border: 1px solid var(--border);
    background: var(--bg); color: var(--text); font-size: 0.9rem;
  }
  .summary-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr)); gap: 0.75rem; margin-bottom: 2rem; }
  .summary-card { padding: 1rem; text-align: center; }
  .summary-label { display: block; font-size: 0.75rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.35rem; }
  .summary-value { display: block; font-size: 1.5rem; font-weight: 700; }
  .summary-value.success { color: #86efac; }
  .summary-value.danger { color: #f87171; }
  .summary-value.accent { color: #a78bfa; }
  .section { margin-bottom: 2rem; }
  .section h2 { font-size: 1.1rem; margin: 0 0 1rem 0; }
  .bar-chart { display: flex; flex-direction: column; gap: 0.4rem; }
  .bar-row { display: grid; grid-template-columns: 6rem 1fr 5rem; gap: 0.5rem; align-items: center; }
  .bar-label { font-size: 0.8rem; color: var(--muted); text-align: right; }
  .bar-track { background: var(--surface); border-radius: 4px; height: 1.2rem; overflow: hidden; }
  .bar-fill { background: var(--accent); height: 100%; border-radius: 4px; min-width: 2px; transition: width 0.3s; }
  .bar-value { font-size: 0.8rem; color: var(--text); }
  .data-table { width: 100%; border-collapse: collapse; }
  .data-table th { text-align: left; font-size: 0.8rem; color: var(--muted); padding: 0.5rem; border-bottom: 1px solid var(--border); }
  .data-table td { padding: 0.5rem; font-size: 0.9rem; border-bottom: 1px solid var(--border); }
  .data-table code { font-size: 0.8rem; background: var(--bg); padding: 0.1rem 0.3rem; border-radius: 3px; }
</style>
