<script lang="ts">
  import type { TimelineEntry } from '../ResponseViewer.svelte';
  import { formatDuration, formatSize, getStatusBadgeColor } from '../../utils/statusUtils';
  import { computeWaterfallTiming, type NetworkTimingBreakdown } from '../../utils/responseMetrics';

  interface Props {
    timeline?: TimelineEntry[];
    currentDurationMs?: number;
    timingBreakdown?: NetworkTimingBreakdown;
  }

  let { timeline = [], currentDurationMs = 0, timingBreakdown }: Props = $props();

  let activeSubTab = $state<'waterfall' | 'history'>('waterfall');

  let activeDuration = $derived(
    currentDurationMs > 0
      ? currentDurationMs
      : (timeline.length > 0 ? timeline[timeline.length - 1].durationMs : 100)
  );

  let waterfall = $derived(
    computeWaterfallTiming(activeDuration, timingBreakdown)
  );

  let phases = $derived.by(() => {
    const total = waterfall.totalMs || 1;
    let currentOffset = 0;

    const list = [
      {
        name: 'DNS Lookup',
        description: 'Resolving domain name via DNS',
        duration: waterfall.dnsLookupMs,
        offset: currentOffset,
        color: 'bg-sky-500',
        textColor: 'text-sky-400',
        borderColor: 'border-sky-500/40'
      },
      {
        name: 'TCP Handshake',
        description: 'Establishing socket connection (SYN/ACK)',
        duration: waterfall.tcpHandshakeMs,
        offset: (currentOffset += waterfall.dnsLookupMs),
        color: 'bg-indigo-500',
        textColor: 'text-indigo-400',
        borderColor: 'border-indigo-500/40'
      },
      {
        name: 'TLS Handshake',
        description: 'SSL/TLS cipher negotiation & certificate check',
        duration: waterfall.tlsHandshakeMs,
        offset: (currentOffset += waterfall.tcpHandshakeMs),
        color: 'bg-amber-500',
        textColor: 'text-amber-400',
        borderColor: 'border-amber-500/40'
      },
      {
        name: 'Waiting (TTFB)',
        description: 'Time to First Byte - server processing request',
        duration: waterfall.ttfbMs,
        offset: (currentOffset += waterfall.tlsHandshakeMs),
        color: 'bg-emerald-500',
        textColor: 'text-emerald-400',
        borderColor: 'border-emerald-500/40'
      },
      {
        name: 'Content Download',
        description: 'Receiving HTTP response payload body',
        duration: waterfall.contentTransferMs,
        offset: (currentOffset += waterfall.ttfbMs),
        color: 'bg-purple-500',
        textColor: 'text-purple-400',
        borderColor: 'border-purple-500/40'
      }
    ];

    return list.map(p => ({
      ...p,
      offsetPct: (p.offset / total) * 100,
      widthPct: Math.max((p.duration / total) * 100, 1.5)
    }));
  });
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <!-- Sub-header Switcher -->
  <div class="h-9 border-b border-apple-hairline-dark/60 px-4 bg-apple-surface-tile-2 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={() => activeSubTab = 'waterfall'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-semibold transition-colors cursor-pointer {activeSubTab === 'waterfall' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'text-apple-body-muted hover:text-white'}"
      >
        <span>⚡ Waterfall Breakdown</span>
      </button>

      <button
        type="button"
        onclick={() => activeSubTab = 'history'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-semibold transition-colors cursor-pointer {activeSubTab === 'history' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'text-apple-body-muted hover:text-white'}"
      >
        <span>History ({timeline.length})</span>
      </button>
    </div>

    <div class="flex items-center gap-2 font-mono text-[11px] text-apple-ink-muted-48">
      <span>Total Latency:</span>
      <span class="text-emerald-400 font-bold">{formatDuration(waterfall.totalMs)}</span>
    </div>
  </div>

  <div class="flex-1 overflow-auto p-4">
    {#if activeSubTab === 'waterfall'}
      <!-- Waterfall Chart -->
      <div class="space-y-6 max-w-4xl mx-auto">
        <!-- Overview summary cards -->
        <div class="grid grid-cols-5 gap-2">
          {#each phases as phase}
            <div class="p-2.5 bg-apple-surface-tile-1 border border-apple-hairline-dark/60 rounded-apple-sm flex flex-col justify-between">
              <div class="flex items-center gap-1.5 mb-1">
                <span class="w-2 h-2 rounded-full {phase.color}"></span>
                <span class="text-[10px] text-apple-body-muted truncate">{phase.name}</span>
              </div>
              <div class="text-xs font-bold font-mono {phase.textColor}">
                {formatDuration(phase.duration)}
              </div>
            </div>
          {/each}
        </div>

        <!-- Combined Timeline Bar -->
        <div class="space-y-1.5">
          <div class="flex justify-between text-[11px] text-apple-ink-muted-48 font-mono">
            <span>0 ms</span>
            <span>{Math.round(waterfall.totalMs / 2)} ms</span>
            <span>{waterfall.totalMs} ms</span>
          </div>

          <div class="h-3 w-full bg-apple-surface-tile-3 rounded-apple-pill overflow-hidden flex border border-apple-hairline-dark/80">
            {#each phases as phase}
              <div
                class="h-full {phase.color} transition-all duration-300"
                style="width: {phase.widthPct}%"
                title="{phase.name}: {formatDuration(phase.duration)}"
              ></div>
            {/each}
          </div>
        </div>

        <!-- Detailed Waterfall Rows -->
        <div class="border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-tile-1">
          <div class="px-4 py-2 border-b border-apple-hairline-dark/60 bg-apple-surface-tile-2 flex items-center justify-between text-[11px] text-apple-body-muted font-semibold">
            <span>Phase</span>
            <span class="w-1/2 text-left pl-2">Timeline Range</span>
            <span class="text-right">Duration</span>
          </div>

          <div class="divide-y divide-apple-hairline-dark/30 font-mono text-[11px]">
            {#each phases as phase}
              <div class="px-4 py-2.5 flex items-center justify-between hover:bg-apple-surface-tile-2/40 transition-colors">
                <div class="flex items-center gap-2 w-1/3">
                  <span class="w-2.5 h-2.5 rounded-sm {phase.color} shrink-0"></span>
                  <div class="flex flex-col">
                    <span class="font-sans font-medium text-apple-body-on-dark">{phase.name}</span>
                    <span class="font-sans text-[10px] text-apple-ink-muted-48">{phase.description}</span>
                  </div>
                </div>

                <!-- Waterfall Offset Bar -->
                <div class="w-1/2 relative h-4 bg-apple-surface-black/60 rounded flex items-center px-1">
                  <div
                    class="absolute h-2.5 rounded-apple-xs {phase.color} shadow-sm"
                    style="left: {phase.offsetPct}%; width: {phase.widthPct}%;"
                  ></div>
                </div>

                <div class="text-right font-bold {phase.textColor} w-16">
                  {formatDuration(phase.duration)}
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {:else}
      <!-- Execution History Table -->
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
    {/if}
  </div>
</div>
