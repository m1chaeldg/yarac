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
    class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 animate-in fade-in duration-150"
    onclick={onClose}
    role="presentation"
  >
    <div
      class="apple-modal-card w-full max-w-2xl flex flex-col overflow-hidden text-apple-body-on-dark font-sans"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="request-info-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-5 py-3.5 border-b border-apple-hairline-dark flex items-center justify-between shrink-0 bg-apple-surface-tile-2">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-apple-sm bg-apple-surface-tile-3 text-apple-primary-on-dark flex items-center justify-center font-bold text-sm border border-apple-hairline-dark">
            ℹ️
          </div>
          <div>
            <h2 id="request-info-title" class="apple-body-strong text-apple-body-on-dark">
              Request Metadata & Audit
            </h2>
            <p class="apple-fine-print text-apple-body-muted">
              Inspection of request properties, memory footprint, and relative file anchoring.
            </p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
          title="Close dialog"
        >
          ✕
        </button>
      </div>

      <!-- Content -->
      <div class="p-5 space-y-4 text-xs overflow-y-auto max-h-[70vh] bg-apple-surface-tile-1">
        <!-- Overview Grid -->
        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-1">
            <span class="apple-micro-legal font-bold uppercase text-apple-ink-muted-48">Request Name</span>
            <div class="font-semibold text-white truncate">{request.name}</div>
          </div>
          <div class="p-3 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-1">
            <span class="apple-micro-legal font-bold uppercase text-apple-ink-muted-48">Parent Collection</span>
            <div class="font-semibold text-white truncate">{collection?.name || 'Root'}</div>
          </div>
          <div class="p-3 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-1">
            <span class="apple-micro-legal font-bold uppercase text-apple-ink-muted-48">HTTP Method</span>
            <div class="font-mono font-bold text-apple-primary-on-dark">{request.method}</div>
          </div>
          <div class="p-3 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-1">
            <span class="apple-micro-legal font-bold uppercase text-apple-ink-muted-48">Internal ID</span>
            <div class="font-mono text-apple-body-muted truncate">{request.id}</div>
          </div>
        </div>

        <!-- Relative Workspace File Anchor -->
        <div class="p-3 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-2">
          <div class="flex items-center justify-between">
            <span class="apple-micro-legal font-bold uppercase text-apple-ink-muted-48">Anchored Workspace Path</span>
            <button
              type="button"
              onclick={copyPath}
              class="apple-fine-print text-apple-primary-on-dark hover:underline font-medium"
            >
              {isCopiedPath ? '✓ Copied!' : 'Copy Path'}
            </button>
          </div>
          <div class="font-mono text-xs text-apple-primary-on-dark bg-apple-surface-black px-2.5 py-1.5 rounded-apple-sm border border-apple-hairline-dark break-all select-all">
            {relativePath}
          </div>
          <p class="apple-micro-legal text-apple-ink-muted-48">
            Resolves relative to workspace root per AGENTS.md relative path anchoring standard.
          </p>
        </div>

        <!-- Request Details Table -->
        <div class="rounded-apple-sm border border-apple-hairline-dark overflow-hidden">
          <table class="w-full text-left text-xs">
            <tbody class="divide-y divide-apple-hairline-dark/60 bg-apple-surface-tile-2/40">
              <tr>
                <td class="px-3 py-2 text-apple-body-muted font-medium w-1/3">Target URL</td>
                <td class="px-3 py-2 font-mono text-white truncate">{request.url}</td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-apple-body-muted font-medium">Headers</td>
                <td class="px-3 py-2 text-white">
                  {request.headers.filter(h => h.enabled).length} active ({request.headers.length} total)
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-apple-body-muted font-medium">Query Parameters</td>
                <td class="px-3 py-2 text-white">
                  {request.params.filter(p => p.enabled).length} active ({request.params.length} total)
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-apple-body-muted font-medium">Authentication</td>
                <td class="px-3 py-2 text-white capitalize font-mono text-[11px]">
                  {request.auth.type}
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-apple-body-muted font-medium">Body Type</td>
                <td class="px-3 py-2 text-white capitalize font-mono text-[11px]">
                  {request.body.type}
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-apple-body-muted font-medium">DSL Size / Lines</td>
                <td class="px-3 py-2 text-white font-mono text-[11px]">
                  {request.dslContent.length} bytes · {request.dslContent.split('\n').length} lines
                </td>
              </tr>
              <tr>
                <td class="px-3 py-2 text-apple-body-muted font-medium">Flow Execution</td>
                <td class="px-3 py-2 text-white font-mono text-[11px]">
                  Max Retries: {request.flow?.max_retries ?? 0} · Delay: {request.flow?.delay_ms ?? 0}ms
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 border-t border-apple-hairline-dark bg-apple-surface-tile-2 flex items-center justify-between">
        <button
          type="button"
          onclick={copyDsl}
          class="apple-button-dark-utility text-xs px-3.5 py-1.5 font-medium"
        >
          {isCopiedDsl ? '✓ Copied DSL' : 'Copy .yarac DSL'}
        </button>

        <button
          type="button"
          onclick={onClose}
          class="apple-button-primary text-xs px-4 py-1.5 font-semibold"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
