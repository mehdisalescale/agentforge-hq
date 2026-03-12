/**
 * API base URL. Empty = same origin. Set VITE_API_URL in .env for dev (e.g. http://localhost:3000).
 */
const API_BASE = typeof import.meta !== 'undefined' && import.meta.env?.VITE_API_URL != null
  ? (import.meta.env.VITE_API_URL as string).replace(/\/$/, '')
  : '';

/** Preset names matching forge-agent AgentPreset enum. */
export type AgentPreset =
  | 'CodeWriter'
  | 'Reviewer'
  | 'Tester'
  | 'Debugger'
  | 'Architect'
  | 'Documenter'
  | 'SecurityAuditor'
  | 'Refactorer'
  | 'Explorer';

export interface Agent {
  id: string;
  name: string;
  model: string;
  system_prompt: string | null;
  allowed_tools: string[] | null;
  max_turns: number | null;
  use_max: boolean;
  preset: AgentPreset | null;
  config: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
}

export interface NewAgent {
  name: string;
  model?: string | null;
  system_prompt?: string | null;
  allowed_tools?: string[] | null;
  max_turns?: number | null;
  use_max?: boolean | null;
  preset?: AgentPreset | null;
  config?: Record<string, unknown> | null;
}

export interface UpdateAgent {
  name?: string | null;
  model?: string | null;
  system_prompt?: string | null;
  allowed_tools?: string[] | null;
  max_turns?: number | null;
  use_max?: boolean | null;
  preset?: AgentPreset | null;
  config?: Record<string, unknown> | null;
}

export const PRESETS: AgentPreset[] = [
  'CodeWriter',
  'Reviewer',
  'Tester',
  'Debugger',
  'Architect',
  'Documenter',
  'SecurityAuditor',
  'Refactorer',
  'Explorer',
];

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text();
    let message = body;
    try {
      const j = JSON.parse(body);
      if (j?.error) message = j.error;
    } catch {
      // use body as message
    }
    throw new Error(message || `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

export async function listAgents(): Promise<Agent[]> {
  const res = await fetch(`${API_BASE}/api/v1/agents`);
  return handleResponse<Agent[]>(res);
}

export async function getAgent(id: string): Promise<Agent> {
  const res = await fetch(`${API_BASE}/api/v1/agents/${encodeURIComponent(id)}`);
  return handleResponse<Agent>(res);
}

export async function createAgent(data: NewAgent): Promise<Agent> {
  const res = await fetch(`${API_BASE}/api/v1/agents`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Agent>(res);
}

export async function updateAgent(id: string, data: UpdateAgent): Promise<Agent> {
  const res = await fetch(`${API_BASE}/api/v1/agents/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Agent>(res);
}

export async function deleteAgent(id: string): Promise<void> {
  const res = await fetch(`${API_BASE}/api/v1/agents/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  await handleResponse<void>(res);
}

// --- Run (Phase 1) ---

export interface RunRequest {
  agent_id: string;
  prompt: string;
  session_id?: string | null;
  directory?: string | null;
}

export interface RunResponse {
  session_id: string;
  message?: string;
}

/** POST run: start a process for the given agent + prompt; optional session_id for resume. */
export async function runAgent(req: RunRequest): Promise<RunResponse> {
  const res = await fetch(`${API_BASE}/api/v1/run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  return handleResponse<RunResponse>(res);
}

// --- Sessions (Phase 1) ---

export interface Session {
  id: string;
  agent_id: string;
  claude_session_id: string | null;
  directory: string;
  status: string;
  cost_usd?: number;
  created_at: string;
  updated_at: string;
}

export async function listSessions(): Promise<Session[]> {
  const res = await fetch(`${API_BASE}/api/v1/sessions`);
  return handleResponse<Session[]>(res);
}

export async function getSession(id: string): Promise<Session> {
  const res = await fetch(`${API_BASE}/api/v1/sessions/${encodeURIComponent(id)}`);
  return handleResponse<Session>(res);
}

export async function deleteSession(id: string): Promise<void> {
  const res = await fetch(`${API_BASE}/api/v1/sessions/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  await handleResponse<void>(res);
}

/** Export session as JSON or Markdown; returns blob URL or throws. */
export function exportSessionUrl(id: string, format: 'json' | 'markdown'): string {
  return `${API_BASE}/api/v1/sessions/${encodeURIComponent(id)}/export?format=${format}`;
}

// --- WebSocket (event stream) ---

/** WebSocket URL for EventBus stream (same origin or VITE_API_URL). */
export function wsUrl(path = '/api/v1/ws'): string {
  if (API_BASE) {
    const base = API_BASE.replace(/^http/, 'ws');
    return `${base}${path}`;
  }
  const proto = typeof location !== 'undefined' && location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${proto}//${typeof location !== 'undefined' ? location.host : 'localhost'}${path}`;
}

/** ForgeEvent wire format: { type, data }. Process output events for streaming. */
export interface ForgeEventWire {
  type: string;
  data?: {
    session_id?: string;
    agent_id?: string;
    kind?: string;
    content?: string;
    exit_code?: number;
    error?: string;
    timestamp?: string;
    [key: string]: unknown;
  };
}

export function isProcessOutputEvent(ev: ForgeEventWire, sessionId: string | null): boolean {
  if (!sessionId || ev.type !== 'ProcessOutput') return false;
  return ev.data?.session_id === sessionId;
}

export function isProcessLifecycleEvent(ev: ForgeEventWire, sessionId: string | null): boolean {
  if (!sessionId) return false;
  return (
    (ev.type === 'ProcessStarted' || ev.type === 'ProcessCompleted' || ev.type === 'ProcessFailed') &&
    ev.data?.session_id === sessionId
  );
}

// --- Skills (Phase 2) ---

export interface Skill {
  id: string;
  name: string;
  description: string | null;
  category: string | null;
  subcategory: string | null;
  content: string;
  source_repo: string | null;
  parameters_json: string | null;
  examples_json: string | null;
  usage_count: number;
  created_at: string;
}

export async function listSkills(): Promise<Skill[]> {
  const res = await fetch(`${API_BASE}/api/v1/skills`);
  return handleResponse<Skill[]>(res);
}

// --- Workflows (Phase 2) ---

export interface Workflow {
  id: string;
  name: string;
  description: string | null;
  definition_json: string;
  created_at: string;
  updated_at: string;
}

export async function listWorkflows(): Promise<Workflow[]> {
  const res = await fetch(`${API_BASE}/api/v1/workflows`);
  return handleResponse<Workflow[]>(res);
}

export interface NewWorkflow {
  name: string;
  description?: string | null;
  definition_json: string;
}

export interface UpdateWorkflowData {
  name?: string;
  description?: string | null;
  definition_json?: string;
}

export async function createWorkflow(data: NewWorkflow): Promise<Workflow> {
  const res = await fetch(`${API_BASE}/api/v1/workflows`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Workflow>(res);
}

export async function updateWorkflow(id: string, data: UpdateWorkflowData): Promise<Workflow> {
  const res = await fetch(`${API_BASE}/api/v1/workflows/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Workflow>(res);
}

export async function deleteWorkflow(id: string): Promise<void> {
  const res = await fetch(`${API_BASE}/api/v1/workflows/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  await handleResponse<void>(res);
}

export interface RunWorkflowRequest {
  prompt: string;
  working_dir?: string;
}

export interface RunWorkflowResponse {
  session_id: string;
  message?: string;
}

export async function runWorkflow(id: string, prompt: string, working_dir?: string): Promise<RunWorkflowResponse> {
  const body: RunWorkflowRequest = { prompt };
  if (working_dir) body.working_dir = working_dir;
  const res = await fetch(`${API_BASE}/api/v1/workflows/${encodeURIComponent(id)}/run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  return handleResponse<RunWorkflowResponse>(res);
}

// --- Memory (Phase 2) ---

export interface Memory {
  id: string;
  category: string;
  content: string;
  confidence: number;
  source_session_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface NewMemory {
  category?: string;
  content: string;
  confidence?: number;
  source_session_id?: string;
}

export interface UpdateMemory {
  content?: string;
  category?: string;
  confidence?: number;
}

export async function listMemories(opts?: { q?: string; limit?: number; offset?: number }): Promise<Memory[]> {
  const params = new URLSearchParams();
  if (opts?.q) params.set('q', opts.q);
  if (opts?.limit != null) params.set('limit', String(opts.limit));
  if (opts?.offset != null) params.set('offset', String(opts.offset));
  const qs = params.toString();
  const res = await fetch(`${API_BASE}/api/v1/memory${qs ? '?' + qs : ''}`);
  return handleResponse<Memory[]>(res);
}

export async function getMemory(id: string): Promise<Memory> {
  const res = await fetch(`${API_BASE}/api/v1/memory/${encodeURIComponent(id)}`);
  return handleResponse<Memory>(res);
}

export async function createMemory(data: NewMemory): Promise<Memory> {
  const res = await fetch(`${API_BASE}/api/v1/memory`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Memory>(res);
}

export async function updateMemory(id: string, data: UpdateMemory): Promise<Memory> {
  const res = await fetch(`${API_BASE}/api/v1/memory/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Memory>(res);
}

export async function deleteMemory(id: string): Promise<void> {
  const res = await fetch(`${API_BASE}/api/v1/memory/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  await handleResponse<void>(res);
}

// --- Hooks (Phase 2) ---

export interface Hook {
  id: string;
  name: string;
  event_type: string;
  timing: string;
  command: string;
  enabled: boolean;
  created_at: string;
}

export interface NewHook {
  name: string;
  event_type: string;
  timing: string;
  command: string;
}

export interface UpdateHook {
  name?: string;
  command?: string;
  enabled?: boolean;
}

export async function listHooks(): Promise<Hook[]> {
  const res = await fetch(`${API_BASE}/api/v1/hooks`);
  return handleResponse<Hook[]>(res);
}

export async function createHook(data: NewHook): Promise<Hook> {
  const res = await fetch(`${API_BASE}/api/v1/hooks`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Hook>(res);
}

export async function updateHook(id: string, data: UpdateHook): Promise<Hook> {
  const res = await fetch(`${API_BASE}/api/v1/hooks/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Hook>(res);
}

export async function deleteHook(id: string): Promise<void> {
  const res = await fetch(`${API_BASE}/api/v1/hooks/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  await handleResponse<void>(res);
}

// --- Schedules (Phase 3) ---

export interface Schedule {
  id: string;
  name: string;
  cron_expr: string;
  agent_id: string;
  prompt: string;
  directory: string;
  enabled: boolean;
  last_run_at: string | null;
  next_run_at: string | null;
  run_count: number;
  created_at: string;
}

export interface NewSchedule {
  name: string;
  cron_expr: string;
  agent_id: string;
  prompt: string;
  directory?: string;
}

export interface UpdateSchedule {
  name?: string;
  cron_expr?: string;
  prompt?: string;
  directory?: string;
  enabled?: boolean;
}

export async function listSchedules(): Promise<Schedule[]> {
  const res = await fetch(`${API_BASE}/api/v1/schedules`);
  return handleResponse<Schedule[]>(res);
}

export async function createSchedule(data: NewSchedule): Promise<Schedule> {
  const res = await fetch(`${API_BASE}/api/v1/schedules`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Schedule>(res);
}

export async function updateSchedule(id: string, data: UpdateSchedule): Promise<Schedule> {
  const res = await fetch(`${API_BASE}/api/v1/schedules/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Schedule>(res);
}

export async function deleteSchedule(id: string): Promise<void> {
  const res = await fetch(`${API_BASE}/api/v1/schedules/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  await handleResponse<void>(res);
}

// --- Analytics (Phase 3) ---

export interface DailyCost {
  date: string;
  cost: number;
}

export interface AgentCostBreakdown {
  agent_id: string;
  total_cost: number;
  session_count: number;
}

export interface SessionStats {
  total: number;
  completed: number;
  failed: number;
  avg_cost: number;
  p90_cost: number;
}

export interface UsageReport {
  total_cost: number;
  daily_costs: DailyCost[];
  agent_breakdown: AgentCostBreakdown[];
  stats: SessionStats;
  projected_monthly_cost: number;
}

export async function getUsageAnalytics(start?: string, end?: string): Promise<UsageReport> {
  const params = new URLSearchParams();
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  const qs = params.toString();
  const res = await fetch(`${API_BASE}/api/v1/analytics/usage${qs ? '?' + qs : ''}`);
  return handleResponse<UsageReport>(res);
}

/** Export session as JSON, Markdown, or HTML; returns URL to open. */
export function exportSessionHtmlUrl(id: string): string {
  return `${API_BASE}/api/v1/sessions/${encodeURIComponent(id)}/export?format=html`;
}

// --- Org & Governance (Wave 3) ---

export interface Company {
  id: string;
  name: string;
  mission?: string | null;
  budget_limit?: number | null;
  budget_used: number;
}

export interface Department {
  id: string;
  company_id: string;
  name: string;
  description?: string | null;
}

export interface OrgPosition {
  id: string;
  company_id: string;
  department_id?: string | null;
  agent_id?: string | null;
  reports_to?: string | null;
  role: string;
  title?: string | null;
}

export interface OrgChartNode {
  position: OrgPosition;
  children: OrgChartNode[];
}

export interface CompanyOrgChart {
  company: Company;
  departments: Department[];
  roots: OrgChartNode[];
}

export async function listCompanies(): Promise<Company[]> {
  const res = await fetch(`${API_BASE}/api/v1/companies`);
  return handleResponse<Company[]>(res);
}

export async function createCompany(data: {
  name: string;
  mission?: string;
  budget_limit?: number;
}): Promise<Company> {
  const res = await fetch(`${API_BASE}/api/v1/companies`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Company>(res);
}

export async function createDepartment(data: {
  company_id: string;
  name: string;
  description?: string;
}): Promise<Department> {
  const res = await fetch(`${API_BASE}/api/v1/departments`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Department>(res);
}

export async function createOrgPosition(data: {
  company_id: string;
  department_id?: string;
  agent_id?: string;
  reports_to?: string;
  role: string;
  title?: string;
}): Promise<OrgPosition> {
  const res = await fetch(`${API_BASE}/api/v1/org-positions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<OrgPosition>(res);
}

export async function getOrgChart(company_id?: string): Promise<CompanyOrgChart> {
  const params = new URLSearchParams();
  if (company_id) params.set('company_id', company_id);
  const qs = params.toString();
  const res = await fetch(`${API_BASE}/api/v1/org-chart${qs ? '?' + qs : ''}`);
  return handleResponse<CompanyOrgChart>(res);
}

// --- Personas (Wave 3) ---

export interface Persona {
  id: string;
  division_slug: string;
  slug: string;
  name: string;
  short_description: string;
  personality?: string | null;
  deliverables?: string | null;
  success_metrics?: string | null;
  workflow?: string | null;
  tags: string[];
  source_file: string;
  created_at: string;
  updated_at: string;
}

export async function listPersonas(opts?: {
  division_slug?: string;
  q?: string;
}): Promise<Persona[]> {
  const params = new URLSearchParams();
  if (opts?.division_slug) params.set('division_slug', opts.division_slug);
  if (opts?.q) params.set('q', opts.q);
  const qs = params.toString();
  const res = await fetch(`${API_BASE}/api/v1/personas${qs ? '?' + qs : ''}`);
  return handleResponse<Persona[]>(res);
}

