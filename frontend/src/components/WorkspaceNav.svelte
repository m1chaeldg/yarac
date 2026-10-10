<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';
  import ContextMenu, { type ContextMenuItem } from './ContextMenu.svelte';
  import CodeGeneratorModal from './CodeGeneratorModal.svelte';
  import RequestInfoModal from './RequestInfoModal.svelte';
  import { resolveUrl, CODE_LANGUAGES, getRelativeRequestPath } from '../utils/codeGenerators';

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

  // Context Menu & Modal States
  let contextMenu = $state<{ x: number; y: number; req: YaracRequest; col: any } | null>(null);
  let codeGenReq = $state<YaracRequest | null>(null);
  let infoReq = $state<{ req: YaracRequest; col: any } | null>(null);
  let renameReq = $state<{ req: YaracRequest; newName: string } | null>(null);
  let deleteReq = $state<YaracRequest | null>(null);
  let toast = $state<{ text: string; icon: string } | null>(null);

  // Drag-and-Drop Reordering States
  let draggedReqId = $state<string | null>(null);
  let draggedColId = $state<string | null>(null);
  let dragOverReqId = $state<string | null>(null);
  let dragOverColId = $state<string | null>(null);
  let dropPosition = $state<'before' | 'after' | 'inside' | null>(null);

  function handleReqDragStart(e: DragEvent, reqId: string) {
    draggedReqId = reqId;
    draggedColId = null;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', reqId);
    }
  }

  function handleReqDragOver(e: DragEvent, targetReqId: string) {
    if (!draggedReqId || draggedReqId === targetReqId) return;
    e.preventDefault();
    dragOverReqId = targetReqId;
    dragOverColId = null;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dropPosition = e.clientY < rect.top + rect.height / 2 ? 'before' : 'after';
  }

  function handleReqDrop(e: DragEvent, targetReq: YaracRequest, targetColId: string) {
    e.preventDefault();
    e.stopPropagation();
    if (!draggedReqId || draggedReqId === targetReq.id) {
      handleDragEnd();
      return;
    }

    const ws = workspaceStore.currentWorkspace;
    const sourceReq = ws.requests.find(r => r.id === draggedReqId);
    if (!sourceReq) return;

    const colReqs = ws.requests.filter(r => r.collectionId === targetColId);
    let targetIndex = colReqs.findIndex(r => r.id === targetReq.id);
    if (targetIndex === -1) targetIndex = 0;
    if (dropPosition === 'after') {
      targetIndex += 1;
    }

    if (sourceReq.collectionId === targetColId) {
      const fromIndex = colReqs.findIndex(r => r.id === draggedReqId);
      if (fromIndex !== -1) {
        const adjustedTarget = fromIndex < targetIndex ? targetIndex - 1 : targetIndex;
        workspaceStore.reorderRequests(targetColId, fromIndex, adjustedTarget);
        showToast('Request reordered', '↕️');
      }
    } else {
      workspaceStore.moveRequestToCollection(draggedReqId, targetColId, targetIndex);
      showToast('Moved to collection', '📁');
    }

    handleDragEnd();
  }

  function handleColHeaderDragStart(e: DragEvent, colId: string) {
    draggedColId = colId;
    draggedReqId = null;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', colId);
    }
  }

  function handleColHeaderDragOver(e: DragEvent, colId: string) {
    e.preventDefault();
    if (draggedReqId) {
      dragOverColId = colId;
      dragOverReqId = null;
      dropPosition = 'inside';
    } else if (draggedColId && draggedColId !== colId) {
      dragOverColId = colId;
      dragOverReqId = null;
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      dropPosition = e.clientY < rect.top + rect.height / 2 ? 'before' : 'after';
    }
  }

  function handleColHeaderDrop(e: DragEvent, targetColId: string) {
    e.preventDefault();
    e.stopPropagation();

    if (draggedReqId) {
      workspaceStore.moveRequestToCollection(draggedReqId, targetColId);
      showToast('Moved into collection', '📁');
    } else if (draggedColId && draggedColId !== targetColId) {
      const cols = workspaceStore.currentWorkspace.collections;
      const fromIdx = cols.findIndex(c => c.id === draggedColId);
      let toIdx = cols.findIndex(c => c.id === targetColId);
      if (fromIdx !== -1 && toIdx !== -1) {
        if (dropPosition === 'after') toIdx += 1;
        const adjustedTarget = fromIdx < toIdx ? toIdx - 1 : toIdx;
        workspaceStore.reorderCollections(fromIdx, adjustedTarget);
        showToast('Collection reordered', '↕️');
      }
    }

    handleDragEnd();
  }

  function handleDragEnd() {
    draggedReqId = null;
    draggedColId = null;
    dragOverColId = null;
    dragOverReqId = null;
    dropPosition = null;
  }

  function showToast(text: string, icon = '✓') {
    toast = { text, icon };
    setTimeout(() => {
      if (toast?.text === text) {
        toast = null;
      }
    }, 2500);
  }

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

  function handleCommitRename() {
    if (!renameReq || !renameReq.newName.trim()) return;
    workspaceStore.renameRequest(renameReq.req.id, renameReq.newName);
    showToast(`Renamed to "${renameReq.newName.trim()}"`, '✏️');
    renameReq = null;
  }

  function handleCommitDelete() {
    if (!deleteReq) return;
    const name = deleteReq.name;
    workspaceStore.deleteRequest(deleteReq.id);
    showToast(`Deleted "${name}"`, '🗑️');
    deleteReq = null;
  }

  function getContextMenuItems(req: YaracRequest, col: any): ContextMenuItem[] {
    return [
      {
        id: 'rename',
        label: 'Rename',
        icon: '✏️',
        shortcut: 'F2',
        action: () => {
          renameReq = { req, newName: req.name };
        }
      },
      {
        id: 'duplicate',
        label: 'Clone / Duplicate',
        icon: '📄',
        shortcut: 'Ctrl+D',
        action: () => {
          const cloned = workspaceStore.duplicateRequest(req.id);
          if (cloned) {
            showToast(`Cloned as "${cloned.name}"`, '📄');
            onSelectView('studio');
          }
        }
      },
      {
        id: 'copy-curl',
        label: 'Copy as cURL',
        icon: '⚡',
        action: async () => {
          const curlLang = CODE_LANGUAGES.find(l => l.id === 'curl');
          const code = curlLang?.generate(req, 'bash') || '';
          try {
            await navigator.clipboard.writeText(code);
            showToast('cURL copied to clipboard!', '⚡');
          } catch (e) {
            console.error(e);
          }
        }
      },
      {
        id: 'copy-url',
        label: 'Copy URL',
        icon: '🔗',
        action: async () => {
          const url = resolveUrl(req);
          try {
            await navigator.clipboard.writeText(url);
            showToast('URL copied to clipboard!', '🔗');
          } catch (e) {
            console.error(e);
          }
        }
      },
      {
        id: 'copy-dsl',
        label: 'Copy .yarac DSL',
        icon: '📋',
        action: async () => {
          try {
            await navigator.clipboard.writeText(req.dslContent);
            showToast('.yarac DSL copied to clipboard!', '📋');
          } catch (e) {
            console.error(e);
          }
        }
      },
      {
        id: 'reveal',
        label: 'Reveal in Explorer',
        icon: '📂',
        action: async () => {
          const relPath = getRelativeRequestPath(col.name, req.name);
          try {
            await navigator.clipboard.writeText(relPath);
            showToast(`Path copied: ${relPath}`, '📂');
          } catch {
            showToast(`File: ${relPath}`, '📂');
          }
        }
      },
      { id: 'sep1', label: '', separator: true, action: () => {} },
      {
        id: 'generate-code',
        label: 'Generate Code...',
        icon: '🚀',
        shortcut: 'Ctrl+G',
        action: () => {
          codeGenReq = req;
        }
      },
      {
        id: 'info',
        label: 'Request Info',
        icon: 'ℹ️',
        action: () => {
          infoReq = { req, col };
        }
      },
      { id: 'sep_dnd', label: '', separator: true, action: () => {} },
      {
        id: 'move-up',
        label: 'Move Up',
        icon: '⬆️',
        shortcut: 'Alt+Up',
        action: () => {
          const colReqs = workspaceStore.currentWorkspace.requests.filter(r => r.collectionId === col.id);
          const idx = colReqs.findIndex(r => r.id === req.id);
          if (idx > 0) {
            workspaceStore.reorderRequests(col.id, idx, idx - 1);
            showToast('Moved request up', '⬆️');
          }
        }
      },
      {
        id: 'move-down',
        label: 'Move Down',
        icon: '⬇️',
        shortcut: 'Alt+Down',
        action: () => {
          const colReqs = workspaceStore.currentWorkspace.requests.filter(r => r.collectionId === col.id);
          const idx = colReqs.findIndex(r => r.id === req.id);
          if (idx >= 0 && idx < colReqs.length - 1) {
            workspaceStore.reorderRequests(col.id, idx, idx + 1);
            showToast('Moved request down', '⬇️');
          }
        }
      },
      { id: 'sep2', label: '', separator: true, action: () => {} },
      {
        id: 'delete',
        label: 'Delete Request',
        icon: '🗑️',
        danger: true,
        shortcut: 'Del',
        action: () => {
          deleteReq = req;
        }
      }
    ];
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

      {#each workspaceStore.currentWorkspace.collections as col, colIdx (col.id)}
        {@const colRequests = workspaceStore.currentWorkspace.requests.filter(r => r.collectionId === col.id)}
        {@const isColOver = dragOverColId === col.id}
        <div class="space-y-0.5">
          <!-- Folder Header with Drag and Drop -->
          <div
            role="button"
            tabindex="0"
            draggable="true"
            ondragstart={(e) => handleColHeaderDragStart(e, col.id)}
            ondragover={(e) => handleColHeaderDragOver(e, col.id)}
            ondrop={(e) => handleColHeaderDrop(e, col.id)}
            ondragend={handleDragEnd}
            class="flex items-center justify-between px-2 py-1 rounded transition-colors text-slate-300 font-semibold group cursor-grab active:cursor-grabbing select-none
              {draggedColId === col.id ? 'opacity-40 border border-dashed border-sky-400' : ''}
              {isColOver && dropPosition === 'inside' ? 'bg-sky-950/80 ring-1 ring-sky-400 text-sky-200' : 'hover:bg-slate-800/40'}
              {isColOver && dropPosition === 'before' ? 'border-t-2 border-sky-400' : ''}
              {isColOver && dropPosition === 'after' ? 'border-b-2 border-sky-400' : ''}"
          >
            <div class="flex items-center gap-1.5 truncate">
              <span class="text-slate-500 opacity-40 group-hover:opacity-100 text-[10px]">⋮⋮</span>
              <span class="text-amber-400 text-xs">📁</span>
              <span class="truncate">{col.name}</span>
            </div>
            <div class="flex items-center gap-1">
              <span class="text-[10px] text-slate-500 font-mono">{colRequests.length}</span>
              <button
                type="button"
                title="Add request to this collection"
                onclick={(e) => { e.stopPropagation(); handleCreateRequest(col.id); }}
                class="opacity-0 group-hover:opacity-100 text-[10px] text-sky-400 hover:text-sky-300 px-1 rounded hover:bg-slate-700"
              >
                +req
              </button>
            </div>
          </div>

          <!-- Requests in this collection -->
          <div class="pl-3 space-y-0.5">
            {#each colRequests as req, reqIdx (req.id)}
              {@const isCurrent = workspaceStore.currentWorkspace.activeTabId === req.id}
              {@const isReqOver = dragOverReqId === req.id}
              {@const isReqDragged = draggedReqId === req.id}
              <div
                role="button"
                tabindex="0"
                draggable="true"
                ondragstart={(e) => handleReqDragStart(e, req.id)}
                ondragover={(e) => handleReqDragOver(e, req.id)}
                ondrop={(e) => handleReqDrop(e, req, col.id)}
                ondragend={handleDragEnd}
                onclick={() => {
                  workspaceStore.openTab(req.id);
                  onSelectView('studio');
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter') {
                    workspaceStore.openTab(req.id);
                    onSelectView('studio');
                  } else if (e.key === 'F2') {
                    renameReq = { req, newName: req.name };
                  } else if (e.key === 'Delete') {
                    deleteReq = req;
                  } else if (e.altKey && e.key === 'ArrowUp') {
                    e.preventDefault();
                    if (reqIdx > 0) {
                      workspaceStore.reorderRequests(col.id, reqIdx, reqIdx - 1);
                      showToast('Moved request up', '⬆️');
                    }
                  } else if (e.altKey && e.key === 'ArrowDown') {
                    e.preventDefault();
                    if (reqIdx < colRequests.length - 1) {
                      workspaceStore.reorderRequests(col.id, reqIdx, reqIdx + 1);
                      showToast('Moved request down', '⬇️');
                    }
                  }
                }}
                oncontextmenu={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  contextMenu = {
                    x: e.clientX,
                    y: e.clientY,
                    req,
                    col
                  };
                }}
                class="px-2 py-1.5 rounded flex items-center justify-between group cursor-grab active:cursor-grabbing transition-all text-xs select-none
                  {isReqDragged ? 'opacity-30 border border-dashed border-sky-400 bg-sky-950/20' : ''}
                  {isReqOver && dropPosition === 'before' ? 'border-t-2 border-sky-400' : ''}
                  {isReqOver && dropPosition === 'after' ? 'border-b-2 border-sky-400' : ''}
                  {isCurrent
                    ? 'bg-sky-950/60 text-sky-300 font-medium border border-sky-800/40'
                    : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
              >
                <div class="flex items-center gap-1.5 truncate">
                  <span class="text-slate-600 opacity-20 group-hover:opacity-80 text-[10px]">⋮⋮</span>
                  <span class="text-[9px] font-bold font-mono uppercase {getMethodColor(req.method)}">
                    {req.method}
                  </span>
                  <span class="truncate">{req.name}</span>
                </div>

                <button
                  type="button"
                  title="Request menu"
                  onclick={(e) => {
                    e.stopPropagation();
                    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                    contextMenu = { x: rect.right, y: rect.bottom, req, col };
                  }}
                  class="opacity-0 group-hover:opacity-100 hover:text-white px-1 py-0.5 rounded text-[10px] text-slate-500 hover:bg-slate-700/60"
                >
                  •••
                </button>
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

<!-- Floating Context Menu -->
{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={getContextMenuItems(contextMenu.req, contextMenu.col)}
    onClose={() => contextMenu = null}
  />
{/if}

<!-- Code Generator Modal -->
<CodeGeneratorModal
  isOpen={Boolean(codeGenReq)}
  request={codeGenReq}
  onClose={() => codeGenReq = null}
/>

<!-- Request Info Modal -->
<RequestInfoModal
  isOpen={Boolean(infoReq)}
  request={infoReq ? infoReq.req : null}
  collection={infoReq ? infoReq.col : null}
  onClose={() => infoReq = null}
/>

<!-- Rename Request Dialog -->
{#if renameReq}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-100"
    onclick={() => renameReq = null}
    role="presentation"
  >
    <div
      class="bg-slate-900 border border-slate-700 rounded-xl p-4 w-80 shadow-2xl space-y-3"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
          <span>✏️</span>
          <span>Rename Request</span>
        </h3>
        <button
          type="button"
          onclick={() => renameReq = null}
          class="text-slate-400 hover:text-slate-200 text-xs"
        >
          ✕
        </button>
      </div>

      <input
        type="text"
        bind:value={renameReq.newName}
        onkeydown={(e) => {
          if (e.key === 'Enter') handleCommitRename();
          if (e.key === 'Escape') renameReq = null;
        }}
        class="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-slate-100 focus:outline-none focus:border-sky-500"
      />

      <div class="flex justify-end gap-2 text-xs">
        <button
          type="button"
          onclick={() => renameReq = null}
          class="px-2.5 py-1 rounded text-slate-400 hover:bg-slate-800"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCommitRename}
          class="px-3 py-1 rounded bg-sky-600 hover:bg-sky-500 text-white font-medium"
        >
          Save
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete Request Confirmation Dialog -->
{#if deleteReq}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-100"
    onclick={() => deleteReq = null}
    role="presentation"
  >
    <div
      class="bg-slate-900 border border-rose-900/60 rounded-xl p-4 w-80 shadow-2xl space-y-3"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center gap-2 text-rose-400 text-xs font-bold">
        <span>⚠️</span>
        <span>Delete Request</span>
      </div>

      <p class="text-xs text-slate-300">
        Are you sure you want to delete <span class="font-semibold text-slate-100">"{deleteReq.name}"</span>? This will remove the request and close its tab.
      </p>

      <div class="flex justify-end gap-2 text-xs">
        <button
          type="button"
          onclick={() => deleteReq = null}
          class="px-2.5 py-1 rounded text-slate-400 hover:bg-slate-800"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCommitDelete}
          class="px-3 py-1 rounded bg-rose-600 hover:bg-rose-500 text-white font-medium"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Toast Feedback -->
{#if toast}
  <div class="fixed bottom-4 right-4 z-50 px-3.5 py-2 rounded-lg bg-slate-800/95 border border-slate-700 text-slate-100 text-xs font-medium shadow-2xl flex items-center gap-2 animate-in fade-in slide-in-from-bottom-2 duration-150 backdrop-blur-md">
    <span>{toast.icon}</span>
    <span>{toast.text}</span>
  </div>
{/if}
