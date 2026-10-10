<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';

  interface Props {
    onNewRequest?: () => void;
  }

  let { onNewRequest }: Props = $props();

  function getMethodColor(method: string): string {
    switch (method) {
      case 'GET': return 'text-sky-400 bg-sky-950/80 border-sky-800/60';
      case 'POST': return 'text-emerald-400 bg-emerald-950/80 border-emerald-800/60';
      case 'PUT': return 'text-amber-400 bg-amber-950/80 border-amber-800/60';
      case 'DELETE': return 'text-rose-400 bg-rose-950/80 border-rose-800/60';
      case 'PATCH': return 'text-purple-400 bg-purple-950/80 border-purple-800/60';
      default: return 'text-slate-400 bg-slate-800 border-slate-700';
    }
  }

  function handleTabClick(reqId: string) {
    workspaceStore.setActiveTab(reqId);
  }

  function handleCloseTab(e: MouseEvent, reqId: string) {
    e.stopPropagation();
    workspaceStore.closeTab(reqId);
  }
</script>

<div class="h-10 border-b border-slate-800 bg-slate-950 flex items-center justify-between px-2 overflow-x-auto select-none no-scrollbar">
  <div class="flex items-center gap-1 min-w-0">
    {#each workspaceStore.openRequests as req (req.id)}
      {@const isActive = workspaceStore.currentWorkspace.activeTabId === req.id}
      <div
        role="tab"
        tabindex="0"
        aria-selected={isActive}
        onclick={() => handleTabClick(req.id)}
        onkeydown={(e) => e.key === 'Enter' && handleTabClick(req.id)}
        class="group relative flex items-center gap-2 px-3 py-1.5 rounded-t-md text-xs font-medium cursor-pointer transition-all border-t border-x border-b-transparent shrink-0
          {isActive
            ? 'bg-slate-900 text-slate-100 border-slate-700 shadow-sm border-b-slate-900 -mb-px z-10'
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/40 border-transparent'}"
      >
        <!-- Method Pill -->
        <span class="text-[9px] font-bold px-1 rounded border uppercase font-mono {getMethodColor(req.method)}">
          {req.method}
        </span>

        <!-- Tab Title -->
        <span class="truncate max-w-[130px]">{req.name}</span>

        <!-- Dirty Indicator -->
        {#if req.isDirty}
          <span class="w-1.5 h-1.5 rounded-full bg-amber-400 shrink-0" title="Unsaved changes"></span>
        {/if}

        <!-- Close Button -->
        <button
          type="button"
          aria-label="Close tab"
          onclick={(e) => handleCloseTab(e, req.id)}
          class="opacity-0 group-hover:opacity-100 hover:bg-slate-800 rounded p-0.5 text-slate-400 hover:text-slate-100 transition-opacity ml-1"
        >
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/each}

    <!-- New Request Tab Button -->
    <button
      type="button"
      onclick={onNewRequest}
      title="Create new request in active collection"
      class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-sky-300 transition-colors ml-1"
    >
      <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
</div>
