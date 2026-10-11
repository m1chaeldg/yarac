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
      <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider">JavaScript Assertions</span>
      <p class="text-[11px] text-apple-ink-muted-48">
        Lightweight assertion statements evaluated directly on the response. Test results display in the Response tab.
      </p>
    </div>

    <!-- Snippets -->
    <div class="flex items-center gap-1.5 flex-wrap">
      {#each ASSERT_SNIPPETS as s}
        <button
          type="button"
          onclick={() => insertAssertSnippet(s.snippet)}
          class="apple-button-dark-utility px-2.5 py-0.5 rounded-apple-sm text-[10px] font-mono text-apple-primary-on-dark border border-apple-hairline-dark/60 transition-colors cursor-pointer"
        >
          {s.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="flex-1 flex flex-col border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black min-h-[220px]">
    <div class="bg-apple-surface-tile-2 px-3 py-1.5 border-b border-apple-hairline-dark/60 flex items-center justify-between text-[11px] text-apple-body-muted">
      <span class="font-mono text-apple-primary-on-dark">// Available matchers: toBe, toEqual, toBeDefined, toBeNull, toBeGreaterThan, toContain</span>
      <span class="text-apple-ink-muted-48 font-mono text-[10px]">Async/Await Default • Live Assertions</span>
    </div>
    <textarea
      value={req.assert || ''}
      oninput={(e) => {
        req.assert = (e.target as HTMLTextAreaElement).value;
        workspaceStore.updateActiveRequest({ assert: req.assert });
      }}
      placeholder={SCRIPT_PLACEHOLDERS.assert}
      class="flex-1 w-full bg-apple-surface-black p-3 font-mono text-xs text-apple-body-on-dark resize-none focus:outline-none"
    ></textarea>
  </div>
</div>
