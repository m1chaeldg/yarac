<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';
  import { backendApi } from '../services/backendApi';

  let selectedCollectionId = $state<string>('all');
  let iterations = $state(5);
  let concurrency = $state(1);
  let delayMs = $state(40);
  let rampUpSec = $state(0);
  let durationSec = $state(30);
  let enableDuration = $state(false);
  let dataInput = $state(`[
  { "userId": "101", "name": "Alice Developer", "role": "admin" },
  { "userId": "102", "name": "Bob Architect", "role": "engineer" },
  { "userId": "103", "name": "Charlie Tester", "role": "qa" }
]`);
  let loadedFileName = $state<string | null>(null);

  let isRunning = $state(false);
  let filterStatus = $state<'all' | 'passed' | 'failed'>('all');

  let runLogs = $state<Array<{
    iteration: number;
    requestName: string;
    method: string;
    status: number;
    durationMs: number;
    assertions: string;
    passed: boolean;
    timestamp: string;
  }>>([]);

  let latencies = $state<number[]>([]);

  let stats = $derived.by(() => {
    if (latencies.length === 0) {
      return {
        count: 0,
        min: 0,
        max: 0,
        mean: 0,
        p50: 0,
        p90: 0,
        p95: 0,
        p99: 0,
        rps: 0,
        errorRate: 0,
        successCount: 0,
        failureCount: 0,
      };
    }

    const sorted = [...latencies].sort((a, b) => a - b);
    const count = sorted.length;
    const min = sorted[0];
    const max = sorted[count - 1];
    const sum = sorted.reduce((a, b) => a + b, 0);
    const mean = Math.round(sum / count);

    const p = (q: number) => sorted[Math.min(Math.floor(count * q), count - 1)];

    const failures = runLogs.filter((l) => !l.passed).length;
    const successes = count - failures;
    const errorRate = count > 0 ? Math.round((failures / count) * 100) : 0;
    const totalElapsedSec = Math.max((sum / concurrency + count * (delayMs / concurrency)) / 1000, 0.1);
    const rps = (count / totalElapsedSec).toFixed(1);

    return {
      count,
      min,
      max,
      mean,
      p50: p(0.5),
      p90: p(0.9),
      p95: p(0.95),
      p99: p(0.99),
      rps: parseFloat(rps),
      errorRate,
      successCount: successes,
      failureCount: failures,
    };
  });

  let filteredLogs = $derived(
    runLogs.filter((l) => {
      if (filterStatus === 'passed') return l.passed;
      if (filterStatus === 'failed') return !l.passed;
      return true;
    })
  );

  function parseInputRecords(raw: string): Array<Record<string, string>> {
    const trimmed = raw.trim();
    if (!trimmed) return [];
    if (trimmed.startsWith('[') || trimmed.startsWith('{')) {
      try {
        const parsed = JSON.parse(trimmed);
        return Array.isArray(parsed) ? parsed : [parsed];
      } catch {
        return [];
      }
    }
    // Parse CSV lines
    const lines = trimmed.split('\n').map(l => l.trim()).filter(Boolean);
    if (lines.length <= 1) return [];
    const headers = lines[0].split(',').map(h => h.trim().replace(/^["']|["']$/g, ''));
    const records: Array<Record<string, string>> = [];
    for (let i = 1; i < lines.length; i++) {
      const values = lines[i].split(',').map(v => v.trim().replace(/^["']|["']$/g, ''));
      const row: Record<string, string> = {};
      headers.forEach((h, idx) => {
        row[h] = values[idx] || '';
      });
      records.push(row);
    }
    return records;
  }

  function handleFileUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    loadedFileName = file.name;
    const reader = new FileReader();
    reader.onload = (ev) => {
      if (ev.target?.result) {
        dataInput = ev.target.result as string;
      }
    };
    reader.readAsText(file);
    input.value = '';
  }

  async function startRun() {
    isRunning = true;
    runLogs = [];
    latencies = [];

    const parsedData = parseInputRecords(dataInput);

    const candidateRequests = selectedCollectionId === 'all'
      ? workspaceStore.currentWorkspace.requests
      : workspaceStore.currentWorkspace.requests.filter((r) => r.collectionId === selectedCollectionId);

    const requestsToRun: YaracRequest[] = candidateRequests.length > 0 ? candidateRequests : [
      {
        id: 'req_runner_default',
        collectionId: 'col_default',
        name: 'Workspace Ping',
        method: 'GET',
        url: 'http://127.0.0.1:9475/health',
        params: [],
        headers: [],
        variables: [],
        auth: { type: 'none' },
        body: { type: 'none', rawContent: '', formData: [] },
        flow: { once_per_run: false, retry_until: '', retry_interval_ms: 0, max_retries: 0, delay_ms: 0, jitter_min_ms: 0, jitter_max_ms: 0 },
        scripts: { preRequest: '', postResponse: '' },
        assert: '',
        tests: '',
        settings: { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 },
        dslContent: 'version = "1.0"\nname = "Workspace Ping"\nmethod = "GET"\nurl = "http://127.0.0.1:9475/health"\n',
        isDirty: false
      }
    ];

    try {
      const result = await backendApi.executeCollectionRun({
        requests: requestsToRun,
        iterations,
        concurrency,
        delayMs,
        rampUpSec: rampUpSec > 0 ? rampUpSec : undefined,
        durationSec: enableDuration && durationSec > 0 ? durationSec : undefined,
        data: parsedData.length > 0 ? parsedData : undefined
      });

      if (result && result.logs) {
        runLogs = result.logs.map((l) => ({
          iteration: l.iteration,
          requestName: l.requestName,
          method: l.method,
          status: l.status,
          durationMs: l.durationMs,
          assertions: l.passed ? `Status ${l.status} ✓` : `Failed (Status ${l.status}) ✗`,
          passed: l.passed,
          timestamp: l.timestamp
        }));
        latencies = result.logs.map((l) => l.durationMs);
      }
    } catch (e) {
      console.error('Collection execution error:', e);
    } finally {
      isRunning = false;
    }
  }

  function stopRun() {
    isRunning = false;
  }

  function exportJunitXml() {
    let xml = `<?xml version="1.0" encoding="UTF-8"?>\n<testsuites name="Yarac Runner" tests="${stats.count}" failures="${stats.failureCount}" time="${(stats.mean * stats.count / 1000).toFixed(2)}">\n`;
    xml += `  <testsuite name="Collection Runner" tests="${stats.count}" failures="${stats.failureCount}">\n`;
    for (const log of runLogs) {
      xml += `    <testcase name="[Iter ${log.iteration}] ${log.requestName}" time="${(log.durationMs / 1000).toFixed(3)}">\n`;
      if (!log.passed) {
        xml += `      <failure message="Request failed with status ${log.status}">${log.assertions}</failure>\n`;
      }
      xml += `    </testcase>\n`;
    }
    xml += `  </testsuite>\n</testsuites>`;
    downloadTextFile('yarac-results.xml', xml, 'application/xml');
  }

  function exportTap() {
    let tap = `TAP version 13\n1..${stats.count}\n`;
    runLogs.forEach((log, idx) => {
      const ok = log.passed ? 'ok' : 'not ok';
      tap += `${ok} ${idx + 1} - [Iter ${log.iteration}] ${log.method} ${log.requestName} (${log.durationMs}ms)\n`;
    });
    downloadTextFile('yarac-results.tap', tap, 'text/plain');
  }

  function exportCsv() {
    let csv = 'Iteration,Request,Method,Status,DurationMs,Passed,Timestamp\n';
    for (const log of runLogs) {
      csv += `${log.iteration},"${log.requestName}","${log.method}",${log.status},${log.durationMs},${log.passed},"${log.timestamp}"\n`;
    }
    downloadTextFile('yarac-results.csv', csv, 'text/csv');
  }

  function downloadTextFile(filename: string, content: string, type: string) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
</script>

<div class="h-full flex flex-col md:flex-row overflow-hidden bg-apple-surface-tile-1 text-apple-primary-on-dark font-sans">
  <!-- Left Configuration Panel -->
  <aside class="w-full md:w-80 border-r border-apple-hairline-dark/60 bg-apple-surface-tile-2 p-4 flex flex-col justify-between shrink-0 overflow-y-auto">
    <div class="space-y-4">
      <div>
        <h3 class="text-apple-subheadline font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-2">Collection Execution</h3>
        <label for="runner-col-select" class="block text-apple-caption text-apple-secondary-on-dark mb-1">Target Collection</label>
        <select
          id="runner-col-select"
          bind:value={selectedCollectionId}
          class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1.5 text-xs text-apple-primary-on-dark focus:border-apple-primary"
        >
          <option value="all">All Collections ({workspaceStore.currentWorkspace.requests.length} requests)</option>
          {#each workspaceStore.currentWorkspace.collections as col}
            <option value={col.id}>{col.name} ({workspaceStore.currentWorkspace.requests.filter(r => r.collectionId === col.id).length} reqs)</option>
          {/each}
        </select>
      </div>

      <!-- Execution Parameters -->
      <div class="space-y-3 bg-apple-surface-tile-3/80 p-3 rounded-apple-md border border-apple-hairline-dark/60">
        <div>
          <div class="flex justify-between text-xs mb-1">
            <label for="run-iterations-input" class="text-apple-secondary-on-dark font-medium">Iterations</label>
            <span class="font-mono text-apple-link-on-dark">{iterations}</span>
          </div>
          <input
            id="run-iterations-input"
            type="range"
            min="1"
            max="100"
            bind:value={iterations}
            class="w-full accent-apple-primary"
          />
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1">
            <label for="run-concurrency-input" class="text-apple-secondary-on-dark font-medium">Concurrency (Tokio Workers)</label>
            <span class="font-mono text-apple-link-on-dark">{concurrency} workers</span>
          </div>
          <input
            id="run-concurrency-input"
            type="range"
            min="1"
            max="50"
            bind:value={concurrency}
            class="w-full accent-apple-primary"
          />
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1">
            <label for="run-rampup-input" class="text-apple-secondary-on-dark font-medium">Linear Ramp-Up</label>
            <span class="font-mono text-apple-link-on-dark">{rampUpSec}s</span>
          </div>
          <input
            id="run-rampup-input"
            type="range"
            min="0"
            max="60"
            step="1"
            bind:value={rampUpSec}
            class="w-full accent-apple-primary"
          />
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1 items-center">
            <div class="flex items-center gap-1.5">
              <input
                id="run-duration-toggle"
                type="checkbox"
                bind:checked={enableDuration}
                class="w-3.5 h-3.5 rounded bg-apple-surface-tile-1 border-apple-hairline-dark accent-apple-primary"
              />
              <label for="run-duration-toggle" class="text-apple-secondary-on-dark font-medium cursor-pointer">Duration Limit</label>
            </div>
            <span class="font-mono text-apple-link-on-dark">{enableDuration ? `${durationSec}s` : 'Off'}</span>
          </div>
          {#if enableDuration}
            <input
              id="run-duration-input"
              type="range"
              min="5"
              max="300"
              step="5"
              bind:value={durationSec}
              class="w-full accent-apple-primary"
            />
          {/if}
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1">
            <label for="run-pacing-input" class="text-apple-secondary-on-dark font-medium">Pacing Delay</label>
            <span class="font-mono text-apple-link-on-dark">{delayMs} ms</span>
          </div>
          <input
            id="run-pacing-input"
            type="range"
            min="0"
            max="500"
            step="10"
            bind:value={delayMs}
            class="w-full accent-apple-primary"
          />
        </div>
      </div>

      <!-- Iteration Data Config (CSV/JSON) -->
      <div>
        <div class="flex items-center justify-between mb-1.5">
          <label for="run-data-input" class="text-apple-subheadline font-semibold text-apple-secondary-on-dark uppercase tracking-wider">
            Iteration Data (CSV/JSON)
          </label>
          <label class="apple-button-dark-utility text-[10px] px-2 py-0.5 rounded-apple-sm text-apple-primary-on-dark cursor-pointer border border-apple-hairline-dark/60">
            <span>📁 Upload</span>
            <input
              type="file"
              accept=".csv,.json"
              onchange={handleFileUpload}
              class="hidden"
            />
          </label>
        </div>
        {#if loadedFileName}
          <div class="text-[10px] text-emerald-400 font-mono mb-1">Loaded: {loadedFileName}</div>
        {/if}
        <textarea
          id="run-data-input"
          rows="5"
          bind:value={dataInput}
          class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-apple-primary-on-dark font-mono text-[11px] rounded-apple-md p-2.5 focus:border-apple-primary"
        ></textarea>
      </div>
    </div>

    <!-- Run Controls Action Button -->
    <div class="pt-4 border-t border-apple-hairline-dark/60">
      {#if isRunning}
        <button
          onclick={stopRun}
          class="w-full py-2.5 rounded-apple-pill bg-rose-600 hover:bg-rose-500 text-white font-semibold text-xs transition-transform active:scale-95 shadow-none flex items-center justify-center gap-2"
        >
          <span class="animate-pulse">⏹</span> Abort Execution
        </button>
      {:else}
        <button
          onclick={startRun}
          class="apple-button-primary w-full py-2.5 rounded-apple-pill font-semibold text-xs flex items-center justify-center gap-2 shadow-none cursor-pointer"
        >
          <span>▶</span> Run Collection ({concurrency} workers)
        </button>
      {/if}
    </div>
  </aside>

  <!-- Right Dashboard & Charts Panel -->
  <main class="flex-1 flex flex-col overflow-hidden p-4 space-y-4">
    <!-- Live Statistical Profiler KPI Cards -->
    <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-3 shrink-0">
      <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
        <span class="text-[10px] text-apple-secondary-on-dark block uppercase font-bold tracking-wider">Throughput</span>
        <span class="text-base font-bold text-apple-link-on-dark font-mono">{stats.rps} <span class="text-[10px] text-apple-secondary-on-dark font-normal">req/s</span></span>
      </div>

      <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
        <span class="text-[10px] text-apple-secondary-on-dark block uppercase font-bold tracking-wider">p50 Latency</span>
        <span class="text-base font-bold text-apple-primary-on-dark font-mono">{stats.p50} <span class="text-[10px] text-apple-secondary-on-dark font-normal">ms</span></span>
      </div>

      <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
        <span class="text-[10px] text-apple-secondary-on-dark block uppercase font-bold tracking-wider">p90 Latency</span>
        <span class="text-base font-bold text-amber-400 font-mono">{stats.p90} <span class="text-[10px] text-apple-secondary-on-dark font-normal">ms</span></span>
      </div>

      <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
        <span class="text-[10px] text-apple-secondary-on-dark block uppercase font-bold tracking-wider">p99 Latency</span>
        <span class="text-base font-bold text-rose-400 font-mono">{stats.p99} <span class="text-[10px] text-apple-secondary-on-dark font-normal">ms</span></span>
      </div>

      <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
        <span class="text-[10px] text-apple-secondary-on-dark block uppercase font-bold tracking-wider">Mean Latency</span>
        <span class="text-base font-bold text-apple-primary-on-dark font-mono">{stats.mean} <span class="text-[10px] text-apple-secondary-on-dark font-normal">ms</span></span>
      </div>

      <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
        <span class="text-[10px] text-apple-secondary-on-dark block uppercase font-bold tracking-wider">Error Rate</span>
        <span class="text-base font-bold {stats.errorRate > 0 ? 'text-rose-400' : 'text-emerald-400'} font-mono">{stats.errorRate}%</span>
      </div>

      <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
        <span class="text-[10px] text-apple-secondary-on-dark block uppercase font-bold tracking-wider">Completed</span>
        <span class="text-base font-bold text-apple-primary-on-dark font-mono">{stats.count}</span>
      </div>
    </div>

    <!-- HDR Latency Percentile Visual Distribution Bar Chart -->
    <div class="bg-apple-surface-tile-2 p-4 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none shrink-0 space-y-2.5">
      <div class="flex items-center justify-between text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-2">
        <span>HDR Latency Percentile Distribution (Tokio Async Worker Pool Profiling)</span>
        <span class="text-[11px] font-mono text-apple-secondary-on-dark">Min: {stats.min}ms | Max: {stats.max}ms</span>
      </div>

      {#each [
        { label: 'Min', val: stats.min, color: 'bg-emerald-500' },
        { label: 'p50', val: stats.p50, color: 'bg-apple-link-on-dark' },
        { label: 'p90', val: stats.p90, color: 'bg-amber-500' },
        { label: 'p95', val: stats.p95, color: 'bg-orange-500' },
        { label: 'p99', val: stats.p99, color: 'bg-rose-500' },
        { label: 'Max', val: stats.max, color: 'bg-red-500' }
      ] as row}
        <div class="flex items-center gap-3 text-xs font-mono">
          <span class="w-8 text-apple-secondary-on-dark font-semibold">{row.label}</span>
          <div class="flex-1 bg-apple-surface-tile-3 h-2.5 rounded-apple-pill overflow-hidden border border-apple-hairline-dark/60">
            <div
              class="{row.color} h-full transition-all duration-300 rounded-apple-pill"
              style="width: {Math.max(3, Math.min(100, (row.val / (stats.max || 1)) * 100))}%"
            ></div>
          </div>
          <span class="w-16 text-right text-apple-primary-on-dark font-bold">{row.val} ms</span>
        </div>
      {/each}
    </div>

    <!-- Execution Timeline Logs & Reports -->
    <div class="flex-1 flex flex-col bg-apple-surface-tile-2 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none overflow-hidden">
      <div class="px-4 py-2.5 border-b border-apple-hairline-dark/60 flex items-center justify-between bg-apple-surface-tile-3/60">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-apple-primary-on-dark uppercase tracking-wider">Execution Timeline</span>
          <span class="text-[10px] px-1.5 py-0.5 rounded-apple-sm bg-apple-surface-tile-3 text-apple-secondary-on-dark border border-apple-hairline-dark/60 font-mono">{runLogs.length} events</span>
        </div>

        <!-- Filter & Export Buttons -->
        <div class="flex items-center gap-2 text-apple-caption">
          <div class="flex items-center gap-1">
            <button
              onclick={() => filterStatus = 'all'}
              class="px-2.5 py-0.5 rounded-apple-pill font-medium transition-colors {filterStatus === 'all' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark' : 'text-apple-secondary-on-dark hover:text-apple-primary-on-dark'}"
            >
              All ({runLogs.length})
            </button>
            <button
              onclick={() => filterStatus = 'passed'}
              class="px-2.5 py-0.5 rounded-apple-pill font-medium transition-colors {filterStatus === 'passed' ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' : 'text-apple-secondary-on-dark hover:text-apple-primary-on-dark'}"
            >
              Passed ({stats.successCount})
            </button>
            <button
              onclick={() => filterStatus = 'failed'}
              class="px-2.5 py-0.5 rounded-apple-pill font-medium transition-colors {filterStatus === 'failed' ? 'bg-rose-950 text-rose-400 border border-rose-800' : 'text-apple-secondary-on-dark hover:text-apple-primary-on-dark'}"
            >
              Failed ({stats.failureCount})
            </button>
          </div>

          <!-- Report Export Actions -->
          {#if runLogs.length > 0}
            <div class="flex items-center gap-1 border-l border-apple-hairline-dark/60 pl-2">
              <button
                onclick={exportJunitXml}
                title="Export JUnit XML test report"
                class="apple-button-dark-utility text-[10px] px-2 py-0.5 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60"
              >
                JUnit XML
              </button>
              <button
                onclick={exportTap}
                title="Export Test Anything Protocol (TAP) report"
                class="apple-button-dark-utility text-[10px] px-2 py-0.5 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60"
              >
                TAP
              </button>
              <button
                onclick={exportCsv}
                title="Export Results as CSV spreadsheet"
                class="apple-button-dark-utility text-[10px] px-2 py-0.5 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60"
              >
                CSV
              </button>
            </div>
          {/if}
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-3 space-y-1.5 font-mono text-xs">
        {#if filteredLogs.length === 0}
          <div class="text-apple-secondary-on-dark text-center py-12">
            No executed requests yet. Adjust parameters and click "Run Collection" to begin.
          </div>
        {:else}
          {#each filteredLogs as log}
            <div class="p-2 rounded-apple-sm bg-apple-surface-tile-3 border border-apple-hairline-dark/60 flex items-center justify-between hover:border-apple-hairline-dark transition-colors">
              <div class="flex items-center gap-2.5">
                <span class="text-[10px] px-1.5 py-0.5 rounded-apple-sm font-bold {log.passed ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' : 'bg-rose-950 text-rose-400 border border-rose-800'}">
                  {log.status}
                </span>
                <span class="text-apple-secondary-on-dark">#{log.iteration}</span>
                <span class="text-apple-primary-on-dark font-semibold">{log.requestName}</span>
                <span class="text-[10px] text-apple-secondary-on-dark">[{log.timestamp}]</span>
              </div>
              <div class="flex items-center gap-4">
                <span class="{log.passed ? 'text-emerald-400' : 'text-rose-400'} font-sans text-[11px]">{log.assertions}</span>
                <span class="text-apple-link-on-dark font-bold w-14 text-right">{log.durationMs}ms</span>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </main>
</div>
