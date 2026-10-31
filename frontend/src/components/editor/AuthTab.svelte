<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';
  import {
    generateCodeVerifier,
    generateCodeChallenge,
    buildAuthorizationUrl,
    exchangeOAuthToken
  } from '../../utils/oauthUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let showBearerSecret = $state(false);
  let showSigv4Secret = $state(false);
  let showClientSecret = $state(false);

  let authCodeInput = $state('');
  let isFetchingToken = $state(false);
  let oauthFeedback = $state('');
  let oauthError = $state('');

  let parentCollection = $derived(
    workspaceStore.currentWorkspace.collections.find(c => c.id === req.collectionId)
  );

  async function handleGeneratePkce() {
    const verifier = generateCodeVerifier(64);
    req.auth.oauth2CodeVerifier = verifier;
    workspaceStore.updateActiveRequest({ auth: req.auth });
    oauthFeedback = 'Generated PKCE Code Verifier';
    setTimeout(() => oauthFeedback = '', 3000);
  }

  async function handleOpenAuthUrl() {
    if (!req.auth.oauth2AuthUrl || !req.auth.oauth2ClientId) {
      oauthError = 'Please configure Authorization URL and Client ID';
      return;
    }
    oauthError = '';
    let challenge: string | undefined;
    if (req.auth.oauth2Pkce !== false) {
      if (!req.auth.oauth2CodeVerifier) {
        req.auth.oauth2CodeVerifier = generateCodeVerifier(64);
      }
      challenge = await generateCodeChallenge(req.auth.oauth2CodeVerifier);
    }
    const redirectUri = req.auth.oauth2RedirectUri || 'http://localhost:8080/callback';
    const authUrl = buildAuthorizationUrl({
      authUrl: req.auth.oauth2AuthUrl,
      clientId: req.auth.oauth2ClientId,
      redirectUri,
      scope: req.auth.oauth2Scope,
      codeChallenge: challenge
    });

    window.open(authUrl, '_blank');
    oauthFeedback = 'Opened browser for authorization';
    setTimeout(() => oauthFeedback = '', 3000);
  }

  async function handleFetchToken() {
    if (!req.auth.oauth2TokenUrl || !req.auth.oauth2ClientId) {
      oauthError = 'Token URL and Client ID are required';
      return;
    }
    isFetchingToken = true;
    oauthError = '';
    oauthFeedback = '';

    const grantType = req.auth.oauth2GrantType || 'authorization_code';

    try {
      const res = await exchangeOAuthToken({
        tokenUrl: req.auth.oauth2TokenUrl,
        grantType,
        clientId: req.auth.oauth2ClientId,
        clientSecret: req.auth.oauth2ClientSecret,
        code: grantType === 'authorization_code' ? authCodeInput : undefined,
        redirectUri: req.auth.oauth2RedirectUri || 'http://localhost:8080/callback',
        codeVerifier: req.auth.oauth2CodeVerifier,
        scope: req.auth.oauth2Scope
      });

      if (res.access_token) {
        req.auth.oauth2AccessToken = res.access_token;
        req.auth.bearerToken = res.access_token;
        workspaceStore.updateActiveRequest({ auth: req.auth });
        oauthFeedback = 'Successfully retrieved Access Token!';
      } else {
        oauthError = 'No access_token in token response';
      }
    } catch (err: any) {
      oauthError = err.message || 'Token request failed';
    } finally {
      isFetchingToken = false;
    }
  }

  function handleClearToken() {
    req.auth.oauth2AccessToken = '';
    workspaceStore.updateActiveRequest({ auth: req.auth });
    oauthFeedback = 'Cleared token';
    setTimeout(() => oauthFeedback = '', 2000);
  }
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
      <option value="oauth2">OAuth 2.0</option>
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
          placeholder="Bearer token or {{token}}"
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

  {:else if req.auth.type === 'oauth2'}
    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-4">
      <div class="flex items-center justify-between pb-2 border-b border-apple-hairline-dark/60">
        <span class="text-xs font-bold text-apple-primary-on-dark">OAuth 2.0 Configuration</span>
        {#if req.auth.oauth2AccessToken}
          <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-emerald-950/80 text-emerald-400 border border-emerald-800 font-mono">
            Token Active
          </span>
        {:else}
          <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-amber-950/80 text-amber-400 border border-amber-800 font-mono">
            No Token
          </span>
        {/if}
      </div>

      <!-- Grant Type -->
      <div class="space-y-1">
        <label for="oauth-grant-type" class="text-xs font-semibold text-apple-body-on-dark">Grant Type</label>
        <select
          id="oauth-grant-type"
          bind:value={req.auth.oauth2GrantType}
          onchange={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary font-medium"
        >
          <option value="authorization_code">Authorization Code (with PKCE)</option>
          <option value="client_credentials">Client Credentials</option>
        </select>
      </div>

      <!-- Endpoints Grid -->
      <div class="grid grid-cols-2 gap-3">
        {#if req.auth.oauth2GrantType !== 'client_credentials'}
          <div class="space-y-1">
            <label for="oauth-auth-url" class="text-xs font-semibold text-apple-body-on-dark">Authorization URL</label>
            <input
              id="oauth-auth-url"
              type="text"
              bind:value={req.auth.oauth2AuthUrl}
              placeholder="https://auth.example.com/oauth/authorize"
              oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
              class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
            />
          </div>
        {/if}

        <div class="space-y-1 {req.auth.oauth2GrantType === 'client_credentials' ? 'col-span-2' : ''}">
          <label for="oauth-token-url" class="text-xs font-semibold text-apple-body-on-dark">Access Token URL</label>
          <input
            id="oauth-token-url"
            type="text"
            bind:value={req.auth.oauth2TokenUrl}
            placeholder="https://auth.example.com/oauth/token"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
      </div>

      <!-- Client Credentials -->
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label for="oauth-client-id" class="text-xs font-semibold text-apple-body-on-dark">Client ID</label>
          <input
            id="oauth-client-id"
            type="text"
            bind:value={req.auth.oauth2ClientId}
            placeholder="Client ID or {{client_id}}"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>

        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <label for="oauth-client-secret" class="text-xs font-semibold text-apple-body-on-dark">Client Secret</label>
            <button
              type="button"
              onclick={() => showClientSecret = !showClientSecret}
              class="text-[10px] text-apple-link-on-dark hover:underline"
            >
              {showClientSecret ? 'Hide' : 'Show'}
            </button>
          </div>
          <input
            id="oauth-client-secret"
            type={showClientSecret ? 'text' : 'password'}
            bind:value={req.auth.oauth2ClientSecret}
            placeholder="Client Secret or {{client_secret}}"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
      </div>

      <!-- Scope & Redirect URI -->
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label for="oauth-scope" class="text-xs font-semibold text-apple-body-on-dark">Scope (Optional)</label>
          <input
            id="oauth-scope"
            type="text"
            bind:value={req.auth.oauth2Scope}
            placeholder="e.g. read write openid"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>

        <div class="space-y-1">
          <label for="oauth-redirect-uri" class="text-xs font-semibold text-apple-body-on-dark">Redirect URI</label>
          <input
            id="oauth-redirect-uri"
            type="text"
            bind:value={req.auth.oauth2RedirectUri}
            placeholder="http://localhost:8080/callback"
            oninput={() => workspaceStore.updateActiveRequest({ auth: req.auth })}
            class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
        </div>
      </div>

      <!-- PKCE Section for Authorization Code -->
      {#if req.auth.oauth2GrantType !== 'client_credentials'}
        <div class="p-3 bg-apple-surface-tile-1 border border-apple-hairline-dark/40 rounded-apple-sm space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold text-apple-body-muted">PKCE (Proof Key for Code Exchange)</span>
            <button
              type="button"
              onclick={handleGeneratePkce}
              class="apple-button-dark-utility px-2 py-0.5 rounded-apple-sm text-[10px] bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 border border-apple-hairline-dark/60 text-apple-primary-on-dark cursor-pointer font-medium"
            >
              Generate Verifier
            </button>
          </div>
          {#if req.auth.oauth2CodeVerifier}
            <div class="text-[10px] font-mono text-apple-ink-muted-48 break-all">
              <span class="text-apple-body-muted">Verifier:</span> {req.auth.oauth2CodeVerifier}
            </div>
          {/if}

          <!-- Code input from redirect -->
          <div class="pt-1 space-y-1">
            <label for="auth-code-input" class="text-[11px] text-apple-body-muted">Authorization Code (from callback redirect)</label>
            <input
              id="auth-code-input"
              type="text"
              bind:value={authCodeInput}
              placeholder="Paste code parameter from authorization redirect..."
              class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
            />
          </div>
        </div>
      {/if}

      <!-- Flow Actions -->
      <div class="flex items-center gap-2 pt-2">
        {#if req.auth.oauth2GrantType !== 'client_credentials'}
          <button
            type="button"
            onclick={handleOpenAuthUrl}
            class="px-3 py-1.5 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 border border-apple-hairline-dark/80 text-apple-body-on-dark font-medium text-xs transition-colors cursor-pointer"
          >
            Authorize in Browser ↗
          </button>
        {/if}

        <button
          type="button"
          onclick={handleFetchToken}
          disabled={isFetchingToken}
          class="apple-button-primary px-3 py-1.5 rounded-apple-sm bg-apple-primary hover:bg-apple-primary-hover text-white font-semibold text-xs transition-colors cursor-pointer disabled:opacity-50"
        >
          {isFetchingToken ? 'Exchanging...' : 'Fetch Access Token'}
        </button>

        {#if req.auth.oauth2AccessToken}
          <button
            type="button"
            onclick={handleClearToken}
            class="px-2.5 py-1.5 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-ink-muted-48 hover:text-white text-xs border border-apple-hairline-dark/60 cursor-pointer"
          >
            Clear Token
          </button>
        {/if}
      </div>

      <!-- Feedback / Errors -->
      {#if oauthFeedback}
        <div class="p-2 rounded bg-emerald-950/60 border border-emerald-800 text-emerald-300 text-xs font-mono">
          ✓ {oauthFeedback}
        </div>
      {/if}
      {#if oauthError}
        <div class="p-2 rounded bg-rose-950/60 border border-rose-800 text-rose-300 text-xs font-mono">
          ✕ {oauthError}
        </div>
      {/if}

      <!-- Token Viewer -->
      {#if req.auth.oauth2AccessToken}
        <div class="p-3 bg-apple-surface-tile-3/80 rounded-apple-md border border-apple-hairline-dark/80 space-y-1.5">
          <div class="flex items-center justify-between text-[11px]">
            <span class="text-apple-body-muted font-semibold">Active Access Token (Bearer)</span>
            <button
              type="button"
              onclick={() => { navigator.clipboard.writeText(req.auth.oauth2AccessToken || ''); oauthFeedback = 'Copied Token'; setTimeout(() => oauthFeedback = '', 2000); }}
              class="text-apple-link-on-dark hover:underline cursor-pointer"
            >
              Copy
            </button>
          </div>
          <div class="font-mono text-[10px] text-emerald-400 break-all bg-apple-surface-black/60 p-2 rounded border border-apple-hairline-dark/40 max-h-20 overflow-auto">
            {req.auth.oauth2AccessToken}
          </div>
        </div>
      {/if}
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
          placeholder="Password or {{password}}"
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
            placeholder="Key value or {{api_key}}"
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
