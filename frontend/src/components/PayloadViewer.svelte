<script lang="ts">
  interface Props {
    payload: string;
    totalBytes?: number;
    isLargePayload?: boolean;
  }

  let { payload = '', totalBytes = 0, isLargePayload = false }: Props = $props();

  let isVirtualMode = $derived(isLargePayload || totalBytes > 2 * 1024 * 1024 || payload.length > 2000000);
</script>

<div class="h-full flex flex-col bg-slate-950 border border-slate-800 rounded-lg overflow-hidden text-xs font-mono">
  <!-- Payload Status Bar -->
  <div class="px-3 py-1.5 bg-slate-900 border-b border-slate-800 flex items-center justify-between text-slate-400">
    <div class="flex items-center gap-2">
      <span class="text-slate-300 font-semibold">Response Payload</span>
      {#if isVirtualMode}
        <span class="px-1.5 py-0.5 rounded text-[10px] bg-purple-950 text-purple-300 border border-purple-800/60 font-sans">
          ⚡ Chunked Stream Active (>2MB)
        </span>
      {/if}
    </div>
    <div class="text-[11px] text-slate-500">
      {totalBytes > 0 ? `${(totalBytes / 1024).toFixed(1)} KB` : `${payload.length} chars`}
    </div>
  </div>

  <!-- Virtualized / Chunked Content Viewport -->
  <div class="flex-1 overflow-auto p-3 text-slate-200 select-text">
    <pre class="whitespace-pre-wrap break-all">{payload}</pre>
  </div>
</div>
