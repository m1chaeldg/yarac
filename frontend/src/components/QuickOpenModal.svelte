<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../stores/workspace.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSelectRequest: (reqId: string) => void;
  }

  let { isOpen, onClose, onSelectRequest }: Props = $props();

  let searchQuery = $state('');
  let selectedIndex = $state(0);
  let searchInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (isOpen) {
      searchQuery = '';
      selectedIndex = 0;
      setTimeout(() => searchInput?.focus(), 50);
    }
  });

  interface SearchItem {
    req: YaracRequest;
    collectionName: string;
    folderPath: string;
  }

  let allItems = $derived.by<SearchItem[]>(() => {
    const ws = workspaceStore.currentWorkspace;
    const items: SearchItem[] = [];

    for (const req of ws.requests) {
      const col = ws.collections.find(c => c.id === req.collectionId);
      items.push({
        req,
        collectionName: col?.name || 'General',
        folderPath: req.folderPath || col?.path || col?.name || ''
      });
    }

    return items;
  });

  let filteredItems = $derived.by<SearchItem[]>(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return allItems;

    return allItems.filter(item => {
      const nameMatch = item.req.name.toLowerCase().includes(q);
      const urlMatch = item.req.url.toLowerCase().includes(q);
      const methodMatch = item.req.method.toLowerCase().includes(q);
      const folderMatch = item.folderPath.toLowerCase().includes(q);
      return nameMatch || urlMatch || methodMatch || folderMatch;
    });
  });

  $effect(() => {
    // Reset selectedIndex if it exceeds length
    if (selectedIndex >= filteredItems.length) {
      selectedIndex = Math.max(0, filteredItems.length - 1);
    }
  });

  function handleSelect(item: SearchItem) {
    onSelectRequest(item.req.id);
    onClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % Math.max(1, filteredItems.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filteredItems.length) % Math.max(1, filteredItems.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = filteredItems[selectedIndex];
      if (item) {
        handleSelect(item);
      }
    }
  }
</script>

<svelte:window onkeydown={(e) => {
  if (isOpen) handleKeyDown(e);
}} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-start justify-center pt-24 apple-frosted-backdrop p-4 font-sans animate-fade-in"
    onclick={onClose}
    role="presentation"
  >
    <div
      class="apple-modal-card w-full max-w-xl flex flex-col text-apple-body-on-dark shadow-2xl overflow-hidden border border-apple-hairline-dark/80"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Search Input Header -->
      <div class="h-12 border-b border-apple-hairline-dark bg-apple-surface-tile-2 px-3.5 flex items-center gap-2.5 shrink-0">
        <span class="text-apple-body-muted text-sm">🔍</span>
        <input
          bind:this={searchInput}
          type="text"
          bind:value={searchQuery}
          placeholder="Type a request name, method, URL or folder... (Esc to close)"
          class="flex-1 bg-transparent text-xs text-apple-body-on-dark placeholder:text-apple-ink-muted-48 focus:outline-none font-mono"
        />
        {#if searchQuery}
          <button
            type="button"
            onclick={() => searchQuery = ''}
            class="text-xs text-apple-body-muted hover:text-white"
          >
            Clear
          </button>
        {/if}
      </div>

      <!-- Results List -->
      <div class="max-h-[380px] overflow-auto p-1.5 space-y-0.5 bg-apple-surface-tile-1">
        {#each filteredItems as item, idx (item.req.id)}
          {@const isSelected = selectedIndex === idx}
          <div
            role="button"
            tabindex="0"
            onclick={() => handleSelect(item)}
            onkeydown={(e) => e.key === 'Enter' && handleSelect(item)}
            onmouseenter={() => selectedIndex = idx}
            class="p-2 rounded-apple-sm transition-colors flex items-center justify-between cursor-pointer
              {isSelected ? 'bg-apple-primary text-white shadow-sm' : 'hover:bg-apple-surface-tile-2 text-apple-body-on-dark'}"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <!-- Method Badge -->
              <span class="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded
                {isSelected ? 'bg-white/20 text-white' : item.req.method === 'GET' ? 'bg-sky-500/10 text-sky-400' : item.req.method === 'POST' ? 'bg-emerald-500/10 text-emerald-400' : item.req.method === 'DELETE' ? 'bg-rose-500/10 text-rose-400' : 'bg-amber-500/10 text-amber-400'}"
              >
                {item.req.method}
              </span>

              <!-- Request Name -->
              <span class="text-xs font-semibold truncate">{item.req.name}</span>

              <!-- Folder / Path Badge -->
              {#if item.folderPath}
                <span class="text-[10px] {isSelected ? 'text-white/70' : 'text-apple-ink-muted-48'} font-mono truncate">
                  {item.folderPath}
                </span>
              {/if}
            </div>

            <!-- URL snippet -->
            <span class="text-[10px] font-mono {isSelected ? 'text-white/80' : 'text-apple-body-muted'} truncate max-w-[180px] ml-2">
              {item.req.url}
            </span>
          </div>
        {/each}

        {#if filteredItems.length === 0}
          <div class="p-8 text-center text-apple-body-muted text-xs">
            No matching requests found for "{searchQuery}".
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="h-8 border-t border-apple-hairline-dark bg-apple-surface-tile-2 px-3 flex items-center justify-between text-[10px] text-apple-ink-muted-48 shrink-0">
        <div class="flex items-center gap-3">
          <span><kbd class="px-1 py-0.2 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark">↑↓</kbd> Navigate</span>
          <span><kbd class="px-1 py-0.2 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark">Enter</kbd> Open</span>
          <span><kbd class="px-1 py-0.2 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark">Esc</kbd> Close</span>
        </div>
        <span>{filteredItems.length} results</span>
      </div>
    </div>
  </div>
{/if}
