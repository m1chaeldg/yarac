<script lang="ts">
  import { backendApi } from '../services/backendApi';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let isRunning = $state(true);
  let transport = $state<'stdio' | 'http'>('stdio');
  let port = $state(9475);
  let token = $state('yarac_mcp_secure_local_token_9941');
  let showToken = $state(false);
  let claudeConfig = $state<string>('');
  let feedback = $state<string | null>(null);

  async function loadMcpDetails() {
    try {
      const status = await backendApi.getMcpStatus();
      isRunning = status.running ?? true;
      transport = (status.transport as any) || 'stdio';
      port = status.port || 9475;

      const cfg = await backendApi.getMcpConfig();
      token = cfg.token || 'yarac_mcp_secure_local_token_9941';
      claudeConfig = JSON.stringify(cfg.claude_config, null, 2);
    } catch (e: any) {
      console.warn('Failed to load MCP status:', e);
    }
  }

  $effect(() => {
    if (isOpen) {
      feedback = null;
      loadMcpDetails();
    }
  });

  async function switchTransport(newTransport: 'stdio' | 'http') {
    transport = newTransport;
    try {
      await backendApi.toggleMcpServer(newTransport);
      feedback = `Switched to ${newTransport.toUpperCase()} transport`;
      setTimeout(() => feedback = null, 2500);
    } catch (e: any) {
      feedback = 'Failed to switch transport';
    }
  }

  async function copyToClipboard(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      feedback = `Copied ${label} to clipboard!`;
      setTimeout(() => feedback = null, 2500);
    } catch (_) {
      feedback = 'Failed to copy';
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 font-sans animate-fade-in">
    <div class="apple-modal-card w-full max-w-3xl h-[600px] flex flex-col text-apple-body-on-dark shadow-2xl">
      <!-- Modal Header -->
      <div class="h-12 border-b border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2.5">
          <span class="text-base text-purple-400">🤖</span>
          <h2 class="apple-body-strong text-apple-body-on-dark tracking-tight">Model Context Protocol (MCP) Server</h2>
          <span class="apple-micro-legal px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-emerald-400 border border-apple-hairline-dark font-mono flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-apple-full bg-emerald-400"></span>
            <span>Active</span>
          </span>
        </div>

        <button
          type="button"
          aria-label="Close MCP modal"
          onclick={onClose}
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
        >
          ✕
        </button>
      </div>

      <!-- Main Body -->
      <div class="flex-1 overflow-auto p-5 space-y-5 bg-apple-surface-tile-1">
        <!-- Transport Configuration Card -->
        <div class="p-4 rounded-apple-md bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <span class="apple-body-strong text-apple-body-on-dark block">Server Transport</span>
              <p class="apple-fine-print text-apple-body-muted">Select communication transport for AI agent orchestrators.</p>
            </div>

            <!-- Transport Selector Pills -->
            <div class="flex items-center bg-apple-surface-tile-3 rounded-apple-pill p-0.5 border border-apple-hairline-dark">
              <button
                type="button"
                onclick={() => switchTransport('stdio')}
                class="px-3 py-1 rounded-apple-pill text-xs font-medium transition-all {transport === 'stdio' ? 'bg-apple-primary text-white font-semibold' : 'text-apple-body-muted hover:text-white'}"
              >
                stdio (Default)
              </button>
              <button
                type="button"
                onclick={() => switchTransport('http')}
                class="px-3 py-1 rounded-apple-pill text-xs font-medium transition-all {transport === 'http' ? 'bg-apple-primary text-white font-semibold' : 'text-apple-body-muted hover:text-white'}"
              >
                HTTP / SSE
              </button>
            </div>
          </div>

          <div class="border-t border-apple-hairline-dark/60 pt-3 text-xs text-apple-body-muted leading-relaxed">
            {#if transport === 'stdio'}
              <p>
                💡 <span class="font-semibold text-apple-body-on-dark">Token-free Stdio Mode:</span> Suitable for local processes launched by Claude Desktop, Antigravity, or Cursor. Standard input and output stream JSON-RPC 2.0 messages directly.
              </p>
            {:else}
              <div class="space-y-2">
                <p>
                  🔒 <span class="font-semibold text-apple-body-on-dark">Authenticated HTTP/SSE Mode:</span> Strictly bound to <code class="text-apple-primary-on-dark font-mono">127.0.0.1:{port}</code> per <code class="font-mono">AGENTS.md</code>. Requires OS Keyring Bearer token on all requests.
                </p>

                <!-- Bearer Token Row -->
                <div class="flex items-center gap-2 pt-1">
                  <div class="flex-1 flex items-center bg-apple-surface-tile-1 border border-apple-hairline-dark rounded px-2.5 py-1">
                    <span class="apple-micro-legal text-apple-ink-muted-48 mr-2 uppercase font-bold">Bearer Token:</span>
                    <span class="font-mono text-xs text-amber-400 flex-1 truncate">
                      {showToken ? token : '••••••••••••••••••••••••••••••••'}
                    </span>
                    <button
                      type="button"
                      onclick={() => showToken = !showToken}
                      class="text-xs text-apple-body-muted hover:text-white ml-2"
                    >
                      {showToken ? '🙈' : '👁️'}
                    </button>
                  </div>

                  <button
                    type="button"
                    onclick={() => copyToClipboard(token, 'Bearer token')}
                    class="apple-button-dark-utility px-3 py-1 text-xs shrink-0"
                  >
                    Copy Token
                  </button>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <!-- Claude Desktop Integration Snippet -->
        <div class="p-4 rounded-apple-md bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-2">
          <div class="flex items-center justify-between">
            <div>
              <span class="apple-body-strong text-apple-body-on-dark block">Claude Desktop Configuration</span>
              <p class="apple-fine-print text-apple-body-muted">Add this block to your <code class="text-apple-primary-on-dark font-mono">claude_desktop_config.json</code>.</p>
            </div>

            <button
              type="button"
              onclick={() => copyToClipboard(claudeConfig, 'Claude Desktop config')}
              class="apple-button-primary px-3 py-1 text-xs font-medium"
            >
              📋 Copy Snippet
            </button>
          </div>

          <pre class="p-3 rounded bg-apple-surface-black border border-apple-hairline-dark font-mono text-xs text-apple-primary-on-dark overflow-x-auto">{claudeConfig}</pre>
        </div>

        <!-- Available MCP Tools -->
        <div class="p-4 rounded-apple-md bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-3">
          <span class="apple-body-strong text-apple-body-on-dark block">Exposed MCP Agent Tools</span>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            <div class="p-3 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark space-y-1">
              <div class="flex items-center gap-2">
                <span class="font-mono font-bold text-apple-primary-on-dark">list_requests</span>
                <span class="apple-micro-legal px-1.5 py-0.2 rounded bg-apple-surface-tile-1 border border-apple-hairline-dark text-apple-body-muted">read</span>
              </div>
              <p class="text-apple-body-muted apple-micro-legal">
                Discovers and returns all <code class="text-apple-primary-on-dark font-mono">.yarac</code> request files and collections across the workspace.
              </p>
            </div>

            <div class="p-3 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark space-y-1">
              <div class="flex items-center gap-2">
                <span class="font-mono font-bold text-apple-primary-on-dark">run_request</span>
                <span class="apple-micro-legal px-1.5 py-0.2 rounded bg-apple-surface-tile-1 border border-apple-hairline-dark text-emerald-400">execute</span>
              </div>
              <p class="text-apple-body-muted apple-micro-legal">
                Executes an endpoint by path with full variable interpolation, scripts, assertions, and secret scrubbing.
              </p>
            </div>
          </div>
        </div>

        {#if feedback}
          <div class="text-xs px-3 py-2 rounded bg-apple-primary/10 border border-apple-primary/40 text-apple-primary-on-dark font-medium animate-fade-in">
            {feedback}
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="h-12 border-t border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <span class="apple-fine-print text-apple-ink-muted-48">
          MCP 2024-11-05 Protocol specification compliant.
        </span>
        <button
          type="button"
          onclick={onClose}
          class="apple-button-dark-utility text-xs px-4 py-1.5 font-semibold"
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
