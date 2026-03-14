<script lang="ts">
  import '../app.css';
</script>

<div class="app">
  <aside class="sidebar">
    <nav class="nav">
      <a class="brand" href="/">AgentForge</a>

      <div class="nav-group">
        <span class="nav-label">Workspace</span>
        <a class="link" href="/">Run</a>
        <a class="link" href="/agents">Agents</a>
        <a class="link" href="/sessions">Sessions</a>
        <a class="link" href="/workflows">Workflows</a>
      </div>

      <div class="nav-group">
        <span class="nav-label">Organization</span>
        <a class="link" href="/companies">Companies</a>
        <a class="link" href="/personas">Personas</a>
        <a class="link" href="/org-chart">Org Chart</a>
        <a class="link" href="/goals">Goals</a>
        <a class="link" href="/approvals">Approvals</a>
      </div>

      <div class="nav-group">
        <span class="nav-label">Configuration</span>
        <a class="link" href="/skills">Skills</a>
        <a class="link" href="/memory">Memory</a>
        <a class="link" href="/hooks">Hooks</a>
        <a class="link" href="/schedules">Schedules</a>
      </div>

      <div class="nav-group">
        <span class="nav-label">Insights</span>
        <a class="link" href="/analytics">Analytics</a>
        <a class="link" href="/settings">Settings</a>
      </div>
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
