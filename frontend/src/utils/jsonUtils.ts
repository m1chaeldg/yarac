/**
 * JSON formatting, compression, type inspection, and tree-model utilities.
 * Pure functions with zero external dependencies, easily testable and maintainable.
 */

export type JsonType = 'string' | 'number' | 'boolean' | 'null' | 'object' | 'array';

/**
 * Validates whether the given string is valid JSON syntax.
 */
export function isValidJson(raw: string): boolean {
  if (!raw || !raw.trim()) return false;
  try {
    JSON.parse(raw);
    return true;
  } catch {
    return false;
  }
}

/**
 * Formats a JSON string with indentation (default 2 spaces).
 * Returns original raw string if invalid JSON.
 */
export function formatJsonString(raw: string, indent = 2): string {
  if (!raw || !raw.trim()) return raw;
  try {
    const parsed = JSON.parse(raw);
    return JSON.stringify(parsed, null, indent);
  } catch {
    return raw;
  }
}

/**
 * Minifies/compresses a JSON string without extra whitespace.
 * Returns original raw string if invalid JSON.
 */
export function compressJsonString(raw: string): string {
  if (!raw || !raw.trim()) return raw;
  try {
    const parsed = JSON.parse(raw);
    return JSON.stringify(parsed);
  } catch {
    return raw;
  }
}

/**
 * Returns the exact JSON type category of a value.
 */
export function getJsonType(val: unknown): JsonType {
  if (val === null) return 'null';
  if (Array.isArray(val)) return 'array';
  const t = typeof val;
  if (t === 'string') return 'string';
  if (t === 'number') return 'number';
  if (t === 'boolean') return 'boolean';
  if (t === 'object') return 'object';
  return 'string';
}

/**
 * Formats a primitive JSON value for visual inspection.
 */
export function formatPrimitiveValue(val: unknown, type: JsonType): string {
  if (type === 'null') return 'null';
  if (type === 'string') return JSON.stringify(val);
  return String(val);
}

/**
 * Helper to compute count summary for container structures.
 */
export function getContainerSummary(val: unknown, type: JsonType): string {
  if (type === 'array' && Array.isArray(val)) {
    return `${val.length} ${val.length === 1 ? 'item' : 'items'}`;
  }
  if (type === 'object' && val && typeof val === 'object') {
    const len = Object.keys(val).length;
    return `${len} ${len === 1 ? 'key' : 'keys'}`;
  }
  return '';
}

export interface JsonSearchMatch {
  path: string;
  key: string;
  value: any;
  matchType: 'key' | 'value';
}

/**
 * Checks whether the input string resembles a JSONPath expression.
 */
export function isJsonPathExpression(query: string): boolean {
  const trimmed = query.trim();
  if (trimmed.startsWith('$') || trimmed.includes('[*]')) return true;
  return /^[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+|\[\d+\])+$/.test(trimmed);
}

/**
 * Recursively searches a JSON object/array for keys or values matching the query string.
 */
export function searchJson(
  root: any,
  query: string,
  maxResults = 100
): JsonSearchMatch[] {
  const matches: JsonSearchMatch[] = [];
  if (!query || !query.trim() || root === null || root === undefined) {
    return matches;
  }
  const needle = query.trim().toLowerCase();

  function walk(current: any, currentPath: string, keyName: string) {
    if (matches.length >= maxResults) return;

    if (keyName && keyName.toLowerCase().includes(needle)) {
      matches.push({
        path: currentPath,
        key: keyName,
        value: current,
        matchType: 'key'
      });
    }

    if (current === null || current === undefined) return;

    if (typeof current !== 'object') {
      const valStr = String(current).toLowerCase();
      if (valStr.includes(needle)) {
        matches.push({
          path: currentPath,
          key: keyName,
          value: current,
          matchType: 'value'
        });
      }
      return;
    }

    if (Array.isArray(current)) {
      for (let i = 0; i < current.length; i++) {
        walk(current[i], `${currentPath}[${i}]`, `[${i}]`);
      }
    } else {
      for (const k of Object.keys(current)) {
        const nextPath = currentPath ? `${currentPath}.${k}` : `$.${k}`;
        walk(current[k], nextPath, k);
      }
    }
  }

  walk(root, '$', '');
  return matches;
}

/**
 * Evaluates a JSONPath query against a JSON root object/array.
 * Supports $, $.prop, $.items[0], $.items[*].name, and $..prop recursive descent.
 */
export function evaluateJsonPath(root: any, expr: string): any {
  if (!expr || !expr.trim()) return root;
  let normalized = expr.trim();
  if (normalized === '$') return root;
  if (normalized.startsWith('$.')) normalized = normalized.substring(2);
  else if (normalized.startsWith('$')) normalized = normalized.substring(1);

  // Recursive descent $..prop
  if (normalized.startsWith('.')) {
    const targetProp = normalized.replace(/^\.+/, '');
    const results: any[] = [];
    function collect(node: any) {
      if (node && typeof node === 'object') {
        if (Array.isArray(node)) {
          for (const item of node) collect(item);
        } else {
          for (const [k, v] of Object.entries(node)) {
            if (k === targetProp) results.push(v);
            collect(v);
          }
        }
      }
    }
    collect(root);
    return results;
  }

  const tokens: string[] = [];
  const regex = /([a-zA-Z0-9_\-]+)|\[(\*|\d+|'[^']+'|"[^"]+")\]/g;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(normalized)) !== null) {
    if (match[1]) {
      tokens.push(match[1]);
    } else if (match[2]) {
      tokens.push(match[2].replace(/^['"]|['"]$/g, ''));
    }
  }

  if (tokens.length === 0) return root;

  let current: any[] = [root];

  for (const token of tokens) {
    const next: any[] = [];
    for (const item of current) {
      if (item === null || item === undefined) continue;

      if (token === '*') {
        if (Array.isArray(item)) {
          next.push(...item);
        } else if (typeof item === 'object') {
          next.push(...Object.values(item));
        }
      } else if (/^\d+$/.test(token)) {
        const index = parseInt(token, 10);
        if (Array.isArray(item) && index >= 0 && index < item.length) {
          next.push(item[index]);
        }
      } else {
        if (typeof item === 'object' && token in item) {
          next.push(item[token]);
        }
      }
    }
    current = next;
    if (current.length === 0) break;
  }

  if (current.length === 0) return undefined;
  if (current.length === 1 && !expr.includes('*')) return current[0];
  return current;
}
