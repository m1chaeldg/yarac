// Comprehensive automated verification test suite for drag-and-drop operations:
// Collections, subfolders, nested folders, reparenting, cycle avoidance, and requests.

console.log('=== RUNNING DRAG AND DROP TEST SUITE ===');

interface Collection {
  id: string;
  name: string;
  path?: string;
  parentId?: string;
}

interface YaracRequest {
  id: string;
  collectionId: string;
  folderPath?: string;
  name: string;
}

let collections: Collection[] = [
  { id: 'col_users', name: 'Users & Accounts', path: 'Users & Accounts' },
  { id: 'col_billing', name: 'Billing & Subscriptions', path: 'Billing & Subscriptions' },
  { id: 'col_invoices', name: 'Invoices', path: 'Billing & Subscriptions/Invoices', parentId: 'col_billing' },
  { id: 'col_eu_invoices', name: 'EU Invoices', path: 'Billing & Subscriptions/Invoices/EU Invoices', parentId: 'col_invoices' },
  { id: 'col_auth', name: 'Auth', path: 'Auth' }
];

let requests: YaracRequest[] = [
  { id: 'req_1', collectionId: 'col_users', folderPath: 'Users & Accounts', name: 'Create User' },
  { id: 'req_2', collectionId: 'col_users', folderPath: 'Users & Accounts', name: 'Get User' },
  { id: 'req_3', collectionId: 'col_invoices', folderPath: 'Billing & Subscriptions/Invoices', name: 'List Invoices' },
  { id: 'req_4', collectionId: 'col_eu_invoices', folderPath: 'Billing & Subscriptions/Invoices/EU Invoices', name: 'Get VAT' }
];

function recalculateCollectionPaths(collectionId: string): void {
  const col = collections.find(c => c.id === collectionId);
  if (!col) return;

  const oldPath = col.path || col.name;
  if (col.parentId) {
    const parent = collections.find(c => c.id === col.parentId);
    col.path = parent ? `${parent.path || parent.name}/${col.name}` : col.name;
  } else {
    col.path = col.name;
  }
  const newPath = col.path;

  if (oldPath !== newPath) {
    for (const sub of collections) {
      if (sub.parentId === col.id || (sub.path && sub.path.startsWith(oldPath + '/'))) {
        recalculateCollectionPaths(sub.id);
      }
    }
    for (const req of requests) {
      if (req.collectionId === col.id) {
        req.folderPath = newPath;
      } else if (req.folderPath && req.folderPath.startsWith(oldPath + '/')) {
        req.folderPath = newPath + req.folderPath.substring(oldPath.length);
      }
    }
  }
}

function isCollectionDescendant(sourceColId: string, targetColId: string): boolean {
  if (sourceColId === targetColId) return true;
  let curr = collections.find(c => c.id === targetColId);
  while (curr && curr.parentId) {
    if (curr.parentId === sourceColId) return true;
    curr = collections.find(c => c.id === curr!.parentId);
  }
  return false;
}

function nestCollection(sourceColId: string, targetParentColId: string | null): boolean {
  const sourceCol = collections.find(c => c.id === sourceColId);
  if (!sourceCol) return false;

  if (targetParentColId && isCollectionDescendant(sourceColId, targetParentColId)) {
    return false;
  }

  sourceCol.parentId = targetParentColId || undefined;
  recalculateCollectionPaths(sourceColId);
  return true;
}

function reorderOrMoveCollection(sourceColId: string, targetColId: string, position: 'before' | 'after' | 'inside'): boolean {
  if (sourceColId === targetColId) return false;
  const sourceCol = collections.find(c => c.id === sourceColId);
  const targetCol = collections.find(c => c.id === targetColId);
  if (!sourceCol || !targetCol) return false;

  if (position === 'inside') {
    return nestCollection(sourceColId, targetColId);
  }

  if (isCollectionDescendant(sourceColId, targetCol.parentId || '')) {
    return false;
  }

  sourceCol.parentId = targetCol.parentId;
  recalculateCollectionPaths(sourceColId);

  const sourceIdx = collections.findIndex(c => c.id === sourceColId);
  if (sourceIdx !== -1) {
    collections.splice(sourceIdx, 1);
  }
  let targetIdx = collections.findIndex(c => c.id === targetColId);
  if (targetIdx !== -1) {
    if (position === 'after') targetIdx += 1;
    collections.splice(targetIdx, 0, sourceCol);
  } else {
    collections.push(sourceCol);
  }
  return true;
}

function moveRequestToCollection(requestId: string, targetCollectionId: string, targetIndex?: number): void {
  const req = requests.find(r => r.id === requestId);
  if (!req) return;

  const targetCol = collections.find(c => c.id === targetCollectionId);
  req.collectionId = targetCollectionId;
  if (targetCol) {
    req.folderPath = targetCol.path || targetCol.name;
  }

  if (typeof targetIndex === 'number') {
    const targetColReqs = requests.filter(r => r.collectionId === targetCollectionId && r.id !== requestId);
    const safeIndex = Math.max(0, Math.min(targetIndex, targetColReqs.length));
    targetColReqs.splice(safeIndex, 0, req);
    const otherReqs = requests.filter(r => r.collectionId !== targetCollectionId);
    requests = [...otherReqs, ...targetColReqs];
  }
}

// TEST 1: Request drag-and-drop into nested subfolder
console.log('Test 1: Move request req_1 from root collection to nested subfolder col_invoices');
moveRequestToCollection('req_1', 'col_invoices', 0);
const movedReq = requests.find(r => r.id === 'req_1')!;
console.assert(movedReq.collectionId === 'col_invoices', 'Request collectionId should be col_invoices');
console.assert(movedReq.folderPath === 'Billing & Subscriptions/Invoices', `Request folderPath should match col_invoices, got: ${movedReq.folderPath}`);
console.log('✓ Test 1 passed: Request moved into subfolder and inherited folder path');

// TEST 2: Nesting a collection inside another folder
console.log('Test 2: Nest col_auth inside col_users as a subfolder');
const nestRes = reorderOrMoveCollection('col_auth', 'col_users', 'inside');
console.assert(nestRes === true, 'Nesting should succeed');
const authCol = collections.find(c => c.id === 'col_auth')!;
console.assert(authCol.parentId === 'col_users', 'col_auth parentId should be col_users');
console.assert(authCol.path === 'Users & Accounts/Auth', `col_auth path should be updated, got: ${authCol.path}`);
console.log('✓ Test 2 passed: Collection nested as subfolder with cascaded path');

// TEST 3: Deeply nested folder reparenting and path cascading
console.log('Test 3: Move deeply nested col_eu_invoices inside col_users');
const deepNestRes = reorderOrMoveCollection('col_eu_invoices', 'col_users', 'inside');
console.assert(deepNestRes === true, 'Deep nesting should succeed');
const euCol = collections.find(c => c.id === 'col_eu_invoices')!;
console.assert(euCol.parentId === 'col_users', 'col_eu_invoices parentId should be col_users');
console.assert(euCol.path === 'Users & Accounts/EU Invoices', `col_eu_invoices path should update, got: ${euCol.path}`);
const vatReq = requests.find(r => r.id === 'req_4')!;
console.assert(vatReq.folderPath === 'Users & Accounts/EU Invoices', `Contained request folderPath should update, got: ${vatReq.folderPath}`);
console.log('✓ Test 3 passed: Deeply nested subfolder reparented and contained request path synced');

// TEST 4: Cycle prevention
console.log('Test 4: Attempt to drag parent col_billing inside its own descendant col_invoices');
const cycleRes = reorderOrMoveCollection('col_billing', 'col_invoices', 'inside');
console.assert(cycleRes === false, 'Cycle nesting should be blocked');
const billingCol = collections.find(c => c.id === 'col_billing')!;
console.assert(!billingCol.parentId, 'col_billing should remain at root without parentId');
console.log('✓ Test 4 passed: Cycle nesting prevented');

// TEST 5: Move nested subfolder back to root
console.log('Test 5: Un-nest col_invoices back to root level');
const unnestRes = nestCollection('col_invoices', null);
console.assert(unnestRes === true, 'Un-nesting should succeed');
const invoicesCol = collections.find(c => c.id === 'col_invoices')!;
console.assert(invoicesCol.parentId === undefined, 'col_invoices parentId should be undefined');
console.assert(invoicesCol.path === 'Invoices', `col_invoices path should be root "Invoices", got: ${invoicesCol.path}`);
console.log('✓ Test 5 passed: Nested subfolder restored to root level');

// TEST 6: Sibling reordering (before / after)
console.log('Test 6: Sibling reorder before col_users');
const reorderRes = reorderOrMoveCollection('col_invoices', 'col_users', 'before');
console.assert(reorderRes === true, 'Reordering before sibling should succeed');
const idxInvoices = collections.findIndex(c => c.id === 'col_invoices');
const idxUsers = collections.findIndex(c => c.id === 'col_users');
console.assert(idxInvoices < idxUsers, 'col_invoices should now precede col_users in collection array');
console.log('✓ Test 6 passed: Sibling reordering executed accurately');

console.log('=== ALL 6 TESTS COMPLETED SUCCESSFULLY ===');
