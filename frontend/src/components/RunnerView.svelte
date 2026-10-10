<script lang="ts">
  let collectionName = $state('User Management Suite');
  let iterations = $state(10);
  let concurrency = $state(2);
  let delayMs = $state(40);
  let dataInput = $state(`[
  { "userId": "101", "name": "Alice Developer", "role": "admin" },
  { "userId": "102", "name": "Bob Architect", "role": "engineer" },
  { "userId": "103", "name": "Charlie Tester", "role": "qa" },
  { "userId": "104", "name": "David Manager", "role": "lead" },
  { "userId": "105", "name": "Eve Security", "role": "auditor" }
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

    const effectiveIterations = Math.max(iterations, parsedData.length);

    for (let i = 0; i < effectiveIterations; i++) {
      if (!isRunning) break;
      currentIteration = i + 1;

      // Realistic latency simulation with jitter
      const baseLatency = 18 + Math.floor(Math.random() * 35);
      const concurrencyFactor = concurrency > 1 ? Math.floor(Math.random() * 15) : 0;
      const duration = baseLatency + concurrencyFactor;

      await new Promise((r) => setTimeout(r, Math.max(delayMs, 20)));

      latencies = [...latencies, duration];
      const isPass = Math.random() > 0.08;

      const record = parsedData[i % parsedData.length];
      const name = record?.name || `user_${i + 1}`;

      const now = new Date();
      const timeStr = now.toTimeString().split(' ')[0] + '.' + String(now.getMilliseconds()).padStart(3, '0');

      runLogs = [
        {
          iteration: i + 1,
          requestName: `POST /users (${name})`,
          method: 'POST',
          status: isPass ? 201 : 503,
          durationMs: duration,
          assertions: isPass ? '2/2 Passed ✓' : 'Assertion failed (Status != 201) ✗',
          passed: isPass,
          timestamp: timeStr,
        },
        ...runLogs,
      ];
    }

    isRunning = false;
  }

  function stopRun() {
    isRunning = false;
  }
</script>

<div class="h-full flex flex-col md:flex-row overflow-hidden bg-slate-950 text-slate-100">
  <!-- Left Configuration Panel -->
  <aside class="w-full md:w-80 border-r border-slate-800 bg-slate-900/40 p-4 flex flex-col justify-between shrink-0 overflow-y-auto">
    <div class="space-y-4">
      <div>
        <h3 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Collection Execution</h3>
        <label for="runner-col-select" class="block text-[11px] text-slate-400 mb-1">Target Collection</label>
        <select
          id="runner-col-select"
          bind:value={collectionName}
          class="w-full bg-slate-950 border border-slate-800 rounded px-2.5 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-sky-500"
        >
          <option>User Management Suite</option>
          <option>Authentication & OAuth2</option>
          <option>Checkout & Payment Flow</option>
        </select>
      </div>

      <!-- Execution Parameters -->
      <div class="space-y-3 bg-slate-950/60 p-3 rounded-lg border border-slate-800/80">
        <div>
          <div class="flex justify-between text-xs mb-1">
            <label for="run-iterations-input" class="text-slate-400 font-medium">Iterations</label>
            <span class="font-mono text-sky-400">{iterations}</span>
          </div>
          <input
            id="run-iterations-input"
            type="range"
            min="1"
            max="100"
            bind:value={iterations}
            class="w-full accent-sky-500"
          />
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1">
            <label for="run-concurrency-input" class="text-slate-400 font-medium">Concurrency (Workers)</label>
            <span class="font-mono text-sky-400">{concurrency} workers</span>
          </div>
          <input
            id="run-concurrency-input"
            type="range"
            min="1"
            max="20"
            bind:value={concurrency}
            class="w-full accent-sky-500"
          />
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1">
            <label for="run-pacing-input" class="text-slate-400 font-medium">Pacing Delay</label>
            <span class="font-mono text-sky-400">{delayMs} ms</span>
          </div>
          <input
            id="run-pacing-input"
            type="range"
            min="0"
            max="500"
            step="10"
            bind:value={delayMs}
            class="w-full accent-sky-500"
          />
        </div>
      </div>

      <!-- Iteration Data Config (CSV/JSON) -->
      <div>
        <label for="run-data-input" class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
          Iteration Data (JSON / CSV)
        </label>
        <textarea
          id="run-data-input"
          rows="5"
          bind:value={dataInput}
          class="w-full bg-slate-950 border border-slate-800 text-slate-300 font-mono text-[11px] rounded-lg p-2.5 focus:outline-none focus:border-sky-500"
        ></textarea>
      </div>
    </div>

    <!-- Run Controls Action Button -->
    <div class="pt-4 border-t border-slate-800">
      {#if isRunning}
        <button
          onclick={stopRun}
          class="w-full py-2.5 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-semibold text-xs transition-colors shadow-lg shadow-rose-900/30 flex items-center justify-center gap-2"
        >
          <span class="animate-pulse">⏹</span> Abort Execution
        </button>
      {:else}
        <button
          onclick={startRun}
          class="w-full py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-xs transition-colors shadow-lg shadow-emerald-900/30 flex items-center justify-center gap-2"
        >
          <span>▶</span> Run Collection ({iterations} iters)
        </button>
      {/if}
    </div>
  </aside>

  <!-- Right Dashboard & Charts Panel -->
  <main class="flex-1 flex flex-col overflow-hidden p-4 space-y-4">
    <!-- Live Statistical Profiler KPI Cards -->
    <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-3 shrink-0">
      <div class="bg-slate-900/80 p-3 rounded-lg border border-slate-800">
        <span class="text-[10px] text-slate-500 block uppercase font-bold tracking-wider">Throughput</span>
        <span class="text-base font-bold text-sky-400 font-mono">{stats.rps} <span class="text-[10px] text-slate-500 font-normal">req/s</span></span>
      </div>

      <div class="bg-slate-900/80 p-3 rounded-lg border border-slate-800">
        <span class="text-[10px] text-slate-500 block uppercase font-bold tracking-wider">p50 Latency</span>
        <span class="text-base font-bold text-slate-200 font-mono">{stats.p50} <span class="text-[10px] text-slate-500 font-normal">ms</span></span>
      </div>

      <div class="bg-slate-900/80 p-3 rounded-lg border border-slate-800">
        <span class="text-[10px] text-slate-500 block uppercase font-bold tracking-wider">p90 Latency</span>
        <span class="text-base font-bold text-amber-400 font-mono">{stats.p90} <span class="text-[10px] text-slate-500 font-normal">ms</span></span>
      </div>

      <div class="bg-slate-900/80 p-3 rounded-lg border border-slate-800">
        <span class="text-[10px] text-slate-500 block uppercase font-bold tracking-wider">p99 Latency</span>
        <span class="text-base font-bold text-rose-400 font-mono">{stats.p99} <span class="text-[10px] text-slate-500 font-normal">ms</span></span>
      </div>

      <div class="bg-slate-900/80 p-3 rounded-lg border border-slate-800">
        <span class="text-[10px] text-slate-500 block uppercase font-bold tracking-wider">Mean Latency</span>
        <span class="text-base font-bold text-slate-300 font-mono">{stats.mean} <span class="text-[10px] text-slate-500 font-normal">ms</span></span>
      </div>

      <div class="bg-slate-900/80 p-3 rounded-lg border border-slate-800">
        <span class="text-[10px] text-slate-500 block uppercase font-bold tracking-wider">Error Rate</span>
        <span class="text-base font-bold {stats.errorRate > 0 ? 'text-rose-400' : 'text-emerald-400'} font-mono">{stats.errorRate}%</span>
      </div>

      <div class="bg-slate-900/80 p-3 rounded-lg border border-slate-800">
        <span class="text-[10px] text-slate-500 block uppercase font-bold tracking-wider">Completed</span>
        <span class="text-base font-bold text-slate-100 font-mono">{stats.count}/{iterations}</span>
      </div>
    </div>

    <!-- HDR Latency Percentile Visual Distribution Bar Chart -->
    <div class="bg-slate-900/50 p-4 rounded-lg border border-slate-800 shrink-0 space-y-2">
      <div class="flex items-center justify-between text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">
        <span>HDR Latency Percentile Distribution (In-Memory Micro-Load Profiling)</span>
        <span class="text-[11px] font-mono text-slate-500">Min: {stats.min}ms | Max: {stats.max}ms</span>
      </div>

      {#each [
        { label: 'Min', val: stats.min, color: 'bg-emerald-500' },
        { label: 'p50', val: stats.p50, color: 'bg-sky-500' },
        { label: 'p90', val: stats.p90, color: 'bg-amber-500' },
        { label: 'p95', val: stats.p95, color: 'bg-orange-500' },
        { label: 'p99', val: stats.p99, color: 'bg-rose-500' },
        { label: 'Max', val: stats.max, color: 'bg-red-500' }
      ] as row}
        <div class="flex items-center gap-3 text-xs font-mono">
          <span class="w-8 text-slate-400 font-semibold">{row.label}</span>
          <div class="flex-1 bg-slate-950 h-3 rounded-full overflow-hidden border border-slate-800/80">
            <div
              class="{row.color} h-full transition-all duration-300 rounded-full"
              style="width: {Math.max(3, Math.min(100, (row.val / (stats.max || 1)) * 100))}%"
            ></div>
          </div>
          <span class="w-16 text-right text-slate-200 font-bold">{row.val} ms</span>
        </div>
      {/each}
    </div>

    <!-- Execution Timeline Logs -->
    <div class="flex-1 flex flex-col bg-slate-900/40 rounded-lg border border-slate-800 overflow-hidden">
      <div class="px-4 py-2.5 border-b border-slate-800 flex items-center justify-between bg-slate-950/60">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-slate-300 uppercase tracking-wider">Execution Timeline</span>
          <span class="text-[10px] px-1.5 py-0.2 rounded bg-slate-800 text-slate-400 font-mono">{runLogs.length} events</span>
        </div>

        <!-- Filter Buttons -->
        <div class="flex items-center gap-1.5 text-[11px]">
          <button
            onclick={() => filterStatus = 'all'}
            class="px-2 py-0.5 rounded font-medium transition-colors {filterStatus === 'all' ? 'bg-slate-700 text-slate-100' : 'text-slate-400 hover:text-slate-200'}"
          >
            All ({runLogs.length})
          </button>
          <button
            onclick={() => filterStatus = 'passed'}
            class="px-2 py-0.5 rounded font-medium transition-colors {filterStatus === 'passed' ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' : 'text-slate-400 hover:text-slate-200'}"
          >
            Passed ({stats.successCount})
          </button>
          <button
            onclick={() => filterStatus = 'failed'}
            class="px-2 py-0.5 rounded font-medium transition-colors {filterStatus === 'failed' ? 'bg-rose-950 text-rose-400 border border-rose-800' : 'text-slate-400 hover:text-slate-200'}"
          >
            Failed ({stats.failureCount})
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-3 space-y-1.5 font-mono text-xs">
        {#if filteredLogs.length === 0}
          <div class="text-slate-500 text-center py-12">
            No executed requests yet. Adjust parameters and click "Run Collection" to begin.
          </div>
        {:else}
          {#each filteredLogs as log}
            <div class="p-2 rounded bg-slate-950/80 border border-slate-800/80 flex items-center justify-between hover:border-slate-700 transition-colors">
              <div class="flex items-center gap-2.5">
                <span class="text-[10px] px-1.5 py-0.5 rounded font-bold {log.passed ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' : 'bg-rose-950 text-rose-400 border border-rose-800'}">
                  {log.status}
                </span>
                <span class="text-slate-500">#{log.iteration}</span>
                <span class="text-slate-200 font-semibold">{log.requestName}</span>
                <span class="text-[10px] text-slate-500">[{log.timestamp}]</span>
              </div>
              <div class="flex items-center gap-4">
                <span class="{log.passed ? 'text-emerald-400' : 'text-rose-400'} font-sans text-[11px]">{log.assertions}</span>
                <span class="text-sky-400 font-bold w-14 text-right">{log.durationMs}ms</span>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </main>
</div>
