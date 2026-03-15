<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import {
    listCompanies,
    listApprovals,
    updateApprovalStatus,
    type Company,
    type Approval,
    type ApprovalStatus,
  } from '$lib/api';
  import { ShieldCheck } from 'lucide-svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import ErrorMessage from '$lib/components/ErrorMessage.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  setContext('pageTitle', 'Approvals');

  let companies = $state<Company[]>([]);
  let selectedCompanyId = $state<string>('');
  let approvals = $state<Approval[]>([]);
  let statusFilter = $state<ApprovalStatus | 'all'>('pending');
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function loadInitial() {
    loading = true;
    error = null;
    try {
      companies = await listCompanies();
      if (companies.length > 0) {
        selectedCompanyId = companies[0].id;
        await reloadApprovals();
      } else {
        approvals = [];
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function changeCompany(id: string) {
    selectedCompanyId = id;
    await reloadApprovals();
  }

  async function reloadApprovals() {
    if (!selectedCompanyId) {
      approvals = [];
      return;
    }
    loading = true;
    error = null;
    try {
      const status = statusFilter === 'all' ? undefined : statusFilter;
      approvals = await listApprovals(selectedCompanyId, status as ApprovalStatus | undefined);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      approvals = [];
    } finally {
      loading = false;
    }
  }

  async function onFilterChange(val: string) {
    statusFilter = val as ApprovalStatus | 'all';
    await reloadApprovals();
  }

  async function decide(approval: Approval, status: ApprovalStatus) {
    try {
      const updated = await updateApprovalStatus(approval.id, {
        status,
        approver: 'UI-approver',
      });
      approvals = approvals.map((a) => (a.id === updated.id ? updated : a));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(() => {
    loadInitial();
  });
</script>

<svelte:head>
  <title>Approvals - AgentForge</title>
</svelte:head>

<div class="approvals-page">
  <header class="page-header">
    <h1>Approvals</h1>
    <div class="toolbar">
      <label class="company-select">
        <span>Company</span>
        <select
          bind:value={selectedCompanyId}
          onchange={(e) => changeCompany((e.target as HTMLSelectElement).value)}
        >
          {#if companies.length === 0}
            <option value="">No companies yet</option>
          {:else}
            {#each companies as c}
              <option value={c.id}>{c.name}</option>
            {/each}
          {/if}
        </select>
      </label>
      <label class="status-filter">
        <span>Status</span>
        <select bind:value={statusFilter} onchange={(e) => onFilterChange((e.target as HTMLSelectElement).value)}>
          <option value="pending">Pending</option>
          <option value="approved">Approved</option>
          <option value="rejected">Rejected</option>
          <option value="all">All</option>
        </select>
      </label>
    </div>
  </header>

  {#if error}
    <ErrorMessage message={error} onretry={loadInitial} />
  {/if}

  {#if loading}
    <Skeleton type="table" lines={4} />
  {:else if !selectedCompanyId}
    <p class="muted">Select a company to see its approvals.</p>
  {:else if approvals.length === 0}
    <EmptyState
      icon={ShieldCheck}
      title="No pending approvals"
      description="Approval requests from agents will appear here for review."
    />
  {:else}
    <table class="table">
      <thead>
        <tr>
          <th>Type</th>
          <th>Status</th>
          <th>Requester</th>
          <th>Approver</th>
          <th>Data</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each approvals as a}
          <tr>
            <td>{a.approval_type}</td>
            <td>
              <span class={`badge badge-${a.status}`}>{a.status}</span>
            </td>
            <td>{a.requester}</td>
            <td>{a.approver ?? '—'}</td>
            <td class="muted code">
              {#if a.data_json}
                {a.data_json.slice(0, 80)}{a.data_json.length > 80 ? '…' : ''}
              {:else}
                —
              {/if}
            </td>
            <td>
              {#if a.status === 'pending'}
                <div class="actions">
                  <button class="btn btn-small" type="button" onclick={() => decide(a, 'approved')}>
                    Approve
                  </button>
                  <button class="btn btn-small btn-ghost" type="button" onclick={() => decide(a, 'rejected')}>
                    Reject
                  </button>
                </div>
              {:else}
                <span class="muted small">No actions</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .approvals-page {
    max-width: 64rem;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .toolbar {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .company-select,
  .status-filter {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .company-select select,
  .status-filter select {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    font-size: 0.9rem;
  }

  .btn {
    padding: 0.45rem 0.9rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.9rem;
  }

  .btn-ghost {
    background: transparent;
    border-color: transparent;
  }

  .btn-small {
    padding: 0.3rem 0.6rem;
    font-size: 0.8rem;
  }

  .muted {
    color: var(--muted);
  }

  .small {
    font-size: 0.8rem;
  }

  .table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .table th,
  .table td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    text-align: left;
  }

  .table th {
    font-weight: 500;
    color: var(--muted);
  }

  .code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New',
      monospace;
    font-size: 0.8rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    font-size: 0.75rem;
    text-transform: capitalize;
  }

  .badge-pending {
    background: rgba(234, 179, 8, 0.15);
    color: #facc15;
    border: 1px solid rgba(234, 179, 8, 0.4);
  }

  .badge-approved {
    background: rgba(34, 197, 94, 0.15);
    color: #bbf7d0;
    border: 1px solid rgba(34, 197, 94, 0.4);
  }

  .badge-rejected {
    background: rgba(239, 68, 68, 0.15);
    color: #fecaca;
    border: 1px solid rgba(239, 68, 68, 0.4);
  }

  .message.error {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }
</style>

