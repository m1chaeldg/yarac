<script lang="ts">
  import {
    formatJsonString,
    compressJsonString,
    searchJson,
    evaluateJsonPath,
    isJsonPathExpression,
    type JsonSearchMatch
  } from '../../utils/jsonUtils';
  import JsonTreeNode from './JsonTreeNode.svelte';

  interface Props {
    bodyText: string;
    parsedJson: any;
    onBodyChange?: (newBody: string) => void;
  }

  let { bodyText, parsedJson, onBodyChange }: Props = $props();

  let expandVersion = $state(0);
  let collapseVersion = $state(0);
  let actionFeedback = $state('');

  // Search and JSONPath states
  let searchQuery = $state('');
  let isFiltered = $state(false);

  let isJsonPath = $derived(isJsonPathExpression(searchQuery));

  let jsonPathResult = $derived.by(() => {
    if (!isJsonPath || !searchQuery.trim() || parsedJson === null || parsedJson === undefined) {
      return undefined;
    }
    try {
      return evaluateJsonPath(parsedJson, searchQuery);
    } catch {
      return undefined;
    }
  });

  let textMatches = $derived<JsonSearchMatch[]>(
    !isJsonPath && searchQuery.trim() && parsedJson !== null && parsedJson !== undefined
      ? searchJson(parsedJson, searchQuery)
      : []
  );

  let displayedJson = $derived.by(() => {
    if (isFiltered && isJsonPath && jsonPathResult !== undefined) {
      return jsonPathResult;
    }
    return parsedJson;
  });

  function handleFormat() {
    if (!bodyText) return;
    try {
      const formatted = formatJsonString(bodyText, 2);
      if (onBodyChange) onBodyChange(formatted);
      flashFeedback('Prettified JSON');
    } catch {
      flashFeedback('Invalid JSON');
    }
  }

  function handleCompress() {
    if (!bodyText) return;
    try {
      const compressed = compressJsonString(bodyText);
      if (onBodyChange) onBodyChange(compressed);
      flashFeedback('Compressed JSON');
    } catch {
      flashFeedback('Invalid JSON');
    }
  }

  function handleExpandAll() {
    expandVersion += 1;
    flashFeedback('Expanded all');
  }

  function handleCollapseAll() {
    collapseVersion += 1;
    flashFeedback('Collapsed all');
  }

  function handleCopy() {
    const textToCopy = isFiltered && jsonPathResult !== undefined
      ? JSON.stringify(jsonPathResult, null, 2)
      : bodyText;
    if (!textToCopy) return;
    navigator.clipboard.writeText(textToCopy);
    flashFeedback('Copied JSON');
  }

  function clearSearch() {
    searchQuery = '';
    isFiltered = false;
  }

  function toggleFilter() {
    isFiltered = !isFiltered;
    if (isFiltered) {
      expandVersion += 1;
    }
  }

  function flashFeedback(msg: string) {
    actionFeedback = msg;
    setTimeout(() => {
      actionFeedback = '';
    }, 2000);
  }
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <!-- Search & JSONPath Toolbar -->
  <div class="h-9 px-3 bg-apple-surface-tile-2 border-b border-apple-hairline-dark/60 flex items-center justify-between shrink-0 gap-2 select-none">
    <div class="flex items-center gap-2 flex-1 max-w-xl">
      <!-- Search Input Container -->
      <div class="relative flex-1 flex items-center">
        <span class="absolute left-2.5 text-apple-ink-muted-48 pointer-events-none text-xs">🔍</span>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search keys, values, or JSONPath (e.g. $.items[*].name)..."
          class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-sm pl-8 pr-16 py-1 text-xs text-apple-body-on-dark font-mono placeholder:text-apple-ink-muted-48 focus:outline-none focus:border-apple-primary"
        />
        {#if searchQuery}
          <button
            type="button"
            onclick={clearSearch}
            title="Clear search"
            class="absolute right-2 text-apple-ink-muted-48 hover:text-white px-1 text-xs cursor-pointer"
          >
            ✕
          </button>
        {/if}
      </div>

      <!-- Match indicators -->
      {#if searchQuery.trim()}
        {#if isJsonPath}
          {#if jsonPathResult !== undefined}
            <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-sky-950 text-sky-300 border border-sky-800 font-mono shrink-0">
              JSONPath match
            </span>
            <button
              type="button"
              onclick={toggleFilter}
              class="px-2 py-0.5 rounded-apple-sm text-[10px] font-semibold transition-colors cursor-pointer shrink-0 {isFiltered ? 'bg-apple-primary text-white' : 'bg-apple-surface-tile-3 text-apple-body-muted hover:text-white border border-apple-hairline-dark/60'}"
            >
              {isFiltered ? 'Show Full' : 'Filter View'}
            </button>
          {:else}
            <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-rose-950 text-rose-300 border border-rose-800 font-mono shrink-0">
              No JSONPath match
            </span>
          {/if}
        {:else}
          <span class="px-2 py-0.5 rounded-apple-pill text-[10px] font-mono shrink-0 {textMatches.length > 0 ? 'bg-amber-950 text-amber-300 border border-amber-800' : 'bg-apple-surface-tile-3 text-apple-ink-muted-48 border border-apple-hairline-dark/60'}">
            {textMatches.length} {textMatches.length === 1 ? 'match' : 'matches'}
          </span>
        {/if}
      {/if}
    </div>

    <!-- Actions & Tree Navigation -->
    <div class="flex items-center gap-1.5 shrink-0">
      <button
        type="button"
        onclick={handleExpandAll}
        title="Expand all tree nodes"
        class="px-2 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
      >
        Expand
      </button>

      <button
        type="button"
        onclick={handleCollapseAll}
        title="Collapse all tree nodes"
        class="px-2 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
      >
        Collapse
      </button>

      <div class="h-4 w-px bg-apple-hairline-dark/80 mx-0.5"></div>

      <button
        type="button"
        onclick={handleFormat}
        title="Prettify JSON with 2-space indentation"
        class="px-2 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
      >
        Format
      </button>

      <button
        type="button"
        onclick={handleCompress}
        title="Minify JSON compact string"
        class="px-2 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
      >
        Compress
      </button>

      <button
        type="button"
        onclick={handleCopy}
        title="Copy JSON to clipboard"
        class="px-2 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-ink-muted-48 hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer flex items-center gap-1"
      >
        <span>📋</span>
      </button>

      {#if actionFeedback}
        <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-apple-primary/20 text-apple-primary-on-dark border border-apple-primary/40 font-mono animate-fade-in">
          ✓ {actionFeedback}
        </span>
      {/if}
    </div>
  </div>

  <!-- Interactive JSON Tree Viewport -->
  <div class="flex-1 overflow-auto p-3 font-mono">
    {#if displayedJson !== null && displayedJson !== undefined}
      <div class="space-y-0.5">
        <JsonTreeNode
          value={displayedJson}
          depth={0}
          {expandVersion}
          {collapseVersion}
          defaultExpanded={true}
          isLast={true}
          searchQuery={isJsonPath ? '' : searchQuery}
        />
      </div>
    {:else}
      <!-- Graceful fallback for non-valid JSON -->
      <div class="p-6 text-center space-y-2">
        <p class="text-amber-400 font-semibold text-xs">⚠️ Response body is not valid JSON</p>
        <p class="text-[11px] text-apple-ink-muted-48">Raw content displayed below:</p>
        <div class="mt-3 p-3 bg-apple-surface-tile-1 border border-apple-hairline-dark/60 rounded-apple-sm text-left">
          <pre class="whitespace-pre-wrap break-all text-apple-body-on-dark text-xs">{bodyText}</pre>
        </div>
      </div>
    {/if}
  </div>
</div>
