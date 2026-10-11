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
      <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider">Jasmine / Bruno BDD Test Suite</span>
      <p class="text-[11px] text-apple-ink-muted-48">
        Structure tests with <code class="text-apple-primary-on-dark font-mono">describe()</code> and <code class="text-apple-primary-on-dark font-mono">it()</code> blocks. Full Faker library available globally.
      </p>
    </div>

    <div class="flex items-center gap-1.5">
      <button
        type="button"
        onclick={() => insertTestSnippet(BDD_TEMPLATE)}
        class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60 text-[11px] font-mono transition-colors cursor-pointer"
      >
        + BDD Template
      </button>
    </div>
  </div>

  <div class="flex-1 flex flex-col border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black min-h-[220px]">
    <div class="bg-apple-surface-tile-2 px-3 py-1.5 border-b border-apple-hairline-dark/60 flex items-center justify-between text-[11px] text-apple-body-muted">
      <span class="font-mono text-apple-primary-on-dark">// globals: describe, it, test, expect, res, req, faker</span>
      <span class="text-apple-ink-muted-48 font-mono text-[10px]">Async/Await Default • Jasmine/Chai Suite</span>
    </div>
    <textarea
      value={req.tests || ''}
      oninput={(e) => {
        req.tests = (e.target as HTMLTextAreaElement).value;
        workspaceStore.updateActiveRequest({ tests: req.tests });
      }}
      placeholder={SCRIPT_PLACEHOLDERS.tests}
      class="flex-1 w-full bg-apple-surface-black p-3 font-mono text-xs text-apple-body-on-dark resize-none focus:outline-none"
    ></textarea>
  </div>
</div>
