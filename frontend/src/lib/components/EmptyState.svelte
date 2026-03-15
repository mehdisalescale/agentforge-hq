<script lang="ts">
  import type { Snippet } from 'svelte';

  let { icon, title, description, actionLabel, onaction }: {
    icon?: any;
    title: string;
    description?: string;
    actionLabel?: string;
    onaction?: () => void;
  } = $props();
</script>

<div class="empty-state">
  {#if icon}
    {@const Icon = icon}
    <div class="empty-icon">
      <Icon size={32} />
    </div>
  {/if}
  <h3>{title}</h3>
  {#if description}
    <p>{description}</p>
  {/if}
  {#if actionLabel && onaction}
    <button class="btn btn-primary" onclick={onaction}>{actionLabel}</button>
  {/if}
</div>

<style>
  .empty-state {
    padding: 3rem 2rem;
    text-align: center;
    color: var(--muted);
  }

  .empty-icon {
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  p {
    margin: 0 0 1.25rem 0;
    font-size: 0.9rem;
    max-width: 24rem;
    margin-left: auto;
    margin-right: auto;
  }
</style>
