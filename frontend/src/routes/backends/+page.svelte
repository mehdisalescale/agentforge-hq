<script lang="ts">
  import { onMount } from 'svelte';

  interface BackendInfo {
    name: string;
    capabilities: {
      supports_streaming: boolean;
      supports_tools: boolean;
      supported_models: string[];
    };
  }

  interface HealthReport {
    name: string;
    status: string;
    message: string | null;
  }

  let backends = $state<BackendInfo[]>([]);
  let health = $state<Map<string, HealthReport>>(new Map());
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function fetchBackends() {
    try {
      const [backendsRes, healthRes] = await Promise.all([
        fetch('/api/v1/backends'),
        fetch('/api/v1/backends/health')
      ]);
      if (backendsRes.ok) backends = await backendsRes.json();
      if (healthRes.ok) {
        const reports: HealthReport[] = await healthRes.json();
        const map = new Map<string, HealthReport>();
        for (const r of reports) map.set(r.name, r);
        health = map;
      }
      error = null;
    } catch (e) {
      error = 'Failed to fetch backend status';
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchBackends();
    const interval = setInterval(fetchBackends, 30000);
    return () => clearInterval(interval);
  });

  function statusColor(status: string): string {
    switch (status) {
      case 'healthy': return 'var(--success, #22c55e)';
      case 'degraded': return 'var(--warning, #eab308)';
      case 'unavailable': return 'var(--danger, #ef4444)';
      default: return 'var(--muted, #6b7280)';
    }
  }
</script>

<svelte:head>
  <title>Backends - AgentForge</title>
</svelte:head>

<div class="page">
  <header class="page-header">
    <h1>Execution Backends</h1>
    <p class="subtitle">Registered process backends and their health status</p>
  </header>

  {#if loading}
    <p class="loading">Loading backends...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if backends.length === 0}
    <p class="empty">No backends registered.</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Backend</th>
            <th>Status</th>
            <th>Streaming</th>
            <th>Tools</th>
            <th>Models</th>
            <th>Message</th>
          </tr>
        </thead>
        <tbody>
          {#each backends as b}
            {@const h = health.get(b.name)}
            <tr>
              <td class="name">{b.name}</td>
              <td>
                <span class="badge" style="background: {statusColor(h?.status ?? 'unknown')}">
                  {h?.status ?? 'unknown'}
                </span>
              </td>
              <td>{b.capabilities.supports_streaming ? 'Yes' : 'No'}</td>
              <td>{b.capabilities.supports_tools ? 'Yes' : 'No'}</td>
              <td class="models">{b.capabilities.supported_models.join(', ')}</td>
              <td class="message">{h?.message ?? '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <p class="refresh-note">Auto-refreshes every 30 seconds</p>
  {/if}
</div>

<style>
  .page {
    padding: 1.5rem;
    max-width: 960px;
  }
  .page-header {
    margin-bottom: 1.5rem;
  }
  .page-header h1 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0;
  }
  .subtitle {
    color: var(--muted, #9ca3af);
    font-size: 0.85rem;
    margin: 0.25rem 0 0;
  }
  .loading, .error, .empty {
    color: var(--muted, #9ca3af);
    font-size: 0.9rem;
  }
  .error {
    color: var(--danger, #ef4444);
  }
  .table-wrap {
    overflow-x: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }
  th {
    text-align: left;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border, #374151);
    color: var(--muted, #9ca3af);
    font-weight: 500;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  td {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid var(--border, #1f2937);
  }
  .name {
    font-weight: 600;
  }
  .badge {
    display: inline-block;
    padding: 0.15rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
    color: #fff;
  }
  .models {
    font-size: 0.8rem;
    color: var(--muted, #9ca3af);
  }
  .message {
    font-size: 0.8rem;
    color: var(--muted, #9ca3af);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .refresh-note {
    margin-top: 1rem;
    font-size: 0.75rem;
    color: var(--muted, #6b7280);
  }
</style>
