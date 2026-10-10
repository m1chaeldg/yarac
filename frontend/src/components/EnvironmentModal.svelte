<script lang="ts">
  import { workspaceStore, type EnvironmentProfile, type KeyValuePair } from '../stores/workspace.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let selectedEnvId = $state<string>('');
  let newEnvName = $state('');
  let isAddingEnv = $state(false);
  let copyMenuVarId = $state<string | null>(null);

  // Initialize selected env when opened or changed
  $effect(() => {
    if (isOpen && !selectedEnvId) {
      selectedEnvId = workspaceStore.currentWorkspace.activeEnvironmentId || workspaceStore.currentWorkspace.environments[0]?.id || '';
    }
  });

  let currentEnv = $derived(
    workspaceStore.currentWorkspace.environments.find(e => e.id === selectedEnvId) ||
    workspaceStore.currentWorkspace.environments[0]
  );

  let otherEnvs = $derived(
    workspaceStore.currentWorkspace.environments.filter(e => e.id !== selectedEnvId)
  );

  function createEnv() {
    if (!newEnvName.trim()) return;
    const created = workspaceStore.addEnvironment(newEnvName);
    selectedEnvId = created.id;
    newEnvName = '';
    isAddingEnv = false;
  }

  function handleDuplicate(envId: string) {
    const dup = workspaceStore.duplicateEnvironment(envId);
    if (dup) {
      selectedEnvId = dup.id;
    }
  }

  function addVariable() {
    if (!currentEnv) return;
    currentEnv.variables.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      isSecret: false
    });
  }

  function removeVariable(idx: number) {
    if (!currentEnv) return;
    currentEnv.variables.splice(idx, 1);
  }

  function copyVarToTarget(v: KeyValuePair, targetEnvId: string) {
    workspaceStore.copyVariableToEnvironment(v, targetEnvId);
    copyMenuVarId = null;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 font-sans animate-fade-in">
    <div class="bg-slate-900 border border-slate-700 w-full max-w-4xl h-[600px] rounded-xl shadow-2xl flex flex-col overflow-hidden text-slate-100">
      <!-- Modal Header -->
      <div class="h-12 border-b border-slate-800 bg-slate-950/80 px-4 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2.5">
          <span class="text-emerald-400 text-base">🌱</span>
          <h2 class="text-sm font-bold tracking-wide text-slate-100">Environment Profiles Manager</h2>
          <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800">
            Bruno Style
          </span>
        </div>
        <button
          type="button"
          aria-label="Close environment manager modal"
          onclick={onClose}
          class="text-slate-400 hover:text-slate-100 p-1.5 rounded hover:bg-slate-800 text-xs font-bold"
        >
          ✕
        </button>
      </div>

      <!-- Modal Body Split Pane -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Left: Environment Profile List -->
        <div class="w-60 border-r border-slate-800 bg-slate-950/40 p-3 flex flex-col justify-between shrink-0">
          <div class="space-y-2">
            <div class="flex items-center justify-between mb-2">
              <span class="text-[11px] font-semibold text-slate-500 uppercase tracking-wider">Profiles</span>
              <button
                type="button"
                onclick={() => isAddingEnv = true}
                class="text-xs text-sky-400 hover:text-sky-300 font-medium"
              >
                + New
              </button>
            </div>

            {#if isAddingEnv}
              <div class="p-2 rounded bg-slate-900 border border-slate-700 space-y-2">
                <input
                  type="text"
                  bind:value={newEnvName}
                  placeholder="Profile Name..."
                  class="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-slate-100 focus:outline-none focus:border-sky-500"
                />
                <div class="flex gap-1.5 justify-end">
                  <button
                    type="button"
                    onclick={() => isAddingEnv = false}
                    class="px-2 py-0.5 text-[11px] rounded hover:bg-slate-800 text-slate-400"
                  >
                    Cancel
                  </button>
                  <button
                    type="button"
                    onclick={createEnv}
                    class="px-2 py-0.5 text-[11px] rounded bg-sky-600 hover:bg-sky-500 text-white font-medium"
                  >
                    Create
                  </button>
                </div>
              </div>
            {/if}

            <div class="space-y-1">
              {#each workspaceStore.currentWorkspace.environments as env (env.id)}
                {@const isSelected = selectedEnvId === env.id}
                {@const isActive = workspaceStore.currentWorkspace.activeEnvironmentId === env.id}
                <div
                  role="button"
                  tabindex="0"
                  onclick={() => selectedEnvId = env.id}
                  onkeydown={(e) => e.key === 'Enter' && (selectedEnvId = env.id)}
                  class="w-full text-left p-2 rounded-lg text-xs transition-colors flex items-center justify-between group cursor-pointer
                    {isSelected
                      ? 'bg-sky-950/60 text-sky-200 border border-sky-800/60 font-semibold'
                      : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
                >
                  <div class="flex items-center gap-2 truncate">
                    <span class="truncate">{env.name}</span>
                    {#if isActive}
                      <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0" title="Active Environment"></span>
                    {/if}
                  </div>

                  <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <!-- Duplicate Button -->
                    <button
                      type="button"
                      title="Duplicate Environment"
                      onclick={(e) => { e.stopPropagation(); handleDuplicate(env.id); }}
                      class="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-sky-300"
                    >
                      ⎘
                    </button>

                    <!-- Delete Button -->
                    {#if workspaceStore.currentWorkspace.environments.length > 1}
                      <button
                        type="button"
                        title="Delete Environment"
                        onclick={(e) => { e.stopPropagation(); workspaceStore.deleteEnvironment(env.id); }}
                        class="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-rose-400"
                      >
                        ✕
                      </button>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>

          <div class="text-[11px] text-slate-500 border-t border-slate-800 pt-2 font-mono">
            Keyring Masking Active
          </div>
        </div>

        <!-- Right: Environment Variables Table & Actions -->
        {#if currentEnv}
          <div class="flex-1 flex flex-col p-4 overflow-hidden">
            <div class="flex items-center justify-between mb-3 shrink-0">
              <div class="flex items-center gap-3">
                <input
                  type="text"
                  bind:value={currentEnv.name}
                  class="bg-transparent text-sm font-bold text-slate-100 border-b border-transparent hover:border-slate-700 focus:border-sky-500 px-1 py-0.5 focus:outline-none"
                />
                {#if workspaceStore.currentWorkspace.activeEnvironmentId === currentEnv.id}
                  <span class="px-2 py-0.5 rounded text-[10px] bg-emerald-950 text-emerald-400 font-bold border border-emerald-800">
                    Active Profile
                  </span>
                {:else}
                  <button
                    type="button"
                    onclick={() => workspaceStore.setActiveEnvironment(currentEnv?.id || '')}
                    class="text-[11px] text-emerald-400 hover:text-emerald-300 px-2 py-0.5 rounded bg-emerald-950/40 border border-emerald-800/40"
                  >
                    Activate Profile
                  </button>
                {/if}
              </div>

              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={addVariable}
                  class="text-xs bg-sky-600 hover:bg-sky-500 text-white font-medium px-3 py-1 rounded shadow-sm transition-colors"
                >
                  + Add Variable
                </button>
              </div>
            </div>

            <!-- Variables Table -->
            <div class="flex-1 overflow-auto border border-slate-800 rounded-lg">
              <table class="w-full border-collapse text-left font-mono text-xs">
                <thead class="sticky top-0 bg-slate-900 border-b border-slate-800 z-10">
                  <tr class="text-slate-400 text-[11px]">
                    <th class="w-8 p-2 text-center"></th>
                    <th class="p-2 border-r border-slate-800">Variable Key</th>
                    <th class="p-2 border-r border-slate-800">Variable Value</th>
                    <th class="w-20 p-2 border-r border-slate-800 text-center font-sans">Secret</th>
                    <th class="w-28 p-2 border-r border-slate-800 text-center font-sans">Copy To</th>
                    <th class="w-8 p-2 text-center"></th>
                  </tr>
                </thead>
                <tbody>
                  {#each currentEnv.variables as v, idx (v.id)}
                    <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                      <td class="p-2 text-center">
                        <input
                          type="checkbox"
                          bind:checked={v.enabled}
                          onchange={() => {
                            workspaceStore.persistToDisk();
                          }}
                          class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                        />
                      </td>
                      <td class="p-1 border-r border-slate-800">
                        <input
                          type="text"
                          bind:value={v.key}
                          placeholder="e.g. base_url"
                          oninput={() => {
                            if (v.key.endsWith('~')) {
                              v.key = v.key.slice(0, -1);
                              v.enabled = false;
                            } else if (v.key.startsWith('~')) {
                              v.key = v.key.slice(1);
                              v.enabled = false;
                            }
                            workspaceStore.persistToDisk();
                          }}
                          class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                        />
                      </td>
                      <td class="p-1 border-r border-slate-800">
                        <input
                          type={v.isSecret ? 'password' : 'text'}
                          bind:value={v.value}
                          placeholder="value"
                          class="w-full bg-transparent px-2 py-1 font-mono {v.isSecret ? 'text-amber-400' : 'text-emerald-400'} focus:outline-none"
                        />
                      </td>
                      <td class="p-1 border-r border-slate-800 text-center">
                        <label class="cursor-pointer inline-flex items-center gap-1 text-[11px] text-slate-400">
                          <input
                            type="checkbox"
                            bind:checked={v.isSecret}
                            class="rounded bg-slate-800 border-slate-700 text-amber-500 focus:ring-0"
                          />
                          <span>{v.isSecret ? '🔒' : '🔓'}</span>
                        </label>
                      </td>
                      <td class="p-1 border-r border-slate-800 text-center relative">
                        {#if otherEnvs.length > 0}
                          <button
                            type="button"
                            onclick={() => copyMenuVarId = copyMenuVarId === v.id ? null : v.id}
                            class="text-[11px] text-sky-400 hover:text-sky-300 px-2 py-0.5 rounded bg-sky-950/40 border border-sky-800/40"
                          >
                            Copy To...
                          </button>

                          {#if copyMenuVarId === v.id}
                            <div class="absolute right-0 top-8 z-30 w-44 rounded-md bg-slate-900 border border-slate-700 shadow-xl p-1 text-left">
                              <span class="block px-2 py-1 text-[10px] text-slate-500 uppercase font-bold">Target Environment</span>
                              {#each otherEnvs as oEnv}
                                <button
                                  type="button"
                                  onclick={() => copyVarToTarget(v, oEnv.id)}
                                  class="w-full text-left px-2 py-1 text-xs text-slate-300 hover:bg-slate-800 hover:text-sky-400 rounded truncate"
                                >
                                  {oEnv.name}
                                </button>
                              {/each}
                            </div>
                          {/if}
                        {:else}
                          <span class="text-[10px] text-slate-600">No other env</span>
                        {/if}
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
                  {#if currentEnv.variables.length === 0}
                    <tr>
                      <td colspan="6" class="p-8 text-center text-slate-500 font-sans">
                        No environment variables set in "{currentEnv.name}". Click "+ Add Variable" above.
                      </td>
                    </tr>
                  {/if}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="h-12 border-t border-slate-800 bg-slate-950/80 px-4 flex items-center justify-between shrink-0">
        <span class="text-[11px] text-slate-500">
          Variables are referenced in URL, headers, and body using <code class="text-sky-400">{"{{key}}"}</code> syntax.
        </span>
        <button
          type="button"
          onclick={onClose}
          class="bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold px-4 py-1.5 rounded transition-colors"
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
