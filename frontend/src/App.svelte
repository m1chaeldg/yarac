<script lang="ts">
  import { workspaceStore, type YaracRequest } from './stores/workspace.svelte';
  import { backendApi } from './services/backendApi';
  import WorkspaceNav from './components/WorkspaceNav.svelte';
  import TabBar from './components/TabBar.svelte';
  import VisualEditor from './components/VisualEditor.svelte';
  import Editor from './components/Editor.svelte';
  import ResponseViewer, { type TimelineEntry } from './components/ResponseViewer.svelte';
  import EnvironmentModal from './components/EnvironmentModal.svelte';
  import ConflictModal from './components/ConflictModal.svelte';
  import InteropModal from './components/InteropModal.svelte';
  import CookieJarModal from './components/CookieJarModal.svelte';
  import GitCommitModal from './components/GitCommitModal.svelte';
  import McpServerModal from './components/McpServerModal.svelte';
  import QuickOpenModal from './components/QuickOpenModal.svelte';
  import RealtimeStreamViewer from './components/RealtimeStreamViewer.svelte';
  import GrpcClientViewer from './components/GrpcClientViewer.svelte';
  import RunnerView from './components/RunnerView.svelte';
  import Resizer from './components/Resizer.svelte';
  import { syncQueryParamsFromUrl, substitutePathVariables } from './utils/urlUtils';

  // Svelte 5 runes
  let activeView = $state<'studio' | 'runner'>('studio');
  let editorMode = $state<'visual' | 'dsl'>('visual');
  let isSending = $state(false);

  // Resizable pane dimensions & layout
  let sidebarWidth = $state(260); // px
  let requestPanePercent = $state(50); // percentage dimension of request pane vs response pane
  let studioLayout = $state<'vertical' | 'horizontal'>('vertical');
  let timeline = $state<TimelineEntry[]>([]);

  // Modals state
  let isEnvModalOpen = $state(false);
  let isInteropOpen = $state(false);
  let isConflictOpen = $state(false);
  let isCookieJarOpen = $state(false);
  let isGitModalOpen = $state(false);
  let isMcpModalOpen = $state(false);
  let isQuickOpen = $state(false);
  let gitBranch = $state('main');
  let gitModifiedCount = $state(0);
  let isQuickSyncing = $state(false);

  async function refreshGitStatus() {
    try {
      const st = await backendApi.getGitStatus();
      gitBranch = st.branch || 'main';
      gitModifiedCount = (st.modified_files?.length || 0) + (st.untracked_files?.length || 0);
    } catch (_) {}
  }

  async function quickSyncGit() {
    if (isQuickSyncing) return;
    isQuickSyncing = true;
    try {
      await backendApi.syncGit();
      await refreshGitStatus();
    } catch (_) {}
    isQuickSyncing = false;
  }

  $effect(() => {
    refreshGitStatus();
  });
  let selectedProtocol = $state<'http' | 'ws' | 'sse' | 'grpc'>('http');

  // Conflict modal sample data
  let conflictFilePath = $state('collections/users/create_user.yarac');
  let baseContent = $state('version = "1.0"\nname = "Create User"\nmethod = "POST"\nurl = "{{base_url}}/api/v1/users"\n');
  let oursContent = $state('version = "1.0"\nname = "Create User Account [Modified]"\nmethod = "POST"\nurl = "{{base_url}}/api/v1/users"\n');
  let theirsContent = $state('version = "1.0"\nname = "Create User (Branch v2)"\nmethod = "POST"\nurl = "{{base_url}}/api/v2/users"\n');

  let activeReq = $derived(workspaceStore.activeRequest);

  function handleSidebarResize(delta: number) {
    sidebarWidth = Math.max(180, Math.min(500, sidebarWidth + delta));
  }

  function handleSplitResize(delta: number) {
    const mainContainer = document.getElementById('studio-split-container');
    if (!mainContainer) return;
    const totalDim = studioLayout === 'vertical' ? mainContainer.clientHeight : mainContainer.clientWidth;
    if (totalDim <= 0) return;
    const deltaPercent = (delta / totalDim) * 100;
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
          if (v.enabled && !v.key.endsWith('~') && v.key.trim()) {
            envVars[v.key.trim()] = v.value;
          }
        }
      }

      const reqToExecute = {
        ...activeReq,
        url: substitutePathVariables(activeReq.url, activeReq.pathVariables)
      };
      const report = await backendApi.executeRequest(reqToExecute, envVars);
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
          assertions: report.assertions || [],
          logs: report.logs || [],
        };
        activeReq.isDirty = false;

        timeline = [
          {
            id: Math.random().toString(36).substring(2, 9),
            timestamp: new Date().toLocaleTimeString(),
            status: activeReq.response.status,
            statusText: activeReq.response.statusText,
            durationMs: activeReq.response.durationMs,
            sizeBytes: activeReq.response.sizeBytes,
            method: activeReq.method,
            url: activeReq.url
          },
          ...timeline
        ];

        workspaceStore.addHistoryItem({
          requestId: activeReq.id,
          requestName: activeReq.name,
          method: activeReq.method,
          url: activeReq.url,
          statusCode: activeReq.response.status,
          durationMs: activeReq.response.durationMs,
          timestamp: new Date().toLocaleTimeString()
        });
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
          assertions: [
            {
              name: 'Request execution',
              passed: false,
              error: err?.message || String(err)
            }
          ],
          logs: []
        };
      }
    } finally {
      isSending = false;
    }
  }
  function handleGlobalKeydown(e: KeyboardEvent) {
    // If an input or textarea is focused and it is not a modifier shortcut, let it pass
    const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const modKey = isMac ? e.metaKey : e.ctrlKey;

    // Ctrl+Enter / Cmd+Enter: Send request
    if (modKey && e.key === 'Enter') {
      e.preventDefault();
      if (activeView === 'studio' && activeReq && !isSending) {
        triggerSend();
      }
      return;
    }

    // Ctrl+S / Cmd+S: Save active request
    if (modKey && (e.key === 's' || e.key === 'S')) {
      e.preventDefault();
      if (activeReq) {
        workspaceStore.saveActiveRequest();
      }
      return;
    }

    // Ctrl+P / Cmd+P or Ctrl+O: Quick Open
    if (modKey && (e.key === 'p' || e.key === 'P' || e.key === 'o' || e.key === 'O')) {
      e.preventDefault();
      isQuickOpen = !isQuickOpen;
      return;
    }

    // Ctrl+N / Cmd+N: New Request
    if (modKey && (e.key === 'n' || e.key === 'N')) {
      e.preventDefault();
      handleCreateNewRequest();
      return;
    }

    // Ctrl+W / Cmd+W: Close active tab
    if (modKey && (e.key === 'w' || e.key === 'W')) {
      e.preventDefault();
      if (activeReq) {
        workspaceStore.closeTab(activeReq.id);
      }
      return;
    }

    // Ctrl+B / Cmd+B: Toggle sidebar
    if (modKey && (e.key === 'b' || e.key === 'B')) {
      e.preventDefault();
      sidebarWidth = sidebarWidth > 0 ? 0 : 260;
      return;
    }

    // Ctrl+Tab / Ctrl+Shift+Tab: Cycle open tabs
    if (modKey && e.key === 'Tab') {
      e.preventDefault();
      const openIds = workspaceStore.currentWorkspace.openTabIds;
      if (openIds.length > 1) {
        const curIdx = openIds.indexOf(workspaceStore.currentWorkspace.activeTabId);
        const nextIdx = e.shiftKey
          ? (curIdx - 1 + openIds.length) % openIds.length
          : (curIdx + 1) % openIds.length;
        workspaceStore.setActiveTab(openIds[nextIdx]);
      }
      return;
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="flex h-screen w-screen overflow-hidden bg-apple-surface-black font-sans text-apple-body-on-dark select-none">
  <!-- Left Sidebar (Workspace Navigation & Collections) with Resizer -->
  <aside
    style="width: {sidebarWidth}px;"
    class="border-r border-apple-hairline-dark/50 bg-apple-surface-tile-1 flex flex-col shrink-0 h-full overflow-hidden"
  >
    <WorkspaceNav
      {activeView}
      onSelectView={(v) => activeView = v}
      onOpenEnvManager={() => isEnvModalOpen = true}
      onOpenInterop={() => isInteropOpen = true}
      onOpenConflict={() => isConflictOpen = true}
      onOpenQuickOpen={() => isQuickOpen = true}
    />
  </aside>

  <!-- Horizontal Resizer handle between sidebar and main workspace -->
  <Resizer
    direction="horizontal"
    onResize={handleSidebarResize}
    onReset={() => sidebarWidth = 260}
  />

  <!-- Main View Area -->
  <main class="flex-1 flex flex-col overflow-hidden h-full bg-apple-surface-black">
    {#if activeView === 'runner'}
      <RunnerView />
    {:else}
      <!-- Request Studio View -->
      <div class="flex-1 flex flex-col overflow-hidden h-full">
        <!-- Tabs Header Bar -->
        <TabBar onNewRequest={handleCreateNewRequest} onOpenQuickOpen={() => isQuickOpen = true} />

        {#if activeReq}
          <!-- Execution Address Bar -->
          <header class="h-14 border-b border-apple-hairline-dark/60 bg-apple-surface-tile-2 px-4 flex items-center justify-between gap-3 shrink-0">
            <div class="flex-1 flex items-center gap-2">
              <!-- Protocol Selector Toggle (HTTP / WS / SSE) -->
              <div class="flex items-center rounded-apple-sm bg-apple-surface-tile-3 border border-apple-hairline-dark/80 p-0.5 shrink-0">
                <button
                  type="button"
                  onclick={() => selectedProtocol = 'http'}
                  class="px-2 py-0.5 text-[10px] font-bold rounded-apple-xs transition-colors {selectedProtocol === 'http' ? 'bg-apple-primary text-white' : 'text-apple-body-muted hover:text-white'}"
                >
                  HTTP
                </button>
                <button
                  type="button"
                  onclick={() => selectedProtocol = 'ws'}
                  class="px-2 py-0.5 text-[10px] font-bold rounded-apple-xs transition-colors {selectedProtocol === 'ws' ? 'bg-apple-primary text-white' : 'text-apple-body-muted hover:text-white'}"
                >
                  WS
                </button>
                <button
                  type="button"
                  onclick={() => selectedProtocol = 'sse'}
                  class="px-2 py-0.5 text-[10px] font-bold rounded-apple-xs transition-colors {selectedProtocol === 'sse' ? 'bg-apple-primary text-white' : 'text-apple-body-muted hover:text-white'}"
                >
                  SSE
                </button>
              </div>

              {#if selectedProtocol === 'http'}
                <!-- HTTP Method -->
                <select
                  bind:value={activeReq.method}
                onchange={() => workspaceStore.updateActiveRequest({ method: activeReq?.method })}
                class="bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-xs font-bold rounded-apple-sm px-2.5 py-1.5 focus:outline-none focus:border-apple-primary font-mono
                  {activeReq.method === 'GET' ? 'text-apple-primary-on-dark' : activeReq.method === 'POST' ? 'text-emerald-400' : activeReq.method === 'PUT' ? 'text-amber-400' : activeReq.method === 'DELETE' ? 'text-rose-400' : 'text-purple-400'}"
              >
                <option>GET</option>
                <option>POST</option>
                <option>PUT</option>
                <option>DELETE</option>
                <option>PATCH</option>
                <option>HEAD</option>
                <option>OPTIONS</option>
              </select>
              {/if}

              <!-- URL Input -->
              <div class="flex-1 relative">
                <input
                  type="text"
                  bind:value={activeReq.url}
                  placeholder={selectedProtocol === 'ws' ? 'Enter WebSocket URL (ws:// or wss://)...' : selectedProtocol === 'sse' ? 'Enter SSE Stream URL (http:// or https://)...' : 'Enter request URL or {{base_url}}/path...'}
                  oninput={() => {
                    if (activeReq) {
                      const syncedParams = syncQueryParamsFromUrl(activeReq.url, activeReq.params);
                      activeReq.params = syncedParams;
                      workspaceStore.updateActiveRequest({ url: activeReq.url, params: syncedParams });
                    }
                  }}
                  class="w-full bg-apple-surface-tile-1 border border-apple-hairline-dark/80 rounded-apple-pill px-4 py-1.5 text-xs font-mono text-apple-body-on-dark placeholder:text-apple-ink-muted-48 focus:outline-none focus:border-apple-primary"
                />
              </div>

              {#if selectedProtocol === 'http'}
                <!-- Send Button (Apple Pill Button) -->
                <button
                  type="button"
                  onclick={triggerSend}
                  disabled={isSending}
                  class="apple-button-primary font-semibold text-xs px-5 py-1.5 rounded-apple-pill transition-all flex items-center gap-1.5 shrink-0"
                  title="Send Request (Ctrl+Enter / ⌘Enter)"
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
                  class="apple-button-dark-utility bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark text-xs font-medium px-3.5 py-1.5 rounded-apple-sm border border-apple-hairline-dark/80 transition-colors"
                  title="Save to disk (Ctrl+S / ⌘S)"
                >
                  Save
                </button>
              {/if}
            </div>

            <!-- Environment Selector & Profile Manager Shortcut -->
            <div class="flex items-center gap-2 border-l border-apple-hairline-dark/60 pl-3">
              <span class="text-[11px] text-apple-body-muted">Env:</span>
              <select
                value={workspaceStore.currentWorkspace.activeEnvironmentId}
                onchange={(e) => workspaceStore.setActiveEnvironment((e.target as HTMLSelectElement).value)}
                class="bg-apple-surface-tile-3 border border-apple-hairline-dark/80 text-xs text-apple-body-on-dark rounded-apple-sm px-2.5 py-1 focus:outline-none font-medium"
              >
                {#each workspaceStore.currentWorkspace.environments as env (env.id)}
                  <option value={env.id}>{env.name}</option>
                {/each}
              </select>

              <button
                type="button"
                title="Manage Cookie Jar"
                onclick={() => isCookieJarOpen = true}
                class="p-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-amber-400 text-xs font-bold border border-apple-hairline-dark/80"
              >
                🍪
              </button>

              <button
                type="button"
                title="Manage Environment Profiles"
                onclick={() => isEnvModalOpen = true}
                class="p-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-primary-on-dark text-xs font-bold border border-apple-hairline-dark/80"
              >
                ⚙
              </button>
            </div>
          </header>

          {#if selectedProtocol === 'ws' || selectedProtocol === 'sse'}
            <!-- Realtime Streaming Studio View -->
            <div class="flex-1 overflow-hidden p-2">
              <div class="h-full border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-tile-1">
                <RealtimeStreamViewer req={activeReq} protocol={selectedProtocol} />
              </div>
            </div>
          {:else}
            <!-- Dual Mode Switcher Bar (Visual UI vs Code .yarac) -->
          <div class="h-9 border-b border-apple-hairline-dark/60 bg-apple-surface-tile-3 flex items-center justify-between px-4 shrink-0">
            <div class="flex items-center gap-2">
              <button
                type="button"
                onclick={() => editorMode = 'visual'}
                class="px-3 py-1 text-xs font-semibold rounded-apple-pill transition-colors flex items-center gap-1.5
                  {editorMode === 'visual'
                    ? 'bg-apple-primary text-white shadow-none'
                    : 'text-apple-body-muted hover:text-white'}"
              >
                <span>🛠 Visual Editor</span>
              </button>

              <button
                type="button"
                onclick={() => editorMode = 'dsl'}
                class="px-3 py-1 text-xs font-semibold rounded-apple-pill transition-colors flex items-center gap-1.5
                  {editorMode === 'dsl'
                    ? 'bg-apple-primary text-white shadow-none'
                    : 'text-apple-body-muted hover:text-white'}"
              >
                <span>📝 Code (.yarac)</span>
              </button>
            </div>

            <span class="text-[11px] text-apple-ink-muted-48 font-mono">
              Lossless Bidirectional Sync Active
            </span>
          </div>

          <!-- Resizable Split Container (Vertical Stacked vs Horizontal Side-by-Side) -->
          <div id="studio-split-container" class="flex-1 {studioLayout === 'vertical' ? 'flex flex-col' : 'flex flex-row'} overflow-hidden bg-apple-surface-black">
            <!-- Request Pane -->
            <div
              style="{studioLayout === 'vertical' ? `height: ${requestPanePercent}%;` : `width: ${requestPanePercent}%;`}"
              class="overflow-hidden flex flex-col p-2 {studioLayout === 'horizontal' ? 'pr-0' : ''}"
            >
              {#if editorMode === 'visual'}
                <div class="h-full border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-tile-1">
                  <VisualEditor />
                </div>
              {:else}
                <div class="h-full border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-tile-1">
                  <Editor
                    content={activeReq.dslContent}
                    onchange={(newDsl) => workspaceStore.updateActiveRequestFromDsl(newDsl)}
                  />
                </div>
              {/if}
            </div>

            <!-- Resizer Handle between Request & Response -->
            <Resizer
              direction={studioLayout === 'vertical' ? 'vertical' : 'horizontal'}
              onResize={handleSplitResize}
              onReset={() => requestPanePercent = 50}
            />

            <!-- Response Pane -->
            <div
              style="{studioLayout === 'vertical' ? `height: ${100 - requestPanePercent}%;` : `width: ${100 - requestPanePercent}%;`}"
              class="overflow-hidden flex flex-col p-2 {studioLayout === 'vertical' ? 'pt-0' : 'pl-0'}"
            >
              <ResponseViewer
                response={activeReq.response}
                {timeline}
                onClearResponse={() => {
                  if (activeReq) activeReq.response = undefined;
                }}
                onToggleLayout={() => {
                  studioLayout = studioLayout === 'vertical' ? 'horizontal' : 'vertical';
                }}
                layoutOrientation={studioLayout}
              />
            </div>
          </div>
          {/if}
        {:else}
          <div class="flex-1 flex flex-col items-center justify-center text-apple-ink-muted-48 space-y-3 bg-apple-surface-black">
            <span class="text-4xl">📁</span>
            <p class="text-sm font-semibold text-white tracking-tight">No Request Open</p>
            <p class="text-xs text-apple-body-muted">Select a request from the sidebar or click below to start.</p>
            <button
              type="button"
              onclick={handleCreateNewRequest}
              class="apple-button-primary px-5 py-2 text-xs font-semibold rounded-apple-pill transition-colors"
            >
              + Create Request
            </button>
          </div>
        {/if}
      </div>
    {/if}
  
    <!-- Global Bottom Status Bar -->
    <footer class="h-6 border-t border-apple-hairline-dark/60 bg-apple-surface-tile-2 px-3 flex items-center justify-between text-[11px] font-mono text-apple-body-muted shrink-0 select-none">
      <div class="flex items-center gap-3">
        <!-- Git Branch & Changes Pill -->
        <button
          type="button"
          onclick={() => isGitModalOpen = true}
          class="flex items-center gap-1.5 hover:text-white transition-colors cursor-pointer"
          title="Open Git Commit & Sync Modal"
        >
          <span class="text-xs">🌿</span>
          <span class="font-bold text-apple-body-on-dark">{gitBranch}</span>
          {#if gitModifiedCount > 0}
            <span class="px-1.5 py-0.2 rounded-apple-pill bg-amber-500/20 text-amber-400 font-semibold border border-amber-500/40 text-[10px]">
              ±{gitModifiedCount}
            </span>
          {:else}
            <span class="text-emerald-400 text-[10px]">✓</span>
          {/if}
        </button>

        <span class="text-apple-hairline-dark">|</span>

        <!-- Quick Sync Action -->
        <button
          type="button"
          onclick={quickSyncGit}
          class="hover:text-apple-primary-on-dark transition-colors flex items-center gap-1 cursor-pointer"
          title="Pull and Push to remote repository"
        >
          <span>{isQuickSyncing ? '⏳' : '⇅'}</span>
          <span>{isQuickSyncing ? 'Syncing...' : 'Sync'}</span>
        </button>
      </div>

      <div class="flex items-center gap-3">
        <!-- Offline Invariant Pill -->
        <span class="flex items-center gap-1 text-[10px] text-emerald-400 font-sans">
          <span class="w-1.5 h-1.5 rounded-apple-full bg-emerald-400"></span>
          <span>100% Offline</span>
        </span>

        <span class="text-apple-hairline-dark">|</span>

        <!-- MCP Server Status Pill -->
        <button
          type="button"
          onclick={() => isMcpModalOpen = true}
          class="flex items-center gap-1.5 hover:text-white transition-colors text-purple-400 cursor-pointer"
          title="Open MCP Server Modal"
        >
          <span>🤖</span>
          <span>MCP (Active)</span>
        </button>

        <span class="text-apple-hairline-dark">|</span>

        <!-- Target Memory -->
        <span class="text-[10px] text-apple-ink-muted-48">RAM &lt; 100MB</span>
      </div>
    </footer>
  </main>

  <!-- Cookie Jar Manager Modal -->
  <GitCommitModal
    isOpen={isGitModalOpen}
    onClose={() => isGitModalOpen = false}
    onSuccess={() => refreshGitStatus()}
  />

  <!-- Model Context Protocol (MCP) Server Modal -->
  <McpServerModal
    isOpen={isMcpModalOpen}
    onClose={() => isMcpModalOpen = false}
  />

  <QuickOpenModal
    isOpen={isQuickOpen}
    onClose={() => isQuickOpen = false}
    onSelectRequest={(reqId) => {
      workspaceStore.openTab(reqId);
      activeView = 'studio';
    }}
  />

  <CookieJarModal
    isOpen={isCookieJarOpen}
    onClose={() => isCookieJarOpen = false}
  />

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
