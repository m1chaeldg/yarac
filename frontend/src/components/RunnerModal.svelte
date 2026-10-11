<script lang="ts">
  // Svelte 5 runes
  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let collectionName = $state('User Management Suite');
  let iterations = $state(5);
  let concurrency = $state(1);
  let delayMs = $state(50);
  let dataInput = $state(`[
  { "username": "alice", "role": "admin" },
  { "username": "bob", "role": "editor" },
  { "username": "charlie", "role": "viewer" },
  { "username": "david", "role": "developer" },
  { "username": "eve", "role": "auditor" }
]`);

  let isRunning = $state(false);
  let currentIteration = $state(0);
  let runLogs = $state<Array<{
    iteration: number;
    requestName: string;
    method: string;
    status: number;
    durationMs: number;
    assertions: string;
    passed: boolean;
  }>>([]);

  // Latencies for HDR Histogram calculation
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
    const errorRate = Math.round((failures / count) * 100);
    const totalElapsedSec = (sum + count * delayMs) / 1000 || 1;
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
    };
  });

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
        iteration: `${i + 1}`,
      }));
    }

    const effectiveIterations = Math.max(iterations, parsedData.length);

    for (let i = 0; i < effectiveIterations; i++) {
      if (!isRunning) break;
      currentIteration = i + 1;

      // Simulated network execution with jitter
      const baseLatency = 20 + Math.floor(Math.random() * 45);
      const simulatedDuration = baseLatency + (concurrency > 1 ? Math.floor(Math.random() * 20) : 0);

      await new Promise((r) => setTimeout(r, Math.max(delayMs, 30)));

      latencies = [...latencies, simulatedDuration];
      const isPass = Math.random() > 0.05; // 95% success rate

      const user = parsedData[i % parsedData.length]?.username || `user_${i + 1}`;

      runLogs = [
        {
          iteration: i + 1,
          requestName: `Create User (${user})`,
          method: 'POST',
          status: isPass ? 201 : 500,
          durationMs: simulatedDuration,
          assertions: isPass ? '2/2 Passed' : '1/2 Passed (Status match failed)',
          passed: isPass,
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

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4">
    <div class="apple-modal-card w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden text-apple-primary-on-dark font-sans shadow-none">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-apple-hairline-dark/60 flex items-center justify-between bg-apple-surface-tile-2">
        <div class="flex items-center gap-3">
          <div class="w-2.5 h-2.5 rounded-full bg-apple-link-on-dark"></div>
          <h2 class="text-apple-headline font-semibold text-apple-primary-on-dark">Collection Runner & Statistical Profiler</h2>
          <span class="text-xs px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-apple-link-on-dark border border-apple-hairline-dark/60 font-mono">Phase 2</span>
        </div>
        <button
          onclick={onClose}
          aria-label="Close runner"
          class="text-apple-secondary-on-dark hover:text-apple-primary-on-dark transition-colors text-lg font-mono"
        >
          ✕
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-6 overflow-y-auto bg-apple-surface-tile-1">
        <!-- Runner Controls Grid -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4 bg-apple-surface-tile-2 p-4 rounded-apple-lg border border-apple-hairline-dark/60">
          <div>
            <label for="runner-col-name" class="block text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1.5">Collection</label>
            <input
              id="runner-col-name"
              type="text"
              bind:value={collectionName}
              class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-apple-primary-on-dark text-xs rounded-apple-sm px-2.5 py-1.5 focus:border-apple-primary"
            />
          </div>

          <div>
            <label for="runner-iterations" class="block text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1.5">Iterations</label>
            <input
              id="runner-iterations"
              type="number"
              min="1"
              max="1000"
              bind:value={iterations}
              class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-apple-primary-on-dark text-xs rounded-apple-sm px-2.5 py-1.5 focus:border-apple-primary font-mono"
            />
          </div>

          <div>
            <label for="runner-concurrency" class="block text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1.5">Concurrency</label>
            <input
              id="runner-concurrency"
              type="number"
              min="1"
              max="50"
              bind:value={concurrency}
              class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-apple-primary-on-dark text-xs rounded-apple-sm px-2.5 py-1.5 focus:border-apple-primary font-mono"
            />
          </div>

          <div>
            <label for="runner-pacing" class="block text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1.5">Pacing (ms)</label>
            <input
              id="runner-pacing"
              type="number"
              min="0"
              step="10"
              bind:value={delayMs}
              class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-apple-primary-on-dark text-xs rounded-apple-sm px-2.5 py-1.5 focus:border-apple-primary font-mono"
            />
          </div>
        </div>

        <!-- Iteration Data Config (CSV/JSON) -->
        <div>
          <label for="runner-data-input" class="block text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1.5">
            Iteration Data (JSON / CSV variables for <span class="font-mono text-apple-link-on-dark">&#123;&#123;var&#125;&#125;</span> injection)
          </label>
          <textarea
            id="runner-data-input"
            rows="3"
            bind:value={dataInput}
            class="apple-input w-full bg-apple-surface-tile-2 border border-apple-hairline-dark/80 text-apple-primary-on-dark font-mono text-xs rounded-apple-md p-2.5 focus:border-apple-primary"
          ></textarea>
        </div>

        <!-- Real-Time Statistical Profiler Cards (Apple Store Utility Cards) -->
        <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-6 gap-3">
          <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
            <span class="text-[11px] text-apple-secondary-on-dark block uppercase font-medium">Throughput</span>
            <span class="text-base font-bold text-apple-link-on-dark font-mono">{stats.rps} <span class="text-xs text-apple-secondary-on-dark font-normal">req/s</span></span>
          </div>

          <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
            <span class="text-[11px] text-apple-secondary-on-dark block uppercase font-medium">p50 Latency</span>
            <span class="text-base font-bold text-apple-primary-on-dark font-mono">{stats.p50} <span class="text-xs text-apple-secondary-on-dark font-normal">ms</span></span>
          </div>

          <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
            <span class="text-[11px] text-apple-secondary-on-dark block uppercase font-medium">p90 Latency</span>
            <span class="text-base font-bold text-amber-400 font-mono">{stats.p90} <span class="text-xs text-apple-secondary-on-dark font-normal">ms</span></span>
          </div>

          <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
            <span class="text-[11px] text-apple-secondary-on-dark block uppercase font-medium">p99 Latency</span>
            <span class="text-base font-bold text-rose-400 font-mono">{stats.p99} <span class="text-xs text-apple-secondary-on-dark font-normal">ms</span></span>
          </div>

          <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
            <span class="text-[11px] text-apple-secondary-on-dark block uppercase font-medium">Error Rate</span>
            <span class="text-base font-bold {stats.errorRate > 0 ? 'text-rose-400' : 'text-emerald-400'} font-mono">{stats.errorRate}%</span>
          </div>

          <div class="bg-apple-surface-tile-2 p-3.5 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none">
            <span class="text-[11px] text-apple-secondary-on-dark block uppercase font-medium">Total Runs</span>
            <span class="text-base font-bold text-apple-primary-on-dark font-mono">{stats.count}</span>
          </div>
        </div>

        <!-- Latency Percentile Visual Distribution Bar Chart -->
        {#if stats.count > 0}
          <div class="bg-apple-surface-tile-2 p-4 rounded-apple-lg border border-apple-hairline-dark/60 shadow-none space-y-2.5">
            <div class="text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-2">HDR Latency Percentile Distribution</div>
            {#each [
              { label: 'Min', val: stats.min, color: 'bg-emerald-500' },
              { label: 'p50', val: stats.p50, color: 'bg-apple-link-on-dark' },
              { label: 'p90', val: stats.p90, color: 'bg-amber-500' },
              { label: 'p95', val: stats.p95, color: 'bg-orange-500' },
              { label: 'p99', val: stats.p99, color: 'bg-rose-500' },
              { label: 'Max', val: stats.max, color: 'bg-red-500' }
            ] as row}
              <div class="flex items-center gap-3 text-xs font-mono">
                <span class="w-8 text-apple-secondary-on-dark">{row.label}</span>
                <div class="flex-1 bg-apple-surface-tile-3 h-2.5 rounded-apple-pill overflow-hidden border border-apple-hairline-dark/60">
                  <div
                    class="{row.color} h-full transition-all duration-300 rounded-apple-pill"
                    style="width: {Math.max(4, Math.min(100, (row.val / (stats.max || 1)) * 100))}%"
                  ></div>
                </div>
                <span class="w-16 text-right text-apple-primary-on-dark font-bold">{row.val} ms</span>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Iteration Execution Logs -->
        <div class="space-y-2">
          <div class="text-xs font-semibold text-apple-secondary-on-dark uppercase tracking-wider flex items-center justify-between">
            <span>Execution Timeline</span>
            <span class="text-[11px] text-apple-secondary-on-dark font-mono">{runLogs.length} items logged</span>
          </div>

          <div class="max-h-48 overflow-y-auto space-y-1.5 pr-1 font-mono text-xs">
            {#if runLogs.length === 0}
              <div class="text-apple-secondary-on-dark text-center py-6 bg-apple-surface-tile-2 rounded-apple-lg border border-dashed border-apple-hairline-dark/60">
                Click "Run Collection" to execute data iterations and capture latency metrics.
              </div>
            {:else}
              {#each runLogs as log}
                <div class="p-2 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark/60 flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="text-[10px] px-1.5 py-0.5 rounded-apple-sm {log.passed ? 'bg-emerald-950 text-emerald-400 border border-emerald-800/40' : 'bg-rose-950 text-rose-400 border border-rose-800/40'} font-bold">
                      {log.status}
                    </span>
                    <span class="text-apple-secondary-on-dark">#{log.iteration}</span>
                    <span class="text-apple-primary-on-dark font-semibold">{log.requestName}</span>
                  </div>
                  <div class="flex items-center gap-3 text-apple-secondary-on-dark">
                    <span class="text-emerald-400">{log.assertions}</span>
                    <span class="text-apple-link-on-dark font-bold">{log.durationMs}ms</span>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-apple-hairline-dark/60 flex items-center justify-between bg-apple-surface-tile-2">
        <div class="text-xs text-apple-secondary-on-dark">
          {#if isRunning}
            <span class="text-apple-link-on-dark animate-pulse font-medium">Running iteration {currentIteration} of {iterations}...</span>
          {:else if stats.count > 0}
            <span class="text-emerald-400 font-medium">Run completed: {stats.count} requests executed</span>
          {:else}
            <span>Ready to execute collection</span>
          {/if}
        </div>

        <div class="flex items-center gap-2">
          {#if isRunning}
            <button
              onclick={stopRun}
              class="px-4 py-1.5 rounded-apple-pill bg-rose-600 hover:bg-rose-500 text-white font-medium text-xs transition-transform active:scale-95 shadow-none"
            >
              Stop Run
            </button>
          {:else}
            <button
              onclick={startRun}
              class="apple-button-primary px-4 py-1.5 rounded-apple-pill font-medium text-xs flex items-center gap-1.5 shadow-none"
            >
              <span>▶</span> Run Collection
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
