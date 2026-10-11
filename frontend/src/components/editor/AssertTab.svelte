<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';
  import { ASSERT_SNIPPETS, SCRIPT_PLACEHOLDERS, appendSnippet } from '../../utils/editorUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  function insertAssertSnippet(snippet: string) {
    if (!req) return;
    req.assert = appendSnippet(req.assert, snippet);
    workspaceStore.updateActiveRequest({ assert: req.assert });
  }
</script>

<div class="h-full flex flex-col space-y-3 font-sans">
  <div class="flex items-center justify-between shrink-0">
    <div>
      <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">JavaScript Assertions</span>
      <p class="text-[11px] text-slate-500">
        Lightweight assertion statements evaluated directly on the response. Test results display in the Response tab.
      </p>
    </div>

    <!-- Snippets -->
    <div class="flex items-center gap-1.5 flex-wrap">
      {#each ASSERT_SNIPPETS as s}
        <button
          type="button"
          onclick={() => insertAssertSnippet(s.snippet)}
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300 transition-colors cursor-pointer"
        >
          {s.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
    <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
      <span class="font-mono text-teal-400">// Available matchers: toBe, toEqual, toBeDefined, toBeNull, toBeGreaterThan, toContain</span>
      <span class="text-slate-500 font-mono text-[10px]">Async/Await Default • Live Assertions</span>
    </div>
    <textarea
      value={req.assert || ''}
      oninput={(e) => {
        req.assert = (e.target as HTMLTextAreaElement).value;
        workspaceStore.updateActiveRequest({ assert: req.assert });
      }}
      placeholder={SCRIPT_PLACEHOLDERS.assert}
      class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-teal-100 resize-none focus:outline-none"
    ></textarea>
  </div>
</div>
