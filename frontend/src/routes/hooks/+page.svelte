<script lang="ts">
  import {
    listHooks,
    createHook,
    updateHook,
    deleteHook,
    type Hook,
    type NewHook,
    type UpdateHook,
  } from '$lib/api';

  const EVENT_TYPES = [
    'ProcessStarted',
    'ProcessCompleted',
    'ProcessFailed',
    'SessionCreated',
    'SessionUpdated',
    'HookStarted',
    'HookCompleted',
    'SubAgentRequested',
    'SubAgentStarted',
    'SubAgentCompleted',
    'SubAgentFailed',
  ];

  const TIMINGS = ['pre', 'post'];

  let hooks = $state<Hook[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Modal state
  let formOpen = $state<'create' | 'edit' | null>(null);
  let editId = $state<string | null>(null);
  let deleteConfirmId = $state<string | null>(null);
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  // Form fields
  let formName = $state('');
  let formEventType = $state('ProcessCompleted');
  let formTiming = $state('post');
  let formCommand = $state('');

  async function loadHooks() {
    loading = true;
    error = null;
    try {
      hooks = await listHooks();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function openCreate() {
    editId = null;
    formName = '';
    formEventType = 'ProcessCompleted';
    formTiming = 'post';
    formCommand = '';
    formError = null;
    formOpen = 'create';
  }

  function openEdit(hook: Hook) {
    editId = hook.id;
    formName = hook.name;
    formEventType = hook.event_type;
    formTiming = hook.timing;
    formCommand = hook.command;
    formError = null;
    formOpen = 'edit';
  }

  function closeForm() {
    formOpen = null;
    editId = null;
    formError = null;
  }

  async function submitCreate() {
    if (!formName.trim()) {
      formError = 'Name is required';
      return;
    }
    if (!formCommand.trim()) {
      formError = 'Command is required';
      return;
    }
    submitting = true;
    formError = null;
    try {
      const payload: NewHook = {
        name: formName.trim(),
        event_type: formEventType,
        timing: formTiming,
        command: formCommand.trim(),
      };
      await createHook(payload);
      closeForm();
      await loadHooks();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function submitEdit() {
    if (!editId) {
      formError = 'Invalid hook';
      return;
    }
    if (!formName.trim()) {
      formError = 'Name is required';
      return;
    }
    submitting = true;
    formError = null;
    try {
      const payload: UpdateHook = {
        name: formName.trim(),
        command: formCommand.trim() || undefined,
      };
      await updateHook(editId, payload);
      closeForm();
      await loadHooks();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function toggleEnabled(hook: Hook) {
    try {
      await updateHook(hook.id, { enabled: !hook.enabled });
      await loadHooks();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function doDelete(id: string) {
    try {
      await deleteHook(id);
      deleteConfirmId = null;
      await loadHooks();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
      });
    } catch {
      return iso;
    }
  }

  loadHooks();
</script>

<svelte:head>
  <title>Hooks · Claude Forge</title>
</svelte:head>

<div class="hooks-page">
  <header class="page-header">
    <h1>Hooks</h1>
    <button class="btn btn-primary" onclick={openCreate}>New Hook</button>
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading hooks...</p>
  {:else if hooks.length === 0}
    <div class="empty-state">
      <p class="muted">No hooks yet. Hooks run shell commands when events occur.</p>
      <button class="btn btn-primary" onclick={openCreate}>New Hook</button>
    </div>
  {:else}
    <div class="hooks-list">
      {#each hooks as hook (hook.id)}
        <article class="card hook-card">
          <div class="hook-row">
            <div class="hook-info">
              <div class="hook-header">
                <h2 class="hook-name">{hook.name}</h2>
                <span class="badge">{hook.event_type}</span>
                <span
                  class="badge timing-badge"
                  class:timing-pre={hook.timing === 'pre'}
                  class:timing-post={hook.timing === 'post'}
                >
                  {hook.timing}
                </span>
              </div>
              <code class="hook-command">{hook.command}</code>
              <span class="card-meta">{formatDate(hook.created_at)}</span>
            </div>
            <div class="hook-controls">
              <label class="toggle-label" aria-label="Toggle enabled">
                <input
                  type="checkbox"
                  class="toggle-input"
                  checked={hook.enabled}
                  onchange={() => toggleEnabled(hook)}
                />
                <span class="toggle-switch"></span>
              </label>
              <button class="btn btn-ghost" onclick={() => openEdit(hook)}>Edit</button>
              <button
                class="btn btn-ghost danger"
                onclick={() => (deleteConfirmId = hook.id)}
                aria-label="Delete {hook.name}"
              >
                Delete
              </button>
            </div>
          </div>
          {#if deleteConfirmId === hook.id}
            <div class="delete-confirm">
              <span>Delete this hook?</span>
              <button class="btn btn-ghost" onclick={() => (deleteConfirmId = null)}>Cancel</button>
              <button class="btn danger" onclick={() => doDelete(hook.id)}>Delete</button>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}

  {#if formOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="form-title">
      <div class="modal">
        <h2 id="form-title">{formOpen === 'create' ? 'New Hook' : 'Edit Hook'}</h2>
        {#if formError}
          <div class="message error">{formError}</div>
        {/if}
        <form
          class="hook-form"
          onsubmit={(e) => {
            e.preventDefault();
            if (formOpen === 'create') submitCreate();
            else submitEdit();
          }}
        >
          <label>
            <span>Name</span>
            <input type="text" bind:value={formName} required placeholder="Hook name" />
          </label>
          {#if formOpen === 'create'}
            <label>
              <span>Event Type</span>
              <select bind:value={formEventType}>
                {#each EVENT_TYPES as et}
                  <option value={et}>{et}</option>
                {/each}
              </select>
            </label>
            <label>
              <span>Timing</span>
              <select bind:value={formTiming}>
                {#each TIMINGS as t}
                  <option value={t}>{t}</option>
                {/each}
              </select>
            </label>
          {:else}
            <div class="readonly-field">
              <span class="readonly-label">Event</span>
              <span>{formEventType} ({formTiming})</span>
            </div>
          {/if}
          <label>
            <span>Command</span>
            <textarea bind:value={formCommand} rows="3" required placeholder="Shell command to run..."></textarea>
          </label>
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
  .hooks-page {
    max-width: 56rem;
  }

  .hooks-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .hook-card {
    padding: 1rem;
  }

  .hook-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
  }

  .hook-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .hook-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  .hook-name {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .timing-badge.timing-pre {
    background: rgba(96, 165, 250, 0.2);
    color: #60a5fa;
  }

  .timing-badge.timing-post {
    background: rgba(134, 239, 172, 0.2);
    color: #86efac;
  }

  .hook-command {
    font-size: 0.8rem;
    color: var(--muted);
    background: var(--bg);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
    max-width: 100%;
  }

  .hook-controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  /* Toggle switch */
  .toggle-label {
    position: relative;
    display: inline-flex;
    align-items: center;
    cursor: pointer;
  }

  .toggle-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-switch {
    width: 36px;
    height: 20px;
    background: var(--border);
    border-radius: 10px;
    position: relative;
    transition: background 0.2s;
  }

  .toggle-switch::after {
    content: '';
    position: absolute;
    width: 16px;
    height: 16px;
    background: var(--text);
    border-radius: 50%;
    top: 2px;
    left: 2px;
    transition: transform 0.2s;
  }

  .toggle-input:checked + .toggle-switch {
    background: var(--accent);
  }

  .toggle-input:checked + .toggle-switch::after {
    transform: translateX(16px);
  }

  /* Form styles */
  .hook-form label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 1rem;
  }

  .hook-form label span {
    font-size: 0.9rem;
    color: var(--muted);
  }

  .hook-form input[type="text"],
  .hook-form select,
  .hook-form textarea {
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.9rem;
  }

  .hook-form textarea {
    resize: vertical;
    min-height: 3rem;
    font-family: monospace;
  }

  .readonly-field {
    margin-bottom: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .readonly-label {
    font-size: 0.9rem;
    color: var(--muted);
  }

  .readonly-field > span:last-child {
    font-size: 0.9rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg);
    border-radius: 6px;
    border: 1px solid var(--border);
    color: var(--muted);
  }
</style>
