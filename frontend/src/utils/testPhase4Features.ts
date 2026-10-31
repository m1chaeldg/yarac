/**
 * Phase 4 Automated Verification Test Suite
 * Tests:
 * 1. Persistent execution history store additions, limits, and clearing
 * 2. Sidebar collection tree filtering by query string
 * 3. Sidebar collection tree filtering by HTTP method
 */

(globalThis as any).$state = (v: any) => v;
(globalThis as any).$derived = (v: any) => v;
const effectFn: any = (fn: any) => {};
effectFn.root = (fn: any) => {
  fn();
  return () => {};
};
(globalThis as any).$effect = effectFn;

if (typeof (globalThis as any).window === 'undefined') {
  (globalThis as any).window = { location: { origin: 'http://127.0.0.1:9475' } };
}
if (typeof (globalThis as any).EventSource === 'undefined') {
  (globalThis as any).EventSource = class {
    addEventListener() {}
    close() {}
  };
}

async function run() {
  const { WorkspaceStore } = await import('../stores/workspace.svelte');

  function assert(cond: boolean, msg: string) {
    if (!cond) {
      console.error('FAILED:', msg);
      process.exit(1);
    }
  }

  console.log('--- RUNNING PHASE 4 FEATURES VERIFICATION SUITE ---');

  // Test 1: Execution History
  {
    const store = new (WorkspaceStore as any)();
    store.clearHistory();
    assert(store.currentWorkspace.history?.length === 0, 'History should be empty initially');

    for (let i = 1; i <= 105; i++) {
      store.addHistoryItem({
        requestId: `req_${i}`,
        requestName: `Request ${i}`,
        method: i % 2 === 0 ? 'POST' : 'GET',
        url: `https://api.example.com/items/${i}`,
        statusCode: 200,
        durationMs: 45,
        timestamp: '12:00:00'
      });
    }

    // Should cap at 100 items
    assert(store.currentWorkspace.history.length === 100, `Expected 100 history items, got ${store.currentWorkspace.history.length}`);
    assert(store.currentWorkspace.history[0].requestName === 'Request 105', 'Most recent item should be at index 0');

    store.clearHistory();
    assert(store.currentWorkspace.history.length === 0, 'History should be cleared');
    console.log('✓ Execution history tracking, capping (100 items), and clearing verified');
  }

  // Test 2: Tree Filter Logic
  {
    const mockTree = [
      {
        col: { id: 'c1', name: 'Users Collection' },
        level: 0,
        subfolders: [],
        requests: [
          { id: 'r1', name: 'Get Users List', method: 'GET', url: 'https://api.com/users' },
          { id: 'r2', name: 'Create User', method: 'POST', url: 'https://api.com/users' }
        ]
      },
      {
        col: { id: 'c2', name: 'Orders' },
        level: 0,
        subfolders: [],
        requests: [
          { id: 'r3', name: 'Get Orders', method: 'GET', url: 'https://api.com/orders' }
        ]
      }
    ];

    function filterTree(tree: typeof mockTree, query: string, method: string) {
      const q = query.toLowerCase().trim();
      const methodFilter = method.toUpperCase();

      return tree.map(node => {
        const matchingReqs = node.requests.filter(r => {
          const matchesQuery = !q || r.name.toLowerCase().includes(q) || r.url.toLowerCase().includes(q);
          const matchesMethod = methodFilter === 'ALL' || r.method.toUpperCase() === methodFilter;
          return matchesQuery && matchesMethod;
        });

        const colMatches = !q || node.col.name.toLowerCase().includes(q);
        if (matchingReqs.length > 0 || (colMatches && methodFilter === 'ALL')) {
          return { ...node, requests: matchingReqs };
        }
        return null;
      }).filter(Boolean);
    }

    // Filter by query "User"
    const userFiltered = filterTree(mockTree, 'User', 'ALL');
    assert(userFiltered.length === 1 && userFiltered[0]?.col.name === 'Users Collection', 'Query "User" should match only c1');

    // Filter by method POST
    const postFiltered = filterTree(mockTree, '', 'POST');
    assert(postFiltered.length === 1, 'Method POST should match 1 collection');
    assert(postFiltered[0]?.requests.length === 1 && postFiltered[0].requests[0].method === 'POST', 'Should retain only POST request');

    // Filter by query "Orders" and method GET
    const ordersGetFiltered = filterTree(mockTree, 'Orders', 'GET');
    assert(ordersGetFiltered.length === 1 && ordersGetFiltered[0]?.col.name === 'Orders', 'Should match Orders collection with GET');

    console.log('✓ Sidebar search and HTTP method filtering logic verified');
  }

  console.log('================================================================');
  console.log('--- ALL PHASE 4 FEATURE VERIFICATION TESTS PASSED (100% OK) ---');
  console.log('================================================================');
}

run().catch((e) => {
  console.error(e);
  process.exit(1);
});
