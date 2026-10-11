<script lang="ts">
  interface Props {
    isOpen: boolean;
    filePath: string;
    baseContent: string;
    oursContent: string;
    theirsContent: string;
    onresolve: (resolvedContent: string) => void;
    oncancel: () => void;
  }

  let {
    isOpen = false,
    filePath = '',
    baseContent = '',
    oursContent = '',
    theirsContent = '',
    onresolve,
    oncancel
  }: Props = $props();

  let activeTab: 'ours' | 'theirs' | 'base' = $state('ours');

  function resolveWith(content: string) {
    onresolve(content);
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-6 animate-in fade-in duration-100">
    <div class="apple-modal-card max-w-4xl w-full max-h-[85vh] flex flex-col overflow-hidden text-apple-body-on-dark font-sans">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-apple-hairline-dark flex items-center justify-between bg-apple-surface-tile-2 shrink-0">
        <div>
          <h3 class="apple-body-strong text-apple-primary-on-dark flex items-center gap-2">
            <span>⚠️</span> External Conflict Detected (3-Way Merge)
          </h3>
          <p class="apple-fine-print text-apple-body-muted mt-1">
            File was modified externally on disk: <span class="font-mono text-white">{filePath}</span>
          </p>
        </div>
        <button
          type="button"
          onclick={oncancel}
          class="apple-button-icon-circular w-7 h-7 text-xs"
          title="Close conflict resolver"
        >
          ✕
        </button>
      </div>

      <!-- Diff Selector Tabs -->
      <div class="flex border-b border-apple-hairline-dark bg-apple-surface-tile-3 px-6 pt-2 gap-2 text-xs shrink-0">
        <button
          type="button"
          class="px-3 py-1.5 rounded-t-apple-sm font-semibold transition-colors {activeTab === 'ours' ? 'bg-apple-surface-tile-1 text-apple-primary-on-dark border-t-2 border-apple-primary' : 'text-apple-body-muted hover:text-white'}"
          onclick={() => activeTab = 'ours'}
        >
          Ours (Unsaved UI Changes)
        </button>
        <button
          type="button"
          class="px-3 py-1.5 rounded-t-apple-sm font-semibold transition-colors {activeTab === 'theirs' ? 'bg-apple-surface-tile-1 text-apple-primary-on-dark border-t-2 border-apple-primary' : 'text-apple-body-muted hover:text-white'}"
          onclick={() => activeTab = 'theirs'}
        >
          Theirs (Disk / External Git Change)
        </button>
        <button
          type="button"
          class="px-3 py-1.5 rounded-t-apple-sm font-semibold transition-colors {activeTab === 'base' ? 'bg-apple-surface-tile-1 text-white border-t-2 border-apple-hairline-dark' : 'text-apple-body-muted hover:text-white'}"
          onclick={() => activeTab = 'base'}
        >
          Base Version
        </button>
      </div>

      <!-- Diff Preview Viewport -->
      <div class="flex-1 overflow-auto p-4 bg-apple-surface-black font-mono text-xs text-apple-body-on-dark select-text">
        <pre class="whitespace-pre-wrap leading-relaxed">{#if activeTab === 'ours'}{oursContent}{:else if activeTab === 'theirs'}{theirsContent}{:else}{baseContent}{/if}</pre>
      </div>

      <!-- Resolution Actions -->
      <div class="px-6 py-3 border-t border-apple-hairline-dark flex justify-between items-center bg-apple-surface-tile-2 shrink-0">
        <span class="apple-fine-print text-apple-body-muted">Choose which version to preserve in the workspace.</span>
        <div class="flex gap-2.5">
          <button
            type="button"
            onclick={() => resolveWith(oursContent)}
            class="apple-button-primary text-xs px-4 py-1.5 font-semibold"
          >
            Keep Ours (Overwrite Disk)
          </button>
          <button
            type="button"
            onclick={() => resolveWith(theirsContent)}
            class="apple-button-secondary-pill text-xs px-4 py-1.5 font-semibold"
          >
            Accept Theirs (Discard UI)
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
