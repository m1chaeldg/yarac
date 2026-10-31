<script lang="ts">
  import { workspaceStore, type EnvironmentProfile, type KeyValuePair } from '../stores/workspace.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let activeTab = $state<'profiles' | 'global' | 'security'>('profiles');
  let selectedEnvId = $state<string>('');
  let newEnvName = $state('');
  let isAddingEnv = $state(false);
  let copyMenuVarId = $state<string | null>(null);
  let showSecrets = $state<Record<string, boolean>>({});

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

  let globalVars = $derived(
    workspaceStore.currentWorkspace.globalVariables || []
  );

  let scrubTaintedSecrets = $derived(
    workspaceStore.currentWorkspace.settings?.scrubTaintedSecrets ?? true
  );

  function toggleScrubSecrets() {
    workspaceStore.updateWorkspaceSettings({
      scrubTaintedSecrets: !scrubTaintedSecrets
    });
  }

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
      isSecret: false,
      storageTarget: 'file'
    });
    workspaceStore.persistToDisk();
  }

  function removeVariable(idx: number) {
    if (!currentEnv) return;
    currentEnv.variables.splice(idx, 1);
    workspaceStore.persistToDisk();
  }

  function addGlobalVariable() {
    const list = [...(workspaceStore.currentWorkspace.globalVariables || [])];
    list.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      isSecret: false,
      storageTarget: 'file'
    });
    workspaceStore.updateGlobalVariables(list);
  }

  function removeGlobalVariable(idx: number) {
    const list = [...(workspaceStore.currentWorkspace.globalVariables || [])];
    list.splice(idx, 1);
    workspaceStore.updateGlobalVariables(list);
  }

  function copyVarToTarget(v: KeyValuePair, targetEnvId: string) {
    workspaceStore.copyVariableToEnvironment(v, targetEnvId);
    copyMenuVarId = null;
  }

  function toggleSecretVisibility(varId: string) {
    showSecrets[varId] = !showSecrets[varId];
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 font-sans animate-fade-in">
    <div class="apple-modal-card w-full max-w-4xl h-[620px] flex flex-col text-apple-body-on-dark shadow-2xl">
      <!-- Modal Header with Navigation Tabs -->
      <div class="h-12 border-b border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-3">
          <span class="text-apple-primary-on-dark text-base">🌱</span>
          <h2 class="apple-body-strong text-apple-body-on-dark tracking-tight">Environment & Secrets</h2>

          <!-- Navigation Pills -->
          <div class="flex items-center bg-apple-surface-tile-3 rounded-apple-pill p-0.5 border border-apple-hairline-dark ml-2">
            <button
              type="button"
              onclick={() => activeTab = 'profiles'}
              class="px-3 py-1 rounded-apple-pill text-xs font-medium transition-all {activeTab === 'profiles' ? 'bg-apple-primary text-white shadow-sm' : 'text-apple-body-muted hover:text-white'}"
            >
              Profiles ({workspaceStore.currentWorkspace.environments.length})
            </button>
            <button
              type="button"
              onclick={() => activeTab = 'global'}
              class="px-3 py-1 rounded-apple-pill text-xs font-medium transition-all {activeTab === 'global' ? 'bg-apple-primary text-white shadow-sm' : 'text-apple-body-muted hover:text-white'}"
            >
              Global Scope ({globalVars.length})
            </button>
            <button
              type="button"
              onclick={() => activeTab = 'security'}
              class="px-3 py-1 rounded-apple-pill text-xs font-medium transition-all flex items-center gap-1.5 {activeTab === 'security' ? 'bg-apple-primary text-white shadow-sm' : 'text-apple-body-muted hover:text-white'}"
            >
              <span>🛡️ Security</span>
              {#if scrubTaintedSecrets}
                <span class="w-1.5 h-1.5 rounded-apple-full bg-emerald-400"></span>
              {/if}
            </button>
          </div>
        </div>

        <button
          type="button"
          aria-label="Close environment manager modal"
          onclick={onClose}
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
        >
          ✕
        </button>
      </div>

      <!-- Tab Content Area -->
      <div class="flex-1 flex overflow-hidden">
        {#if activeTab === 'profiles'}
          <!-- Left: Environment Profile List -->
          <div class="w-60 border-r border-apple-hairline-dark bg-apple-surface-tile-3 p-3 flex flex-col justify-between shrink-0">
            <div class="space-y-2">
              <div class="flex items-center justify-between mb-2">
                <span class="apple-micro-legal font-semibold text-apple-ink-muted-48 uppercase tracking-wider">Profiles</span>
                <button
                  type="button"
                  onclick={() => isAddingEnv = true}
                  class="text-xs text-apple-primary-on-dark hover:underline font-medium"
                >
                  + New
                </button>
              </div>

              {#if isAddingEnv}
                <div class="p-2 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-2">
                  <input
                    type="text"
                    bind:value={newEnvName}
                    placeholder="Profile Name..."
                    class="w-full bg-apple-surface-tile-1 border border-apple-hairline-dark rounded-apple-sm px-2.5 py-1 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary"
                  />
                  <div class="flex gap-1.5 justify-end">
                    <button
                      type="button"
                      onclick={() => isAddingEnv = false}
                      class="px-2 py-0.5 text-xs rounded-apple-sm text-apple-body-muted hover:bg-apple-surface-tile-3"
                    >
                      Cancel
                    </button>
                    <button
                      type="button"
                      onclick={createEnv}
                      class="apple-button-primary px-2.5 py-0.5 text-xs font-medium"
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
                    class="w-full text-left p-2 rounded-apple-sm text-xs transition-colors flex items-center justify-between group cursor-pointer
                      {isSelected
                        ? 'bg-apple-surface-tile-1 text-apple-primary-on-dark border border-apple-primary/40 font-semibold'
                        : 'text-apple-body-muted hover:bg-apple-surface-tile-2 hover:text-white'}"
                  >
                    <div class="flex items-center gap-2 truncate">
                      <span class="truncate">{env.name}</span>
                      {#if isActive}
                        <span class="w-1.5 h-1.5 rounded-apple-full bg-apple-primary-on-dark shrink-0" title="Active Environment"></span>
                      {/if}
                    </div>

                    <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <!-- Duplicate Button -->
                      <button
                        type="button"
                        title="Duplicate Environment"
                        onclick={(e) => { e.stopPropagation(); handleDuplicate(env.id); }}
                        class="p-1 rounded-apple-sm hover:bg-apple-surface-tile-3 text-apple-body-muted hover:text-apple-primary-on-dark"
                      >
                        ⎘
                      </button>

                      <!-- Delete Button -->
                      {#if workspaceStore.currentWorkspace.environments.length > 1}
                        <button
                          type="button"
                          title="Delete Environment"
                          onclick={(e) => { e.stopPropagation(); workspaceStore.deleteEnvironment(env.id); }}
                          class="p-1 rounded-apple-sm hover:bg-apple-surface-tile-3 text-apple-body-muted hover:text-rose-400"
                        >
                          ✕
                        </button>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            </div>

            <div class="apple-micro-legal text-apple-ink-muted-48 border-t border-apple-hairline-dark pt-2 font-mono flex items-center justify-between">
              <span>Keyring Masking</span>
              <span class="text-emerald-400">Active</span>
            </div>
          </div>

          <!-- Right: Environment Variables Table & Actions -->
          {#if currentEnv}
            <div class="flex-1 flex flex-col p-4 overflow-hidden bg-apple-surface-tile-1">
              <div class="flex items-center justify-between mb-3 shrink-0">
                <div class="flex items-center gap-3">
                  <input
                    type="text"
                    bind:value={currentEnv.name}
                    class="bg-transparent text-sm font-bold text-apple-body-on-dark border-b border-transparent hover:border-apple-hairline-dark focus:border-apple-primary px-1 py-0.5 focus:outline-none"
                  />
                  {#if workspaceStore.currentWorkspace.activeEnvironmentId === currentEnv.id}
                    <span class="px-2.5 py-0.5 rounded-apple-pill apple-micro-legal bg-apple-primary/20 text-apple-primary-on-dark font-bold border border-apple-primary/40">
                      Active Profile
                    </span>
                  {:else}
                    <button
                      type="button"
                      onclick={() => workspaceStore.setActiveEnvironment(currentEnv?.id || '')}
                      class="text-xs text-apple-primary-on-dark hover:underline px-2.5 py-0.5 rounded-apple-pill bg-apple-surface-tile-2 border border-apple-hairline-dark"
                    >
                      Activate Profile
                    </button>
                  {/if}
                </div>

                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    onclick={addVariable}
                    class="apple-button-primary text-xs px-3.5 py-1"
                  >
                    + Add Variable
                  </button>
                </div>
              </div>

              <!-- Variables Table -->
              <div class="flex-1 overflow-auto border border-apple-hairline-dark rounded-apple-sm bg-apple-surface-tile-1">
                <table class="w-full border-collapse text-left font-mono text-xs">
                  <thead class="sticky top-0 bg-apple-surface-tile-2 border-b border-apple-hairline-dark z-10">
                    <tr class="text-apple-body-muted apple-micro-legal">
                      <th class="w-8 p-2 text-center"></th>
                      <th class="w-1/4 p-2 border-r border-apple-hairline-dark">Variable Key</th>
                      <th class="w-1/3 p-2 border-r border-apple-hairline-dark">Variable Value</th>
                      <th class="w-32 p-2 border-r border-apple-hairline-dark text-center font-sans">Storage</th>
                      <th class="w-20 p-2 border-r border-apple-hairline-dark text-center font-sans">Secret</th>
                      <th class="w-24 p-2 border-r border-apple-hairline-dark text-center font-sans">Copy To</th>
                      <th class="w-8 p-2 text-center"></th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each currentEnv.variables as v, idx (v.id)}
                      {@const isRevealed = Boolean(showSecrets[v.id])}
                      <tr class="border-b border-apple-hairline-dark/60 hover:bg-apple-surface-tile-2/40">
                        <td class="p-2 text-center">
                          <input
                            type="checkbox"
                            bind:checked={v.enabled}
                            onchange={() => workspaceStore.persistToDisk()}
                            class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark text-apple-primary focus:ring-0"
                          />
                        </td>
                        <td class="p-1 border-r border-apple-hairline-dark">
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
                            class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none"
                          />
                        </td>
                        <td class="p-1 border-r border-apple-hairline-dark">
                          <div class="flex items-center gap-1">
                            <input
                              type={v.isSecret && !isRevealed ? 'password' : 'text'}
                              bind:value={v.value}
                              placeholder="value"
                              oninput={() => workspaceStore.persistToDisk()}
                              class="w-full bg-transparent px-2 py-1 font-mono {v.isSecret ? 'text-amber-400' : 'text-apple-primary-on-dark'} focus:outline-none"
                            />
                            {#if v.isSecret}
                              <button
                                type="button"
                                title={isRevealed ? 'Mask secret' : 'Reveal secret'}
                                onclick={() => toggleSecretVisibility(v.id)}
                                class="text-xs px-1 text-apple-body-muted hover:text-white"
                              >
                                {isRevealed ? '🙈' : '👁️'}
                              </button>
                            {/if}
                          </div>
                        </td>
                        <td class="p-1 border-r border-apple-hairline-dark text-center font-sans">
                          <select
                            bind:value={v.storageTarget}
                            onchange={() => workspaceStore.persistToDisk()}
                            class="bg-apple-surface-tile-3 border border-apple-hairline-dark rounded px-1.5 py-0.5 text-[11px] text-apple-body-on-dark focus:outline-none cursor-pointer"
                          >
                            <option value="file">📁 File (.env)</option>
                            <option value="keyring">🔒 OS Keyring</option>
                            <option value="local">🚫 Local (.local)</option>
                          </select>
                        </td>
                        <td class="p-1 border-r border-apple-hairline-dark text-center">
                          <label class="cursor-pointer inline-flex items-center gap-1 apple-micro-legal text-apple-body-muted">
                            <input
                              type="checkbox"
                              bind:checked={v.isSecret}
                              onchange={() => workspaceStore.persistToDisk()}
                              class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark text-amber-500 focus:ring-0"
                            />
                            <span>{v.isSecret ? '🔒' : '🔓'}</span>
                          </label>
                        </td>
                        <td class="p-1 border-r border-apple-hairline-dark text-center relative">
                          {#if otherEnvs.length > 0}
                            <button
                              type="button"
                              onclick={() => copyMenuVarId = copyMenuVarId === v.id ? null : v.id}
                              class="apple-micro-legal text-apple-primary-on-dark hover:underline px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-2 border border-apple-hairline-dark"
                            >
                              Copy To...
                            </button>

                            {#if copyMenuVarId === v.id}
                              <div class="absolute right-0 top-8 z-30 w-44 rounded-apple-md bg-apple-surface-tile-2 border border-apple-hairline-dark p-1 text-left shadow-lg">
                                <span class="block px-2 py-1 apple-micro-legal text-apple-ink-muted-48 uppercase font-bold">Target Profile</span>
                                {#each otherEnvs as oEnv}
                                  <button
                                    type="button"
                                    onclick={() => copyVarToTarget(v, oEnv.id)}
                                    class="w-full text-left px-2 py-1 text-xs text-apple-body-muted hover:bg-apple-surface-tile-3 hover:text-white rounded-apple-sm truncate"
                                  >
                                    {oEnv.name}
                                  </button>
                                {/each}
                              </div>
                            {/if}
                          {:else}
                            <span class="apple-micro-legal text-apple-ink-muted-48">—</span>
                          {/if}
                        </td>
                        <td class="p-2 text-center">
                          <button
                            type="button"
                            onclick={() => removeVariable(idx)}
                            class="text-apple-ink-muted-48 hover:text-rose-400"
                          >
                            ✕
                          </button>
                        </td>
                      </tr>
                    {/each}
                    {#if currentEnv.variables.length === 0}
                      <tr>
                        <td colspan="7" class="p-8 text-center text-apple-body-muted font-sans apple-caption">
                          No environment variables set in "{currentEnv.name}". Click "+ Add Variable" above.
                        </td>
                      </tr>
                    {/if}
                  </tbody>
                </table>
              </div>
            </div>
          {/if}

        {:else if activeTab === 'global'}
          <!-- Workspace Global Variables Pane -->
          <div class="flex-1 flex flex-col p-4 overflow-hidden bg-apple-surface-tile-1">
            <div class="flex items-center justify-between mb-3 shrink-0">
              <div>
                <h3 class="apple-body-strong text-apple-body-on-dark">Workspace Scope Variables</h3>
                <p class="apple-fine-print text-apple-body-muted">Active across all collections and environment profiles in this workspace.</p>
              </div>

              <button
                type="button"
                onclick={addGlobalVariable}
                class="apple-button-primary text-xs px-3.5 py-1"
              >
                + Add Global Variable
              </button>
            </div>

            <!-- Global Variables Table -->
            <div class="flex-1 overflow-auto border border-apple-hairline-dark rounded-apple-sm bg-apple-surface-tile-1">
              <table class="w-full border-collapse text-left font-mono text-xs">
                <thead class="sticky top-0 bg-apple-surface-tile-2 border-b border-apple-hairline-dark z-10">
                  <tr class="text-apple-body-muted apple-micro-legal">
                    <th class="w-8 p-2 text-center"></th>
                    <th class="w-1/3 p-2 border-r border-apple-hairline-dark">Variable Key</th>
                    <th class="w-1/2 p-2 border-r border-apple-hairline-dark">Variable Value</th>
                    <th class="w-32 p-2 border-r border-apple-hairline-dark text-center font-sans">Storage</th>
                    <th class="w-20 p-2 border-r border-apple-hairline-dark text-center font-sans">Secret</th>
                    <th class="w-8 p-2 text-center"></th>
                  </tr>
                </thead>
                <tbody>
                  {#each globalVars as v, idx (v.id)}
                    {@const isRevealed = Boolean(showSecrets[v.id])}
                    <tr class="border-b border-apple-hairline-dark/60 hover:bg-apple-surface-tile-2/40">
                      <td class="p-2 text-center">
                        <input
                          type="checkbox"
                          bind:checked={v.enabled}
                          onchange={() => workspaceStore.updateGlobalVariables(globalVars)}
                          class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark text-apple-primary focus:ring-0"
                        />
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark">
                        <input
                          type="text"
                          bind:value={v.key}
                          placeholder="e.g. global_api_token"
                          oninput={() => workspaceStore.updateGlobalVariables(globalVars)}
                          class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none"
                        />
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark">
                        <div class="flex items-center gap-1">
                          <input
                            type={v.isSecret && !isRevealed ? 'password' : 'text'}
                            bind:value={v.value}
                            placeholder="value"
                            oninput={() => workspaceStore.updateGlobalVariables(globalVars)}
                            class="w-full bg-transparent px-2 py-1 font-mono {v.isSecret ? 'text-amber-400' : 'text-apple-primary-on-dark'} focus:outline-none"
                          />
                          {#if v.isSecret}
                            <button
                              type="button"
                              title={isRevealed ? 'Mask secret' : 'Reveal secret'}
                              onclick={() => toggleSecretVisibility(v.id)}
                              class="text-xs px-1 text-apple-body-muted hover:text-white"
                            >
                              {isRevealed ? '🙈' : '👁️'}
                            </button>
                          {/if}
                        </div>
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark text-center font-sans">
                        <select
                          bind:value={v.storageTarget}
                          onchange={() => workspaceStore.updateGlobalVariables(globalVars)}
                          class="bg-apple-surface-tile-3 border border-apple-hairline-dark rounded px-1.5 py-0.5 text-[11px] text-apple-body-on-dark focus:outline-none cursor-pointer"
                        >
                          <option value="file">📁 File (.env)</option>
                          <option value="keyring">🔒 OS Keyring</option>
                          <option value="local">🚫 Local (.local)</option>
                        </select>
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark text-center">
                        <label class="cursor-pointer inline-flex items-center gap-1 apple-micro-legal text-apple-body-muted">
                          <input
                            type="checkbox"
                            bind:checked={v.isSecret}
                            onchange={() => workspaceStore.updateGlobalVariables(globalVars)}
                            class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark text-amber-500 focus:ring-0"
                          />
                          <span>{v.isSecret ? '🔒' : '🔓'}</span>
                        </label>
                      </td>
                      <td class="p-2 text-center">
                        <button
                          type="button"
                          onclick={() => removeGlobalVariable(idx)}
                          class="text-apple-ink-muted-48 hover:text-rose-400"
                        >
                          ✕
                        </button>
                      </td>
                    </tr>
                  {/each}
                  {#if globalVars.length === 0}
                    <tr>
                      <td colspan="6" class="p-8 text-center text-apple-body-muted font-sans apple-caption">
                        No workspace-scope variables defined. Click "+ Add Global Variable" above.
                      </td>
                    </tr>
                  {/if}
                </tbody>
              </table>
            </div>
          </div>

        {:else if activeTab === 'security'}
          <!-- Security & Scrubbing Configuration Pane -->
          <div class="flex-1 flex flex-col p-6 overflow-auto bg-apple-surface-tile-1 space-y-6">
            <div class="border border-apple-hairline-dark rounded-apple-lg p-5 bg-apple-surface-tile-2/60 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="apple-body-strong text-apple-body-on-dark flex items-center gap-2">
                    <span>🛡️ Scrub Tainted Secrets</span>
                    <span class="apple-micro-legal px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark font-mono">
                      security.scrub_tainted_secrets
                    </span>
                  </h4>
                  <p class="apple-fine-print text-apple-body-muted mt-1">
                    Automatically redact secrets from all execution logs, test outputs, runner results, and exported reports.
                  </p>
                </div>

                <label class="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={scrubTaintedSecrets}
                    onchange={toggleScrubSecrets}
                    class="sr-only peer"
                  />
                  <div class="w-11 h-6 bg-apple-surface-tile-3 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-apple-primary"></div>
                </label>
              </div>

              <div class="border-t border-apple-hairline-dark/60 pt-4 grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-sans">
                <div class="p-3 rounded-apple-md bg-apple-surface-tile-3 border border-apple-hairline-dark space-y-1.5">
                  <div class="font-semibold text-apple-body-on-dark flex items-center gap-1.5">
                    <span class="text-emerald-400">✓</span>
                    <span>Secrets with Length &gt;= 6</span>
                  </div>
                  <p class="text-apple-body-muted apple-micro-legal leading-relaxed">
                    Undergo automated global substring scrubbing. Anywhere this secret token appears in headers, URLs, response bodies, or CLI logs, it is replaced with <code class="text-apple-primary-on-dark font-mono">[REDACTED]</code>.
                  </p>
                </div>

                <div class="p-3 rounded-apple-md bg-apple-surface-tile-3 border border-apple-hairline-dark space-y-1.5">
                  <div class="font-semibold text-apple-body-on-dark flex items-center gap-1.5">
                    <span class="text-amber-400">⚡</span>
                    <span>Secrets with Length &lt; 6</span>
                  </div>
                  <p class="text-apple-body-muted apple-micro-legal leading-relaxed">
                    Require exact token word boundaries or header-specific matching to avoid false-positive redaction of common short substrings.
                  </p>
                </div>
              </div>
            </div>

            <!-- Storage Targets Breakdown -->
            <div class="border border-apple-hairline-dark rounded-apple-lg p-5 bg-apple-surface-tile-2/60 space-y-3">
              <h4 class="apple-body-strong text-apple-body-on-dark">Storage Target Security Architecture</h4>
              <div class="space-y-2 text-xs text-apple-body-muted">
                <div class="flex items-start gap-2.5 p-2 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark">
                  <span class="text-sm">📁</span>
                  <div>
                    <span class="font-semibold text-apple-body-on-dark">File (.env.yarac)</span>
                    <p class="apple-micro-legal text-apple-ink-muted-48">Non-secret values and checked-in configuration. Stored as plain text in the workspace directory.</p>
                  </div>
                </div>

                <div class="flex items-start gap-2.5 p-2 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark">
                  <span class="text-sm">🔒</span>
                  <div>
                    <span class="font-semibold text-apple-body-on-dark">OS Keyring</span>
                    <p class="apple-micro-legal text-apple-ink-muted-48">Backed by Windows Credential Manager, macOS Keychain, or Linux Secret Service via Rust domain crates. Never written to disk.</p>
                  </div>
                </div>

                <div class="flex items-start gap-2.5 p-2 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark">
                  <span class="text-sm">🚫</span>
                  <div>
                    <span class="font-semibold text-apple-body-on-dark">Local (.yarac.local)</span>
                    <p class="apple-micro-legal text-apple-ink-muted-48">Developer-specific machine overrides automatically gitignored. Never committed to version control.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="h-12 border-t border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <span class="apple-fine-print text-apple-ink-muted-48">
          Variables are referenced in URL, headers, body & scripts using <code class="text-apple-primary-on-dark">{"{{key}}"}</code> syntax.
        </span>
        <button
          type="button"
          onclick={onClose}
          class="apple-button-dark-utility text-xs px-4 py-1.5 font-semibold"
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
