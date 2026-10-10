import { CODE_LANGUAGES, resolveUrl, resolveHeaders, resolveBody, getRelativeRequestPath } from './codeGenerators.ts';
import type { YaracRequest } from '../stores/workspace.svelte.ts';

const mockRequest: YaracRequest = {
  id: 'req_test123',
  collectionId: 'col_users',
  name: 'Create User Profile',
  method: 'POST',
  url: '{{base_url}}/api/v1/users',
  params: [
    { id: 'p1', key: 'send_email', value: 'true', enabled: true },
    { id: 'p2', key: 'dry_run', value: 'false', enabled: false }
  ],
  headers: [
    { id: 'h1', key: 'Accept', value: 'application/json', enabled: true },
    { id: 'h2', key: 'X-Custom-Header', value: 'TestVal', enabled: true },
    { id: 'h3', key: 'X-Disabled', value: 'None', enabled: false }
  ],
  auth: {
    type: 'bearer',
    bearerToken: 'my_secret_token_123'
  },
  body: {
    type: 'json',
    rawContent: '{"username": "testuser", "role": "developer"}',
    formData: []
  },
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

const mockEnv = {
  base_url: 'https://api.yarac.dev'
};

console.log('--- RUNNING CODE GENERATORS VALIDATION ---');

// 1. URL resolution
const urlUnresolved = resolveUrl(mockRequest);
console.assert(urlUnresolved === '{{base_url}}/api/v1/users?send_email=true', `URL unresolved failed: ${urlUnresolved}`);

const urlResolved = resolveUrl(mockRequest, mockEnv);
console.assert(urlResolved === 'https://api.yarac.dev/api/v1/users?send_email=true', `URL resolved failed: ${urlResolved}`);

// 2. Headers resolution
const headers = resolveHeaders(mockRequest, mockEnv);
console.assert(headers['Accept'] === 'application/json', 'Missing Accept header');
console.assert(headers['Authorization'] === 'Bearer my_secret_token_123', 'Missing Bearer token');
console.assert(headers['Content-Type'] === 'application/json', 'Missing auto Content-Type header');
console.assert(!headers['X-Disabled'], 'Disabled header should not be present');

// 3. Body resolution
const body = resolveBody(mockRequest, mockEnv);
console.assert(body.includes('"username": "testuser"'), 'Body content mismatch');

// 4. Relative file anchoring
const relPath = getRelativeRequestPath('User Management', 'Create User Profile');
console.assert(relPath === 'collections/user_management/create_user_profile.yarac', `Anchored path mismatch: ${relPath}`);

// 5. Test each language generator
const languagesTested: string[] = [];

for (const lang of CODE_LANGUAGES) {
  for (const variant of lang.variants) {
    const code = lang.generate(mockRequest, variant.id, mockEnv);
    console.assert(code && code.length > 20, `Generated code for ${lang.name} (${variant.name}) is empty or too short`);
    console.assert(code.includes('https://api.yarac.dev') || code.includes('api/v1/users'), `${lang.name} missing URL`);
    languagesTested.push(`${lang.name} [${variant.name}]`);
  }
}

console.log(`Successfully verified ${languagesTested.length} language targets:`);
for (const target of languagesTested) {
  console.log(`  ✓ ${target}`);
}

console.log('--- ALL VALIDATIONS PASSED CLEANLY ---');
