import type { YaracRequest } from '../../stores/workspace.svelte';

export function resolveUrl(req: YaracRequest, envVars?: Record<string, string>): string {
  let url = req.url || 'http://localhost:3000';
  if (envVars) {
    for (const [k, v] of Object.entries(envVars)) {
      url = url.split(`{{${k}}}`).join(v);
    }
  }

  // Active query parameters
  const activeParams = req.params.filter(p => p.enabled && !p.key.endsWith('~') && p.key.trim());
  if (activeParams.length > 0) {
    const separator = url.includes('?') ? '&' : '?';
    const qs = activeParams
      .map(p => {
        let val = p.value;
        if (envVars) {
          for (const [k, v] of Object.entries(envVars)) {
            val = val.split(`{{${k}}}`).join(v);
          }
        }
        return `${encodeURIComponent(p.key)}=${encodeURIComponent(val)}`;
      })
      .join('&');
    url = `${url}${separator}${qs}`;
  }

  return url;
}

export function resolveHeaders(req: YaracRequest, envVars?: Record<string, string>): Record<string, string> {
  const headers: Record<string, string> = {};

  for (const h of req.headers.filter(h => h.enabled && !h.key.endsWith('~') && h.key.trim())) {
    let val = h.value;
    if (envVars) {
      for (const [k, v] of Object.entries(envVars)) {
        val = val.split(`{{${k}}}`).join(v);
      }
    }
    headers[h.key] = val;
  }

  // Auth headers
  if (req.auth) {
    if (req.auth.type === 'bearer' && req.auth.bearerToken) {
      let token = req.auth.bearerToken;
      if (envVars) {
        for (const [k, v] of Object.entries(envVars)) {
          token = token.split(`{{${k}}}`).join(v);
        }
      }
      headers['Authorization'] = `Bearer ${token}`;
    } else if (req.auth.type === 'basic' && (req.auth.basicUsername || req.auth.basicPassword)) {
      let u = req.auth.basicUsername || '';
      let p = req.auth.basicPassword || '';
      if (envVars) {
        for (const [k, v] of Object.entries(envVars)) {
          u = u.split(`{{${k}}}`).join(v);
          p = p.split(`{{${k}}}`).join(v);
        }
      }
      const b64 = typeof btoa !== 'undefined' ? btoa(`${u}:${p}`) : Buffer.from(`${u}:${p}`).toString('base64');
      headers['Authorization'] = `Basic ${b64}`;
    } else if (req.auth.type === 'apikey' && req.auth.apiKeyLocation === 'header' && req.auth.apiKeyName) {
      let val = req.auth.apiKeyValue || '';
      if (envVars) {
        for (const [k, v] of Object.entries(envVars)) {
          val = val.split(`{{${k}}}`).join(v);
        }
      }
      headers[req.auth.apiKeyName] = val;
    }
  }

  // Auto content-type header for body if not present
  if (req.body) {
    if (req.body.type === 'json' && !headers['Content-Type']) {
      headers['Content-Type'] = 'application/json';
    } else if (req.body.type === 'form-urlencoded' && !headers['Content-Type']) {
      headers['Content-Type'] = 'application/x-www-form-urlencoded';
    }
  }

  return headers;
}

export function resolveBody(req: YaracRequest, envVars?: Record<string, string>): string {
  if (!req.body || req.body.type === 'none') return '';
  if (req.body.type === 'json' || req.body.type === 'raw') {
    let content = req.body.rawContent || '';
    if (envVars) {
      for (const [k, v] of Object.entries(envVars)) {
        content = content.split(`{{${k}}}`).join(v);
      }
    }
    return content;
  }
  if (req.body.type === 'form-urlencoded') {
    const active = req.body.formData.filter(f => f.enabled && !f.key.endsWith('~') && f.key.trim());
    return active
      .map(f => {
        let val = f.value;
        if (envVars) {
          for (const [k, v] of Object.entries(envVars)) {
            val = val.split(`{{${k}}}`).join(v);
          }
        }
        return `${encodeURIComponent(f.key)}=${encodeURIComponent(val)}`;
      })
      .join('&');
  }
  return '';
}

export function getRelativeRequestPath(collectionName: string, requestName: string): string {
  const sanitize = (s: string) => s.toLowerCase().replace(/[^a-z0-9_-]+/g, '_').replace(/^_+|_+$/g, '');
  const c = sanitize(collectionName) || 'general';
  const r = sanitize(requestName) || 'request';
  return `collections/${c}/${r}.yarac`;
}
