<script lang="ts">
  let { lines = 3, type = 'text' }: { lines?: number; type?: 'text' | 'card' | 'table' } = $props();
</script>

{#if type === 'card'}
  <div class="skeleton-cards">
    {#each Array(lines) as _}
      <div class="skeleton-card">
        <div class="skeleton-line w-60"></div>
        <div class="skeleton-line w-80"></div>
        <div class="skeleton-line w-40"></div>
      </div>
    {/each}
  </div>
{:else if type === 'table'}
  <div class="skeleton-table">
    {#each Array(lines) as _}
      <div class="skeleton-row">
        <div class="skeleton-line w-20"></div>
        <div class="skeleton-line w-40"></div>
        <div class="skeleton-line w-30"></div>
      </div>
    {/each}
  </div>
{:else}
  <div class="skeleton-text">
    {#each Array(lines) as _, i}
      <div class="skeleton-line" style="width: {80 - i * 15}%"></div>
    {/each}
  </div>
{/if}

<style>
  .skeleton-line {
    height: 0.875rem;
    background: var(--surface-hover);
    border-radius: 4px;
    animation: shimmer 1.5s ease-in-out infinite;
    margin-bottom: 0.5rem;
  }

  .skeleton-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .skeleton-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 1.25rem;
  }

  .skeleton-row {
    display: flex;
    gap: 1rem;
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--border);
  }

  .skeleton-table {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.5rem 1rem;
  }

  .w-20 { width: 20%; }
  .w-30 { width: 30%; }
  .w-40 { width: 40%; }
  .w-60 { width: 60%; }
  .w-80 { width: 80%; }

  @keyframes shimmer {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.8; }
  }
</style>
