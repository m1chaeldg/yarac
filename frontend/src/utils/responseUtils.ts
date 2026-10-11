/**
 * Response Viewer utilities for table flattening, sorting, collection discovery, and header filtering.
 */

export interface FlatRow {
  [key: string]: any;
}

export interface CollectionOption {
  id: string;
  label: string;
  isCollection: boolean;
  itemCount: number;
  path: string;
}

export type ObjectViewMode = 'properties' | 'record';

export interface TableResult {
  columns: string[];
  rows: FlatRow[];
  collections: CollectionOption[];
  isObject: boolean;
  selectedCollectionId: string;
}

/**
 * Flattens nested object properties into dot-notation keys.
 */
export function flattenObject(obj: any, prefix = '', maxDepth = 4, currentDepth = 0): FlatRow {
  const result: FlatRow = {};
  if (obj === null || obj === undefined) return result;

  for (const key of Object.keys(obj)) {
    const val = obj[key];
    const newKey = prefix ? `${prefix}.${key}` : key;

    if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
      if (currentDepth < maxDepth) {
        Object.assign(result, flattenObject(val, newKey, maxDepth, currentDepth + 1));
      } else {
        result[newKey] = JSON.stringify(val);
      }
    } else if (Array.isArray(val)) {
      if (val.length === 0) {
        result[newKey] = '[]';
      } else if (typeof val[0] === 'object' && val[0] !== null) {
        result[newKey] = `[${val.length} items]`;
      } else {
        // Array of primitives
        result[newKey] = val.map(v => (v === null ? 'null' : String(v))).join(', ');
      }
    } else {
      result[newKey] = val;
    }
  }
  return result;
}

/**
 * Inspects a value to determine its simple human-readable type.
 */
export function getValueType(val: any): 'string' | 'number' | 'boolean' | 'null' | 'array' | 'object' | 'undefined' {
  if (val === null) return 'null';
  if (val === undefined) return 'undefined';
  if (Array.isArray(val)) return 'array';
  return typeof val as any;
}

/**
 * Discovers available array collections and datasets within a JSON payload.
 */
export function discoverCollections(parsedJson: any, maxDepth = 2): CollectionOption[] {
  const collections: CollectionOption[] = [];
  if (!parsedJson || typeof parsedJson !== 'object') return collections;

  if (Array.isArray(parsedJson)) {
    collections.push({
      id: 'root',
      label: `Root Array (${parsedJson.length} items)`,
      isCollection: true,
      itemCount: parsedJson.length,
      path: ''
    });
    return collections;
  }

  // Root is an object
  collections.push({
    id: 'root',
    label: `Root Object (${Object.keys(parsedJson).length} fields)`,
    isCollection: false,
    itemCount: Object.keys(parsedJson).length,
    path: ''
  });

  function scan(curr: any, path: string, depth: number) {
    if (!curr || typeof curr !== 'object' || depth > maxDepth) return;

    for (const [key, val] of Object.entries(curr)) {
      const currentPath = path ? `${path}.${key}` : key;
      if (Array.isArray(val)) {
        collections.push({
          id: currentPath,
          label: `${currentPath} [${val.length} items]`,
          isCollection: true,
          itemCount: val.length,
          path: currentPath
        });
      } else if (val !== null && typeof val === 'object') {
        scan(val, currentPath, depth + 1);
      }
    }
  }

  scan(parsedJson, '', 1);
  return collections;
}

/**
 * Resolves a nested value from an object via dot-separated path.
 */
export function resolvePath(obj: any, path: string): any {
  if (!path || path === 'root') return obj;
  const parts = path.split('.');
  let curr = obj;
  for (const part of parts) {
    if (curr === null || curr === undefined || typeof curr !== 'object') return undefined;
    curr = curr[part];
  }
  return curr;
}

/**
 * Converts a single object into a 3-column Property / Type / Value representation.
 */
export function generateObjectPropertiesTable(obj: any): { columns: string[]; rows: FlatRow[] } {
  const flat = flattenObject(obj);
  const rows: FlatRow[] = Object.entries(flat).map(([key, val]) => {
    return {
      Property: key,
      Type: getValueType(val),
      Value: val === null ? 'null' : (typeof val === 'object' ? JSON.stringify(val) : String(val))
    };
  });

  return {
    columns: ['Property', 'Type', 'Value'],
    rows
  };
}

/**
 * Parses and transforms JSON into tabular format with collection picking and object mode support.
 */
export function parseTableData(
  parsedJson: any,
  selectedCollectionId = 'root',
  objectMode: ObjectViewMode = 'properties'
): TableResult {
  if (!parsedJson || (typeof parsedJson !== 'object' && !Array.isArray(parsedJson))) {
    return { columns: [], rows: [], collections: [], isObject: false, selectedCollectionId: 'root' };
  }

  const collections = discoverCollections(parsedJson);

  // Default selection: if root is an object but contains a collection, can still default to root or chosen
  let activeId = selectedCollectionId;
  const exists = collections.find(c => c.id === activeId);
  if (!exists) {
    // If selected doesn't exist, default to first collection or root
    activeId = collections[0]?.id || 'root';
  }

  const targetData = activeId === 'root' ? parsedJson : resolvePath(parsedJson, activeId);

  let rows: FlatRow[] = [];
  let columns: string[] = [];
  const isTargetObject = targetData !== null && typeof targetData === 'object' && !Array.isArray(targetData);

  if (Array.isArray(targetData)) {
    rows = targetData.map((item, idx) => {
      if (typeof item === 'object' && item !== null) {
        return { '#': idx + 1, ...flattenObject(item) };
      }
      return { '#': idx + 1, Value: item === null ? 'null' : String(item) };
    });

    const colSet = new Set<string>();
    rows.forEach(r => Object.keys(r).forEach(k => colSet.add(k)));
    columns = Array.from(colSet);
    if (columns.includes('#')) {
      columns.splice(columns.indexOf('#'), 1);
      columns.unshift('#');
    }
  } else if (isTargetObject) {
    if (objectMode === 'properties') {
      const propTable = generateObjectPropertiesTable(targetData);
      columns = propTable.columns;
      rows = propTable.rows;
    } else {
      // Record mode: 1 single row, all properties as columns
      const flat = flattenObject(targetData);
      rows = [{ '#': 1, ...flat }];
      columns = ['#', ...Object.keys(flat)];
    }
  }

  return {
    columns,
    rows,
    collections,
    isObject: isTargetObject,
    selectedCollectionId: activeId
  };
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

