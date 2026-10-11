<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';
  import { createEmptyParam } from '../../utils/editorUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  function addParam() {
    req.params.push(createEmptyParam());
    workspaceStore.updateActiveRequest({ params: req.params });
  }

  function removeParam(index: number) {
    req.params.splice(index, 1);
    workspaceStore.updateActiveRequest({ params: req.params });
  }
</script>

<div class="space-y-3 font-mono text-xs">
  <div class="flex items-center justify-between">
    <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider font-sans">Query Parameters</span>
    <button
      type="button"
      onclick={addParam}
      class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60 text-xs font-medium font-sans cursor-pointer"
    >
      + Add Parameter
    </button>
  </div>

  <div class="border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black">
    <table class="w-full border-collapse text-left">
      <thead>
        <tr class="bg-apple-surface-tile-2 text-apple-body-muted border-b border-apple-hairline-dark/60 text-[11px]">
          <th class="w-8 p-2 text-center"></th>
          <th class="p-2 border-r border-apple-hairline-dark/60">Parameter Key</th>
          <th class="p-2 border-r border-apple-hairline-dark/60">Value</th>
          <th class="p-2 border-r border-apple-hairline-dark/60 font-sans">Description</th>
          <th class="w-10 p-2 text-center"></th>
        </tr>
      </thead>
      <tbody>
        {#each req.params as param, idx (param.id)}
          <tr class="border-b border-apple-hairline-dark/40 hover:bg-apple-surface-tile-1/50 transition-colors">
            <td class="p-2 text-center">
              <input
                type="checkbox"
                bind:checked={param.enabled}
                onchange={() => workspaceStore.updateActiveRequest({ params: req.params })}
                class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60">
              <input
                type="text"
                bind:value={param.key}
                placeholder="e.g. limit"
                oninput={() => {
                  if (param.key.endsWith('~')) {
                    param.key = param.key.slice(0, -1);
                    param.enabled = false;
                  } else if (param.key.startsWith('~')) {
                    param.key = param.key.slice(1);
                    param.enabled = false;
                  }
                  workspaceStore.updateActiveRequest({ params: req.params });
                }}
                class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60">
              <input
                type="text"
                bind:value={param.value}
                placeholder="e.g. 50"
                oninput={() => workspaceStore.updateActiveRequest({ params: req.params })}
                class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60 font-sans">
              <input
                type="text"
                bind:value={param.description}
                placeholder="Description"
                oninput={() => workspaceStore.updateActiveRequest({ params: req.params })}
                class="w-full bg-transparent px-2 py-1 text-apple-body-muted focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-2 text-center">
              <button
                type="button"
                onclick={() => removeParam(idx)}
                class="text-apple-ink-muted-48 hover:text-rose-400 font-sans transition-colors"
              >
                ✕
              </button>
            </td>
          </tr>
        {/each}
        {#if req.params.length === 0}
          <tr>
            <td colspan="5" class="p-4 text-center text-apple-ink-muted-48 font-sans">
              No query parameters configured. Click "+ Add Parameter" above.
            </td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>
</div>
