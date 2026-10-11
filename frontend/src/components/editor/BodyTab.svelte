<script lang="ts">
  import { workspaceStore, type YaracRequest, type BodyType } from '../../stores/workspace.svelte';
  import { compressXmlString, formatXmlString } from '../../utils/xmlUtils';
  import { setRequestBodyType } from '../../utils/editorUtils';
  import { formatSize } from '../../utils/statusUtils';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  let bodyActionFeedback = $state('');
  let isBinaryDragging = $state(false);
  let binaryFileInput = $state<HTMLInputElement | null>(null);
  let multipartFileInput = $state<HTMLInputElement | null>(null);
  let activeMultipartRowIdx = $state<number | null>(null);
  let binarySelectedFileInfo = $state<{ name: string; size?: number; type?: string } | null>(null);

  function switchBodyType(bType: BodyType) {
    setRequestBodyType(req, bType);
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function addFormData(type: 'text' | 'file' = 'text') {
    if (!req.body.formData) req.body.formData = [];
    req.body.formData.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      description: '',
      type,
      contentType: type === 'file' ? 'application/octet-stream' : undefined
    });
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function removeFormData(idx: number) {
    if (!req.body.formData) return;
    req.body.formData.splice(idx, 1);
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function cleanFormData() {
    if (!req.body.formData) return;
    req.body.formData = req.body.formData.filter(f => f.key.trim() || f.value.trim());
    workspaceStore.updateActiveRequest({ body: req.body });
    feedback('Cleaned empty rows');
  }

  function formatOrCompressBody(action: 'format' | 'compress') {
    if (!req?.body?.rawContent) return;
    try {
      if (req.body.type === 'json') {
        const parsed = JSON.parse(req.body.rawContent);
        req.body.rawContent = action === 'format' ? JSON.stringify(parsed, null, 2) : JSON.stringify(parsed);
      } else if (req.body.type === 'xml') {
        req.body.rawContent = action === 'format' ? formatXmlString(req.body.rawContent) : compressXmlString(req.body.rawContent);
      }
      workspaceStore.updateActiveRequest({ body: req.body });
      feedback(action === 'format' ? 'Prettified' : 'Compressed');
    } catch (e: any) {
      feedback('Syntax error: ' + (e.message || String(e)));
    }
  }

  function feedback(msg: string) {
    bodyActionFeedback = msg;
    setTimeout(() => (bodyActionFeedback = ''), 2500);
  }

  function handleBinaryFileSelect(file: File) {
    req.body.binaryFilePath = `./fixtures/${file.name}`;
    req.body.binaryContentType = file.type || 'application/octet-stream';
    binarySelectedFileInfo = { name: file.name, size: file.size, type: file.type };
    workspaceStore.updateActiveRequest({ body: req.body });
    feedback(`Attached binary file: ${file.name}`);
  }

  function onBinaryFileInputChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file) {
      handleBinaryFileSelect(file);
    }
    target.value = '';
  }

  function handleBinaryDrop(e: DragEvent) {
    e.preventDefault();
    isBinaryDragging = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) {
      handleBinaryFileSelect(file);
    }
  }

  function clearBinaryFile() {
    req.body.binaryFilePath = '';
    binarySelectedFileInfo = null;
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function triggerMultipartFileBrowse(idx: number) {
    activeMultipartRowIdx = idx;
    multipartFileInput?.click();
  }

  function onMultipartFileInputChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file && activeMultipartRowIdx !== null && req?.body?.formData) {
      const item = req.body.formData[activeMultipartRowIdx];
      if (item) {
        item.value = `./fixtures/${file.name}`;
        item.type = 'file';
        if (file.type) item.contentType = file.type;
        workspaceStore.updateActiveRequest({ body: req.body });
        feedback(`Attached file: ${file.name}`);
      }
    }
    activeMultipartRowIdx = null;
    target.value = '';
  }
</script>

<div class="h-full flex flex-col space-y-3 font-sans text-xs">
  <!-- Body Type Selector & Action Toolbar -->
  <div class="flex items-center justify-between shrink-0 flex-wrap gap-2 pb-1 border-b border-slate-800/80">
    <div class="flex items-center gap-2 flex-wrap">
      <span class="text-xs text-slate-400 font-medium">Body Type:</span>
      <div class="flex items-center gap-1.5 flex-wrap">
        {#each ['none', 'json', 'xml', 'text', 'form-urlencoded', 'multipart', 'binary'] as bType}
          <button
            type="button"
            onclick={() => switchBodyType(bType as BodyType)}
            class="px-2.5 py-1 rounded text-xs font-medium transition-all {req.body.type === bType ? 'bg-sky-600/30 text-sky-300 border border-sky-500/50 shadow-sm shadow-sky-950' : 'bg-slate-900 border border-slate-800 text-slate-400 hover:text-slate-200 hover:border-slate-700'}"
          >
            {bType}
          </button>
        {/each}
      </div>
    </div>

    <!-- Toolbar Actions -->
    <div class="flex items-center gap-2">
      {#if bodyActionFeedback}
        <span class="text-[11px] px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-800 text-emerald-300 font-mono">
          {bodyActionFeedback}
        </span>
      {/if}

      {#if req.body.type === 'json' || req.body.type === 'xml' || req.body.type === 'text' || req.body.type === 'raw'}
        <button
          type="button"
          onclick={() => formatOrCompressBody('format')}
          class="px-2 py-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-300 border border-slate-800 hover:border-slate-700 text-xs font-medium flex items-center gap-1 transition-colors"
          title="Format / Prettify payload"
        >
          <span class="text-amber-400">✨</span> Format
        </button>
        <button
          type="button"
          onclick={() => formatOrCompressBody('compress')}
          class="px-2 py-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-300 border border-slate-800 hover:border-slate-700 text-xs font-medium flex items-center gap-1 transition-colors"
          title="Minify / Compress payload"
        >
          <span class="text-sky-400">⚡</span> Compress
        </button>
        <button
          type="button"
          onclick={() => {
            if (req?.body?.rawContent) {
              navigator.clipboard.writeText(req.body.rawContent);
              feedback('Copied to clipboard');
            }
          }}
          class="px-2 py-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-800 text-xs transition-colors"
          title="Copy body to clipboard"
        >
          📋 Copy
        </button>

      {:else if req.body.type === 'form-urlencoded'}
        <button
          type="button"
          onclick={() => addFormData('text')}
          class="px-2 py-1 rounded bg-sky-600/20 text-sky-300 hover:bg-sky-600/30 border border-sky-500/40 text-xs font-medium flex items-center gap-1"
        >
          + Add Field
        </button>
        <button
          type="button"
          onclick={cleanFormData}
          class="px-2 py-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-800 text-xs"
        >
          Clean / Trim
        </button>

      {:else if req.body.type === 'multipart'}
        <button
          type="button"
          onclick={() => addFormData('text')}
          class="px-2 py-1 rounded bg-sky-600/20 text-sky-300 hover:bg-sky-600/30 border border-sky-500/40 text-xs font-medium flex items-center gap-1"
        >
          + Text Field
        </button>
        <button
          type="button"
          onclick={() => addFormData('file')}
          class="px-2 py-1 rounded bg-purple-600/20 text-purple-300 hover:bg-purple-600/30 border border-purple-500/40 text-xs font-medium flex items-center gap-1"
        >
          📁 + File Upload
        </button>
        <button
          type="button"
          onclick={cleanFormData}
          class="px-2 py-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-800 text-xs"
        >
          Clean / Trim
        </button>
      {/if}
    </div>
  </div>

  <!-- JSON / XML / TEXT EDITOR -->
  {#if req.body.type === 'json' || req.body.type === 'xml' || req.body.type === 'text' || req.body.type === 'raw'}
    <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[260px]">
      <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
        <div class="flex items-center gap-2">
          <span class="font-semibold text-slate-300">
            {req.body.type === 'json' ? 'JSON Payload' : req.body.type === 'xml' ? 'XML Document' : 'Plain Text Payload'}
          </span>
          <span class="text-[10px] px-1.5 py-0.2 rounded bg-slate-800/80 text-sky-400 font-mono">
            {req.body.type === 'json' ? 'application/json' : req.body.type === 'xml' ? 'application/xml' : 'text/plain'}
          </span>
        </div>
        <div class="flex items-center gap-3 font-mono text-slate-500 text-[10px]">
          <span>{(req.body.rawContent || '').split('\n').length} lines</span>
          <span>{(req.body.rawContent || '').length} chars</span>
        </div>
      </div>
      <textarea
        bind:value={req.body.rawContent}
        oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
        placeholder={req.body.type === 'json' ? '{\n  "key": "value"\n}' : req.body.type === 'xml' ? '<root>\n  <item>value</item>\n</root>' : 'Raw body content'}
        class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-slate-100 resize-none focus:outline-none leading-relaxed"
        spellcheck="false"
      ></textarea>
    </div>

  <!-- FORM-URLENCODED -->
  {:else if req.body.type === 'form-urlencoded'}
    <div class="space-y-2 flex-1 flex flex-col">
      <div class="border border-slate-800 rounded-lg overflow-hidden bg-slate-950">
        <table class="w-full border-collapse text-left font-mono text-xs">
          <thead>
            <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
              <th class="w-8 p-2 text-center"></th>
              <th class="p-2 border-r border-slate-800">Parameter Key</th>
              <th class="p-2 border-r border-slate-800">Parameter Value</th>
              <th class="w-10 p-2 text-center"></th>
            </tr>
          </thead>
          <tbody>
            {#each (req.body.formData || []) as item, idx (item.id)}
              <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                <td class="p-2 text-center">
                  <input
                    type="checkbox"
                    bind:checked={item.enabled}
                    onchange={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                  />
                </td>
                <td class="p-1 border-r border-slate-800">
                  <input
                    type="text"
                    bind:value={item.key}
                    placeholder="param_name"
                    oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="w-full bg-transparent px-2 py-1 text-sky-300 focus:outline-none"
                  />
                </td>
                <td class="p-1 border-r border-slate-800">
                  <input
                    type="text"
                    bind:value={item.value}
                    placeholder="value"
                    oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                  />
                </td>
                <td class="p-2 text-center">
                  <button
                    type="button"
                    onclick={() => removeFormData(idx)}
                    class="text-slate-500 hover:text-rose-400"
                  >
                    ✕
                  </button>
                </td>
              </tr>
            {/each}
            {#if (!req.body.formData || req.body.formData.length === 0)}
              <tr>
                <td colspan="4" class="p-6 text-center text-slate-500 font-sans">
                  No URL-encoded fields defined. Click "+ Add Field" above.
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>
    </div>

  <!-- MULTIPART FORM DATA -->
  {:else if req.body.type === 'multipart'}
    <div class="space-y-2 flex-1 flex flex-col">
      <input
        type="file"
        bind:this={multipartFileInput}
        onchange={onMultipartFileInputChange}
        class="hidden"
      />
      <div class="border border-slate-800 rounded-lg overflow-hidden bg-slate-950">
        <table class="w-full border-collapse text-left font-mono text-xs">
          <thead>
            <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
              <th class="w-8 p-2 text-center"></th>
              <th class="p-2 border-r border-slate-800 w-1/4">Field Name</th>
              <th class="p-2 border-r border-slate-800 w-24 text-center">Type</th>
              <th class="p-2 border-r border-slate-800">Value / File Path</th>
              <th class="w-10 p-2 text-center"></th>
            </tr>
          </thead>
          <tbody>
            {#each (req.body.formData || []) as item, idx (item.id)}
              <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                <td class="p-2 text-center">
                  <input
                    type="checkbox"
                    bind:checked={item.enabled}
                    onchange={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                  />
                </td>
                <td class="p-1 border-r border-slate-800">
                  <input
                    type="text"
                    bind:value={item.key}
                    placeholder="field_name"
                    oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="w-full bg-transparent px-2 py-1 text-sky-300 focus:outline-none"
                  />
                </td>
                <td class="p-1 border-r border-slate-800 text-center font-sans">
                  <button
                    type="button"
                    onclick={() => {
                      item.type = item.type === 'file' ? 'text' : 'file';
                      workspaceStore.updateActiveRequest({ body: req.body });
                    }}
                    class="px-2 py-0.5 rounded text-[11px] font-semibold {item.type === 'file' ? 'bg-purple-950/80 text-purple-300 border border-purple-800/60' : 'bg-slate-800 text-slate-300 border border-slate-700'}"
                  >
                    {item.type === 'file' ? '📁 File' : 'Aa Text'}
                  </button>
                </td>
                <td class="p-1 border-r border-slate-800">
                  {#if item.type === 'file'}
                    <div class="flex items-center gap-1.5 px-1">
                      <span class="text-purple-400 font-mono text-[11px]">@</span>
                      <input
                        type="text"
                        value={item.value.startsWith('@') ? item.value.slice(1) : item.value}
                        oninput={(e) => {
                          item.value = (e.target as HTMLInputElement).value;
                          workspaceStore.updateActiveRequest({ body: req.body });
                        }}
                        placeholder="Relative path e.g. ./fixtures/avatar.png"
                        class="w-full bg-transparent py-1 text-purple-200 focus:outline-none font-mono text-xs"
                      />
                      <button
                        type="button"
                        onclick={() => triggerMultipartFileBrowse(idx)}
                        class="px-2 py-0.5 rounded bg-purple-950/80 hover:bg-purple-900 border border-purple-800/60 text-purple-200 text-[11px] font-sans font-medium whitespace-nowrap cursor-pointer shrink-0"
                      >
                        📁 Browse...
                      </button>
                    </div>
                  {:else}
                    <input
                      type="text"
                      bind:value={item.value}
                      placeholder="value"
                      oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  {/if}
                </td>
                <td class="p-2 text-center">
                  <button
                    type="button"
                    onclick={() => removeFormData(idx)}
                    class="text-slate-500 hover:text-rose-400"
                  >
                    ✕
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <p class="text-[11px] text-slate-500 font-sans">
        💡 <span class="font-medium text-slate-400">Relative Path Anchoring:</span> File uploads strictly resolve relative to workspace root (e.g. <code class="text-purple-300 font-mono">./fixtures/avatar.png</code>).
      </p>
    </div>

  <!-- BINARY FILE UPLOAD -->
  {:else if req.body.type === 'binary'}
    <div class="space-y-4 max-w-2xl bg-slate-900/40 p-5 border border-slate-800 rounded-lg">
      <input
        type="file"
        bind:this={binaryFileInput}
        onchange={onBinaryFileInputChange}
        class="hidden"
      />
      <div class="flex items-start justify-between">
        <div>
          <span class="text-xs font-semibold text-slate-300 uppercase tracking-wider block">Binary File Payload</span>
          <p class="text-[11px] text-slate-500 mt-1">
            Upload raw file content directly as request body. Resolves relative to workspace root.
          </p>
        </div>
        {#if req.body.binaryFilePath}
          <button
            type="button"
            onclick={clearBinaryFile}
            class="text-[11px] text-slate-500 hover:text-rose-400 flex items-center gap-1 cursor-pointer"
          >
            ✕ Clear File
          </button>
        {/if}
      </div>

      <div
        role="region"
        aria-label="File dropzone"
        ondragover={(e) => { e.preventDefault(); isBinaryDragging = true; }}
        ondragleave={() => { isBinaryDragging = false; }}
        ondrop={handleBinaryDrop}
        class="border-2 border-dashed rounded-lg p-5 text-center {isBinaryDragging ? 'border-sky-500 bg-sky-500/10' : req.body.binaryFilePath ? 'border-slate-700 bg-slate-950/60' : 'border-slate-800 hover:border-slate-700 bg-slate-950/30'}"
      >
        {#if req.body.binaryFilePath}
          <div class="flex items-center justify-between gap-4 text-left">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-10 h-10 rounded bg-sky-500/10 border border-sky-500/30 flex items-center justify-center text-xl shrink-0">
                📦
              </div>
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <span class="font-mono text-xs font-semibold text-sky-300 truncate">
                    {binarySelectedFileInfo?.name || req.body.binaryFilePath.split(/[/\\]/).pop()}
                  </span>
                  {#if binarySelectedFileInfo?.size}
                    <span class="text-[10px] text-slate-400 bg-slate-800 px-1.5 py-0.5 rounded font-mono">
                      {formatSize(binarySelectedFileInfo.size)}
                    </span>
                  {/if}
                </div>
                <p class="text-[11px] text-slate-500 font-mono truncate mt-0.5">
                  {req.body.binaryFilePath}
                </p>
              </div>
            </div>
            <button
              type="button"
              onclick={() => binaryFileInput?.click()}
              class="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium border border-slate-700 cursor-pointer"
            >
              📁 Change File...
            </button>
          </div>
        {:else}
          <div class="space-y-2 py-4">
            <div class="text-3xl">📁</div>
            <div class="text-xs text-slate-300">
              Drag and drop your file here, or
              <button
                type="button"
                onclick={() => binaryFileInput?.click()}
                class="text-sky-400 hover:underline font-semibold ml-1 cursor-pointer"
              >
                browse local files
              </button>
            </div>
            <p class="text-[11px] text-slate-500 font-mono">
              Payload is anchored relative to workspace
            </p>
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="p-8 text-center text-slate-500 italic">
      No body payload configured for this request.
    </div>
  {/if}
</div>
