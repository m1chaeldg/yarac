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
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-6">
    <div class="bg-slate-900 border border-slate-700 rounded-xl shadow-2xl max-w-4xl w-full max-h-[85vh] flex flex-col overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/50">
        <div>
          <h3 class="text-lg font-bold text-amber-400 flex items-center gap-2">
            <span>⚠️</span> External Conflict Detected (3-Way Merge)
          </h3>
          <p class="text-xs text-slate-400 mt-0.5">
            File was modified externally on disk: <span class="font-mono text-slate-300">{filePath}</span>
          </p>
        </div>
        <button
          onclick={oncancel}
          class="text-slate-400 hover:text-slate-200 text-sm px-2 py-1 rounded"
        >
          ✕
        </button>
      </div>

      <!-- Diff Selector Tabs -->
      <div class="flex border-b border-slate-800 bg-slate-950/30 px-6 pt-2 gap-2 text-xs">
        <button
          class="px-3 py-1.5 rounded-t font-medium transition-colors {activeTab === 'ours' ? 'bg-slate-800 text-sky-400 border-t-2 border-sky-400' : 'text-slate-400 hover:text-slate-300'}"
          onclick={() => activeTab = 'ours'}
        >
          Ours (Unsaved UI Changes)
        </button>
        <button
          class="px-3 py-1.5 rounded-t font-medium transition-colors {activeTab === 'theirs' ? 'bg-slate-800 text-emerald-400 border-t-2 border-emerald-400' : 'text-slate-400 hover:text-slate-300'}"
          onclick={() => activeTab = 'theirs'}
        >
          Theirs (Disk / External Git Change)
        </button>
        <button
          class="px-3 py-1.5 rounded-t font-medium transition-colors {activeTab === 'base' ? 'bg-slate-800 text-slate-300 border-t-2 border-slate-400' : 'text-slate-400 hover:text-slate-300'}"
          onclick={() => activeTab = 'base'}
        >
          Base Version
        </button>
      </div>

      <!-- Diff Preview Viewport -->
      <div class="flex-1 overflow-auto p-4 bg-slate-950 font-mono text-xs text-slate-300 select-text">
        <pre class="whitespace-pre-wrap">{#if activeTab === 'ours'}{oursContent}{:else if activeTab === 'theirs'}{theirsContent}{:else}{baseContent}{/if}</pre>
      </div>

      <!-- Resolution Actions -->
      <div class="px-6 py-3 border-t border-slate-800 flex justify-between items-center bg-slate-950/70">
        <span class="text-xs text-slate-400">Choose which version to preserve in the workspace.</span>
        <div class="flex gap-2.5">
          <button
            onclick={() => resolveWith(oursContent)}
            class="px-3.5 py-1.5 rounded text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white transition-colors"
          >
            Keep Ours (Overwrite Disk)
          </button>
          <button
            onclick={() => resolveWith(theirsContent)}
            class="px-3.5 py-1.5 rounded text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white transition-colors"
          >
            Accept Theirs (Discard UI)
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
