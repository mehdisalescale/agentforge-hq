<script lang="ts">
  import {
    listSchedules,
    createSchedule,
    updateSchedule,
    deleteSchedule,
    listAgents,
    type Schedule,
    type NewSchedule,
    type UpdateSchedule,
    type Agent,
  } from '$lib/api';

  const CRON_PRESETS: { label: string; value: string }[] = [
    { label: 'Every minute', value: '0 * * * * * *' },
    { label: 'Every hour', value: '0 0 * * * * *' },
    { label: 'Daily at 9 AM', value: '0 0 9 * * * *' },
    { label: 'Weekly Mon 9 AM', value: '0 0 9 * * 1 *' },
    { label: 'Custom', value: '' },
  ];

  let schedules = $state<Schedule[]>([]);
  let agents = $state<Agent[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let formOpen = $state<'create' | 'edit' | null>(null);
  let editId = $state<string | null>(null);
  let deleteConfirmId = $state<string | null>(null);
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  let formName = $state('');
  let formCron = $state('0 0 9 * * * *');
  let formAgentId = $state('');
  let formPrompt = $state('');
  let formDirectory = $state('.');

  async function load() {
    loading = true;
    error = null;
    try {
      [schedules, agents] = await Promise.all([listSchedules(), listAgents()]);
      if (agents.length > 0 && !formAgentId) formAgentId = agents[0].id;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function openCreate() {
    editId = null;
    formName = '';
    formCron = '0 0 9 * * * *';
    formAgentId = agents.length > 0 ? agents[0].id : '';
    formPrompt = '';
    formDirectory = '.';
    formError = null;
    formOpen = 'create';
  }

  function openEdit(s: Schedule) {
    editId = s.id;
    formName = s.name;
    formCron = s.cron_expr;
    formAgentId = s.agent_id;
    formPrompt = s.prompt;
    formDirectory = s.directory;
    formError = null;
    formOpen = 'edit';
  }

  function closeForm() {
    formOpen = null;
    editId = null;
    formError = null;
  }

  async function submitCreate() {
    if (!formName.trim()) { formError = 'Name is required'; return; }
    if (!formCron.trim()) { formError = 'Cron expression is required'; return; }
    if (!formAgentId) { formError = 'Agent is required'; return; }
    if (!formPrompt.trim()) { formError = 'Prompt is required'; return; }
    submitting = true;
    formError = null;
    try {
      const payload: NewSchedule = {
        name: formName.trim(),
        cron_expr: formCron.trim(),
        agent_id: formAgentId,
        prompt: formPrompt.trim(),
        directory: formDirectory.trim() || '.',
      };
      await createSchedule(payload);
      closeForm();
      await load();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function submitEdit() {
    if (!editId) return;
    submitting = true;
    formError = null;
    try {
      const payload: UpdateSchedule = {
        name: formName.trim() || undefined,
        cron_expr: formCron.trim() || undefined,
        prompt: formPrompt.trim() || undefined,
        directory: formDirectory.trim() || undefined,
      };
      await updateSchedule(editId, payload);
      closeForm();
      await load();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function toggleEnabled(s: Schedule) {
    try {
      await updateSchedule(s.id, { enabled: !s.enabled });
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function doDelete(id: string) {
    try {
      await deleteSchedule(id);
      deleteConfirmId = null;
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function agentName(id: string): string {
    const a = agents.find((a) => a.id === id);
    return a?.name ?? id.slice(0, 8) + '...';
  }

  function formatDate(iso: string | null): string {
    if (!iso) return '--';
    try {
      return new Date(iso).toLocaleString('en-US', {
        month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
      });
    } catch { return iso; }
  }

  load();
</script>

<svelte:head>
  <title>Schedules &middot; AgentForge</title>
</svelte:head>

<div class="schedules-page">
  <header class="page-header">
    <h1>Schedules</h1>
    <button class="btn btn-primary" onclick={openCreate}>New Schedule</button>
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading schedules...</p>
  {:else if schedules.length === 0}
    <div class="empty-state">
      <p class="muted">No schedules yet. Schedule agents to run on a cron.</p>
      <button class="btn btn-primary" onclick={openCreate}>New Schedule</button>
    </div>
  {:else}
    <div class="schedule-list">
      {#each schedules as s (s.id)}
        <article class="card schedule-card">
          <div class="schedule-row">
            <div class="schedule-info">
              <div class="schedule-header">
                <h2 class="schedule-name">{s.name}</h2>
                <span class="badge">{s.cron_expr}</span>
                <span class="badge agent-badge">{agentName(s.agent_id)}</span>
              </div>
              <p class="schedule-prompt">{s.prompt}</p>
              <div class="schedule-meta">
                <span>Runs: {s.run_count}</span>
                <span>Last: {formatDate(s.last_run_at)}</span>
                <span>Next: {formatDate(s.next_run_at)}</span>
              </div>
            </div>
            <div class="schedule-controls">
              <label class="toggle-label" aria-label="Toggle enabled">
                <input type="checkbox" class="toggle-input" checked={s.enabled} onchange={() => toggleEnabled(s)} />
                <span class="toggle-switch"></span>
              </label>
              <button class="btn btn-ghost" onclick={() => openEdit(s)}>Edit</button>
              <button class="btn btn-ghost danger" onclick={() => (deleteConfirmId = s.id)}>Delete</button>
            </div>
          </div>
          {#if deleteConfirmId === s.id}
            <div class="delete-confirm">
              <span>Delete this schedule?</span>
              <button class="btn btn-ghost" onclick={() => (deleteConfirmId = null)}>Cancel</button>
              <button class="btn danger" onclick={() => doDelete(s.id)}>Delete</button>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}

  {#if formOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true">
      <div class="modal">
        <h2>{formOpen === 'create' ? 'New Schedule' : 'Edit Schedule'}</h2>
        {#if formError}
          <div class="message error">{formError}</div>
        {/if}
        <form class="schedule-form" onsubmit={(e) => { e.preventDefault(); formOpen === 'create' ? submitCreate() : submitEdit(); }}>
          <label><span>Name</span><input type="text" bind:value={formName} required placeholder="Schedule name" /></label>
          <label>
            <span>Cron Expression</span>
            <select onchange={(e) => { const v = (e.target as HTMLSelectElement).value; if (v) formCron = v; }}>
              {#each CRON_PRESETS as p}
                <option value={p.value} selected={p.value === formCron}>{p.label}</option>
              {/each}
            </select>
            <input type="text" bind:value={formCron} placeholder="sec min hour dom month dow year" />
          </label>
          <label>
            <span>Agent</span>
            <select bind:value={formAgentId}>
              {#each agents as a}
                <option value={a.id}>{a.name}</option>
              {/each}
            </select>
          </label>
          <label><span>Prompt</span><textarea bind:value={formPrompt} rows="3" required placeholder="What should the agent do?"></textarea></label>
          <label><span>Directory</span><input type="text" bind:value={formDirectory} placeholder="." /></label>
          <div class="form-actions">
            <button type="button" class="btn btn-ghost" onclick={closeForm}>Cancel</button>
            <button type="submit" class="btn btn-primary" disabled={submitting}>
              {submitting ? 'Saving...' : formOpen === 'create' ? 'Create' : 'Save'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .schedules-page { max-width: 56rem; }
  .schedule-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .schedule-card { padding: 1rem; }
  .schedule-row { display: flex; align-items: flex-start; justify-content: space-between; gap: 1rem; }
  .schedule-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 0.4rem; }
  .schedule-header { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; }
  .schedule-name { margin: 0; font-size: 1rem; font-weight: 600; }
  .agent-badge { background: rgba(167, 139, 250, 0.2); color: #a78bfa; }
  .schedule-prompt { margin: 0; font-size: 0.85rem; color: var(--muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%; }
  .schedule-meta { display: flex; gap: 1rem; font-size: 0.75rem; color: var(--muted); }
  .schedule-controls { display: flex; align-items: center; gap: 0.5rem; flex-shrink: 0; }
  .toggle-label { position: relative; display: inline-flex; align-items: center; cursor: pointer; }
  .toggle-input { position: absolute; opacity: 0; width: 0; height: 0; }
  .toggle-switch { width: 36px; height: 20px; background: var(--border); border-radius: 10px; position: relative; transition: background 0.2s; }
  .toggle-switch::after { content: ''; position: absolute; width: 16px; height: 16px; background: var(--text); border-radius: 50%; top: 2px; left: 2px; transition: transform 0.2s; }
  .toggle-input:checked + .toggle-switch { background: var(--accent); }
  .toggle-input:checked + .toggle-switch::after { transform: translateX(16px); }
  .schedule-form label { display: flex; flex-direction: column; gap: 0.35rem; margin-bottom: 1rem; }
  .schedule-form label span { font-size: 0.9rem; color: var(--muted); }
  .schedule-form input[type="text"], .schedule-form select, .schedule-form textarea {
    padding: 0.5rem 0.75rem; border-radius: 6px; border: 1px solid var(--border);
    background: var(--bg); color: var(--text); font-size: 0.9rem;
  }
  .schedule-form textarea { resize: vertical; min-height: 3rem; font-family: monospace; }
  .delete-confirm { display: flex; align-items: center; gap: 0.5rem; margin-top: 0.5rem; padding-top: 0.5rem; border-top: 1px solid var(--border); }
</style>
