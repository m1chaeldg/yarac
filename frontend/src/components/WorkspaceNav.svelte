<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';
  import { backendApi } from '../services/backendApi';
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
  let addingSubfolderParentId = $state<string | null>(null);
  let newSubfolderName = $state('');

  // Context Menu & Modal States
  let contextMenu = $state<{ x: number; y: number; req: YaracRequest; col: any } | null>(null);
  let colContextMenu = $state<{ x: number; y: number; col: any } | null>(null);
  let codeGenReq = $state<YaracRequest | null>(null);
  let infoReq = $state<{ req: YaracRequest; col: any } | null>(null);
  let renameReq = $state<{ req: YaracRequest; newName: string } | null>(null);
  let deleteReq = $state<YaracRequest | null>(null);
  let renameCol = $state<{ col: any; newName: string } | null>(null);
  let deleteCol = $state<any | null>(null);
  let toast = $state<{ text: string; icon: string } | null>(null);

  // Hierarchical Collection Tree Structure
  export interface TreeNode {
    col: any;
    level: number;
    subfolders: TreeNode[];
    requests: YaracRequest[];
  }

  let collectionTree = $derived.by(() => {
    const ws = workspaceStore.currentWorkspace;
    const cols = ws.collections;
    const reqs = ws.requests;

    function buildNode(col: any, level: number): TreeNode {
      const subcols = cols.filter(c => c.parentId === col.id);
      const colReqs = reqs.filter(r => r.collectionId === col.id);
      return {
        col,
        level,
        subfolders: subcols.map(sc => buildNode(sc, level + 1)),
        requests: colReqs
      };
    }

    // Root collections are those without parentId, or whose parentId is not in cols
    const colIds = new Set(cols.map(c => c.id));
    const rootCols = cols.filter(c => !c.parentId || !colIds.has(c.parentId));
    return rootCols.map(rc => buildNode(rc, 0));
  });

  // Collapsible Tree View States
  let collapsedColIds = $state<Record<string, boolean>>({});

  function isColCollapsed(colId: string): boolean {
    return !!collapsedColIds[colId];
  }

  function toggleColCollapse(colId: string) {
    collapsedColIds[colId] = !collapsedColIds[colId];
  }

  function expandAll() {
    collapsedColIds = {};
    showToast('Expanded all collections', '📂');
  }

  function collapseAll() {
    const newMap: Record<string, boolean> = {};
    for (const col of workspaceStore.currentWorkspace.collections) {
      newMap[col.id] = true;
    }
    collapsedColIds = newMap;
    showToast('Collapsed all collections', '📁');
  }

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
    if (!draggedReqId && (!draggedColId || draggedColId === colId)) return;
    e.preventDefault();

    if (draggedReqId) {
      dragOverColId = colId;
      dragOverReqId = null;
      dropPosition = 'inside';
    } else if (draggedColId && draggedColId !== colId) {
      // Disallow dragging into any of its own descendants to prevent cyclic parent-child links
      if (workspaceStore.isCollectionDescendant(draggedColId, colId)) {
        return;
      }

      dragOverColId = colId;
      dragOverReqId = null;
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const relativeY = e.clientY - rect.top;
      const height = rect.height;

      // 3-zone calculation: top 25% = before, bottom 25% = after, middle 50% = inside
      if (relativeY < height * 0.25) {
        dropPosition = 'before';
      } else if (relativeY > height * 0.75) {
        dropPosition = 'after';
      } else {
        dropPosition = 'inside';
      }
    }
  }

  function handleColHeaderDrop(e: DragEvent, targetColId: string) {
    e.preventDefault();
    e.stopPropagation();

    if (draggedReqId) {
      workspaceStore.moveRequestToCollection(draggedReqId, targetColId);
      collapsedColIds[targetColId] = false;
      showToast('Moved into folder', '📁');
    } else if (draggedColId && draggedColId !== targetColId) {
      if (workspaceStore.isCollectionDescendant(draggedColId, targetColId)) {
        showToast('Cannot move folder into itself or subfolder', '⚠️');
        handleDragEnd();
        return;
      }

      const pos = dropPosition || 'inside';
      const success = workspaceStore.reorderOrMoveCollection(draggedColId, targetColId, pos);
      if (success) {
        if (pos === 'inside') {
          collapsedColIds[targetColId] = false;
          showToast('Nested into folder', '📁');
        } else {
          showToast('Folder reordered', '↕️');
        }
      }
    }

    handleDragEnd();
  }

  function handleDropToRoot(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();

    if (draggedColId) {
      const sourceCol = workspaceStore.currentWorkspace.collections.find(c => c.id === draggedColId);
      if (sourceCol && sourceCol.parentId) {
        workspaceStore.nestCollection(draggedColId, null);
        showToast('Moved folder to root', '📁');
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

  function handleCreateSubfolder(parentId: string) {
    if (!newSubfolderName.trim()) return;
    workspaceStore.addFolder(parentId, newSubfolderName);
    collapsedColIds[parentId] = false;
    newSubfolderName = '';
    addingSubfolderParentId = null;
    showToast('Subfolder created', '📁');
  }

  function handleCreateRequest(collectionId: string) {
    if (isColCollapsed(collectionId)) {
      collapsedColIds[collectionId] = false;
    }
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

  function handleCommitRenameCol() {
    if (!renameCol || !renameCol.newName.trim()) return;
    workspaceStore.renameCollection(renameCol.col.id, renameCol.newName);
    showToast(`Renamed collection to "${renameCol.newName.trim()}"`, '✏️');
    renameCol = null;
  }

  function handleCommitDeleteCol() {
    if (!deleteCol) return;
    const name = deleteCol.name;
    const id = deleteCol.id;
    workspaceStore.deleteCollection(id);
    delete collapsedColIds[id];
    showToast(`Deleted collection "${name}"`, '🗑️');
    deleteCol = null;
  }

  function getColContextMenuItems(col: any): ContextMenuItem[] {
    const isCollapsed = isColCollapsed(col.id);
    return [
      {
        id: 'new-req',
        label: 'New Request',
        icon: '➕',
        action: () => {
          handleCreateRequest(col.id);
        }
      },
      {
        id: 'new-folder',
        label: 'New Subfolder',
        icon: '📁',
        action: () => {
          addingSubfolderParentId = col.id;
          collapsedColIds[col.id] = false;
        }
      },
      {
        id: 'toggle-collapse',
        label: isCollapsed ? 'Expand Collection' : 'Collapse Collection',
        icon: isCollapsed ? '📂' : '📁',
        action: () => {
          toggleColCollapse(col.id);
        }
      },
      {
        id: 'expand-all',
        label: 'Expand All',
        icon: '⏬',
        action: () => {
          expandAll();
        }
      },
      {
        id: 'collapse-all',
        label: 'Collapse All',
        icon: '⏫',
        action: () => {
          collapseAll();
        }
      },
      { id: 'sep-collapse', label: '', separator: true, action: () => {} },
      {
        id: 'reveal-col',
        label: 'Reveal in Explorer',
        icon: '📂',
        action: async () => {
          await backendApi.revealInExplorer({ collectionName: col.path || col.name });
          showToast(`Revealed "${col.name}" in Explorer`, '📂');
        }
      },
      { id: 'sep-col', label: '', separator: true, action: () => {} },
      {
        id: 'rename-col',
        label: 'Rename Collection',
        icon: '✏️',
        action: () => {
          renameCol = { col, newName: col.name };
        }
      },
      {
        id: 'delete-col',
        label: 'Delete Collection',
        icon: '🗑️',
        action: () => {
          deleteCol = col;
        }
      }
    ];
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
          await backendApi.revealInExplorer({
            collectionName: req.folderPath || col.path || col.name,
            requestName: req.name
          });
          showToast(`Revealed "${req.name}" in Explorer`, '📂');
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

<div class="h-full flex flex-col justify-between bg-apple-surface-tile-1 select-none overflow-hidden text-xs text-white">
  <div class="flex-1 flex flex-col overflow-hidden">
    <!-- Workspace Selector Bar -->
    <div class="h-12 border-b border-apple-hairline-dark/60 px-3 flex items-center justify-between gap-2 shrink-0 bg-apple-surface-tile-1">
      <div class="flex-1 min-w-0">
        <select
          value={workspaceStore.currentWorkspace.id}
          onchange={(e) => workspaceStore.switchWorkspace((e.target as HTMLSelectElement).value)}
          class="w-full bg-apple-surface-tile-2 border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs font-semibold text-apple-body-on-dark truncate focus:outline-none focus:border-apple-primary"
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
        class="p-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-2 text-apple-primary-on-dark font-bold text-xs shrink-0 border border-apple-hairline-dark/80"
      >
        +
      </button>
    </div>

    <!-- Inline Add Workspace Form -->
    {#if isAddingWorkspace}
      <div class="p-2 border-b border-apple-hairline-dark/60 bg-apple-surface-tile-2 space-y-1.5 shrink-0">
        <input
          type="text"
          bind:value={newWorkspaceName}
          placeholder="New Workspace Name..."
          class="w-full bg-apple-surface-tile-1 border border-apple-hairline-dark/80 rounded-apple-sm px-2.5 py-1 text-xs text-white focus:outline-none focus:border-apple-primary"
        />
        <div class="flex justify-end gap-1 text-[11px]">
          <button
            type="button"
            onclick={() => isAddingWorkspace = false}
            class="px-2 py-0.5 rounded-apple-sm text-apple-body-muted hover:bg-apple-surface-tile-3"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={handleCreateWorkspace}
            class="apple-button-primary px-2.5 py-0.5 rounded-apple-pill text-white font-medium text-xs"
          >
            Create
          </button>
        </div>
      </div>
    {/if}

    <!-- Main Navigation Switcher -->
    <div class="p-2 border-b border-apple-hairline-dark/60 space-y-1 shrink-0 bg-apple-surface-tile-1">
      <button
        type="button"
        onclick={() => onSelectView('studio')}
        class="w-full text-left px-2.5 py-1.5 rounded-apple-sm text-xs font-semibold flex items-center gap-2.5 transition-colors
          {activeView === 'studio'
            ? 'bg-apple-primary text-white shadow-none'
            : 'text-apple-body-muted hover:bg-apple-surface-tile-2 hover:text-white'}"
      >
        <span>⚡</span>
        <span>Request Studio</span>
      </button>

      <button
        type="button"
        onclick={() => onSelectView('runner')}
        class="w-full text-left px-2.5 py-1.5 rounded-apple-sm text-xs font-semibold flex items-center justify-between transition-colors
          {activeView === 'runner'
            ? 'bg-apple-primary text-white shadow-none'
            : 'text-apple-body-muted hover:bg-apple-surface-tile-2 hover:text-white'}"
      >
        <div class="flex items-center gap-2.5">
          <span>📊</span>
          <span>Collection Runner</span>
        </div>
        <span class="text-[9px] px-1 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark font-mono border border-apple-hairline-dark">HDR</span>
      </button>
    </div>

    <!-- Collections & Requests Tree -->
    <div class="flex-1 overflow-y-auto p-2 space-y-3">
      <div class="flex items-center justify-between px-1">
        <span class="apple-micro-legal font-bold text-apple-ink-muted-48 uppercase tracking-wider">Collections</span>
        <div class="flex items-center gap-1">
          <button
            type="button"
            title="Expand All Collections"
            aria-label="Expand All Collections"
            onclick={expandAll}
            class="p-1 rounded-apple-sm text-apple-body-muted hover:text-white hover:bg-apple-surface-tile-2 transition-colors"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="7 13 12 18 17 13"></polyline>
              <polyline points="7 6 12 11 17 6"></polyline>
            </svg>
          </button>
          <button
            type="button"
            title="Collapse All Collections"
            aria-label="Collapse All Collections"
            onclick={collapseAll}
            class="p-1 rounded-apple-sm text-apple-body-muted hover:text-white hover:bg-apple-surface-tile-2 transition-colors"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="17 11 12 6 7 11"></polyline>
              <polyline points="17 18 12 13 7 18"></polyline>
            </svg>
          </button>
          <button
            type="button"
            onclick={() => isAddingCollection = !isAddingCollection}
            class="apple-micro-legal text-apple-primary-on-dark hover:underline font-medium font-mono px-1 py-0.5 rounded-apple-sm"
            title="Add Collection"
          >
            +folder
          </button>
        </div>
      </div>

      {#if isAddingCollection}
        <div class="p-2 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-1.5">
          <input
            type="text"
            bind:value={newCollectionName}
            placeholder="Collection Name..."
            class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-2 py-1 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
          />
          <div class="flex justify-end gap-1.5 text-[11px]">
            <button
              type="button"
              onclick={() => isAddingCollection = false}
              class="px-2 py-0.5 rounded-apple-sm text-apple-body-muted hover:bg-apple-surface-tile-3"
            >
              Cancel
            </button>
            <button
              type="button"
              onclick={handleCreateCollection}
              class="apple-button-primary px-2.5 py-0.5 text-xs font-medium"
            >
              Add
            </button>
          </div>
        </div>
      {/if}

      {#snippet renderTreeNode(node: TreeNode)}
        {@const col = node.col}
        {@const colRequests = node.requests}
        {@const isColOver = dragOverColId === col.id}
        {@const collapsed = isColCollapsed(col.id)}
        {@const indentPx = node.level * 12}
        <div class="space-y-0.5" style="padding-left: {indentPx}px">
          <!-- Folder Header with Drag and Drop -->
          <div
            role="button"
            tabindex="0"
            aria-expanded={!collapsed}
            draggable="true"
            ondragstart={(e) => handleColHeaderDragStart(e, col.id)}
            ondragover={(e) => handleColHeaderDragOver(e, col.id)}
            ondrop={(e) => handleColHeaderDrop(e, col.id)}
            ondragend={handleDragEnd}
            onclick={() => toggleColCollapse(col.id)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                toggleColCollapse(col.id);
              } else if (e.key === 'ArrowRight') {
                if (collapsed) {
                  e.preventDefault();
                  toggleColCollapse(col.id);
                }
              } else if (e.key === 'ArrowLeft') {
                if (!collapsed) {
                  e.preventDefault();
                  toggleColCollapse(col.id);
                }
              }
            }}
            oncontextmenu={(e) => {
              e.preventDefault();
              e.stopPropagation();
              colContextMenu = {
                x: e.clientX,
                y: e.clientY,
                col
              };
            }}
            class="flex items-center justify-between px-2 py-1.5 rounded-apple-sm transition-colors text-apple-body-on-dark font-semibold group cursor-pointer select-none
              {draggedColId === col.id ? 'opacity-40 border border-dashed border-apple-primary' : ''}
              {isColOver && dropPosition === 'inside' ? 'bg-apple-primary/20 ring-1 ring-apple-primary text-apple-primary-on-dark' : 'hover:bg-apple-surface-tile-2/60'}
              {isColOver && dropPosition === 'before' ? 'border-t-2 border-apple-primary' : ''}
              {isColOver && dropPosition === 'after' ? 'border-b-2 border-apple-primary' : ''}"
          >
            <div class="flex items-center gap-1.5 truncate">
              <span
                class="text-apple-ink-muted-48 opacity-40 group-hover:opacity-100 text-[10px] cursor-grab active:cursor-grabbing"
                title="Drag to reorder"
                onclick={(e) => e.stopPropagation()}
                role="presentation"
              >⋮⋮</span>
              <button
                type="button"
                aria-label={collapsed ? `Expand ${col.name}` : `Collapse ${col.name}`}
                onclick={(e) => {
                  e.stopPropagation();
                  toggleColCollapse(col.id);
                }}
                class="w-3.5 h-3.5 flex items-center justify-center text-apple-body-muted hover:text-white transition-transform duration-150 {collapsed ? '' : 'rotate-90'}"
              >
                <svg class="w-2.5 h-2.5 fill-current" viewBox="0 0 24 24">
                  <path d="M8 5v14l11-7z"/>
                </svg>
              </button>
              <span class="text-apple-primary-on-dark text-xs">{collapsed ? '📁' : '📂'}</span>
              <span class="truncate">{col.name}</span>
            </div>
            <div class="flex items-center gap-1">
              <span class="text-[10px] text-apple-ink-muted-48 font-mono">{colRequests.length}</span>
              <button
                type="button"
                title="Add subfolder"
                onclick={(e) => { e.stopPropagation(); addingSubfolderParentId = col.id; collapsedColIds[col.id] = false; }}
                class="opacity-0 group-hover:opacity-100 text-[10px] text-amber-400 hover:text-amber-300 px-1 rounded hover:bg-slate-700"
              >
                +sub
              </button>
              <button
                type="button"
                title="Add request to this folder"
                onclick={(e) => { e.stopPropagation(); handleCreateRequest(col.id); }}
                class="opacity-0 group-hover:opacity-100 text-[10px] text-apple-primary-on-dark hover:underline px-1 rounded-apple-sm hover:bg-apple-surface-tile-3"
              >
                +req
              </button>
              <button
                type="button"
                title="Folder menu"
                onclick={(e) => {
                  e.stopPropagation();
                  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                  colContextMenu = { x: rect.right, y: rect.bottom, col };
                }}
                class="opacity-0 group-hover:opacity-100 hover:text-white px-1 py-0.5 rounded-apple-sm text-[10px] text-apple-body-muted hover:bg-apple-surface-tile-3"
              >
                •••
              </button>
            </div>
          </div>

          <!-- Inline Add Subfolder Form -->
          {#if addingSubfolderParentId === col.id}
            <div class="p-2 ml-4 rounded bg-slate-950 border border-slate-800 space-y-1">
              <input
                type="text"
                bind:value={newSubfolderName}
                placeholder="Subfolder name..."
                onkeydown={(e) => {
                  if (e.key === 'Enter') handleCreateSubfolder(col.id);
                  if (e.key === 'Escape') addingSubfolderParentId = null;
                }}
                class="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs text-slate-100 focus:outline-none"
              />
              <div class="flex justify-end gap-1 text-[11px]">
                <button
                  type="button"
                  onclick={() => addingSubfolderParentId = null}
                  class="px-2 py-0.5 rounded text-slate-400 hover:bg-slate-800"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={() => handleCreateSubfolder(col.id)}
                  class="px-2 py-0.5 rounded bg-sky-600 text-white font-medium"
                >
                  Add
                </button>
              </div>
            </div>
          {/if}

          <!-- Requests in this folder -->
          {#if !collapsed}
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
                  class="px-2 py-1.5 rounded-apple-sm flex items-center justify-between group cursor-grab active:cursor-grabbing transition-all text-xs select-none
                    {isReqDragged ? 'opacity-30 border border-dashed border-apple-primary bg-apple-primary/10' : ''}
                    {isReqOver && dropPosition === 'before' ? 'border-t-2 border-apple-primary' : ''}
                    {isReqOver && dropPosition === 'after' ? 'border-b-2 border-apple-primary' : ''}
                    {isCurrent
                      ? 'bg-apple-primary/20 text-white font-medium border border-apple-primary/40'
                      : 'text-apple-body-muted hover:bg-apple-surface-tile-2/80 hover:text-white'}"
                >
                  <div class="flex items-center gap-1.5 truncate">
                    <span class="text-apple-ink-muted-48 opacity-20 group-hover:opacity-80 text-[10px]">⋮⋮</span>
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
                    class="opacity-0 group-hover:opacity-100 hover:text-white px-1 py-0.5 rounded-apple-sm text-[10px] text-apple-body-muted hover:bg-apple-surface-tile-3"
                  >
                    •••
                  </button>
                </div>
              {/each}

              <!-- Nested Subfolders -->
              {#each node.subfolders as subnode (subnode.col.id)}
                {@render renderTreeNode(subnode)}
              {/each}
            </div>
          {/if}
        </div>
      {/snippet}

      {#each collectionTree as rootNode (rootNode.col.id)}
        {@render renderTreeNode(rootNode)}
      {/each}

      <!-- Drop target for moving nested folders back to root level -->
      {#if draggedColId}
        <div
          role="region"
          aria-label="Move to root"
          ondragover={(e) => { e.preventDefault(); e.stopPropagation(); }}
          ondrop={handleDropToRoot}
          class="mt-2 p-2 border-2 border-dashed border-apple-hairline-dark/60 hover:border-apple-primary/80 rounded-apple-sm text-center text-[11px] text-apple-body-muted hover:text-apple-primary-on-dark transition-colors cursor-pointer bg-apple-surface-tile-2/40"
        >
          <span>📁 Drop here to move to root</span>
        </div>
      {/if}
    </div>
  </div>

  <!-- Bottom Workspace Controls -->
  <div class="p-2 border-t border-apple-hairline-dark/60 bg-apple-surface-tile-1 text-xs space-y-1 shrink-0">
    <!-- Environment Profiles Shortcut -->
    <button
      type="button"
      onclick={onOpenEnvManager}
      class="w-full text-left px-2.5 py-1.5 rounded-apple-sm text-apple-body-on-dark hover:bg-apple-surface-tile-2 transition-colors flex items-center justify-between border border-transparent hover:border-apple-hairline-dark/60"
    >
      <div class="flex items-center gap-1.5 truncate">
        <span>🌱</span>
        <span class="truncate">
          Env: {workspaceStore.activeEnvironment?.name || 'None'}
        </span>
      </div>
      <span class="text-[10px] px-1.5 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark font-mono">
        Edit
      </span>
    </button>

    <!-- Interop Hub -->
    <button
      type="button"
      onclick={onOpenInterop}
      class="w-full text-left px-2.5 py-1.5 rounded-apple-sm text-apple-body-on-dark hover:bg-apple-surface-tile-2 transition-colors flex items-center justify-between border border-transparent hover:border-apple-hairline-dark/60"
    >
      <span>Interop (OpenAPI / Bruno)</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark font-mono">3.1</span>
    </button>

    <!-- 3-Way Conflict Simulation -->
    <button
      type="button"
      onclick={onOpenConflict}
      class="w-full text-left px-2.5 py-1.5 rounded-apple-sm text-apple-body-on-dark hover:bg-apple-surface-tile-2 transition-colors flex items-center justify-between border border-transparent hover:border-apple-hairline-dark/60"
    >
      <span>Simulate 3-Way Conflict</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark font-mono">Test</span>
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

{#if colContextMenu}
  <ContextMenu
    x={colContextMenu.x}
    y={colContextMenu.y}
    items={getColContextMenuItems(colContextMenu.col)}
    onClose={() => colContextMenu = null}
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
    class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 animate-in fade-in duration-100"
    onclick={() => renameReq = null}
    role="presentation"
  >
    <div
      class="apple-modal-card p-4 w-80 space-y-3 font-sans text-apple-body-on-dark"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center justify-between">
        <h3 class="apple-caption-strong text-apple-body-on-dark flex items-center gap-1.5">
          <span>✏️</span>
          <span>Rename Request</span>
        </h3>
        <button
          type="button"
          onclick={() => renameReq = null}
          class="apple-button-icon-circular w-6 h-6 text-xs"
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
        class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-2.5 py-1.5 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
      />

      <div class="flex justify-end gap-2 text-xs">
        <button
          type="button"
          onclick={() => renameReq = null}
          class="apple-button-dark-utility text-xs px-2.5 py-1"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCommitRename}
          class="apple-button-primary text-xs px-3 py-1 font-medium"
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
    class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 animate-in fade-in duration-100"
    onclick={() => deleteReq = null}
    role="presentation"
  >
    <div
      class="apple-modal-card p-4 w-80 space-y-3 font-sans text-apple-body-on-dark border-rose-900/60"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center gap-2 text-rose-400 apple-caption-strong">
        <span>⚠️</span>
        <span>Delete Request</span>
      </div>

      <p class="apple-fine-print text-apple-body-muted">
        Are you sure you want to delete <span class="font-semibold text-white">"{deleteReq.name}"</span>? This will remove the request and close its tab.
      </p>

      <div class="flex justify-end gap-2 text-xs">
        <button
          type="button"
          onclick={() => deleteReq = null}
          class="apple-button-dark-utility text-xs px-2.5 py-1"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCommitDelete}
          class="apple-button-primary text-xs px-3 py-1 font-medium bg-rose-600 hover:bg-rose-500"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Rename Collection Dialog -->
{#if renameCol}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 animate-in fade-in duration-100"
    onclick={() => renameCol = null}
    role="presentation"
  >
    <div
      class="apple-modal-card p-4 w-80 space-y-3 font-sans text-apple-body-on-dark"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center justify-between">
        <h3 class="apple-caption-strong text-apple-body-on-dark flex items-center gap-1.5">
          <span>✏️</span>
          <span>Rename Collection</span>
        </h3>
        <button
          type="button"
          onclick={() => renameCol = null}
          class="apple-button-icon-circular w-6 h-6 text-xs"
        >
          ✕
        </button>
      </div>

      <input
        type="text"
        bind:value={renameCol.newName}
        onkeydown={(e) => {
          if (e.key === 'Enter') handleCommitRenameCol();
          if (e.key === 'Escape') renameCol = null;
        }}
        class="w-full bg-apple-surface-black border border-apple-hairline-dark rounded-apple-sm px-2.5 py-1.5 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
      />

      <div class="flex justify-end gap-2 text-xs">
        <button
          type="button"
          onclick={() => renameCol = null}
          class="apple-button-dark-utility text-xs px-2.5 py-1"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCommitRenameCol}
          class="apple-button-primary text-xs px-3 py-1 font-medium"
        >
          Save
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete Collection Confirmation Dialog -->
{#if deleteCol}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 animate-in fade-in duration-100"
    onclick={() => deleteCol = null}
    role="presentation"
  >
    <div
      class="apple-modal-card p-4 w-80 space-y-3 font-sans text-apple-body-on-dark border-rose-900/60"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center gap-2 text-rose-400 apple-caption-strong">
        <span>⚠️</span>
        <span>Delete Collection</span>
      </div>

      <p class="apple-fine-print text-apple-body-muted">
        Are you sure you want to delete collection <span class="font-semibold text-white">"{deleteCol.name}"</span>? All requests inside will be removed from disk.
      </p>

      <div class="flex justify-end gap-2 text-xs">
        <button
          type="button"
          onclick={() => deleteCol = null}
          class="apple-button-dark-utility text-xs px-2.5 py-1"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCommitDeleteCol}
          class="apple-button-primary text-xs px-3 py-1 font-medium bg-rose-600 hover:bg-rose-500"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Toast Feedback -->
{#if toast}
  <div class="fixed bottom-4 right-4 z-50 px-3.5 py-2 rounded-apple-pill apple-frosted-glass bg-apple-surface-tile-2 border border-apple-hairline-dark text-white text-xs font-medium flex items-center gap-2 animate-in fade-in slide-in-from-bottom-2 duration-150">
    <span>{toast.icon}</span>
    <span>{toast.text}</span>
  </div>
{/if}
