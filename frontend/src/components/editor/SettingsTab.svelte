<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let certFileInput = $state<HTMLInputElement | null>(null);
  let keyFileInput = $state<HTMLInputElement | null>(null);

  function ensureSettings() {
    if (!req.settings) {
      req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
    }
  }

  function handleCertFileSelect(file: File) {
    ensureSettings();
    if (req.settings) {
      req.settings.clientCertPath = `./fixtures/${file.name}`;
      workspaceStore.updateActiveRequest({ settings: req.settings });
    }
  }

  function handleKeyFileSelect(file: File) {
    ensureSettings();
    if (req.settings) {
      req.settings.clientKeyPath = `./fixtures/${file.name}`;
      workspaceStore.updateActiveRequest({ settings: req.settings });
    }
  }
</script>

<div class="max-w-2xl space-y-6 font-sans text-xs">
  <div>
    <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider block mb-2">
      Request Execution Settings
    </span>

    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-4">
      <!-- Follow Redirects -->
      <div class="flex items-center justify-between">
        <div>
          <label for="set-follow-redirect" class="text-xs font-semibold text-apple-body-on-dark block cursor-pointer">Follow Redirects</label>
          <span class="text-[11px] text-apple-ink-muted-48">Automatically follow HTTP 3xx redirection responses</span>
        </div>
        <input
          id="set-follow-redirect"
          type="checkbox"
          checked={req.settings?.followRedirects ?? true}
          onchange={(e) => {
            ensureSettings();
            if (req.settings) {
              req.settings.followRedirects = (e.target as HTMLInputElement).checked;
              workspaceStore.updateActiveRequest({ settings: req.settings });
            }
          }}
          class="w-4 h-4 rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0 cursor-pointer"
        />
      </div>

      <div class="border-t border-apple-hairline-dark/60"></div>

      <!-- Max Redirects -->
      <div class="flex items-center justify-between">
        <div>
          <label for="set-max-redirects" class="text-xs font-semibold text-apple-body-on-dark block">Max Redirects</label>
          <span class="text-[11px] text-apple-ink-muted-48">Maximum number of consecutive redirects before aborting</span>
        </div>
        <input
          id="set-max-redirects"
          type="number"
          min="0"
          max="50"
          value={req.settings?.maxRedirects ?? 5}
          oninput={(e) => {
            ensureSettings();
            if (req.settings) {
              req.settings.maxRedirects = parseInt((e.target as HTMLInputElement).value || '5', 10);
              workspaceStore.updateActiveRequest({ settings: req.settings });
            }
          }}
          class="w-24 bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary text-right"
        />
      </div>

      <div class="border-t border-apple-hairline-dark/60"></div>

      <!-- Timeout (ms) -->
      <div class="flex items-center justify-between">
        <div>
          <label for="set-timeout" class="text-xs font-semibold text-apple-body-on-dark block">Timeout (ms)</label>
          <span class="text-[11px] text-apple-ink-muted-48">Maximum execution time before request cancels. <strong>0</strong> = indefinite (no timeout)</span>
        </div>
        <div class="flex items-center gap-2">
          <input
            id="set-timeout"
            type="number"
            min="0"
            step="500"
            value={req.settings?.timeoutMs ?? 0}
            oninput={(e) => {
              ensureSettings();
              if (req.settings) {
                req.settings.timeoutMs = parseInt((e.target as HTMLInputElement).value || '0', 10);
                workspaceStore.updateActiveRequest({ settings: req.settings });
              }
            }}
            class="w-28 bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary text-right"
          />
          <span class="text-[11px] text-apple-body-muted font-mono">ms</span>
        </div>
      </div>

      <div class="border-t border-apple-hairline-dark/60"></div>

      <!-- Verify SSL -->
      <div class="flex items-center justify-between">
        <div>
          <label for="set-verify-ssl" class="text-xs font-semibold text-apple-body-on-dark block cursor-pointer">Verify SSL Certificates</label>
          <span class="text-[11px] text-apple-ink-muted-48">Enforce valid TLS / HTTPS certificates for remote server</span>
        </div>
        <input
          id="set-verify-ssl"
          type="checkbox"
          checked={req.settings?.verifySsl ?? true}
          onchange={(e) => {
            ensureSettings();
            if (req.settings) {
              req.settings.verifySsl = (e.target as HTMLInputElement).checked;
              workspaceStore.updateActiveRequest({ settings: req.settings });
            }
          }}
          class="w-4 h-4 rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0 cursor-pointer"
        />
      </div>

      <div class="border-t border-apple-hairline-dark/60"></div>

      <!-- Inherit Auth -->
      <div class="flex items-center justify-between">
        <div>
          <label for="set-inherit-auth" class="text-xs font-semibold text-apple-body-on-dark block cursor-pointer">Inherit Collection Auth</label>
          <span class="text-[11px] text-apple-ink-muted-48">Automatically attach parent collection authentication credentials</span>
        </div>
        <input
          id="set-inherit-auth"
          type="checkbox"
          checked={req.settings?.inheritAuth ?? (req.auth.type === 'inherit')}
          onchange={(e) => {
            ensureSettings();
            if (req.settings) {
              req.settings.inheritAuth = (e.target as HTMLInputElement).checked;
              workspaceStore.updateActiveRequest({ settings: req.settings });
            }
          }}
          class="w-4 h-4 rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0 cursor-pointer"
        />
      </div>
    </div>
  </div>

  <!-- mTLS Client Certificates Section -->
  <div>
    <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider block mb-2">
      mTLS Client Certificates
    </span>

    <input
      type="file"
      accept=".pem,.crt,.cer,.p12,.pfx"
      bind:this={certFileInput}
      onchange={(e) => {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (file) handleCertFileSelect(file);
        (e.target as HTMLInputElement).value = '';
      }}
      class="hidden"
    />

    <input
      type="file"
      accept=".key,.pem"
      bind:this={keyFileInput}
      onchange={(e) => {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (file) handleKeyFileSelect(file);
        (e.target as HTMLInputElement).value = '';
      }}
      class="hidden"
    />

    <div class="p-4 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg space-y-4">
      <!-- Certificate File -->
      <div>
        <label for="set-client-cert" class="text-xs font-semibold text-apple-body-on-dark block mb-1">Client Certificate (.pem, .crt)</label>
        <div class="flex items-center gap-2">
          <input
            id="set-client-cert"
            type="text"
            placeholder="e.g. ./fixtures/certs/client.crt"
            value={req.settings?.clientCertPath || ''}
            oninput={(e) => {
              ensureSettings();
              if (req.settings) {
                req.settings.clientCertPath = (e.target as HTMLInputElement).value;
                workspaceStore.updateActiveRequest({ settings: req.settings });
              }
            }}
            class="flex-1 bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
          <button
            type="button"
            onclick={() => certFileInput?.click()}
            class="apple-button-dark-utility px-2.5 py-1 text-xs text-apple-primary-on-dark hover:bg-apple-surface-tile-3 shrink-0"
          >
            📁 Browse...
          </button>
          {#if req.settings?.clientCertPath}
            <button
              type="button"
              onclick={() => {
                if (req.settings) {
                  req.settings.clientCertPath = undefined;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }
              }}
              class="text-apple-ink-muted-48 hover:text-rose-400 text-xs px-1"
              title="Clear certificate"
            >
              ✕
            </button>
          {/if}
        </div>
      </div>

      <!-- Private Key File -->
      <div>
        <label for="set-client-key" class="text-xs font-semibold text-apple-body-on-dark block mb-1">Client Private Key (.key)</label>
        <div class="flex items-center gap-2">
          <input
            id="set-client-key"
            type="text"
            placeholder="e.g. ./fixtures/certs/client.key"
            value={req.settings?.clientKeyPath || ''}
            oninput={(e) => {
              ensureSettings();
              if (req.settings) {
                req.settings.clientKeyPath = (e.target as HTMLInputElement).value;
                workspaceStore.updateActiveRequest({ settings: req.settings });
              }
            }}
            class="flex-1 bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
          <button
            type="button"
            onclick={() => keyFileInput?.click()}
            class="apple-button-dark-utility px-2.5 py-1 text-xs text-apple-primary-on-dark hover:bg-apple-surface-tile-3 shrink-0"
          >
            📁 Browse...
          </button>
          {#if req.settings?.clientKeyPath}
            <button
              type="button"
              onclick={() => {
                if (req.settings) {
                  req.settings.clientKeyPath = undefined;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }
              }}
              class="text-apple-ink-muted-48 hover:text-rose-400 text-xs px-1"
              title="Clear private key"
            >
              ✕
            </button>
          {/if}
        </div>
      </div>

      <!-- Passphrase -->
      <div>
        <label for="set-client-passphrase" class="text-xs font-semibold text-apple-body-on-dark block mb-1">Key Passphrase (Optional)</label>
        <input
          id="set-client-passphrase"
          type="password"
          placeholder="Private key passphrase if encrypted"
          value={req.settings?.clientCertPassphrase || ''}
          oninput={(e) => {
            ensureSettings();
            if (req.settings) {
              req.settings.clientCertPassphrase = (e.target as HTMLInputElement).value;
              workspaceStore.updateActiveRequest({ settings: req.settings });
            }
          }}
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-mono text-amber-400 focus:outline-none focus:border-apple-primary"
        />
      </div>

      <div class="apple-micro-legal text-apple-ink-muted-48 leading-relaxed pt-1 border-t border-apple-hairline-dark/60 font-sans">
        💡 <span class="font-medium text-apple-body-muted">Relative Path Anchoring:</span> All client certificates and private keys resolve relative to the workspace root directory.
      </div>
    </div>
  </div>
</div>
