<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import {
    Sparkles, Zap, Bot, History, GitBranch,
    Building2, Users, Network, Target, ShieldCheck,
    Puzzle, Brain, Webhook, Clock,
    BarChart3, Settings, ArrowRight, ArrowLeft,
    Rocket, X, ChevronRight, Check, Play
  } from 'lucide-svelte';

  const STORAGE_KEY = 'agentforge_onboarding_done';

  let visible = $state(false);
  let step = $state(0);

  if (browser) {
    const done = localStorage.getItem(STORAGE_KEY);
    if (!done) visible = true;
  }

  function dismiss() {
    visible = false;
    if (browser) localStorage.setItem(STORAGE_KEY, '1');
  }

  function next() {
    if (step < steps.length - 1) step++;
  }

  function prev() {
    if (step > 0) step--;
  }

  function goTo(href: string) {
    dismiss();
    goto(href);
  }

  interface FeatureItem {
    icon: typeof Zap;
    name: string;
    desc: string;
    href: string;
  }

  interface Step {
    id: string;
    label: string;
    title: string;
    subtitle: string;
    features: FeatureItem[];
  }

  const steps: Step[] = [
    {
      id: 'welcome',
      label: 'Welcome',
      title: 'Welcome to AgentForge',
      subtitle: 'The self-improving AI workforce platform. Build teams of intelligent agents, organize them into companies, set goals with budgets, and let them execute real work — all with governance controls.',
      features: [
        { icon: Building2, name: 'Build Organizations', desc: 'Create companies with departments and org charts', href: '/companies' },
        { icon: Users, name: 'Hire AI Personas', desc: 'Choose from 100+ pre-built agent personas', href: '/personas' },
        { icon: Zap, name: 'Execute Tasks', desc: 'Run agents with real-time streaming output', href: '/' },
        { icon: ShieldCheck, name: 'Govern Decisions', desc: 'Approval workflows and budget controls', href: '/approvals' },
      ],
    },
    {
      id: 'workspace',
      label: 'Workspace',
      title: 'Your Workspace',
      subtitle: 'The workspace is where you run agents, manage their configurations, review session history, and orchestrate multi-step workflows.',
      features: [
        { icon: Zap, name: 'Run', desc: 'Execute agents with prompts. See streaming output, tool calls, and sub-agent swim lanes in real time.', href: '/' },
        { icon: Bot, name: 'Agents', desc: 'Create and configure agents with custom system prompts, models, and tool permissions.', href: '/agents' },
        { icon: History, name: 'Sessions', desc: 'Browse past agent runs. Review outputs, resume interrupted sessions, or re-run with new prompts.', href: '/sessions' },
        { icon: GitBranch, name: 'Workflows', desc: 'Define multi-step automations that chain agents together with conditional logic.', href: '/workflows' },
      ],
    },
    {
      id: 'organization',
      label: 'Organization',
      title: 'Organization & Governance',
      subtitle: 'Model your AI workforce like a real company. Create organizations, hire personas into roles, visualize reporting lines, and govern with goals and approvals.',
      features: [
        { icon: Building2, name: 'Companies', desc: 'Create organizations with a name, mission, and budget. Everything else is scoped to a company.', href: '/companies' },
        { icon: Users, name: 'Personas', desc: 'Browse 100+ pre-built personas across 11 divisions. Hire them into your company to auto-create agents and org positions.', href: '/personas' },
        { icon: Network, name: 'Org Chart', desc: 'Visualize the reporting hierarchy of your AI workforce. See who reports to whom at a glance.', href: '/org-chart' },
        { icon: Target, name: 'Goals', desc: 'Set company-level objectives and track status. Goals give agents direction and keep work aligned.', href: '/goals' },
        { icon: ShieldCheck, name: 'Approvals', desc: 'Governance layer for decisions that need explicit sign-off. Review, approve, or reject requests.', href: '/approvals' },
      ],
    },
    {
      id: 'configuration',
      label: 'Configuration',
      title: 'Configuration & Automation',
      subtitle: 'Fine-tune how your agents work. Equip them with skills, give them persistent memory, wire up event hooks, and schedule recurring tasks.',
      features: [
        { icon: Puzzle, name: 'Skills', desc: 'Agent capability library with 30+ skills — from code review and debugging to deep research and TDD.', href: '/skills' },
        { icon: Brain, name: 'Memory', desc: 'Persistent knowledge base that agents can read and write. Memories carry context across sessions.', href: '/memory' },
        { icon: Webhook, name: 'Hooks', desc: 'Event-driven automation. Trigger actions when agents start, complete, fail, or produce specific outputs.', href: '/hooks' },
        { icon: Clock, name: 'Schedules', desc: 'Cron-based scheduling for recurring agent runs. Set it and forget it — agents run on your timetable.', href: '/schedules' },
      ],
    },
    {
      id: 'insights',
      label: 'Insights',
      title: 'Insights & Settings',
      subtitle: 'Monitor costs, track usage patterns, and configure platform-wide settings.',
      features: [
        { icon: BarChart3, name: 'Analytics', desc: 'Cost tracking, token usage, and run statistics. See which agents cost the most and where budgets stand.', href: '/analytics' },
        { icon: Settings, name: 'Settings', desc: 'Platform configuration — CLI command, rate limits, CORS, budget thresholds, and safety controls.', href: '/settings' },
      ],
    },
    {
      id: 'getstarted',
      label: 'Get Started',
      title: 'You\'re Ready to Go',
      subtitle: 'Follow these three steps to launch your first AI workforce.',
      features: [
        { icon: Building2, name: '1. Create a Company', desc: 'Start by defining your organization with a name, mission, and budget.', href: '/companies' },
        { icon: Users, name: '2. Hire Personas', desc: 'Browse the catalog and hire AI personas into your company. They\'ll appear in the org chart automatically.', href: '/personas' },
        { icon: Rocket, name: '3. Run Your First Agent', desc: 'Select an agent, give it a prompt, and watch it work in real time with streaming output.', href: '/' },
      ],
    },
  ];

  let currentStep = $derived(steps[step]);
  let isFirst = $derived(step === 0);
  let isLast = $derived(step === steps.length - 1);
  let progress = $derived(((step + 1) / steps.length) * 100);
</script>

{#if visible}
<div class="onboarding-overlay" role="dialog" aria-modal="true" aria-label="Welcome to AgentForge">
  <div class="onboarding-container">
    <!-- Header -->
    <div class="ob-header">
      <div class="ob-brand">
        <Zap size={20} />
        <span>AgentForge</span>
      </div>
      <button class="ob-close" onclick={dismiss} aria-label="Skip onboarding">
        <X size={18} />
      </button>
    </div>

    <!-- Progress bar -->
    <div class="ob-progress-track">
      <div class="ob-progress-fill" style="width: {progress}%"></div>
    </div>

    <!-- Step indicators -->
    <div class="ob-steps">
      {#each steps as s, i}
        <button
          class="ob-step-dot"
          class:active={i === step}
          class:done={i < step}
          onclick={() => (step = i)}
          aria-label="Go to {s.label}"
        >
          {#if i < step}
            <Check size={12} />
          {:else}
            <span class="ob-step-num">{i + 1}</span>
          {/if}
          <span class="ob-step-label">{s.label}</span>
        </button>
      {/each}
    </div>

    <!-- Content -->
    <div class="ob-content">
      <div class="ob-content-inner">
        {#if currentStep.id === 'welcome'}
          <div class="ob-welcome-icon">
            <Sparkles size={40} />
          </div>
        {/if}

        <h1 class="ob-title">{currentStep.title}</h1>
        <p class="ob-subtitle">{currentStep.subtitle}</p>

        <div class="ob-features" class:ob-features-2col={currentStep.features.length > 3}>
          {#each currentStep.features as feature}
            {@const Icon = feature.icon}
            <button class="ob-feature-card" onclick={() => goTo(feature.href)}>
              <div class="ob-feature-icon">
                <Icon size={22} />
              </div>
              <div class="ob-feature-text">
                <strong>{feature.name}</strong>
                <span>{feature.desc}</span>
              </div>
              <div class="ob-feature-arrow">
                <ChevronRight size={16} />
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Footer navigation -->
    <div class="ob-footer">
      <div class="ob-footer-left">
        <button class="ob-skip" onclick={dismiss}>Skip tour</button>
      </div>
      <div class="ob-footer-right">
        {#if !isFirst}
          <button class="ob-btn ob-btn-secondary" onclick={prev}>
            <ArrowLeft size={16} />
            Back
          </button>
        {/if}
        {#if isLast}
          <button class="ob-btn ob-btn-primary" onclick={dismiss}>
            <Play size={16} />
            Start Building
          </button>
        {:else}
          <button class="ob-btn ob-btn-primary" onclick={next}>
            Continue
            <ArrowRight size={16} />
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>
{/if}

<style>
  .onboarding-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(12px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    animation: ob-fade-in 300ms ease;
  }

  @keyframes ob-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .onboarding-container {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 16px;
    width: 100%;
    max-width: 52rem;
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(129, 140, 248, 0.08);
    overflow: hidden;
  }

  /* Header */
  .ob-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .ob-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--accent);
    font-weight: 700;
    font-size: 1rem;
    letter-spacing: -0.02em;
  }

  .ob-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    transition: all var(--transition);
  }

  .ob-close:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  /* Progress */
  .ob-progress-track {
    height: 2px;
    background: var(--border);
  }

  .ob-progress-fill {
    height: 100%;
    background: var(--accent);
    transition: width 300ms ease;
    border-radius: 1px;
  }

  /* Step dots */
  .ob-steps {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    padding: 1rem 1.5rem 0.5rem;
  }

  .ob-step-dot {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.65rem;
    border-radius: 20px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    font-size: 0.75rem;
    transition: all var(--transition);
  }

  .ob-step-dot:hover {
    background: var(--surface-hover);
    color: var(--text-secondary);
  }

  .ob-step-dot.active {
    background: var(--accent-muted);
    color: var(--accent);
    border-color: rgba(129, 140, 248, 0.25);
  }

  .ob-step-dot.done {
    color: var(--success);
  }

  .ob-step-dot.done :global(svg) {
    color: var(--success);
  }

  .ob-step-num {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.1rem;
    height: 1.1rem;
    border-radius: 50%;
    background: var(--border);
    font-size: 0.65rem;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .ob-step-dot.active .ob-step-num {
    background: var(--accent);
    color: var(--bg);
  }

  .ob-step-label {
    display: none;
  }

  .ob-step-dot.active .ob-step-label {
    display: inline;
    font-weight: 600;
  }

  @media (min-width: 640px) {
    .ob-step-label {
      display: inline;
    }
  }

  /* Content */
  .ob-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem 2rem 1rem;
  }

  .ob-content-inner {
    max-width: 44rem;
    margin: 0 auto;
  }

  .ob-welcome-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 4.5rem;
    height: 4.5rem;
    border-radius: 20px;
    background: linear-gradient(135deg, var(--accent-muted), rgba(129, 140, 248, 0.08));
    color: var(--accent);
    margin-bottom: 1.25rem;
    animation: ob-float 3s ease-in-out infinite;
  }

  @keyframes ob-float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-6px); }
  }

  .ob-title {
    margin: 0 0 0.5rem;
    font-size: 1.65rem;
    font-weight: 700;
    letter-spacing: -0.03em;
    color: var(--text);
    line-height: 1.2;
  }

  .ob-subtitle {
    margin: 0 0 1.75rem;
    font-size: 0.95rem;
    color: var(--muted);
    line-height: 1.6;
    max-width: 36rem;
  }

  /* Feature cards */
  .ob-features {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .ob-features.ob-features-2col {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.5rem;
  }

  @media (min-width: 640px) {
    .ob-features.ob-features-2col {
      grid-template-columns: 1fr 1fr;
    }
  }

  .ob-feature-card {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    padding: 0.85rem 1rem;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
    transition: all var(--transition);
    text-align: left;
    color: var(--text);
    font-family: inherit;
  }

  .ob-feature-card:hover {
    border-color: var(--accent);
    background: rgba(129, 140, 248, 0.04);
    box-shadow: 0 0 0 1px rgba(129, 140, 248, 0.08);
  }

  .ob-feature-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    flex-shrink: 0;
    border-radius: 10px;
    background: var(--accent-muted);
    color: var(--accent);
  }

  .ob-feature-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .ob-feature-text strong {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text);
  }

  .ob-feature-text span {
    font-size: 0.8rem;
    color: var(--muted);
    line-height: 1.4;
  }

  .ob-feature-arrow {
    flex-shrink: 0;
    color: var(--muted);
    opacity: 0;
    transform: translateX(-4px);
    transition: all var(--transition);
  }

  .ob-feature-card:hover .ob-feature-arrow {
    opacity: 1;
    transform: translateX(0);
    color: var(--accent);
  }

  /* Footer */
  .ob-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border);
    gap: 1rem;
  }

  .ob-footer-left {
    flex-shrink: 0;
  }

  .ob-footer-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .ob-skip {
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

  .ob-skip:hover {
    color: var(--text-secondary);
    background: var(--surface-hover);
  }

  .ob-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: all var(--transition);
    font-family: inherit;
  }

  .ob-btn-primary {
    background: var(--accent);
    color: var(--bg);
  }

  .ob-btn-primary:hover {
    background: var(--accent-hover);
  }

  .ob-btn-secondary {
    background: var(--surface-hover);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .ob-btn-secondary:hover {
    border-color: var(--border-hover);
    color: var(--text);
  }
</style>
