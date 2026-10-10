<script lang="ts">
  export interface ContextMenuItem {
    id: string;
    label: string;
    icon?: string;
    shortcut?: string;
    danger?: boolean;
    separator?: boolean;
    action: () => void;
  }

  interface Props {
    x: number;
    y: number;
    items: ContextMenuItem[];
    onClose: () => void;
  }

  let { x, y, items, onClose }: Props = $props();

  let menuElement = $state<HTMLDivElement | null>(null);
  let adjustedX = $state(0);
  let adjustedY = $state(0);

  $effect(() => {
    let newX = x;
    let newY = y;
    if (menuElement && typeof window !== 'undefined') {
      const rect = menuElement.getBoundingClientRect();
      const padding = 10;
      if (newX + rect.width > window.innerWidth - padding) {
        newX = Math.max(padding, window.innerWidth - rect.width - padding);
      }
      if (newY + rect.height > window.innerHeight - padding) {
        newY = Math.max(padding, window.innerHeight - rect.height - padding);
      }
    }
    adjustedX = newX;
    adjustedY = newY;
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop capture -->
<div
  class="fixed inset-0 z-50 bg-transparent"
  onclick={onClose}
  oncontextmenu={(e) => { e.preventDefault(); onClose(); }}
  role="presentation"
>
  <div
    bind:this={menuElement}
    style="top: {adjustedY}px; left: {adjustedX}px;"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="menu"
    tabindex="-1"
    class="fixed z-50 w-60 bg-slate-900 border border-slate-700/80 rounded-lg shadow-2xl shadow-black/80 py-1.5 text-xs text-slate-200 select-none animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md"
  >
    {#each items as item (item.id)}
      {#if item.separator}
        <div class="my-1 border-t border-slate-800" role="separator"></div>
      {:else}
        <button
          type="button"
          role="menuitem"
          onclick={() => {
            onClose();
            item.action();
          }}
          class="w-full text-left px-3 py-1.5 flex items-center justify-between gap-2 transition-colors
            {item.danger
              ? 'text-rose-400 hover:bg-rose-950/60 hover:text-rose-200'
              : 'hover:bg-sky-600/20 hover:text-white'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            {#if item.icon}
              <span class="w-4 text-center text-[13px] shrink-0 opacity-80">{item.icon}</span>
            {/if}
            <span class="truncate font-medium">{item.label}</span>
          </div>

          {#if item.shortcut}
            <span class="text-[10px] font-mono text-slate-500 shrink-0 uppercase tracking-wider">{item.shortcut}</span>
          {/if}
        </button>
      {/if}
    {/each}
  </div>
</div>
