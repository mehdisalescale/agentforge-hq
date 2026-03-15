<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import {
    Building2, Users, Zap, ArrowRight, X, Sparkles,
    Network, Target, ShieldCheck, Bot, History,
    Puzzle, BarChart3, Settings
  } from 'lucide-svelte';

  const STORAGE_KEY = 'agentforge_onboarding_done';

  let visible = $state(false);

  if (browser) {
    const done = localStorage.getItem(STORAGE_KEY);
    if (!done) visible = true;
  }

  function dismiss() {
    visible = false;
    if (browser) localStorage.setItem(STORAGE_KEY, '1');
  }

  function goTo(href: string) {
    dismiss();
    goto(href);
  }

  const steps = [
    {
      num: 1,
      icon: Building2,
      title: 'Create a company',
      desc: 'Name it, give it a mission and budget. Everything else lives under a company.',
      action: 'Create Company',
      href: '/companies',
    },
    {
      num: 2,
      icon: Users,
      title: 'Hire AI personas',
      desc: 'Pick from 100+ specialists — engineers, designers, PMs — and add them to your org.',
      action: 'Browse Personas',
      href: '/personas',
    },
    {
      num: 3,
      icon: Zap,
      title: 'Run your first agent',
      desc: 'Give an agent a task in plain English. Watch it work with streaming output.',
      action: 'Go to Dashboard',
      href: '/',
    },
  ];

  const extras = [
    { icon: Bot, name: 'Agents', href: '/agents' },
    { icon: History, name: 'Sessions', href: '/sessions' },
    { icon: Network, name: 'Org Chart', href: '/org-chart' },
    { icon: Target, name: 'Goals', href: '/goals' },
    { icon: ShieldCheck, name: 'Approvals', href: '/approvals' },
    { icon: Puzzle, name: 'Skills', href: '/skills' },
    { icon: BarChart3, name: 'Analytics', href: '/analytics' },
    { icon: Settings, name: 'Settings', href: '/settings' },
  ];
</script>

{#if visible}
<div class="onboarding-overlay" role="dialog" aria-modal="true" aria-label="Get started with AgentForge">
  <div class="onboarding-card">
    <button class="dismiss" onclick={dismiss} aria-label="Dismiss">
      <X size={16} />
    </button>

    <div class="header">
      <div class="logo-icon"><Sparkles size={24} /></div>
      <h1>Get started with AgentForge</h1>
      <p>Three steps to your first AI workforce.</p>
    </div>

    <ol class="steps">
      {#each steps as step}
        {@const Icon = step.icon}
        <li class="step">
          <div class="step-marker">{step.num}</div>
          <div class="step-body">
            <div class="step-top">
              <Icon size={18} class="step-icon" />
              <strong>{step.title}</strong>
            </div>
            <p>{step.desc}</p>
            <button class="step-action" onclick={() => goTo(step.href)}>
              {step.action}
              <ArrowRight size={14} />
            </button>
          </div>
        </li>
      {/each}
    </ol>

    <div class="extras">
      <span class="extras-label">Also available</span>
      <div class="extras-grid">
        {#each extras as item}
          {@const Icon = item.icon}
          <button class="extra-chip" onclick={() => goTo(item.href)}>
            <Icon size={14} />
            {item.name}
          </button>
        {/each}
      </div>
    </div>

    <div class="footer">
      <button class="skip" onclick={dismiss}>I'll explore on my own</button>
    </div>
  </div>
</div>
{/if}

<style>
  .onboarding-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    animation: fade-in 200ms ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .onboarding-card {
    position: relative;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 16px;
    width: 100%;
    max-width: 32rem;
    padding: 2rem 1.75rem 1.5rem;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }

  .dismiss {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    transition: all var(--transition);
  }

  .dismiss:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .header {
    text-align: center;
    margin-bottom: 1.75rem;
  }

  .logo-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 3rem;
    height: 3rem;
    border-radius: 12px;
    background: var(--accent-muted);
    color: var(--accent);
    margin-bottom: 0.75rem;
  }

  .header h1 {
    margin: 0 0 0.35rem;
    font-size: 1.25rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text);
  }

  .header p {
    margin: 0;
    font-size: 0.875rem;
    color: var(--muted);
  }

  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .step {
    display: flex;
    gap: 0.85rem;
    align-items: flex-start;
  }

  .step-marker {
    flex-shrink: 0;
    width: 1.6rem;
    height: 1.6rem;
    border-radius: 50%;
    background: var(--accent);
    color: var(--bg);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 700;
    margin-top: 0.1rem;
  }

  .step-body {
    flex: 1;
    min-width: 0;
  }

  .step-top {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.2rem;
  }

  .step-top :global(svg) {
    color: var(--accent);
    flex-shrink: 0;
    opacity: 0.7;
  }

  .step-top strong {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text);
  }

  .step-body p {
    margin: 0 0 0.4rem;
    font-size: 0.8rem;
    color: var(--muted);
    line-height: 1.45;
  }

  .step-action {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--accent);
    font-size: 0.78rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition);
    font-family: inherit;
  }

  .step-action:hover {
    background: var(--accent-muted);
    border-color: var(--accent);
    gap: 0.45rem;
  }

  .extras {
    margin-top: 1.5rem;
    padding-top: 1.25rem;
    border-top: 1px solid var(--border);
  }

  .extras-label {
    display: block;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    margin-bottom: 0.5rem;
  }

  .extras-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .extra-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.55rem;
    border-radius: 5px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.72rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition);
    font-family: inherit;
  }

  .extra-chip :global(svg) {
    color: var(--muted);
    flex-shrink: 0;
  }

  .extra-chip:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-muted);
  }

  .extra-chip:hover :global(svg) {
    color: var(--accent);
  }

  .footer {
    margin-top: 1rem;
    text-align: center;
  }

  .skip {
    padding: 0.4rem 0.75rem;
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: 0.8rem;
    cursor: pointer;
    border-radius: 6px;
    transition: all var(--transition);
    font-family: inherit;
  }

  .skip:hover {
    color: var(--text-secondary);
    background: var(--surface-hover);
  }
</style>
