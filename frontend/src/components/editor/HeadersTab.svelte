<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';
  import { createEmptyHeader, COMMON_HEADER_PRESETS } from '../../utils/editorUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  function addHeader(presetKey = '') {
    req.headers.push(createEmptyHeader(presetKey));
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }

  function removeHeader(index: number) {
    req.headers.splice(index, 1);
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }
</script>

<div class="space-y-3 font-mono text-xs">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider font-sans">HTTP Request Headers</span>
      <select
        onchange={(e) => {
          const val = (e.target as HTMLSelectElement).value;
          if (val) {
            addHeader(val);
            (e.target as HTMLSelectElement).value = '';
          }
        }}
        class="bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-[11px] text-apple-body-on-dark rounded-apple-sm px-2.5 py-0.5 focus:outline-none focus:border-apple-primary font-sans"
      >
        <option value="">+ Add Common Header</option>
        {#each COMMON_HEADER_PRESETS as h}
          <option value={h}>{h}</option>
        {/each}
      </select>
    </div>

    <button
      type="button"
      onclick={() => addHeader()}
      class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60 text-xs font-medium font-sans cursor-pointer"
    >
      + Add Header
    </button>
  </div>

  <div class="border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black">
    <table class="w-full border-collapse text-left">
      <thead>
        <tr class="bg-apple-surface-tile-2 text-apple-body-muted border-b border-apple-hairline-dark/60 text-[11px]">
          <th class="w-8 p-2 text-center"></th>
          <th class="p-2 border-r border-apple-hairline-dark/60">Header Key</th>
          <th class="p-2 border-r border-apple-hairline-dark/60">Header Value</th>
          <th class="p-2 border-r border-apple-hairline-dark/60 font-sans">Description</th>
          <th class="w-10 p-2 text-center"></th>
        </tr>
      </thead>
      <tbody>
        {#each req.headers as header, idx (header.id)}
          <tr class="border-b border-apple-hairline-dark/40 hover:bg-apple-surface-tile-1/50 transition-colors">
            <td class="p-2 text-center">
              <input
                type="checkbox"
                bind:checked={header.enabled}
                onchange={() => workspaceStore.updateActiveRequest({ headers: req.headers })}
                class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60">
              <input
                type="text"
                bind:value={header.key}
                placeholder="e.g. Content-Type"
                oninput={() => {
                  if (header.key.endsWith('~')) {
                    header.key = header.key.slice(0, -1);
                    header.enabled = false;
                  } else if (header.key.startsWith('~')) {
                    header.key = header.key.slice(1);
                    header.enabled = false;
                  }
                  workspaceStore.updateActiveRequest({ headers: req.headers });
                }}
                class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60">
              <input
                type="text"
                bind:value={header.value}
                placeholder="e.g. application/json"
                oninput={() => workspaceStore.updateActiveRequest({ headers: req.headers })}
                class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-1 border-r border-apple-hairline-dark/60 font-sans">
              <input
                type="text"
                bind:value={header.description}
                placeholder="Description"
                oninput={() => workspaceStore.updateActiveRequest({ headers: req.headers })}
                class="w-full bg-transparent px-2 py-1 text-apple-body-muted focus:outline-none placeholder:text-apple-ink-muted-48"
              />
            </td>
            <td class="p-2 text-center">
              <button
                type="button"
                onclick={() => removeHeader(idx)}
                class="text-apple-ink-muted-48 hover:text-rose-400 font-sans transition-colors"
              >
                ✕
              </button>
            </td>
          </tr>
        {/each}
        {#if req.headers.length === 0}
          <tr>
            <td colspan="5" class="p-4 text-center text-apple-ink-muted-48 font-sans">
              No headers defined. Click "+ Add Header" to add custom request headers.
            </td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>
</div>
