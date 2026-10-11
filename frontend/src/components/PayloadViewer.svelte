<script lang="ts">
  interface Props {
    payload: string;
    totalBytes?: number;
    isLargePayload?: boolean;
  }

  let { payload = '', totalBytes = 0, isLargePayload = false }: Props = $props();

  let isVirtualMode = $derived(isLargePayload || totalBytes > 2 * 1024 * 1024 || payload.length > 2000000);
</script>

<div class="h-full flex flex-col bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-md overflow-hidden text-xs font-mono">
  <!-- Payload Status Bar -->
  <div class="px-3 py-1.5 bg-apple-surface-tile-3 border-b border-apple-hairline-dark/60 flex items-center justify-between text-apple-secondary-on-dark">
    <div class="flex items-center gap-2">
      <span class="text-apple-primary-on-dark font-semibold">Response Payload</span>
      {#if isVirtualMode}
        <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-apple-surface-tile-2 text-apple-link-on-dark border border-apple-hairline-dark/80 font-sans">
          ⚡ Chunked Stream Active (>2MB)
        </span>
      {/if}
    </div>
    <div class="text-[11px] text-apple-secondary-on-dark">
      {totalBytes > 0 ? `${(totalBytes / 1024).toFixed(1)} KB` : `${payload.length} chars`}
    </div>
  </div>

  <!-- Virtualized / Chunked Content Viewport -->
  <div class="flex-1 overflow-auto p-3 text-apple-primary-on-dark select-text">
    <pre class="whitespace-pre-wrap break-all">{payload}</pre>
  </div>
</div>
