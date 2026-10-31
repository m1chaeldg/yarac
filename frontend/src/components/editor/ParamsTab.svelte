<script lang="ts">
  import { workspaceStore, type YaracRequest, type KeyValuePair } from '../../stores/workspace.svelte';
  import { createEmptyParam } from '../../utils/editorUtils';
  import { extractPathVariables, buildUrlWithQueryParams } from '../../utils/urlUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let detectedPathVarNames = $derived(extractPathVariables(req.url));

  // Sync detected path variable names into req.pathVariables
  $effect(() => {
    if (!req.pathVariables) req.pathVariables = [];
    const existingMap = new Map<string, KeyValuePair>();
    for (const pv of req.pathVariables) {
      if (pv.key) existingMap.set(pv.key, pv);
    }

    const updatedList: KeyValuePair[] = [];
    for (const name of detectedPathVarNames) {
      const existing = existingMap.get(name);
      if (existing) {
        updatedList.push(existing);
      } else {
        updatedList.push({
          id: Math.random().toString(36).substring(2, 9),
          key: name,
          value: '',
          enabled: true,
          description: `Path parameter :${name}`
        });
      }
    }
    if (updatedList.length > 0 || req.pathVariables.length > 0) {
      req.pathVariables = updatedList;
    }
  });

  function addParam() {
    req.params.push(createEmptyParam());
    updateParamsAndSyncUrl();
  }

  function removeParam(index: number) {
    req.params.splice(index, 1);
    updateParamsAndSyncUrl();
  }

  function updateParamsAndSyncUrl() {
    const updatedUrl = buildUrlWithQueryParams(req.url, req.params);
    req.url = updatedUrl;
    workspaceStore.updateActiveRequest({ params: req.params, url: updatedUrl });
  }

  function updatePathVariable() {
    workspaceStore.updateActiveRequest({ pathVariables: req.pathVariables });
  }
</script>

<div class="space-y-6 font-mono text-xs">
  <!-- Query Parameters Section -->
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <div>
        <span class="text-[11px] font-semibold text-apple-body-muted uppercase tracking-wider font-sans">Query Parameters</span>
        <p class="text-[11px] text-apple-ink-muted-48 font-sans">
          Key-value parameters appended to the query string (<code class="text-apple-primary-on-dark font-mono">?key=value</code>).
        </p>
      </div>
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
                  onchange={updateParamsAndSyncUrl}
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
                    updateParamsAndSyncUrl();
                  }}
                  class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
                />
              </td>
              <td class="p-1 border-r border-apple-hairline-dark/60">
                <input
                  type="text"
                  bind:value={param.value}
                  placeholder="e.g. 50"
                  oninput={updateParamsAndSyncUrl}
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
                  class="text-apple-ink-muted-48 hover:text-rose-400 font-sans transition-colors cursor-pointer"
                  title="Remove parameter"
                >
                  ✕
                </button>
              </td>
            </tr>
          {/each}
          {#if req.params.length === 0}
            <tr>
              <td colspan="5" class="p-4 text-center text-apple-ink-muted-48 font-sans">
                No query parameters configured. Click "+ Add Parameter" above or enter them in the URL.
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Path Variables Section (Dynamic when detected in URL) -->
  {#if (req.pathVariables && req.pathVariables.length > 0) || detectedPathVarNames.length > 0}
    <div class="space-y-3 pt-2 border-t border-apple-hairline-dark/40">
      <div class="flex items-center justify-between">
        <div>
          <span class="text-[11px] font-semibold text-purple-400 uppercase tracking-wider font-sans">
            Path Variables ({detectedPathVarNames.length})
          </span>
          <p class="text-[11px] text-apple-ink-muted-48 font-sans">
            Variables embedded in the URL path (e.g. <code class="text-purple-300 font-mono">:userId</code> or <code class="text-purple-300 font-mono">&#123;orderId&#125;</code>) evaluated on execution.
          </p>
        </div>
      </div>

      <div class="border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black">
        <table class="w-full border-collapse text-left">
          <thead>
            <tr class="bg-apple-surface-tile-2 text-apple-body-muted border-b border-apple-hairline-dark/60 text-[11px]">
              <th class="p-2 border-r border-apple-hairline-dark/60 w-1/3">Variable Name</th>
              <th class="p-2 border-r border-apple-hairline-dark/60">Replacement Value</th>
              <th class="p-2 font-sans">Description</th>
            </tr>
          </thead>
          <tbody>
            {#each (req.pathVariables || []) as pv (pv.id)}
              <tr class="border-b border-apple-hairline-dark/40 hover:bg-apple-surface-tile-1/50 transition-colors">
                <td class="p-2 border-r border-apple-hairline-dark/60 font-bold text-purple-400">
                  :{pv.key}
                </td>
                <td class="p-1 border-r border-apple-hairline-dark/60">
                  <input
                    type="text"
                    bind:value={pv.value}
                    placeholder={`Value for :${pv.key}`}
                    oninput={updatePathVariable}
                    class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
                  />
                </td>
                <td class="p-1 font-sans">
                  <input
                    type="text"
                    bind:value={pv.description}
                    placeholder="Description"
                    oninput={updatePathVariable}
                    class="w-full bg-transparent px-2 py-1 text-apple-body-muted focus:outline-none placeholder:text-apple-ink-muted-48"
                  />
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>
