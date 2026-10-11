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
