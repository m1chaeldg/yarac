<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';

  interface Props {
    activeView: 'studio' | 'runner';
    onSelectView: (view: 'studio' | 'runner') => void;
    onOpenInterop: () => void;
    onOpenConflict: () => void;
    onOpenEnvManager: () => void;
  }

  let { activeView, onSelectView, onOpenInterop, onOpenConflict, onOpenEnvManager }: Props = $props();

  let isAddingWorkspace = $state(false);
  let newWorkspaceName = $state('');
  let isAddingCollection = $state(false);
  let newCollectionName = $state('');

  function handleCreateWorkspace() {
    if (!newWorkspaceName.trim()) return;
    workspaceStore.addWorkspace(newWorkspaceName);
    newWorkspaceName = '';
    isAddingWorkspace = false;
  }

  function handleCreateCollection() {
    if (!newCollectionName.trim()) return;
    workspaceStore.addCollection(newCollectionName);
    newCollectionName = '';
    isAddingCollection = false;
  }

  function handleCreateRequest(collectionId: string) {
    workspaceStore.createRequest(collectionId, 'New Request', 'GET');
    onSelectView('studio');
  }

  function getMethodColor(method: string): string {
    switch (method) {
      case 'GET': return 'text-sky-400';
      case 'POST': return 'text-emerald-400';
      case 'PUT': return 'text-amber-400';
      case 'DELETE': return 'text-rose-400';
      case 'PATCH': return 'text-purple-400';
      default: return 'text-slate-400';
    }
  }
</script>

<div class="h-full flex flex-col justify-between bg-slate-900/60 select-none overflow-hidden text-xs">
  <div class="flex-1 flex flex-col overflow-hidden">
    <!-- Workspace Selector Bar -->
    <div class="h-12 border-b border-slate-800 px-3 flex items-center justify-between gap-2 shrink-0">
      <div class="flex-1 min-w-0">
        <select
          value={workspaceStore.currentWorkspace.id}
          onchange={(e) => workspaceStore.switchWorkspace((e.target as HTMLSelectElement).value)}
          class="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs font-semibold text-slate-100 truncate focus:outline-none focus:border-sky-500"
        >
          {#each workspaceStore.workspaces as ws (ws.id)}
            <option value={ws.id}>{ws.name}</option>
          {/each}
        </select>
      </div>

      <button
        type="button"
        title="Add new workspace"
        onclick={() => isAddingWorkspace = !isAddingWorkspace}
        class="p-1 rounded bg-slate-800 hover:bg-slate-700 text-sky-400 font-bold text-xs shrink-0"
      >
        +
      </button>
    </div>

    <!-- Inline Add Workspace Form -->
    {#if isAddingWorkspace}
      <div class="p-2 border-b border-slate-800 bg-slate-950/80 space-y-1.5 shrink-0">
        <input
          type="text"
          bind:value={newWorkspaceName}
          placeholder="New Workspace Name..."
          class="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs text-slate-100 focus:outline-none"
        />
        <div class="flex justify-end gap-1 text-[11px]">
          <button
            type="button"
            onclick={() => isAddingWorkspace = false}
            class="px-2 py-0.5 rounded text-slate-400 hover:bg-slate-800"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={handleCreateWorkspace}
            class="px-2 py-0.5 rounded bg-sky-600 hover:bg-sky-500 text-white font-medium"
          >
            Create
          </button>
        </div>
      </div>
    {/if}

    <!-- Main Navigation Switcher -->
    <div class="p-2 border-b border-slate-800/80 space-y-1 shrink-0">
      <button
        type="button"
        onclick={() => onSelectView('studio')}
        class="w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-2.5 transition-colors
          {activeView === 'studio'
            ? 'bg-sky-600/20 text-sky-300 border border-sky-500/30'
            : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
      >
        <span>⚡</span>
        <span>Request Studio</span>
      </button>

      <button
        type="button"
        onclick={() => onSelectView('runner')}
        class="w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-semibold flex items-center justify-between transition-colors
          {activeView === 'runner'
            ? 'bg-emerald-600/20 text-emerald-300 border border-emerald-500/30'
            : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
      >
        <div class="flex items-center gap-2.5">
          <span>📊</span>
          <span>Collection Runner</span>
        </div>
        <span class="text-[9px] px-1 rounded bg-emerald-950 text-emerald-400 font-mono border border-emerald-800">HDR</span>
      </button>
    </div>

    <!-- Collections & Requests Tree -->
    <div class="flex-1 overflow-y-auto p-2 space-y-3">
      <div class="flex items-center justify-between px-1">
        <span class="text-[10px] font-bold text-slate-500 uppercase tracking-wider">Collections</span>
        <button
          type="button"
          onclick={() => isAddingCollection = !isAddingCollection}
          class="text-[10px] text-sky-400 hover:text-sky-300 font-medium font-mono"
        >
          +folder
        </button>
      </div>

      {#if isAddingCollection}
        <div class="p-2 rounded bg-slate-950 border border-slate-800 space-y-1">
          <input
            type="text"
            bind:value={newCollectionName}
            placeholder="Collection Name..."
            class="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs text-slate-100 focus:outline-none"
          />
          <div class="flex justify-end gap-1 text-[11px]">
            <button
              type="button"
              onclick={() => isAddingCollection = false}
              class="px-2 py-0.5 rounded text-slate-400 hover:bg-slate-800"
            >
              Cancel
            </button>
            <button
              type="button"
              onclick={handleCreateCollection}
              class="px-2 py-0.5 rounded bg-sky-600 text-white font-medium"
            >
              Add
            </button>
          </div>
        </div>
      {/if}

      {#each workspaceStore.currentWorkspace.collections as col (col.id)}
        {@const colRequests = workspaceStore.currentWorkspace.requests.filter(r => r.collectionId === col.id)}
        <div class="space-y-0.5">
          <!-- Folder Header -->
          <div class="flex items-center justify-between px-2 py-1 rounded hover:bg-slate-800/40 text-slate-300 font-semibold group cursor-pointer">
            <div class="flex items-center gap-1.5 truncate">
              <span class="text-amber-400 text-xs">📁</span>
              <span class="truncate">{col.name}</span>
            </div>
            <button
              type="button"
              title="Add request to this collection"
              onclick={(e) => { e.stopPropagation(); handleCreateRequest(col.id); }}
              class="opacity-0 group-hover:opacity-100 text-[10px] text-sky-400 hover:text-sky-300 px-1 rounded hover:bg-slate-700"
            >
              +req
            </button>
          </div>

          <!-- Requests in this collection -->
          <div class="pl-3 space-y-0.5">
            {#each colRequests as req (req.id)}
              {@const isCurrent = workspaceStore.currentWorkspace.activeTabId === req.id}
              <div
                role="button"
                tabindex="0"
                onclick={() => {
                  workspaceStore.openTab(req.id);
                  onSelectView('studio');
                }}
                onkeydown={(e) => e.key === 'Enter' && (workspaceStore.openTab(req.id), onSelectView('studio'))}
                class="px-2 py-1.5 rounded flex items-center gap-2 cursor-pointer transition-colors text-xs
                  {isCurrent
                    ? 'bg-sky-950/60 text-sky-300 font-medium border border-sky-800/40'
                    : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
              >
                <span class="text-[9px] font-bold font-mono uppercase {getMethodColor(req.method)}">
                  {req.method}
                </span>
                <span class="truncate">{req.name}</span>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Bottom Workspace Controls -->
  <div class="p-2 border-t border-slate-800 bg-slate-950/40 text-xs space-y-1 shrink-0">
    <!-- Environment Profiles Shortcut -->
    <button
      type="button"
      onclick={onOpenEnvManager}
      class="w-full text-left px-2 py-1.5 rounded text-emerald-300 hover:bg-emerald-950/40 transition-colors flex items-center justify-between border border-transparent hover:border-emerald-800/40"
    >
      <div class="flex items-center gap-1.5 truncate">
        <span>🌱</span>
        <span class="truncate">
          Env: {workspaceStore.activeEnvironment?.name || 'None'}
        </span>
      </div>
      <span class="text-[10px] px-1 rounded bg-emerald-950 text-emerald-400 border border-emerald-800 font-mono">
        Edit
      </span>
    </button>

    <!-- Interop Hub -->
    <button
      type="button"
      onclick={onOpenInterop}
      class="w-full text-left px-2 py-1.5 rounded text-sky-300 hover:bg-sky-950/40 transition-colors flex items-center justify-between border border-transparent hover:border-sky-800/40"
    >
      <span>Interop (OpenAPI / Bruno)</span>
      <span class="text-[10px] px-1 rounded bg-sky-950 text-sky-400 border border-sky-800">3.1</span>
    </button>

    <!-- 3-Way Conflict Simulation -->
    <button
      type="button"
      onclick={onOpenConflict}
      class="w-full text-left px-2 py-1.5 rounded text-amber-300 hover:bg-amber-950/40 transition-colors flex items-center justify-between"
    >
      <span>Simulate 3-Way Conflict</span>
      <span class="text-[10px] px-1 rounded bg-amber-950 border border-amber-800">Test</span>
    </button>
  </div>
</div>
