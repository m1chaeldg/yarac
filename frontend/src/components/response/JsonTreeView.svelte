<script lang="ts">
  import { formatJsonString, compressJsonString } from '../../utils/jsonUtils';
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
    if (!bodyText) return;
    navigator.clipboard.writeText(bodyText);
    flashFeedback('Copied JSON');
  }

  function flashFeedback(msg: string) {
    actionFeedback = msg;
    setTimeout(() => {
      actionFeedback = '';
    }, 2000);
  }
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <!-- Unified JSON Actions Toolbar -->
  <div class="h-9 px-3 bg-apple-surface-tile-2 border-b border-apple-hairline-dark/60 flex items-center justify-between shrink-0 gap-2 select-none">
    <!-- Tree Navigation & Structure Controls -->
    <div class="flex items-center gap-1.5">
      <button
        type="button"
        onclick={handleExpandAll}
        title="Expand all tree nodes"
        class="px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer flex items-center gap-1.5"
      >
        <span>↕</span> Expand All
      </button>

      <button
        type="button"
        onclick={handleCollapseAll}
        title="Collapse all tree nodes"
        class="px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer flex items-center gap-1.5"
      >
        <span>↔</span> Collapse All
      </button>

      <div class="h-4 w-px bg-apple-hairline-dark/80 mx-1"></div>

      <!-- JSON Transform Actions -->
      <button
        type="button"
        onclick={handleFormat}
        title="Prettify JSON with 2-space indentation"
        class="px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
      >
        Format
      </button>

      <button
        type="button"
        onclick={handleCompress}
        title="Minify JSON compact string"
        class="px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer"
      >
        Compress
      </button>
    </div>

    <!-- Right Controls: Feedback & Quick Copy -->
    <div class="flex items-center gap-2">
      {#if actionFeedback}
        <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-apple-primary/20 text-apple-primary-on-dark border border-apple-primary/40 font-mono animate-fade-in">
          ✓ {actionFeedback}
        </span>
      {/if}

      <button
        type="button"
        onclick={handleCopy}
        title="Copy raw JSON to clipboard"
        class="px-2 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-ink-muted-48 hover:text-white border border-apple-hairline-dark/80 text-[11px] font-medium transition-colors cursor-pointer flex items-center gap-1"
      >
        <span>📋</span> Copy
      </button>
    </div>
  </div>

  <!-- Interactive JSON Tree Viewport -->
  <div class="flex-1 overflow-auto p-3 font-mono">
    {#if parsedJson !== null && parsedJson !== undefined}
      <div class="space-y-0.5">
        <JsonTreeNode
          value={parsedJson}
          depth={0}
          {expandVersion}
          {collapseVersion}
          defaultExpanded={true}
          isLast={true}
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
