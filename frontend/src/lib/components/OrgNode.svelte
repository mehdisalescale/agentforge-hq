<script lang="ts">
  import type { OrgChartNode, Department } from '$lib/api';
  import OrgNode from '$lib/components/OrgNode.svelte';

  let { node, departments, depth = 0, maxDepth = 50 }: {
    node: OrgChartNode;
    departments: Department[];
    depth?: number;
    maxDepth?: number;
  } = $props();

  function deptName(id: string | null | undefined): string {
    if (!id) return '—';
    const d = departments.find(x => x.id === id);
    return d ? d.name : id.slice(0, 8) + '...';
  }

  // eslint-disable-next-line -- depth is intentionally captured once at mount
  let collapsed = $state(depth > 2); // depth doesn't change for a given node instance
  let hasChildren = $derived(node.children && node.children.length > 0);
</script>

<div class="org-node" style="--depth: {depth}">
  <div class="org-card">
    <div class="card-top">
      {#if hasChildren}
        <button class="toggle" onclick={() => collapsed = !collapsed} aria-label={collapsed ? 'Expand' : 'Collapse'}>
          {collapsed ? '+' : '−'}
        </button>
      {/if}
      <div class="title">{node.position.title ?? node.position.role}</div>
    </div>
    <div class="meta">
      <span class="meta-label">Dept:</span>
      <span>{deptName(node.position.department_id ?? undefined)}</span>
    </div>
  </div>

  {#if hasChildren && !collapsed}
    <div class="children">
      {#if maxDepth <= 0}
        <span class="depth-limit">...</span>
      {:else}
        {#each node.children as child (child.position.id)}
          <OrgNode node={child} {departments} depth={depth + 1} maxDepth={maxDepth - 1} />
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .org-node {
    position: relative;
    padding-left: 1.25rem;
    margin: 0.5rem 0;
  }

  .org-node::before {
    content: '';
    position: absolute;
    left: 0.5rem;
    top: 0;
    bottom: 0;
    border-left: 1px solid var(--border);
  }

  .org-card {
    position: relative;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    min-width: 10rem;
  }

  .org-card::before {
    content: '';
    position: absolute;
    left: -0.75rem;
    top: 50%;
    width: 0.75rem;
    border-top: 1px solid var(--border);
  }

  .card-top {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .toggle {
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--muted);
    width: 1.2rem;
    height: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 0.8rem;
    flex-shrink: 0;
  }

  .toggle:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .title {
    font-size: 0.9rem;
    font-weight: 600;
  }

  .meta {
    display: flex;
    gap: 0.25rem;
    font-size: 0.75rem;
    color: var(--muted);
    margin-top: 0.15rem;
  }

  .meta-label {
    opacity: 0.7;
  }

  .children {
    margin-left: 1.25rem;
    margin-top: 0.25rem;
  }

  .depth-limit {
    display: block;
    color: var(--muted);
    font-size: 0.85rem;
    padding: 0.25rem 0;
  }
</style>
