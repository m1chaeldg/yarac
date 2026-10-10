<script lang="ts">
  import { workspaceStore, type KeyValuePair, type AuthType, type BodyType } from '../stores/workspace.svelte';
  import FlowPanel from './FlowPanel.svelte';

  let activeSubTab = $state<'params' | 'headers' | 'variables' | 'auth' | 'body' | 'scripts' | 'assert' | 'tests' | 'settings' | 'flow'>('headers');
  let activeScriptType = $state<'preRequest' | 'postResponse'>('preRequest');

  let req = $derived(workspaceStore.activeRequest);

  // Common Header suggestions
  const commonHeaders = [
    'Content-Type',
    'Accept',
    'Authorization',
    'User-Agent',
    'Cache-Control',
    'X-Request-Id',
    'X-Api-Key'
  ];

  // Bruno & Faker dynamic variables quick list
  const dynamicVars = [
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

  function addParam() {
    if (!req) return;
    req.params.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      description: ''
    });
    workspaceStore.updateActiveRequest({ params: req.params });
  }

  function removeParam(index: number) {
    if (!req) return;
    req.params.splice(index, 1);
    workspaceStore.updateActiveRequest({ params: req.params });
  }

  function addHeader(presetKey = '') {
    if (!req) return;
    req.headers.push({
      id: Math.random().toString(36).substring(2, 9),
      key: presetKey,
      value: presetKey === 'Content-Type' ? 'application/json' : '',
      enabled: true,
      description: ''
    });
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }

  function removeHeader(index: number) {
    if (!req) return;
    req.headers.splice(index, 1);
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }

  function addVariable() {
    if (!req) return;
    if (!req.variables) req.variables = [];
    req.variables.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      description: '',
      isSecret: false
    });
    workspaceStore.updateActiveRequest({ variables: req.variables });
  }

  function removeVariable(index: number) {
    if (!req || !req.variables) return;
    req.variables.splice(index, 1);
    workspaceStore.updateActiveRequest({ variables: req.variables });
  }

  function addFormData() {
    if (!req) return;
    req.body.formData.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true
    });
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function removeFormData(index: number) {
    if (!req) return;
    req.body.formData.splice(index, 1);
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function formatJsonBody() {
    if (!req || req.body.type !== 'json') return;
    try {
      const parsed = JSON.parse(req.body.rawContent);
      req.body.rawContent = JSON.stringify(parsed, null, 2);
      workspaceStore.updateActiveRequest({ body: req.body });
    } catch (e) {
      // Keep as is if invalid JSON
    }
  }

  function insertScriptSnippet(snippet: string) {
    if (!req) return;
    if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
    if (activeScriptType === 'preRequest') {
      const cur = req.scripts.preRequest ? req.scripts.preRequest + '\n' : '';
      req.scripts.preRequest = cur + snippet;
    } else {
      const cur = req.scripts.postResponse ? req.scripts.postResponse + '\n' : '';
      req.scripts.postResponse = cur + snippet;
    }
    workspaceStore.updateActiveRequest({ scripts: req.scripts });
  }

  function insertAssertSnippet(snippet: string) {
    if (!req) return;
    const cur = req.assert ? req.assert + '\n' : '';
    req.assert = cur + snippet;
    workspaceStore.updateActiveRequest({ assert: req.assert });
  }

  function insertTestSnippet(snippet: string) {
    if (!req) return;
    const cur = req.tests ? req.tests + '\n' : '';
    req.tests = cur + snippet;
    workspaceStore.updateActiveRequest({ tests: req.tests });
  }

  let showBearerSecret = $state(false);

  const bearerTokenPlaceholder = 'e.g. {{api_bearer_token}}';
  const basicPasswordPlaceholder = 'Password or {{api_password}}';
  const apiKeyPlaceholder = '{{api_key}}';
  const varValuePlaceholder = 'value or {{$uuid}}';
  const jsonBodyPlaceholder = '{\n  "key": "value"\n}';
  const preScriptPlaceholder = '// JavaScript pre-request script\n// Available globals: req, yarac.getVar, yarac.setVar, faker\nconst token = faker.string.uuid();\nyarac.setVar("session_token", token);';
  const postScriptPlaceholder = '// JavaScript post-response script\n// Available globals: res, req, yarac.getVar, yarac.setVar, faker\nif (res.status === 200) {\n  const data = res.json();\n  console.log("Received response:", data);\n}';
  const assertPlaceholder = '// Single-line assertions:\nexpect(res.status).toBe(200);\nexpect(res.time).toBeLessThan(1500);\nexpect(res.json().status).toBe("success");';
  const testsPlaceholder = 'describe("User Service API", () => {\n  it("should return 200 OK", () => {\n    expect(res.status).toBe(200);\n  });\n\n  it("should match schema and payload", () => {\n    const data = res.json();\n    expect(data.id).toBeDefined();\n    expect(data.email).toContain("@");\n  });\n});';
</script>

{#if req}
  <div class="h-full flex flex-col bg-slate-950 text-slate-200 text-xs overflow-hidden">
    <!-- Sub-Tab Navigation Bar -->
    <div class="h-9 border-b border-slate-800 bg-slate-900/60 px-3 flex items-center justify-between shrink-0 overflow-x-auto">
      <div class="flex items-center gap-1">
        <!-- PARAMS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'params'}
          class="px-2 py-1 rounded font-medium transition-colors {activeSubTab === 'params' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Params</span>
          {#if req.params.filter(p => p.enabled && p.key).length > 0}
            <span class="ml-1 text-[10px] px-1 rounded-full bg-sky-950 text-sky-400 border border-sky-800/60">
              {req.params.filter(p => p.enabled && p.key).length}
            </span>
          {/if}
        </button>

        <!-- HEADERS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'headers'}
          class="px-2 py-1 rounded font-medium transition-colors {activeSubTab === 'headers' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Headers</span>
          {#if req.headers.filter(h => h.enabled && h.key).length > 0}
            <span class="ml-1 text-[10px] px-1 rounded-full bg-sky-950 text-sky-400 border border-sky-800/60">
              {req.headers.filter(h => h.enabled && h.key).length}
            </span>
          {/if}
        </button>

        <!-- VARIABLES -->
        <button
          type="button"
          onclick={() => activeSubTab = 'variables'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'variables' ? 'bg-slate-800 text-amber-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Variables</span>
          {#if (req.variables || []).filter(v => v.enabled && v.key).length > 0}
            <span class="text-[10px] px-1 rounded-full bg-amber-950 text-amber-400 border border-amber-800/60">
              {(req.variables || []).filter(v => v.enabled && v.key).length}
            </span>
          {/if}
        </button>

        <!-- AUTH -->
        <button
          type="button"
          onclick={() => activeSubTab = 'auth'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'auth' ? 'bg-slate-800 text-emerald-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Auth</span>
          {#if req.auth.type !== 'none' && req.auth.type !== 'inherit'}
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
          {/if}
        </button>

        <!-- BODY -->
        <button
          type="button"
          onclick={() => activeSubTab = 'body'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'body' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Body</span>
          {#if req.body.type !== 'none'}
            <span class="text-[10px] px-1 rounded bg-slate-800 text-amber-400 font-mono uppercase">
              {req.body.type}
            </span>
          {/if}
        </button>

        <!-- SCRIPTS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'scripts'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'scripts' ? 'bg-slate-800 text-indigo-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Scripts</span>
          {#if (req.scripts?.preRequest?.trim() || req.scripts?.postResponse?.trim())}
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
          {/if}
        </button>

        <!-- ASSERT -->
        <button
          type="button"
          onclick={() => activeSubTab = 'assert'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'assert' ? 'bg-slate-800 text-teal-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Assert</span>
          {#if req.assert?.trim()}
            <span class="w-1.5 h-1.5 rounded-full bg-teal-400"></span>
          {/if}
        </button>

        <!-- TESTS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'tests'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'tests' ? 'bg-slate-800 text-pink-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Tests</span>
          {#if req.tests?.trim()}
            <span class="w-1.5 h-1.5 rounded-full bg-pink-400"></span>
          {/if}
        </button>

        <!-- SETTINGS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'settings'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'settings' ? 'bg-slate-800 text-cyan-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Settings</span>
        </button>

        <!-- FLOW -->
        <button
          type="button"
          onclick={() => activeSubTab = 'flow'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'flow' ? 'bg-slate-800 text-purple-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Flow</span>
          {#if req.flow.once_per_run || req.flow.retry_until}
            <span class="w-1.5 h-1.5 rounded-full bg-purple-400"></span>
          {/if}
        </button>
      </div>

      <!-- Quick Actions -->
      {#if activeSubTab === 'headers'}
        <div class="flex items-center gap-2">
          <select
            onchange={(e) => {
              const val = (e.target as HTMLSelectElement).value;
              if (val) {
                addHeader(val);
                (e.target as HTMLSelectElement).value = '';
              }
            }}
            class="bg-slate-800 border border-slate-700 text-[11px] text-slate-300 rounded px-2 py-0.5 focus:outline-none"
          >
            <option value="">+ Add Common Header</option>
            {#each commonHeaders as h}
              <option value={h}>{h}</option>
            {/each}
          </select>
        </div>
      {:else if activeSubTab === 'body' && req.body.type === 'json'}
        <button
          type="button"
          onclick={formatJsonBody}
          class="text-[11px] text-sky-400 hover:text-sky-300 px-2 py-0.5 rounded hover:bg-slate-800 transition-colors font-mono"
        >
          Pretty JSON
        </button>
      {/if}
    </div>

    <!-- Sub-Tab Content Viewport -->
    <div class="flex-1 overflow-y-auto p-3">
      <!-- PARAMS TAB -->
      {#if activeSubTab === 'params'}
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Query Parameters</span>
            <button
              type="button"
              onclick={addParam}
              class="text-xs text-sky-400 hover:text-sky-300 font-medium"
            >
              + Add Parameter
            </button>
          </div>

          <table class="w-full border-collapse border border-slate-800 text-left font-mono">
            <thead>
              <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
                <th class="w-8 p-2 text-center"></th>
                <th class="p-2 border-r border-slate-800">Parameter Key</th>
                <th class="p-2 border-r border-slate-800">Value</th>
                <th class="p-2 border-r border-slate-800 font-sans">Description</th>
                <th class="w-10 p-2 text-center"></th>
              </tr>
            </thead>
            <tbody>
              {#each req.params as param, idx (param.id)}
                <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                  <td class="p-2 text-center">
                    <input
                      type="checkbox"
                      bind:checked={param.enabled}
                      onchange={() => workspaceStore.updateActiveRequest({ params: req?.params })}
                      class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={param.key}
                      placeholder="e.g. limit"
                      oninput={() => workspaceStore.updateActiveRequest({ params: req?.params })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={param.value}
                      placeholder="e.g. 50"
                      oninput={() => workspaceStore.updateActiveRequest({ params: req?.params })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800 font-sans">
                    <input
                      type="text"
                      bind:value={param.description}
                      placeholder="Description"
                      oninput={() => workspaceStore.updateActiveRequest({ params: req?.params })}
                      class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
                    />
                  </td>
                  <td class="p-2 text-center">
                    <button
                      type="button"
                      onclick={() => removeParam(idx)}
                      class="text-slate-500 hover:text-rose-400"
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              {/each}
              {#if req.params.length === 0}
                <tr>
                  <td colspan="5" class="p-4 text-center text-slate-500 font-sans">
                    No query parameters configured. Click "+ Add Parameter" above.
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>

      <!-- HEADERS TAB -->
      {:else if activeSubTab === 'headers'}
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">HTTP Request Headers</span>
            <button
              type="button"
              onclick={() => addHeader()}
              class="text-xs text-sky-400 hover:text-sky-300 font-medium"
            >
              + Add Header
            </button>
          </div>

          <table class="w-full border-collapse border border-slate-800 text-left font-mono">
            <thead>
              <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
                <th class="w-8 p-2 text-center"></th>
                <th class="p-2 border-r border-slate-800">Header Key</th>
                <th class="p-2 border-r border-slate-800">Header Value</th>
                <th class="p-2 border-r border-slate-800 font-sans">Description</th>
                <th class="w-10 p-2 text-center"></th>
              </tr>
            </thead>
            <tbody>
              {#each req.headers as header, idx (header.id)}
                <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                  <td class="p-2 text-center">
                    <input
                      type="checkbox"
                      bind:checked={header.enabled}
                      onchange={() => workspaceStore.updateActiveRequest({ headers: req?.headers })}
                      class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={header.key}
                      placeholder="e.g. Content-Type"
                      oninput={() => workspaceStore.updateActiveRequest({ headers: req?.headers })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={header.value}
                      placeholder="e.g. application/json"
                      oninput={() => workspaceStore.updateActiveRequest({ headers: req?.headers })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800 font-sans">
                    <input
                      type="text"
                      bind:value={header.description}
                      placeholder="Description"
                      oninput={() => workspaceStore.updateActiveRequest({ headers: req?.headers })}
                      class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
                    />
                  </td>
                  <td class="p-2 text-center">
                    <button
                      type="button"
                      onclick={() => removeHeader(idx)}
                      class="text-slate-500 hover:text-rose-400"
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              {/each}
              {#if req.headers.length === 0}
                <tr>
                  <td colspan="5" class="p-4 text-center text-slate-500 font-sans">
                    No headers defined. Click "+ Add Header" to add custom request headers.
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>

      <!-- VARIABLES TAB -->
      {:else if activeSubTab === 'variables'}
        <div class="space-y-3 font-sans">
          <div class="flex items-center justify-between">
            <div>
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Request-Level Variables</span>
              <p class="text-[11px] text-slate-500 mt-0.5">
                Variables defined here take precedence for this request. Use <code class="text-amber-400 font-mono">{"{{var_name}}"}</code> in URL, headers, or body.
              </p>
            </div>
            <button
              type="button"
              onclick={addVariable}
              class="text-xs text-amber-400 hover:text-amber-300 font-medium"
            >
              + Add Variable
            </button>
          </div>

          <table class="w-full border-collapse border border-slate-800 text-left font-mono">
            <thead>
              <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
                <th class="w-8 p-2 text-center"></th>
                <th class="p-2 border-r border-slate-800">Variable Key</th>
                <th class="p-2 border-r border-slate-800">Value</th>
                <th class="w-16 p-2 text-center border-r border-slate-800">Secret</th>
                <th class="p-2 border-r border-slate-800 font-sans">Description</th>
                <th class="w-10 p-2 text-center"></th>
              </tr>
            </thead>
            <tbody>
              {#each (req.variables || []) as v, idx (v.id)}
                <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                  <td class="p-2 text-center">
                    <input
                      type="checkbox"
                      bind:checked={v.enabled}
                      onchange={() => workspaceStore.updateActiveRequest({ variables: req?.variables })}
                      class="rounded bg-slate-800 border-slate-700 text-amber-500 focus:ring-0"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={v.key}
                      placeholder="e.g. user_id"
                      oninput={() => workspaceStore.updateActiveRequest({ variables: req?.variables })}
                      class="w-full bg-transparent px-2 py-1 text-amber-300 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type={v.isSecret ? 'password' : 'text'}
                      bind:value={v.value}
                      placeholder={varValuePlaceholder}
                      oninput={() => workspaceStore.updateActiveRequest({ variables: req?.variables })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 text-center border-r border-slate-800">
                    <button
                      type="button"
                      onclick={() => {
                        v.isSecret = !v.isSecret;
                        workspaceStore.updateActiveRequest({ variables: req?.variables });
                      }}
                      class="text-xs px-1.5 py-0.5 rounded {v.isSecret ? 'text-amber-400 bg-amber-950/60 border border-amber-800/60' : 'text-slate-500 hover:text-slate-300'}"
                      title={v.isSecret ? 'Secret masked' : 'Plain text'}
                    >
                      {v.isSecret ? '🔒' : '👁'}
                    </button>
                  </td>
                  <td class="p-1 border-r border-slate-800 font-sans">
                    <input
                      type="text"
                      bind:value={v.description}
                      placeholder="Description"
                      oninput={() => workspaceStore.updateActiveRequest({ variables: req?.variables })}
                      class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
                    />
                  </td>
                  <td class="p-2 text-center">
                    <button
                      type="button"
                      onclick={() => removeVariable(idx)}
                      class="text-slate-500 hover:text-rose-400"
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              {/each}
              {#if (!req.variables || req.variables.length === 0)}
                <tr>
                  <td colspan="6" class="p-4 text-center text-slate-500 font-sans">
                    No request variables defined. Click "+ Add Variable" to scope variables to this request.
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>

          <!-- Dynamic Variables Cheat Sheet -->
          <div class="mt-4 p-3 bg-slate-900/40 border border-slate-800 rounded-lg">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block mb-2">
              Bruno Dynamic Variables (Use anywhere in URLs, Headers, Body, or Variables)
            </span>
            <div class="flex flex-wrap gap-1.5">
              {#each dynamicVars as dv}
                <button
                  type="button"
                  onclick={() => {
                    navigator.clipboard.writeText(`{{${dv.label}}}`);
                  }}
                  title="{dv.desc} (Click to copy)"
                  class="px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 text-sky-400 border border-slate-700/60 text-[10px] font-mono transition-colors flex items-center gap-1"
                >
                  <span>{`{{${dv.label}}}`}</span>
                  <span class="text-slate-500 text-[9px]">{dv.desc}</span>
                </button>
              {/each}
            </div>
          </div>
        </div>

      <!-- AUTH TAB -->
      {:else if activeSubTab === 'auth'}
        <div class="max-w-xl space-y-4 font-sans">
          <div class="flex items-center gap-3">
            <label for="auth-type-select" class="text-xs text-slate-400 font-medium">Auth Type:</label>
            <select
              id="auth-type-select"
              bind:value={req.auth.type}
              onchange={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
              class="bg-slate-800 border border-slate-700 text-slate-200 text-xs rounded px-2.5 py-1.5 focus:outline-none focus:border-sky-500 font-semibold"
            >
              <option value="inherit">Inherit auth from Collection</option>
              <option value="bearer">Bearer Token</option>
              <option value="basic">Basic Auth</option>
              <option value="apikey">API Key</option>
              <option value="none">No Auth</option>
            </select>
          </div>

          {#if req.auth.type === 'bearer'}
            <div class="p-3 bg-slate-900/60 border border-slate-800 rounded-lg space-y-2">
              <label for="bearer-token-input" class="text-xs font-semibold text-slate-300">Bearer Token</label>
              <div class="relative">
                <input
                  id="bearer-token-input"
                  type={showBearerSecret ? 'text' : 'password'}
                  bind:value={req.auth.bearerToken}
                  placeholder={bearerTokenPlaceholder}
                  oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-emerald-400 focus:outline-none focus:border-sky-500 pr-16"
                />
                <button
                  type="button"
                  onclick={() => showBearerSecret = !showBearerSecret}
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-slate-400 hover:text-slate-200 px-1 py-0.5 rounded bg-slate-800"
                >
                  {showBearerSecret ? 'Hide' : 'Show'}
                </button>
              </div>
              <p class="text-[11px] text-slate-500">
                Variables like <code class="text-sky-400 font-mono">{"{{api_bearer_token}}"}</code> are automatically resolved.
              </p>
            </div>

          {:else if req.auth.type === 'basic'}
            <div class="p-3 bg-slate-900/60 border border-slate-800 rounded-lg space-y-3">
              <div class="space-y-1">
                <label for="basic-username-input" class="text-xs font-semibold text-slate-300">Username</label>
                <input
                  id="basic-username-input"
                  type="text"
                  bind:value={req.auth.basicUsername}
                  placeholder="e.g. admin"
                  oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                />
              </div>
              <div class="space-y-1">
                <label for="basic-password-input" class="text-xs font-semibold text-slate-300">Password</label>
                <input
                  id="basic-password-input"
                  type="password"
                  bind:value={req.auth.basicPassword}
                  placeholder={basicPasswordPlaceholder}
                  oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                />
              </div>
            </div>

          {:else if req.auth.type === 'apikey'}
            <div class="p-3 bg-slate-900/60 border border-slate-800 rounded-lg space-y-3">
              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-1">
                  <label for="apikey-name-input" class="text-xs font-semibold text-slate-300">Key Name</label>
                  <input
                    id="apikey-name-input"
                    type="text"
                    bind:value={req.auth.apiKeyName}
                    placeholder="X-API-Key"
                    oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                    class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                  />
                </div>
                <div class="space-y-1">
                  <label for="apikey-value-input" class="text-xs font-semibold text-slate-300">Value</label>
                  <input
                    id="apikey-value-input"
                    type="text"
                    bind:value={req.auth.apiKeyValue}
                    placeholder={apiKeyPlaceholder}
                    oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                    class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                  />
                </div>
              </div>
            </div>

          {:else if req.auth.type === 'inherit'}
            <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-lg text-slate-400 text-xs">
              This request inherits authentication credentials configured at the collection or workspace level.
            </div>
          {/if}
        </div>

      <!-- BODY TAB -->
      {:else if activeSubTab === 'body'}
        <div class="h-full flex flex-col space-y-3">
          <div class="flex items-center gap-3">
            <span class="text-xs text-slate-400 font-medium">Body Type:</span>
            <div class="flex items-center gap-2">
              {#each ['none', 'json', 'form-urlencoded', 'multipart', 'raw'] as bType}
                <button
                  type="button"
                  onclick={() => {
                    if (req) {
                      req.body.type = bType as BodyType;
                      workspaceStore.updateActiveRequest({ body: req.body });
                    }
                  }}
                  class="px-2.5 py-1 rounded text-xs font-medium transition-colors {req.body.type === bType ? 'bg-sky-600/30 text-sky-300 border border-sky-500/50' : 'bg-slate-900 border border-slate-800 text-slate-400 hover:text-slate-200'}"
                >
                  {bType}
                </button>
              {/each}
            </div>
          </div>

          {#if req.body.type === 'json' || req.body.type === 'raw'}
            <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
              <div class="bg-slate-900/60 px-3 py-1 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
                <span>{req.body.type === 'json' ? 'JSON Payload' : 'Raw Text Payload'}</span>
                <span class="font-mono text-slate-500">{req.body.rawContent.length} chars</span>
              </div>
              <textarea
                bind:value={req.body.rawContent}
                oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                placeholder={req.body.type === 'json' ? jsonBodyPlaceholder : 'Enter raw body text here...'}
                class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-slate-100 resize-none focus:outline-none"
              ></textarea>
            </div>

          {:else if req.body.type === 'form-urlencoded' || req.body.type === 'multipart'}
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Form Fields</span>
                <button
                  type="button"
                  onclick={addFormData}
                  class="text-xs text-sky-400 hover:text-sky-300 font-medium"
                >
                  + Add Field
                </button>
              </div>

              <table class="w-full border-collapse border border-slate-800 text-left font-mono">
                <thead>
                  <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
                    <th class="w-8 p-2 text-center"></th>
                    <th class="p-2 border-r border-slate-800">Field Key</th>
                    <th class="p-2 border-r border-slate-800">Field Value</th>
                    <th class="w-10 p-2 text-center"></th>
                  </tr>
                </thead>
                <tbody>
                  {#each req.body.formData as item, idx (item.id)}
                    <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                      <td class="p-2 text-center">
                        <input
                          type="checkbox"
                          bind:checked={item.enabled}
                          onchange={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                          class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                        />
                      </td>
                      <td class="p-1 border-r border-slate-800">
                        <input
                          type="text"
                          bind:value={item.key}
                          placeholder="key"
                          oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                          class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                        />
                      </td>
                      <td class="p-1 border-r border-slate-800">
                        <input
                          type="text"
                          bind:value={item.value}
                          placeholder="value"
                          oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                          class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                        />
                      </td>
                      <td class="p-2 text-center">
                        <button
                          type="button"
                          onclick={() => removeFormData(idx)}
                          class="text-slate-500 hover:text-rose-400"
                        >
                          ✕
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>

          {:else}
            <div class="p-6 text-center text-slate-500 border border-dashed border-slate-800 rounded-lg font-sans">
              This request does not send a body payload. Select JSON, Form, or Raw above to configure a body.
            </div>
          {/if}
        </div>

      <!-- SCRIPTS TAB (PRE-REQUEST & POST-RESPONSE) -->
      {:else if activeSubTab === 'scripts'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <div class="flex items-center justify-between shrink-0">
            <div class="flex items-center gap-1.5 bg-slate-900 border border-slate-800 p-0.5 rounded-lg">
              <button
                type="button"
                onclick={() => activeScriptType = 'preRequest'}
                class="px-3 py-1 rounded text-xs font-semibold transition-colors {activeScriptType === 'preRequest' ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200'}"
              >
                Pre-request Script
              </button>
              <button
                type="button"
                onclick={() => activeScriptType = 'postResponse'}
                class="px-3 py-1 rounded text-xs font-semibold transition-colors {activeScriptType === 'postResponse' ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200'}"
              >
                Post-response Script
              </button>
            </div>

            <!-- Snippets Quick Bar -->
            <div class="flex items-center gap-1.5 flex-wrap">
              <span class="text-[10px] text-slate-500">Insert Snippet:</span>
              {#if activeScriptType === 'preRequest'}
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("const email = faker.internet.email();\nyarac.setVar('user_email', email);")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Faker Email
                </button>
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("const name = faker.person.fullName();\nyarac.setVar('user_name', name);")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Faker Name
                </button>
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("req.headers['X-Timestamp'] = Date.now().toString();")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Header Timestamp
                </button>
              {:else}
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("const data = res.json();\nif (data?.token) {\n  yarac.setVar('auth_token', data.token);\n}")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Extract Token
                </button>
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("console.log('Response Status:', res.status);")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Log Status
                </button>
              {/if}
            </div>
          </div>

          <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
            <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
              <span class="font-mono text-indigo-400">
                {activeScriptType === 'preRequest' ? '// Runs before HTTP request is dispatched' : '// Runs immediately after HTTP response is received'}
              </span>
              <span class="text-slate-500 font-mono text-[10px]">Isolated Deno Core Runtime (5000ms max)</span>
            </div>

            {#if activeScriptType === 'preRequest'}
              <textarea
                value={req.scripts?.preRequest || ''}
                oninput={(e) => {
                  if (!req) return;
                  if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
                  req.scripts.preRequest = (e.target as HTMLTextAreaElement).value;
                  workspaceStore.updateActiveRequest({ scripts: req.scripts });
                }}
                placeholder={preScriptPlaceholder}
                class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-indigo-100 resize-none focus:outline-none"
              ></textarea>
            {:else}
              <textarea
                value={req.scripts?.postResponse || ''}
                oninput={(e) => {
                  if (!req) return;
                  if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
                  req.scripts.postResponse = (e.target as HTMLTextAreaElement).value;
                  workspaceStore.updateActiveRequest({ scripts: req.scripts });
                }}
                placeholder={postScriptPlaceholder}
                class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-indigo-100 resize-none focus:outline-none"
              ></textarea>
            {/if}
          </div>
        </div>

      <!-- ASSERT TAB -->
      {:else if activeSubTab === 'assert'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <div class="flex items-center justify-between shrink-0">
            <div>
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">JavaScript Assertions</span>
              <p class="text-[11px] text-slate-500">
                Lightweight assertion statements evaluated directly on the response. Test results display in the Response tab.
              </p>
            </div>

            <!-- Snippets -->
            <div class="flex items-center gap-1.5 flex-wrap">
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.status).toBe(200);")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Status 200
              </button>
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.time).toBeLessThan(1000);")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Latency &lt; 1000ms
              </button>
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.headers['content-type']).toContain('application/json');")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Content-Type JSON
              </button>
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.json().id).toBeDefined();")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Body ID Defined
              </button>
            </div>
          </div>

          <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
            <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
              <span class="font-mono text-teal-400">// Available matchers: toBe, toEqual, toBeDefined, toBeNull, toBeGreaterThan, toContain</span>
              <span class="text-slate-500 font-mono text-[10px]">Response results shown under "Test Results"</span>
            </div>
            <textarea
              value={req.assert || ''}
              oninput={(e) => {
                if (!req) return;
                req.assert = (e.target as HTMLTextAreaElement).value;
                workspaceStore.updateActiveRequest({ assert: req.assert });
              }}
              placeholder={assertPlaceholder}
              class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-teal-100 resize-none focus:outline-none"
            ></textarea>
          </div>
        </div>

      <!-- TESTS TAB (JASMINE / BRUNO BDD) -->
      {:else if activeSubTab === 'tests'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <div class="flex items-center justify-between shrink-0">
            <div>
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Jasmine / Bruno BDD Test Suite</span>
              <p class="text-[11px] text-slate-500">
                Structure tests with <code class="text-pink-400 font-mono">describe()</code> and <code class="text-pink-400 font-mono">it()</code> blocks. Full Faker library available globally.
              </p>
            </div>

            <div class="flex items-center gap-1.5">
              <button
                type="button"
                onclick={() => insertTestSnippet(`describe('Response Contract', () => {
  it('should return valid status and body', () => {
    expect(res.status).toBe(200);
    const body = res.json();
    expect(body).toBeDefined();
  });
});`)}
                class="px-2.5 py-1 rounded bg-pink-950/80 hover:bg-pink-900 text-pink-300 border border-pink-800 text-[11px] font-mono"
              >
                + BDD Template
              </button>
            </div>
          </div>

          <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
            <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
              <span class="font-mono text-pink-400">// globals: describe, it, test, expect, res, req, faker</span>
              <span class="text-slate-500 font-mono text-[10px]">Jasmine / Chai assertion suite</span>
            </div>
            <textarea
              value={req.tests || ''}
              oninput={(e) => {
                if (!req) return;
                req.tests = (e.target as HTMLTextAreaElement).value;
                workspaceStore.updateActiveRequest({ tests: req.tests });
              }}
              placeholder={testsPlaceholder}
              class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-pink-100 resize-none focus:outline-none"
            ></textarea>
          </div>
        </div>

      <!-- SETTINGS TAB -->
      {:else if activeSubTab === 'settings'}
        <div class="max-w-2xl space-y-4 font-sans">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block">
            Request Execution Settings
          </span>

          <div class="p-4 bg-slate-900/60 border border-slate-800 rounded-lg space-y-4">
            <!-- Follow Redirects -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-follow-redirect" class="text-xs font-semibold text-slate-200 block cursor-pointer">Follow Redirects</label>
                <span class="text-[11px] text-slate-500">Automatically follow HTTP 3xx redirection responses</span>
              </div>
              <input
                id="set-follow-redirect"
                type="checkbox"
                checked={req.settings?.followRedirects ?? true}
                onchange={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.followRedirects = (e.target as HTMLInputElement).checked;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
              />
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Max Redirects -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-max-redirects" class="text-xs font-semibold text-slate-200 block">Max Redirects</label>
                <span class="text-[11px] text-slate-500">Maximum number of consecutive redirects before aborting</span>
              </div>
              <input
                id="set-max-redirects"
                type="number"
                min="0"
                max="50"
                value={req.settings?.maxRedirects ?? 5}
                oninput={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.maxRedirects = parseInt((e.target as HTMLInputElement).value || '5', 10);
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-24 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500 text-right"
              />
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Timeout (ms) -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-timeout" class="text-xs font-semibold text-slate-200 block">Timeout (ms)</label>
                <span class="text-[11px] text-slate-500">Maximum execution time before request cancels. <strong>0</strong> = indefinite (no timeout)</span>
              </div>
              <div class="flex items-center gap-2">
                <input
                  id="set-timeout"
                  type="number"
                  min="0"
                  step="500"
                  value={req.settings?.timeoutMs ?? 0}
                  oninput={(e) => {
                    if (!req) return;
                    if (!req.settings) {
                      req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                    }
                    req.settings.timeoutMs = parseInt((e.target as HTMLInputElement).value || '0', 10);
                    workspaceStore.updateActiveRequest({ settings: req.settings });
                  }}
                  class="w-28 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500 text-right"
                />
                <span class="text-[11px] text-slate-400 font-mono">ms</span>
              </div>
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Verify SSL -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-verify-ssl" class="text-xs font-semibold text-slate-200 block cursor-pointer">Verify SSL Certificates</label>
                <span class="text-[11px] text-slate-500">Enforce valid TLS / HTTPS certificates for remote server</span>
              </div>
              <input
                id="set-verify-ssl"
                type="checkbox"
                checked={req.settings?.verifySsl ?? true}
                onchange={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.verifySsl = (e.target as HTMLInputElement).checked;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
              />
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Inherit Auth -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-inherit-auth" class="text-xs font-semibold text-slate-200 block cursor-pointer">Inherit Collection Auth</label>
                <span class="text-[11px] text-slate-500">Automatically attach parent collection authentication credentials</span>
              </div>
              <input
                id="set-inherit-auth"
                type="checkbox"
                checked={req.settings?.inheritAuth ?? (req.auth.type === 'inherit')}
                onchange={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.inheritAuth = (e.target as HTMLInputElement).checked;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
              />
            </div>
          </div>
        </div>

      <!-- FLOW TAB -->
      {:else if activeSubTab === 'flow'}
        <div class="h-full border border-slate-800 rounded-lg overflow-hidden bg-slate-900/30">
          <FlowPanel
            flow={req.flow}
            onchange={(updated) => {
              if (req) {
                req.flow = updated;
                workspaceStore.updateActiveRequest({ flow: updated });
              }
            }}
          />
        </div>
      {/if}
    </div>
  </div>
{/if}
