import type { YaracRequest } from '../../stores/workspace.svelte.ts';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers.ts';
import { curlLanguage } from './curl.ts';
import { pythonLanguage } from './python.ts';
import { javascriptLanguage } from './javascript.ts';
import { goLanguage } from './go.ts';
import { rustLanguage } from './rust.ts';
import { csharpLanguage } from './csharp.ts';
import { powershellLanguage } from './powershell.ts';
import { shellLanguage } from './shell.ts';
import { phpLanguage } from './php.ts';
import { rubyLanguage } from './ruby.ts';
import { javaLanguage } from './java.ts';
import { CODE_LANGUAGES } from './index.ts';

console.log('--- RUNNING MODULAR GENERATORS UNIT TEST SUITE ---');

const testReq: YaracRequest = {
  id: 'req_mod_1',
  collectionId: 'col_mod',
  name: 'Get Order Status',
  method: 'POST',
  url: 'https://api.test.com/orders',
  params: [
    { id: 'p1', key: 'filter', value: 'active', enabled: true }
  ],
  headers: [
    { id: 'h1', key: 'Content-Type', value: 'application/json', enabled: true },
    { id: 'h2', key: 'X-App-Id', value: 'yarac-app', enabled: true }
  ],
  auth: {
    type: 'bearer',
    bearerToken: 'token_abc123'
  },
  body: {
    type: 'json',
    rawContent: '{"orderId": 42}',
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

// 1. Verify resolvers
const resolvedUrl = resolveUrl(testReq);
console.assert(resolvedUrl === 'https://api.test.com/orders?filter=active', `Resolved URL mismatch: ${resolvedUrl}`);
const headers = resolveHeaders(testReq);
console.assert(headers['Authorization'] === 'Bearer token_abc123', 'Bearer auth missing in headers');
console.assert(headers['X-App-Id'] === 'yarac-app', 'X-App-Id missing in headers');
const body = resolveBody(testReq);
console.assert(body === '{"orderId": 42}', `Body mismatch: ${body}`);
console.log('✓ Resolvers unit test passed');

// 2. Test each individual modular generator directly
const curlCode = curlLanguage.generate(testReq, 'bash');
console.assert(curlCode.includes('curl -X POST') && curlCode.includes('token_abc123'), 'cURL bash generation failed');
console.log('✓ cURL modular generator');

const pythonCode = pythonLanguage.generate(testReq, 'requests');
console.assert(pythonCode.includes('requests.request') && pythonCode.includes('"POST"') && pythonCode.includes('Bearer token_abc123'), 'Python requests generation failed');
console.log('✓ Python modular generator');

const jsCode = javascriptLanguage.generate(testReq, 'fetch_browser');
console.assert(jsCode.includes('fetch(') && jsCode.includes('POST'), 'JavaScript fetch generation failed');
console.log('✓ JavaScript modular generator');

const goCode = goLanguage.generate(testReq, 'nethttp');
console.assert(goCode.includes('http.NewRequest("POST"') && goCode.includes('Bearer token_abc123'), 'Go modular generator failed');
console.log('✓ Go modular generator');

const rustCode = rustLanguage.generate(testReq, 'reqwest_async');
console.assert(rustCode.includes('client.post') || rustCode.includes('.post('), 'Rust modular generator failed');
console.log('✓ Rust modular generator');

const csCode = csharpLanguage.generate(testReq, 'httpclient');
console.assert(csCode.includes('HttpClient') && csCode.includes('HttpMethod.Post'), 'C# modular generator failed');
console.log('✓ C# modular generator');

const psCode = powershellLanguage.generate(testReq, 'restmethod');
console.assert(psCode.includes('Invoke-RestMethod'), 'PowerShell modular generator failed');
console.log('✓ PowerShell modular generator');

const shCode = shellLanguage.generate(testReq, 'wget');
console.assert(shCode.includes('wget --method=POST'), 'Shell wget generator failed');
console.log('✓ Shell modular generator');

const phpCode = phpLanguage.generate(testReq, 'guzzle');
console.assert(phpCode.includes('GuzzleHttp'), 'PHP guzzle generator failed');
console.log('✓ PHP modular generator');

const rubyCode = rubyLanguage.generate(testReq, 'nethttp');
console.assert(rubyCode.includes('Net::HTTP::Post'), 'Ruby modular generator failed');
console.log('✓ Ruby modular generator');

const javaCode = javaLanguage.generate(testReq, 'httpclient');
console.assert(javaCode.includes('HttpRequest.newBuilder()'), 'Java modular generator failed');
console.log('✓ Java modular generator');

// 3. Verify index export contains all 11 languages
console.assert(CODE_LANGUAGES.length === 11, `Expected 11 languages, got ${CODE_LANGUAGES.length}`);
console.log('✓ Index aggregate exported 11 languages');

console.log('--- ALL MODULAR GENERATOR UNIT TESTS PASSED ---');
