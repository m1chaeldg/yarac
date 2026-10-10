<script lang="ts">
  import type { YaracRequest, Collection } from '../stores/workspace.svelte';
  import { getRelativeRequestPath } from '../utils/codeGenerators';

  interface Props {
    isOpen: boolean;
    request: YaracRequest | null;
    collection: Collection | null;
    onClose: () => void;
  }

  let { isOpen, request, collection, onClose }: Props = $props();

  let relativePath = $derived(
    request && collection ? getRelativeRequestPath(collection.name, request.name) : ''
  );

  let isCopiedPath = $state(false);
  let isCopiedDsl = $state(false);

  async function copyPath() {
    if (!relativePath) return;
    try {
      await navigator.clipboard.writeText(relativePath);
      isCopiedPath = true;
      setTimeout(() => isCopiedPath = false, 2000);
    } catch (err) {
      console.error(err);
    }
  }

  async function copyDsl() {
    if (!request?.dslContent) return;
    try {
      await navigator.clipboard.writeText(request.dslContent);
      isCopiedDsl = true;
      setTimeout(() => isCopiedDsl = false, 2000);
    } catch (err) {
      console.error(err);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen && request}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150"
    onclick={onClose}
    role="presentation"
  >
    <div
      class="bg-slate-900 border border-slate-800 rounded-xl shadow-2xl shadow-black/90 w-full max-w-2xl flex flex-col overflow-hidden text-slate-100"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="request-info-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-5 py-3.5 border-b border-slate-800 flex items-center justify-between shrink-0 bg-slate-950/60">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-emerald-600/20 text-emerald-400 flex items-center justify-center font-bold text-sm border border-emerald-500/30">
            ℹ️
          </div>
          <div>
            <h2 id="request-info-title" class="text-sm font-bold text-slate-100">
              Request Metadata & Audit
            </h2>
            <p class="text-[11px] text-slate-400">
              Inspection of request properties, memory footprint, and relative file anchoring.
            </p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="w-7 h-7 rounded-lg text-slate-400 hover:text-slate-100 hover:bg-slate-800 flex items-center justify-center text-sm font-bold transition-colors"
          title="Close dialog"
        >
          ✕
        </button>
      </div>

      <!-- Content -->
      <div class="p-5 space-y-4 text-xs overflow-y-auto max-h-[70vh]">
        <!-- Overview Grid -->
        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-1">
            <span class="text-[10px] font-bold uppercase text-slate-500">Request Name</span>
            <div class="font-semibold text-slate-200 truncate">{request.name}</div>
          </div>
          <div class="p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-1">
            <span class="text-[10px] font-bold uppercase text-slate-500">Parent Collection</span>
            <div class="font-semibold text-slate-200 truncate">{collection?.name || 'Root'}</div>
          </div>
          <div class="p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-1">
            <span class="text-[10px] font-bold uppercase text-slate-500">HTTP Method</span>
            <div class="font-mono font-bold text-sky-400">{request.method}</div>
          </div>
          <div class="p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-1">
            <span class="text-[10px] font-bold uppercase text-slate-500">Internal ID</span>
            <div class="font-mono text-slate-400 truncate">{request.id}</div>
          </div>
        </div>

        <!-- Relative Workspace File Anchor -->
        <div class="p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-[10px] font-bold uppercase text-slate-500">Anchored Workspace Path</span>
            <button
              type="button"
              onclick={copyPath}
              class="text-[11px] text-sky-400 hover:text-sky-300 font-medium"
            >
              {isCopiedPath ? '✓ Copied!' : 'Copy Path'}
            </button>
          </div>
          <div class="font-mono text-xs text-amber-300/90 bg-slate-900 px-2.5 py-1.5 rounded border border-slate-800 break-all select-all">
            {relativePath}
          </div>
          <p class="text-[10px] text-slate-500">
            Resolves relative to workspace root per AGENTS.md relative path anchoring standard.
          </p>
        </div>

        <!-- Request Details Table -->
        <div class="rounded-lg border border-slate-800 overflow-hidden">
          <table class="w-full text-left text-xs">
            <tbody class="divide-y divide-slate-800/80 bg-slate-950/60">
              <tr>
                <td class="px-3 py-2 text-slate-400 font-medium w-1/3">Target URL</td>
                <td class="px-3 py-2 font-mono text-slate-200 truncate">{request.url}</td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-slate-400 font-medium">Headers</td>
                <td class="px-3 py-2 text-slate-200">
                  {request.headers.filter(h => h.enabled).length} active ({request.headers.length} total)
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-slate-400 font-medium">Query Parameters</td>
                <td class="px-3 py-2 text-slate-200">
                  {request.params.filter(p => p.enabled).length} active ({request.params.length} total)
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-slate-400 font-medium">Authentication</td>
                <td class="px-3 py-2 text-slate-200 capitalize font-mono text-[11px]">
                  {request.auth.type}
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-slate-400 font-medium">Body Type</td>
                <td class="px-3 py-2 text-slate-200 capitalize font-mono text-[11px]">
                  {request.body.type}
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-slate-400 font-medium">DSL Size / Lines</td>
                <td class="px-3 py-2 text-slate-200 font-mono text-[11px]">
                  {request.dslContent.length} bytes · {request.dslContent.split('\n').length} lines
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-slate-400 font-medium">Flow Execution</td>
                <td class="px-3 py-2 text-slate-200 font-mono text-[11px]">
                  Max Retries: {request.flow?.max_retries ?? 0} · Delay: {request.flow?.delay_ms ?? 0}ms
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 border-t border-slate-800 bg-slate-950/60 flex items-center justify-between">
        <button
          type="button"
          onclick={copyDsl}
          class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium transition-colors"
        >
          {isCopiedDsl ? '✓ Copied DSL' : 'Copy .yarac DSL'}
        </button>

        <button
          type="button"
          onclick={onClose}
          class="px-4 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold transition-colors"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
