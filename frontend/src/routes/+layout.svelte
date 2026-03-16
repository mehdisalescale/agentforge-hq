<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import {
    Home, Bot, History, Zap,
    Building2, Users, Network, Target, ShieldCheck,
    Puzzle, Server,
    BarChart3, Settings,
    Wifi, WifiOff,
    Menu, X
  } from 'lucide-svelte';

  let sidebarOpen = $state(false);

  function toggleSidebar() {
    sidebarOpen = !sidebarOpen;
  }

  let healthWarning = $state<string | null>(null);
  let wsConnected = $state(false);
  let wsRetryCount = $state(0);

  function connectWs() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/v1/ws`;

    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      wsConnected = true;
      wsRetryCount = 0;
    };

    ws.onclose = () => {
      wsConnected = false;
      const delay = Math.min(1000 * Math.pow(2, wsRetryCount), 30000);
      wsRetryCount++;
      setTimeout(connectWs, delay);
    };

    ws.onerror = () => {
      ws.close();
    };
  }

  onMount(async () => {
    connectWs();
    try {
      const res = await fetch('/api/v1/health');
      if (res.ok) {
        const health = await res.json();
        if (!health.cli_available) {
          healthWarning = `CLI "${health.cli_command}" not found. Agent runs will fail. Install it or set FORGE_CLI_COMMAND.`;
        }
      }
    } catch { /* server not reachable */ }
  });

  interface NavLink {
    href: string;
    text: string;
    icon: typeof Home;
  }
  interface NavGroup {
    label: string;
    links: NavLink[];
  }

  const primaryLinks: NavLink[] = [
    { href: '/', text: 'Home', icon: Home },
    { href: '/agents', text: 'Agents', icon: Bot },
    { href: '/sessions', text: 'Sessions', icon: History },
  ];

  const navGroups: NavGroup[] = [
    {
      label: 'Organization',
      links: [
        { href: '/companies', text: 'Companies', icon: Building2 },
        { href: '/personas', text: 'Personas', icon: Users },
        { href: '/org-chart', text: 'Org Chart', icon: Network },
      ]
    },
    {
      label: 'Governance',
      links: [
        { href: '/goals', text: 'Goals', icon: Target },
        { href: '/approvals', text: 'Approvals', icon: ShieldCheck },
      ]
    },
  ];

  const utilityLinks: NavLink[] = [
    { href: '/skills', text: 'Skills', icon: Puzzle },
    { href: '/backends', text: 'Backends', icon: Server },
    { href: '/analytics', text: 'Analytics', icon: BarChart3 },
    { href: '/settings', text: 'Settings', icon: Settings },
  ];

  function isActive(href: string, pathname: string): boolean {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  }

  // Close sidebar on navigation
  $effect(() => {
    const _ = $page.url.pathname;
    sidebarOpen = false;
  });
</script>

{#if healthWarning}
  <div class="health-banner">
    <span>Warning: {healthWarning}</span>
    <button onclick={() => healthWarning = null}>Dismiss</button>
  </div>
{/if}

<button class="hamburger" onclick={toggleSidebar} aria-label="Toggle navigation">
  {#if sidebarOpen}
    <X size={20} />
  {:else}
    <Menu size={20} />
  {/if}
</button>

{#if sidebarOpen}
  <div class="sidebar-overlay" onclick={() => sidebarOpen = false} role="presentation"></div>
{/if}

<div class="app">
  <aside class="sidebar" class:open={sidebarOpen}>
    <nav class="nav" aria-label="Main navigation">
      <a class="brand" href="/">
        <Zap size={18} />
        AgentForge
      </a>

      <div class="nav-group">
        {#each primaryLinks as link}
          {@const Icon = link.icon}
          <a
            class="link"
            class:active={isActive(link.href, $page.url.pathname)}
            href={link.href}
            aria-current={isActive(link.href, $page.url.pathname) ? 'page' : undefined}
          >
            <Icon size={16} />
            {link.text}
          </a>
        {/each}
      </div>

      {#each navGroups as group}
        <div class="nav-group">
          <span class="nav-label">{group.label}</span>
          {#each group.links as link}
            {@const Icon = link.icon}
            <a
              class="link"
              class:active={isActive(link.href, $page.url.pathname)}
              href={link.href}
            >
              <Icon size={16} />
              {link.text}
            </a>
          {/each}
        </div>
      {/each}

      <div class="nav-divider"></div>
      <div class="nav-group">
        {#each utilityLinks as link}
          {@const Icon = link.icon}
          <a
            class="link"
            class:active={isActive(link.href, $page.url.pathname)}
            href={link.href}
            aria-current={isActive(link.href, $page.url.pathname) ? 'page' : undefined}
          >
            <Icon size={16} />
            {link.text}
          </a>
        {/each}
      </div>
    </nav>
  </aside>
  <main class="main" id="main-content">
    <slot />
  </main>
  <footer class="statusbar" role="status">
    <span>v0.6.0-dev</span>
    <span class="ws-status" class:connected={wsConnected} title={wsConnected ? 'Connected to event stream' : 'Disconnected — reconnecting...'}>
      {#if wsConnected}
        <Wifi size={12} />
      {:else}
        <WifiOff size={12} />
      {/if}
    </span>
    <span class="statusbar-note">AI workforce platform</span>
  </footer>
</div>

<style>
  .health-banner {
    background: #7c2d12;
    color: #fed7aa;
    padding: 0.5rem 1rem;
    font-size: 0.8rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .health-banner button {
    background: transparent;
    border: 1px solid #fed7aa;
    color: #fed7aa;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }
  .health-banner button:hover {
    background: rgba(254, 215, 170, 0.15);
  }
  .ws-status {
    display: flex;
    align-items: center;
    color: var(--danger);
    opacity: 0.7;
  }

  .ws-status.connected {
    color: var(--success);
  }

  .nav-divider {
    height: 1px;
    background: var(--border);
    margin: 0.25rem 0.75rem;
    opacity: 0.5;
  }
</style>
