import type { FlowState } from '../components/FlowPanel.svelte';

export interface KeyValuePair {
  id: string;
  key: string;
  value: string;
  enabled: boolean;
  description?: string;
  isSecret?: boolean;
}

export type AuthType = 'inherit' | 'none' | 'bearer' | 'basic' | 'apikey';

export interface AuthConfig {
  type: AuthType;
  bearerToken?: string;
  basicUsername?: string;
  basicPassword?: string;
  apiKeyName?: string;
  apiKeyValue?: string;
  apiKeyLocation?: 'header' | 'query';
}

export type BodyType = 'none' | 'json' | 'form-urlencoded' | 'multipart' | 'raw';

export interface BodyConfig {
  type: BodyType;
  rawContent: string;
  formData: KeyValuePair[];
}

export interface YaracRequest {
  id: string;
  collectionId: string;
  name: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';
  url: string;
  params: KeyValuePair[];
  headers: KeyValuePair[];
  auth: AuthConfig;
  body: BodyConfig;
  flow: FlowState;
  dslContent: string;
  isDirty?: boolean;
  response?: {
    status: number;
    statusText: string;
    durationMs: number;
    sizeBytes: number;
    headers: Record<string, string>;
    body: string;
    timestamp: string;
  };
}

export interface EnvironmentProfile {
  id: string;
  name: string;
  variables: KeyValuePair[];
}

export interface Collection {
  id: string;
  name: string;
}

export interface Workspace {
  id: string;
  name: string;
  collections: Collection[];
  requests: YaracRequest[];
  environments: EnvironmentProfile[];
  activeEnvironmentId: string;
  openTabIds: string[];
  activeTabId: string;
}

const STORAGE_KEY = 'yarac_workspaces_v2';

function generateId(): string {
  return Math.random().toString(36).substring(2, 10);
}

// Lossless DSL generator
export function serializeToDsl(req: YaracRequest): string {
  const lines: string[] = [];
  lines.push(`version = "1.0"`);
  lines.push(`name = "${req.name}"`);
  lines.push(`method = "${req.method}"`);
  lines.push(`url = "${req.url}"`);
  lines.push(``);

  lines.push(`[settings]`);
  lines.push(`inherit_auth = ${req.auth.type === 'inherit'}`);
  lines.push(`verify_ssl = true`);
  lines.push(`follow_redirects = true`);
  lines.push(`timeout_ms = 15000`);
  lines.push(``);

  const activeHeaders = req.headers.filter(h => h.enabled && h.key.trim());
  if (activeHeaders.length > 0) {
    lines.push(`[headers]`);
    for (const h of activeHeaders) {
      lines.push(`${h.key.trim()} = "${h.value.replace(/"/g, '\\"')}"`);
    }
    lines.push(``);
  }

  if (req.auth.type !== 'inherit' && req.auth.type !== 'none') {
    lines.push(`[auth]`);
    lines.push(`type = "${req.auth.type}"`);
    if (req.auth.type === 'bearer' && req.auth.bearerToken) {
      lines.push(`token = "${req.auth.bearerToken.replace(/"/g, '\\"')}"`);
    } else if (req.auth.type === 'basic') {
      lines.push(`username = "${req.auth.basicUsername || ''}"`);
      lines.push(`password = "${req.auth.basicPassword || ''}"`);
    } else if (req.auth.type === 'apikey') {
      lines.push(`key = "${req.auth.apiKeyName || ''}"`);
      lines.push(`value = "${req.auth.apiKeyValue || ''}"`);
      lines.push(`placement = "${req.auth.apiKeyLocation || 'header'}"`);
    }
    lines.push(``);
  }

  lines.push(`[flow]`);
  lines.push(`once_per_run = ${req.flow.once_per_run}`);
  lines.push(`retry_until = "${req.flow.retry_until}"`);
  lines.push(`retry_interval_ms = ${req.flow.retry_interval_ms}`);
  lines.push(`max_retries = ${req.flow.max_retries}`);
  lines.push(`delay_ms = ${req.flow.delay_ms}`);
  lines.push(`jitter_min_ms = ${req.flow.jitter_min_ms}`);
  lines.push(`jitter_max_ms = ${req.flow.jitter_max_ms}`);
  lines.push(``);

  if (req.body.type === 'json' && req.body.rawContent.trim()) {
    lines.push(`[body.json]`);
    lines.push('```json');
    lines.push(req.body.rawContent.trim());
    lines.push('```');
    lines.push(``);
  } else if (req.body.type === 'raw' && req.body.rawContent.trim()) {
    lines.push(`[body.text]`);
    lines.push('```text');
    lines.push(req.body.rawContent.trim());
    lines.push('```');
    lines.push(``);
  }

  return lines.join('\n');
}

// Lossless DSL parser helper to update visual representation
export function parseDslToRequest(dsl: string, current: YaracRequest): YaracRequest {
  const updated: YaracRequest = { ...current };

  const nameMatch = dsl.match(/name\s*=\s*"([^"]+)"/);
  if (nameMatch) updated.name = nameMatch[1];

  const methodMatch = dsl.match(/method\s*=\s*"([^"]+)"/);
  if (methodMatch) updated.method = methodMatch[1] as any;

  const urlMatch = dsl.match(/url\s*=\s*"([^"]+)"/);
  if (urlMatch) updated.url = urlMatch[1];

  const headersMatch = dsl.match(/\[headers\]([\s\S]*?)(?=\n\[|$)/);
  if (headersMatch) {
    const headerLines = headersMatch[1].split('\n').filter(l => l.includes('='));
    const parsedHeaders: KeyValuePair[] = headerLines.map(line => {
      const parts = line.split('=');
      const k = parts[0].trim();
      const v = parts.slice(1).join('=').trim().replace(/^"(.*)"$/, '$1');
      return { id: generateId(), key: k, value: v, enabled: true };
    });
    if (parsedHeaders.length > 0) {
      updated.headers = parsedHeaders;
    }
  }

  const authMatch = dsl.match(/\[auth\]([\s\S]*?)(?=\n\[|$)/);
  if (authMatch) {
    const text = authMatch[1];
    const typeMatch = text.match(/type\s*=\s*"([^"]+)"/);
    if (typeMatch) {
      updated.auth.type = typeMatch[1] as AuthType;
      const tokenMatch = text.match(/token\s*=\s*"([^"]+)"/);
      if (tokenMatch) updated.auth.bearerToken = tokenMatch[1];
    }
  }

  const bodyJsonMatch = dsl.match(/\[body\.json\]\s*```json([\s\S]*?)```/);
  if (bodyJsonMatch) {
    updated.body.type = 'json';
    updated.body.rawContent = bodyJsonMatch[1].trim();
  }

  return updated;
}

const defaultInitialWorkspaces: Workspace[] = [
  {
    id: 'ws_default',
    name: 'Core Services API',
    collections: [
      { id: 'col_users', name: 'Users & Accounts' },
      { id: 'col_billing', name: 'Billing & Subscriptions' }
    ],
    environments: [
      {
        id: 'env_prod',
        name: 'Production',
        variables: [
          { id: 'v1', key: 'base_url', value: 'https://api.yarac.dev', enabled: true },
          { id: 'v2', key: 'api_bearer_token', value: 'ey_prod_token_998124', enabled: true, isSecret: true },
          { id: 'v3', key: 'rate_limit_per_min', value: '600', enabled: true }
        ]
      },
      {
        id: 'env_staging',
        name: 'Staging',
        variables: [
          { id: 'v4', key: 'base_url', value: 'https://staging.api.yarac.dev', enabled: true },
          { id: 'v5', key: 'api_bearer_token', value: 'ey_staging_tok_382104', enabled: true, isSecret: true },
          { id: 'v6', key: 'rate_limit_per_min', value: '1200', enabled: true }
        ]
      },
      {
        id: 'env_local',
        name: 'Local Dev',
        variables: [
          { id: 'v7', key: 'base_url', value: 'http://127.0.0.1:8080', enabled: true },
          { id: 'v8', key: 'api_bearer_token', value: 'dev_local_secret_mock', enabled: true, isSecret: true }
        ]
      }
    ],
    activeEnvironmentId: 'env_prod',
    openTabIds: ['req_create_user', 'req_get_user'],
    activeTabId: 'req_create_user',
    requests: [
      {
        id: 'req_create_user',
        collectionId: 'col_users',
        name: 'Create User Account',
        method: 'POST',
        url: '{{base_url}}/api/v1/users',
        params: [
          { id: 'p1', key: 'notify', value: 'true', enabled: true, description: 'Send welcome email' }
        ],
        headers: [
          { id: 'h1', key: 'Content-Type', value: 'application/json', enabled: true },
          { id: 'h2', key: 'Accept', value: 'application/json', enabled: true },
          { id: 'h3', key: 'X-Request-Id', value: '{{$uuid}}', enabled: true }
        ],
        auth: {
          type: 'bearer',
          bearerToken: '{{api_bearer_token}}'
        },
        body: {
          type: 'json',
          rawContent: '{\n  "email": "developer_{{$randomInt}}@yarac.dev",\n  "role": "admin",\n  "tier": "enterprise",\n  "metrics": {\n    "logins": 42,\n    "projects": 12,\n    "storage_gb": 85\n  }\n}',
          formData: []
        },
        flow: {
          once_per_run: false,
          retry_until: "response.status == 200 && response.body.status == 'ready'",
          retry_interval_ms: 1000,
          max_retries: 5,
          delay_ms: 150,
          jitter_min_ms: 50,
          jitter_max_ms: 150
        },
        dslContent: '',
        isDirty: false,
        response: {
          status: 201,
          statusText: 'Created',
          durationMs: 48,
          sizeBytes: 8192,
          headers: {
            'content-type': 'application/json; charset=utf-8',
            'x-ratelimit-remaining': '599',
            'x-trace-id': 'trc_99818420b'
          },
          body: JSON.stringify({
            id: 'usr_99184820',
            status: 'created',
            created_at: '2026-10-10T10:00:00Z',
            active: true,
            tier: 'enterprise',
            stats: {
              logins_count: 145,
              active_sessions: 3,
              quota_utilized_pct: 68.5,
              api_calls_today: 4320
            },
            teams: [
              { name: 'Core Platform', members: 12, lead: 'Alice Chen' },
              { name: 'Security & Auth', members: 6, lead: 'Marcus Vance' },
              { name: 'Data Pipeline', members: 18, lead: 'Sofia Rodriguez' }
            ]
          }, null, 2),
          timestamp: '2026-10-10 10:00:00'
        }
      },
      {
        id: 'req_get_user',
        collectionId: 'col_users',
        name: 'Fetch User Profile',
        method: 'GET',
        url: '{{base_url}}/api/v1/users/usr_99184820',
        params: [
          { id: 'p2', key: 'expand', value: 'teams,permissions', enabled: true }
        ],
        headers: [
          { id: 'h4', key: 'Accept', value: 'application/json', enabled: true },
          { id: 'h5', key: 'Authorization', value: 'Bearer {{api_bearer_token}}', enabled: true }
        ],
        auth: {
          type: 'bearer',
          bearerToken: '{{api_bearer_token}}'
        },
        body: {
          type: 'none',
          rawContent: '',
          formData: []
        },
        flow: {
          once_per_run: false,
          retry_until: '',
          retry_interval_ms: 500,
          max_retries: 3,
          delay_ms: 0,
          jitter_min_ms: 0,
          jitter_max_ms: 0
        },
        dslContent: '',
        isDirty: false,
        response: {
          status: 200,
          statusText: 'OK',
          durationMs: 24,
          sizeBytes: 4096,
          headers: {
            'content-type': 'application/json; charset=utf-8',
            'cache-control': 'private, max-age=60'
          },
          body: JSON.stringify({
            id: 'usr_99184820',
            email: 'admin@yarac.dev',
            role: 'superadmin',
            account_tier: 'enterprise',
            metrics: {
              active_keys: 4,
              verified_domains: 2,
              webhook_endpoints: 8,
              failed_attempts: 0
            }
          }, null, 2),
          timestamp: '2026-10-10 10:01:15'
        }
      },
      {
        id: 'req_update_creds',
        collectionId: 'col_users',
        name: 'Update Credentials',
        method: 'PUT',
        url: '{{base_url}}/api/v1/users/usr_99184820/credentials',
        params: [],
        headers: [
          { id: 'h6', key: 'Content-Type', value: 'application/json', enabled: true }
        ],
        auth: {
          type: 'bearer',
          bearerToken: '{{api_bearer_token}}'
        },
        body: {
          type: 'json',
          rawContent: '{\n  "action": "rotate_keys",\n  "expire_old_in_hours": 24\n}',
          formData: []
        },
        flow: {
          once_per_run: false,
          retry_until: '',
          retry_interval_ms: 500,
          max_retries: 3,
          delay_ms: 0,
          jitter_min_ms: 0,
          jitter_max_ms: 0
        },
        dslContent: '',
        isDirty: false
      }
    ]
  },
  {
    id: 'ws_e2e',
    name: 'E2E Integration Test Suite',
    collections: [
      { id: 'col_smoke', name: 'Smoke Tests' }
    ],
    environments: [
      {
        id: 'env_ci',
        name: 'CI Pipeline',
        variables: [
          { id: 'v10', key: 'base_url', value: 'http://ci.local:9000', enabled: true },
          { id: 'v11', key: 'ci_token', value: 'token_mock_test_ci', enabled: true, isSecret: true }
        ]
      }
    ],
    activeEnvironmentId: 'env_ci',
    openTabIds: ['req_health_check'],
    activeTabId: 'req_health_check',
    requests: [
      {
        id: 'req_health_check',
        collectionId: 'col_smoke',
        name: 'Service Health Check',
        method: 'GET',
        url: '{{base_url}}/healthz',
        params: [],
        headers: [
          { id: 'h7', key: 'Accept', value: 'application/json', enabled: true }
        ],
        auth: { type: 'none' },
        body: { type: 'none', rawContent: '', formData: [] },
        flow: {
          once_per_run: true,
          retry_until: 'response.status == 200',
          retry_interval_ms: 1000,
          max_retries: 10,
          delay_ms: 0,
          jitter_min_ms: 0,
          jitter_max_ms: 0
        },
        dslContent: '',
        isDirty: false
      }
    ]
  }
];

// Initialize initial DSLs
for (const ws of defaultInitialWorkspaces) {
  for (const r of ws.requests) {
    r.dslContent = serializeToDsl(r);
  }
}

function loadWorkspaces(): Workspace[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed;
      }
    }
  } catch (e) {
    console.warn('Failed to load workspaces from localStorage:', e);
  }
  return defaultInitialWorkspaces;
}

export class WorkspaceStore {
  workspaces = $state<Workspace[]>(loadWorkspaces());
  activeWorkspaceId = $state<string>(loadWorkspaces()[0]?.id || 'ws_default');

  constructor() {
    $effect.root(() => {
      $effect(() => {
        // Auto-save whenever workspaces change
        try {
          localStorage.setItem(STORAGE_KEY, JSON.stringify(this.workspaces));
        } catch (err) {
          console.error('Failed to auto-save workspace state:', err);
        }
      });
    });
  }

  get currentWorkspace(): Workspace {
    const found = this.workspaces.find(w => w.id === this.activeWorkspaceId);
    return found || this.workspaces[0];
  }

  get activeEnvironment(): EnvironmentProfile | undefined {
    return this.currentWorkspace.environments.find(
      e => e.id === this.currentWorkspace.activeEnvironmentId
    );
  }

  get openRequests(): YaracRequest[] {
    const ws = this.currentWorkspace;
    return ws.openTabIds
      .map(id => ws.requests.find(r => r.id === id))
      .filter((r): r is YaracRequest => Boolean(r));
  }

  get activeRequest(): YaracRequest | undefined {
    const ws = this.currentWorkspace;
    return ws.requests.find(r => r.id === ws.activeTabId) || this.openRequests[0];
  }

  // Workspace actions
  switchWorkspace(id: string) {
    if (this.workspaces.some(w => w.id === id)) {
      this.activeWorkspaceId = id;
    }
  }

  addWorkspace(name: string): Workspace {
    const newWs: Workspace = {
      id: 'ws_' + generateId(),
      name: name.trim() || 'New Workspace',
      collections: [{ id: 'col_' + generateId(), name: 'Default Collection' }],
      requests: [],
      environments: [
        {
          id: 'env_' + generateId(),
          name: 'Development',
          variables: [{ id: generateId(), key: 'base_url', value: 'http://localhost:3000', enabled: true }]
        }
      ],
      activeEnvironmentId: '',
      openTabIds: [],
      activeTabId: ''
    };
    newWs.activeEnvironmentId = newWs.environments[0].id;
    this.workspaces.push(newWs);
    this.activeWorkspaceId = newWs.id;
    return newWs;
  }

  // Collection actions
  addCollection(name: string): string {
    const colId = 'col_' + generateId();
    this.currentWorkspace.collections.push({
      id: colId,
      name: name.trim() || 'New Collection'
    });
    return colId;
  }

  // Request actions
  createRequest(collectionId: string, name = 'New Request', method: YaracRequest['method'] = 'GET'): YaracRequest {
    const reqId = 'req_' + generateId();
    const newReq: YaracRequest = {
      id: reqId,
      collectionId,
      name,
      method,
      url: '{{base_url}}/api/v1/resource',
      params: [],
      headers: [
        { id: generateId(), key: 'Accept', value: 'application/json', enabled: true }
      ],
      auth: { type: 'inherit' },
      body: { type: 'none', rawContent: '', formData: [] },
      flow: {
        once_per_run: false,
        retry_until: '',
        retry_interval_ms: 500,
        max_retries: 3,
        delay_ms: 0,
        jitter_min_ms: 0,
        jitter_max_ms: 0
      },
      dslContent: '',
      isDirty: false
    };
    newReq.dslContent = serializeToDsl(newReq);
    this.currentWorkspace.requests.push(newReq);
    this.openTab(reqId);
    return newReq;
  }

  // Tabs management
  openTab(requestId: string) {
    const ws = this.currentWorkspace;
    if (!ws.openTabIds.includes(requestId)) {
      ws.openTabIds.push(requestId);
    }
    ws.activeTabId = requestId;
  }

  closeTab(requestId: string) {
    const ws = this.currentWorkspace;
    const index = ws.openTabIds.indexOf(requestId);
    if (index !== -1) {
      ws.openTabIds.splice(index, 1);
      if (ws.activeTabId === requestId) {
        ws.activeTabId = ws.openTabIds[Math.max(0, index - 1)] || '';
      }
    }
  }

  setActiveTab(requestId: string) {
    this.currentWorkspace.activeTabId = requestId;
  }

  // Request update & DSL sync
  updateActiveRequest(updates: Partial<YaracRequest>, syncDsl = true) {
    const req = this.activeRequest;
    if (!req) return;

    Object.assign(req, updates);
    req.isDirty = true;

    if (syncDsl) {
      req.dslContent = serializeToDsl(req);
    }
  }

  updateActiveRequestFromDsl(newDsl: string) {
    const req = this.activeRequest;
    if (!req) return;

    req.dslContent = newDsl;
    const parsed = parseDslToRequest(newDsl, req);
    Object.assign(req, parsed);
    req.isDirty = true;
  }

  saveActiveRequest() {
    const req = this.activeRequest;
    if (req) {
      req.isDirty = false;
    }
  }

  // Environment Profile management (Bruno-style)
  setActiveEnvironment(envId: string) {
    this.currentWorkspace.activeEnvironmentId = envId;
  }

  addEnvironment(name: string): EnvironmentProfile {
    const newEnv: EnvironmentProfile = {
      id: 'env_' + generateId(),
      name: name.trim() || 'New Environment',
      variables: []
    };
    this.currentWorkspace.environments.push(newEnv);
    this.currentWorkspace.activeEnvironmentId = newEnv.id;
    return newEnv;
  }

  duplicateEnvironment(envId: string): EnvironmentProfile | null {
    const source = this.currentWorkspace.environments.find(e => e.id === envId);
    if (!source) return null;

    const dup: EnvironmentProfile = {
      id: 'env_' + generateId(),
      name: `${source.name} (Copy)`,
      variables: source.variables.map(v => ({ ...v, id: generateId() }))
    };
    this.currentWorkspace.environments.push(dup);
    return dup;
  }

  deleteEnvironment(envId: string) {
    const ws = this.currentWorkspace;
    if (ws.environments.length <= 1) return; // Keep at least one
    ws.environments = ws.environments.filter(e => e.id !== envId);
    if (ws.activeEnvironmentId === envId) {
      ws.activeEnvironmentId = ws.environments[0]?.id || '';
    }
  }

  copyVariableToEnvironment(variable: KeyValuePair, targetEnvId: string) {
    const target = this.currentWorkspace.environments.find(e => e.id === targetEnvId);
    if (!target) return;

    const existingIndex = target.variables.findIndex(v => v.key === variable.key);
    const copied: KeyValuePair = { ...variable, id: generateId() };

    if (existingIndex >= 0) {
      target.variables[existingIndex] = copied;
    } else {
      target.variables.push(copied);
    }
  }
}

export const workspaceStore = new WorkspaceStore();
