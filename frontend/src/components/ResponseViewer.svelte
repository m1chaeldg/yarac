<script lang="ts">
  interface ResponseData {
    status: number;
    statusText: string;
    durationMs: number;
    sizeBytes: number;
    headers: Record<string, string>;
    body: string;
    timestamp?: string;
  }

  interface Props {
    response?: ResponseData;
  }

  let { response }: Props = $props();

  let activeView = $state<'json' | 'table' | 'visualizer' | 'html' | 'headers' | 'raw'>('json');
  let chartType = $state<'pie' | 'bar'>('pie');
  let tableSearch = $state('');
  let copied = $state(false);

  let parsedJson = $derived.by(() => {
    if (!response?.body) return null;
    try {
      return JSON.parse(response.body);
    } catch {
      return null;
    }
  });

  // Flatten JSON helper for Table view
  interface FlatRow {
    [key: string]: string | number | boolean;
  }

  function flattenObject(obj: any, prefix = ''): FlatRow {
    const result: FlatRow = {};
    for (const key of Object.keys(obj)) {
      const val = obj[key];
      const newKey = prefix ? `${prefix}.${key}` : key;
      if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
        Object.assign(result, flattenObject(val, newKey));
      } else if (Array.isArray(val)) {
        if (val.length > 0 && typeof val[0] === 'object') {
          result[newKey] = `[${val.length} objects]`;
        } else {
          result[newKey] = JSON.stringify(val);
        }
      } else {
        result[newKey] = val;
      }
    }
    return result;
  }

  let tableData = $derived.by(() => {
    if (!parsedJson) return { columns: [], rows: [] };

    let rows: FlatRow[] = [];
    if (Array.isArray(parsedJson)) {
      rows = parsedJson.map((item, idx) => {
        if (typeof item === 'object' && item !== null) {
          return { '#': idx + 1, ...flattenObject(item) };
        }
        return { '#': idx + 1, value: item };
      });
    } else if (typeof parsedJson === 'object' && parsedJson !== null) {
      // If there's an array property inside (like users, items, teams, etc.), prioritize listing that array
      const arrayKey = Object.keys(parsedJson).find(k => Array.isArray(parsedJson[k]) && parsedJson[k].length > 0 && typeof parsedJson[k][0] === 'object');
      if (arrayKey) {
        rows = parsedJson[arrayKey].map((item: any, idx: number) => ({
          '#': idx + 1,
          ...flattenObject(item)
        }));
      } else {
        // Flatten single object into Key-Value rows
        const flattened = flattenObject(parsedJson);
        rows = Object.keys(flattened).map((key, idx) => ({
          '#': idx + 1,
          property: key,
          value: String(flattened[key])
        }));
      }
    }

    const columnSet = new Set<string>();
    rows.forEach(r => Object.keys(r).forEach(k => columnSet.add(k)));
    const columns = Array.from(columnSet);

    // Apply search filter
    if (tableSearch.trim()) {
      const query = tableSearch.toLowerCase();
      rows = rows.filter(r => Object.values(r).some(v => String(v).toLowerCase().includes(query)));
    }

    return { columns, rows };
  });

  // Extract Numerical Data for Visualizer (Pie & Bar charts)
  interface ChartDataPoint {
    label: string;
    value: number;
    color: string;
  }

  const chartColors = [
    '#38bdf8', // sky-400
    '#34d399', // emerald-400
    '#fbbf24', // amber-400
    '#a78bfa', // purple-400
    '#f472b6', // pink-400
    '#fb7185', // rose-400
    '#2dd4bf', // teal-400
    '#60a5fa'  // blue-400
  ];

  let chartPoints = $derived.by((): ChartDataPoint[] => {
    if (!parsedJson) return [];

    const points: ChartDataPoint[] = [];

    // Check if JSON has array of objects with label and value
    if (Array.isArray(parsedJson)) {
      parsedJson.slice(0, 10).forEach((item, idx) => {
        if (typeof item === 'object' && item !== null) {
          const numKey = Object.keys(item).find(k => typeof item[k] === 'number');
          const strKey = Object.keys(item).find(k => typeof item[k] === 'string') || 'item_' + idx;
          if (numKey) {
            points.push({
              label: String(item[strKey] || `#${idx + 1}`),
              value: Number(item[numKey]),
              color: chartColors[idx % chartColors.length]
            });
          }
        }
      });
    } else if (typeof parsedJson === 'object' && parsedJson !== null) {
      // Find sub-array with numbers, or numeric key-values
      const arrayKey = Object.keys(parsedJson).find(k => Array.isArray(parsedJson[k]) && parsedJson[k].length > 0);
      if (arrayKey && Array.isArray(parsedJson[arrayKey])) {
        parsedJson[arrayKey].slice(0, 8).forEach((item: any, idx: number) => {
          if (typeof item === 'object' && item !== null) {
            const numKey = Object.keys(item).find(k => typeof item[k] === 'number');
            const strKey = Object.keys(item).find(k => typeof item[k] === 'string') || 'name';
            if (numKey) {
              points.push({
                label: String(item[strKey] || `#${idx + 1}`),
                value: Number(item[numKey]),
                color: chartColors[idx % chartColors.length]
              });
            }
          }
        });
      }

      if (points.length === 0) {
        // Flatten numeric fields from stats or root object
        let idx = 0;
        const traverse = (obj: any, prefix = '') => {
          for (const k of Object.keys(obj)) {
            const val = obj[k];
            if (typeof val === 'number') {
              points.push({
                label: prefix ? `${prefix}.${k}` : k,
                value: val,
                color: chartColors[idx % chartColors.length]
              });
              idx++;
            } else if (typeof val === 'object' && val !== null && !Array.isArray(val) && idx < 8) {
              traverse(val, k);
            }
          }
        };
        traverse(parsedJson);
      }
    }

    return points.slice(0, 8);
  });

  // Calculate SVG Pie Chart paths
  let piePaths = $derived.by(() => {
    const total = chartPoints.reduce((acc, p) => acc + Math.max(0, p.value), 0);
    if (total === 0) return [];

    let currentAngle = 0;
    return chartPoints.map(p => {
      const sliceAngle = (Math.max(0, p.value) / total) * 2 * Math.PI;
      const startAngle = currentAngle;
      const endAngle = currentAngle + sliceAngle;
      currentAngle = endAngle;

      // Center is (100, 100), radius is 80
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

  // Calculate SVG Bar Chart coordinates
  let barData = $derived.by(() => {
    const maxVal = Math.max(...chartPoints.map(p => p.value), 1);
    return chartPoints.map(p => ({
      ...p,
      heightPct: Math.round((p.value / maxVal) * 100)
    }));
  });

  function copyPayload() {
    if (!response?.body) return;
    navigator.clipboard.writeText(response.body);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }

  function getStatusBadge(status: number) {
    if (status >= 200 && status < 300) return 'bg-emerald-950 text-emerald-400 border-emerald-800';
    if (status >= 300 && status < 400) return 'bg-sky-950 text-sky-400 border-sky-800';
    if (status >= 400 && status < 500) return 'bg-amber-950 text-amber-400 border-amber-800';
    return 'bg-rose-950 text-rose-400 border-rose-800';
  }
</script>

<div class="h-full flex flex-col bg-slate-950 border border-slate-800 rounded-lg overflow-hidden text-xs">
  <!-- Top Response Summary Status Bar -->
  <div class="h-10 bg-slate-900 border-b border-slate-800 px-3 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-1.5">
        {#if response}
          <span class="px-2 py-0.5 rounded text-[11px] font-bold border font-mono {getStatusBadge(response.status)}">
            {response.status} {response.statusText}
          </span>
          <span class="text-slate-400 font-mono text-[11px] ml-1">
            ⏱ <span class="text-emerald-400 font-semibold">{response.durationMs}ms</span>
          </span>
          <span class="text-slate-500 font-mono text-[11px]">
            📦 {(response.sizeBytes / 1024).toFixed(1)} KB
          </span>
        {:else}
          <span class="text-slate-500 italic">No response received yet</span>
        {/if}
      </div>

      {#if response && response.sizeBytes > 2 * 1024 * 1024}
        <span class="px-1.5 py-0.5 rounded text-[10px] bg-purple-950 text-purple-300 border border-purple-800 font-mono">
          ⚡ Virtualized Stream (>2MB)
        </span>
      {/if}
    </div>

    <!-- View Switcher -->
    <div class="flex items-center gap-1">
      <button
        type="button"
        onclick={() => activeView = 'json'}
        class="px-2.5 py-1 rounded text-xs font-medium transition-colors {activeView === 'json' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        JSON
      </button>

      <button
        type="button"
        onclick={() => activeView = 'table'}
        class="px-2.5 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1 {activeView === 'table' ? 'bg-slate-800 text-emerald-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        <span>Table</span>
        {#if tableData.rows.length > 0}
          <span class="text-[10px] px-1 rounded-full bg-emerald-950 text-emerald-400 border border-emerald-800">
            {tableData.rows.length}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => activeView = 'visualizer'}
        class="px-2.5 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1 {activeView === 'visualizer' ? 'bg-slate-800 text-amber-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        <span>Visualizer</span>
        <span class="text-[10px] px-1 rounded bg-amber-950 text-amber-400 border border-amber-800 font-mono">Chart</span>
      </button>

      <button
        type="button"
        onclick={() => activeView = 'html'}
        class="px-2.5 py-1 rounded text-xs font-medium transition-colors {activeView === 'html' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        HTML
      </button>

      <button
        type="button"
        onclick={() => activeView = 'headers'}
        class="px-2.5 py-1 rounded text-xs font-medium transition-colors {activeView === 'headers' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        Headers
      </button>

      <button
        type="button"
        onclick={() => activeView = 'raw'}
        class="px-2.5 py-1 rounded text-xs font-medium transition-colors {activeView === 'raw' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        Raw
      </button>

      <!-- Copy Button -->
      {#if response?.body}
        <button
          type="button"
          onclick={copyPayload}
          class="ml-2 px-2 py-0.5 rounded border border-slate-700 bg-slate-800/80 hover:bg-slate-700 text-slate-300 text-[11px] font-mono transition-colors"
        >
          {copied ? '✓ Copied' : 'Copy'}
        </button>
      {/if}
    </div>
  </div>

  <!-- Response Viewport Content -->
  <div class="flex-1 overflow-auto p-3">
    {#if !response}
      <div class="h-full flex flex-col items-center justify-center text-slate-500 space-y-2">
        <span class="text-3xl">⚡</span>
        <p class="font-medium text-xs">Ready to dispatch request.</p>
        <p class="text-[11px] text-slate-600">Hit "Send" to inspect the live response.</p>
      </div>

    <!-- JSON VIEW -->
    {:else if activeView === 'json'}
      <div class="font-mono text-slate-200 text-xs">
        <pre class="whitespace-pre-wrap break-all leading-relaxed">{response.body}</pre>
      </div>

    <!-- FLATTENED TABLE VIEW -->
    {:else if activeView === 'table'}
      <div class="h-full flex flex-col space-y-3 font-sans">
        <div class="flex items-center justify-between shrink-0">
          <div class="flex items-center gap-2">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
              Flattened Data Table ({tableData.rows.length} rows)
            </span>
          </div>
          <input
            type="text"
            bind:value={tableSearch}
            placeholder="Search columns or rows..."
            class="bg-slate-900 border border-slate-800 text-slate-200 text-xs rounded px-2.5 py-1 focus:outline-none focus:border-sky-500 w-64"
          />
        </div>

        {#if tableData.rows.length > 0}
          <div class="flex-1 overflow-auto border border-slate-800 rounded-lg">
            <table class="w-full border-collapse text-left font-mono text-[11px]">
              <thead class="sticky top-0 bg-slate-900 border-b border-slate-800 z-10">
                <tr>
                  {#each tableData.columns as col}
                    <th class="p-2 border-r border-slate-800 text-slate-300 font-semibold whitespace-nowrap">
                      {col}
                    </th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each tableData.rows as row, rIdx}
                  <tr class="border-b border-slate-800/80 hover:bg-slate-900/40">
                    {#each tableData.columns as col}
                      <td class="p-2 border-r border-slate-800 text-slate-300 max-w-xs truncate" title={String(row[col] ?? '')}>
                        {#if typeof row[col] === 'boolean'}
                          <span class="font-bold {row[col] ? 'text-emerald-400' : 'text-rose-400'}">{String(row[col])}</span>
                        {:else if typeof row[col] === 'number'}
                          <span class="text-sky-400 font-semibold">{row[col]}</span>
                        {:else}
                          {row[col] ?? ''}
                        {/if}
                      </td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="p-8 text-center text-slate-500">
            No tabular data could be extracted from this response.
          </div>
        {/if}
      </div>

    <!-- VISUALIZER / CHARTS VIEW -->
    {:else if activeView === 'visualizer'}
      <div class="h-full flex flex-col space-y-4 font-sans">
        <div class="flex items-center justify-between shrink-0">
          <div class="flex items-center gap-2">
            <span class="text-xs font-semibold text-slate-300">Response Data Visualizer</span>
            <span class="text-[11px] text-slate-500">Auto-extracted metrics from payload</span>
          </div>

          <!-- Chart Type Toggle -->
          <div class="flex items-center gap-1 bg-slate-900 p-0.5 rounded border border-slate-800">
            <button
              type="button"
              onclick={() => chartType = 'pie'}
              class="px-2.5 py-1 rounded text-xs font-medium transition-colors {chartType === 'pie' ? 'bg-sky-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
            >
              🥧 Pie Chart
            </button>
            <button
              type="button"
              onclick={() => chartType = 'bar'}
              class="px-2.5 py-1 rounded text-xs font-medium transition-colors {chartType === 'bar' ? 'bg-sky-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
            >
              📊 Bar Chart
            </button>
          </div>
        </div>

        {#if chartPoints.length > 0}
          <div class="flex-1 flex flex-col md:flex-row items-center justify-around gap-6 p-4 border border-slate-800 rounded-lg bg-slate-900/30">
            {#if chartType === 'pie'}
              <!-- SVG PIE CHART -->
              <div class="relative w-56 h-56 shrink-0">
                <svg viewBox="0 0 200 200" class="w-full h-full transform -rotate-90">
                  {#each piePaths as slice}
                    <path
                      d={slice.pathData}
                      fill={slice.color}
                      stroke="#020617"
                      stroke-width="2"
                      class="transition-all hover:opacity-80 cursor-pointer"
                    >
                      <title>{slice.label}: {slice.value} ({slice.percentage}%)</title>
                    </path>
                  {/each}
                </svg>
              </div>

            {:else}
              <!-- SVG BAR CHART -->
              <div class="w-full max-w-md h-56 flex items-end gap-3 px-4 pb-4 border-b border-l border-slate-700">
                {#each barData as bar}
                  <div class="flex-1 flex flex-col items-center gap-1.5 h-full justify-end group">
                    <span class="text-[10px] text-slate-400 font-mono opacity-0 group-hover:opacity-100 transition-opacity">
                      {bar.value}
                    </span>
                    <div
                      style="height: {bar.heightPct}%; background-color: {bar.color};"
                      class="w-full rounded-t transition-all group-hover:opacity-80"
                    ></div>
                    <span class="text-[10px] text-slate-400 truncate w-full text-center" title={bar.label}>
                      {bar.label.split('.').pop()}
                    </span>
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Legend & Stats -->
            <div class="space-y-2 max-w-xs w-full">
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block mb-2">Metrics Breakdown</span>
              <div class="space-y-1.5 max-h-48 overflow-y-auto">
                {#each chartPoints as point, idx}
                  <div class="flex items-center justify-between text-xs p-1.5 rounded bg-slate-900/60 border border-slate-800/80">
                    <div class="flex items-center gap-2 truncate">
                      <span class="w-2.5 h-2.5 rounded-full shrink-0" style="background-color: {point.color}"></span>
                      <span class="text-slate-300 truncate" title={point.label}>{point.label}</span>
                    </div>
                    <span class="font-mono font-bold text-slate-100 ml-2">{point.value}</span>
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {:else}
          <div class="p-8 text-center text-slate-500">
            No numerical metrics detected in the JSON response to chart. Send a request returning counts, stats, or numeric arrays to visualize.
          </div>
        {/if}
      </div>

    <!-- HTML PREVIEW -->
    {:else if activeView === 'html'}
      <div class="h-full flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-white">
        <div class="bg-slate-900 text-slate-400 px-3 py-1 text-[11px] border-b border-slate-800 flex items-center justify-between">
          <span>Sandboxed HTML Render Viewport</span>
          <span class="text-[10px] text-emerald-400">Sandbox: allow-same-origin</span>
        </div>
        <iframe
          srcdoc={response.body}
          sandbox="allow-same-origin"
          title="HTML Response Preview"
          class="flex-1 w-full h-full border-none"
        ></iframe>
      </div>

    <!-- HEADERS VIEW -->
    {:else if activeView === 'headers'}
      <div class="space-y-2 font-mono">
        <table class="w-full border-collapse border border-slate-800 text-left text-xs">
          <thead>
            <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800">
              <th class="p-2 border-r border-slate-800">Header Name</th>
              <th class="p-2">Header Value</th>
            </tr>
          </thead>
          <tbody>
            {#each Object.entries(response.headers) as [key, value]}
              <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                <td class="p-2 border-r border-slate-800 text-sky-400 font-semibold">{key}</td>
                <td class="p-2 text-slate-200 select-all">{value}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

    <!-- RAW VIEW -->
    {:else if activeView === 'raw'}
      <div class="font-mono text-slate-200 text-xs">
        <pre class="whitespace-pre-wrap break-all">{response.body}</pre>
      </div>
    {/if}
  </div>
</div>
