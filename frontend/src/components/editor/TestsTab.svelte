<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';
  import { BDD_TEMPLATE, SCRIPT_PLACEHOLDERS, appendSnippet } from '../../utils/editorUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  function insertTestSnippet(snippet: string) {
    if (!req) return;
    req.tests = appendSnippet(req.tests, snippet);
    workspaceStore.updateActiveRequest({ tests: req.tests });
  }
</script>

<div class="h-full flex flex-col space-y-3 font-sans">
  <div class="flex items-center justify-between shrink-0">
    <div>
      <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Jasmine / Bruno BDD Test Suite</span>
      <p class="text-[11px] text-slate-500">
        Structure tests with <code class="text-pink-400 font-mono">describe()</code> and <code class="text-pink-400 font-mono">it()</code> blocks. Full Faker library available globally.
      </p>
    </div>

    <div class="flex items-center gap-1.5">
      <button
        type="button"
        onclick={() => insertTestSnippet(BDD_TEMPLATE)}
        class="px-2.5 py-1 rounded bg-pink-950/80 hover:bg-pink-900 text-pink-300 border border-pink-800 text-[11px] font-mono transition-colors cursor-pointer"
      >
        + BDD Template
      </button>
    </div>
  </div>

  <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
    <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
      <span class="font-mono text-pink-400">// globals: describe, it, test, expect, res, req, faker</span>
      <span class="text-slate-500 font-mono text-[10px]">Jasmine / Chai assertion suite</span>
    </div>
    <textarea
      value={req.tests || ''}
      oninput={(e) => {
        req.tests = (e.target as HTMLTextAreaElement).value;
        workspaceStore.updateActiveRequest({ tests: req.tests });
      }}
      placeholder={SCRIPT_PLACEHOLDERS.tests}
      class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-pink-100 resize-none focus:outline-none"
    ></textarea>
  </div>
</div>
