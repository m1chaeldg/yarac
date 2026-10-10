<script lang="ts">
  export interface FlowState {
    once_per_run: boolean;
    retry_until: string;
    retry_interval_ms: number;
    max_retries: number;
    delay_ms: number;
    jitter_min_ms: number;
    jitter_max_ms: number;
  }

  interface Props {
    flow: FlowState;
    onchange: (updated: FlowState) => void;
  }

  let { flow, onchange }: Props = $props();

  function update<K extends keyof FlowState>(key: K, value: FlowState[K]) {
    onchange({
      ...flow,
      [key]: value,
    });
  }

  function applyPreset(preset: string) {
    if (preset === 'poll-ready') {
      onchange({
        ...flow,
        retry_until: "response.status == 200 && response.body.status == 'ready'",
        retry_interval_ms: 1000,
        max_retries: 5,
      });
    } else if (preset === 'poll-created') {
      onchange({
        ...flow,
        retry_until: "response.status == 201",
        retry_interval_ms: 500,
        max_retries: 3,
      });
    } else if (preset === 'human-jitter') {
      onchange({
        ...flow,
        delay_ms: 200,
        jitter_min_ms: 50,
        jitter_max_ms: 250,
      });
    }
  }
</script>

<div class="h-full overflow-y-auto p-4 space-y-6 text-slate-200">
  <!-- Header Title -->
  <div class="flex items-center justify-between border-b border-slate-800 pb-3">
    <div>
      <h3 class="text-sm font-semibold text-slate-100 flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
        Declarative Flow Controllers & Request Pacing
      </h3>
      <p class="text-xs text-slate-400 mt-0.5">
        Configure once-per-run gating, polling conditions, and randomized think-time jitter.
      </p>
    </div>
    <div class="flex items-center gap-1.5 text-[11px]">
      <span class="text-slate-500">Quick Presets:</span>
      <button
        type="button"
        onclick={() => applyPreset('poll-ready')}
        class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-sky-300 font-mono transition-colors"
      >
        Poll Status
      </button>
      <button
        type="button"
        onclick={() => applyPreset('human-jitter')}
        class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-emerald-300 font-mono transition-colors"
      >
        Human Jitter
      </button>
    </div>
  </div>

  <!-- Section 1: Execution Gating (Once Per Run) -->
  <div class="bg-slate-900/60 border border-slate-800 rounded-lg p-4 space-y-3">
    <div class="flex items-center justify-between">
      <div>
        <label for="flow-once-per-run" class="text-xs font-semibold text-slate-200 block">
          Once-Per-Run Gating (<code class="text-sky-400 font-mono text-[11px]">flow.once_per_run</code>)
        </label>
        <span class="text-[11px] text-slate-400 block mt-0.5">
          Execute this request only on iteration 1. Subsequent iterations in collection runs or load testing will skip this call.
        </span>
      </div>
      <input
        id="flow-once-per-run"
        type="checkbox"
        checked={flow.once_per_run}
        onchange={(e) => update('once_per_run', e.currentTarget.checked)}
        class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
      />
    </div>
  </div>

  <!-- Section 2: Declarative Polling (retry_until) -->
  <div class="bg-slate-900/60 border border-slate-800 rounded-lg p-4 space-y-4">
    <div class="border-b border-slate-800/80 pb-2">
      <span class="text-xs font-semibold text-slate-200 block">
        Declarative Polling (<code class="text-sky-400 font-mono text-[11px]">flow.retry_until</code>)
      </span>
      <span class="text-[11px] text-slate-400 block mt-0.5">
        Repeats dispatch until the predicate expression evaluates to <span class="text-emerald-400 font-mono">true</span> or maximum retries are exhausted.
      </span>
    </div>

    <div>
      <label for="flow-predicate" class="block text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-1">
        Predicate Expression
      </label>
      <input
        id="flow-predicate"
        type="text"
        placeholder="e.g. response.status == 200 && response.body.status == 'ready'"
        value={flow.retry_until}
        oninput={(e) => update('retry_until', e.currentTarget.value)}
        class="w-full bg-slate-950 border border-slate-800 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
      />
      <div class="flex items-center gap-2 mt-1.5 text-[10px] text-slate-500 font-mono">
        <span>Available:</span>
        <code class="text-sky-400">response.status</code>
        <code class="text-sky-400">response.body.&lt;field&gt;</code>
        <code class="text-sky-400">response.header('Name')</code>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label for="flow-interval" class="block text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-1">
          Retry Interval (ms)
        </label>
        <input
          id="flow-interval"
          type="number"
          min="100"
          step="100"
          value={flow.retry_interval_ms}
          oninput={(e) => update('retry_interval_ms', parseInt(e.currentTarget.value) || 1000)}
          class="w-full bg-slate-950 border border-slate-800 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
        />
      </div>

      <div>
        <label for="flow-max-retries" class="block text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-1">
          Max Retries Cap
        </label>
        <input
          id="flow-max-retries"
          type="number"
          min="1"
          max="100"
          value={flow.max_retries}
          oninput={(e) => update('max_retries', parseInt(e.currentTarget.value) || 3)}
          class="w-full bg-slate-950 border border-slate-800 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
        />
      </div>
    </div>
  </div>

  <!-- Section 3: Pacing & Timers (Jitter) -->
  <div class="bg-slate-900/60 border border-slate-800 rounded-lg p-4 space-y-4">
    <div class="border-b border-slate-800/80 pb-2">
      <span class="text-xs font-semibold text-slate-200 block">
        Request Pacing & Think-Time Jitter (<code class="text-sky-400 font-mono text-[11px]">flow.delay_ms</code> / <code class="text-sky-400 font-mono text-[11px]">flow.jitter_*</code>)
      </span>
      <span class="text-[11px] text-slate-400 block mt-0.5">
        Simulates natural client pacing between calls and prevents unnatural traffic spikes across worker pools.
      </span>
    </div>

    <div class="grid grid-cols-3 gap-4">
      <div>
        <label for="flow-delay-ms" class="block text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-1">
          Base Delay (ms)
        </label>
        <input
          id="flow-delay-ms"
          type="number"
          min="0"
          step="50"
          value={flow.delay_ms}
          oninput={(e) => update('delay_ms', parseInt(e.currentTarget.value) || 0)}
          class="w-full bg-slate-950 border border-slate-800 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
        />
      </div>

      <div>
        <label for="flow-jitter-min" class="block text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-1">
          Jitter Min (ms)
        </label>
        <input
          id="flow-jitter-min"
          type="number"
          min="0"
          step="25"
          value={flow.jitter_min_ms}
          oninput={(e) => update('jitter_min_ms', parseInt(e.currentTarget.value) || 0)}
          class="w-full bg-slate-950 border border-slate-800 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
        />
      </div>

      <div>
        <label for="flow-jitter-max" class="block text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-1">
          Jitter Max (ms)
        </label>
        <input
          id="flow-jitter-max"
          type="number"
          min="0"
          step="25"
          value={flow.jitter_max_ms}
          oninput={(e) => update('jitter_max_ms', parseInt(e.currentTarget.value) || 0)}
          class="w-full bg-slate-950 border border-slate-800 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
        />
      </div>
    </div>

    <!-- Calculated Range Preview -->
    <div class="p-3 bg-slate-950/80 rounded border border-slate-800 flex items-center justify-between text-xs font-mono">
      <span class="text-slate-400">Effective Pacing Window:</span>
      <span class="text-sky-300 font-bold">
        {flow.delay_ms + flow.jitter_min_ms} ms — {flow.delay_ms + flow.jitter_max_ms} ms per request
      </span>
    </div>
  </div>
</div>
