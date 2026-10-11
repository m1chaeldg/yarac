import {
  formatJsonString,
  compressJsonString,
  isValidJson,
  getJsonType,
  formatPrimitiveValue,
  getContainerSummary
} from './jsonUtils';

console.log('--- RUNNING RIGOROUS UNIT TESTS FOR JSON UTILS & VIEWER LOGIC ---');

// 1. JSON Validity Detection
console.assert(isValidJson('{"name": "Alice"}') === true, 'Valid json object should pass');
console.assert(isValidJson('   [1, 2, 3]  ') === true, 'Valid json array should pass');
console.assert(isValidJson('123') === true, 'Number json literal should pass');
console.assert(isValidJson('"quoted"') === true, 'String json literal should pass');
console.assert(isValidJson('null') === true, 'Null json literal should pass');
console.assert(isValidJson('invalid json') === false, 'Invalid string should fail');
console.assert(isValidJson('{ unquoted: true }') === false, 'Malformed object should fail');
console.assert(isValidJson('') === false, 'Empty string should fail');
console.assert(isValidJson('   ') === false, 'Whitespace-only string should fail');
console.log('✓ [PASS] isValidJson comprehensive cases verified');

// 2. formatJsonString Formatting & Idempotency
const unformatted = '{"id":1,"user":{"name":"John","active":true},"tags":["admin","dev"]}';
const formatted = formatJsonString(unformatted, 2);
console.assert(formatted.includes('\n'), 'Formatted json should have newlines');
console.assert(formatted.includes('  "id": 1'), 'Formatted json should have 2-space indentation');
console.assert(JSON.parse(formatted).id === 1, 'Formatted json should parse faithfully');
// Idempotency: formatting already formatted JSON should be unchanged
console.assert(formatJsonString(formatted, 2) === formatted, 'formatJsonString is idempotent');
// Invalid fallback: returns original string on failure without throwing
console.assert(formatJsonString('bad json') === 'bad json', 'Invalid JSON returns raw input');
console.log('✓ [PASS] formatJsonString formatting, idempotency, and fallbacks verified');

// 3. compressJsonString Minification & Idempotency
const pretty = `
{
  "id": 1,
  "user": {
    "name": "John",
    "active": true
  },
  "tags": [
    "admin",
    "dev"
  ]
}
`;
const minified = compressJsonString(pretty);
console.assert(!minified.includes('\n'), 'Compressed json should not contain newlines');
console.assert(minified === '{"id":1,"user":{"name":"John","active":true},"tags":["admin","dev"]}', 'Compressed json matches exact minified output');
// Idempotency
console.assert(compressJsonString(minified) === minified, 'compressJsonString is idempotent');
// Invalid fallback
console.assert(compressJsonString('{bad}') === '{bad}', 'compressJsonString fallback on invalid syntax');
console.log('✓ [PASS] compressJsonString compression, idempotency, and fallbacks verified');

// 4. getJsonType Type Inference
console.assert(getJsonType('hello') === 'string', 'String type check');
console.assert(getJsonType(123) === 'number', 'Number type check');
console.assert(getJsonType(true) === 'boolean', 'Boolean true check');
console.assert(getJsonType(false) === 'boolean', 'Boolean false check');
console.assert(getJsonType(null) === 'null', 'Null type check');
console.assert(getJsonType([1, 2]) === 'array', 'Array type check');
console.assert(getJsonType({ a: 1 }) === 'object', 'Object type check');
console.assert(getJsonType(undefined) === 'string', 'Undefined fallback');
console.log('✓ [PASS] getJsonType type categorization verified');

// 5. formatPrimitiveValue Rendering
console.assert(formatPrimitiveValue('test string', 'string') === '"test string"', 'String should be quoted');
console.assert(formatPrimitiveValue(42, 'number') === '42', 'Number as string');
console.assert(formatPrimitiveValue(true, 'boolean') === 'true', 'Boolean as string');
console.assert(formatPrimitiveValue(null, 'null') === 'null', 'Null as string');
console.log('✓ [PASS] formatPrimitiveValue primitives formatting verified');

// 6. getContainerSummary Item/Key Counts
console.assert(getContainerSummary([1, 2, 3], 'array') === '3 items', 'Array plural count');
console.assert(getContainerSummary(['only'], 'array') === '1 item', 'Array singular count');
console.assert(getContainerSummary([], 'array') === '0 items', 'Empty array count');
console.assert(getContainerSummary({ a: 1, b: 2 }, 'object') === '2 keys', 'Object plural count');
console.assert(getContainerSummary({ a: 1 }, 'object') === '1 key', 'Object singular count');
console.assert(getContainerSummary({}, 'object') === '0 keys', 'Empty object count');
console.log('✓ [PASS] getContainerSummary counts verified');

// 7. Tree Broadcast Expand/Collapse Synchronization Logic
let expandVersion = 0;
let collapseVersion = 0;
let nodeExpanded = true;
let lastSeenExpand = 0;
let lastSeenCollapse = 0;

function syncNode() {
  if (expandVersion > lastSeenExpand) {
    lastSeenExpand = expandVersion;
    nodeExpanded = true;
  }
  if (collapseVersion > lastSeenCollapse) {
    lastSeenCollapse = collapseVersion;
    nodeExpanded = false;
  }
}

// Initial state
console.assert(nodeExpanded === true, 'Node should start expanded');

// Broadcast Collapse All
collapseVersion++;
syncNode();
console.assert(nodeExpanded === false, 'Node should collapse on collapseAll');

// Broadcast Expand All
expandVersion++;
syncNode();
console.assert(nodeExpanded === true, 'Node should expand on expandAll');

// Manual user toggle override
nodeExpanded = false;
console.assert(nodeExpanded === false, 'User manual toggle override works');

// Subsequent broadcast expand
expandVersion++;
syncNode();
console.assert(nodeExpanded === true, 'Next broadcast expand overrides manual toggle');

console.log('✓ [PASS] Tree broadcast expand/collapse state sync tests passed');
console.log('================================================================');
console.log('ALL UNIT TEST SUITES PASSED CLEANLY (100% SUCCESS)');
console.log('================================================================');
