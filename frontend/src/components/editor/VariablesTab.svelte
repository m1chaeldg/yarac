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
      <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider">Request-Level Variables</span>
      <p class="text-[11px] text-apple-ink-muted-48 mt-0.5">
        Variables defined here take precedence for this request. Use <code class="text-apple-primary-on-dark font-mono">{"{{var_name}}"}</code> in URL, headers, or body.
      </p>
    </div>
    <button
      type="button"
      onclick={addVariable}
      class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60 text-xs font-medium cursor-pointer"
    >
      + Add Variable
    </button>
  </div>

  <div class="border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black font-mono">
    <table class="w-full border-collapse text-left">
      <thead>
        <tr class="bg-apple-surface-tile-2 text-apple-body-muted border-b border-apple-hairline-dark/60 text-[11px]">
          <th class="w-8 p-2 text-center"></th>
          <th class="p-2 border-r border-apple-hairline-dark/60">Variable Key</th>
          <th class="p-2 border-r border-apple-hairline-dark/60">Value</th>
          <th class="w-16 p-2 text-center border-r border-apple-hairline-dark/60">Secret</th>
          <th class="p-2 border-r border-apple-hairline-dark/60 font-sans">Description</th>
          <th class="w-10 p-2 text-center"></th>
        </tr>
      </thead>
      <tbody>
        {#each (req.variables || []) as v, idx (v.id)}
          <tr class="border-b border-apple-hairline-dark/40 hover:bg-apple-surface-tile-1/50 transition-colors">
            <td class="p-2 text-center">
              <input
                type="checkbox"
                bind:checked={v.enabled}
                onchange={() => workspaceStore.updateActiveRequest({ variables: req.variables })}
                class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60">
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
                class="w-full bg-transparent px-2 py-1 text-apple-primary-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60">
              <input
                type={v.isSecret ? 'password' : 'text'}
                bind:value={v.value}
                placeholder="Value"
                oninput={() => workspaceStore.updateActiveRequest({ variables: req.variables })}
                class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-1 text-center border-r border-apple-hairline-dark/60">
              <button
                type="button"
                onclick={() => {
                  v.isSecret = !v.isSecret;
                  workspaceStore.updateActiveRequest({ variables: req.variables });
                }}
                class="apple-button-dark-utility text-xs px-1.5 py-0.5 rounded-apple-sm border border-apple-hairline-dark/60 {v.isSecret ? 'text-apple-primary-on-dark bg-apple-surface-tile-3' : 'text-apple-ink-muted-48 hover:text-white'}"
                title={v.isSecret ? 'Secret masked' : 'Plain text'}
              >
                {v.isSecret ? '🔒' : '👁'}
              </button>
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60 font-sans">
              <input
                type="text"
                bind:value={v.description}
                placeholder="Description"
                oninput={() => workspaceStore.updateActiveRequest({ variables: req.variables })}
                class="w-full bg-transparent px-2 py-1 text-apple-body-muted focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-2 text-center">
              <button
                type="button"
                onclick={() => removeVariable(idx)}
                class="text-apple-ink-muted-48 hover:text-rose-400 transition-colors"
              >
                ✕
              </button>
            </td>
          </tr>
        {/each}
        {#if (!req.variables || req.variables.length === 0)}
          <tr>
            <td colspan="6" class="p-4 text-center text-apple-ink-muted-48 font-sans">
              No request variables defined. Click "+ Add Variable" to scope variables to this request.
            </td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>

  <!-- Dynamic Variables Cheat Sheet -->
  <div class="mt-4 p-3.5 bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-lg">
    <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider block mb-2 font-sans">
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
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-pill bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-primary-on-dark border border-apple-hairline-dark/60 text-[10px] font-mono transition-colors flex items-center gap-1 cursor-pointer"
        >
          <span>{`{{${dv.label}}}`}</span>
          <span class="text-apple-ink-muted-48 text-[9px]">{dv.desc}</span>
        </button>
      {/each}
    </div>
  </div>
</div>
