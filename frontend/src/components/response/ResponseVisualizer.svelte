<script lang="ts">
  interface Props {
    parsedJson: any;
    bodyText?: string;
    mode?: 'charts' | 'html';
  }

  let { parsedJson, bodyText = '', mode = 'charts' }: Props = $props();

  let chartType = $state<'pie' | 'bar'>('pie');

  const chartColors = [
    '#38bdf8', '#818cf8', '#c084fc', '#f472b6',
    '#fb7185', '#fb923c', '#facc15', '#4ade80'
  ];

  let chartPoints = $derived.by(() => {
    if (!parsedJson) return [];
    let points: Array<{ label: string; value: number; color: string }> = [];

    if (Array.isArray(parsedJson)) {
      parsedJson.slice(0, 8).forEach((item, idx) => {
        if (typeof item === 'object' && item !== null) {
          let label = `Item ${idx + 1}`;
          let value = 0;
          for (const [k, v] of Object.entries(item)) {
            if (typeof v === 'string' && label.startsWith('Item')) {
              label = v;
            } else if (typeof v === 'number' && value === 0) {
              value = v;
            }
          }
          if (value > 0) {
            points.push({ label, value, color: chartColors[idx % chartColors.length] });
          }
        }
      });
    } else if (typeof parsedJson === 'object' && parsedJson !== null) {
      let idx = 0;
      for (const [k, v] of Object.entries(parsedJson)) {
        if (typeof v === 'number') {
          points.push({ label: k, value: v, color: chartColors[idx % chartColors.length] });
          idx++;
        }
      }
    }
    return points.slice(0, 8);
  });

  let piePaths = $derived.by(() => {
    const total = chartPoints.reduce((acc, p) => acc + Math.max(0, p.value), 0);
    if (total === 0) return [];

    let currentAngle = 0;
    return chartPoints.map(p => {
      const sliceAngle = (Math.max(0, p.value) / total) * 2 * Math.PI;
      const startAngle = currentAngle;
      const endAngle = currentAngle + sliceAngle;
      currentAngle = endAngle;

      const r = 80;
      const cx = 100;
      const cy = 100;
      const x1 = cx + r * Math.cos(startAngle);
      const y1 = cy + r * Math.sin(startAngle);
      const x2 = cx + r * Math.cos(endAngle);
      const y2 = cy + r * Math.sin(endAngle);

      const largeArcFlag = sliceAngle > Math.PI ? 1 : 0;
      const pathData = `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${largeArcFlag} 1 ${x2} ${y2} Z`;

      return {
        ...p,
        pathData,
        percentage: ((p.value / total) * 100).toFixed(1)
      };
    });
  });

  let barData = $derived.by(() => {
    const maxVal = Math.max(...chartPoints.map(p => p.value), 1);
    return chartPoints.map(p => ({
      ...p,
      heightPct: Math.round((p.value / maxVal) * 100)
    }));
  });
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  {#if mode === 'html'}
    <div class="h-full w-full bg-white">
      <iframe
        srcdoc={bodyText}
        title="HTML Preview"
        sandbox="allow-same-origin"
        class="w-full h-full border-0"
      ></iframe>
    </div>
  {:else}
    <!-- Chart Controls -->
    <div class="h-9 border-b border-apple-hairline-dark/60 px-4 bg-apple-surface-tile-2 flex items-center justify-between shrink-0">
      <span class="text-apple-body-muted font-medium font-sans">Data Visualization</span>
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          onclick={() => chartType = 'pie'}
          class="px-3 py-0.5 rounded-apple-pill text-xs font-medium transition-colors cursor-pointer {chartType === 'pie' ? 'apple-button-primary text-white font-semibold shadow-none' : 'apple-button-dark-utility bg-apple-surface-tile-3 text-apple-body-muted hover:text-white border border-apple-hairline-dark/60'}"
        >
          Pie Chart
        </button>
        <button
          type="button"
          onclick={() => chartType = 'bar'}
          class="px-3 py-0.5 rounded-apple-pill text-xs font-medium transition-colors cursor-pointer {chartType === 'bar' ? 'apple-button-primary text-white font-semibold shadow-none' : 'apple-button-dark-utility bg-apple-surface-tile-3 text-apple-body-muted hover:text-white border border-apple-hairline-dark/60'}"
        >
          Bar Chart
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-auto p-6 flex flex-col items-center justify-center">
      {#if chartPoints.length === 0}
        <div class="text-center text-apple-ink-muted-48 italic font-sans">
          No numeric data found in response body to visualize.
        </div>
      {:else if chartType === 'pie'}
        <div class="flex flex-col md:flex-row items-center gap-8">
          <svg viewBox="0 0 200 200" class="w-56 h-56 transform -rotate-90">
            {#each piePaths as slice}
              <path d={slice.pathData} fill={slice.color} class="hover:opacity-80 transition-opacity">
                <title>{slice.label}: {slice.value} ({slice.percentage}%)</title>
              </path>
            {/each}
          </svg>

          <div class="space-y-1.5 font-mono text-[11px]">
            {#each piePaths as slice}
              <div class="flex items-center gap-2">
                <span class="w-3 h-3 rounded-apple-xs shrink-0" style="background-color: {slice.color};"></span>
                <span class="text-apple-body-on-dark truncate max-w-xs">{slice.label}:</span>
                <span class="text-apple-body-muted font-semibold">{slice.value} ({slice.percentage}%)</span>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="w-full max-w-xl space-y-3 font-mono text-[11px]">
          {#each barData as bar}
            <div class="space-y-1">
              <div class="flex justify-between text-apple-body-on-dark">
                <span class="truncate max-w-md">{bar.label}</span>
                <span class="text-apple-body-muted font-semibold">{bar.value}</span>
              </div>
              <div class="h-4 bg-apple-surface-tile-2 rounded-apple-pill overflow-hidden border border-apple-hairline-dark/40">
                <div
                  class="h-full rounded-apple-pill transition-all duration-300"
                  style="width: {bar.heightPct}%; background-color: {bar.color};"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
