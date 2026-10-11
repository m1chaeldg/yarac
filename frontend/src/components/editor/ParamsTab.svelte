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
    <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider font-sans">Query Parameters</span>
    <button
      type="button"
      onclick={addParam}
      class="text-xs text-sky-400 hover:text-sky-300 font-medium font-sans"
    >
      + Add Parameter
    </button>
  </div>

  <table class="w-full border-collapse border border-slate-800 text-left">
    <thead>
      <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
        <th class="w-8 p-2 text-center"></th>
        <th class="p-2 border-r border-slate-800">Parameter Key</th>
        <th class="p-2 border-r border-slate-800">Value</th>
        <th class="p-2 border-r border-slate-800 font-sans">Description</th>
        <th class="w-10 p-2 text-center"></th>
      </tr>
    </thead>
    <tbody>
      {#each req.params as param, idx (param.id)}
        <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
          <td class="p-2 text-center">
            <input
              type="checkbox"
              bind:checked={param.enabled}
              onchange={() => workspaceStore.updateActiveRequest({ params: req.params })}
              class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
            />
          </td>
          <td class="p-1 border-r border-slate-800">
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
              class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
            />
          </td>
          <td class="p-1 border-r border-slate-800">
            <input
              type="text"
              bind:value={param.value}
              placeholder="e.g. 50"
              oninput={() => workspaceStore.updateActiveRequest({ params: req.params })}
              class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
            />
          </td>
          <td class="p-1 border-r border-slate-800 font-sans">
            <input
              type="text"
              bind:value={param.description}
              placeholder="Description"
              oninput={() => workspaceStore.updateActiveRequest({ params: req.params })}
              class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
            />
          </td>
          <td class="p-2 text-center">
            <button
              type="button"
              onclick={() => removeParam(idx)}
              class="text-slate-500 hover:text-rose-400 font-sans"
            >
              ✕
            </button>
          </td>
        </tr>
      {/each}
      {#if req.params.length === 0}
        <tr>
          <td colspan="5" class="p-4 text-center text-slate-500 font-sans">
            No query parameters configured. Click "+ Add Parameter" above.
          </td>
        </tr>
      {/if}
    </tbody>
  </table>
</div>
