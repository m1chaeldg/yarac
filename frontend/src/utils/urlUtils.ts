import type { KeyValuePair } from '../stores/workspace.svelte';

/**
 * Extracts path variables matching :param or {param} from a URL
 */
export function extractPathVariables(url: string): string[] {
  if (!url) return [];
  const matches = new Set<string>();

  // Match :param where it is not part of protocol (e.g. http: or https:) or port (:8080)
  const colonRegex = /(?<!https?):\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g;
  let match;
  while ((match = colonRegex.exec(url)) !== null) {
    if (match[1]) matches.add(match[1]);
  }

  // Match single-brace {param} (skipping double-brace {{...}} template variables)
  const braceRegex = /(?<!\{)\{([a-zA-Z_][a-zA-Z0-9_]*)\}(?!\})/g;
  while ((match = braceRegex.exec(url)) !== null) {
    if (match[1]) matches.add(match[1]);
  }

  return Array.from(matches);
}

/**
 * Synchronizes query parameters parsed from URL string into KeyValuePair array
 */
export function syncQueryParamsFromUrl(url: string, existingParams: KeyValuePair[] = []): KeyValuePair[] {
  if (!url) return existingParams;
  const questionIdx = url.indexOf('?');
  if (questionIdx === -1) {
    return existingParams;
  }
  const queryString = url.slice(questionIdx + 1);
  if (!queryString.trim()) return existingParams;

  const pairs = queryString.split('&');
  const result: KeyValuePair[] = [];
  const existingMap = new Map<string, KeyValuePair>();
  for (const p of existingParams) {
    if (p.key) existingMap.set(p.key.trim(), p);
  }

  for (const pair of pairs) {
    if (!pair.trim()) continue;
    const eqIdx = pair.indexOf('=');
    const rawKey = eqIdx !== -1 ? pair.slice(0, eqIdx) : pair;
    const rawVal = eqIdx !== -1 ? pair.slice(eqIdx + 1) : '';
    let key = rawKey;
    let value = rawVal;
    try {
      key = decodeURIComponent(rawKey);
      value = decodeURIComponent(rawVal);
    } catch (_) {}

    const existing = existingMap.get(key);
    if (existing) {
      result.push({
        ...existing,
        value,
        enabled: true
      });
      existingMap.delete(key);
    } else {
      result.push({
        id: Math.random().toString(36).substring(2, 9),
        key,
        value,
        enabled: true,
        description: ''
      });
    }
  }

  // Retain remaining disabled params
  for (const [_, p] of existingMap) {
    result.push({ ...p, enabled: false });
  }

  return result;
}

/**
 * Builds full URL by combining base URL path and enabled query parameters
 */
export function buildUrlWithQueryParams(baseUrl: string, params: KeyValuePair[] = []): string {
  if (!baseUrl) return '';
  const cleanBase = baseUrl.split('?')[0];
  const enabledParams = params.filter(p => p.enabled && p.key.trim() && !p.key.endsWith('~'));
  if (enabledParams.length === 0) return cleanBase;

  const queryParts = enabledParams.map(p => {
    return `${encodeURIComponent(p.key.trim())}=${encodeURIComponent(p.value || '')}`;
  });

  return `${cleanBase}?${queryParts.join('&')}`;
}

/**
 * Replaces path variables (:param or {param}) in URL with provided path variable values
 */
export function substitutePathVariables(url: string, pathVars: KeyValuePair[] = []): string {
  if (!url) return '';
  let resolved = url;
  for (const pv of pathVars) {
    if (pv.key && pv.value !== undefined) {
      resolved = resolved.replace(new RegExp(`:${pv.key}\\b`, 'g'), encodeURIComponent(pv.value));
      resolved = resolved.replace(new RegExp(`\\{${pv.key}\\}`, 'g'), encodeURIComponent(pv.value));
    }
  }
  return resolved;
}
