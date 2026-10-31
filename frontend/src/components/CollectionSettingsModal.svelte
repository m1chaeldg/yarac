<script lang="ts">
  import { workspaceStore, type Collection, type AuthConfig, type KeyValuePair } from '../stores/workspace.svelte';

  interface Props {
    isOpen: boolean;
    collection: Collection | null;
    onClose: () => void;
  }

  let { isOpen, collection, onClose }: Props = $props();

  let activeTab = $state<'overview' | 'auth' | 'headers' | 'variables' | 'scripts'>('overview');

  let colName = $state('');
  let colDesc = $state('');
  let colAuth = $state<AuthConfig>({ type: 'none' });
  let colHeaders = $state<KeyValuePair[]>([]);
  let colVariables = $state<KeyValuePair[]>([]);
  let preScript = $state('');
  let postScript = $state('');

  let showBearerSecret = $state(false);
  let showSigv4Secret = $state(false);

  $effect(() => {
    if (isOpen && collection) {
      colName = collection.name || '';
      colDesc = collection.description || '';
      colAuth = collection.auth ? JSON.parse(JSON.stringify(collection.auth)) : { type: 'none' };
      colHeaders = collection.headers ? JSON.parse(JSON.stringify(collection.headers)) : [];
      colVariables = collection.variables ? JSON.parse(JSON.stringify(collection.variables)) : [];
      preScript = collection.scripts?.preRequest || '';
      postScript = collection.scripts?.postResponse || '';
    }
  });

  function addHeader() {
    colHeaders.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true
    });
  }

  function removeHeader(idx: number) {
    colHeaders.splice(idx, 1);
  }

  function addVariable() {
    colVariables.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      isSecret: false
    });
  }

  function removeVariable(idx: number) {
    colVariables.splice(idx, 1);
  }

  function handleSave() {
    if (!collection) return;
    workspaceStore.updateCollection(collection.id, {
      name: colName,
      description: colDesc,
      auth: colAuth,
      headers: colHeaders.filter(h => h.key.trim()),
      variables: colVariables.filter(v => v.key.trim()),
      scripts: {
        preRequest: preScript,
        postResponse: postScript
      }
    });
    onClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen && collection}
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 font-sans animate-fade-in">
    <div class="apple-modal-card w-full max-w-3xl max-h-[85vh] flex flex-col text-apple-body-on-dark overflow-hidden">
      <!-- Modal Header -->
      <div class="h-12 border-b border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2.5">
          <span class="text-apple-link-on-dark text-base">📁</span>
          <h2 class="apple-body-strong text-apple-body-on-dark tracking-tight">Collection Settings</h2>
          <span class="apple-micro-legal uppercase font-mono px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark">
            {collection.name}
          </span>
        </div>
        <button
          type="button"
          aria-label="Close collection settings modal"
          onclick={onClose}
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
        >
          ✕
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="px-6 border-b border-apple-hairline-dark bg-apple-surface-tile-3 flex gap-4 text-xs shrink-0">
        {#each [
          { id: 'overview', label: 'Overview' },
          { id: 'auth', label: 'Inherited Auth' },
          { id: 'headers', label: 'Default Headers' },
          { id: 'variables', label: 'Variables' },
          { id: 'scripts', label: 'Scripts & Tests' }
        ] as tab}
          <button
            type="button"
            onclick={() => activeTab = tab.id as any}
            class="py-3 font-semibold border-b-2 transition-colors {activeTab === tab.id ? 'border-apple-primary text-apple-primary-on-dark' : 'border-transparent text-apple-body-muted hover:text-white'}"
          >
            {tab.label}
          </button>
        {/each}
      </div>

      <!-- Modal Body -->
      <div class="flex-1 p-6 overflow-y-auto space-y-4 bg-apple-surface-tile-1 text-xs">
        {#if activeTab === 'overview'}
          <div class="space-y-4 max-w-lg">
            <div class="space-y-1">
              <label for="col-name-input" class="text-xs font-semibold text-apple-body-on-dark">Collection Name</label>
              <input
                id="col-name-input"
                type="text"
                bind:value={colName}
                class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs text-apple-body-on-dark focus:border-apple-primary focus:outline-none"
              />
            </div>

            <div class="space-y-1">
              <label for="col-desc-input" class="text-xs font-semibold text-apple-body-on-dark">Description</label>
              <textarea
                id="col-desc-input"
                rows="3"
                bind:value={colDesc}
                placeholder="Optional description of API collection..."
                class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm p-3 text-xs text-apple-body-on-dark focus:border-apple-primary focus:outline-none"
              ></textarea>
            </div>

            <div class="p-3 bg-apple-surface-tile-2 rounded-apple-sm border border-apple-hairline-dark/60 font-mono text-[11px] text-apple-body-muted">
              <div>Disk Location: <span class="text-apple-primary-on-dark">{collection.path || collection.name}</span></div>
            </div>
          </div>

        {:else if activeTab === 'auth'}
          <div class="space-y-4 max-w-lg">
            <p class="text-apple-body-muted">
              Configure credentials inherited by requests inside this collection with <code class="text-apple-link-on-dark">inherit_auth = true</code>.
            </p>

            <div class="flex items-center gap-3">
              <label for="col-auth-type" class="text-xs font-medium text-apple-body-muted">Auth Type:</label>
              <select
                id="col-auth-type"
                bind:value={colAuth.type}
                class="bg-apple-surface-tile-3 border border-apple-hairline-dark text-apple-body-on-dark text-xs rounded-apple-sm px-2.5 py-1.5 focus:border-apple-primary font-medium"
              >
                <option value="none">No Auth</option>
                <option value="bearer">Bearer Token</option>
                <option value="basic">Basic Auth</option>
                <option value="apikey">API Key</option>
                <option value="sigv4">AWS SigV4 Signer</option>
              </select>
            </div>

            {#if colAuth.type === 'bearer'}
              <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-2">
                <label for="col-bearer-input" class="text-xs font-semibold text-apple-body-on-dark">Bearer Token</label>
                <div class="relative">
                  <input
                    id="col-bearer-input"
                    type={showBearerSecret ? 'text' : 'password'}
                    bind:value={colAuth.bearerToken}
                    placeholder="Bearer token or {{token}}"
                    class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark pr-16 focus:outline-none focus:border-apple-primary"
                  />
                  <button
                    type="button"
                    onclick={() => showBearerSecret = !showBearerSecret}
                    class="apple-button-dark-utility absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-apple-body-muted hover:text-white px-2 py-0.5 rounded-apple-sm border border-apple-hairline-dark/60"
                  >
                    {showBearerSecret ? 'Hide' : 'Show'}
                  </button>
                </div>
              </div>

            {:else if colAuth.type === 'basic'}
              <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-3">
                <div class="space-y-1">
                  <label for="col-basic-user" class="text-xs font-semibold text-apple-body-on-dark">Username</label>
                  <input
                    id="col-basic-user"
                    type="text"
                    bind:value={colAuth.basicUsername}
                    placeholder="Username or {{user}}"
                    class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                  />
                </div>
                <div class="space-y-1">
                  <label for="col-basic-pass" class="text-xs font-semibold text-apple-body-on-dark">Password</label>
                  <input
                    id="col-basic-pass"
                    type="password"
                    bind:value={colAuth.basicPassword}
                    placeholder="Password or {{password}}"
                    class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                  />
                </div>
              </div>

            {:else if colAuth.type === 'apikey'}
              <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-3">
                <div class="grid grid-cols-2 gap-3">
                  <div class="space-y-1">
                    <label for="col-api-name" class="text-xs font-semibold text-apple-body-on-dark">Key Name</label>
                    <input
                      id="col-api-name"
                      type="text"
                      bind:value={colAuth.apiKeyName}
                      placeholder="X-API-Key"
                      class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                  </div>
                  <div class="space-y-1">
                    <label for="col-api-val" class="text-xs font-semibold text-apple-body-on-dark">Key Value</label>
                    <input
                      id="col-api-val"
                      type="text"
                      bind:value={colAuth.apiKeyValue}
                      placeholder="Value or {{api_key}}"
                      class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                  </div>
                </div>
                <div class="space-y-1">
                  <label for="col-api-loc" class="text-xs font-semibold text-apple-body-on-dark">Placement</label>
                  <select
                    id="col-api-loc"
                    bind:value={colAuth.apiKeyLocation}
                    class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary font-medium"
                  >
                    <option value="header">Header (e.g. X-API-Key: value)</option>
                    <option value="query">Query Parameter (e.g. ?api_key=value)</option>
                  </select>
                </div>
              </div>

            {:else if colAuth.type === 'sigv4'}
              <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-3">
                <div class="grid grid-cols-2 gap-3">
                  <div class="space-y-1">
                    <label for="col-sigv4-key" class="text-xs font-semibold text-apple-body-on-dark">Access Key ID</label>
                    <input
                      id="col-sigv4-key"
                      type="text"
                      bind:value={colAuth.awsAccessKeyId}
                      placeholder="AKIAIOSFODNN7EXAMPLE"
                      class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                  </div>
                  <div class="space-y-1">
                    <div class="flex items-center justify-between">
                      <label for="col-sigv4-sec" class="text-xs font-semibold text-apple-body-on-dark">Secret Key</label>
                      <button
                        type="button"
                        onclick={() => showSigv4Secret = !showSigv4Secret}
                        class="text-[10px] text-apple-link-on-dark hover:underline"
                      >
                        {showSigv4Secret ? 'Hide' : 'Show'}
                      </button>
                    </div>
                    <input
                      id="col-sigv4-sec"
                      type={showSigv4Secret ? 'text' : 'password'}
                      bind:value={colAuth.awsSecretAccessKey}
                      placeholder="Secret Key or {{aws_secret}}"
                      class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                  </div>
                </div>
                <div class="grid grid-cols-2 gap-3">
                  <div class="space-y-1">
                    <label for="col-sigv4-reg" class="text-xs font-semibold text-apple-body-on-dark">Region</label>
                    <input
                      id="col-sigv4-reg"
                      type="text"
                      bind:value={colAuth.awsRegion}
                      placeholder="us-east-1"
                      class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                  </div>
                  <div class="space-y-1">
                    <label for="col-sigv4-srv" class="text-xs font-semibold text-apple-body-on-dark">Service</label>
                    <input
                      id="col-sigv4-srv"
                      type="text"
                      bind:value={colAuth.awsService}
                      placeholder="execute-api"
                      class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                  </div>
                </div>
              </div>
            {/if}
          </div>

        {:else if activeTab === 'headers'}
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-apple-body-muted text-xs">Default headers appended to all requests in this collection:</span>
              <button
                type="button"
                onclick={addHeader}
                class="apple-button-dark-utility text-xs px-2.5 py-1 text-apple-primary-on-dark border border-apple-hairline-dark"
              >
                + Add Header
              </button>
            </div>
            {#if colHeaders.length === 0}
              <div class="text-apple-ink-muted-48 italic py-6 text-center">No collection headers configured yet.</div>
            {:else}
              <div class="space-y-2">
                {#each colHeaders as h, idx}
                  <div class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={h.enabled}
                      class="w-4 h-4 rounded bg-apple-surface-black border-apple-hairline-dark"
                    />
                    <input
                      type="text"
                      bind:value={h.key}
                      placeholder="Header Key..."
                      class="flex-1 bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-2.5 py-1 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                    <input
                      type="text"
                      bind:value={h.value}
                      placeholder="Header Value..."
                      class="flex-1 bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-2.5 py-1 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                    <button
                      type="button"
                      onclick={() => removeHeader(idx)}
                      class="text-apple-body-muted hover:text-rose-400 p-1"
                    >
                      ✕
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

        {:else if activeTab === 'variables'}
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-apple-body-muted text-xs">Scoped variables evaluated for requests in this collection:</span>
              <button
                type="button"
                onclick={addVariable}
                class="apple-button-dark-utility text-xs px-2.5 py-1 text-apple-primary-on-dark border border-apple-hairline-dark"
              >
                + Add Variable
              </button>
            </div>
            {#if colVariables.length === 0}
              <div class="text-apple-ink-muted-48 italic py-6 text-center">No collection variables configured yet.</div>
            {:else}
              <div class="space-y-2">
                {#each colVariables as v, idx}
                  <div class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={v.enabled}
                      class="w-4 h-4 rounded bg-apple-surface-black border-apple-hairline-dark"
                    />
                    <input
                      type="text"
                      bind:value={v.key}
                      placeholder="Variable name..."
                      class="w-1/3 bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-2.5 py-1 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                    <input
                      type={v.isSecret ? 'password' : 'text'}
                      bind:value={v.value}
                      placeholder="Value..."
                      class="flex-1 bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-2.5 py-1 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                    />
                    <button
                      type="button"
                      onclick={() => v.isSecret = !v.isSecret}
                      class="text-xs p-1 rounded hover:bg-apple-surface-tile-2 text-apple-body-muted"
                      title={v.isSecret ? 'Masked Secret' : 'Plain Text'}
                    >
                      {v.isSecret ? '🔒' : '🔓'}
                    </button>
                    <button
                      type="button"
                      onclick={() => removeVariable(idx)}
                      class="text-apple-body-muted hover:text-rose-400 p-1"
                    >
                      ✕
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

        {:else if activeTab === 'scripts'}
          <div class="space-y-4">
            <div class="space-y-1">
              <label for="col-pre-script" class="text-xs font-semibold text-apple-body-on-dark">Pre-Request Script</label>
              <textarea
                id="col-pre-script"
                rows="4"
                bind:value={preScript}
                placeholder="// Runs before every request in this collection..."
                class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm p-3 font-mono text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
              ></textarea>
            </div>
            <div class="space-y-1">
              <label for="col-post-script" class="text-xs font-semibold text-apple-body-on-dark">Post-Response Tests</label>
              <textarea
                id="col-post-script"
                rows="4"
                bind:value={postScript}
                placeholder="// Runs after every request in this collection..."
                class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm p-3 font-mono text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
              ></textarea>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-3 border-t border-apple-hairline-dark flex justify-end gap-2 bg-apple-surface-tile-2 shrink-0">
        <button
          type="button"
          onclick={onClose}
          class="apple-button-dark-utility text-xs px-4 py-1.5"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleSave}
          class="apple-button-primary text-xs px-5 py-1.5 font-semibold"
        >
          Save Collection Settings
        </button>
      </div>
    </div>
  </div>
{/if}
