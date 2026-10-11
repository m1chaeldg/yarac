/**
 * Response Viewer utilities for table flattening, sorting, and header filtering.
 */

export interface FlatRow {
  [key: string]: string | number | boolean;
}

export function flattenObject(obj: any, prefix = ''): FlatRow {
  const result: FlatRow = {};
  if (obj === null || obj === undefined) return result;

  for (const key of Object.keys(obj)) {
    const val = obj[key];
    const newKey = prefix ? `${prefix}.${key}` : key;
    if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
      Object.assign(result, flattenObject(val, newKey));
    } else if (Array.isArray(val)) {
      if (val.length > 0 && typeof val[0] === 'object') {
        result[newKey] = `[${val.length} objects]`;
      } else {
        result[newKey] = JSON.stringify(val);
      }
    } else {
      result[newKey] = val;
    }
  }
  return result;
}

export function parseTableData(parsedJson: any): { columns: string[]; rows: FlatRow[] } {
  if (!parsedJson) return { columns: [], rows: [] };

  let rows: FlatRow[] = [];
  if (Array.isArray(parsedJson)) {
    rows = parsedJson.map((item, idx) => {
      if (typeof item === 'object' && item !== null) {
        return { '#': idx + 1, ...flattenObject(item) };
      }
      return { '#': idx + 1, value: String(item) };
    });
  } else if (typeof parsedJson === 'object' && parsedJson !== null) {
    const flat = flattenObject(parsedJson);
    rows = Object.entries(flat).map(([k, v]) => ({ key: k, value: v }));
  }

  const colSet = new Set<string>();
  rows.forEach(r => Object.keys(r).forEach(k => colSet.add(k)));
  const columns = Array.from(colSet);
  if (columns.includes('#')) {
    columns.splice(columns.indexOf('#'), 1);
    columns.unshift('#');
  }

  return { columns, rows };
}

export function filterAndSortHeaders(
  headers: Record<string, string> | undefined,
  search: string
): Array<{ key: string; value: string }> {
  if (!headers) return [];
  const entries = Object.entries(headers).map(([key, value]) => ({ key, value }));
  if (!search.trim()) return entries;
  const s = search.toLowerCase();
  return entries.filter(h => h.key.toLowerCase().includes(s) || h.value.toLowerCase().includes(s));
}
