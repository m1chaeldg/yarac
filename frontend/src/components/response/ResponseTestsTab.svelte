<script lang="ts">
  import type { AssertionResult } from '../ResponseViewer.svelte';
  import { calculateTestSummary } from '../../utils/responseMetrics';

  interface Props {
    assertions?: AssertionResult[];
    logs?: string[];
  }

  let { assertions = [], logs = [] }: Props = $props();

  let summary = $derived(calculateTestSummary(assertions));
</script>

<div class="h-full flex flex-col bg-slate-950 overflow-hidden text-xs">
  <!-- Test Metrics Summary Bar -->
  <div class="h-10 border-b border-slate-800 px-4 bg-slate-900/60 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <span class="text-slate-400 font-medium">Test Results:</span>
        <span class="px-2 py-0.5 rounded text-xs font-bold font-mono {summary.failed > 0 ? 'bg-rose-950 text-rose-400 border border-rose-800' : 'bg-emerald-950 text-emerald-400 border border-emerald-800'}">
          {summary.passed} / {summary.total} Passed
        </span>
      </div>

      <div class="flex items-center gap-3 text-[11px] font-mono text-slate-400">
        <span class="flex items-center gap-1">
          <span class="w-2 h-2 rounded-full bg-emerald-500"></span> {summary.passed} Passed
        </span>
        <span class="flex items-center gap-1">
          <span class="w-2 h-2 rounded-full bg-rose-500"></span> {summary.failed} Failed
        </span>
        <span class="text-slate-600">|</span>
        <span>Pass rate: <strong class={summary.failed > 0 ? 'text-rose-400' : 'text-emerald-400'}>{summary.passRate}%</strong></span>
      </div>
    </div>
  </div>

  <div class="flex-1 overflow-auto p-4 space-y-4">
    <!-- Assertions List -->
    <div>
      <h4 class="text-[11px] uppercase tracking-wider font-bold text-slate-500 mb-2">Assertions</h4>
      {#if assertions.length === 0}
        <div class="p-6 text-center text-slate-500 italic bg-slate-900/30 rounded border border-slate-800/60">
          No test assertions executed for this request.
        </div>
      {:else}
        <div class="space-y-1.5 font-mono text-[11px]">
          {#each assertions as assertion}
            <div class="p-2.5 rounded border flex items-start justify-between gap-3 {assertion.passed ? 'bg-emerald-950/20 border-emerald-800/40 text-slate-200' : 'bg-rose-950/20 border-rose-800/40 text-slate-200'}">
              <div class="flex items-start gap-2">
                <span class={assertion.passed ? 'text-emerald-400 font-bold' : 'text-rose-400 font-bold'}>
                  {assertion.passed ? '✓' : '✗'}
                </span>
                <div>
                  <div class="font-medium {assertion.passed ? 'text-emerald-300' : 'text-rose-300'}">
                    {assertion.name}
                  </div>
                  {#if assertion.error}
                    <div class="text-[10px] text-rose-400 mt-1 pl-2 border-l border-rose-800 font-sans">
                      {assertion.error}
                    </div>
                  {/if}
                </div>
              </div>

              {#if assertion.duration_ms !== undefined}
                <span class="text-[10px] text-slate-500 shrink-0">
                  {assertion.duration_ms}ms
                </span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Script Console Logs -->
    {#if logs && logs.length > 0}
      <div>
        <h4 class="text-[11px] uppercase tracking-wider font-bold text-slate-500 mb-2">Console Output</h4>
        <div class="p-3 bg-slate-900 border border-slate-800 rounded font-mono text-[11px] space-y-1 text-slate-300">
          {#each logs as log}
            <div class="text-sky-300 whitespace-pre-wrap">{log}</div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
