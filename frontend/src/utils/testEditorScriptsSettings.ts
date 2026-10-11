import {
  getDefaultRequestSettings,
  updateRequestSetting,
  appendSnippet,
  SCRIPT_SNIPPETS,
  ASSERT_SNIPPETS,
  BDD_TEMPLATE,
  SCRIPT_PLACEHOLDERS,
  type RequestSettings
} from './editorUtils.ts';

console.log('--- RUNNING EDITOR SCRIPTS & SETTINGS UNIT TEST SUITE ---');

// 1. Default request settings test
const defaultSettings = getDefaultRequestSettings();
console.assert(defaultSettings.inheritAuth === true, 'Default inheritAuth should be true');
console.assert(defaultSettings.verifySsl === true, 'Default verifySsl should be true');
console.assert(defaultSettings.followRedirects === true, 'Default followRedirects should be true');
console.assert(defaultSettings.maxRedirects === 5, 'Default maxRedirects should be 5');
console.assert(defaultSettings.timeoutMs === 0, 'Default timeoutMs should be 0');
console.log('✓ getDefaultRequestSettings returns correct defaults');

// 2. Setting mutation test
const customSettings = updateRequestSetting(defaultSettings, 'timeoutMs', 2500);
console.assert(customSettings.timeoutMs === 2500, 'timeoutMs not updated properly');
console.assert(customSettings.verifySsl === true, 'verifySsl was mutated unexpectedly');

const sslDisabled = updateRequestSetting(customSettings, 'verifySsl', false);
console.assert(sslDisabled.verifySsl === false, 'verifySsl not toggled properly');
console.assert(sslDisabled.timeoutMs === 2500, 'timeoutMs preserved');
console.log('✓ updateRequestSetting updates keys immutably');

// 3. appendSnippet test
const emptyResult = appendSnippet('', 'expect(res.status).toBe(200);');
console.assert(emptyResult === 'expect(res.status).toBe(200);', 'Appending to empty string failed');

const populatedResult = appendSnippet('const x = 1;', 'const y = 2;');
console.assert(populatedResult === 'const x = 1;\nconst y = 2;', 'Appending to existing code failed');
console.log('✓ appendSnippet handles empty and non-empty code correctly');

// 4. Presets and catalogue checks
console.assert(SCRIPT_SNIPPETS.preRequest.length >= 3, 'Pre-request snippets list incomplete');
console.assert(SCRIPT_SNIPPETS.postResponse.length >= 2, 'Post-response snippets list incomplete');
console.assert(ASSERT_SNIPPETS.length >= 4, 'Assertion snippets list incomplete');
console.assert(typeof BDD_TEMPLATE === 'string' && BDD_TEMPLATE.includes('describe('), 'BDD template missing describe()');
console.assert(SCRIPT_PLACEHOLDERS.preRequest.includes('faker'), 'Pre-request placeholder missing faker');
console.assert(SCRIPT_PLACEHOLDERS.postResponse.includes('res.status'), 'Post-response placeholder missing res.status');
console.log('✓ SCRIPT_SNIPPETS, ASSERT_SNIPPETS, BDD_TEMPLATE, and SCRIPT_PLACEHOLDERS populated');

console.log('--- ALL SCRIPTS & SETTINGS UNIT TESTS PASSED ---');
