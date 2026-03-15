<script lang="ts">
  import { browser } from '$app/environment';
  import {
    ChevronDown, ChevronUp, X,
    Users, BarChart3, Target, ShieldCheck
  } from 'lucide-svelte';

  const STORAGE_KEY = 'agentforge_guide_dismissed';

  interface Props {
    agentCount: number;
    sessionCount: number;
    totalCost: number;
  }
  let { agentCount, sessionCount, totalCost }: Props = $props();

  let dismissed = $state(false);

  if (browser) {
    dismissed = localStorage.getItem(STORAGE_KEY) === '1';
  }

  function dismiss() {
    dismissed = true;
    if (browser) localStorage.setItem(STORAGE_KEY, '1');
  }

  function show() {
    dismissed = false;
    if (browser) localStorage.removeItem(STORAGE_KEY);
  }

  interface Tip {
    icon: typeof Users;
    text: string;
    href: string;
  }

  let tips = $derived.by<Tip[]>(() => {
    const t: Tip[] = [];
    if (agentCount === 0) {
      t.push({ icon: Users, text: 'Browse Personas and hire your first agent into a company.', href: '/personas' });
    } else if (sessionCount === 0) {
      t.push({ icon: Users, text: `You have ${agentCount} agent${agentCount !== 1 ? 's' : ''} ready. Pick one below, type a prompt, and hit Run.`, href: '/' });
    } else {
      t.push({ icon: BarChart3, text: `${sessionCount} run${sessionCount !== 1 ? 's' : ''} so far ($${totalCost.toFixed(2)} spent). Check Analytics for details.`, href: '/analytics' });
    }
    t.push({ icon: Target, text: 'Set Goals for your company — they get injected into agent prompts automatically.', href: '/goals' });
    t.push({ icon: ShieldCheck, text: 'Use Approvals to gate sensitive actions with explicit yes/no decisions.', href: '/approvals' });
    return t;
  });
</script>

{#if dismissed}
  <button class="guide-toggle" onclick={show}>
    <ChevronDown size={14} />
    Getting Started
  </button>
{:else}
  <div class="guide-card">
    <div class="guide-header">
      <h3>Getting Started</h3>
      <button class="guide-dismiss" onclick={dismiss} aria-label="Dismiss guide">
        <X size={14} />
      </button>
    </div>

    <p class="guide-why">
      <strong>Why AgentForge?</strong> You could run <code>claude</code> directly, but then you
      get no budget controls, no audit trail, no specialized personas, and no org structure.
      AgentForge wraps your AI tools with governance and observability.
    </p>

    <div class="guide-tips">
      {#each tips as tip}
        {@const Icon = tip.icon}
        <a class="guide-tip" href={tip.href}>
          <span class="tip-icon"><Icon size={14} /></span>
          <span>{tip.text}</span>
        </a>
      {/each}
    </div>
  </div>
{/if}

<style>
  .guide-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.35rem 0.7rem;
    border-radius: 6px;
    border: 1px dashed var(--border);
    background: transparent;
    color: var(--muted);
    font-size: 0.78rem;
    font-family: inherit;
    cursor: pointer;
    transition: all 150ms;
    margin-bottom: 1rem;
  }
  .guide-toggle:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .guide-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent);
    border-radius: 8px;
    padding: 1rem 1.25rem;
    margin-bottom: 1.5rem;
  }

  .guide-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.6rem;
  }

  .guide-header h3 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text);
  }

  .guide-dismiss {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .guide-dismiss:hover {
    background: var(--surface-hover, rgba(255,255,255,0.06));
    color: var(--text);
  }

  .guide-why {
    margin: 0 0 0.75rem;
    font-size: 0.82rem;
    color: var(--muted);
    line-height: 1.5;
  }
  .guide-why strong {
    color: var(--text);
  }
  .guide-why code {
    font-size: 0.8em;
    padding: 0.1rem 0.3rem;
    background: var(--bg);
    border-radius: 3px;
  }

  .guide-tips {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .guide-tip {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    font-size: 0.8rem;
    color: var(--text);
    text-decoration: none;
    line-height: 1.4;
    transition: background 150ms;
  }
  .guide-tip:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .tip-icon {
    flex-shrink: 0;
    color: var(--accent);
    margin-top: 0.1rem;
  }
</style>
