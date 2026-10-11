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
  <div class="flex items-center justify-between shrink-0 flex-wrap gap-2 pb-1 border-b border-apple-hairline-dark/60">
    <div class="flex items-center gap-2 flex-wrap">
      <span class="text-xs text-apple-body-muted font-medium">Body Type:</span>
      <div class="flex items-center gap-1.5 flex-wrap">
        {#each ['none', 'json', 'xml', 'text', 'form-urlencoded', 'multipart', 'binary'] as bType}
          <button
            type="button"
            onclick={() => switchBodyType(bType as BodyType)}
            class="px-3 py-1 rounded-apple-pill text-xs font-medium transition-all {req.body.type === bType ? 'apple-button-primary font-semibold text-white shadow-none' : 'apple-button-dark-utility bg-apple-surface-tile-3 border border-apple-hairline-dark/60 text-apple-body-muted hover:text-white'}"
          >
            {bType}
          </button>
        {/each}
      </div>
    </div>

    <!-- Toolbar Actions -->
    <div class="flex items-center gap-2">
      {#if bodyActionFeedback}
        <span class="text-[11px] px-2.5 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 border border-apple-hairline-dark/60 text-apple-primary-on-dark font-mono">
          {bodyActionFeedback}
        </span>
      {/if}

      {#if req.body.type === 'json' || req.body.type === 'xml' || req.body.type === 'text' || req.body.type === 'raw'}
        <button
          type="button"
          onclick={() => formatOrCompressBody('format')}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark border border-apple-hairline-dark/60 text-xs font-medium flex items-center gap-1 transition-colors cursor-pointer"
          title="Format / Prettify payload"
        >
          <span>✨</span> Format
        </button>
        <button
          type="button"
          onclick={() => formatOrCompressBody('compress')}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark border border-apple-hairline-dark/60 text-xs font-medium flex items-center gap-1 transition-colors cursor-pointer"
          title="Minify / Compress payload"
        >
          <span>⚡</span> Compress
        </button>
        <button
          type="button"
          onclick={() => {
            if (req?.body?.rawContent) {
              navigator.clipboard.writeText(req.body.rawContent);
              feedback('Copied to clipboard');
            }
          }}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-muted hover:text-white border border-apple-hairline-dark/60 text-xs transition-colors cursor-pointer"
          title="Copy body to clipboard"
        >
          📋 Copy
        </button>

      {:else if req.body.type === 'form-urlencoded'}
        <button
          type="button"
          onclick={() => addFormData('text')}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-primary-on-dark border border-apple-hairline-dark/60 text-xs font-medium flex items-center gap-1 cursor-pointer"
        >
          + Add Field
        </button>
        <button
          type="button"
          onclick={cleanFormData}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-muted hover:text-white border border-apple-hairline-dark/60 text-xs cursor-pointer"
        >
          Clean / Trim
        </button>

      {:else if req.body.type === 'multipart'}
        <button
          type="button"
          onclick={() => addFormData('text')}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-primary-on-dark border border-apple-hairline-dark/60 text-xs font-medium flex items-center gap-1 cursor-pointer"
        >
          + Text Field
        </button>
        <button
          type="button"
          onclick={() => addFormData('file')}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-primary-on-dark border border-apple-hairline-dark/60 text-xs font-medium flex items-center gap-1 cursor-pointer"
        >
          📁 + File Upload
        </button>
        <button
          type="button"
          onclick={cleanFormData}
          class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-muted hover:text-white border border-apple-hairline-dark/60 text-xs cursor-pointer"
        >
          Clean / Trim
        </button>
      {/if}
    </div>
  </div>

  <!-- JSON / XML / TEXT EDITOR -->
  {#if req.body.type === 'json' || req.body.type === 'xml' || req.body.type === 'text' || req.body.type === 'raw'}
    <div class="flex-1 flex flex-col border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black min-h-[260px]">
      <div class="bg-apple-surface-tile-2 px-3 py-1.5 border-b border-apple-hairline-dark/60 flex items-center justify-between text-[11px] text-apple-body-muted">
        <div class="flex items-center gap-2">
          <span class="font-semibold text-apple-body-on-dark">
            {req.body.type === 'json' ? 'JSON Payload' : req.body.type === 'xml' ? 'XML Document' : 'Plain Text Payload'}
          </span>
          <span class="text-[10px] px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark font-mono border border-apple-hairline-dark/60">
            {req.body.type === 'json' ? 'application/json' : req.body.type === 'xml' ? 'application/xml' : 'text/plain'}
          </span>
        </div>
        <div class="flex items-center gap-3 font-mono text-apple-ink-muted-48 text-[10px]">
          <span>{(req.body.rawContent || '').split('\n').length} lines</span>
          <span>{(req.body.rawContent || '').length} chars</span>
        </div>
      </div>
      <textarea
        bind:value={req.body.rawContent}
        oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
        placeholder={req.body.type === 'json' ? '{\n  "key": "value"\n}' : req.body.type === 'xml' ? '<root>\n  <item>value</item>\n</root>' : 'Raw body content'}
        class="flex-1 w-full bg-apple-surface-black p-3 font-mono text-xs text-apple-body-on-dark resize-none focus:outline-none leading-relaxed"
        spellcheck="false"
      ></textarea>
    </div>

  <!-- FORM-URLENCODED -->
  {:else if req.body.type === 'form-urlencoded'}
    <div class="space-y-2 flex-1 flex flex-col">
      <div class="border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black">
        <table class="w-full border-collapse text-left font-mono text-xs">
          <thead>
            <tr class="bg-apple-surface-tile-2 text-apple-body-muted border-b border-apple-hairline-dark/60 text-[11px]">
              <th class="w-8 p-2 text-center"></th>
              <th class="p-2 border-r border-apple-hairline-dark/60">Parameter Key</th>
              <th class="p-2 border-r border-apple-hairline-dark/60">Parameter Value</th>
              <th class="w-10 p-2 text-center"></th>
            </tr>
          </thead>
          <tbody>
            {#each (req.body.formData || []) as item, idx (item.id)}
              <tr class="border-b border-apple-hairline-dark/40 hover:bg-apple-surface-tile-1/50 transition-colors">
                <td class="p-2 text-center">
                  <input
                    type="checkbox"
                    bind:checked={item.enabled}
                    onchange={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0"
                  />
                </td>
                <td class="p-1 border-r border-apple-hairline-dark/60">
                  <input
                    type="text"
                    bind:value={item.key}
                    placeholder="param_name"
                    oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="w-full bg-transparent px-2 py-1 text-apple-primary-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
                  />
                </td>
                <td class="p-1 border-r border-apple-hairline-dark/60">
                  <input
                    type="text"
                    bind:value={item.value}
                    placeholder="value"
                    oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
                  />
                </td>
                <td class="p-2 text-center">
                  <button
                    type="button"
                    onclick={() => removeFormData(idx)}
                    class="text-apple-ink-muted-48 hover:text-rose-400 transition-colors"
                  >
                    ✕
                  </button>
                </td>
              </tr>
            {/each}
            {#if (!req.body.formData || req.body.formData.length === 0)}
              <tr>
                <td colspan="4" class="p-6 text-center text-apple-ink-muted-48 font-sans">
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
      <div class="border border-apple-hairline-dark/60 rounded-apple-lg overflow-hidden bg-apple-surface-black">
        <table class="w-full border-collapse text-left font-mono text-xs">
          <thead>
            <tr class="bg-apple-surface-tile-2 text-apple-body-muted border-b border-apple-hairline-dark/60 text-[11px]">
              <th class="w-8 p-2 text-center"></th>
              <th class="p-2 border-r border-apple-hairline-dark/60 w-1/4">Field Name</th>
              <th class="p-2 border-r border-apple-hairline-dark/60 w-24 text-center">Type</th>
              <th class="p-2 border-r border-apple-hairline-dark/60">Value / File Path</th>
              <th class="w-10 p-2 text-center"></th>
            </tr>
          </thead>
          <tbody>
            {#each (req.body.formData || []) as item, idx (item.id)}
              <tr class="border-b border-apple-hairline-dark/40 hover:bg-apple-surface-tile-1/50 transition-colors">
                <td class="p-2 text-center">
                  <input
                    type="checkbox"
                    bind:checked={item.enabled}
                    onchange={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark/80 text-apple-primary focus:ring-0"
                  />
                </td>
                <td class="p-1 border-r border-apple-hairline-dark/60">
                  <input
                    type="text"
                    bind:value={item.key}
                    placeholder="field_name"
                    oninput={() => workspaceStore.updateActiveRequest({ body: req.body })}
                    class="w-full bg-transparent px-2 py-1 text-apple-primary-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
                  />
                </td>
                <td class="p-1 border-r border-apple-hairline-dark/60 text-center font-sans">
                  <button
                    type="button"
                    onclick={() => {
                      item.type = item.type === 'file' ? 'text' : 'file';
                      workspaceStore.updateActiveRequest({ body: req.body });
                    }}
                    class="apple-button-dark-utility px-2 py-0.5 rounded-apple-sm text-[11px] font-semibold border border-apple-hairline-dark/60 {item.type === 'file' ? 'bg-apple-surface-tile-3 text-apple-primary-on-dark' : 'bg-apple-surface-tile-1 text-apple-body-muted'}"
                  >
                    {item.type === 'file' ? '📁 File' : 'Aa Text'}
                  </button>
                </td>
                <td class="p-1 border-r border-apple-hairline-dark/60">
                  {#if item.type === 'file'}
                    <div class="flex items-center gap-1.5 px-1">
                      <span class="text-apple-primary-on-dark font-mono text-[11px]">@</span>
                      <input
                        type="text"
                        value={item.value.startsWith('@') ? item.value.slice(1) : item.value}
                        oninput={(e) => {
                          item.value = (e.target as HTMLInputElement).value;
                          workspaceStore.updateActiveRequest({ body: req.body });
                        }}
                        placeholder="Relative path e.g. ./fixtures/avatar.png"
                        class="w-full bg-transparent py-1 text-apple-body-on-dark focus:outline-none font-mono text-xs placeholder:text-apple-ink-muted-48"
                      />
                      <button
                        type="button"
                        onclick={() => triggerMultipartFileBrowse(idx)}
                        class="apple-button-dark-utility px-2 py-0.5 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 border border-apple-hairline-dark/60 text-apple-primary-on-dark text-[11px] font-sans font-medium whitespace-nowrap cursor-pointer shrink-0"
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
                      class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none placeholder:text-apple-ink-muted-48"
                    />
                  {/if}
                </td>
                <td class="p-2 text-center">
                  <button
                    type="button"
                    onclick={() => removeFormData(idx)}
                    class="text-apple-ink-muted-48 hover:text-rose-400 transition-colors"
                  >
                    ✕
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <p class="text-[11px] text-apple-ink-muted-48 font-sans">
        💡 <span class="font-medium text-apple-body-muted">Relative Path Anchoring:</span> File uploads strictly resolve relative to workspace root (e.g. <code class="text-apple-primary-on-dark font-mono">./fixtures/avatar.png</code>).
      </p>
    </div>

  <!-- BINARY FILE UPLOAD -->
  {:else if req.body.type === 'binary'}
    <div class="space-y-4 max-w-2xl bg-apple-surface-tile-2 p-5 border border-apple-hairline-dark/60 rounded-apple-lg">
      <input
        type="file"
        bind:this={binaryFileInput}
        onchange={onBinaryFileInputChange}
        class="hidden"
      />
      <div class="flex items-start justify-between">
        <div>
          <span class="text-xs font-semibold text-apple-body-on-dark uppercase tracking-wider block">Binary File Payload</span>
          <p class="text-[11px] text-apple-ink-muted-48 mt-1">
            Upload raw file content directly as request body. Resolves relative to workspace root.
          </p>
        </div>
        {#if req.body.binaryFilePath}
          <button
            type="button"
            onclick={clearBinaryFile}
            class="text-[11px] text-apple-ink-muted-48 hover:text-rose-400 flex items-center gap-1 cursor-pointer transition-colors"
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
        class="border-2 border-dashed rounded-apple-lg p-5 text-center transition-colors {isBinaryDragging ? 'border-apple-primary bg-apple-primary/10' : req.body.binaryFilePath ? 'border-apple-hairline-dark bg-apple-surface-black' : 'border-apple-hairline-dark/60 hover:border-apple-hairline-dark bg-apple-surface-tile-1'}"
      >
        {#if req.body.binaryFilePath}
          <div class="flex items-center justify-between gap-4 text-left">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-10 h-10 rounded-apple-md bg-apple-surface-tile-3 border border-apple-hairline-dark/60 flex items-center justify-center text-xl shrink-0">
                📦
              </div>
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <span class="font-mono text-xs font-semibold text-apple-primary-on-dark truncate">
                    {binarySelectedFileInfo?.name || req.body.binaryFilePath.split(/[/\\]/).pop()}
                  </span>
                  {#if binarySelectedFileInfo?.size}
                    <span class="text-[10px] text-apple-body-muted bg-apple-surface-tile-3 border border-apple-hairline-dark/60 px-1.5 py-0.5 rounded-apple-sm font-mono">
                      {formatSize(binarySelectedFileInfo.size)}
                    </span>
                  {/if}
                </div>
                <p class="text-[11px] text-apple-ink-muted-48 font-mono truncate mt-0.5">
                  {req.body.binaryFilePath}
                </p>
              </div>
            </div>
            <button
              type="button"
              onclick={() => binaryFileInput?.click()}
              class="apple-button-dark-utility px-2.5 py-1 rounded-apple-sm bg-apple-surface-tile-3 hover:bg-apple-surface-tile-1 text-apple-body-on-dark text-xs font-medium border border-apple-hairline-dark/60 cursor-pointer"
            >
              📁 Change File...
            </button>
          </div>
        {:else}
          <div class="space-y-2 py-4">
            <div class="text-3xl">📁</div>
            <div class="text-xs text-apple-body-muted">
              Drag and drop your file here, or
              <button
                type="button"
                onclick={() => binaryFileInput?.click()}
                class="text-apple-primary-on-dark hover:underline font-semibold ml-1 cursor-pointer"
              >
                browse local files
              </button>
            </div>
            <p class="text-[11px] text-apple-ink-muted-48 font-mono">
              Payload is anchored relative to workspace
            </p>
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="p-8 text-center text-apple-ink-muted-48 italic">
      No body payload configured for this request.
    </div>
  {/if}
</div>
