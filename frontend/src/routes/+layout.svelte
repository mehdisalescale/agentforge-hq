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
      <a class="link" href="/settings">Settings</a>
    </nav>
  </aside>
  <main class="main">
    <slot />
  </main>
  <footer class="statusbar">
    <span>Phase 1</span>
    <span class="statusbar-note">Run + Sessions UI (Agent F)</span>
  </footer>
</div>
