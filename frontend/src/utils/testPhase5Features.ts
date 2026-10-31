import {
  evaluateJsonPath,
  searchJson,
  isJsonPathExpression
} from './jsonUtils';
import { computeWaterfallTiming } from './responseMetrics';

function assert(condition: boolean, msg: string) {
  if (!condition) {
    throw new Error(`Assertion failed: ${msg}`);
  }
}

console.log('=== Running Phase 5 Automated Verification ===');

// Test 1: JSONPath evaluation
const sampleData = {
  store: {
    book: [
      { category: 'reference', author: 'Nigel Rees', title: 'Sayings of the Century', price: 8.95 },
      { category: 'fiction', author: 'Evelyn Waugh', title: 'Sword of Honour', price: 12.99 },
      { category: 'fiction', author: 'Herman Melville', title: 'Moby Dick', isbn: '0-553-21311-3', price: 8.99 }
    ],
    bicycle: {
      color: 'red',
      price: 19.95
    }
  },
  user: {
    name: 'Alice',
    email: 'alice@example.com'
  }
};

const root = evaluateJsonPath(sampleData, '$');
assert(root.store !== undefined, 'Root $ should return entire object');

const userEmail = evaluateJsonPath(sampleData, '$.user.email');
assert(userEmail === 'alice@example.com', `Expected alice@example.com, got ${userEmail}`);

const firstBookAuthor = evaluateJsonPath(sampleData, '$.store.book[0].author');
assert(firstBookAuthor === 'Nigel Rees', `Expected Nigel Rees, got ${firstBookAuthor}`);

const allBookPrices = evaluateJsonPath(sampleData, '$.store.book[*].price');
assert(Array.isArray(allBookPrices) && allBookPrices.length === 3, 'Expected array of 3 book prices');
assert(allBookPrices[0] === 8.95 && allBookPrices[1] === 12.99, 'Book prices match');

const allTitles = evaluateJsonPath(sampleData, '$..title');
assert(Array.isArray(allTitles) && allTitles.length === 3, 'Recursive descent $..title should find 3 titles');
assert(allTitles.includes('Moby Dick'), 'Titles includes Moby Dick');

console.log('✓ Test 1 Passed: JSONPath evaluation works across dot notation, arrays, wildcards, and recursive descent');

// Test 2: JSON Text Search
const searchMatches = searchJson(sampleData, 'fiction');
assert(searchMatches.length === 2, `Expected 2 matches for 'fiction', got ${searchMatches.length}`);
assert(searchMatches[0].path === '$.store.book[1].category', 'First match is book 1 category');
assert(searchMatches[1].path === '$.store.book[2].category', 'Second match is book 2 category');

const keyMatches = searchJson(sampleData, 'bicycle');
assert(keyMatches.length >= 1, 'Expected at least 1 match for key bicycle');
assert(keyMatches[0].key === 'bicycle', 'Key match found');

console.log('✓ Test 2 Passed: JSON text search finds keys and values with accurate JSON paths');

// Test 3: JSONPath Detection
assert(isJsonPathExpression('$.store.book'), '$.store.book is JSONPath');
assert(isJsonPathExpression('store.book[0]'), 'store.book[0] is JSONPath');
assert(isJsonPathExpression('items[*].name'), 'items[*].name is JSONPath');
assert(!isJsonPathExpression('plain search query'), 'plain search query is not JSONPath');

console.log('✓ Test 3 Passed: JSONPath expression syntax detector');

// Test 4: Network Waterfall Timing Breakdown
const timing = computeWaterfallTiming(200);
assert(timing.totalMs === 200, 'Total ms is 200');
assert(timing.dnsLookupMs > 0, 'DNS lookup ms > 0');
assert(timing.tcpHandshakeMs > 0, 'TCP handshake ms > 0');
assert(timing.tlsHandshakeMs > 0, 'TLS handshake ms > 0');
assert(timing.ttfbMs > 0, 'TTFB ms > 0');
assert(timing.contentTransferMs > 0, 'Content transfer ms > 0');

const sum = timing.dnsLookupMs + timing.tcpHandshakeMs + timing.tlsHandshakeMs + timing.ttfbMs + timing.contentTransferMs;
assert(sum === timing.totalMs, `Expected sum ${sum} to equal total ${timing.totalMs}`);

const customTiming = computeWaterfallTiming(150, {
  dnsLookupMs: 10,
  tcpHandshakeMs: 20,
  tlsHandshakeMs: 30,
  ttfbMs: 60,
  contentTransferMs: 30,
  totalMs: 150
});
assert(customTiming.dnsLookupMs === 10 && customTiming.ttfbMs === 60, 'Custom timing breakdown preserved');

console.log('✓ Test 4 Passed: Waterfall timing breakdown accurately divides latency phases');

console.log('=== All Phase 5 Feature Tests Succeeded ===');
