<script lang="ts">
  import { workspaceStore, type YaracRequest } from './stores/workspace.svelte';
  import { backendApi } from './services/backendApi';
  import WorkspaceNav from './components/WorkspaceNav.svelte';
  import TabBar from './components/TabBar.svelte';
  import VisualEditor from './components/VisualEditor.svelte';
  import Editor from './components/Editor.svelte';
  import ResponseViewer from './components/ResponseViewer.svelte';
  import EnvironmentModal from './components/EnvironmentModal.svelte';
  import ConflictModal from './components/ConflictModal.svelte';
  import InteropModal from './components/InteropModal.svelte';
  import RunnerView from './components/RunnerView.svelte';
  import Resizer from './components/Resizer.svelte';

  // Svelte 5 runes
  let activeView = $state<'studio' | 'runner'>('studio');
  let editorMode = $state<'visual' | 'dsl'>('visual');
  let isSending = $state(false);

  // Resizable pane dimensions
  let sidebarWidth = $state(260); // px
  let requestPanePercent = $state(52); // percentage height of request pane vs response pane

  // Modals state
  let isEnvModalOpen = $state(false);
  let isInteropOpen = $state(false);
  let isConflictOpen = $state(false);

  // Conflict modal sample data
  let conflictFilePath = $state('collections/users/create_user.yarac');
  let baseContent = $state('version = "1.0"\nname = "Create User"\nmethod = "POST"\nurl = "{{base_url}}/api/v1/users"\n');
  let oursContent = $state('version = "1.0"\nname = "Create User Account [Modified]"\nmethod = "POST"\nurl = "{{base_url}}/api/v1/users"\n');
  let theirsContent = $state('version = "1.0"\nname = "Create User (Branch v2)"\nmethod = "POST"\nurl = "{{base_url}}/api/v2/users"\n');

  let activeReq = $derived(workspaceStore.activeRequest);

  function handleSidebarResize(delta: number) {
    sidebarWidth = Math.max(180, Math.min(500, sidebarWidth + delta));
  }

  function handleVerticalSplitResize(delta: number) {
    const mainContainer = document.getElementById('studio-split-container');
    if (!mainContainer) return;
    const containerHeight = mainContainer.clientHeight;
    if (containerHeight <= 0) return;
    const deltaPercent = (delta / containerHeight) * 100;
    requestPanePercent = Math.max(20, Math.min(80, requestPanePercent + deltaPercent));
  }

  function handleCreateNewRequest() {
    const colId = workspaceStore.currentWorkspace.collections[0]?.id || workspaceStore.addCollection('General');
    workspaceStore.createRequest(colId, 'New Request', 'GET');
    activeView = 'studio';
  }

  async function triggerSend() {
    if (!activeReq || isSending) return;
    isSending = true;

    try {
      const activeEnv = workspaceStore.currentWorkspace.environments.find(
        (e) => e.id === workspaceStore.currentWorkspace.activeEnvironmentId
      );
      const envVars: Record<string, string> = {};
      if (activeEnv) {
        for (const v of activeEnv.variables) {
          if (v.enabled && v.key.trim()) {
            envVars[v.key.trim()] = v.value;
          }
        }
      }

      const report = await backendApi.executeRequest(activeReq, envVars);
      if (activeReq) {
        const bodyStr = report.response_body ?? '';
        activeReq.response = {
          status: report.status_code || (report.error ? 0 : 200),
          statusText: report.error
            ? report.error
            : report.status_code
              ? `Status ${report.status_code}`
              : 'OK',
          durationMs: report.duration_ms,
          sizeBytes: bodyStr.length,
          headers: report.response_headers || {},
          body: bodyStr,
          timestamp: new Date().toLocaleTimeString(),
        };
        activeReq.isDirty = false;
      }
    } catch (err: any) {
      if (activeReq) {
        activeReq.response = {
          status: 0,
          statusText: err?.message || 'Execution Error',
          durationMs: 0,
          sizeBytes: 0,
          headers: {},
          body: JSON.stringify({ error: err?.message || String(err) }, null, 2),
          timestamp: new Date().toLocaleTimeString(),
        };
      }
    } finally {
      isSending = false;
    }
  }
</script>

<div class="flex h-screen w-screen overflow-hidden bg-slate-950 font-sans text-slate-100 select-none">
  <!-- Left Sidebar (Workspace Navigation & Collections) with Resizer -->
  <aside
    style="width: {sidebarWidth}px;"
    class="border-r border-slate-800 bg-slate-900/60 flex flex-col shrink-0 h-full overflow-hidden"
  >
    <WorkspaceNav
      {activeView}
      onSelectView={(v) => activeView = v}
      onOpenEnvManager={() => isEnvModalOpen = true}
      onOpenInterop={() => isInteropOpen = true}
      onOpenConflict={() => isConflictOpen = true}
    />
  </aside>

  <!-- Horizontal Resizer handle between sidebar and main workspace -->
  <Resizer
    direction="horizontal"
    onResize={handleSidebarResize}
    onReset={() => sidebarWidth = 260}
  />

  <!-- Main View Area -->
  <main class="flex-1 flex flex-col overflow-hidden h-full">
    {#if activeView === 'runner'}
      <RunnerView />
    {:else}
      <!-- Request Studio View -->
      <div class="flex-1 flex flex-col overflow-hidden h-full">
        <!-- Tabs Header Bar -->
        <TabBar onNewRequest={handleCreateNewRequest} />

        {#if activeReq}
          <!-- Execution Address Bar -->
          <header class="h-14 border-b border-slate-800 bg-slate-900/40 px-4 flex items-center justify-between gap-3 shrink-0">
            <div class="flex-1 flex items-center gap-2">
              <!-- HTTP Method -->
              <select
                bind:value={activeReq.method}
                onchange={() => workspaceStore.updateActiveRequest({ method: activeReq?.method })}
                class="bg-slate-800 border border-slate-700 text-xs font-bold rounded-md px-2.5 py-1.5 focus:outline-none focus:border-sky-500 font-mono
                  {activeReq.method === 'GET' ? 'text-sky-400' : activeReq.method === 'POST' ? 'text-emerald-400' : activeReq.method === 'PUT' ? 'text-amber-400' : activeReq.method === 'DELETE' ? 'text-rose-400' : 'text-purple-400'}"
              >
                <option>GET</option>
                <option>POST</option>
                <option>PUT</option>
                <option>DELETE</option>
                <option>PATCH</option>
                <option>HEAD</option>
                <option>OPTIONS</option>
              </select>

              <!-- URL Input -->
              <div class="flex-1 relative">
                <input
                  type="text"
                  bind:value={activeReq.url}
                  placeholder={'Enter request URL or {{base_url}}/path...'}
                  oninput={() => workspaceStore.updateActiveRequest({ url: activeReq?.url })}
                  class="w-full bg-slate-900 border border-slate-700 rounded-md px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                />
              </div>

              <!-- Send Button -->
              <button
                type="button"
                onclick={triggerSend}
                disabled={isSending}
                class="bg-sky-600 hover:bg-sky-500 text-white font-semibold text-xs px-4 py-1.5 rounded-md shadow-sm transition-all flex items-center gap-1.5 shrink-0"
              >
                {#if isSending}
                  <span class="animate-spin text-xs">⟳</span> Sending...
                {:else}
                  <span>▶</span> Send
                {/if}
              </button>

              <!-- Save Button -->
              <button
                type="button"
                onclick={() => workspaceStore.saveActiveRequest()}
                class="bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold px-3 py-1.5 rounded-md border border-slate-700 transition-colors"
              >
                Save
              </button>
            </div>

            <!-- Environment Selector & Profile Manager Shortcut -->
            <div class="flex items-center gap-2 border-l border-slate-800 pl-3">
              <span class="text-[11px] text-slate-400">Env:</span>
              <select
                value={workspaceStore.currentWorkspace.activeEnvironmentId}
                onchange={(e) => workspaceStore.setActiveEnvironment((e.target as HTMLSelectElement).value)}
                class="bg-slate-800/80 border border-slate-700 text-xs text-slate-300 rounded px-2 py-1 focus:outline-none font-medium"
              >
                {#each workspaceStore.currentWorkspace.environments as env (env.id)}
                  <option value={env.id}>{env.name}</option>
                {/each}
              </select>

              <button
                type="button"
                title="Manage Environment Profiles (Bruno Style)"
                onclick={() => isEnvModalOpen = true}
                class="p-1 rounded bg-slate-800 hover:bg-slate-700 text-emerald-400 hover:text-emerald-300 text-xs font-bold"
              >
                ⚙
              </button>
            </div>
          </header>

          <!-- Dual Mode Switcher Bar (Visual UI vs Code .yarac) -->
          <div class="h-9 border-b border-slate-800 bg-slate-950 flex items-center justify-between px-4 shrink-0">
            <div class="flex items-center gap-2">
              <button
                type="button"
                onclick={() => editorMode = 'visual'}
                class="px-3 py-1 text-xs font-semibold rounded transition-colors flex items-center gap-1.5
                  {editorMode === 'visual'
                    ? 'bg-sky-600/20 text-sky-300 border border-sky-500/30'
                    : 'text-slate-400 hover:text-slate-200'}"
              >
                <span>🛠 Visual Editor</span>
              </button>

              <button
                type="button"
                onclick={() => editorMode = 'dsl'}
                class="px-3 py-1 text-xs font-semibold rounded transition-colors flex items-center gap-1.5
                  {editorMode === 'dsl'
                    ? 'bg-sky-600/20 text-sky-300 border border-sky-500/30'
                    : 'text-slate-400 hover:text-slate-200'}"
              >
                <span>📝 Code (.yarac)</span>
              </button>
            </div>

            <span class="text-[11px] text-slate-500 font-mono">
              Lossless Bidirectional Sync Active
            </span>
          </div>

          <!-- Resizable Vertical Split Container: Request Panel (Top) vs Response Panel (Bottom) -->
          <div id="studio-split-container" class="flex-1 flex flex-col overflow-hidden bg-slate-950">
            <!-- Request Pane -->
            <div style="height: {requestPanePercent}%;" class="overflow-hidden flex flex-col p-2">
              {#if editorMode === 'visual'}
                <div class="h-full border border-slate-800 rounded-lg overflow-hidden bg-slate-950">
                  <VisualEditor />
                </div>
              {:else}
                <div class="h-full border border-slate-800 rounded-lg overflow-hidden bg-slate-950">
                  <Editor
                    content={activeReq.dslContent}
                    onchange={(newDsl) => workspaceStore.updateActiveRequestFromDsl(newDsl)}
                  />
                </div>
              {/if}
            </div>

            <!-- Resizer Handle between Request & Response -->
            <Resizer
              direction="vertical"
              onResize={handleVerticalSplitResize}
              onReset={() => requestPanePercent = 52}
            />

            <!-- Response Pane -->
            <div style="height: {100 - requestPanePercent}%;" class="overflow-hidden flex flex-col p-2 pt-0">
              <ResponseViewer response={activeReq.response} />
            </div>
          </div>
        {:else}
          <div class="flex-1 flex flex-col items-center justify-center text-slate-500 space-y-3">
            <span class="text-4xl">📁</span>
            <p class="text-sm font-semibold text-slate-300">No Request Open</p>
            <p class="text-xs text-slate-500">Select a request from the sidebar or click below to start.</p>
            <button
              type="button"
              onclick={handleCreateNewRequest}
              class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold rounded-md shadow-sm transition-colors"
            >
              + Create Request
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </main>

  <!-- Environment Profiles Manager Modal (Bruno style) -->
  <EnvironmentModal
    isOpen={isEnvModalOpen}
    onClose={() => isEnvModalOpen = false}
  />

  <!-- 3-Way Conflict Modal -->
  <ConflictModal
    isOpen={isConflictOpen || !!workspaceStore.activeConflict}
    filePath={workspaceStore.activeConflict?.filePath ?? conflictFilePath}
    baseContent={workspaceStore.activeConflict?.baseContent ?? baseContent}
    oursContent={workspaceStore.activeConflict?.oursContent ?? oursContent}
    theirsContent={workspaceStore.activeConflict?.theirsContent ?? theirsContent}
    onresolve={(res) => {
      if (workspaceStore.activeConflict) {
        workspaceStore.resolveConflict(res);
      } else if (activeReq) {
        workspaceStore.updateActiveRequestFromDsl(res);
      }
      isConflictOpen = false;
    }}
    oncancel={() => {
      if (workspaceStore.activeConflict) {
        workspaceStore.resolveConflict(null);
      }
      isConflictOpen = false;
    }}
  />

  <!-- Interoperability Hub Modal (OpenAPI v3.1 / Bruno) -->
  <InteropModal
    isOpen={isInteropOpen}
    onClose={() => isInteropOpen = false}
  />
</div>
