<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let showBearerSecret = $state(false);
  let showSigv4Secret = $state(false);

  let parentCollection = $derived(
    workspaceStore.currentWorkspace.collections.find(c => c.id === req.collectionId)
  );
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
      <option value="sigv4">AWS SigV4 Signer</option>
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
          <label for="apikey-value-input" class="text-xs font-semibold text-apple-body-on-dark">Key Value</label>
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
      <div class="space-y-1">
        <label for="apikey-placement-select" class="text-xs font-semibold text-apple-body-on-dark">Placement</label>
        <select
          id="apikey-placement-select"
          bind:value={req.auth.apiKeyLocation}
          onchange={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary font-medium"
        >
          <option value="header">Header (e.g. X-API-Key: value)</option>
          <option value="query">Query Parameter (e.g. ?api_key=value)</option>
        </select>
      </div>
    </div>

  {:else if req.auth.type === 'sigv4'}
    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label for="sigv4-access-key" class="text-xs font-semibold text-apple-body-on-dark">Access Key ID</label>
          <input
            id="sigv4-access-key"
            type="text"
            bind:value={req.auth.awsAccessKeyId}
            placeholder="AKIAIOSFODNN7EXAMPLE"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <label for="sigv4-secret-key" class="text-xs font-semibold text-apple-body-on-dark">Secret Access Key</label>
            <button
              type="button"
              onclick={() => showSigv4Secret = !showSigv4Secret}
              class="text-[10px] text-apple-link-on-dark hover:underline"
            >
              {showSigv4Secret ? 'Hide' : 'Show'}
            </button>
          </div>
          <input
            id="sigv4-secret-key"
            type={showSigv4Secret ? 'text' : 'password'}
            bind:value={req.auth.awsSecretAccessKey}
            placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label for="sigv4-region" class="text-xs font-semibold text-apple-body-on-dark">AWS Region</label>
          <input
            id="sigv4-region"
            type="text"
            bind:value={req.auth.awsRegion}
            placeholder="us-east-1"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
        <div class="space-y-1">
          <label for="sigv4-service" class="text-xs font-semibold text-apple-body-on-dark">Service Name</label>
          <input
            id="sigv4-service"
            type="text"
            bind:value={req.auth.awsService}
            placeholder="execute-api"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
      </div>
      <div class="space-y-1">
        <label for="sigv4-session-token" class="text-xs font-semibold text-apple-body-on-dark">Session Token (Optional)</label>
        <input
          id="sigv4-session-token"
          type="password"
          bind:value={req.auth.awsSessionToken}
          placeholder="Optional STS temporary security session token"
          oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
        />
      </div>
    </div>

  {:else if req.auth.type === 'inherit'}
    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-2 text-xs">
      <div class="flex items-center gap-2">
        <span class="text-apple-link-on-dark font-semibold">📁 Inherit Collection Auth</span>
      </div>
      <p class="text-apple-body-muted">
        This request inherits authentication credentials configured on <span class="font-semibold text-white">"{parentCollection?.name || 'Parent Collection'}"</span>.
      </p>
      {#if parentCollection?.auth && parentCollection.auth.type !== 'none' && parentCollection.auth.type !== 'inherit'}
        <div class="mt-2 p-2 bg-apple-surface-tile-3 rounded-apple-sm border border-apple-hairline-dark/80 text-[11px] font-mono">
          <span class="text-apple-ink-muted-48">Active parent auth: </span>
          <span class="text-emerald-400 font-bold uppercase">{parentCollection.auth.type}</span>
        </div>
      {:else}
        <div class="mt-2 p-2 bg-apple-surface-tile-3 rounded-apple-sm border border-apple-hairline-dark/80 text-[11px] text-apple-ink-muted-48">
          No parent authentication configured yet. Configure auth in Collection Settings.
        </div>
      {/if}
    </div>
  {/if}
</div>
