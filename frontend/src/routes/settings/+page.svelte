<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { getSettings, type RuntimeSettings } from '$lib/api';

  setContext('pageTitle', 'Settings');

  const API_BASE = typeof import.meta !== 'undefined' && import.meta.env?.VITE_API_URL != null
    ? (import.meta.env.VITE_API_URL as string).replace(/\/$/, '')
    : '';

  interface HealthData {
    status: string;
    version: string;
    uptime_secs: number;
  }

  let health = $state<HealthData | null>(null);
  let settings = $state<RuntimeSettings | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const envVarMeta: { key: keyof RuntimeSettings; env: string; description: string }[] = [
    { key: 'db_path', env: 'FORGE_DB_PATH', description: 'SQLite database file path' },
    { key: 'port', env: 'FORGE_PORT', description: 'Server listen port' },
    { key: 'host', env: 'FORGE_HOST', description: 'Server bind address' },
    { key: 'cli_command', env: 'FORGE_CLI_COMMAND', description: 'CLI executable to spawn for agent processes' },
    { key: 'cors_origin', env: 'FORGE_CORS_ORIGIN', description: 'CORS allowed origin header' },
    { key: 'rate_limit_max', env: 'FORGE_RATE_LIMIT_MAX', description: 'Rate limiter max tokens (requests per window)' },
    { key: 'rate_limit_refill_ms', env: 'FORGE_RATE_LIMIT_REFILL_MS', description: 'Rate limiter token refill interval in milliseconds' },
    { key: 'budget_warn', env: 'FORGE_BUDGET_WARN', description: 'Cost warning threshold in USD' },
    { key: 'budget_limit', env: 'FORGE_BUDGET_LIMIT', description: 'Hard budget limit in USD (blocks further runs)' },
  ];

  function formatUptime(secs: number): string {
    if (secs < 60) return `${secs}s`;
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    if (m < 60) return `${m}m ${s}s`;
    const h = Math.floor(m / 60);
    const rm = m % 60;
    if (h < 24) return `${h}h ${rm}m`;
    const d = Math.floor(h / 24);
    const rh = h % 24;
    return `${d}d ${rh}h ${rm}m`;
  }

  async function fetchAll() {
    loading = true;
    error = null;
    try {
      const [healthRes, settingsRes] = await Promise.allSettled([
        fetch(`${API_BASE}/api/v1/health`).then(async (res) => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`);
          return res.json() as Promise<HealthData>;
        }),
        getSettings(),
      ]);
      health = healthRes.status === 'fulfilled' ? healthRes.value : null;
      settings = settingsRes.status === 'fulfilled' ? settingsRes.value : null;
      if (healthRes.status === 'rejected') {
        error = healthRes.reason instanceof Error ? healthRes.reason.message : String(healthRes.reason);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      health = null;
      settings = null;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchAll();
  });
</script>

<svelte:head>
  <title>Settings · AgentForge</title>
</svelte:head>

<div class="settings-page">
  <header class="page-header">
    <h1>Settings</h1>
    <button class="btn" onclick={fetchAll} disabled={loading}>
      {loading ? 'Reloading...' : 'Reload'}
    </button>
  </header>

  <!-- System Info -->
  <section class="section">
    <h2 class="section-title">System</h2>
    <div class="info-grid">
      <div class="info-card">
        <span class="info-label">Version</span>
        <span class="info-value">{health ? health.version : '0.4.0-dev'}</span>
      </div>
      <div class="info-card">
        <span class="info-label">Status</span>
        {#if loading}
          <span class="info-value muted">Checking...</span>
        {:else if error}
          <span class="info-value status-bad">Offline</span>
        {:else}
          <span class="info-value status-ok">{health?.status ?? 'unknown'}</span>
        {/if}
      </div>
      <div class="info-card">
        <span class="info-label">Uptime</span>
        <span class="info-value">{health ? formatUptime(health.uptime_secs) : '--'}</span>
      </div>
      <div class="info-card">
        <span class="info-label">Database</span>
        <span class="info-value mono">{settings?.db_path ?? '~/.agentforge/forge.db'}</span>
      </div>
    </div>
  </section>

  {#if error}
    <div class="message error" role="alert">
      Could not reach the backend: {error}
    </div>
  {/if}

  <!-- Environment Variables -->
  <section class="section">
    <h2 class="section-title">Configuration</h2>
    <p class="section-desc">Environment variables control Forge behavior. Set these before starting the server.</p>
    <div class="env-table-wrapper">
      <table class="env-table">
        <thead>
          <tr>
            <th>Variable</th>
            <th>Current Value</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          {#each envVarMeta as ev}
            <tr>
              <td class="mono env-name">{ev.env}</td>
              <td class="mono env-default">{settings ? (settings[ev.key] ?? '(not set)') : '--'}</td>
              <td class="env-desc">{ev.description}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <!-- About -->
  <section class="section">
    <h2 class="section-title">About</h2>
    <p class="about-text">
      AgentForge is a self-improving AI workforce platform. Browse 100+ agent personas, hire them into org charts, and let them execute real work with budgets and governance. Single binary, zero deps.
    </p>
  </section>
</div>

<style>
  .settings-page {
    max-width: 56rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .page-header h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .section {
    margin-bottom: 2rem;
  }

  .section-title {
    font-size: 1.1rem;
    margin: 0 0 0.75rem 0;
    font-weight: 600;
  }

  .section-desc {
    font-size: 0.85rem;
    color: var(--muted);
    margin: 0 0 1rem 0;
  }

  /* Info grid */
  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 0.75rem;
  }

  .info-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0.85rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .info-label {
    font-size: 0.75rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .info-value {
    font-size: 1rem;
    font-weight: 600;
  }

  .status-ok {
    color: #86efac;
  }

  .status-bad {
    color: #fca5a5;
  }

  .mono {
    font-family: ui-monospace, 'Cascadia Code', 'Source Code Pro', monospace;
    font-size: 0.85rem;
  }

  /* Env table */
  .env-table-wrapper {
    overflow-x: auto;
  }

  .env-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  .env-table th {
    text-align: left;
    padding: 0.6rem 0.75rem;
    border-bottom: 2px solid var(--border);
    color: var(--muted);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
  }

  .env-table td {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid var(--border);
    vertical-align: top;
  }

  .env-table tbody tr:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .env-name {
    color: var(--accent);
    font-weight: 500;
    white-space: nowrap;
  }

  .env-default {
    color: var(--muted);
    white-space: nowrap;
  }

  .env-desc {
    color: var(--text);
  }

  /* About */
  .about-text {
    font-size: 0.9rem;
    color: var(--muted);
    line-height: 1.5;
    margin: 0;
  }

  /* Shared */
  .message.error {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1.5rem;
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
    font-size: 0.9rem;
  }

  .muted {
    color: var(--muted);
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
  }

  .btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.06);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
