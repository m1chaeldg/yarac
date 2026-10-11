import { flattenObject, parseTableData, filterAndSortHeaders, discoverCollections } from './responseUtils.ts';

console.log('--- RUNNING RESPONSE UTILS TEST SUITE ---');

// Test 1: Flatten shallow object
const shallow = { a: 1, b: 'text', c: true };
const flatShallow = flattenObject(shallow);
console.assert(flatShallow['a'] === 1 && flatShallow['b'] === 'text' && flatShallow['c'] === true, 'Shallow flatten failed');
console.log('✓ Shallow object flattening');

// Test 2: Flatten deep nested object with array of primitives
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
console.assert(flatNested['user.profile.roles'] === 'admin, developer', 'Array of primitives flattening failed');
console.assert(flatNested['active'] === true, 'Top-level key failed');
console.log('✓ Deep nested object flattening & primitive arrays');

// Test 3: Table data extraction from JSON array
const jsonArray = [
  { id: 1, name: 'Item 1' },
  { id: 2, name: 'Item 2', extra: 'value' }
];
const tableDataArray = parseTableData(jsonArray);
console.assert(tableDataArray.rows.length === 2, `Expected 2 rows, got ${tableDataArray.rows.length}`);
console.assert(tableDataArray.columns.includes('name'), 'Columns must include name');
console.assert(tableDataArray.columns.includes('id'), 'Columns must include id');
console.assert(tableDataArray.columns[0] === '#', 'First column must be #');
console.log('✓ JSON array table extraction');

// Test 4: Single Object in Properties Mode
const singleObj = { id: 42, username: 'octocat', meta: { isVerified: true, score: 98.5 } };
const tableDataObjProps = parseTableData(singleObj, 'root', 'properties');
console.assert(tableDataObjProps.isObject === true, 'Should mark isObject as true');
console.assert(tableDataObjProps.columns.join(',') === 'Property,Type,Value', 'Properties table columns must be Property,Type,Value');
console.assert(tableDataObjProps.rows.length === 4, `Expected 4 properties, got ${tableDataObjProps.rows.length}`);
const idRow = tableDataObjProps.rows.find(r => r.Property === 'id');
console.assert(idRow && idRow.Type === 'number' && idRow.Value === '42', 'Row type and value matching failed');
console.log('✓ Single Object in Properties mode');

// Test 5: Single Object in Record Mode
const tableDataObjRecord = parseTableData(singleObj, 'root', 'record');
console.assert(tableDataObjRecord.rows.length === 1, 'Expected 1 record row');
console.assert(tableDataObjRecord.columns.includes('meta.isVerified'), 'Record mode should have meta.isVerified column');
console.log('✓ Single Object in Record mode');

// Test 6: Embedded Collection Discovery & Selection
const wrappedResponse = {
  status: 'ok',
  count: 2,
  items: [
    { sku: 'A1', price: 10 },
    { sku: 'B2', price: 20 }
  ]
};
const collections = discoverCollections(wrappedResponse);
console.assert(collections.length === 2, `Expected 2 collection options, got ${collections.length}`);
console.assert(collections.some(c => c.id === 'items' && c.itemCount === 2), 'Failed to discover items collection');

const tableDataEmbedded = parseTableData(wrappedResponse, 'items');
console.assert(tableDataEmbedded.rows.length === 2, `Expected 2 rows for items, got ${tableDataEmbedded.rows.length}`);
console.assert(tableDataEmbedded.columns.includes('sku'), 'Columns must include sku');
console.assert(tableDataEmbedded.isObject === false, 'Target should be array, not object');
console.log('✓ Embedded collection discovery & selection');

// Test 7: Filter and sort headers
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

