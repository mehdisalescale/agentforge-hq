<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { listSkills, type Skill } from '$lib/api';

  setContext('pageTitle', 'Skills');

  let skills = $state<Skill[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let categoryFilter = $state('');
  let expandedIds = $state<Set<string>>(new Set());

  let categories = $derived.by<string[]>(() => {
    const cats = new Set<string>();
    for (const s of skills) {
      if (s.category) cats.add(s.category);
    }
    return Array.from(cats).sort();
  });

  let filtered = $derived.by<Skill[]>(() => {
    if (!categoryFilter) return skills;
    return skills.filter((s) => s.category === categoryFilter);
  });

  function parseTags(skill: Skill): string[] {
    if (!skill.parameters_json) return [];
    try {
      const parsed = JSON.parse(skill.parameters_json);
      if (Array.isArray(parsed?.tags)) return parsed.tags;
    } catch {
      // ignore
    }
    return [];
  }

  function contentPreview(content: string): { preview: string; full: string; needsExpand: boolean } {
    const lines = content.split('\n');
    const preview = lines.slice(0, 2).join('\n');
    return {
      preview,
      full: content,
      needsExpand: lines.length > 2,
    };
  }

  function toggleExpand(id: string) {
    const next = new Set(expandedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedIds = next;
  }

  async function loadSkills() {
    loading = true;
    error = null;
    try {
      skills = await listSkills();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      skills = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadSkills();
  });
</script>

<svelte:head>
  <title>Skills · Claude Forge</title>
</svelte:head>

<div class="skills-page">
  <header class="page-header">
    <h1>Skills</h1>
    {#if categories.length > 0}
      <select class="category-filter" bind:value={categoryFilter}>
        <option value="">All categories</option>
        {#each categories as cat}
          <option value={cat}>{cat}</option>
        {/each}
      </select>
    {/if}
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
    <p class="muted">Skills API may not be available. Ensure the backend is running.</p>
  {:else if loading}
    <p class="muted">Loading skills...</p>
  {:else if skills.length === 0}
    <div class="empty-state">
      <p class="empty-icon">&#128218;</p>
      <p class="muted">No skills yet. Skills are reusable prompt templates that agents can use.</p>
      <p class="muted hint">Add SKILL.md files to your project directories, or create them via the API.</p>
    </div>
  {:else if filtered.length === 0}
    <p class="muted">No skills match the selected category.</p>
  {:else}
    <div class="skill-cards">
      {#each filtered as skill (skill.id)}
        {@const tags = parseTags(skill)}
        {@const cp = contentPreview(skill.content)}
        <article class="card">
          <div class="card-header">
            <h2 class="card-title">{skill.name}</h2>
            {#if skill.category}
              <span class="badge">{skill.category}{#if skill.subcategory} / {skill.subcategory}{/if}</span>
            {/if}
            <span class="usage-count" title="Usage count">{skill.usage_count} use{skill.usage_count !== 1 ? 's' : ''}</span>
          </div>

          {#if tags.length > 0}
            <div class="tag-row">
              {#each tags as tag}
                <span class="tag-pill">{tag}</span>
              {/each}
            </div>
          {/if}

          {#if skill.description}
            <p class="card-desc">{skill.description}</p>
          {/if}

          <div class="content-preview">
            {#if expandedIds.has(skill.id)}
              <pre class="content-text">{cp.full}</pre>
            {:else}
              <pre class="content-text">{cp.preview}</pre>
            {/if}
            {#if cp.needsExpand}
              <button class="btn btn-ghost expand-btn" onclick={() => toggleExpand(skill.id)}>
                {expandedIds.has(skill.id) ? 'Show less' : 'Show more'}
              </button>
            {/if}
          </div>

          {#if skill.source_repo}
            <div class="card-footer">
              <span class="source-label">Source:</span>
              <span class="source-value">{skill.source_repo}</span>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .skills-page {
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

  .category-filter {
    padding: 0.4rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.85rem;
  }

  .empty-state {
    padding: 3rem 2rem;
    text-align: center;
  }

  .empty-icon {
    font-size: 2.5rem;
    margin: 0 0 0.75rem 0;
    opacity: 0.5;
  }

  .hint {
    font-size: 0.85rem;
    margin-top: 0.5rem;
  }

  .skill-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .card-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  .card-title {
    margin: 0;
    font-size: 1.1rem;
    flex: 1 1 auto;
  }

  .badge {
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    background: rgba(167, 139, 250, 0.2);
    border-radius: 4px;
    color: var(--accent);
  }

  .usage-count {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
  }

  .tag-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .tag-pill {
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    border-radius: 9999px;
    background: rgba(96, 165, 250, 0.15);
    color: #93c5fd;
    border: 1px solid rgba(96, 165, 250, 0.25);
  }

  .card-desc {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
    line-height: 1.4;
  }

  .content-preview {
    border-top: 1px solid var(--border);
    padding-top: 0.5rem;
  }

  .content-text {
    margin: 0;
    font-size: 0.8rem;
    color: var(--muted);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: ui-monospace, 'Cascadia Code', 'Source Code Pro', monospace;
    max-height: 20rem;
    overflow: auto;
  }

  .expand-btn {
    font-size: 0.78rem;
    padding: 0.2rem 0.5rem;
    margin-top: 0.25rem;
    color: var(--accent);
  }

  .card-footer {
    border-top: 1px solid var(--border);
    padding-top: 0.5rem;
    font-size: 0.75rem;
    color: var(--muted);
    display: flex;
    gap: 0.35rem;
  }

  .source-label {
    opacity: 0.7;
  }

  .source-value {
    font-family: ui-monospace, 'Cascadia Code', 'Source Code Pro', monospace;
    word-break: break-all;
  }

  .message.error {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .muted {
    color: var(--muted);
  }
</style>
