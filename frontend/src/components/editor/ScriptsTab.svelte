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
    <div class="flex items-center gap-1.5 bg-slate-900 border border-slate-800 p-0.5 rounded-lg">
      <button
        type="button"
        onclick={() => activeScriptType = 'preRequest'}
        class="px-3 py-1 rounded text-xs font-semibold transition-colors {activeScriptType === 'preRequest' ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200 cursor-pointer'}"
      >
        Pre-request Script
      </button>
      <button
        type="button"
        onclick={() => activeScriptType = 'postResponse'}
        class="px-3 py-1 rounded text-xs font-semibold transition-colors {activeScriptType === 'postResponse' ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200 cursor-pointer'}"
      >
        Post-response Script
      </button>
    </div>

    <!-- Snippets Quick Bar -->
    <div class="flex items-center gap-1.5 flex-wrap">
      <span class="text-[10px] text-slate-500">Insert Snippet:</span>
      {#if activeScriptType === 'preRequest'}
        {#each SCRIPT_SNIPPETS.preRequest as s}
          <button
            type="button"
            onclick={() => insertScriptSnippet(s.snippet)}
            class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300 transition-colors cursor-pointer"
          >
            {s.label}
          </button>
        {/each}
      {:else}
        {#each SCRIPT_SNIPPETS.postResponse as s}
          <button
            type="button"
            onclick={() => insertScriptSnippet(s.snippet)}
            class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300 transition-colors cursor-pointer"
          >
            {s.label}
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
    <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
      <span class="font-mono text-indigo-400">
        {activeScriptType === 'preRequest' ? '// Runs before HTTP request is dispatched' : '// Runs immediately after HTTP response is received'}
      </span>
      <span class="text-slate-500 font-mono text-[10px]">Isolated Deno Core Runtime (5000ms max)</span>
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
        class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-indigo-100 resize-none focus:outline-none"
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
        class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-indigo-100 resize-none focus:outline-none"
      ></textarea>
    {/if}
  </div>
</div>
