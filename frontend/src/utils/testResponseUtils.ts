import { flattenObject, parseTableData, filterAndSortHeaders } from './responseUtils.ts';

console.log('--- RUNNING RESPONSE UTILS TEST SUITE ---');

// Test 1: Flatten shallow object
const shallow = { a: 1, b: 'text', c: true };
const flatShallow = flattenObject(shallow);
console.assert(flatShallow['a'] === 1 && flatShallow['b'] === 'text' && flatShallow['c'] === true, 'Shallow flatten failed');
console.log('✓ Shallow object flattening');

// Test 2: Flatten deep nested object
const nested = {
  user: {
    id: 101,
    profile: {
      name: 'Alice',
      roles: ['admin', 'developer']
    }
  },
  active: true
};
const flatNested = flattenObject(nested);
console.assert(flatNested['user.id'] === 101, 'Nested key user.id failed');
console.assert(flatNested['user.profile.name'] === 'Alice', 'Nested key user.profile.name failed');
console.assert(flatNested['active'] === true, 'Top-level key failed');
console.log('✓ Deep nested object flattening');

// Test 3: Table data extraction from JSON array
const jsonArray = [
  { id: 1, name: 'Item 1' },
  { id: 2, name: 'Item 2', extra: 'value' }
];
const tableData = parseTableData(jsonArray);
console.assert(tableData.rows.length === 2, `Expected 2 rows, got ${tableData.rows.length}`);
console.assert(tableData.columns.includes('name'), 'Columns must include name');
console.assert(tableData.columns.includes('id'), 'Columns must include id');
console.log('✓ JSON array table extraction');

// Test 4: Filter and sort headers
const sampleHeaders = {
  'Content-Type': 'application/json',
  'Authorization': 'Bearer 12345',
  'X-Trace-Id': 'abc-xyz',
  'Cache-Control': 'no-cache'
};
const filtered = filterAndSortHeaders(sampleHeaders, 'trace');
console.assert(filtered.length === 1, `Expected 1 match for 'trace', got ${filtered.length}`);
console.assert(filtered[0].key === 'X-Trace-Id', 'Key mismatch for trace filter');
console.log('✓ Headers search filtering');

console.log('--- ALL RESPONSE UTILS TESTS PASSED CLEANLY ---');
