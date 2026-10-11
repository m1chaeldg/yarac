<script lang="ts">
  import ResponseTableViewer from './response/ResponseTableViewer.svelte';
  import ResponseHeadersTab from './response/ResponseHeadersTab.svelte';
  import ResponseTestsTab from './response/ResponseTestsTab.svelte';
  import ResponseTimelineTab from './response/ResponseTimelineTab.svelte';
  import ResponseVisualizer from './response/ResponseVisualizer.svelte';
  import JsonTreeView from './response/JsonTreeView.svelte';
  import { parseTableData } from '../utils/responseUtils';
  import { calculateTestSummary } from '../utils/responseMetrics';
  import { formatDuration, formatSize, getStatusBadgeColor } from '../utils/statusUtils';
  import { formatXmlString } from '../utils/xmlUtils';
  import { formatJsonString, compressJsonString } from '../utils/jsonUtils';

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
  let jsonSubMode = $state<'code' | 'tree'>('code');
  let jsonExpandVersion = $state(0);
  let jsonCollapseVersion = $state(0);
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

  // JSON formatting & compression actions
  function handleFormatJson() {
    if (!response?.body) return;
    try {
      response.body = formatJsonString(response.body, 2);
      flashFeedback('Prettified JSON');
    } catch (e: any) {
      flashFeedback('Invalid JSON');
    }
  }

  function handleCompressJson() {
    if (!response?.body) return;
    try {
      response.body = compressJsonString(response.body);
      flashFeedback('Compressed JSON');
    } catch (e: any) {
      flashFeedback('Invalid JSON');
    }
  }

  function handleExpandAllTree() {
    jsonExpandVersion += 1;
    flashFeedback('Expanded all nodes');
  }

  function handleCollapseAllTree() {
    jsonCollapseVersion += 1;
    flashFeedback('Collapsed all nodes');
  }

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

<div class="h-full flex flex-col bg-apple-surface-tile-1 border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden text-xs">
  <!-- Top Response Summary Status Bar & Tab Switcher -->
  <div class="h-10 bg-apple-surface-tile-2 border-b border-apple-hairline-dark/60 px-3 flex items-center justify-between shrink-0 gap-2">
    <div class="flex items-center gap-3 shrink-0">
      <div class="flex items-center gap-1.5">
        {#if response}
          <span class="px-2 py-0.5 rounded-apple-pill text-[11px] font-bold border font-mono {getStatusBadgeColor(response.status)}">
            {response.status} {response.statusText}
          </span>
          <span class="text-apple-body-muted font-mono text-[11px] ml-1">
            ⏱ <span class="text-emerald-400 font-semibold">{formatDuration(response.durationMs)}</span>
          </span>
          <span class="text-apple-ink-muted-48 font-mono text-[11px]">
            📦 {formatSize(response.sizeBytes)}
          </span>
        {:else}
          <span class="text-apple-ink-muted-48 italic">No response received yet</span>
        {/if}
      </div>

      {#if copyFeedback}
        <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-apple-primary/20 text-apple-primary-on-dark border border-apple-primary/40 font-mono animate-fade-in">
          ✓ {copyFeedback}
        </span>
      {/if}
    </div>

    <!-- View Switcher Tabs -->
    <div class="flex items-center gap-1 overflow-x-auto">
      <button
        type="button"
        onclick={() => activeView = 'json'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors cursor-pointer {activeView === 'json' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        JSON
      </button>

      <button
        type="button"
        onclick={() => activeView = 'table'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors flex items-center gap-1 cursor-pointer {activeView === 'table' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        <span>Table</span>
        {#if tableData.rows.length > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-body-muted border border-apple-hairline-dark/60 font-mono">
            {tableData.rows.length}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => activeView = 'visualizer'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors flex items-center gap-1 cursor-pointer {activeView === 'visualizer' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        <span>Charts</span>
      </button>

      <button
        type="button"
        onclick={() => activeView = 'html'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors cursor-pointer {activeView === 'html' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        HTML
      </button>

      <button
        type="button"
        onclick={() => activeView = 'xml'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors cursor-pointer {activeView === 'xml' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        XML
      </button>

      <button
        type="button"
        onclick={() => activeView = 'raw'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors cursor-pointer {activeView === 'raw' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        Raw
      </button>

      <button
        type="button"
        onclick={() => activeView = 'headers'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors flex items-center gap-1 cursor-pointer {activeView === 'headers' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        <span>Headers</span>
        {#if response?.headers}
          <span class="text-[10px] text-apple-ink-muted-48 font-mono">({Object.keys(response.headers).length})</span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => activeView = 'tests'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors flex items-center gap-1 cursor-pointer {activeView === 'tests' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        <span>Test Results</span>
        {#if response?.assertions && response.assertions.length > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded-apple-pill font-mono {testSummary.failed > 0 ? 'bg-rose-950/80 text-rose-400 border border-rose-800/80' : 'bg-emerald-950/80 text-emerald-400 border border-emerald-800/80'}">
            {summaryPassedText(testSummary)}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => activeView = 'timeline'}
        class="px-2.5 py-1 rounded-apple-sm text-xs font-medium transition-colors flex items-center gap-1 cursor-pointer {activeView === 'timeline' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
      >
        <span>Timeline</span>
        {#if timeline.length > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-body-muted border border-apple-hairline-dark/60 font-mono">
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
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm border border-apple-hairline-dark/80 bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-white text-xs font-bold transition-colors cursor-pointer"
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
          <div class="absolute right-0 top-full mt-1 w-48 bg-apple-surface-tile-2 border border-apple-hairline-dark rounded-apple-md shadow-none z-50 py-1 text-apple-body-on-dark text-xs font-sans">
            <button
              type="button"
              onclick={copyResponse}
              class="w-full text-left px-3 py-1.5 hover:bg-apple-surface-tile-3 flex items-center gap-2"
            >
              <span>📋</span> Copy Response Body
            </button>
            <button
              type="button"
              onclick={() => downloadResponse(true)}
              class="w-full text-left px-3 py-1.5 hover:bg-apple-surface-tile-3 flex items-center gap-2"
            >
              <span>💾</span> Save as JSON
            </button>
            <button
              type="button"
              onclick={() => downloadResponse(false)}
              class="w-full text-left px-3 py-1.5 hover:bg-apple-surface-tile-3 flex items-center gap-2"
            >
              <span>⬇</span> Download Raw (.txt)
            </button>
            <div class="border-t border-apple-hairline-dark/60 my-1"></div>
            {#if onToggleLayout}
              <button
                type="button"
                onclick={() => {
                  onToggleLayout();
                  isToolsMenuOpen = false;
                }}
                class="w-full text-left px-3 py-1.5 hover:bg-apple-surface-tile-3 flex items-center gap-2"
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
      <div class="h-full flex flex-col overflow-hidden">
        <!-- JSON Controls Sub-toolbar -->
        <div class="h-8 px-3 bg-apple-surface-tile-3 border-b border-apple-hairline-dark/60 flex items-center justify-between shrink-0 gap-2">
          <!-- View submode switcher (Code / Tree) -->
          <div class="flex items-center gap-1">
            <button
              type="button"
              onclick={() => jsonSubMode = 'code'}
              class="px-2 py-0.5 rounded-apple-sm text-[11px] font-medium transition-colors cursor-pointer {jsonSubMode === 'code' ? 'bg-apple-surface-tile-1 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
            >
              Code
            </button>
            <button
              type="button"
              onclick={() => jsonSubMode = 'tree'}
              class="px-2 py-0.5 rounded-apple-sm text-[11px] font-medium transition-colors cursor-pointer {jsonSubMode === 'tree' ? 'bg-apple-surface-tile-1 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
            >
              Tree
            </button>
          </div>

          <!-- Action buttons based on submode -->
          <div class="flex items-center gap-1.5">
            {#if jsonSubMode === 'code'}
              <button
                type="button"
                onclick={handleFormatJson}
                title="Prettify JSON (2-space indent)"
                class="px-2 py-0.5 rounded-apple-sm bg-apple-surface-tile-2 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
              >
                Format
              </button>
              <button
                type="button"
                onclick={handleCompressJson}
                title="Minify JSON"
                class="px-2 py-0.5 rounded-apple-sm bg-apple-surface-tile-2 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
              >
                Compress
              </button>
            {:else}
              <button
                type="button"
                onclick={handleExpandAllTree}
                title="Expand all tree nodes"
                class="px-2 py-0.5 rounded-apple-sm bg-apple-surface-tile-2 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer flex items-center gap-1"
              >
                <span>↕</span> Expand All
              </button>
              <button
                type="button"
                onclick={handleCollapseAllTree}
                title="Collapse all tree nodes"
                class="px-2 py-0.5 rounded-apple-sm bg-apple-surface-tile-2 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer flex items-center gap-1"
              >
                <span>↔</span> Collapse All
              </button>
            {/if}
          </div>
        </div>

        <!-- JSON Viewport Content -->
        <div class="flex-1 overflow-auto">
          {#if jsonSubMode === 'code'}
            <div class="p-3 font-mono text-slate-200 text-xs">
              <pre class="whitespace-pre-wrap break-all leading-relaxed">{response.body}</pre>
            </div>
          {:else}
            {#if parsedJson !== null}
              <JsonTreeView
                data={parsedJson}
                expandVersion={jsonExpandVersion}
                collapseVersion={jsonCollapseVersion}
              />
            {:else}
              <div class="p-6 text-center text-apple-ink-muted-48 italic">
                Invalid JSON body. Switch to "Code" view to inspect syntax.
              </div>
            {/if}
          {/if}
        </div>
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
