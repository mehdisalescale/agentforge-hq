<script lang="ts">
  import { AlertTriangle, RefreshCw } from 'lucide-svelte';

  let { message, onretry }: { message: string; onretry?: () => void } = $props();
</script>

<div class="error-message" role="alert">
  <div class="error-content">
    <AlertTriangle size={16} />
    <span>{message}</span>
  </div>
  {#if onretry}
    <button class="btn btn-ghost" onclick={onretry}>
      <RefreshCw size={14} />
      Retry
    </button>
  {/if}
</div>

<style>
  .error-message {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-radius: var(--radius);
    background: var(--danger-muted);
    color: #fca5a5;
    border: 1px solid rgba(248, 113, 113, 0.3);
    margin-bottom: 1rem;
  }

  .error-content {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
  }
</style>
