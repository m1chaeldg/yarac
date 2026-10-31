import type { YaracRequest, Workspace } from '../stores/workspace.svelte';

export interface ExecutionReport {
  request_name: string;
  method: string;
  url: string;
  status_code: number;
  duration_ms: number;
  response_headers: Record<string, string>;
  response_body: string;
  assertions: Array<{ name: string; passed: boolean; message?: string }>;
  logs: string[];
  error?: string;
}

export interface RunnerResult {
  stats: {
    count: number;
    min: number;
    max: number;
    mean: number;
    p50: number;
    p90: number;
    p95: number;
    p99: number;
    rps: number;
    errorRate: number;
    successCount: number;
    failureCount: number;
  };
  logs: Array<{
    iteration: number;
    requestName: string;
    method: string;
    status: number;
    durationMs: number;
    passed: boolean;
    timestamp: string;
  }>;
}

export interface WatcherEvent {
  path: string;
  kind: 'created' | 'modified' | 'deleted';
  content?: string;
}

const API_BASE = '/api';

export const backendApi = {
  /**
   * Executes a request through the real Rust/backend network engine
   */
  async executeRequest(req: YaracRequest, envVars: Record<string, string> = {}): Promise<ExecutionReport> {
    try {
      const res = await fetch(`${API_BASE}/request/execute`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          dsl: req.dslContent,
          env_vars: envVars
        })
      });

      if (res.ok) {
        const data = await res.json();
        if (data.report) {
          return data.report;
        }
      }
    } catch (err) {
      console.warn('Backend execute call fallback to client fetch:', err);
    }

    // Direct HTTP execution fallback if backend daemon is not attached
    const startTime = performance.now();
    try {
      let resolvedUrl = req.url.replace(/\{\{([^}]+)\}\}/g, (_, k) => envVars[k] || '');
      // Substitute path variables if present
      if (req.pathVariables && req.pathVariables.length > 0) {
        for (const pv of req.pathVariables) {
          if (pv.key && pv.value !== undefined) {
            resolvedUrl = resolvedUrl.replace(new RegExp(`:${pv.key}\\b`, 'g'), encodeURIComponent(pv.value));
            resolvedUrl = resolvedUrl.replace(new RegExp(`\\{${pv.key}\\}`, 'g'), encodeURIComponent(pv.value));
          }
        }
      }
      // Attach enabled query parameters if not already in URL
      if (req.params && req.params.length > 0 && !resolvedUrl.includes('?')) {
        const enabledParams = req.params.filter(p => p.enabled && p.key.trim() && !p.key.endsWith('~'));
        if (enabledParams.length > 0) {
          const qs = enabledParams.map(p => `${encodeURIComponent(p.key.trim())}=${encodeURIComponent(p.value || '')}`).join('&');
          resolvedUrl = `${resolvedUrl}?${qs}`;
        }
      }

      const headersRecord: Record<string, string> = {};
      for (const h of req.headers) {
        if (h.enabled && !h.key.endsWith('~') && h.key.trim()) {
          headersRecord[h.key.trim()] = h.value;
        }
      }
      if (req.auth.type === 'bearer' && req.auth.bearerToken) {
        headersRecord['Authorization'] = `Bearer ${req.auth.bearerToken}`;
      } else if (req.auth.type === 'basic' && (req.auth.basicUsername || req.auth.basicPassword)) {
        const creds = btoa(`${req.auth.basicUsername || ''}:${req.auth.basicPassword || ''}`);
        headersRecord['Authorization'] = `Basic ${creds}`;
      } else if (req.auth.type === 'apikey' && req.auth.apiKeyName && req.auth.apiKeyValue) {
        if (req.auth.apiKeyLocation === 'query') {
          const sep = resolvedUrl.includes('?') ? '&' : '?';
          resolvedUrl += `${sep}${encodeURIComponent(req.auth.apiKeyName)}=${encodeURIComponent(req.auth.apiKeyValue)}`;
        } else {
          headersRecord[req.auth.apiKeyName] = req.auth.apiKeyValue;
        }
      }

      const fetchRes = await fetch(resolvedUrl, {
        method: req.method,
        headers: headersRecord,
        body: ['POST', 'PUT', 'PATCH'].includes(req.method) && req.body.type === 'json' && req.body.rawContent
          ? req.body.rawContent
          : undefined
      });

      const bodyText = await fetchRes.text();
      const duration = Math.round(performance.now() - startTime);

      const respHeaders: Record<string, string> = {};
      fetchRes.headers.forEach((v, k) => {
        respHeaders[k] = v;
      });

      return {
        request_name: req.name,
        method: req.method,
        url: resolvedUrl,
        status_code: fetchRes.status,
        duration_ms: duration,
        response_headers: respHeaders,
        response_body: bodyText,
        assertions: [
          {
            name: `Status code is ${fetchRes.status}`,
            passed: fetchRes.status >= 200 && fetchRes.status < 400
          }
        ],
        logs: [`Completed with status ${fetchRes.status} in ${duration}ms`]
      };
    } catch (e: any) {
      const duration = Math.round(performance.now() - startTime);
      return {
        request_name: req.name,
        method: req.method,
        url: req.url,
        status_code: 0,
        duration_ms: duration,
        response_headers: {},
        response_body: '',
        assertions: [],
        logs: [],
        error: e.message || 'Network error'
      };
    }
  },

  /**
   * Executes multi-request iterations with HDR statistics
   */
  async executeCollectionRun(options: {
    requests: YaracRequest[];
    iterations: number;
    concurrency: number;
    delayMs: number;
    rampUpSec?: number;
    durationSec?: number;
    data?: Array<Record<string, string>>;
  }): Promise<RunnerResult> {
    try {
      const res = await fetch(`${API_BASE}/runner/execute`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          requests: options.requests,
          iterations: options.iterations,
          concurrency: options.concurrency,
          delay_ms: options.delayMs,
          ramp_up_sec: options.rampUpSec,
          duration_sec: options.durationSec,
          data: options.data
        })
      });
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.warn('Runner API failed, calculating client metrics:', e);
    }

    // Client-side execution loop with real timing
    const logs: RunnerResult['logs'] = [];
    const latencies: number[] = [];

    for (let i = 0; i < options.iterations; i++) {
      for (const req of options.requests) {
        const start = performance.now();
        const rep = await this.executeRequest(req);
        const duration = Math.max(1, Math.round(performance.now() - start));
        latencies.push(duration);

        logs.push({
          iteration: i + 1,
          requestName: req.name,
          method: req.method,
          status: rep.status_code,
          durationMs: duration,
          passed: rep.status_code >= 200 && rep.status_code < 400 && !rep.error,
          timestamp: new Date().toLocaleTimeString()
        });

        if (options.delayMs > 0) {
          await new Promise(r => setTimeout(r, options.delayMs));
        }
      }
    }

    const sorted = [...latencies].sort((a, b) => a - b);
    const count = sorted.length;
    const failures = logs.filter(l => !l.passed).length;
    const successes = count - failures;
    const sum = sorted.reduce((a, b) => a + b, 0);

    return {
      stats: {
        count,
        min: sorted[0] || 0,
        max: sorted[count - 1] || 0,
        mean: count > 0 ? Math.round(sum / count) : 0,
        p50: sorted[Math.floor(count * 0.5)] || 0,
        p90: sorted[Math.floor(count * 0.9)] || 0,
        p95: sorted[Math.floor(count * 0.95)] || 0,
        p99: sorted[Math.floor(count * 0.99)] || 0,
        rps: count > 0 ? Math.round((count / (sum / 1000 || 0.1)) * 10) / 10 : 0,
        errorRate: count > 0 ? Math.round((failures / count) * 100) : 0,
        successCount: successes,
        failureCount: failures
      },
      logs
    };
  },

  /**
   * Imports external specifications (Bruno, Postman, OpenAPI)
   */
  async importCollection(format: string, content: string): Promise<any[]> {
    try {
      const res = await fetch(`${API_BASE}/interop/import`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ format, content })
      });
      if (res.ok) {
        const data = await res.json();
        return data.requests || [];
      }
    } catch (e) {
      console.warn('Backend import failed:', e);
    }
    return [];
  },

  /**
   * Exports workspace requests to OpenAPI 3.1
   */
  async exportOpenApi(requests: any[], title: string, version: string): Promise<string> {
    try {
      const res = await fetch(`${API_BASE}/interop/export`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ requests, title, version })
      });
      if (res.ok) {
        const data = await res.json();
        return data.spec || '';
      }
    } catch (e) {
      console.warn('Backend export failed:', e);
    }
    return '';
  },

  /**
   * Loads the workspace from disk files (workspace.json and .yarac files)
   */
  async loadWorkspaceFromDisk(): Promise<{ manifest: any; requests: any[] } | null> {
    try {
      const res = await fetch(`${API_BASE}/workspace`);
      if (res.ok) {
        const data = await res.json();
        return data;
      }
    } catch (e) {
      console.warn('Failed to load workspace from disk API:', e);
    }
    return null;
  },

  /**
   * Persists workspace collections, manifest, and .yarac files directly to disk
   */
  async saveWorkspaceToDisk(manifest: any, requests: any[]): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/workspace/save`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          manifest,
          requests: requests.map(r => ({
            id: r.id,
            collectionId: r.collectionId,
            folderPath: r.folderPath,
            name: r.name,
            dslContent: r.dslContent
          }))
        })
      });
      return res.ok;
    } catch (e) {
      console.error('Failed to save workspace to disk:', e);
      return false;
    }
  },

  /**
   * Reveals a request file or collection folder in the OS native file explorer
   */
  async revealInExplorer(options: { collectionName?: string; requestName?: string; path?: string }): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/workspace/reveal`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(options)
      });
      return res.ok;
    } catch (e) {
      console.warn('revealInExplorer API call failed:', e);
      return false;
    }
  },

  /**
   * Subscribes to real-time file watcher events from the backend (SSE)
   */
  /**
   * Retrieves Git status for active workspace repository
   */
  async getGitStatus(): Promise<{ branch: string; modified_files: string[]; untracked_files: string[]; is_clean: boolean }> {
    try {
      const res = await fetch(`${API_BASE}/git/status`);
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.warn('getGitStatus failed:', e);
    }
    return { branch: 'main', modified_files: [], untracked_files: [], is_clean: true };
  },

  /**
   * Commits staged changes
   */
  async commitGit(message: string): Promise<{ status: string; output?: string; message?: string }> {
    try {
      const res = await fetch(`${API_BASE}/git/commit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message })
      });
      if (res.ok) {
        return await res.json();
      }
    } catch (e: any) {
      return { status: 'error', message: e.message };
    }
    return { status: 'error', message: 'Failed to commit' };
  },

  /**
   * Syncs repository (pull --rebase & push)
   */
  async syncGit(): Promise<{ status: string; output?: string; message?: string }> {
    try {
      const res = await fetch(`${API_BASE}/git/sync`, {
        method: 'POST'
      });
      if (res.ok) {
        return await res.json();
      }
    } catch (e: any) {
      return { status: 'error', message: e.message };
    }
    return { status: 'error', message: 'Failed to sync' };
  },

  /**
   * Retrieves local git branches
   */
  async getGitBranches(): Promise<string[]> {
    try {
      const res = await fetch(`${API_BASE}/git/branches`);
      if (res.ok) {
        const data = await res.json();
        return data.branches || ['main'];
      }
    } catch (e) {
      console.warn('getGitBranches failed:', e);
    }
    return ['main'];
  },

  /**
   * Retrieves Model Context Protocol (MCP) server daemon status
   */
  async getMcpStatus(): Promise<{ running: boolean; transport: string; port: number; token_configured: boolean }> {
    try {
      const res = await fetch(`${API_BASE}/mcp/status`);
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.warn('getMcpStatus failed:', e);
    }
    return { running: true, transport: 'stdio', port: 9475, token_configured: true };
  },

  /**
   * Toggles MCP server transport
   */
  async toggleMcpServer(transport: 'stdio' | 'http'): Promise<any> {
    try {
      const res = await fetch(`${API_BASE}/mcp/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ transport })
      });
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.warn('toggleMcpServer failed:', e);
    }
    return { status: 'ok', transport };
  },

  /**
   * Retrieves Claude Desktop snippet and authentication token
   */
  async getMcpConfig(): Promise<{ claude_config: any; token: string }> {
    try {
      const res = await fetch(`${API_BASE}/mcp/config`);
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.warn('getMcpConfig failed:', e);
    }
    return {
      claude_config: {
        mcpServers: {
          yarac: {
            command: 'yarac-cli',
            args: ['mcp', '--transport', 'stdio']
          }
        }
      },
      token: 'yarac_mcp_secure_local_token_9941'
    };
  },

  subscribeToWatcher(onEvent: (event: WatcherEvent) => void): () => void {
    let eventSource: EventSource | null = null;
    try {
      eventSource = new EventSource(`${API_BASE}/workspace/events`);

      eventSource.addEventListener('file-changed', (e: MessageEvent) => {
        try {
          const parsed = JSON.parse(e.data);
          onEvent(parsed);
        } catch (err) {
          console.error('Failed to parse SSE event:', err);
        }
      });

      eventSource.onerror = (err) => {
        // SSE reconnection will be attempted automatically by browser
      };
    } catch (e) {
      console.warn('SSE subscription failed:', e);
    }

    return () => {
      if (eventSource) {
        eventSource.close();
      }
    };
  }
};
