<script lang="ts">
  import { filterAndSortHeaders } from '../../utils/responseUtils';

  interface Props {
    headers?: Record<string, string>;
  }

  let { headers }: Props = $props();

  let headersSearch = $state('');
  let copyFeedback = $state('');

  let filteredHeaders = $derived(filterAndSortHeaders(headers, headersSearch));

  function copyAllHeaders() {
    if (!headers) return;
    const text = Object.entries(headers)
      .map(([k, v]) => `${k}: ${v}`)
      .join('\n');
    navigator.clipboard.writeText(text);
    copyFeedback = 'Headers copied';
    setTimeout(() => (copyFeedback = ''), 2000);
  }
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <!-- Search & Actions Bar -->
  <div class="h-9 border-b border-apple-hairline-dark/60 px-3 bg-apple-surface-tile-2 flex items-center justify-between gap-3 shrink-0">
    <div class="flex items-center gap-2 flex-1 max-w-sm">
      <input
        type="text"
        bind:value={headersSearch}
        placeholder="Search headers..."
        class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-pill px-3 py-1 text-apple-body-on-dark text-xs focus:outline-none focus:border-apple-primary font-mono placeholder:text-apple-ink-muted-48"
      />
    </div>

    <div class="flex items-center gap-2">
      {#if copyFeedback}
        <span class="text-[11px] text-apple-primary-on-dark font-mono">✓ {copyFeedback}</span>
      {/if}
      <button
        type="button"
        onclick={copyAllHeaders}
        class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark border border-apple-hairline-dark/60 text-xs font-medium transition-colors cursor-pointer"
      >
        Copy All
      </button>
      <span class="text-apple-ink-muted-48 text-[11px] font-mono">
        {filteredHeaders.length} {filteredHeaders.length === 1 ? 'header' : 'headers'}
      </span>
    </div>
  </div>

  <!-- Headers Table -->
  <div class="flex-1 overflow-auto">
    {#if filteredHeaders.length === 0}
      <div class="p-8 text-center text-apple-ink-muted-48 italic">No headers found</div>
    {:else}
      <table class="w-full text-left border-collapse">
        <thead class="sticky top-0 bg-apple-surface-tile-2 border-b border-apple-hairline-dark/60 text-[11px] font-semibold text-apple-body-muted z-10">
          <tr>
            <th class="px-4 py-2 w-1/3">Header Name</th>
            <th class="px-4 py-2">Value</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-apple-hairline-dark/40 font-mono text-[11px]">
          {#each filteredHeaders as h, i}
            <tr class="hover:bg-apple-surface-tile-1/50 transition-colors {i % 2 === 0 ? 'bg-apple-surface-black' : 'bg-apple-surface-tile-1/30'}">
              <td class="px-4 py-2 text-apple-primary-on-dark font-medium select-all">{h.key}</td>
              <td class="px-4 py-2 text-apple-body-on-dark select-all break-all">{h.value}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
