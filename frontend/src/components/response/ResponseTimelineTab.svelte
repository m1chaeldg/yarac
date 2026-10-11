<script lang="ts">
  import type { TimelineEntry } from '../ResponseViewer.svelte';
  import { formatDuration, formatSize, getStatusBadgeColor } from '../../utils/statusUtils';

  interface Props {
    timeline?: TimelineEntry[];
  }

  let { timeline = [] }: Props = $props();
</script>

<div class="h-full flex flex-col bg-slate-950 overflow-hidden text-xs">
  <div class="h-9 border-b border-slate-800 px-4 bg-slate-900/60 flex items-center justify-between shrink-0">
    <span class="text-slate-400 font-medium">Execution History Timeline</span>
    <span class="text-slate-500 font-mono text-[11px]">{timeline.length} requests</span>
  </div>

  <div class="flex-1 overflow-auto">
    {#if timeline.length === 0}
      <div class="p-8 text-center text-slate-500 italic">No execution history recorded yet.</div>
    {:else}
      <table class="w-full text-left border-collapse font-mono text-[11px]">
        <thead class="sticky top-0 bg-slate-900 border-b border-slate-800 text-[10px] text-slate-400">
          <tr>
            <th class="px-3 py-2">Timestamp</th>
            <th class="px-3 py-2">Method</th>
            <th class="px-3 py-2">Status</th>
            <th class="px-3 py-2 text-right">Duration</th>
            <th class="px-3 py-2 text-right">Size</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#each timeline as entry, i}
            <tr class="hover:bg-slate-900/40 transition-colors {i % 2 === 0 ? 'bg-slate-950' : 'bg-slate-900/20'}">
              <td class="px-3 py-1.5 text-slate-500">{new Date(entry.timestamp).toLocaleTimeString()}</td>
              <td class="px-3 py-1.5 font-bold text-sky-400">{entry.method || 'GET'}</td>
              <td class="px-3 py-1.5">
                <span class="px-1.5 py-0.5 rounded text-[10px] border {getStatusBadgeColor(entry.status)}">
                  {entry.status} {entry.statusText}
                </span>
              </td>
              <td class="px-3 py-1.5 text-right text-emerald-400">{formatDuration(entry.durationMs)}</td>
              <td class="px-3 py-1.5 text-right text-slate-400">{formatSize(entry.sizeBytes)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
