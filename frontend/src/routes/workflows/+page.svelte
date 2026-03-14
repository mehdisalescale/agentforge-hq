<script lang="ts">
  import { setContext } from 'svelte';
  import { onMount } from 'svelte';
  import {
    listWorkflows,
    listAgents,
    createWorkflow,
    updateWorkflow,
    deleteWorkflow,
    runWorkflow,
    type Workflow,
    type Agent,
  } from '$lib/api';

  setContext('pageTitle', 'Workflows');

  // --- Types for pipeline definition ---
  interface SequentialStep {
    type: 'Sequential';
    agent_id: string;
    prompt_template: string;
  }

  interface FanoutStep {
    type: 'Fanout';
    agent_ids: string[];
    prompt_template: string;
  }

  type PipelineStep = SequentialStep | FanoutStep;

  // --- State ---
  let workflows = $state<Workflow[]>([]);
  let agents = $state<Agent[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Editor state
  let editorOpen = $state<'create' | 'edit' | null>(null);
  let editId = $state<string | null>(null);
  let formName = $state('');
  let formDescription = $state('');
  let formSteps = $state<PipelineStep[]>([]);
  let formError = $state<string | null>(null);
  let submitting = $state(false);

  // Run state
  let runModalId = $state<string | null>(null);
  let runPrompt = $state('');
  let runDir = $state('');
  let runError = $state<string | null>(null);
  let runResult = $state<string | null>(null);
  let runSubmitting = $state(false);

  // Delete confirm
  let deleteConfirmId = $state<string | null>(null);

  // --- Helpers ---

  function parseSteps(w: Workflow): PipelineStep[] {
    try {
      const def = JSON.parse(w.definition_json);
      if (!Array.isArray(def?.steps)) return [];
      return def.steps.map((s: Record<string, unknown>) => {
        if (s.Sequential) {
          const seq = s.Sequential as Record<string, unknown>;
          return {
            type: 'Sequential' as const,
            agent_id: (seq.agent_id as string) ?? '',
            prompt_template: (seq.prompt_template as string) ?? '',
          };
        }
        if (s.Fanout) {
          const fan = s.Fanout as Record<string, unknown>;
          return {
            type: 'Fanout' as const,
            agent_ids: Array.isArray(fan.agent_ids) ? (fan.agent_ids as string[]) : [],
            prompt_template: (fan.prompt_template as string) ?? '',
          };
        }
        return { type: 'Sequential' as const, agent_id: '', prompt_template: '' };
      });
    } catch {
      return [];
    }
  }

  function stepsToJson(steps: PipelineStep[]): string {
    const mapped = steps.map((s) => {
      if (s.type === 'Sequential') {
        return { Sequential: { agent_id: s.agent_id, prompt_template: s.prompt_template } };
      }
      return { Fanout: { agent_ids: s.agent_ids, prompt_template: s.prompt_template } };
    });
    return JSON.stringify({ steps: mapped });
  }

  function getAgentName(id: string): string {
    const a = agents.find((x) => x.id === id);
    return a ? a.name : id.slice(0, 8) + '...';
  }

  function stepLabel(step: PipelineStep): string {
    if (step.type === 'Sequential') {
      return getAgentName(step.agent_id);
    }
    if (step.agent_ids.length === 0) return 'Fanout (none)';
    return step.agent_ids.map(getAgentName).join(' | ');
  }

  // --- Data loading ---

  async function loadData() {
    loading = true;
    error = null;
    try {
      const [wf, ag] = await Promise.all([listWorkflows(), listAgents()]);
      workflows = wf;
      agents = ag;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      workflows = [];
    } finally {
      loading = false;
    }
  }

  // --- Editor ---

  function openCreate() {
    editId = null;
    formName = '';
    formDescription = '';
    formSteps = [];
    formError = null;
    editorOpen = 'create';
  }

  function openEdit(w: Workflow) {
    editId = w.id;
    formName = w.name;
    formDescription = w.description ?? '';
    formSteps = parseSteps(w);
    formError = null;
    editorOpen = 'edit';
  }

  function closeEditor() {
    editorOpen = null;
    editId = null;
    formError = null;
  }

  function addStep(type: 'Sequential' | 'Fanout') {
    if (type === 'Sequential') {
      formSteps = [...formSteps, { type: 'Sequential', agent_id: agents[0]?.id ?? '', prompt_template: '{{input}}' }];
    } else {
      formSteps = [...formSteps, { type: 'Fanout', agent_ids: agents[0] ? [agents[0].id] : [], prompt_template: '{{input}}' }];
    }
  }

  function removeStep(index: number) {
    formSteps = formSteps.filter((_, i) => i !== index);
  }

  function moveStep(index: number, direction: -1 | 1) {
    const newIndex = index + direction;
    if (newIndex < 0 || newIndex >= formSteps.length) return;
    const copy = [...formSteps];
    const tmp = copy[index];
    copy[index] = copy[newIndex];
    copy[newIndex] = tmp;
    formSteps = copy;
  }

  function addFanoutAgent(stepIndex: number) {
    const step = formSteps[stepIndex];
    if (step.type !== 'Fanout') return;
    step.agent_ids = [...step.agent_ids, agents[0]?.id ?? ''];
    formSteps = formSteps;
  }

  function removeFanoutAgent(stepIndex: number, agentIndex: number) {
    const step = formSteps[stepIndex];
    if (step.type !== 'Fanout') return;
    step.agent_ids = step.agent_ids.filter((_, i) => i !== agentIndex);
    formSteps = formSteps;
  }

  async function submitForm() {
    if (!formName.trim()) {
      formError = 'Name is required';
      return;
    }
    if (formSteps.length === 0) {
      formError = 'Add at least one step';
      return;
    }
    submitting = true;
    formError = null;
    try {
      const payload = {
        name: formName.trim(),
        description: formDescription.trim() || null,
        definition_json: stepsToJson(formSteps),
      };
      if (editorOpen === 'create') {
        await createWorkflow(payload);
      } else if (editId) {
        await updateWorkflow(editId, payload);
      }
      closeEditor();
      await loadData();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  // --- Delete ---

  async function doDelete(id: string) {
    try {
      await deleteWorkflow(id);
      deleteConfirmId = null;
      await loadData();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  // --- Run ---

  function openRun(id: string) {
    runModalId = id;
    runPrompt = '';
    runDir = '';
    runError = null;
    runResult = null;
  }

  function closeRun() {
    runModalId = null;
    runError = null;
    runResult = null;
  }

  async function submitRun() {
    if (!runModalId || !runPrompt.trim()) {
      runError = 'Prompt is required';
      return;
    }
    runSubmitting = true;
    runError = null;
    runResult = null;
    try {
      const res = await runWorkflow(runModalId, runPrompt.trim(), runDir.trim() || undefined);
      runResult = `Started session: ${res.session_id}`;
    } catch (e) {
      runError = e instanceof Error ? e.message : String(e);
    } finally {
      runSubmitting = false;
    }
  }

  onMount(() => {
    loadData();
  });
</script>

<svelte:head>
  <title>Workflows - AgentForge</title>
</svelte:head>

<div class="workflows-page">
  <header class="page-header">
    <h1>Workflows</h1>
    <button class="btn btn-primary" onclick={openCreate}>New workflow</button>
  </header>

  {#if error}
    <div class="message error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading workflows...</p>
  {:else if workflows.length === 0 && !editorOpen}
    <div class="empty-state">
      <div class="workflow-placeholder">
        <div class="wf-diagram">
          <div class="wf-node wf-start">Start</div>
          <div class="wf-arrow"></div>
          <div class="wf-node wf-step">Agent A</div>
          <div class="wf-arrow"></div>
          <div class="wf-node wf-step">Agent B</div>
          <div class="wf-arrow"></div>
          <div class="wf-node wf-end">Done</div>
        </div>
      </div>
      <h2 class="empty-title">No workflows yet</h2>
      <p class="muted">Workflows are sequences of agent tasks. Define steps, assign agents, and let Forge orchestrate the pipeline.</p>
      <button class="btn btn-primary" style="margin-top: 1rem;" onclick={openCreate}>Create workflow</button>
    </div>
  {:else}
    <div class="workflow-cards">
      {#each workflows as w (w.id)}
        {@const steps = parseSteps(w)}
        <article class="card">
          <div class="card-header">
            <h2 class="card-title">{w.name}</h2>
            {#if steps.length > 0}
              <span class="step-count">{steps.length} step{steps.length !== 1 ? 's' : ''}</span>
            {/if}
          </div>
          {#if w.description}
            <p class="card-desc">{w.description}</p>
          {/if}
          {#if steps.length > 0}
            <div class="wf-steps-inline">
              {#each steps as step, i}
                {#if i > 0}
                  <span class="wf-arrow-inline"></span>
                {/if}
                <span class="wf-step-chip" class:wf-step-fanout={step.type === 'Fanout'}>
                  {#if step.type === 'Fanout'}
                    <span class="step-type-icon" title="Fanout">F</span>
                  {/if}
                  {stepLabel(step)}
                </span>
              {/each}
            </div>
          {/if}
          <div class="card-actions">
            <button class="btn btn-ghost" onclick={() => openRun(w.id)}>Run</button>
            <button class="btn btn-ghost" onclick={() => openEdit(w)}>Edit</button>
            <button class="btn btn-ghost danger" onclick={() => (deleteConfirmId = w.id)}>Delete</button>
          </div>
          {#if deleteConfirmId === w.id}
            <div class="delete-confirm">
              <span>Delete this workflow?</span>
              <button class="btn btn-ghost" onclick={() => (deleteConfirmId = null)}>Cancel</button>
              <button class="btn danger" onclick={() => doDelete(w.id)}>Delete</button>
            </div>
          {/if}
          <div class="card-footer">
            <span class="card-meta">Created {new Date(w.created_at).toLocaleDateString()}</span>
            {#if w.updated_at !== w.created_at}
              <span class="card-meta">Updated {new Date(w.updated_at).toLocaleDateString()}</span>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {/if}

  <!-- Editor Modal -->
  {#if editorOpen}
    <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="editor-title">
      <div class="modal modal-wide">
        <h2 id="editor-title">{editorOpen === 'create' ? 'Create workflow' : 'Edit workflow'}</h2>
        {#if formError}
          <div class="message error">{formError}</div>
        {/if}
        <form
          class="wf-form"
          onsubmit={(e) => { e.preventDefault(); submitForm(); }}
        >
          <label>
            <span>Name</span>
            <input type="text" bind:value={formName} required placeholder="Workflow name" />
          </label>
          <label>
            <span>Description</span>
            <input type="text" bind:value={formDescription} placeholder="Optional description" />
          </label>

          <div class="steps-section">
            <div class="steps-header">
              <span class="steps-title">Steps ({formSteps.length})</span>
              <div class="steps-add-btns">
                <button type="button" class="btn btn-sm" onclick={() => addStep('Sequential')}>+ Sequential</button>
                <button type="button" class="btn btn-sm" onclick={() => addStep('Fanout')}>+ Fanout</button>
              </div>
            </div>

            {#if formSteps.length === 0}
              <p class="muted steps-empty">No steps yet. Add a Sequential or Fanout step above.</p>
            {:else}
              <div class="steps-list">
                {#each formSteps as step, i}
                  <div class="step-card">
                    <div class="step-card-header">
                      <span class="step-type-badge" class:fanout={step.type === 'Fanout'}>
                        {step.type === 'Sequential' ? 'SEQ' : 'FAN'}
                      </span>
                      <span class="step-number">Step {i + 1}</span>
                      <div class="step-card-actions">
                        <button type="button" class="btn btn-sm btn-ghost" onclick={() => moveStep(i, -1)} disabled={i === 0} title="Move up">Up</button>
                        <button type="button" class="btn btn-sm btn-ghost" onclick={() => moveStep(i, 1)} disabled={i === formSteps.length - 1} title="Move down">Dn</button>
                        <button type="button" class="btn btn-sm btn-ghost danger" onclick={() => removeStep(i)} title="Remove step">X</button>
                      </div>
                    </div>

                    {#if step.type === 'Sequential'}
                      <label class="step-field">
                        <span>Agent</span>
                        <select bind:value={step.agent_id}>
                          {#each agents as a}
                            <option value={a.id}>{a.name}</option>
                          {/each}
                          {#if agents.length === 0}
                            <option value="" disabled>No agents available</option>
                          {/if}
                        </select>
                      </label>
                    {:else}
                      <div class="fanout-agents">
                        <span class="step-field-label">Agents</span>
                        {#each step.agent_ids as _aid, ai}
                          <div class="fanout-agent-row">
                            <select bind:value={step.agent_ids[ai]}>
                              {#each agents as a}
                                <option value={a.id}>{a.name}</option>
                              {/each}
                            </select>
                            <button type="button" class="btn btn-sm btn-ghost danger" onclick={() => removeFanoutAgent(i, ai)}>X</button>
                          </div>
                        {/each}
                        <button type="button" class="btn btn-sm" onclick={() => addFanoutAgent(i)}>+ Agent</button>
                      </div>
                    {/if}

                    <label class="step-field">
                      <span>Prompt Template</span>
                      <textarea bind:value={step.prompt_template} rows="2" placeholder="Use {{input}} for previous output"></textarea>
                    </label>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <div class="form-actions">
            <button type="button" class="btn btn-ghost" onclick={closeEditor}>Cancel</button>
            <button type="submit" class="btn btn-primary" disabled={submitting}>
              {submitting ? 'Saving...' : editorOpen === 'create' ? 'Create' : 'Save'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Run Modal -->
  {#if runModalId}
    <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="run-title">
      <div class="modal">
        <h2 id="run-title">Run Workflow</h2>
        {#if runError}
          <div class="message error">{runError}</div>
        {/if}
        {#if runResult}
          <div class="message success">{runResult}</div>
        {/if}
        <form
          class="wf-form"
          onsubmit={(e) => { e.preventDefault(); submitRun(); }}
        >
          <label>
            <span>Prompt</span>
            <textarea bind:value={runPrompt} rows="3" required placeholder="Enter workflow input prompt..."></textarea>
          </label>
          <label>
            <span>Working Directory <span class="optional">(optional)</span></span>
            <input type="text" bind:value={runDir} placeholder="/path/to/project" />
          </label>
          <div class="form-actions">
            <button type="button" class="btn btn-ghost" onclick={closeRun}>Close</button>
            <button type="submit" class="btn btn-primary" disabled={runSubmitting}>
              {runSubmitting ? 'Running...' : 'Run'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .workflows-page {
    max-width: 56rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .page-header h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
  }

  .empty-title {
    margin: 1.5rem 0 0.5rem 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  /* Placeholder workflow diagram */
  .workflow-placeholder {
    padding: 1.5rem;
    border: 1px dashed var(--border);
    border-radius: 8px;
    background: rgba(167, 139, 250, 0.03);
  }

  .wf-diagram {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    flex-wrap: wrap;
  }

  .wf-node {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 500;
    white-space: nowrap;
  }

  .wf-start {
    background: rgba(134, 239, 172, 0.15);
    color: #86efac;
    border: 1px solid rgba(134, 239, 172, 0.3);
  }

  .wf-step {
    background: rgba(167, 139, 250, 0.15);
    color: var(--accent);
    border: 1px solid rgba(167, 139, 250, 0.3);
  }

  .wf-end {
    background: rgba(251, 191, 36, 0.15);
    color: #fbbf24;
    border: 1px solid rgba(251, 191, 36, 0.3);
  }

  .wf-arrow {
    width: 2rem;
    height: 2px;
    background: var(--border);
    position: relative;
    margin: 0 0.25rem;
  }

  .wf-arrow::after {
    content: '';
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    border-left: 6px solid var(--border);
    border-top: 4px solid transparent;
    border-bottom: 4px solid transparent;
  }

  /* Workflow cards */
  .workflow-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .card-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  .card-title {
    margin: 0;
    font-size: 1.1rem;
    flex: 1 1 auto;
  }

  .step-count {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .card-desc {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
    line-height: 1.4;
  }

  .card-actions {
    display: flex;
    gap: 0.5rem;
  }

  .card-footer {
    border-top: 1px solid var(--border);
    padding-top: 0.5rem;
    display: flex;
    gap: 1rem;
  }

  .card-meta {
    font-size: 0.75rem;
    color: var(--muted);
  }

  /* Inline step visualization */
  .wf-steps-inline {
    display: flex;
    align-items: center;
    gap: 0;
    flex-wrap: wrap;
    padding: 0.5rem 0;
  }

  .wf-step-chip {
    font-size: 0.75rem;
    padding: 0.2rem 0.6rem;
    border-radius: 4px;
    background: rgba(167, 139, 250, 0.15);
    color: var(--accent);
    border: 1px solid rgba(167, 139, 250, 0.25);
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .wf-step-fanout {
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
    border-color: rgba(59, 130, 246, 0.25);
  }

  .step-type-icon {
    font-weight: 700;
    font-size: 0.65rem;
    opacity: 0.7;
  }

  .wf-arrow-inline {
    display: inline-block;
    width: 1.25rem;
    height: 2px;
    background: var(--border);
    position: relative;
    margin: 0 0.15rem;
    flex-shrink: 0;
  }

  .wf-arrow-inline::after {
    content: '';
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    border-left: 5px solid var(--border);
    border-top: 3px solid transparent;
    border-bottom: 3px solid transparent;
  }

  /* Delete confirm */
  .delete-confirm {
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    font-size: 0.85rem;
  }

  /* Modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }

  .modal {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    max-width: 28rem;
    width: 100%;
    max-height: 90vh;
    overflow: auto;
  }

  .modal-wide {
    max-width: 40rem;
  }

  .modal h2 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
  }

  /* Form */
  .wf-form label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.75rem;
  }

  .wf-form label span {
    font-size: 0.9rem;
    color: var(--muted);
  }

  .wf-form input[type="text"],
  .wf-form select,
  .wf-form textarea {
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.9rem;
    font-family: inherit;
  }

  .wf-form textarea {
    resize: vertical;
    min-height: 3rem;
  }

  .optional {
    font-weight: 400;
    color: var(--muted);
    font-size: 0.8rem;
  }

  /* Steps section */
  .steps-section {
    margin: 0.5rem 0;
  }

  .steps-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .steps-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text);
  }

  .steps-add-btns {
    display: flex;
    gap: 0.35rem;
  }

  .steps-empty {
    font-size: 0.85rem;
    text-align: center;
    padding: 1rem 0;
  }

  .steps-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .step-card {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.75rem;
  }

  .step-card-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .step-type-badge {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    background: rgba(167, 139, 250, 0.2);
    color: var(--accent);
  }

  .step-type-badge.fanout {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
  }

  .step-number {
    font-size: 0.8rem;
    color: var(--muted);
    flex: 1;
  }

  .step-card-actions {
    display: flex;
    gap: 0.25rem;
  }

  .step-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 0.5rem;
  }

  .step-field span {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .step-field select,
  .step-field textarea {
    padding: 0.4rem 0.6rem;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    font-size: 0.85rem;
    font-family: inherit;
  }

  .step-field textarea {
    resize: vertical;
    min-height: 2.5rem;
  }

  .step-field-label {
    font-size: 0.8rem;
    color: var(--muted);
    margin-bottom: 0.25rem;
  }

  /* Fanout agent list */
  .fanout-agents {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.5rem;
  }

  .fanout-agent-row {
    display: flex;
    gap: 0.35rem;
    align-items: center;
  }

  .fanout-agent-row select {
    flex: 1;
    padding: 0.4rem 0.6rem;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    font-size: 0.85rem;
  }

  /* Buttons */
  .btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    font-family: inherit;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent);
    color: #0f0f12;
    border-color: var(--accent);
  }

  .btn-primary:hover {
    filter: brightness(1.1);
  }

  .btn-ghost {
    background: transparent;
    border-color: transparent;
  }

  .btn-ghost.danger:hover {
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
  }

  .btn.danger {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.4);
    color: #fca5a5;
  }

  .btn.danger:hover {
    background: rgba(239, 68, 68, 0.3);
  }

  .btn-sm {
    padding: 0.25rem 0.6rem;
    font-size: 0.8rem;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .message.error {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .message.success {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    background: rgba(34, 197, 94, 0.15);
    color: #86efac;
    border: 1px solid rgba(34, 197, 94, 0.3);
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
  }

  .muted {
    color: var(--muted);
  }
</style>
