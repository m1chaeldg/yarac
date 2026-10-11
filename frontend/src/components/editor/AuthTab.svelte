<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let showBearerSecret = $state(false);
</script>

<div class="max-w-xl space-y-4 font-sans text-xs">
  <div class="flex items-center gap-3">
    <label for="auth-type-select" class="text-xs text-slate-400 font-medium">Auth Type:</label>
    <select
      id="auth-type-select"
      bind:value={req.auth.type}
      onchange={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
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
          placeholder="Bearer token or {`{{token}}`}"
          oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
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
          oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
          class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
        />
      </div>
      <div class="space-y-1">
        <label for="basic-password-input" class="text-xs font-semibold text-slate-300">Password</label>
        <input
          id="basic-password-input"
          type="password"
          bind:value={req.auth.basicPassword}
          placeholder="Password or {`{{password}}`}"
          oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
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
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
          />
        </div>
        <div class="space-y-1">
          <label for="apikey-value-input" class="text-xs font-semibold text-slate-300">Value</label>
          <input
            id="apikey-value-input"
            type="text"
            bind:value={req.auth.apiKeyValue}
            placeholder="Key value or {`{{api_key}}`}"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
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
