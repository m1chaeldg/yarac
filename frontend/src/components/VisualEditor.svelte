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

  let activeSubTab = $state<'body' | 'params' | 'headers' | 'variables' | 'auth' | 'scripts' | 'assert' | 'tests' | 'settings' | 'flow'>('body');

  let req = $derived(workspaceStore.activeRequest);

  function addHeader(presetKey = '') {
    if (!req) return;
    req.headers.push(createEmptyHeader(presetKey));
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }
</script>

{#if req}
  <div class="h-full flex flex-col bg-apple-surface-tile-1 text-apple-body-on-dark text-xs overflow-hidden">
    <!-- Sub-Tab Navigation Bar -->
    <div class="h-9 border-b border-apple-hairline-dark/60 bg-apple-surface-tile-2 px-3 flex items-center justify-between shrink-0 overflow-x-auto">
      <div class="flex items-center gap-1">
        <!-- BODY -->
        <button
          type="button"
          onclick={() => activeSubTab = 'body'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'body' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Body</span>
          {#if req.body.type !== 'none'}
            <span class="text-[10px] px-1.5 py-0.2 rounded-apple-pill bg-apple-surface-tile-1 text-amber-400 font-mono uppercase border border-apple-hairline-dark/60">
              {req.body.type}
            </span>
          {/if}
        </button>

        <!-- PARAMS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'params'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors cursor-pointer {activeSubTab === 'params' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Params</span>
          {#if req.params.filter(p => p.enabled && p.key).length > 0}
            <span class="ml-1 text-[10px] px-1.5 rounded-apple-pill bg-apple-primary/20 text-apple-primary-on-dark border border-apple-primary/40">
              {req.params.filter(p => p.enabled && p.key).length}
            </span>
          {/if}
        </button>

        <!-- HEADERS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'headers'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors cursor-pointer {activeSubTab === 'headers' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Headers</span>
          {#if req.headers.filter(h => h.enabled && h.key).length > 0}
            <span class="ml-1 text-[10px] px-1.5 rounded-apple-pill bg-apple-primary/20 text-apple-primary-on-dark border border-apple-primary/40">
              {req.headers.filter(h => h.enabled && h.key).length}
            </span>
          {/if}
        </button>

        <!-- VARIABLES -->
        <button
          type="button"
          onclick={() => activeSubTab = 'variables'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'variables' ? 'bg-apple-surface-tile-3 text-amber-400 font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Variables</span>
          {#if (req.variables || []).filter(v => v.enabled && v.key).length > 0}
            <span class="text-[10px] px-1.5 rounded-apple-pill bg-amber-950/80 text-amber-400 border border-amber-800/80">
              {(req.variables || []).filter(v => v.enabled && v.key).length}
            </span>
          {/if}
        </button>

        <!-- AUTH -->
        <button
          type="button"
          onclick={() => activeSubTab = 'auth'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'auth' ? 'bg-apple-surface-tile-3 text-emerald-400 font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Auth</span>
          {#if req.auth.type !== 'none' && req.auth.type !== 'inherit'}
            <span class="w-1.5 h-1.5 rounded-apple-full bg-emerald-400"></span>
          {/if}
        </button>

        <!-- SCRIPTS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'scripts'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'scripts' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Scripts</span>
          {#if (req.scripts?.preRequest?.trim() || req.scripts?.postResponse?.trim())}
            <span class="w-1.5 h-1.5 rounded-apple-full bg-apple-primary-on-dark"></span>
          {/if}
        </button>

        <!-- ASSERT -->
        <button
          type="button"
          onclick={() => activeSubTab = 'assert'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'assert' ? 'bg-apple-surface-tile-3 text-teal-400 font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Assert</span>
          {#if req.assert?.trim()}
            <span class="w-1.5 h-1.5 rounded-apple-full bg-teal-400"></span>
          {/if}
        </button>

        <!-- TESTS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'tests'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'tests' ? 'bg-apple-surface-tile-3 text-pink-400 font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Tests</span>
          {#if req.tests?.trim()}
            <span class="w-1.5 h-1.5 rounded-apple-full bg-pink-400"></span>
          {/if}
        </button>

        <!-- SETTINGS -->
        <button
          type="button"
          onclick={() => activeSubTab = 'settings'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'settings' ? 'bg-apple-surface-tile-3 text-slate-200 font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Settings</span>
        </button>

        <!-- FLOW -->
        <button
          type="button"
          onclick={() => activeSubTab = 'flow'}
          class="px-2.5 py-1 rounded-apple-sm font-medium transition-colors flex items-center gap-1 cursor-pointer {activeSubTab === 'flow' ? 'bg-apple-surface-tile-3 text-purple-400 font-semibold' : 'text-apple-body-muted hover:text-white'}"
        >
          <span>Flow</span>
          {#if req.flow.once_per_run || req.flow.retry_until}
            <span class="w-1.5 h-1.5 rounded-apple-full bg-purple-400"></span>
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
            class="bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-[11px] text-slate-200 rounded-apple-sm px-2.5 py-0.5 focus:outline-none focus:border-apple-primary"
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
      {#if activeSubTab === 'body'}
        <BodyTab {req} />
      {:else if activeSubTab === 'params'}
        <ParamsTab {req} />
      {:else if activeSubTab === 'headers'}
        <HeadersTab {req} />
      {:else if activeSubTab === 'variables'}
        <VariablesTab {req} />
      {:else if activeSubTab === 'auth'}
        <AuthTab {req} />
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
