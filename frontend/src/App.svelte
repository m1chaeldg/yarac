<script lang="ts">
  import Editor from './components/Editor.svelte';
  import ConflictModal from './components/ConflictModal.svelte';
  import PayloadViewer from './components/PayloadViewer.svelte';

  // Svelte 5 runes
  let activeMethod = $state('POST');
  let activeUrl = $state('{{base_url}}/api/v1/users');
  let activeEnv = $state('Production');
  let isSending = $state(false);
  let responseStatus = $state<number | null>(null);
  let responseDuration = $state<number | null>(null);
  let responseBody = $state<string>('{\n  "id": "usr_99184820",\n  "status": "created",\n  "created_at": "2026-10-10T10:00:00Z"\n}');
  let activeTab = $state<'editor' | 'response' | 'scripts'>('editor');

  // Conflict modal state
  let isConflictOpen = $state(false);
  let conflictFilePath = $state('collections/users/create_user.yarac');
  let baseContent = $state('version = "1.0"\nname = "Create User"\nmethod = "POST"\nurl = "{{base_url}}/api/v1/users"\n');
  let oursContent = $state('version = "1.0"\nname = "Create User Account [Modified]"\nmethod = "POST"\nurl = "{{base_url}}/api/v1/users"\n');
  let theirsContent = $state('version = "1.0"\nname = "Create User (Branch v2)"\nmethod = "POST"\nurl = "{{base_url}}/api/v2/users"\n');

  // Current request DSL
  let requestDsl = $state(`version = "1.0"
name = "Create User Account"
method = "POST"
url = "{{base_url}}/api/v1/users"

[settings]
inherit_auth = true
verify_ssl = true
follow_redirects = true
timeout_ms = 15000

[headers]
Content-Type = "application/json"
Accept = "application/json"
X-Request-Id = "{{$uuid}}"

[auth]
type = "bearer"
token = "{{api_bearer_token}}"

[body.json]
\`\`\`json
{
  "email": "developer_{{$randomInt}}@yarac.dev",
  "role": "admin"
}
\`\`\`

[scripts.post-response]
\`\`\`javascript
yarac.test("User created successfully", () => {
  expect(yarac.response.status).to.equal(201);
  const data = yarac.response.json();
  expect(data.id).to.be.a("string");
});
\`\`\`
`);

  function triggerSend() {
    isSending = true;
    setTimeout(() => {
      isSending = false;
      responseStatus = 201;
      responseDuration = 48;
      activeTab = 'response';
    }, 300);
  }

  function simulateExternalConflict() {
    isConflictOpen = true;
  }
</script>

<div class="flex h-screen w-screen overflow-hidden bg-slate-950 font-sans text-slate-100">
  <!-- Left Sidebar (Workspace Navigation) -->
  <aside class="w-64 border-r border-slate-800 bg-slate-900/60 flex flex-col justify-between shrink-0">
    <div>
      <div class="h-12 border-b border-slate-800 px-4 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="w-3 h-3 rounded-full bg-sky-500 shadow-sm shadow-sky-500/50"></span>
          <span class="font-bold text-sm tracking-wide text-slate-100">YARAC</span>
        </div>
        <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-slate-800 text-slate-400">v0.1.0</span>
      </div>

      <div class="p-3 text-xs">
        <div class="text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2 px-2">Collections</div>
        <div class="space-y-0.5">
          <div class="px-2 py-1.5 rounded bg-sky-950/60 text-sky-300 font-medium flex items-center gap-2 cursor-pointer border border-sky-800/40">
            <span class="text-[10px] font-bold text-emerald-400">POST</span>
            <span class="truncate">Create User Account</span>
          </div>
          <div class="px-2 py-1.5 rounded hover:bg-slate-800/60 text-slate-400 flex items-center gap-2 cursor-pointer transition-colors">
            <span class="text-[10px] font-bold text-sky-400">GET</span>
            <span class="truncate">Fetch User Profile</span>
          </div>
          <div class="px-2 py-1.5 rounded hover:bg-slate-800/60 text-slate-400 flex items-center gap-2 cursor-pointer transition-colors">
            <span class="text-[10px] font-bold text-amber-400">PUT</span>
            <span class="truncate">Update Credentials</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom Workspace Controls -->
    <div class="p-3 border-t border-slate-800 bg-slate-950/40 text-xs">
      <button
        onclick={simulateExternalConflict}
        class="w-full text-left px-2 py-1.5 rounded text-amber-300 hover:bg-amber-950/40 transition-colors flex items-center justify-between"
      >
        <span>Simulate 3-Way Conflict</span>
        <span class="text-[10px] px-1 rounded bg-amber-950 border border-amber-800">Test</span>
      </button>
    </div>
  </aside>

  <!-- Main View Area -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <!-- Top Execution Address Bar -->
    <header class="h-14 border-b border-slate-800 bg-slate-900/40 px-4 flex items-center justify-between gap-3 shrink-0">
      <div class="flex-1 flex items-center gap-2">
        <select
          bind:value={activeMethod}
          class="bg-slate-800 border border-slate-700 text-xs font-bold text-emerald-400 rounded-md px-2.5 py-1.5 focus:outline-none focus:border-sky-500"
        >
          <option>GET</option>
          <option>POST</option>
          <option>PUT</option>
          <option>DELETE</option>
          <option>PATCH</option>
        </select>

        <div class="flex-1 relative">
          <input
            type="text"
            bind:value={activeUrl}
            class="w-full bg-slate-900 border border-slate-700 rounded-md px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
          />
        </div>

        <button
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
      </div>

      <!-- Environment Selector Badge -->
      <div class="flex items-center gap-2 border-l border-slate-800 pl-3">
        <span class="text-[11px] text-slate-400">Env:</span>
        <select
          bind:value={activeEnv}
          class="bg-slate-800/80 border border-slate-700 text-xs text-slate-300 rounded px-2 py-1 focus:outline-none"
        >
          <option>Production</option>
          <option>Staging</option>
          <option>Development</option>
        </select>
      </div>
    </header>

    <!-- Sub-Header Tabs -->
    <div class="h-10 border-b border-slate-800 bg-slate-950 flex items-center justify-between px-4 shrink-0">
      <div class="flex gap-2">
        <button
          onclick={() => activeTab = 'editor'}
          class="px-3 py-1 text-xs font-medium rounded transition-colors {activeTab === 'editor' ? 'bg-slate-800 text-sky-400' : 'text-slate-400 hover:text-slate-200'}"
        >
          .yarac Editor
        </button>
        <button
          onclick={() => activeTab = 'response'}
          class="px-3 py-1 text-xs font-medium rounded transition-colors flex items-center gap-1.5 {activeTab === 'response' ? 'bg-slate-800 text-emerald-400' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Response</span>
          {#if responseStatus}
            <span class="px-1 py-0.2 rounded text-[10px] bg-emerald-950 text-emerald-400 font-bold border border-emerald-800">
              {responseStatus}
            </span>
          {/if}
        </button>
      </div>

      {#if responseDuration}
        <div class="text-[11px] text-slate-400 font-mono">
          Time: <span class="text-emerald-400 font-semibold">{responseDuration}ms</span>
        </div>
      {/if}
    </div>

    <!-- Active Viewport Content -->
    <div class="flex-1 overflow-hidden p-3 bg-slate-950">
      {#if activeTab === 'editor'}
        <div class="h-full border border-slate-800 rounded-lg overflow-hidden">
          <Editor content={requestDsl} onchange={(c) => requestDsl = c} />
        </div>
      {:else}
        <PayloadViewer payload={responseBody} />
      {/if}
    </div>
  </main>

  <!-- 3-Way Conflict Modal -->
  <ConflictModal
    isOpen={isConflictOpen}
    filePath={conflictFilePath}
    baseContent={baseContent}
    oursContent={oursContent}
    theirsContent={theirsContent}
    onresolve={(res) => {
      requestDsl = res;
      isConflictOpen = false;
    }}
    oncancel={() => isConflictOpen = false}
  />
</div>
