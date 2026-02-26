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
