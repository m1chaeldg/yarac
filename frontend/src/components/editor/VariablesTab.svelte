<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';
  import { createEmptyVariable, DYNAMIC_VARIABLES_LIST } from '../../utils/editorUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  function addVariable() {
    if (!req.variables) req.variables = [];
    req.variables.push(createEmptyVariable());
    workspaceStore.updateActiveRequest({ variables: req.variables });
  }

  function removeVariable(index: number) {
    if (!req.variables) return;
    req.variables.splice(index, 1);
    workspaceStore.updateActiveRequest({ variables: req.variables });
  }
</script>

<div class="space-y-3 font-sans text-xs">
  <div class="flex items-center justify-between">
    <div>
      <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Request-Level Variables</span>
      <p class="text-[11px] text-slate-500 mt-0.5">
        Variables defined here take precedence for this request. Use <code class="text-amber-400 font-mono">{"{{var_name}}"}</code> in URL, headers, or body.
      </p>
    </div>
    <button
      type="button"
      onclick={addVariable}
      class="text-xs text-amber-400 hover:text-amber-300 font-medium"
    >
      + Add Variable
    </button>
  </div>

  <table class="w-full border-collapse border border-slate-800 text-left font-mono">
    <thead>
      <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
        <th class="w-8 p-2 text-center"></th>
        <th class="p-2 border-r border-slate-800">Variable Key</th>
        <th class="p-2 border-r border-slate-800">Value</th>
        <th class="w-16 p-2 text-center border-r border-slate-800">Secret</th>
        <th class="p-2 border-r border-slate-800 font-sans">Description</th>
        <th class="w-10 p-2 text-center"></th>
      </tr>
    </thead>
    <tbody>
      {#each (req.variables || []) as v, idx (v.id)}
        <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
          <td class="p-2 text-center">
            <input
              type="checkbox"
              bind:checked={v.enabled}
              onchange={() => workspaceStore.updateActiveRequest({ variables: req.variables })}
              class="rounded bg-slate-800 border-slate-700 text-amber-500 focus:ring-0"
            />
          </td>
          <td class="p-1 border-r border-slate-800">
            <input
              type="text"
              bind:value={v.key}
              placeholder="e.g. user_id"
              oninput={() => {
                if (v.key.endsWith('~')) {
                  v.key = v.key.slice(0, -1);
                  v.enabled = false;
                } else if (v.key.startsWith('~')) {
                  v.key = v.key.slice(1);
                  v.enabled = false;
                }
                workspaceStore.updateActiveRequest({ variables: req.variables });
              }}
              class="w-full bg-transparent px-2 py-1 text-amber-300 focus:outline-none"
            />
          </td>
          <td class="p-1 border-r border-slate-800">
            <input
              type={v.isSecret ? 'password' : 'text'}
              bind:value={v.value}
              placeholder="Value"
              oninput={() => workspaceStore.updateActiveRequest({ variables: req.variables })}
              class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
            />
          </td>
          <td class="p-1 text-center border-r border-slate-800">
            <button
              type="button"
              onclick={() => {
                v.isSecret = !v.isSecret;
                workspaceStore.updateActiveRequest({ variables: req.variables });
              }}
              class="text-xs px-1.5 py-0.5 rounded {v.isSecret ? 'text-amber-400 bg-amber-950/60 border border-amber-800/60' : 'text-slate-500 hover:text-slate-300'}"
              title={v.isSecret ? 'Secret masked' : 'Plain text'}
            >
              {v.isSecret ? '🔒' : '👁'}
            </button>
          </td>
          <td class="p-1 border-r border-slate-800 font-sans">
            <input
              type="text"
              bind:value={v.description}
              placeholder="Description"
              oninput={() => workspaceStore.updateActiveRequest({ variables: req.variables })}
              class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
            />
          </td>
          <td class="p-2 text-center">
            <button
              type="button"
              onclick={() => removeVariable(idx)}
              class="text-slate-500 hover:text-rose-400"
            >
              ✕
            </button>
          </td>
        </tr>
      {/each}
      {#if (!req.variables || req.variables.length === 0)}
        <tr>
          <td colspan="6" class="p-4 text-center text-slate-500 font-sans">
            No request variables defined. Click "+ Add Variable" to scope variables to this request.
          </td>
        </tr>
      {/if}
    </tbody>
  </table>

  <!-- Dynamic Variables Cheat Sheet -->
  <div class="mt-4 p-3 bg-slate-900/40 border border-slate-800 rounded-lg">
    <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block mb-2">
      Bruno Dynamic Variables (Use anywhere in URLs, Headers, Body, or Variables)
    </span>
    <div class="flex flex-wrap gap-1.5">
      {#each DYNAMIC_VARIABLES_LIST as dv}
        <button
          type="button"
          onclick={() => {
            navigator.clipboard.writeText(`{{${dv.label}}}`);
          }}
          title="{dv.desc} (Click to copy)"
          class="px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 text-sky-400 border border-slate-700/60 text-[10px] font-mono transition-colors flex items-center gap-1"
        >
          <span>{`{{${dv.label}}}`}</span>
          <span class="text-slate-500 text-[9px]">{dv.desc}</span>
        </button>
      {/each}
    </div>
  </div>
</div>
