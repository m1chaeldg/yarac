<script lang="ts">
  import { workspaceStore, type KeyValuePair, type AuthType, type BodyType } from '../stores/workspace.svelte';
  import FlowPanel from './FlowPanel.svelte';

  let activeSubTab = $state<'params' | 'headers' | 'auth' | 'body' | 'flow'>('headers');

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

  let showBearerSecret = $state(false);
</script>

{#if req}
  <div class="h-full flex flex-col bg-slate-950 text-slate-200 text-xs overflow-hidden">
    <!-- Sub-Tab Navigation Bar -->
    <div class="h-9 border-b border-slate-800 bg-slate-900/60 px-3 flex items-center justify-between shrink-0">
      <div class="flex items-center gap-1">
        <button
          type="button"
          onclick={() => activeSubTab = 'params'}
          class="px-2.5 py-1 rounded font-medium transition-colors {activeSubTab === 'params' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Params</span>
          {#if req.params.filter(p => p.enabled && p.key).length > 0}
            <span class="ml-1 text-[10px] px-1 rounded-full bg-sky-950 text-sky-400 border border-sky-800/60">
              {req.params.filter(p => p.enabled && p.key).length}
            </span>
          {/if}
        </button>

        <button
          type="button"
          onclick={() => activeSubTab = 'headers'}
          class="px-2.5 py-1 rounded font-medium transition-colors {activeSubTab === 'headers' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Headers</span>
          {#if req.headers.filter(h => h.enabled && h.key).length > 0}
            <span class="ml-1 text-[10px] px-1 rounded-full bg-sky-950 text-sky-400 border border-sky-800/60">
              {req.headers.filter(h => h.enabled && h.key).length}
            </span>
          {/if}
        </button>

        <button
          type="button"
          onclick={() => activeSubTab = 'auth'}
          class="px-2.5 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'auth' ? 'bg-slate-800 text-emerald-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Auth</span>
          {#if req.auth.type !== 'none' && req.auth.type !== 'inherit'}
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
          {/if}
        </button>

        <button
          type="button"
          onclick={() => activeSubTab = 'body'}
          class="px-2.5 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'body' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Body</span>
          {#if req.body.type !== 'none'}
            <span class="text-[10px] px-1 rounded bg-slate-800 text-amber-400 font-mono uppercase">
              {req.body.type}
            </span>
          {/if}
        </button>

        <button
          type="button"
          onclick={() => activeSubTab = 'flow'}
          class="px-2.5 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'flow' ? 'bg-slate-800 text-purple-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
                  placeholder="e.g. {{api_bearer_token}}"
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
                Variables like <code class="text-sky-400 font-mono">{"{{api_bearer_token}}"}</code> are automatically resolved from your active environment.
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
                  placeholder="Password or {{api_password}}"
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
                    placeholder="{{api_key}}"
                    oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                    class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                  />
                </div>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-slate-400">Add to:</span>
                <label class="flex items-center gap-1.5 text-xs text-slate-300 cursor-pointer">
                  <input
                    type="radio"
                    bind:group={req.auth.apiKeyLocation}
                    value="header"
                    onchange={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  /> Header
                </label>
                <label class="flex items-center gap-1.5 text-xs text-slate-300 cursor-pointer ml-3">
                  <input
                    type="radio"
                    bind:group={req.auth.apiKeyLocation}
                    value="query"
                    onchange={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  /> Query Params
                </label>
              </div>
            </div>

          {:else if req.auth.type === 'inherit'}
            <div class="p-3 bg-slate-900/40 border border-slate-800 rounded-lg text-slate-400 text-xs">
              This request automatically inherits the authentication settings defined at the parent Collection level.
            </div>
          {/if}
        </div>

      <!-- BODY TAB -->
      {:else if activeSubTab === 'body'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <div class="flex items-center gap-3">
            <span class="text-xs text-slate-400 font-medium">Body Type:</span>
            <div class="flex items-center gap-2">
              {#each (['none', 'json', 'form-urlencoded', 'multipart', 'raw'] as const) as bType}
                <button
                  type="button"
                  onclick={() => {
                    if (req) {
                      req.body.type = bType;
                      workspaceStore.updateActiveRequest({ body: req.body });
                    }
                  }}
                  class="px-2.5 py-1 rounded text-xs font-medium uppercase font-mono transition-colors
                    {req.body.type === bType
                      ? 'bg-sky-600/20 text-sky-300 border border-sky-500/40'
                      : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'}"
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
                placeholder={req.body.type === 'json' ? '{\n  "key": "value"\n}' : 'Enter raw body text here...'}
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
            <div class="p-6 text-center text-slate-500 border border-dashed border-slate-800 rounded-lg">
              This request does not send a body payload. Select JSON, Form, or Raw above to configure a body.
            </div>
          {/if}
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
