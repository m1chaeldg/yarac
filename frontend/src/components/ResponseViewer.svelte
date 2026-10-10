<script lang="ts">
  export interface AssertionResult {
    name: string;
    passed: boolean;
    error?: string;
    duration_ms?: number;
  }

  export interface ResponseData {
    status: number;
    statusText: string;
    durationMs: number;
    sizeBytes: number;
    headers: Record<string, string>;
    body: string;
    timestamp?: string;
    assertions?: AssertionResult[];
    logs?: string[];
  }

  export interface TimelineEntry {
    id: string;
    timestamp: string;
    status: number;
    statusText: string;
    durationMs: number;
    sizeBytes: number;
    method?: string;
    url?: string;
  }

  interface Props {
    response?: ResponseData;
    timeline?: TimelineEntry[];
    onClearResponse?: () => void;
    onToggleLayout?: () => void;
    layoutOrientation?: 'vertical' | 'horizontal';
  }

  let {
    response,
    timeline = [],
    onClearResponse,
    onToggleLayout,
    layoutOrientation = 'vertical'
  }: Props = $props();

  let activeView = $state<'json' | 'table' | 'visualizer' | 'html' | 'xml' | 'raw' | 'headers' | 'tests' | 'timeline'>('json');
  let chartType = $state<'pie' | 'bar'>('pie');
  let tableSearch = $state('');
  let columnFilter = $state<string>('');
  let sortColumn = $state<string | null>(null);
  let sortDir = $state<'asc' | 'desc' | null>(null);
  let headersSearch = $state('');
  let isToolsMenuOpen = $state(false);
  let copyFeedback = $state('');

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
      const arrayKey = Object.keys(parsedJson).find(k => Array.isArray(parsedJson[k]) && parsedJson[k].length > 0 && typeof parsedJson[k][0] === 'object');
      if (arrayKey) {
        rows = parsedJson[arrayKey].map((item: any, idx: number) => ({
          '#': idx + 1,
          ...flattenObject(item)
        }));
      } else {
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

    // Apply text search filter
    if (tableSearch.trim()) {
      const query = tableSearch.toLowerCase();
      rows = rows.filter(r => {
        if (columnFilter && r[columnFilter] !== undefined) {
          return String(r[columnFilter]).toLowerCase().includes(query);
        }
        return Object.values(r).some(v => String(v).toLowerCase().includes(query));
      });
    }

    // Apply column sorting
    if (sortColumn && sortDir) {
      const key = sortColumn;
      const factor = sortDir === 'asc' ? 1 : -1;
      rows = [...rows].sort((a, b) => {
        const valA = a[key];
        const valB = b[key];
        if (valA === undefined || valA === null) return 1 * factor;
        if (valB === undefined || valB === null) return -1 * factor;
        if (typeof valA === 'number' && typeof valB === 'number') {
          return (valA - valB) * factor;
        }
        return String(valA).localeCompare(String(valB)) * factor;
      });
    }

    return { columns, rows };
  });

  function handleSort(col: string) {
    if (sortColumn !== col) {
      sortColumn = col;
      sortDir = 'asc';
    } else if (sortDir === 'asc') {
      sortDir = 'desc';
    } else {
      sortColumn = null;
      sortDir = null;
    }
  }

  // XML Formatting helper
  let formattedXml = $derived.by(() => {
    if (!response?.body) return '';
    const text = response.body.trim();
    if (!text.startsWith('<')) return text;
    try {
      let formatted = '';
      const reg = /(>)(<)(\/*)/g;
      let xml = text.replace(reg, '$1\r\n$2$3');
      let pad = 0;
      xml.split('\r\n').forEach(node => {
        let indent = 0;
        if (node.match(/.+<\/\w[^>]*>$/)) {
          indent = 0;
        } else if (node.match(/^<\/\w/)) {
          if (pad !== 0) pad -= 1;
        } else if (node.match(/^<\w[^>]*[^\/]>.*$/)) {
          indent = 1;
        } else {
          indent = 0;
        }
        formatted += '  '.repeat(pad) + node + '\r\n';
        pad += indent;
      });
      return formatted.trim();
    } catch {
      return text;
    }
  });

  // Extract Numerical Data for Visualizer (Pie & Bar charts)
  interface ChartDataPoint {
    label: string;
    value: number;
    color: string;
  }

  const chartColors = [
    '#38bdf8', '#34d399', '#fbbf24', '#a78bfa',
    '#f472b6', '#fb7185', '#2dd4bf', '#60a5fa'
  ];

  let chartPoints = $derived.by((): ChartDataPoint[] => {
    if (!parsedJson) return [];
    const points: ChartDataPoint[] = [];

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

  // Filtered Headers
  let filteredHeaders = $derived.by(() => {
    if (!response?.headers) return [];
    const entries = Object.entries(response.headers);
    if (!headersSearch.trim()) return entries;
    const q = headersSearch.toLowerCase();
    return entries.filter(([k, v]) => k.toLowerCase().includes(q) || v.toLowerCase().includes(q));
  });

  // Test Results Metrics
  let testSummary = $derived.by(() => {
    const assertions = response?.assertions || [];
    const total = assertions.length;
    const passed = assertions.filter(a => a.passed).length;
    const failed = total - passed;
    return { total, passed, failed, passRate: total > 0 ? Math.round((passed / total) * 100) : 100 };
  });

  // Tool Actions
  function copyResponse() {
    if (!response?.body) return;
    navigator.clipboard.writeText(response.body);
    flashFeedback('Copied to clipboard');
    isToolsMenuOpen = false;
  }

  function downloadResponse(asJson = true) {
    if (!response?.body) return;
    const ext = asJson ? 'json' : 'txt';
    const blob = new Blob([response.body], { type: asJson ? 'application/json' : 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `response-${Date.now()}.${ext}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    flashFeedback('Response downloaded');
    isToolsMenuOpen = false;
  }

  function clearCurrentResponse() {
    if (onClearResponse) {
      onClearResponse();
    }
    isToolsMenuOpen = false;
  }

  function flashFeedback(msg: string) {
    copyFeedback = msg;
    setTimeout(() => {
      copyFeedback = '';
    }, 2000);
  }

  function getStatusBadge(status: number) {
    if (status >= 200 && status < 300) return 'bg-emerald-950 text-emerald-400 border-emerald-800';
    if (status >= 300 && status < 400) return 'bg-sky-950 text-sky-400 border-sky-800';
    if (status >= 400 && status < 500) return 'bg-amber-950 text-amber-400 border-amber-800';
    return 'bg-rose-950 text-rose-400 border-rose-800';
  }
</script>

<div class="h-full flex flex-col bg-slate-950 border border-slate-800 rounded-lg overflow-hidden text-xs">
  <!-- Top Response Summary Status Bar & Tab Switcher -->
  <div class="h-10 bg-slate-900 border-b border-slate-800 px-3 flex items-center justify-between shrink-0 gap-2">
    <div class="flex items-center gap-3 shrink-0">
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

      {#if copyFeedback}
        <span class="px-2 py-0.5 rounded text-[10px] bg-sky-950 text-sky-300 border border-sky-800 font-mono animate-fade-in">
          ✓ {copyFeedback}
        </span>
      {/if}
    </div>

    <!-- View Switcher Tabs -->
    <div class="flex items-center gap-1 overflow-x-auto">
      <button
        type="button"
        onclick={() => activeView = 'json'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors {activeView === 'json' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        JSON
      </button>

      <button
        type="button"
        onclick={() => activeView = 'table'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1 {activeView === 'table' ? 'bg-slate-800 text-emerald-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
        class="px-2 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1 {activeView === 'visualizer' ? 'bg-slate-800 text-amber-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        <span>Charts</span>
      </button>

      <button
        type="button"
        onclick={() => activeView = 'html'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors {activeView === 'html' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        HTML
      </button>

      <button
        type="button"
        onclick={() => activeView = 'xml'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors {activeView === 'xml' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        XML
      </button>

      <button
        type="button"
        onclick={() => activeView = 'raw'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors {activeView === 'raw' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        Raw
      </button>

      <button
        type="button"
        onclick={() => activeView = 'headers'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1 {activeView === 'headers' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        <span>Headers</span>
        {#if response?.headers}
          <span class="text-[10px] text-slate-500 font-mono">({Object.keys(response.headers).length})</span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => activeView = 'tests'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1 {activeView === 'tests' ? 'bg-slate-800 text-pink-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        <span>Test Results</span>
        {#if response?.assertions && response.assertions.length > 0}
          <span class="text-[10px] px-1 rounded-full {testSummary.failed > 0 ? 'bg-rose-950 text-rose-400 border border-rose-800' : 'bg-emerald-950 text-emerald-400 border border-emerald-800'}">
            {testSummary.passed}/{testSummary.total}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => activeView = 'timeline'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1 {activeView === 'timeline' ? 'bg-slate-800 text-purple-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
      >
        <span>Timeline</span>
        {#if timeline.length > 0}
          <span class="text-[10px] px-1 rounded-full bg-purple-950 text-purple-400 border border-purple-800">
            {timeline.length}
          </span>
        {/if}
      </button>

      <!-- Tools Ellipsis Dropdown (...) -->
      <div class="relative ml-1">
        <button
          type="button"
          onclick={() => isToolsMenuOpen = !isToolsMenuOpen}
          title="Response Tools"
          class="px-2 py-1 rounded border border-slate-700 bg-slate-800/80 hover:bg-slate-700 text-slate-200 text-xs font-bold transition-colors"
        >
          •••
        </button>

        {#if isToolsMenuOpen}
          <div
            role="dialog"
            tabindex="-1"
            class="fixed inset-0 z-40"
            onclick={() => isToolsMenuOpen = false}
            onkeydown={(e) => { if (e.key === 'Escape') isToolsMenuOpen = false; }}
          ></div>
          <div class="absolute right-0 top-full mt-1 w-48 bg-slate-900 border border-slate-700 rounded-lg shadow-xl z-50 py-1 text-slate-200 text-xs font-sans">
            <button
              type="button"
              onclick={copyResponse}
              class="w-full text-left px-3 py-1.5 hover:bg-slate-800 flex items-center gap-2"
            >
              <span>📋</span> Copy Response Body
            </button>
            <button
              type="button"
              onclick={() => downloadResponse(true)}
              class="w-full text-left px-3 py-1.5 hover:bg-slate-800 flex items-center gap-2"
            >
              <span>💾</span> Save as JSON
            </button>
            <button
              type="button"
              onclick={() => downloadResponse(false)}
              class="w-full text-left px-3 py-1.5 hover:bg-slate-800 flex items-center gap-2"
            >
              <span>⬇</span> Download Raw (.txt)
            </button>
            <div class="border-t border-slate-800 my-1"></div>
            {#if onToggleLayout}
              <button
                type="button"
                onclick={() => {
                  onToggleLayout();
                  isToolsMenuOpen = false;
                }}
                class="w-full text-left px-3 py-1.5 hover:bg-slate-800 flex items-center gap-2"
              >
                <span>⮂</span> Switch to {layoutOrientation === 'vertical' ? 'Side-by-Side (Horizontal)' : 'Stacked (Vertical)'}
              </button>
            {/if}
            <button
              type="button"
              onclick={clearCurrentResponse}
              class="w-full text-left px-3 py-1.5 hover:bg-rose-950/60 text-rose-400 flex items-center gap-2"
            >
              <span>🗑</span> Clear Response
            </button>
          </div>
        {/if}
      </div>
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

    <!-- FLATTENED TABLE VIEW WITH FILTER & SORTING -->
    {:else if activeView === 'table'}
      <div class="h-full flex flex-col space-y-3 font-sans">
        <div class="flex items-center justify-between shrink-0 gap-2 flex-wrap">
          <div class="flex items-center gap-2">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
              Flattened Data Table ({tableData.rows.length} rows)
            </span>
            {#if sortColumn}
              <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-950 text-sky-400 border border-sky-800 font-mono">
                Sorted by {sortColumn} ({sortDir})
              </span>
            {/if}
          </div>

          <div class="flex items-center gap-2">
            <!-- Column filter selector -->
            {#if tableData.columns.length > 0}
              <select
                bind:value={columnFilter}
                class="bg-slate-900 border border-slate-800 text-slate-300 text-xs rounded px-2 py-1 focus:outline-none"
              >
                <option value="">All Columns</option>
                {#each tableData.columns as col}
                  <option value={col}>{col}</option>
                {/each}
              </select>
            {/if}

            <input
              type="text"
              bind:value={tableSearch}
              placeholder="Search data rows..."
              class="bg-slate-900 border border-slate-800 text-slate-200 text-xs rounded px-2.5 py-1 focus:outline-none focus:border-sky-500 w-56"
            />
          </div>
        </div>

        {#if tableData.rows.length > 0}
          <div class="flex-1 overflow-auto border border-slate-800 rounded-lg">
            <table class="w-full border-collapse text-left font-mono text-[11px]">
              <thead class="sticky top-0 bg-slate-900 border-b border-slate-800 z-10 select-none">
                <tr>
                  {#each tableData.columns as col}
                    <th
                      onclick={() => handleSort(col)}
                      class="p-2 border-r border-slate-800 text-slate-300 font-semibold whitespace-nowrap cursor-pointer hover:bg-slate-800/80 transition-colors"
                    >
                      <div class="flex items-center justify-between gap-2">
                        <span>{col}</span>
                        {#if sortColumn === col}
                          <span class="text-sky-400 font-bold">{sortDir === 'asc' ? '▲' : '▼'}</span>
                        {:else}
                          <span class="text-slate-600 text-[10px]">↕</span>
                        {/if}
                      </div>
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
            No tabular data matched your search query.
          </div>
        {/if}
      </div>

    <!-- CHARTS / VISUALIZER VIEW -->
    {:else if activeView === 'visualizer'}
      <div class="h-full flex flex-col space-y-4 font-sans">
        <div class="flex items-center justify-between shrink-0">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
            Response Data Visualization ({chartPoints.length} Metrics Found)
          </span>

          <div class="flex items-center gap-1 bg-slate-900 border border-slate-800 p-0.5 rounded-lg">
            <button
              type="button"
              onclick={() => chartType = 'pie'}
              class="px-2.5 py-1 rounded text-xs font-semibold transition-colors {chartType === 'pie' ? 'bg-amber-600/30 text-amber-300 border border-amber-500/40' : 'text-slate-400 hover:text-slate-200'}"
            >
              🥧 Pie Chart
            </button>
            <button
              type="button"
              onclick={() => chartType = 'bar'}
              class="px-2.5 py-1 rounded text-xs font-semibold transition-colors {chartType === 'bar' ? 'bg-amber-600/30 text-amber-300 border border-amber-500/40' : 'text-slate-400 hover:text-slate-200'}"
            >
              📊 Bar Chart
            </button>
          </div>
        </div>

        {#if chartPoints.length > 0}
          <div class="flex-1 flex flex-col md:flex-row items-center justify-around p-4 border border-slate-800 rounded-lg bg-slate-900/20 overflow-auto">
            {#if chartType === 'pie'}
              <div class="w-64 h-64 shrink-0 relative flex items-center justify-center">
                <svg viewBox="0 0 200 200" class="w-full h-full transform -rotate-90 drop-shadow-md">
                  {#each piePaths as slice}
                    <path
                      d={slice.pathData}
                      fill={slice.color}
                      class="transition-all hover:opacity-85 cursor-pointer"
                    >
                      <title>{slice.label}: {slice.value} ({slice.percentage}%)</title>
                    </path>
                  {/each}
                </svg>
              </div>
            {:else}
              <div class="w-full max-w-lg h-64 flex items-end justify-around gap-3 pt-6 pb-2 border-b border-slate-800">
                {#each barData as b}
                  <div class="flex-1 flex flex-col items-center gap-1.5 h-full justify-end group">
                    <span class="text-[10px] font-mono text-slate-400 opacity-0 group-hover:opacity-100 transition-opacity">
                      {b.value}
                    </span>
                    <div
                      style="height: {Math.max(8, b.heightPct)}%; background-color: {b.color};"
                      class="w-full rounded-t transition-all group-hover:brightness-110"
                      title="{b.label}: {b.value}"
                    ></div>
                    <span class="text-[9px] text-slate-400 font-mono truncate max-w-[60px] text-center" title={b.label}>
                      {b.label}
                    </span>
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Legend List -->
            <div class="space-y-2 mt-4 md:mt-0 font-mono text-xs max-w-xs w-full">
              {#each chartPoints as pt, idx}
                <div class="flex items-center justify-between p-1.5 rounded bg-slate-900/60 border border-slate-800/80">
                  <div class="flex items-center gap-2 truncate">
                    <span class="w-2.5 h-2.5 rounded-full shrink-0" style="background-color: {pt.color};"></span>
                    <span class="text-slate-300 truncate" title={pt.label}>{pt.label}</span>
                  </div>
                  <span class="font-bold text-sky-400 shrink-0 ml-2">{pt.value}</span>
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="p-8 text-center text-slate-500">
            No numeric fields found in response body to generate charts.
          </div>
        {/if}
      </div>

    <!-- HTML VIEW -->
    {:else if activeView === 'html'}
      <div class="h-full flex flex-col space-y-3 font-sans">
        <div class="flex items-center justify-between shrink-0">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">HTML Rendered Preview</span>
        </div>
        <div class="flex-1 border border-slate-800 rounded-lg overflow-hidden bg-white">
          <iframe
            srcdoc={response.body}
            title="HTML Response View"
            class="w-full h-full border-0"
            sandbox="allow-same-origin"
          ></iframe>
        </div>
      </div>

    <!-- XML VIEW -->
    {:else if activeView === 'xml'}
      <div class="h-full flex flex-col space-y-2 font-mono text-xs">
        <div class="flex items-center justify-between shrink-0">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider font-sans">XML Formatted Document</span>
        </div>
        <div class="flex-1 overflow-auto border border-slate-800 rounded-lg bg-slate-950 p-3 text-emerald-300">
          <pre class="whitespace-pre-wrap break-all leading-relaxed">{formattedXml}</pre>
        </div>
      </div>

    <!-- RAW TEXT VIEW -->
    {:else if activeView === 'raw'}
      <div class="font-mono text-slate-200 text-xs">
        <pre class="whitespace-pre-wrap break-all leading-relaxed">{response.body}</pre>
      </div>

    <!-- HEADERS VIEW -->
    {:else if activeView === 'headers'}
      <div class="h-full flex flex-col space-y-3 font-sans">
        <div class="flex items-center justify-between shrink-0">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
            Response Headers ({filteredHeaders.length})
          </span>
          <input
            type="text"
            bind:value={headersSearch}
            placeholder="Search headers..."
            class="bg-slate-900 border border-slate-800 text-slate-200 text-xs rounded px-2.5 py-1 focus:outline-none focus:border-sky-500 w-56"
          />
        </div>

        <div class="flex-1 overflow-auto border border-slate-800 rounded-lg">
          <table class="w-full border-collapse text-left font-mono text-xs">
            <thead class="bg-slate-900 border-b border-slate-800">
              <tr>
                <th class="p-2 border-r border-slate-800 text-slate-400 w-1/3">Header Name</th>
                <th class="p-2 text-slate-400">Header Value</th>
              </tr>
            </thead>
            <tbody>
              {#each filteredHeaders as [key, value]}
                <tr class="border-b border-slate-800/80 hover:bg-slate-900/40">
                  <td class="p-2 border-r border-slate-800 font-semibold text-sky-400">{key}</td>
                  <td class="p-2 text-slate-300 break-all select-all">{value}</td>
                </tr>
              {/each}
              {#if filteredHeaders.length === 0}
                <tr>
                  <td colspan="2" class="p-6 text-center text-slate-500">
                    No headers matched your search.
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>
      </div>

    <!-- TEST RESULTS VIEW -->
    {:else if activeView === 'tests'}
      <div class="h-full flex flex-col space-y-3 font-sans">
        <div class="flex items-center justify-between shrink-0 p-3 bg-slate-900/70 border border-slate-800 rounded-lg">
          <div class="flex items-center gap-3">
            <span class="text-xs font-bold text-slate-200">Test Execution Summary</span>
            <span class="px-2 py-0.5 rounded text-xs font-mono font-bold {testSummary.failed === 0 ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' : 'bg-rose-950 text-rose-400 border border-rose-800'}">
              {testSummary.passed} / {testSummary.total} Passed ({testSummary.passRate}%)
            </span>
          </div>
          {#if testSummary.failed > 0}
            <span class="text-rose-400 text-xs font-semibold">
              ⚠️ {testSummary.failed} assertion failure(s)
            </span>
          {/if}
        </div>

        <!-- Assertion List -->
        <div class="flex-1 overflow-auto space-y-2">
          {#if response?.assertions && response.assertions.length > 0}
            {#each response.assertions as ast}
              <div class="p-3 rounded-lg border {ast.passed ? 'bg-emerald-950/20 border-emerald-900/50' : 'bg-rose-950/20 border-rose-900/50'} flex flex-col gap-1.5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold {ast.passed ? 'text-emerald-400' : 'text-rose-400'}">
                      {ast.passed ? '✓ PASS' : '✕ FAIL'}
                    </span>
                    <span class="font-mono text-xs text-slate-200">{ast.name}</span>
                  </div>
                  {#if ast.duration_ms !== undefined}
                    <span class="text-[10px] font-mono text-slate-500">{ast.duration_ms}ms</span>
                  {/if}
                </div>

                {#if ast.error}
                  <div class="mt-1 p-2 bg-slate-950/80 border border-rose-900/60 rounded font-mono text-[11px] text-rose-300 whitespace-pre-wrap">
                    {ast.error}
                  </div>
                {/if}
              </div>
            {/each}
          {:else}
            <div class="p-8 text-center text-slate-500 border border-dashed border-slate-800 rounded-lg">
              No assertions or tests configured for this request. Add assertions in the <strong>Assert</strong> or <strong>Tests</strong> tab of the request editor.
            </div>
          {/if}

          <!-- Script Console Logs -->
          {#if response?.logs && response.logs.length > 0}
            <div class="mt-4 border border-slate-800 rounded-lg overflow-hidden bg-slate-950">
              <div class="bg-slate-900 px-3 py-1.5 border-b border-slate-800 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
                Script Console Output
              </div>
              <div class="p-3 font-mono text-xs space-y-1 text-slate-300">
                {#each response.logs as logLine}
                  <div class="flex gap-2">
                    <span class="text-slate-600 select-none">&gt;</span>
                    <span class="whitespace-pre-wrap">{logLine}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>

    <!-- TIMELINE VIEW -->
    {:else if activeView === 'timeline'}
      <div class="h-full flex flex-col space-y-3 font-sans">
        <div class="flex items-center justify-between shrink-0">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
            Execution History & Timeline ({timeline.length} runs)
          </span>
        </div>

        <div class="flex-1 overflow-auto border border-slate-800 rounded-lg">
          {#if timeline.length > 0}
            <table class="w-full border-collapse text-left font-mono text-xs">
              <thead class="sticky top-0 bg-slate-900 border-b border-slate-800">
                <tr>
                  <th class="p-2 border-r border-slate-800 text-slate-400 w-12 text-center">#</th>
                  <th class="p-2 border-r border-slate-800 text-slate-400">Timestamp</th>
                  <th class="p-2 border-r border-slate-800 text-slate-400">Status</th>
                  <th class="p-2 border-r border-slate-800 text-slate-400">Duration</th>
                  <th class="p-2 text-slate-400">Payload Size</th>
                </tr>
              </thead>
              <tbody>
                {#each timeline as item, idx}
                  <tr class="border-b border-slate-800/80 hover:bg-slate-900/40">
                    <td class="p-2 border-r border-slate-800 text-center text-slate-500 font-bold">{timeline.length - idx}</td>
                    <td class="p-2 border-r border-slate-800 text-slate-300">{item.timestamp}</td>
                    <td class="p-2 border-r border-slate-800">
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-bold border {getStatusBadge(item.status)}">
                        {item.status} {item.statusText}
                      </span>
                    </td>
                    <td class="p-2 border-r border-slate-800 text-emerald-400">{item.durationMs}ms</td>
                    <td class="p-2 text-slate-400">{(item.sizeBytes / 1024).toFixed(1)} KB</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {:else}
            <div class="p-8 text-center text-slate-500">
              No historical execution runs recorded yet. Dispatched requests will be cataloged here.
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
