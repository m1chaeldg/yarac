<script lang="ts">
  import { workspaceStore } from '../stores/workspace.svelte';
  import ParamsTab from './editor/ParamsTab.svelte';
  import HeadersTab from './editor/HeadersTab.svelte';
  import VariablesTab from './editor/VariablesTab.svelte';
  import AuthTab from './editor/AuthTab.svelte';
  import BodyTab from './editor/BodyTab.svelte';
  import ScriptsTab from './editor/ScriptsTab.svelte';
  import AssertTab from './editor/AssertTab.svelte';
  import TestsTab from './editor/TestsTab.svelte';
  import SettingsTab from './editor/SettingsTab.svelte';
  import FlowPanel from './FlowPanel.svelte';
  import { COMMON_HEADER_PRESETS, createEmptyHeader } from '../utils/editorUtils';

  let activeSubTab = $state<'params' | 'headers' | 'variables' | 'auth' | 'body' | 'scripts' | 'assert' | 'tests' | 'settings' | 'flow'>('headers');

  let req = $derived(workspaceStore.activeRequest);

  function addHeader(presetKey = '') {
    if (!req) return;
    req.headers.push(createEmptyHeader(presetKey));
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }
</script>

{#if req}
  <div class="h-full flex flex-col bg-slate-950 text-slate-200 text-xs overflow-hidden">
    <!-- Sub-Tab Navigation Bar -->
    <div class="h-9 border-b border-slate-800 bg-slate-900/60 px-3 flex items-center justify-between shrink-0 overflow-x-auto">
      <div class="flex items-center gap-1">
        <!-- PARAMS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'params'}
          class="px-2 py-1 rounded font-medium transition-colors cursor-pointer {activeSubTab === 'params' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Params</span>
          {#if req.params.filter(p => p.enabled && p.key).length > 0}
            <span class="ml-1 text-[10px] px-1 rounded-full bg-sky-950 text-sky-400 border border-sky-800/60">
              {req.params.filter(p => p.enabled && p.key).length}
            </span>
          {/if}
        </button>

        <!-- HEADERS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'headers'}
          class="px-2 py-1 rounded font-medium transition-colors cursor-pointer {activeSubTab === 'headers' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Headers</span>
          {#if req.headers.filter(h => h.enabled && h.key).length > 0}
            <span class="ml-1 text-[10px] px-1 rounded-full bg-sky-950 text-sky-400 border border-sky-800/60">
              {req.headers.filter(h => h.enabled && h.key).length}
            </span>
          {/if}
        </button>

        <!-- VARIABLES -->
        <button
          type="button"
          onclick={() => activeSubTab = 'variables'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'variables' ? 'bg-slate-800 text-amber-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Variables</span>
          {#if (req.variables || []).filter(v => v.enabled && v.key).length > 0}
            <span class="text-[10px] px-1 rounded-full bg-amber-950 text-amber-400 border border-amber-800/60">
              {(req.variables || []).filter(v => v.enabled && v.key).length}
            </span>
          {/if}
        </button>

        <!-- AUTH -->
        <button
          type="button"
          onclick={() => activeSubTab = 'auth'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'auth' ? 'bg-slate-800 text-emerald-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Auth</span>
          {#if req.auth.type !== 'none' && req.auth.type !== 'inherit'}
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
          {/if}
        </button>

        <!-- BODY -->
        <button
          type="button"
          onclick={() => activeSubTab = 'body'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'body' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Body</span>
          {#if req.body.type !== 'none'}
            <span class="text-[10px] px-1 rounded bg-slate-800 text-amber-400 font-mono uppercase">
              {req.body.type}
            </span>
          {/if}
        </button>

        <!-- SCRIPTS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'scripts'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'scripts' ? 'bg-slate-800 text-indigo-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Scripts</span>
          {#if (req.scripts?.preRequest?.trim() || req.scripts?.postResponse?.trim())}
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
          {/if}
        </button>

        <!-- ASSERT -->
        <button
          type="button"
          onclick={() => activeSubTab = 'assert'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'assert' ? 'bg-slate-800 text-teal-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Assert</span>
          {#if req.assert?.trim()}
            <span class="w-1.5 h-1.5 rounded-full bg-teal-400"></span>
          {/if}
        </button>

        <!-- TESTS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'tests'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'tests' ? 'bg-slate-800 text-pink-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Tests</span>
          {#if req.tests?.trim()}
            <span class="w-1.5 h-1.5 rounded-full bg-pink-400"></span>
          {/if}
        </button>

        <!-- SETTINGS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'settings'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'settings' ? 'bg-slate-800 text-cyan-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Settings</span>
        </button>

        <!-- FLOW -->
        <button
          type="button"
          onclick={() => activeSubTab = 'flow'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'flow' ? 'bg-slate-800 text-purple-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Flow</span>
          {#if req.flow.once_per_run || req.flow.retry_until}
            <span class="w-1.5 h-1.5 rounded-full bg-purple-400"></span>
          {/if}
        </button>
      </div>

      <!-- Quick Actions -->
      {#if activeSubTab === 'headers'}
        <div class="flex items-center gap-2">
          <select
            onchange={(e) => {
              const val = (e.target as HTMLSelectElement).value;
              if (val) {
                addHeader(val);
                (e.target as HTMLSelectElement).value = '';
              }
            }}
            class="bg-slate-800 border border-slate-700 text-[11px] text-slate-300 rounded px-2 py-0.5 focus:outline-none"
          >
            <option value="">+ Add Common Header</option>
            {#each COMMON_HEADER_PRESETS as h}
              <option value={h}>{h}</option>
            {/each}
          </select>
        </div>
      {/if}
    </div>

    <!-- Sub-Tab Content Viewport -->
    <div class="flex-1 overflow-y-auto p-3">
      {#if activeSubTab === 'params'}
        <ParamsTab {req} />
      {:else if activeSubTab === 'headers'}
        <HeadersTab {req} />
      {:else if activeSubTab === 'variables'}
        <VariablesTab {req} />
      {:else if activeSubTab === 'auth'}
        <AuthTab {req} />
      {:else if activeSubTab === 'body'}
        <BodyTab {req} />
      {:else if activeSubTab === 'scripts'}
        <ScriptsTab {req} />
      {:else if activeSubTab === 'assert'}
        <AssertTab {req} />
      {:else if activeSubTab === 'tests'}
        <TestsTab {req} />
      {:else if activeSubTab === 'settings'}
        <SettingsTab {req} />
      {:else if activeSubTab === 'flow'}
        <div class="h-full border border-slate-800 rounded-lg overflow-hidden bg-slate-900/30">
          <FlowPanel
            flow={req.flow}
            onchange={(updated) => {
              if (req) {
                req.flow = updated;
                workspaceStore.updateActiveRequest({ flow: updated });
              }
            }}
          />
        </div>
      {/if}
    </div>
  </div>
{/if}
