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

<div class="h-full overflow-y-auto p-4 space-y-6 text-apple-primary-on-dark font-sans text-apple-callout">
  <!-- Header Title -->
  <div class="flex items-center justify-between border-b border-apple-hairline-dark/60 pb-3">
    <div>
      <h3 class="text-apple-headline font-semibold text-apple-primary-on-dark flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full bg-apple-link-on-dark"></span>
        Declarative Flow Controllers & Request Pacing
      </h3>
      <p class="text-apple-caption text-apple-secondary-on-dark mt-0.5">
        Configure once-per-run gating, polling conditions, and randomized think-time jitter.
      </p>
    </div>
    <div class="flex items-center gap-1.5 text-apple-caption">
      <span class="text-apple-secondary-on-dark">Quick Presets:</span>
      <button
        type="button"
        onclick={() => applyPreset('poll-ready')}
        class="apple-button-dark-utility text-apple-link-on-dark font-mono text-[11px]"
      >
        Poll Status
      </button>
      <button
        type="button"
        onclick={() => applyPreset('human-jitter')}
        class="apple-button-dark-utility text-apple-link-on-dark font-mono text-[11px]"
      >
        Human Jitter
      </button>
    </div>
  </div>

  <!-- Section 1: Execution Gating (Once Per Run) -->
  <div class="bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-md p-4 space-y-3">
    <div class="flex items-center justify-between">
      <div>
        <label for="flow-once-per-run" class="text-apple-callout font-semibold text-apple-primary-on-dark block">
          Once-Per-Run Gating (<code class="text-apple-link-on-dark font-mono text-[11px]">flow.once_per_run</code>)
        </label>
        <span class="text-apple-caption text-apple-secondary-on-dark block mt-0.5">
          Execute this request only on iteration 1. Subsequent iterations in collection runs or load testing will skip this call.
        </span>
      </div>
      <input
        id="flow-once-per-run"
        type="checkbox"
        checked={flow.once_per_run}
        onchange={(e) => update('once_per_run', e.currentTarget.checked)}
        class="w-4 h-4 rounded-apple-sm bg-apple-surface-tile-3 border-apple-hairline-dark text-apple-primary focus:ring-0 focus:ring-offset-0 cursor-pointer"
      />
    </div>
  </div>

  <!-- Section 2: Declarative Polling (retry_until) -->
  <div class="bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-md p-4 space-y-4">
    <div class="border-b border-apple-hairline-dark/60 pb-2">
      <span class="text-apple-callout font-semibold text-apple-primary-on-dark block">
        Declarative Polling (<code class="text-apple-link-on-dark font-mono text-[11px]">flow.retry_until</code>)
      </span>
      <span class="text-apple-caption text-apple-secondary-on-dark block mt-0.5">
        Repeats dispatch until the predicate expression evaluates to <span class="text-apple-link-on-dark font-mono">true</span> or maximum retries are exhausted.
      </span>
    </div>

    <div>
      <label for="flow-predicate" class="block text-[11px] font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1">
        Predicate Expression
      </label>
      <input
        id="flow-predicate"
        type="text"
        placeholder="e.g. response.status == 200 && response.body.status == 'ready'"
        value={flow.retry_until}
        oninput={(e) => update('retry_until', e.currentTarget.value)}
        class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark focus:border-apple-primary"
      />
      <div class="flex items-center gap-2 mt-1.5 text-[10px] text-apple-secondary-on-dark font-mono">
        <span>Available:</span>
        <code class="text-apple-link-on-dark">response.status</code>
        <code class="text-apple-link-on-dark">response.body.&lt;field&gt;</code>
        <code class="text-apple-link-on-dark">response.header('Name')</code>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label for="flow-interval" class="block text-[11px] font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1">
          Retry Interval (ms)
        </label>
        <input
          id="flow-interval"
          type="number"
          min="100"
          step="100"
          value={flow.retry_interval_ms}
          oninput={(e) => update('retry_interval_ms', parseInt(e.currentTarget.value) || 1000)}
          class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark focus:border-apple-primary"
        />
      </div>

      <div>
        <label for="flow-max-retries" class="block text-[11px] font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1">
          Max Retries Cap
        </label>
        <input
          id="flow-max-retries"
          type="number"
          min="1"
          max="100"
          value={flow.max_retries}
          oninput={(e) => update('max_retries', parseInt(e.currentTarget.value) || 3)}
          class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark focus:border-apple-primary"
        />
      </div>
    </div>
  </div>

  <!-- Section 3: Pacing & Timers (Jitter) -->
  <div class="bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-md p-4 space-y-4">
    <div class="border-b border-apple-hairline-dark/60 pb-2">
      <span class="text-apple-callout font-semibold text-apple-primary-on-dark block">
        Request Pacing & Think-Time Jitter (<code class="text-apple-link-on-dark font-mono text-[11px]">flow.delay_ms</code> / <code class="text-apple-link-on-dark font-mono text-[11px]">flow.jitter_*</code>)
      </span>
      <span class="text-apple-caption text-apple-secondary-on-dark block mt-0.5">
        Simulates natural client pacing between calls and prevents unnatural traffic spikes across worker pools.
      </span>
    </div>

    <div class="grid grid-cols-3 gap-4">
      <div>
        <label for="flow-delay-ms" class="block text-[11px] font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1">
          Base Delay (ms)
        </label>
        <input
          id="flow-delay-ms"
          type="number"
          min="0"
          step="50"
          value={flow.delay_ms}
          oninput={(e) => update('delay_ms', parseInt(e.currentTarget.value) || 0)}
          class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark focus:border-apple-primary"
        />
      </div>

      <div>
        <label for="flow-jitter-min" class="block text-[11px] font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1">
          Jitter Min (ms)
        </label>
        <input
          id="flow-jitter-min"
          type="number"
          min="0"
          step="25"
          value={flow.jitter_min_ms}
          oninput={(e) => update('jitter_min_ms', parseInt(e.currentTarget.value) || 0)}
          class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark focus:border-apple-primary"
        />
      </div>

      <div>
        <label for="flow-jitter-max" class="block text-[11px] font-semibold text-apple-secondary-on-dark uppercase tracking-wider mb-1">
          Jitter Max (ms)
        </label>
        <input
          id="flow-jitter-max"
          type="number"
          min="0"
          step="25"
          value={flow.jitter_max_ms}
          oninput={(e) => update('jitter_max_ms', parseInt(e.currentTarget.value) || 0)}
          class="apple-input w-full bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-3 py-1.5 text-xs font-mono text-apple-primary-on-dark focus:border-apple-primary"
        />
      </div>
    </div>

    <!-- Calculated Range Preview -->
    <div class="p-3 bg-apple-surface-tile-3 rounded-apple-sm border border-apple-hairline-dark/60 flex items-center justify-between text-xs font-mono">
      <span class="text-apple-secondary-on-dark">Effective Pacing Window:</span>
      <span class="text-apple-link-on-dark font-bold">
        {flow.delay_ms + flow.jitter_min_ms} ms — {flow.delay_ms + flow.jitter_max_ms} ms per request
      </span>
    </div>
  </div>
</div>
