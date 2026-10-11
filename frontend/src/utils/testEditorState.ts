import {
  createEmptyParam,
  createEmptyHeader,
  createEmptyVariable,
  setRequestBodyType,
  setAuthType,
  insertScriptSnippet,
  COMMON_HEADER_PRESETS,
  DYNAMIC_VARIABLES_LIST
} from './editorUtils.ts';
import type { YaracRequest } from '../stores/workspace.svelte.ts';

console.log('--- RUNNING EDITOR STATE UNIT TEST SUITE ---');

const baseReq: YaracRequest = {
  id: 'req_edit_1',
  collectionId: 'col_1',
  name: 'Test Request',
  method: 'GET',
  url: 'https://api.example.com',
  params: [],
  headers: [],
  variables: [],
  auth: { type: 'none' },
  body: { type: 'none', rawContent: '', formData: [] },
  settings: {
    inherit_auth: false,
    verify_ssl: true,
    follow_redirects: true,
    timeout_ms: 10000
  },
  scripts: { pre_request: '', post_response: '' },
  flow: {
    once_per_run: false,
    retry_until: '',
    retry_interval_ms: 500,
    max_retries: 3,
    delay_ms: 0,
    jitter_min_ms: 0,
    jitter_max_ms: 0
  },
  dslContent: ''
};

// 1. Param creation
const p = createEmptyParam();
console.assert(p.key === '' && p.value === '' && p.enabled === true, 'Empty param mismatch');
console.log('✓ Empty param creation');

// 2. Header creation with preset
const h1 = createEmptyHeader();
console.assert(h1.key === '' && h1.value === '', 'Empty header mismatch');
const h2 = createEmptyHeader('Content-Type');
console.assert(h2.key === 'Content-Type' && h2.value === 'application/json', 'Content-Type preset header mismatch');
console.log('✓ Header creation with presets');

// 3. Variable creation
const v = createEmptyVariable();
console.assert(v.isSecret === false && v.enabled === true, 'Empty variable mismatch');
console.log('✓ Variable creation');

// 4. Body type mutation
const reqWithBody = JSON.parse(JSON.stringify(baseReq));
setRequestBodyType(reqWithBody, 'binary');
console.assert(reqWithBody.body.type === 'binary', 'Binary body type mismatch');
console.assert(reqWithBody.body.binaryContentType === 'application/octet-stream', 'Binary content type default missing');

setRequestBodyType(reqWithBody, 'json');
console.assert(reqWithBody.body.type === 'json', 'JSON body type mismatch');
console.log('✓ Body type transitions');

// 5. Auth type mutation
const reqWithAuth = JSON.parse(JSON.stringify(baseReq));
setAuthType(reqWithAuth, 'bearer');
console.assert(reqWithAuth.auth.type === 'bearer', 'Bearer auth mismatch');
setAuthType(reqWithAuth, 'basic');
console.assert(reqWithAuth.auth.type === 'basic', 'Basic auth mismatch');
setAuthType(reqWithAuth, 'apikey');
console.assert(reqWithAuth.auth.type === 'apikey', 'API Key auth mismatch');
console.log('✓ Auth type transitions');

// 6. Script snippet injection
const existingScript = '// existing code\n';
const snippet = 'pm.test("Status 200", () => {});';
const injected = insertScriptSnippet(existingScript, snippet);
console.assert(injected.includes('// existing code') && injected.includes('pm.test'), 'Snippet injection failed');
console.log('✓ Script snippet injection');

// 7. Verify presets and dynamic variables catalogues
console.assert(COMMON_HEADER_PRESETS.includes('Content-Type'), 'Missing Content-Type in presets');
console.assert(DYNAMIC_VARIABLES_LIST.some(d => d.label === '$uuid'), 'Missing $uuid in dynamic variables');
console.log('✓ Presets and dynamic variables catalogues');

console.log('--- ALL EDITOR STATE UNIT TESTS PASSED ---');
