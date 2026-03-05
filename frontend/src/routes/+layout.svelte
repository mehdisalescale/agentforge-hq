<script lang="ts">
  import '../app.css';
</script>

<div class="app">
  <aside class="sidebar">
    <nav class="nav">
      <a class="brand" href="/">Claude Forge</a>
      <a class="link" href="/">Run</a>
      <a class="link" href="/agents">Agents</a>
      <a class="link" href="/sessions">Sessions</a>
      <a class="link" href="/workflows">Workflows</a>
      <a class="link" href="/skills">Skills</a>
      <a class="link" href="/memory">Memory</a>
      <a class="link" href="/hooks">Hooks</a>
      <a class="link" href="/schedules">Schedules</a>
      <a class="link" href="/analytics">Analytics</a>
      <a class="link" href="/settings">Settings</a>
    </nav>
  </aside>
  <main class="main">
    <slot />
  </main>
  <footer class="statusbar">
    <span>v0.5.0-dev</span>
    <span class="statusbar-note">Multi-agent orchestrator</span>
  </footer>
</div>
