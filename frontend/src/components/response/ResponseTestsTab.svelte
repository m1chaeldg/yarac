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

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <!-- Test Metrics Summary Bar -->
  <div class="h-10 border-b border-apple-hairline-dark/60 px-4 bg-apple-surface-tile-2 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <span class="text-apple-body-muted font-medium">Test Results:</span>
        <span class="px-2.5 py-0.5 rounded-apple-pill text-xs font-bold font-mono {summary.failed > 0 ? 'bg-rose-950/80 text-rose-400 border border-rose-800/80' : 'bg-emerald-950/80 text-emerald-400 border border-emerald-800/80'}">
          {summary.passed} / {summary.total} Passed
        </span>
      </div>

      <div class="flex items-center gap-3 text-[11px] font-mono text-apple-body-muted">
        <span class="flex items-center gap-1">
          <span class="w-2 h-2 rounded-apple-full bg-emerald-500"></span> {summary.passed} Passed
        </span>
        <span class="flex items-center gap-1">
          <span class="w-2 h-2 rounded-apple-full bg-rose-500"></span> {summary.failed} Failed
        </span>
        <span class="text-apple-ink-muted-48">|</span>
        <span>Pass rate: <strong class={summary.failed > 0 ? 'text-rose-400' : 'text-emerald-400'}>{summary.passRate}%</strong></span>
      </div>
    </div>
  </div>

  <div class="flex-1 overflow-auto p-4 space-y-4">
    <!-- Assertions List -->
    <div>
      <h4 class="text-[11px] uppercase tracking-wider font-semibold text-apple-body-muted mb-2 font-sans">Assertions</h4>
      {#if assertions.length === 0}
        <div class="p-6 text-center text-apple-ink-muted-48 italic bg-apple-surface-tile-2 rounded-apple-md border border-apple-hairline-dark/60 font-sans">
          No test assertions executed for this request.
        </div>
      {:else}
        <div class="space-y-1.5 font-mono text-[11px]">
          {#each assertions as assertion}
            <div class="p-3 rounded-apple-md border border-apple-hairline-dark/60 bg-apple-surface-tile-2 flex items-start justify-between gap-3 text-apple-body-on-dark">
              <div class="flex items-start gap-2.5">
                <span class={assertion.passed ? 'text-emerald-400 font-bold' : 'text-rose-400 font-bold'}>
                  {assertion.passed ? '✓' : '✗'}
                </span>
                <div>
                  <div class="font-medium text-apple-body-on-dark">
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
                <span class="text-[10px] text-apple-ink-muted-48 shrink-0">
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
        <h4 class="text-[11px] uppercase tracking-wider font-semibold text-apple-body-muted mb-2 font-sans">Console Output</h4>
        <div class="p-3 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-md font-mono text-[11px] space-y-1 text-apple-primary-on-dark">
          {#each logs as log}
            <div class="whitespace-pre-wrap">{log}</div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
