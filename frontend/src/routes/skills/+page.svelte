<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import { listSkills, type Skill } from '$lib/api';

  setContext('pageTitle', 'Skills');

  let skills = $state<Skill[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

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

<div class="page skills-page">
  <h1>Skills</h1>
  {#if error}
    <p class="error">{error}</p>
    <p class="muted">Skills API may not be available. Ensure the backend is running.</p>
  {:else if loading}
    <p class="muted">Loading skills…</p>
  {:else if skills.length === 0}
    <p class="muted">No skills yet.</p>
  {:else}
    <ul class="skills-list">
      {#each skills as skill (skill.id)}
        <li class="skill-item">
          <span class="skill-name">{skill.name}</span>
          {#if skill.category}
            <span class="skill-meta">{skill.category}{#if skill.subcategory} / {skill.subcategory}{/if}</span>
          {/if}
          {#if skill.description}
            <p class="skill-desc">{skill.description}</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .page h1 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
  }
  .skills-page .error {
    color: #f87171;
    margin: 0 0 0.5rem 0;
  }
  .skills-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .skill-item {
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .skill-name {
    font-weight: 600;
  }
  .skill-meta {
    font-size: 0.85rem;
    color: var(--muted);
    margin-left: 0.5rem;
  }
  .skill-desc {
    margin: 0.35rem 0 0 0;
    font-size: 0.9rem;
    color: var(--muted);
  }
</style>
