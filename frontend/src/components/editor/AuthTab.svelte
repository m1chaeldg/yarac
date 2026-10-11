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
    <label for="auth-type-select" class="text-xs text-apple-body-muted font-medium">Auth Type:</label>
    <select
      id="auth-type-select"
      bind:value={req.auth.type}
      onchange={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
      class="bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-apple-body-on-dark text-xs rounded-apple-sm px-2.5 py-1.5 focus:outline-none focus:border-apple-primary font-medium"
    >
      <option value="inherit">Inherit auth from Collection</option>
      <option value="bearer">Bearer Token</option>
      <option value="basic">Basic Auth</option>
      <option value="apikey">API Key</option>
      <option value="none">No Auth</option>
    </select>
  </div>

  {#if req.auth.type === 'bearer'}
    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-2">
      <label for="bearer-token-input" class="text-xs font-semibold text-apple-body-on-dark">Bearer Token</label>
      <div class="relative">
        <input
          id="bearer-token-input"
          type={showBearerSecret ? 'text' : 'password'}
          bind:value={req.auth.bearerToken}
          placeholder="Bearer token or {`{{token}}`}"
          oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark focus:outline-none focus:border-apple-primary pr-16"
        />
        <button
          type="button"
          onclick={() => showBearerSecret = !showBearerSecret}
          class="apple-button-dark-utility absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-apple-body-muted hover:text-white px-2 py-0.5 rounded-apple-sm border border-apple-hairline-dark/60 bg-apple-surface-tile-3 transition-colors cursor-pointer"
        >
          {showBearerSecret ? 'Hide' : 'Show'}
        </button>
      </div>
      <p class="text-[11px] text-apple-ink-muted-48">
        Variables like <code class="text-apple-primary-on-dark font-mono">{"{{api_bearer_token}}"}</code> are automatically resolved.
      </p>
    </div>

  {:else if req.auth.type === 'basic'}
    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-3">
      <div class="space-y-1">
        <label for="basic-username-input" class="text-xs font-semibold text-apple-body-on-dark">Username</label>
        <input
          id="basic-username-input"
          type="text"
          bind:value={req.auth.basicUsername}
          placeholder="e.g. admin"
          oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
        />
      </div>
      <div class="space-y-1">
        <label for="basic-password-input" class="text-xs font-semibold text-apple-body-on-dark">Password</label>
        <input
          id="basic-password-input"
          type="password"
          bind:value={req.auth.basicPassword}
          placeholder="Password or {`{{password}}`}"
          oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
        />
      </div>
    </div>

  {:else if req.auth.type === 'apikey'}
    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label for="apikey-name-input" class="text-xs font-semibold text-apple-body-on-dark">Key Name</label>
          <input
            id="apikey-name-input"
            type="text"
            bind:value={req.auth.apiKeyName}
            placeholder="X-API-Key"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
        <div class="space-y-1">
          <label for="apikey-value-input" class="text-xs font-semibold text-apple-body-on-dark">Value</label>
          <input
            id="apikey-value-input"
            type="text"
            bind:value={req.auth.apiKeyValue}
            placeholder="Key value or {`{{api_key}}`}"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
      </div>
    </div>

  {:else if req.auth.type === 'inherit'}
    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg text-apple-body-muted text-xs">
      This request inherits authentication credentials configured at the collection or workspace level.
    </div>
  {/if}
</div>
