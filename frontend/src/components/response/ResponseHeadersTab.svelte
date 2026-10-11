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

<div class="h-full flex flex-col bg-slate-950 overflow-hidden text-xs">
  <!-- Search & Actions Bar -->
  <div class="h-9 border-b border-slate-800 px-3 bg-slate-900/60 flex items-center justify-between gap-3 shrink-0">
    <div class="flex items-center gap-2 flex-1 max-w-sm">
      <input
        type="text"
        bind:value={headersSearch}
        placeholder="Search headers..."
        class="w-full bg-slate-950 border border-slate-800 rounded px-2 py-1 text-slate-200 text-xs focus:outline-none focus:border-sky-500 font-mono"
      />
    </div>

    <div class="flex items-center gap-2">
      {#if copyFeedback}
        <span class="text-[11px] text-emerald-400 font-mono">✓ {copyFeedback}</span>
      {/if}
      <button
        type="button"
        onclick={copyAllHeaders}
        class="px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors"
      >
        Copy All
      </button>
      <span class="text-slate-500 text-[11px] font-mono">
        {filteredHeaders.length} {filteredHeaders.length === 1 ? 'header' : 'headers'}
      </span>
    </div>
  </div>

  <!-- Headers Table -->
  <div class="flex-1 overflow-auto">
    {#if filteredHeaders.length === 0}
      <div class="p-8 text-center text-slate-500 italic">No headers found</div>
    {:else}
      <table class="w-full text-left border-collapse">
        <thead class="sticky top-0 bg-slate-900 border-b border-slate-800 text-[11px] font-semibold text-slate-400 z-10">
          <tr>
            <th class="px-4 py-2 w-1/3">Header Name</th>
            <th class="px-4 py-2">Value</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-mono text-[11px]">
          {#each filteredHeaders as h, i}
            <tr class="hover:bg-slate-900/40 transition-colors {i % 2 === 0 ? 'bg-slate-950' : 'bg-slate-900/20'}">
              <td class="px-4 py-2 text-sky-400 font-medium select-all">{h.key}</td>
              <td class="px-4 py-2 text-slate-300 select-all break-all">{h.value}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
