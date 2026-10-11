<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';
  import { backendApi } from '../services/backendApi';

  let selectedCollectionId = $state<string>('all');
  let iterations = $state(5);
  let concurrency = $state(1);
  let delayMs = $state(40);
  let dataInput = $state(`[
  { "userId": "101", "name": "Alice Developer", "role": "admin" },
  { "userId": "102", "name": "Bob Architect", "role": "engineer" },
  { "userId": "103", "name": "Charlie Tester", "role": "qa" }
]`);

  let isRunning = $state(false);
  let currentIteration = $state(0);
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
    const errorRate = Math.round((failures / count) * 100);
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

  async function startRun() {
    isRunning = true;
    currentIteration = 0;
    runLogs = [];
    latencies = [];

    let parsedData: Array<Record<string, string>> = [];
    try {
      parsedData = JSON.parse(dataInput);
    } catch {
      parsedData = Array.from({ length: iterations }, (_, i) => ({
        userId: `${i + 1}`,
        name: `User ${i + 1}`,
      }));
    }

    const effectiveIterations = Math.max(iterations, 1);

    // Collect base active environment variables
    const activeEnv = workspaceStore.currentWorkspace.environments.find(
      (e) => e.id === workspaceStore.currentWorkspace.activeEnvironmentId
    );
    const baseEnvVars: Record<string, string> = {};
    if (activeEnv) {
      for (const v of activeEnv.variables) {
        if (v.enabled && v.key.trim()) {
          baseEnvVars[v.key.trim()] = v.value;
        }
      }
    }

    // Determine target requests
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
        auth: { type: 'none' },
        body: { type: 'none', rawContent: '', formData: [] },
        flow: { once_per_run: false, retry_until: '', retry_interval_ms: 0, max_retries: 0, delay_ms: 0, jitter_min_ms: 0, jitter_max_ms: 0 },
        dslContent: 'version = "1.0"\nname = "Workspace Ping"\nmethod = "GET"\nurl = "http://127.0.0.1:9475/health"\n',
        isDirty: false
      }
    ];

    for (let i = 0; i < effectiveIterations; i++) {
      if (!isRunning) break;
      currentIteration = i + 1;

      const record = parsedData.length > 0 ? parsedData[i % parsedData.length] : {};
      const mergedEnv = { ...baseEnvVars, ...record };

      for (const req of requestsToRun) {
        if (!isRunning) break;

        const rep = await backendApi.executeRequest(req, mergedEnv);
        const duration = Math.max(1, rep.duration_ms);
        latencies = [...latencies, duration];

        const isPass = !rep.error && rep.status_code >= 200 && rep.status_code < 400;
        const now = new Date();
        const timeStr = now.toTimeString().split(' ')[0] + '.' + String(now.getMilliseconds()).padStart(3, '0');

        let assertionText = 'No assertions';
        if (rep.assertions && rep.assertions.length > 0) {
          const passCount = rep.assertions.filter((a) => a.passed).length;
          assertionText = `${passCount}/${rep.assertions.length} Passed ${passCount === rep.assertions.length ? '✓' : '✗'}`;
        } else if (isPass) {
          assertionText = `Status ${rep.status_code} ✓`;
        } else {
          assertionText = rep.error || `Status ${rep.status_code} ✗`;
        }

        runLogs = [
          {
            iteration: i + 1,
            requestName: `${req.name} (${record?.name || req.method})`,
            method: req.method,
            status: rep.status_code,
            durationMs: duration,
            assertions: assertionText,
            passed: isPass,
            timestamp: timeStr,
          },
          ...runLogs,
        ];

        if (delayMs > 0) {
          await new Promise((r) => setTimeout(r, delayMs));
        }
      }
    }

    isRunning = false;
  }

  function stopRun() {
    isRunning = false;
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
            <label for="run-concurrency-input" class="text-apple-secondary-on-dark font-medium">Concurrency (Workers)</label>
            <span class="font-mono text-apple-link-on-dark">{concurrency} workers</span>
          </div>
          <input
            id="run-concurrency-input"
            type="range"
            min="1"
            max="20"
            bind:value={concurrency}
            class="w-full accent-apple-primary"
          />
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
        <label for="run-data-input" class="block text-apple-subheadline font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1.5">
          Iteration Data (JSON / CSV)
        </label>
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
          class="apple-button-primary w-full py-2.5 rounded-apple-pill font-semibold text-xs flex items-center justify-center gap-2 shadow-none"
        >
          <span>▶</span> Run Collection ({iterations} iters)
        </button>
      {/if}
    </div>
  </aside>

  <!-- Right Dashboard & Charts Panel -->
  <main class="flex-1 flex flex-col overflow-hidden p-4 space-y-4">
    <!-- Live Statistical Profiler KPI Cards (Apple Store Utility Cards) -->
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
        <span class="text-base font-bold text-apple-primary-on-dark font-mono">{stats.count}/{iterations}</span>
      </div>
    </div>

    <!-- HDR Latency Percentile Visual Distribution Bar Chart -->
    <div class="bg-apple-surface-tile-2 p-4 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none shrink-0 space-y-2.5">
      <div class="flex items-center justify-between text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-2">
        <span>HDR Latency Percentile Distribution (In-Memory Micro-Load Profiling)</span>
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

    <!-- Execution Timeline Logs -->
    <div class="flex-1 flex flex-col bg-apple-surface-tile-2 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none overflow-hidden">
      <div class="px-4 py-2.5 border-b border-apple-hairline-dark/60 flex items-center justify-between bg-apple-surface-tile-3/60">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-apple-primary-on-dark uppercase tracking-wider">Execution Timeline</span>
          <span class="text-[10px] px-1.5 py-0.5 rounded-apple-sm bg-apple-surface-tile-3 text-apple-secondary-on-dark border border-apple-hairline-dark/60 font-mono">{runLogs.length} events</span>
        </div>

        <!-- Filter Buttons -->
        <div class="flex items-center gap-1.5 text-apple-caption">
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
