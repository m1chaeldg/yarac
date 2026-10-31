// Svelte 5 mock polyfills for Node test execution
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

// Mock fetch for offline node test to prevent unhandled relative /api/workspace errors
const origFetch = globalThis.fetch;
(globalThis as any).fetch = async (url: any, opts: any) => {
  if (typeof url === 'string' && url.startsWith('/api')) {
    return { ok: false, status: 404, json: async () => ({}) };
  }
  return origFetch(url, opts);
};

import {
  generateCodeVerifier,
  generateCodeChallenge,
  buildAuthorizationUrl
} from './oauthUtils';

function assert(condition: boolean, msg: string) {
  if (!condition) {
    throw new Error(`Assertion failed: ${msg}`);
  }
}

async function runTests() {
  const { WorkspaceStore } = await import('../stores/workspace.svelte');

  console.log('=== Running Phase 6 Automated Verification ===');

  // Test 1: PKCE Code Verifier Generation
  const verifier64 = generateCodeVerifier(64);
  assert(verifier64.length === 64, `Expected verifier length 64, got ${verifier64.length}`);
  const pkceCharset = /^[A-Za-z0-9\-._~]+$/;
  assert(pkceCharset.test(verifier64), 'Verifier contains only RFC 7636 valid characters');

  const verifier128 = generateCodeVerifier(128);
  assert(verifier128.length === 128, 'Verifier generates 128 characters correctly');

  console.log('✓ Test 1 Passed: PKCE Code Verifier generator meets RFC 7636 entropy and charset');

  // Test 2: RFC 7636 Appendix B Test Vector
  // RFC 7636 Test Vector:
  // code_verifier: dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
  // code_challenge: E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM
  const rfcVerifier = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';
  const expectedChallenge = 'E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM';
  const computedChallenge = await generateCodeChallenge(rfcVerifier);

  assert(
    computedChallenge === expectedChallenge,
    `RFC 7636 test vector failed! Expected: ${expectedChallenge}, Got: ${computedChallenge}`
  );

  console.log('✓ Test 2 Passed: SHA-256 base64url Code Challenge matches RFC 7636 Appendix B standard test vector');

  // Test 3: OAuth 2.0 Authorization URL Construction
  const authUrl = buildAuthorizationUrl({
    authUrl: 'https://auth.example.com/oauth/authorize',
    clientId: 'test-client-id',
    redirectUri: 'http://localhost:8080/callback',
    scope: 'openid profile email',
    state: 'state_xyz123',
    codeChallenge: computedChallenge,
    codeChallengeMethod: 'S256'
  });

  const parsedUrl = new URL(authUrl);
  assert(parsedUrl.searchParams.get('response_type') === 'code', 'response_type is code');
  assert(parsedUrl.searchParams.get('client_id') === 'test-client-id', 'client_id is test-client-id');
  assert(parsedUrl.searchParams.get('redirect_uri') === 'http://localhost:8080/callback', 'redirect_uri matches');
  assert(parsedUrl.searchParams.get('scope') === 'openid profile email', 'scope matches');
  assert(parsedUrl.searchParams.get('state') === 'state_xyz123', 'state matches');
  assert(parsedUrl.searchParams.get('code_challenge') === expectedChallenge, 'code_challenge matches');
  assert(parsedUrl.searchParams.get('code_challenge_method') === 'S256', 'code_challenge_method is S256');

  console.log('✓ Test 3 Passed: Authorization URL query parameter builder');

  // Test 4: WorkspaceStore OAuth 2.0 Type Integration
  const store = new (WorkspaceStore as any)();
  const collectionId = store.currentWorkspace.collections[0].id;
  const newReq = store.createRequest(collectionId, 'OAuth Test Req', 'GET');
  store.openTab(newReq.id);

  store.updateActiveRequest({
    auth: {
      type: 'oauth2',
      oauth2GrantType: 'authorization_code',
      oauth2ClientId: 'my-client-id',
      oauth2TokenUrl: 'https://auth.example.com/oauth/token',
      oauth2AccessToken: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.sample_oauth_token',
      bearerToken: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.sample_oauth_token'
    }
  });

  const updated = store.activeRequest;
  assert(updated !== null, 'Active request exists');
  assert(updated?.auth.type === 'oauth2', 'Auth type updated to oauth2');
  assert(updated?.auth.oauth2ClientId === 'my-client-id', 'Client ID saved');
  assert(updated?.auth.oauth2AccessToken?.includes('sample_oauth_token') === true, 'Access token retained');

  console.log('✓ Test 4 Passed: Workspace store handles OAuth 2.0 credentials and tokens');

  console.log('=== All Phase 6 Feature Tests Succeeded ===');
}

runTests().catch(e => {
  console.error(e);
  process.exit(1);
});
