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
      const resolvedUrl = req.url.replace(/\{\{([^}]+)\}\}/g, (_, k) => envVars[k] || '');
      const headersRecord: Record<string, string> = {};
      for (const h of req.headers) {
        if (h.enabled && h.key.trim()) {
          headersRecord[h.key.trim()] = h.value;
        }
      }
      if (req.auth.type === 'bearer' && req.auth.bearerToken) {
        headersRecord['Authorization'] = `Bearer ${req.auth.bearerToken}`;
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
  }): Promise<RunnerResult> {
    try {
      const res = await fetch(`${API_BASE}/runner/execute`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          requests: options.requests,
          iterations: options.iterations,
          concurrency: options.concurrency,
          delay_ms: options.delayMs
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
   * Subscribes to real-time file watcher events from the backend (SSE)
   */
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
