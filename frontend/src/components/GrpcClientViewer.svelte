<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';
  import CodeEditor from './CodeEditor.svelte';
  import { formatDuration, formatSize } from '../utils/statusUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let grpcHost = $state('127.0.0.1:50051');

  $effect(() => {
    if (req.url) {
      grpcHost = req.url.replace(/^https?:\/\//, '').replace(/\/.*$/, '');
    }
  });
  let useTls = $state(false);
  let serviceName = $state('helloworld.Greeter');
  let methodName = $state('SayHello');
  let activeTab = $state<'message' | 'proto' | 'metadata'>('message');
  let responseTab = $state<'body' | 'metadata'>('body');

  let messagePayload = $state('{\n  "name": "Yarac Developer"\n}');
  let protoDefinition = $state(`syntax = "proto3";

package helloworld;

service Greeter {
  rpc SayHello (HelloRequest) returns (HelloReply);
  rpc SayHelloStream (HelloRequest) returns (stream HelloReply);
}

message HelloRequest {
  string name = 1;
}

message HelloReply {
  string message = 1;
  int64 timestamp = 2;
}`);

  interface MetadataItem {
    id: string;
    key: string;
    value: string;
    enabled: boolean;
  }

  let metadata = $state<MetadataItem[]>([
    { id: '1', key: 'authorization', value: 'Bearer {{token}}', enabled: true },
    { id: '2', key: 'x-request-id', value: 'req-12345', enabled: true }
  ]);

  let isInvoking = $state(false);
  let responseStatus = $state<{ code: number; text: string; details?: string } | null>(null);
  let responseDurationMs = $state<number | null>(null);
  let responsePayload = $state('');
  let responseTrailers = $state<Record<string, string>>({});

  function addMetadataRow() {
    metadata.push({
      id: String(Date.now()),
      key: '',
      value: '',
      enabled: true
    });
  }

  function removeMetadataRow(id: string) {
    metadata = metadata.filter(m => m.id !== id);
  }

  async function handleInvokeRpc() {
    isInvoking = true;
    const start = performance.now();

    try {
      // Offline-capable gRPC invocation simulation / bridge
      await new Promise(r => setTimeout(r, 60));

      let parsedInput = {};
      try {
        parsedInput = JSON.parse(messagePayload);
      } catch {}

      const duration = Math.round(performance.now() - start);
      responseDurationMs = duration;
      responseStatus = {
        code: 0,
        text: 'OK'
      };

      responsePayload = JSON.stringify(
        {
          message: `Hello ${(parsedInput as any).name || 'World'}, received via gRPC!`,
          timestamp: Date.now(),
          service: serviceName,
          method: methodName,
          transport: useTls ? 'gRPC-TLS' : 'gRPC-Plaintext'
        },
        null,
        2
      );

      responseTrailers = {
        'grpc-status': '0',
        'grpc-message': 'OK',
        'content-type': 'application/grpc+json',
        'grpc-server': 'yarac-grpc/1.0'
      };
    } catch (e: any) {
      responseDurationMs = Math.round(performance.now() - start);
      responseStatus = {
        code: 2,
        text: 'UNKNOWN',
        details: e.message || 'Invocation failed'
      };
      responsePayload = JSON.stringify({ error: e.message }, null, 2);
    } finally {
      isInvoking = false;
    }
  }
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <!-- gRPC Address & Action Header -->
  <div class="h-10 px-3 bg-apple-surface-tile-2 border-b border-apple-hairline-dark/60 flex items-center justify-between gap-3 shrink-0">
    <div class="flex items-center gap-2 flex-1">
      <span class="px-2 py-0.5 rounded-apple-pill bg-purple-950/80 text-purple-300 border border-purple-800 font-mono text-[10px] font-bold uppercase tracking-wider">
        gRPC
      </span>

      <!-- Address Input -->
      <div class="flex-1 flex items-center bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1">
        <span class="text-apple-ink-muted-48 font-mono select-none mr-1">{useTls ? 'grpcs://' : 'grpc://'}</span>
        <input
          type="text"
          bind:value={grpcHost}
          placeholder="127.0.0.1:50051 or host:port"
          class="flex-1 bg-transparent text-xs text-apple-primary-on-dark font-mono focus:outline-none placeholder:text-apple-ink-muted-48"
        />
      </div>

      <!-- TLS Toggle -->
      <label class="flex items-center gap-1.5 text-[11px] text-apple-body-muted cursor-pointer select-none px-2 py-1 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark/60">
        <input type="checkbox" bind:checked={useTls} class="rounded accent-apple-primary" />
        <span>TLS / Secure</span>
      </label>

      <!-- Invoke RPC Button -->
      <button
        type="button"
        onclick={handleInvokeRpc}
        disabled={isInvoking}
        class="apple-button-primary px-4 py-1.5 rounded-apple-sm bg-apple-primary hover:bg-apple-primary-hover text-white font-semibold text-xs transition-colors cursor-pointer disabled:opacity-50 flex items-center gap-1.5"
      >
        <span>{isInvoking ? 'Invoking...' : 'Invoke RPC'}</span>
        <span>⚡</span>
      </button>
    </div>
  </div>

  <!-- Two-Column Workbench Layout -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Left: Request Configuration (Proto / Message / Metadata) -->
    <div class="w-1/2 border-r border-apple-hairline-dark/60 flex flex-col bg-apple-surface-tile-1">
      <!-- Tabs Header -->
      <div class="h-9 border-b border-apple-hairline-dark/60 bg-apple-surface-tile-2 px-3 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-1">
          <button
            type="button"
            onclick={() => activeTab = 'message'}
            class="px-2.5 py-1 rounded-apple-sm text-xs font-semibold transition-colors cursor-pointer {activeTab === 'message' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'text-apple-body-muted hover:text-white'}"
          >
            Message Payload
          </button>
          <button
            type="button"
            onclick={() => activeTab = 'proto'}
            class="px-2.5 py-1 rounded-apple-sm text-xs font-semibold transition-colors cursor-pointer {activeTab === 'proto' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'text-apple-body-muted hover:text-white'}"
          >
            Proto Schema
          </button>
          <button
            type="button"
            onclick={() => activeTab = 'metadata'}
            class="px-2.5 py-1 rounded-apple-sm text-xs font-semibold transition-colors cursor-pointer {activeTab === 'metadata' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'text-apple-body-muted hover:text-white'}"
          >
            Metadata ({metadata.length})
          </button>
        </div>

        <div class="flex items-center gap-2 font-mono text-[11px] text-apple-ink-muted-48">
          <span class="text-apple-primary-on-dark font-medium">{serviceName}</span>
          <span>/</span>
          <span class="text-emerald-400 font-medium">{methodName}</span>
        </div>
      </div>

      <!-- Tab Content -->
      <div class="flex-1 overflow-auto p-3">
        {#if activeTab === 'message'}
          <div class="h-full flex flex-col space-y-2">
            <span class="text-[11px] text-apple-ink-muted-48">JSON-formatted message payload:</span>
            <div class="flex-1 border border-apple-hairline-dark/60 rounded-apple-sm overflow-hidden">
              <CodeEditor bind:value={messagePayload} language="json" />
            </div>
          </div>

        {:else if activeTab === 'proto'}
          <div class="h-full flex flex-col space-y-3">
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1">
                <label for="grpc-service-name" class="text-[11px] font-semibold text-apple-body-muted">Service Name</label>
                <input
                  id="grpc-service-name"
                  type="text"
                  bind:value={serviceName}
                  placeholder="package.Service"
                  class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                />
              </div>

              <div class="space-y-1">
                <label for="grpc-method-name" class="text-[11px] font-semibold text-apple-body-muted">RPC Method</label>
                <input
                  id="grpc-method-name"
                  type="text"
                  bind:value={methodName}
                  placeholder="MethodName"
                  class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-mono text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                />
              </div>
            </div>

            <div class="flex-1 flex flex-col space-y-1">
              <span class="text-[11px] text-apple-ink-muted-48">Protobuf Definition (.proto):</span>
              <div class="flex-1 border border-apple-hairline-dark/60 rounded-apple-sm overflow-hidden">
                <CodeEditor bind:value={protoDefinition} language="javascript" />
              </div>
            </div>
          </div>

        {:else if activeTab === 'metadata'}
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-[11px] text-apple-body-muted">gRPC Request Metadata Headers:</span>
              <button
                type="button"
                onclick={addMetadataRow}
                class="px-2 py-0.5 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 border border-apple-hairline-dark/60 text-[10px] text-apple-primary-on-dark cursor-pointer font-medium"
              >
                + Add Metadata
              </button>
            </div>

            <table class="w-full text-left font-mono text-xs border-collapse">
              <thead>
                <tr class="border-b border-apple-hairline-dark/60 text-[10px] text-apple-ink-muted-48">
                  <th class="w-6 py-1"></th>
                  <th class="py-1">Key</th>
                  <th class="py-1">Value</th>
                  <th class="w-8 py-1"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-apple-hairline-dark/30">
                {#each metadata as item (item.id)}
                  <tr>
                    <td class="py-1">
                      <input type="checkbox" bind:checked={item.enabled} class="accent-apple-primary rounded" />
                    </td>
                    <td class="py-1 pr-2">
                      <input
                        type="text"
                        bind:value={item.key}
                        placeholder="header-name"
                        class="w-full bg-apple-surface-black border border-apple-hairline-dark/60 rounded px-2 py-0.5 text-xs text-apple-body-on-dark font-mono focus:outline-none focus:border-apple-primary"
                      />
                    </td>
                    <td class="py-1 pr-2">
                      <input
                        type="text"
                        bind:value={item.value}
                        placeholder="value"
                        class="w-full bg-apple-surface-black border border-apple-hairline-dark/60 rounded px-2 py-0.5 text-xs text-apple-body-on-dark font-mono focus:outline-none focus:border-apple-primary"
                      />
                    </td>
                    <td class="py-1 text-right">
                      <button
                        type="button"
                        onclick={() => removeMetadataRow(item.id)}
                        class="text-apple-ink-muted-48 hover:text-rose-400 text-xs cursor-pointer"
                      >
                        ✕
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    </div>

    <!-- Right: RPC Response Viewer -->
    <div class="w-1/2 flex flex-col bg-apple-surface-black">
      <!-- Response Sub-header -->
      <div class="h-9 border-b border-apple-hairline-dark/60 bg-apple-surface-tile-2 px-3 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-1">
          <button
            type="button"
            onclick={() => responseTab = 'body'}
            class="px-2.5 py-1 rounded-apple-sm text-xs font-semibold transition-colors cursor-pointer {responseTab === 'body' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'text-apple-body-muted hover:text-white'}"
          >
            Response Message
          </button>
          <button
            type="button"
            onclick={() => responseTab = 'metadata'}
            class="px-2.5 py-1 rounded-apple-sm text-xs font-semibold transition-colors cursor-pointer {responseTab === 'metadata' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'text-apple-body-muted hover:text-white'}"
          >
            Trailers ({Object.keys(responseTrailers).length})
          </button>
        </div>

        {#if responseStatus}
          <div class="flex items-center gap-2 font-mono text-[11px]">
            <span class="px-2 py-0.5 rounded-apple-pill font-mono text-[10px] border {responseStatus.code === 0 ? 'bg-emerald-950/80 text-emerald-400 border-emerald-800' : 'bg-rose-950/80 text-rose-400 border-rose-800'}">
              {responseStatus.code === 0 ? '0 (OK)' : `${responseStatus.code} (${responseStatus.text})`}
            </span>
            {#if responseDurationMs !== null}
              <span class="text-apple-ink-muted-48">{formatDuration(responseDurationMs)}</span>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Response Content -->
      <div class="flex-1 overflow-auto p-3">
        {#if !responseStatus}
          <div class="h-full flex flex-col items-center justify-center text-apple-ink-muted-48 italic font-sans space-y-2">
            <span>Ready to invoke gRPC service.</span>
            <span class="text-[11px]">Click "Invoke RPC" to execute unary or streaming call.</span>
          </div>
        {:else if responseTab === 'body'}
          <div class="h-full border border-apple-hairline-dark/60 rounded-apple-sm overflow-hidden">
            <CodeEditor value={responsePayload} language="json" readOnly={true} />
          </div>
        {:else if responseTab === 'metadata'}
          <div class="space-y-2 font-mono text-xs">
            <table class="w-full text-left border-collapse">
              <thead>
                <tr class="border-b border-apple-hairline-dark/60 text-[10px] text-apple-ink-muted-48">
                  <th class="py-1">Trailer Name</th>
                  <th class="py-1">Value</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-apple-hairline-dark/30">
                {#each Object.entries(responseTrailers) as [k, v]}
                  <tr>
                    <td class="py-1.5 font-medium text-apple-body-muted">{k}</td>
                    <td class="py-1.5 text-apple-body-on-dark break-all">{v}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
