<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';
  import { SCRIPT_SNIPPETS, SCRIPT_PLACEHOLDERS, appendSnippet } from '../../utils/editorUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let activeScriptType = $state<'preRequest' | 'postResponse'>('preRequest');

  function insertScriptSnippet(snippet: string) {
    if (!req) return;
    if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
    if (activeScriptType === 'preRequest') {
      req.scripts.preRequest = appendSnippet(req.scripts.preRequest, snippet);
    } else {
      req.scripts.postResponse = appendSnippet(req.scripts.postResponse, snippet);
    }
    workspaceStore.updateActiveRequest({ scripts: req.scripts });
  }
</script>

<div class="h-full flex flex-col space-y-3 font-sans">
  <div class="flex items-center justify-between shrink-0">
    <div class="flex items-center gap-1.5 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 p-0.5 rounded-apple-sm">
      <button
        type="button"
        onclick={() => activeScriptType = 'preRequest'}
        class="px-3 py-1 rounded-apple-sm text-xs font-medium transition-colors {activeScriptType === 'preRequest' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white cursor-pointer'}"
      >
        Pre-request Script
      </button>
      <button
        type="button"
        onclick={() => activeScriptType = 'postResponse'}
        class="px-3 py-1 rounded-apple-sm text-xs font-medium transition-colors {activeScriptType === 'postResponse' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white cursor-pointer'}"
      >
        Post-response Script
      </button>
    </div>

    <!-- Snippets Quick Bar -->
    <div class="flex items-center gap-1.5 flex-wrap">
      <span class="text-[10px] text-apple-ink-muted-48">Insert Snippet:</span>
      {#if activeScriptType === 'preRequest'}
        {#each SCRIPT_SNIPPETS.preRequest as s}
          <button
            type="button"
            onclick={() => insertScriptSnippet(s.snippet)}
            class="apple-button-dark-utility px-2 py-0.5 rounded-apple-sm border border-apple-hairline-dark/60 text-[10px] font-mono text-apple-primary-on-dark transition-colors cursor-pointer"
          >
            {s.label}
          </button>
        {/each}
      {:else}
        {#each SCRIPT_SNIPPETS.postResponse as s}
          <button
            type="button"
            onclick={() => insertScriptSnippet(s.snippet)}
            class="apple-button-dark-utility px-2 py-0.5 rounded-apple-sm border border-apple-hairline-dark/60 text-[10px] font-mono text-apple-primary-on-dark transition-colors cursor-pointer"
          >
            {s.label}
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <div class="flex-1 flex flex-col border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black min-h-[220px]">
    <div class="bg-apple-surface-tile-2 px-3 py-1.5 border-b border-apple-hairline-dark/60 flex items-center justify-between text-[11px] text-apple-body-muted">
      <span class="font-mono text-apple-primary-on-dark">
        {activeScriptType === 'preRequest' ? '// Runs before HTTP request is dispatched' : '// Runs immediately after HTTP response is received'}
      </span>
      <span class="text-apple-ink-muted-48 font-mono text-[10px]">Async/Await Default • Isolated Deno Core (5000ms max)</span>
    </div>

    {#if activeScriptType === 'preRequest'}
      <textarea
        value={req.scripts?.preRequest || ''}
        oninput={(e) => {
          if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
          req.scripts.preRequest = (e.target as HTMLTextAreaElement).value;
          workspaceStore.updateActiveRequest({ scripts: req.scripts });
        }}
        placeholder={SCRIPT_PLACEHOLDERS.preRequest}
        class="flex-1 w-full bg-apple-surface-black p-3 font-mono text-xs text-apple-body-on-dark resize-none focus:outline-none"
      ></textarea>
    {:else}
      <textarea
        value={req.scripts?.postResponse || ''}
        oninput={(e) => {
          if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
          req.scripts.postResponse = (e.target as HTMLTextAreaElement).value;
          workspaceStore.updateActiveRequest({ scripts: req.scripts });
        }}
        placeholder={SCRIPT_PLACEHOLDERS.postResponse}
        class="flex-1 w-full bg-apple-surface-black p-3 font-mono text-xs text-apple-body-on-dark resize-none focus:outline-none"
      ></textarea>
    {/if}
  </div>
</div>
