import type { KeyValuePair, AuthType, BodyType, YaracRequest } from '../stores/workspace.svelte';

export const COMMON_HEADER_PRESETS = [
  'Content-Type',
  'Accept',
  'Authorization',
  'User-Agent',
  'Cache-Control',
  'X-Request-Id',
  'X-Api-Key'
];

export const DYNAMIC_VARIABLES_LIST = [
  { label: '$uuid', desc: 'UUID v4' },
  { label: '$timestamp', desc: 'Current UNIX timestamp' },
  { label: '$isoTimestamp', desc: 'ISO 8601 timestamp' },
  { label: '$randomEmail', desc: 'Random email address' },
  { label: '$randomFullName', desc: 'Random full name' },
  { label: '$randomInt', desc: 'Random integer (1-1000)' },
  { label: '$randomPassword', desc: 'Random secure password' },
  { label: '$randomCity', desc: 'Random city' },
  { label: '$randomCountry', desc: 'Random country' },
  { label: '$randomPrice', desc: 'Random price (0-1000)' },
  { label: '$randomCompany', desc: 'Random company' }
];

export function createEmptyParam(): KeyValuePair {
  return {
    id: Math.random().toString(36).substring(2, 9),
    key: '',
    value: '',
    enabled: true,
    description: ''
  };
}

export function createEmptyHeader(presetKey = ''): KeyValuePair {
  return {
    id: Math.random().toString(36).substring(2, 9),
    key: presetKey,
    value: presetKey === 'Content-Type' ? 'application/json' : '',
    enabled: true,
    description: ''
  };
}

export function createEmptyVariable(): KeyValuePair & { isSecret?: boolean } {
  return {
    id: Math.random().toString(36).substring(2, 9),
    key: '',
    value: '',
    enabled: true,
    description: '',
    isSecret: false
  };
}

export function setRequestBodyType(req: YaracRequest, bType: BodyType): void {
  if (!req || !req.body) return;
  req.body.type = bType;
  if (!req.body.formData) req.body.formData = [];
  if (!req.body.rawContent) req.body.rawContent = '';
  if (bType === 'binary' && !req.body.binaryContentType) {
    req.body.binaryContentType = 'application/octet-stream';
  }
}

export function setAuthType(req: YaracRequest, authType: AuthType): void {
  if (!req) return;
  if (!req.auth) req.auth = { type: 'none' };
  req.auth.type = authType;
}

export function insertScriptSnippet(existing: string | undefined, snippet: string): string {
  const current = (existing || '').trim();
  if (!current) return snippet;
  return `${current}\n\n${snippet}`;
}

export interface RequestSettings {
  inheritAuth: boolean;
  verifySsl: boolean;
  followRedirects: boolean;
  maxRedirects: number;
  timeoutMs: number;
}

export function getDefaultRequestSettings(): RequestSettings {
  return {
    inheritAuth: true,
    verifySsl: true,
    followRedirects: true,
    maxRedirects: 5,
    timeoutMs: 0
  };
}

export function updateRequestSetting<K extends keyof RequestSettings>(
  settings: RequestSettings,
  key: K,
  value: RequestSettings[K]
): RequestSettings {
  return {
    ...settings,
    [key]: value
  };
}

export function appendSnippet(existing: string | undefined, snippet: string): string {
  if (!existing || existing.trim() === '') return snippet;
  return `${existing.trimEnd()}\n${snippet}`;
}

export const SCRIPT_SNIPPETS = {
  preRequest: [
    { label: '+ Faker Email', snippet: "const email = faker.internet.email();\nyarac.setVar('user_email', email);" },
    { label: '+ Faker Name', snippet: "const name = faker.person.fullName();\nyarac.setVar('user_name', name);" },
    { label: '+ Header Timestamp', snippet: "req.headers['X-Timestamp'] = Date.now().toString();" }
  ],
  postResponse: [
    { label: '+ Extract Token', snippet: "const data = res.json();\nif (data?.token) {\n  yarac.setVar('auth_token', data.token);\n}" },
    { label: '+ Log Status', snippet: "console.log('Response Status:', res.status);" }
  ]
};

export const ASSERT_SNIPPETS = [
  { label: '+ Status 200', snippet: 'expect(res.status).toBe(200);' },
  { label: '+ Latency < 1000ms', snippet: 'expect(res.time).toBeLessThan(1000);' },
  { label: '+ Content-Type JSON', snippet: "expect(res.headers['content-type']).toContain('application/json');" },
  { label: '+ Body ID Defined', snippet: 'expect(res.json().id).toBeDefined();' }
];

export const BDD_TEMPLATE = `describe('Response Contract', () => {
  it('should return valid status and body', () => {
    expect(res.status).toBe(200);
    const body = res.json();
    expect(body).toBeDefined();
  });
});`;

export const SCRIPT_PLACEHOLDERS = {
  preRequest: '// JavaScript pre-request script\n// Available globals: req, yarac.getVar, yarac.setVar, faker\nconst token = faker.string.uuid();\nyarac.setVar("session_token", token);',
  postResponse: '// JavaScript post-response script\n// Available globals: res, req, yarac.getVar, yarac.setVar, faker\nif (res.status === 200) {\n  const data = res.json();\n  console.log("Received response:", data);\n}',
  assert: '// Single-line assertions:\nexpect(res.status).toBe(200);\nexpect(res.time).toBeLessThan(1500);\nexpect(res.json().status).toBe("success");',
  tests: 'describe("User Service API", () => {\n  it("should return 200 OK", () => {\n    expect(res.status).toBe(200);\n  });\n\n  it("should match schema and payload", () => {\n    const data = res.json();\n    expect(data.id).toBeDefined();\n    expect(data.email).toContain("@");\n  });\n});'
};

