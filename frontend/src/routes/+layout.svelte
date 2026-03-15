<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import Onboarding from '$lib/components/Onboarding.svelte';
  import {
    Zap, Bot, History,
    Building2, Users, Network, Target, ShieldCheck,
    Puzzle,
    BarChart3, Settings
  } from 'lucide-svelte';

  const navGroups = [
    {
      label: 'Workspace',
      links: [
        { href: '/', text: 'Run', icon: Zap },
        { href: '/agents', text: 'Agents', icon: Bot },
        { href: '/sessions', text: 'Sessions', icon: History },
      ]
    },
    {
      label: 'Organization',
      links: [
        { href: '/companies', text: 'Companies', icon: Building2 },
        { href: '/personas', text: 'Personas', icon: Users },
        { href: '/org-chart', text: 'Org Chart', icon: Network },
        { href: '/goals', text: 'Goals', icon: Target },
        { href: '/approvals', text: 'Approvals', icon: ShieldCheck },
      ]
    },
    {
      label: 'Configuration',
      links: [
        { href: '/skills', text: 'Skills', icon: Puzzle },
      ]
    },
    {
      label: 'Insights',
      links: [
        { href: '/analytics', text: 'Analytics', icon: BarChart3 },
        { href: '/settings', text: 'Settings', icon: Settings },
      ]
    },
  ];

  function isActive(href: string, pathname: string): boolean {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  }
</script>

<Onboarding />

<div class="app">
  <aside class="sidebar">
    <nav class="nav">
      <a class="brand" href="/">
        <Zap size={18} />
        AgentForge
      </a>

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
    </nav>
  </aside>
  <main class="main">
    <slot />
  </main>
  <footer class="statusbar">
    <span>v0.6.0-dev</span>
    <span class="statusbar-note">AI workforce platform</span>
  </footer>
</div>
