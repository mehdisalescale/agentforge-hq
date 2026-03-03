<script lang="ts">
  import {
    listMemories,
    createMemory,
    updateMemory,
    deleteMemory,
    getMemory,
    type Memory,
    type NewMemory,
    type UpdateMemory,
  } from '$lib/api';

  let memories = $state<Memory[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let searchQuery = $state('');
  let searchTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

  // Modal state
  let formOpen = $state<'create' | 'edit' | null>(null);
  let editId = $state<string | null>(null);
  let deleteConfirmId = $state<string | null>(null);
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  // Form fields
  let formContent = $state('');
  let formCategory = $state('');
  let formConfidence = $state(80);

  async function loadMemories(query?: string) {
    loading = true;
    error = null;
    try {
      memories = await listMemories(query ? { q: query } : { limit: 50 });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleSearch() {
    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      loadMemories(searchQuery.trim() || undefined);
    }, 300);
  }

  function openCreate() {
    editId = null;
    formContent = '';
    formCategory = 'general';
    formConfidence = 80;
    formError = null;
    formOpen = 'create';
  }

  async function openEdit(id: string) {
    formError = null;
    try {
      const m = await getMemory(id);
      editId = id;
      formContent = m.content;
      formCategory = m.category;
      formConfidence = Math.round(m.confidence * 100);
      formOpen = 'edit';
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    }
  }

  function closeForm() {
    formOpen = null;
    editId = null;
    formError = null;
  }

  async function submitCreate() {
    if (!formContent.trim()) {
      formError = 'Content is required';
      return;
    }
    submitting = true;
    formError = null;
    try {
      const payload: NewMemory = {
        content: formContent.trim(),
        category: formCategory.trim() || 'general',
        confidence: formConfidence / 100,
      };
      await createMemory(payload);
      closeForm();
      await loadMemories(searchQuery.trim() || undefined);
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function submitEdit() {
    if (!editId || !formContent.trim()) {
      formError = editId ? 'Content is required' : 'Invalid memory';
      return;
    }
    submitting = true;
    formError = null;
    try {
      const payload: UpdateMemory = {
        content: formContent.trim(),
        category: formCategory.trim() || 'general',
        confidence: formConfidence / 100,
      };
      await updateMemory(editId, payload);
      closeForm();
      await loadMemories(searchQuery.trim() || undefined);
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function doDelete(id: string) {
    try {
      await deleteMemory(id);
      deleteConfirmId = null;
      await loadMemories(searchQuery.trim() || undefined);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function confidenceColor(value: number): string {
    if (value < 0.3) return '#ef4444';
    if (value <= 0.7) return '#eab308';
    return '#22c55e';
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return iso;
    }
  }

  loadMemories();
</script>

<svelte:head>
  <title>Memory · Claude Forge</title>
</svelte:head>

<div class="memory-page">
  <header class="page-header">
    <h1>Memory</h1>
    <button class="btn btn-primary" onclick={openCreate}>Add Memory</button>
  </header>

  <div class="search-bar">
    <input
      type="text"
      placeholder="Search memories..."
      bind:value={searchQuery}
      oninput={handleSearch}
    />
  </div>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading memories...</p>
  {:else if memories.length === 0}
    <div class="empty-state">
      <p class="muted">
        {searchQuery.trim() ? 'No memories match your search.' : 'No memories yet. Memories are extracted from session transcripts.'}
      </p>
      {#if !searchQuery.trim()}
        <button class="btn btn-primary" onclick={openCreate}>Add Memory</button>
      {/if}
    </div>
  {:else}
    <div class="memory-cards">
      {#each memories as memory (memory.id)}
        <article class="card">
          <div class="card-header">
            <span class="badge category-badge">{memory.category}</span>
            <span class="card-meta">{formatDate(memory.created_at)}</span>
          </div>
          <p class="card-content">{memory.content}</p>
          <div class="confidence-row">
            <span class="confidence-label">Confidence</span>
            <div class="confidence-bar-track">
              <div
                class="confidence-bar-fill"
                style="width: {Math.round(memory.confidence * 100)}%; background: {confidenceColor(memory.confidence)}"
              ></div>
            </div>
            <span class="confidence-value">{Math.round(memory.confidence * 100)}%</span>
          </div>
          {#if memory.source_session_id}
            <div class="card-meta source-row">Session: {memory.source_session_id.slice(0, 8)}...</div>
          {/if}
          <div class="card-actions">
            <button class="btn btn-ghost" onclick={() => openEdit(memory.id)}>Edit</button>
            <button
              class="btn btn-ghost danger"
              onclick={() => (deleteConfirmId = memory.id)}
              aria-label="Delete memory"
            >
              Delete
            </button>
          </div>
          {#if deleteConfirmId === memory.id}
            <div class="delete-confirm">
              <span>Delete this memory?</span>
              <button class="btn btn-ghost" onclick={() => (deleteConfirmId = null)}>Cancel</button>
              <button class="btn danger" onclick={() => doDelete(memory.id)}>Delete</button>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}

  {#if formOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="form-title">
      <div class="modal">
        <h2 id="form-title">{formOpen === 'create' ? 'Add Memory' : 'Edit Memory'}</h2>
        {#if formError}
          <div class="message error">{formError}</div>
        {/if}
        <form
          class="memory-form"
          onsubmit={(e) => {
            e.preventDefault();
            if (formOpen === 'create') submitCreate();
            else submitEdit();
          }}
        >
          <label>
            <span>Content</span>
            <textarea bind:value={formContent} rows="4" required placeholder="Memory content..."></textarea>
          </label>
          <label>
            <span>Category</span>
            <input type="text" bind:value={formCategory} placeholder="e.g. general, preference, fact" />
          </label>
          <label>
            <span>Confidence: {formConfidence}%</span>
            <div class="slider-row">
              <input
                type="range"
                min="0"
                max="100"
                bind:value={formConfidence}
              />
              <div
                class="slider-indicator"
                style="background: {confidenceColor(formConfidence / 100)}"
              ></div>
            </div>
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
  .memory-page {
    max-width: 56rem;
  }

  .search-bar {
    margin-bottom: 1.5rem;
  }

  .search-bar input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.9rem;
  }

  .memory-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }

  .card-content {
    margin: 0 0 0.75rem 0;
    font-size: 0.9rem;
    color: var(--text);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .category-badge {
    text-transform: capitalize;
  }

  .confidence-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .confidence-label {
    font-size: 0.75rem;
    color: var(--muted);
    flex-shrink: 0;
  }

  .confidence-bar-track {
    flex: 1;
    height: 6px;
    background: var(--border);
    border-radius: 3px;
    overflow: hidden;
  }

  .confidence-bar-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.2s;
  }

  .confidence-value {
    font-size: 0.75rem;
    color: var(--muted);
    flex-shrink: 0;
    min-width: 2.5rem;
    text-align: right;
  }

  .source-row {
    margin-bottom: 0.5rem;
    font-size: 0.8rem;
  }

  /* Form styles */
  .memory-form label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 1rem;
  }

  .memory-form label span {
    font-size: 0.9rem;
    color: var(--muted);
  }

  .memory-form input[type="text"],
  .memory-form textarea {
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.9rem;
  }

  .memory-form textarea {
    resize: vertical;
    min-height: 4rem;
  }

  .slider-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .slider-row input[type="range"] {
    flex: 1;
    accent-color: var(--accent);
  }

  .slider-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }
</style>
