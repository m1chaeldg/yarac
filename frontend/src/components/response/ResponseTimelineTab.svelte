<script lang="ts">
  import type { TimelineEntry } from '../ResponseViewer.svelte';
  import { formatDuration, formatSize, getStatusBadgeColor } from '../../utils/statusUtils';

  interface Props {
    timeline?: TimelineEntry[];
  }

  let { timeline = [] }: Props = $props();
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <div class="h-9 border-b border-apple-hairline-dark/60 px-4 bg-apple-surface-tile-2 flex items-center justify-between shrink-0">
    <span class="text-apple-body-muted font-medium font-sans">Execution History Timeline</span>
    <span class="text-apple-ink-muted-48 font-mono text-[11px]">{timeline.length} requests</span>
  </div>

  <div class="flex-1 overflow-auto">
    {#if timeline.length === 0}
      <div class="p-8 text-center text-apple-ink-muted-48 italic font-sans">No execution history recorded yet.</div>
    {:else}
      <table class="w-full text-left border-collapse font-mono text-[11px]">
        <thead class="sticky top-0 bg-apple-surface-tile-2 border-b border-apple-hairline-dark/60 text-[10px] text-apple-body-muted font-sans font-semibold">
          <tr>
            <th class="px-3 py-2">Timestamp</th>
            <th class="px-3 py-2">Method</th>
            <th class="px-3 py-2">Status</th>
            <th class="px-3 py-2 text-right">Duration</th>
            <th class="px-3 py-2 text-right">Size</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-apple-hairline-dark/40">
          {#each timeline as entry, i}
            <tr class="hover:bg-apple-surface-tile-1/50 transition-colors {i % 2 === 0 ? 'bg-apple-surface-black' : 'bg-apple-surface-tile-1/30'}">
              <td class="px-3 py-1.5 text-apple-ink-muted-48">{new Date(entry.timestamp).toLocaleTimeString()}</td>
              <td class="px-3 py-1.5 font-bold text-apple-primary-on-dark">{entry.method || 'GET'}</td>
              <td class="px-3 py-1.5">
                <span class="px-2 py-0.5 rounded-apple-pill text-[10px] border font-mono {getStatusBadgeColor(entry.status)}">
                  {entry.status} {entry.statusText}
                </span>
              </td>
              <td class="px-3 py-1.5 text-right text-emerald-400">{formatDuration(entry.durationMs)}</td>
              <td class="px-3 py-1.5 text-right text-apple-body-muted">{formatSize(entry.sizeBytes)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
