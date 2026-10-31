<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';

  interface Props {
    req: YaracRequest;
    protocol: 'ws' | 'sse';
  }

  let { req, protocol }: Props = $props();

  interface StreamMessage {
    id: string;
    timestamp: string;
    direction: 'in' | 'out';
    data: string;
    size: number;
    isBinary?: boolean;
  }

  let isConnected = $state(false);
  let isConnecting = $state(false);
  let statusText = $state('Disconnected');
  let messages = $state<StreamMessage[]>([]);
  let messageFilter = $state<'all' | 'in' | 'out'>('all');
  let outgoingMessage = $state('{\n  "event": "ping",\n  "timestamp": ' + Date.now() + '\n}');

  let activeWs: WebSocket | null = null;
  let activeEventSource: EventSource | null = null;

  function resolveUrlWithEnv(rawUrl: string): string {
    let url = rawUrl;
    const activeEnv = workspaceStore.currentWorkspace.environments.find(
      e => e.id === workspaceStore.currentWorkspace.activeEnvironmentId
    );
    if (activeEnv) {
      for (const v of activeEnv.variables) {
        if (v.enabled && v.key) {
          url = url.replace(new RegExp(`\\{\\{${v.key}\\}\\}`, 'g'), v.value);
        }
      }
    }
    return url;
  }

  function connect() {
    disconnect();
    isConnecting = true;
    statusText = 'Connecting...';

    const resolvedUrl = resolveUrlWithEnv(req.url);

    if (protocol === 'ws') {
      try {
        let wsUrl = resolvedUrl;
        if (wsUrl.startsWith('http://')) wsUrl = 'ws://' + wsUrl.substring(7);
        else if (wsUrl.startsWith('https://')) wsUrl = 'wss://' + wsUrl.substring(8);
        else if (!wsUrl.startsWith('ws://') && !wsUrl.startsWith('wss://')) {
          wsUrl = 'ws://' + wsUrl;
        }

        activeWs = new WebSocket(wsUrl);

        activeWs.onopen = () => {
          isConnected = true;
          isConnecting = false;
          statusText = 'Connected to ' + wsUrl;
          addMessage('in', `[SYSTEM] Connected to ${wsUrl}`, 0);
        };

        activeWs.onmessage = (event) => {
          const dataStr = typeof event.data === 'string' ? event.data : '[Binary Frame]';
          addMessage('in', dataStr, typeof event.data === 'string' ? event.data.length : 0);
        };

        activeWs.onerror = (e) => {
          statusText = 'Connection Error';
          addMessage('in', '[SYSTEM ERROR] WebSocket connection error', 0);
        };

        activeWs.onclose = (event) => {
          isConnected = false;
          isConnecting = false;
          statusText = `Closed (code: ${event.code})`;
          addMessage('in', `[SYSTEM] Connection closed (code ${event.code})`, 0);
          activeWs = null;
        };
      } catch (err: any) {
        isConnecting = false;
        statusText = 'Failed to connect: ' + err.message;
        addMessage('in', `[SYSTEM ERROR] ${err.message}`, 0);
      }
    } else {
      // Server-Sent Events (SSE)
      try {
        activeEventSource = new EventSource(resolvedUrl);

        activeEventSource.onopen = () => {
          isConnected = true;
          isConnecting = false;
          statusText = 'SSE Stream Connected';
          addMessage('in', `[SYSTEM] SSE Connection opened to ${resolvedUrl}`, 0);
        };

        activeEventSource.onmessage = (event) => {
          addMessage('in', event.data, event.data.length);
        };

        activeEventSource.onerror = () => {
          isConnected = false;
          isConnecting = false;
          statusText = 'SSE Connection Closed / Error';
          addMessage('in', '[SYSTEM] SSE Stream disconnected or connection failed', 0);
          activeEventSource?.close();
          activeEventSource = null;
        };
      } catch (err: any) {
        isConnecting = false;
        statusText = 'Failed: ' + err.message;
        addMessage('in', `[SYSTEM ERROR] ${err.message}`, 0);
      }
    }
  }

  function disconnect() {
    if (activeWs) {
      activeWs.close();
      activeWs = null;
    }
    if (activeEventSource) {
      activeEventSource.close();
      activeEventSource = null;
    }
    isConnected = false;
    isConnecting = false;
    statusText = 'Disconnected';
  }

  function addMessage(direction: 'in' | 'out', data: string, size: number) {
    messages = [
      ...messages,
      {
        id: Math.random().toString(36).substring(2, 9),
        timestamp: new Date().toLocaleTimeString(),
        direction,
        data,
        size
      }
    ];
  }

  function sendMessage() {
    if (!activeWs || !isConnected || !outgoingMessage.trim()) return;
    try {
      activeWs.send(outgoingMessage);
      addMessage('out', outgoingMessage, outgoingMessage.length);
    } catch (err: any) {
      addMessage('in', `[SEND ERROR] ${err.message}`, 0);
    }
  }

  let filteredMessages = $derived.by(() => {
    if (messageFilter === 'in') return messages.filter(m => m.direction === 'in');
    if (messageFilter === 'out') return messages.filter(m => m.direction === 'out');
    return messages;
  });

  function clearMessages() {
    messages = [];
  }

  onDestroy(() => {
    disconnect();
  });
</script>

<div class="h-full flex flex-col bg-apple-surface-black font-sans text-xs select-none">
  <!-- Realtime Toolbar Header -->
  <div class="h-10 border-b border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-2.5">
      <span class="text-sm">{protocol === 'ws' ? '⚡' : '📡'}</span>
      <span class="font-bold text-apple-body-on-dark">{protocol === 'ws' ? 'WebSocket Client' : 'Server-Sent Events (SSE)'}</span>
      <span class="w-2 h-2 rounded-apple-full {isConnected ? 'bg-emerald-400' : isConnecting ? 'bg-amber-400 animate-pulse' : 'bg-apple-ink-muted-48'}"></span>
      <span class="text-[11px] font-mono {isConnected ? 'text-emerald-400' : 'text-apple-body-muted'}">{statusText}</span>
    </div>

    <div class="flex items-center gap-2">
      <!-- Filter Controls -->
      <div class="flex items-center bg-apple-surface-tile-3 rounded-apple-pill p-0.5 border border-apple-hairline-dark">
        <button
          type="button"
          onclick={() => messageFilter = 'all'}
          class="px-2 py-0.5 text-[10px] rounded-apple-pill font-medium {messageFilter === 'all' ? 'bg-apple-primary text-white' : 'text-apple-body-muted hover:text-white'}"
        >
          All ({messages.length})
        </button>
        <button
          type="button"
          onclick={() => messageFilter = 'in'}
          class="px-2 py-0.5 text-[10px] rounded-apple-pill font-medium {messageFilter === 'in' ? 'bg-apple-primary text-white' : 'text-apple-body-muted hover:text-white'}"
        >
          ⬇ Received
        </button>
        {#if protocol === 'ws'}
          <button
            type="button"
            onclick={() => messageFilter = 'out'}
            class="px-2 py-0.5 text-[10px] rounded-apple-pill font-medium {messageFilter === 'out' ? 'bg-apple-primary text-white' : 'text-apple-body-muted hover:text-white'}"
          >
            ⬆ Sent
          </button>
        {/if}
      </div>

      <button
        type="button"
        onclick={clearMessages}
        class="apple-button-dark-utility px-2.5 py-0.5 text-xs text-apple-body-muted hover:text-white rounded-apple-sm"
        title="Clear timeline"
      >
        Clear
      </button>

      {#if isConnected || isConnecting}
        <button
          type="button"
          onclick={disconnect}
          class="apple-button-primary px-3 py-1 text-xs bg-rose-600 hover:bg-rose-500 font-semibold"
        >
          Disconnect
        </button>
      {:else}
        <button
          type="button"
          onclick={connect}
          class="apple-button-primary px-3.5 py-1 text-xs font-semibold"
        >
          Connect
        </button>
      {/if}
    </div>
  </div>

  <!-- Messages Timeline Feed -->
  <div class="flex-1 overflow-auto p-3 space-y-1.5 font-mono text-xs">
    {#each filteredMessages as msg (msg.id)}
      <div class="p-2 rounded-apple-sm border transition-colors
        {msg.direction === 'out'
          ? 'bg-apple-primary/10 border-apple-primary/30 text-apple-primary-on-dark ml-8'
          : msg.data.startsWith('[SYSTEM')
            ? 'bg-apple-surface-tile-3 border-apple-hairline-dark text-apple-body-muted'
            : 'bg-apple-surface-tile-2 border-apple-hairline-dark text-apple-body-on-dark mr-8'}"
      >
        <div class="flex items-center justify-between text-[10px] text-apple-ink-muted-48 mb-1">
          <span class="flex items-center gap-1 font-sans">
            <span>{msg.direction === 'out' ? '⬆ Sent' : '⬇ Received'}</span>
            {#if msg.size > 0}
              <span>• {msg.size} bytes</span>
            {/if}
          </span>
          <span>{msg.timestamp}</span>
        </div>
        <div class="whitespace-pre-wrap break-all leading-relaxed select-text font-mono">
          {msg.data}
        </div>
      </div>
    {/each}

    {#if filteredMessages.length === 0}
      <div class="h-full flex items-center justify-center text-apple-ink-muted-48 italic font-sans">
        {isConnected ? 'Connected. Waiting for incoming stream frames...' : 'Click "Connect" above to begin streaming.'}
      </div>
    {/if}
  </div>

  <!-- WebSocket Message Composer (Only for WebSocket) -->
  {#if protocol === 'ws'}
    <div class="p-3 border-t border-apple-hairline-dark bg-apple-surface-tile-2 shrink-0 space-y-2">
      <div class="flex items-center justify-between text-[11px] text-apple-body-muted">
        <span class="font-medium">Send Message Frame</span>
        <button
          type="button"
          onclick={() => {
            try {
              outgoingMessage = JSON.stringify(JSON.parse(outgoingMessage), null, 2);
            } catch (_) {}
          }}
          class="text-apple-primary-on-dark hover:underline"
        >
          Prettify JSON
        </button>
      </div>
      <div class="flex gap-2">
        <textarea
          bind:value={outgoingMessage}
          placeholder="Message payload (JSON, text)..."
          rows="2"
          class="flex-1 bg-apple-surface-black border border-apple-hairline-dark rounded p-2 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary resize-none leading-relaxed"
        ></textarea>
        <button
          type="button"
          disabled={!isConnected || !outgoingMessage.trim()}
          onclick={sendMessage}
          class="apple-button-primary px-4 py-2 font-semibold text-xs disabled:opacity-40 self-end"
        >
          Send
        </button>
      </div>
    </div>
  {/if}
</div>
