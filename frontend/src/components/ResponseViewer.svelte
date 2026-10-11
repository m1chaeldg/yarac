<script lang="ts">
  import ResponseTableViewer from './response/ResponseTableViewer.svelte';
  import ResponseHeadersTab from './response/ResponseHeadersTab.svelte';
  import ResponseTestsTab from './response/ResponseTestsTab.svelte';
  import ResponseTimelineTab from './response/ResponseTimelineTab.svelte';
  import ResponseVisualizer from './response/ResponseVisualizer.svelte';
  import { parseTableData } from '../utils/responseUtils';
  import { calculateTestSummary } from '../utils/responseMetrics';
  import { formatDuration, formatSize, getStatusBadgeColor } from '../utils/statusUtils';
  import { formatXmlString } from '../utils/xmlUtils';

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

  let tableData = $derived(parseTableData(parsedJson));
  let testSummary = $derived(calculateTestSummary(response?.assertions));

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
</script>

<div class="h-full flex flex-col bg-slate-950 border border-slate-800 rounded-lg overflow-hidden text-xs">
  <!-- Top Response Summary Status Bar & Tab Switcher -->
  <div class="h-10 bg-slate-900 border-b border-slate-800 px-3 flex items-center justify-between shrink-0 gap-2">
    <div class="flex items-center gap-3 shrink-0">
      <div class="flex items-center gap-1.5">
        {#if response}
          <span class="px-2 py-0.5 rounded text-[11px] font-bold border font-mono {getStatusBadgeColor(response.status)}">
            {response.status} {response.statusText}
          </span>
          <span class="text-slate-400 font-mono text-[11px] ml-1">
            ⏱ <span class="text-emerald-400 font-semibold">{formatDuration(response.durationMs)}</span>
          </span>
          <span class="text-slate-500 font-mono text-[11px]">
            📦 {formatSize(response.sizeBytes)}
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
            {summaryPassedText(testSummary)}
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
  <div class="flex-1 overflow-auto">
    {#if !response}
      <div class="h-full flex flex-col items-center justify-center text-slate-500 space-y-2 p-6">
        <span class="text-3xl">⚡</span>
        <p class="font-medium text-xs">Ready to dispatch request.</p>
        <p class="text-[11px] text-slate-600">Hit "Send" to inspect the live response.</p>
      </div>
    {:else if activeView === 'json'}
      <div class="p-3 font-mono text-slate-200 text-xs">
        <pre class="whitespace-pre-wrap break-all leading-relaxed">{response.body}</pre>
      </div>
    {:else if activeView === 'table'}
      <ResponseTableViewer columns={tableData.columns} rows={tableData.rows} />
    {:else if activeView === 'visualizer'}
      <ResponseVisualizer parsedJson={parsedJson} mode="charts" />
    {:else if activeView === 'html'}
      <ResponseVisualizer parsedJson={null} bodyText={response.body} mode="html" />
    {:else if activeView === 'xml'}
      <div class="p-3 font-mono text-slate-200 text-xs">
        <pre class="whitespace-pre-wrap break-all leading-relaxed">{formatXmlString(response.body)}</pre>
      </div>
    {:else if activeView === 'raw'}
      <div class="p-3 font-mono text-slate-200 text-xs">
        <pre class="whitespace-pre-wrap break-all leading-relaxed">{response.body}</pre>
      </div>
    {:else if activeView === 'headers'}
      <ResponseHeadersTab headers={response.headers} />
    {:else if activeView === 'tests'}
      <ResponseTestsTab assertions={response.assertions} logs={response.logs} />
    {:else if activeView === 'timeline'}
      <ResponseTimelineTab timeline={timeline} />
    {/if}
  </div>
</div>

<script lang="ts" module>
  function summaryPassedText(summary: { passed: number; total: number }): string {
    return `${summary.passed}/${summary.total}`;
  }
</script>
