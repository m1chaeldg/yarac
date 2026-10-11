<script lang="ts">
  import { workspaceStore, type KeyValuePair, type AuthType, type BodyType } from '../stores/workspace.svelte';
  import FlowPanel from './FlowPanel.svelte';

  let activeSubTab = $state<'params' | 'headers' | 'variables' | 'auth' | 'body' | 'scripts' | 'assert' | 'tests' | 'settings' | 'flow'>('headers');
  let activeScriptType = $state<'preRequest' | 'postResponse'>('preRequest');

  let req = $derived(workspaceStore.activeRequest);

  // Common Header suggestions
  const commonHeaders = [
    'Content-Type',
    'Accept',
    'Authorization',
    'User-Agent',
    'Cache-Control',
    'X-Request-Id',
    'X-Api-Key'
  ];

  // Bruno & Faker dynamic variables quick list
  const dynamicVars = [
    { label: '$uuid', desc: 'UUID v4' },
    { label: '$timestamp', desc: 'Current UNIX timestamp' },
    { label: '$isoTimestamp', desc: 'ISO 8601 timestamp' },
    { label: '$randomEmail', desc: 'Random email address' },
    { label: '$randomFullName', desc: 'Random full name' },
    { label: '$randomInt', desc: 'Random integer (1-1000)' },
    { label: '$randomPassword', desc: 'Random secure password' },
    { label: '$randomCity', desc: 'Random city' },
    { label: '$randomCountry', desc: 'Random country' },
    { label: '$randomPrice', desc: 'Random price (0-1000)' },
    { label: '$randomCompany', desc: 'Random company' }
  ];

  function addParam() {
    if (!req) return;
    req.params.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      description: ''
    });
    workspaceStore.updateActiveRequest({ params: req.params });
  }

  function removeParam(index: number) {
    if (!req) return;
    req.params.splice(index, 1);
    workspaceStore.updateActiveRequest({ params: req.params });
  }

  function addHeader(presetKey = '') {
    if (!req) return;
    req.headers.push({
      id: Math.random().toString(36).substring(2, 9),
      key: presetKey,
      value: presetKey === 'Content-Type' ? 'application/json' : '',
      enabled: true,
      description: ''
    });
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }

  function removeHeader(index: number) {
    if (!req) return;
    req.headers.splice(index, 1);
    workspaceStore.updateActiveRequest({ headers: req.headers });
  }

  function addVariable() {
    if (!req) return;
    if (!req.variables) req.variables = [];
    req.variables.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      description: '',
      isSecret: false
    });
    workspaceStore.updateActiveRequest({ variables: req.variables });
  }

  function removeVariable(index: number) {
    if (!req || !req.variables) return;
    req.variables.splice(index, 1);
    workspaceStore.updateActiveRequest({ variables: req.variables });
  }

  let bodyActionFeedback = $state('');

  function setBodyType(bType: BodyType) {
    if (!req) return;
    req.body.type = bType;
    if (!req.body.formData) req.body.formData = [];
    if (!req.body.rawContent) req.body.rawContent = '';
    if (bType === 'binary' && !req.body.binaryContentType) {
      req.body.binaryContentType = 'application/octet-stream';
    }

    // Auto-align Content-Type header if helpful
    const typeHeaderMap: Partial<Record<BodyType, string>> = {
      json: 'application/json',
      xml: 'application/xml',
      text: 'text/plain',
      'form-urlencoded': 'application/x-www-form-urlencoded',
      multipart: 'multipart/form-data',
      binary: req.body.binaryContentType || 'application/octet-stream'
    };
    const targetHeader = typeHeaderMap[bType];
    if (targetHeader) {
      const ctHeader = req.headers.find(h => h.key.toLowerCase() === 'content-type');
      if (ctHeader && ctHeader.enabled) {
        ctHeader.value = targetHeader;
      }
    }

    workspaceStore.updateActiveRequest({ body: req.body, headers: req.headers });
  }

  function addFormData(type: 'text' | 'file' = 'text') {
    if (!req) return;
    if (!req.body.formData) req.body.formData = [];
    req.body.formData.push({
      id: Math.random().toString(36).substring(2, 9),
      key: '',
      value: '',
      enabled: true,
      type
    });
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function removeFormData(index: number) {
    if (!req || !req.body.formData) return;
    req.body.formData.splice(index, 1);
    workspaceStore.updateActiveRequest({ body: req.body });
  }

  function cleanFormData() {
    if (!req || !req.body.formData) return;
    req.body.formData = req.body.formData
      .filter(f => f.key.trim() || f.value.trim())
      .map(f => ({ ...f, key: f.key.trim(), value: f.value.trim() }));
    workspaceStore.updateActiveRequest({ body: req.body });
    bodyActionFeedback = 'Cleaned empty form fields';
    setTimeout(() => (bodyActionFeedback = ''), 2000);
  }

  function formatJsonBody() {
    if (!req || req.body.type !== 'json') return;
    try {
      const parsed = JSON.parse(req.body.rawContent || '{}');
      req.body.rawContent = JSON.stringify(parsed, null, 2);
      workspaceStore.updateActiveRequest({ body: req.body });
      bodyActionFeedback = 'Formatted JSON';
      setTimeout(() => (bodyActionFeedback = ''), 2000);
    } catch (e: any) {
      bodyActionFeedback = 'Invalid JSON: ' + (e.message || String(e));
      setTimeout(() => (bodyActionFeedback = ''), 3000);
    }
  }

  function compressJsonBody() {
    if (!req || req.body.type !== 'json') return;
    try {
      const parsed = JSON.parse(req.body.rawContent || '{}');
      req.body.rawContent = JSON.stringify(parsed);
      workspaceStore.updateActiveRequest({ body: req.body });
      bodyActionFeedback = 'Compressed JSON';
      setTimeout(() => (bodyActionFeedback = ''), 2000);
    } catch (e: any) {
      bodyActionFeedback = 'Invalid JSON: ' + (e.message || String(e));
      setTimeout(() => (bodyActionFeedback = ''), 3000);
    }
  }

  function compressXmlString(xml: string): string {
    if (!xml) return '';
    // 1. Preserve CDATA blocks
    const cdataBlocks: string[] = [];
    let processed = xml.replace(/<!\[CDATA\[[\s\S]*?\]\]>/g, (match) => {
      cdataBlocks.push(match);
      return `___CDATA_${cdataBlocks.length - 1}___`;
    });

    // 2. Remove comments completely in compressed mode
    processed = processed.replace(/<!--[\s\S]*?-->/g, '');

    // 3. Remove leading/trailing newlines and spaces across whole document
    processed = processed.trim();

    // 4. Remove all whitespace between tags: >  \n  < -> ><
    processed = processed.replace(/>\s+</g, '><');

    // 5. Trim whitespace around text contents between tags: >  text  < -> >text<
    processed = processed.replace(/>\s*([^<]+?)\s*</g, (_, text) => {
      return `>${text.trim()}<`;
    });

    // 6. Normalize and collapse spaces/newlines inside opening tags & attributes
    processed = processed.replace(/<([^>!][^>]*?)>/g, (_, tagContent) => {
      return '<' + tagContent.replace(/\s+/g, ' ').trim() + '>';
    });

    // 7. Collapse any whitespace that was generated
    processed = processed.replace(/>\s+</g, '><');

    // 8. Restore CDATA blocks
    processed = processed.replace(/___CDATA_(\d+)___/g, (_, idx) => {
      return cdataBlocks[parseInt(idx, 10)] || '';
    });

    return processed.trim();
  }

  function compressXmlBody() {
    if (!req || req.body.type !== 'xml') return;
    try {
      req.body.rawContent = compressXmlString(req.body.rawContent || '');
      workspaceStore.updateActiveRequest({ body: req.body });
      bodyActionFeedback = 'Compressed XML';
      setTimeout(() => (bodyActionFeedback = ''), 2000);
    } catch (e: any) {
      bodyActionFeedback = 'Error compressing XML: ' + (e.message || String(e));
      setTimeout(() => (bodyActionFeedback = ''), 3000);
    }
  }

  function formatXmlBody() {
    if (!req || req.body.type !== 'xml') return;
    try {
      const clean = compressXmlString(req.body.rawContent || '');
      let formatted = '';
      let pad = 0;
      const tokens = clean.match(/(<!\[CDATA\[[\s\S]*?\]\]>|<!--[\s\S]*?-->|<\?[^?]+\?>|<!DOCTYPE[^>]+>|<\/[^>]+>|<[^\/>]+(\/>|>)|[^<]+)/g) || [clean];

      for (let i = 0; i < tokens.length; i++) {
        const token = tokens[i].trim();
        if (!token) continue;

        if (token.startsWith('</')) {
          pad = Math.max(0, pad - 1);
          formatted += '  '.repeat(pad) + token + '\n';
        } else if (token.startsWith('<') && !token.startsWith('<?') && !token.startsWith('<!') && !token.endsWith('/>') && !token.startsWith('<!--')) {
          const nextToken = tokens[i + 1]?.trim();
          const nextNextToken = tokens[i + 2]?.trim();
          const tagNameMatch = token.match(/<([^\s>]+)/);
          const tagName = tagNameMatch ? tagNameMatch[1] : '';

          if (nextToken && !nextToken.startsWith('<') && nextNextToken === `</${tagName}>`) {
            formatted += '  '.repeat(pad) + token + nextToken + nextNextToken + '\n';
            i += 2;
          } else {
            formatted += '  '.repeat(pad) + token + '\n';
            pad += 1;
          }
        } else {
          formatted += '  '.repeat(pad) + token + '\n';
        }
      }
      req.body.rawContent = formatted.trimEnd();
      workspaceStore.updateActiveRequest({ body: req.body });
      bodyActionFeedback = 'Formatted XML';
      setTimeout(() => (bodyActionFeedback = ''), 2000);
    } catch (e: any) {
      bodyActionFeedback = 'Error formatting XML: ' + (e.message || String(e));
      setTimeout(() => (bodyActionFeedback = ''), 3000);
    }
  }

  function formatTextBody() {
    if (!req || (req.body.type !== 'text' && req.body.type !== 'raw')) return;
    req.body.rawContent = (req.body.rawContent || '')
      .split('\n')
      .map(line => line.trimEnd())
      .join('\n')
      .trim();
    workspaceStore.updateActiveRequest({ body: req.body });
    bodyActionFeedback = 'Normalized lines & whitespace';
    setTimeout(() => (bodyActionFeedback = ''), 2000);
  }

  function compressTextBody() {
    if (!req || (req.body.type !== 'text' && req.body.type !== 'raw')) return;
    req.body.rawContent = (req.body.rawContent || '').replace(/\s+/g, ' ').trim();
    workspaceStore.updateActiveRequest({ body: req.body });
    bodyActionFeedback = 'Compressed text whitespace';
    setTimeout(() => (bodyActionFeedback = ''), 2000);
  }

  function formatOrCompressBody(action: 'format' | 'compress') {
    if (!req) return;
    if (req.body.type === 'json') {
      action === 'format' ? formatJsonBody() : compressJsonBody();
    } else if (req.body.type === 'xml') {
      action === 'format' ? formatXmlBody() : compressXmlBody();
    } else if (req.body.type === 'text' || req.body.type === 'raw') {
      action === 'format' ? formatTextBody() : compressTextBody();
    } else if (req.body.type === 'form-urlencoded' || req.body.type === 'multipart') {
      cleanFormData();
    }
  }

  function insertScriptSnippet(snippet: string) {
    if (!req) return;
    if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
    if (activeScriptType === 'preRequest') {
      const cur = req.scripts.preRequest ? req.scripts.preRequest + '\n' : '';
      req.scripts.preRequest = cur + snippet;
    } else {
      const cur = req.scripts.postResponse ? req.scripts.postResponse + '\n' : '';
      req.scripts.postResponse = cur + snippet;
    }
    workspaceStore.updateActiveRequest({ scripts: req.scripts });
  }

  function insertAssertSnippet(snippet: string) {
    if (!req) return;
    const cur = req.assert ? req.assert + '\n' : '';
    req.assert = cur + snippet;
    workspaceStore.updateActiveRequest({ assert: req.assert });
  }

  function insertTestSnippet(snippet: string) {
    if (!req) return;
    const cur = req.tests ? req.tests + '\n' : '';
    req.tests = cur + snippet;
    workspaceStore.updateActiveRequest({ tests: req.tests });
  }

  let showBearerSecret = $state(false);

  const bearerTokenPlaceholder = 'e.g. {{api_bearer_token}}';
  const basicPasswordPlaceholder = 'Password or {{api_password}}';
  const apiKeyPlaceholder = '{{api_key}}';
  const varValuePlaceholder = 'value or {{$uuid}}';
  const jsonBodyPlaceholder = '{\n  "key": "value"\n}';
  const xmlBodyPlaceholder = '<?xml version="1.0" encoding="UTF-8"?>\n<request>\n  <item id="1">sample</item>\n</request>';
  const textBodyPlaceholder = 'Enter plain text body payload here...';
  const formValuePlaceholder = 'value or {{param}}';
  const multipartValuePlaceholder = 'Text value or {{param}}';
  const preScriptPlaceholder = '// JavaScript pre-request script\n// Available globals: req, yarac.getVar, yarac.setVar, faker\nconst token = faker.string.uuid();\nyarac.setVar("session_token", token);';
  const postScriptPlaceholder = '// JavaScript post-response script\n// Available globals: res, req, yarac.getVar, yarac.setVar, faker\nif (res.status === 200) {\n  const data = res.json();\n  console.log("Received response:", data);\n}';
  const assertPlaceholder = '// Single-line assertions:\nexpect(res.status).toBe(200);\nexpect(res.time).toBeLessThan(1500);\nexpect(res.json().status).toBe("success");';
  const testsPlaceholder = 'describe("User Service API", () => {\n  it("should return 200 OK", () => {\n    expect(res.status).toBe(200);\n  });\n\n  it("should match schema and payload", () => {\n    const data = res.json();\n    expect(data.id).toBeDefined();\n    expect(data.email).toContain("@");\n  });\n});';

  let binaryFileInput = $state<HTMLInputElement | null>(null);
  let multipartFileInput = $state<HTMLInputElement | null>(null);
  let activeMultipartRowIdx = $state<number | null>(null);
  let isBinaryDragging = $state(false);
  let binarySelectedFileInfo = $state<{ name: string; size: number; type: string } | null>(null);

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function handleBinaryFileSelect(file: File) {
    if (!req) return;
    // Relative Path Anchoring per AGENTS.md: all file uploads strictly resolve relative to workspace root
    req.body.binaryFilePath = `./fixtures/${file.name}`;
    if (file.type) {
      req.body.binaryContentType = file.type;
    } else if (!req.body.binaryContentType) {
      req.body.binaryContentType = 'application/octet-stream';
    }
    binarySelectedFileInfo = {
      name: file.name,
      size: file.size,
      type: file.type || 'application/octet-stream'
    };
    workspaceStore.updateActiveRequest({ body: req.body });
    bodyActionFeedback = `Selected file: ${file.name}`;
    setTimeout(() => (bodyActionFeedback = ''), 2500);
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
    if (!req) return;
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
        if (file.type) {
          item.contentType = file.type;
        }
        workspaceStore.updateActiveRequest({ body: req.body });
        bodyActionFeedback = `Attached file to ${item.key || 'field'}: ${file.name}`;
        setTimeout(() => (bodyActionFeedback = ''), 2500);
      }
    }
    activeMultipartRowIdx = null;
    target.value = '';
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
          class="px-2 py-1 rounded font-medium transition-colors {activeSubTab === 'params' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors {activeSubTab === 'headers' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'variables' ? 'bg-slate-800 text-amber-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'auth' ? 'bg-slate-800 text-emerald-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'body' ? 'bg-slate-800 text-sky-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'scripts' ? 'bg-slate-800 text-indigo-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'assert' ? 'bg-slate-800 text-teal-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'tests' ? 'bg-slate-800 text-pink-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'settings' ? 'bg-slate-800 text-cyan-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
        >
          <span>Settings</span>
        </button>

        <!-- FLOW -->
        <button
          type="button"
          onclick={() => activeSubTab = 'flow'}
          class="px-2 py-1 rounded font-medium transition-colors flex items-center gap-1 {activeSubTab === 'flow' ? 'bg-slate-800 text-purple-400 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
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
            {#each commonHeaders as h}
              <option value={h}>{h}</option>
            {/each}
          </select>
        </div>
      {/if}
    </div>

    <!-- Sub-Tab Content Viewport -->
    <div class="flex-1 overflow-y-auto p-3">
      <!-- PARAMS TAB -->
      {#if activeSubTab === 'params'}
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Query Parameters</span>
            <button
              type="button"
              onclick={addParam}
              class="text-xs text-sky-400 hover:text-sky-300 font-medium"
            >
              + Add Parameter
            </button>
          </div>

          <table class="w-full border-collapse border border-slate-800 text-left font-mono">
            <thead>
              <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
                <th class="w-8 p-2 text-center"></th>
                <th class="p-2 border-r border-slate-800">Parameter Key</th>
                <th class="p-2 border-r border-slate-800">Value</th>
                <th class="p-2 border-r border-slate-800 font-sans">Description</th>
                <th class="w-10 p-2 text-center"></th>
              </tr>
            </thead>
            <tbody>
              {#each req.params as param, idx (param.id)}
                <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                  <td class="p-2 text-center">
                    <input
                      type="checkbox"
                      bind:checked={param.enabled}
                      onchange={() => workspaceStore.updateActiveRequest({ params: req?.params })}
                      class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
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
                        workspaceStore.updateActiveRequest({ params: req?.params });
                      }}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={param.value}
                      placeholder="e.g. 50"
                      oninput={() => workspaceStore.updateActiveRequest({ params: req?.params })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800 font-sans">
                    <input
                      type="text"
                      bind:value={param.description}
                      placeholder="Description"
                      oninput={() => workspaceStore.updateActiveRequest({ params: req?.params })}
                      class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
                    />
                  </td>
                  <td class="p-2 text-center">
                    <button
                      type="button"
                      onclick={() => removeParam(idx)}
                      class="text-slate-500 hover:text-rose-400"
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              {/each}
              {#if req.params.length === 0}
                <tr>
                  <td colspan="5" class="p-4 text-center text-slate-500 font-sans">
                    No query parameters configured. Click "+ Add Parameter" above.
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>

      <!-- HEADERS TAB -->
      {:else if activeSubTab === 'headers'}
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">HTTP Request Headers</span>
            <button
              type="button"
              onclick={() => addHeader()}
              class="text-xs text-sky-400 hover:text-sky-300 font-medium"
            >
              + Add Header
            </button>
          </div>

          <table class="w-full border-collapse border border-slate-800 text-left font-mono">
            <thead>
              <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
                <th class="w-8 p-2 text-center"></th>
                <th class="p-2 border-r border-slate-800">Header Key</th>
                <th class="p-2 border-r border-slate-800">Header Value</th>
                <th class="p-2 border-r border-slate-800 font-sans">Description</th>
                <th class="w-10 p-2 text-center"></th>
              </tr>
            </thead>
            <tbody>
              {#each req.headers as header, idx (header.id)}
                <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                  <td class="p-2 text-center">
                    <input
                      type="checkbox"
                      bind:checked={header.enabled}
                      onchange={() => workspaceStore.updateActiveRequest({ headers: req?.headers })}
                      class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={header.key}
                      placeholder="e.g. Content-Type"
                      oninput={() => {
                        if (header.key.endsWith('~')) {
                          header.key = header.key.slice(0, -1);
                          header.enabled = false;
                        } else if (header.key.startsWith('~')) {
                          header.key = header.key.slice(1);
                          header.enabled = false;
                        }
                        workspaceStore.updateActiveRequest({ headers: req?.headers });
                      }}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={header.value}
                      placeholder="e.g. application/json"
                      oninput={() => workspaceStore.updateActiveRequest({ headers: req?.headers })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800 font-sans">
                    <input
                      type="text"
                      bind:value={header.description}
                      placeholder="Description"
                      oninput={() => workspaceStore.updateActiveRequest({ headers: req?.headers })}
                      class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
                    />
                  </td>
                  <td class="p-2 text-center">
                    <button
                      type="button"
                      onclick={() => removeHeader(idx)}
                      class="text-slate-500 hover:text-rose-400"
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              {/each}
              {#if req.headers.length === 0}
                <tr>
                  <td colspan="5" class="p-4 text-center text-slate-500 font-sans">
                    No headers defined. Click "+ Add Header" to add custom request headers.
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>

      <!-- VARIABLES TAB -->
      {:else if activeSubTab === 'variables'}
        <div class="space-y-3 font-sans">
          <div class="flex items-center justify-between">
            <div>
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Request-Level Variables</span>
              <p class="text-[11px] text-slate-500 mt-0.5">
                Variables defined here take precedence for this request. Use <code class="text-amber-400 font-mono">{"{{var_name}}"}</code> in URL, headers, or body.
              </p>
            </div>
            <button
              type="button"
              onclick={addVariable}
              class="text-xs text-amber-400 hover:text-amber-300 font-medium"
            >
              + Add Variable
            </button>
          </div>

          <table class="w-full border-collapse border border-slate-800 text-left font-mono">
            <thead>
              <tr class="bg-slate-900/80 text-slate-400 border-b border-slate-800 text-[11px]">
                <th class="w-8 p-2 text-center"></th>
                <th class="p-2 border-r border-slate-800">Variable Key</th>
                <th class="p-2 border-r border-slate-800">Value</th>
                <th class="w-16 p-2 text-center border-r border-slate-800">Secret</th>
                <th class="p-2 border-r border-slate-800 font-sans">Description</th>
                <th class="w-10 p-2 text-center"></th>
              </tr>
            </thead>
            <tbody>
              {#each (req.variables || []) as v, idx (v.id)}
                <tr class="border-b border-slate-800/80 hover:bg-slate-900/30">
                  <td class="p-2 text-center">
                    <input
                      type="checkbox"
                      bind:checked={v.enabled}
                      onchange={() => workspaceStore.updateActiveRequest({ variables: req?.variables })}
                      class="rounded bg-slate-800 border-slate-700 text-amber-500 focus:ring-0"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type="text"
                      bind:value={v.key}
                      placeholder="e.g. user_id"
                      oninput={() => {
                        if (v.key.endsWith('~')) {
                          v.key = v.key.slice(0, -1);
                          v.enabled = false;
                        } else if (v.key.startsWith('~')) {
                          v.key = v.key.slice(1);
                          v.enabled = false;
                        }
                        workspaceStore.updateActiveRequest({ variables: req?.variables });
                      }}
                      class="w-full bg-transparent px-2 py-1 text-amber-300 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 border-r border-slate-800">
                    <input
                      type={v.isSecret ? 'password' : 'text'}
                      bind:value={v.value}
                      placeholder={varValuePlaceholder}
                      oninput={() => workspaceStore.updateActiveRequest({ variables: req?.variables })}
                      class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                    />
                  </td>
                  <td class="p-1 text-center border-r border-slate-800">
                    <button
                      type="button"
                      onclick={() => {
                        v.isSecret = !v.isSecret;
                        workspaceStore.updateActiveRequest({ variables: req?.variables });
                      }}
                      class="text-xs px-1.5 py-0.5 rounded {v.isSecret ? 'text-amber-400 bg-amber-950/60 border border-amber-800/60' : 'text-slate-500 hover:text-slate-300'}"
                      title={v.isSecret ? 'Secret masked' : 'Plain text'}
                    >
                      {v.isSecret ? '🔒' : '👁'}
                    </button>
                  </td>
                  <td class="p-1 border-r border-slate-800 font-sans">
                    <input
                      type="text"
                      bind:value={v.description}
                      placeholder="Description"
                      oninput={() => workspaceStore.updateActiveRequest({ variables: req?.variables })}
                      class="w-full bg-transparent px-2 py-1 text-slate-400 focus:outline-none"
                    />
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
              {#if (!req.variables || req.variables.length === 0)}
                <tr>
                  <td colspan="6" class="p-4 text-center text-slate-500 font-sans">
                    No request variables defined. Click "+ Add Variable" to scope variables to this request.
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>

          <!-- Dynamic Variables Cheat Sheet -->
          <div class="mt-4 p-3 bg-slate-900/40 border border-slate-800 rounded-lg">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block mb-2">
              Bruno Dynamic Variables (Use anywhere in URLs, Headers, Body, or Variables)
            </span>
            <div class="flex flex-wrap gap-1.5">
              {#each dynamicVars as dv}
                <button
                  type="button"
                  onclick={() => {
                    navigator.clipboard.writeText(`{{${dv.label}}}`);
                  }}
                  title="{dv.desc} (Click to copy)"
                  class="px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 text-sky-400 border border-slate-700/60 text-[10px] font-mono transition-colors flex items-center gap-1"
                >
                  <span>{`{{${dv.label}}}`}</span>
                  <span class="text-slate-500 text-[9px]">{dv.desc}</span>
                </button>
              {/each}
            </div>
          </div>
        </div>

      <!-- AUTH TAB -->
      {:else if activeSubTab === 'auth'}
        <div class="max-w-xl space-y-4 font-sans">
          <div class="flex items-center gap-3">
            <label for="auth-type-select" class="text-xs text-slate-400 font-medium">Auth Type:</label>
            <select
              id="auth-type-select"
              bind:value={req.auth.type}
              onchange={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
              class="bg-slate-800 border border-slate-700 text-slate-200 text-xs rounded px-2.5 py-1.5 focus:outline-none focus:border-sky-500 font-semibold"
            >
              <option value="inherit">Inherit auth from Collection</option>
              <option value="bearer">Bearer Token</option>
              <option value="basic">Basic Auth</option>
              <option value="apikey">API Key</option>
              <option value="none">No Auth</option>
            </select>
          </div>

          {#if req.auth.type === 'bearer'}
            <div class="p-3 bg-slate-900/60 border border-slate-800 rounded-lg space-y-2">
              <label for="bearer-token-input" class="text-xs font-semibold text-slate-300">Bearer Token</label>
              <div class="relative">
                <input
                  id="bearer-token-input"
                  type={showBearerSecret ? 'text' : 'password'}
                  bind:value={req.auth.bearerToken}
                  placeholder={bearerTokenPlaceholder}
                  oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-emerald-400 focus:outline-none focus:border-sky-500 pr-16"
                />
                <button
                  type="button"
                  onclick={() => showBearerSecret = !showBearerSecret}
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-slate-400 hover:text-slate-200 px-1 py-0.5 rounded bg-slate-800"
                >
                  {showBearerSecret ? 'Hide' : 'Show'}
                </button>
              </div>
              <p class="text-[11px] text-slate-500">
                Variables like <code class="text-sky-400 font-mono">{"{{api_bearer_token}}"}</code> are automatically resolved.
              </p>
            </div>

          {:else if req.auth.type === 'basic'}
            <div class="p-3 bg-slate-900/60 border border-slate-800 rounded-lg space-y-3">
              <div class="space-y-1">
                <label for="basic-username-input" class="text-xs font-semibold text-slate-300">Username</label>
                <input
                  id="basic-username-input"
                  type="text"
                  bind:value={req.auth.basicUsername}
                  placeholder="e.g. admin"
                  oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                />
              </div>
              <div class="space-y-1">
                <label for="basic-password-input" class="text-xs font-semibold text-slate-300">Password</label>
                <input
                  id="basic-password-input"
                  type="password"
                  bind:value={req.auth.basicPassword}
                  placeholder={basicPasswordPlaceholder}
                  oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                />
              </div>
            </div>

          {:else if req.auth.type === 'apikey'}
            <div class="p-3 bg-slate-900/60 border border-slate-800 rounded-lg space-y-3">
              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-1">
                  <label for="apikey-name-input" class="text-xs font-semibold text-slate-300">Key Name</label>
                  <input
                    id="apikey-name-input"
                    type="text"
                    bind:value={req.auth.apiKeyName}
                    placeholder="X-API-Key"
                    oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                    class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                  />
                </div>
                <div class="space-y-1">
                  <label for="apikey-value-input" class="text-xs font-semibold text-slate-300">Value</label>
                  <input
                    id="apikey-value-input"
                    type="text"
                    bind:value={req.auth.apiKeyValue}
                    placeholder={apiKeyPlaceholder}
                    oninput={() => workspaceStore.updateActiveRequest({ auth: req?.auth })}
                    class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500"
                  />
                </div>
              </div>
            </div>

          {:else if req.auth.type === 'inherit'}
            <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-lg text-slate-400 text-xs">
              This request inherits authentication credentials configured at the collection or workspace level.
            </div>
          {/if}
        </div>

      <!-- BODY TAB -->
      {:else if activeSubTab === 'body'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <!-- Body Type Selector & Action Toolbar -->
          <div class="flex items-center justify-between shrink-0 flex-wrap gap-2 pb-1 border-b border-slate-800/80">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-xs text-slate-400 font-medium">Body Type:</span>
              <div class="flex items-center gap-1.5 flex-wrap">
                {#each ['none', 'json', 'xml', 'text', 'form-urlencoded', 'multipart', 'binary'] as bType}
                  <button
                    type="button"
                    onclick={() => setBodyType(bType as BodyType)}
                    class="px-2.5 py-1 rounded text-xs font-medium transition-all {req.body.type === bType ? 'bg-sky-600/30 text-sky-300 border border-sky-500/50 shadow-sm shadow-sky-950' : 'bg-slate-900 border border-slate-800 text-slate-400 hover:text-slate-200 hover:border-slate-700'}"
                  >
                    {bType}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Toolbar Actions depending on Body Type -->
            <div class="flex items-center gap-2">
              {#if bodyActionFeedback}
                <span class="text-[11px] px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-800 text-emerald-300 font-mono animate-fade">
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
                      bodyActionFeedback = 'Copied to clipboard';
                      setTimeout(() => bodyActionFeedback = '', 2000);
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
                  title="Clean empty fields"
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
                  title="Clean empty fields"
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
                oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                placeholder={req.body.type === 'json' ? jsonBodyPlaceholder : req.body.type === 'xml' ? xmlBodyPlaceholder : textBodyPlaceholder}
                class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-slate-100 resize-none focus:outline-none leading-relaxed"
                spellcheck="false"
              ></textarea>
            </div>

          <!-- FORM-URLENCODED -->
          {:else if req.body.type === 'form-urlencoded'}
            <div class="space-y-2 flex-1 flex flex-col">
              <div class="flex items-center justify-between text-[11px] text-slate-400">
                <span>Form URL-Encoded Key/Value Pairs (<code class="text-sky-400">application/x-www-form-urlencoded</code>)</span>
                <span class="font-mono text-slate-500">{(req.body.formData || []).filter(f => f.enabled && f.key).length} active fields</span>
              </div>

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
                            onchange={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                            class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                          />
                        </td>
                        <td class="p-1 border-r border-slate-800">
                          <input
                            type="text"
                            bind:value={item.key}
                            placeholder="param_name"
                            oninput={() => {
                              if (item.key.endsWith('~')) {
                                item.key = item.key.slice(0, -1);
                                item.enabled = false;
                              } else if (item.key.startsWith('~')) {
                                item.key = item.key.slice(1);
                                item.enabled = false;
                              }
                              workspaceStore.updateActiveRequest({ body: req?.body });
                            }}
                            class="w-full bg-transparent px-2 py-1 text-sky-300 focus:outline-none"
                          />
                        </td>
                        <td class="p-1 border-r border-slate-800">
                          <input
                            type="text"
                            bind:value={item.value}
                            placeholder={formValuePlaceholder}
                            oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                            class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                          />
                        </td>
                        <td class="p-2 text-center">
                          <button
                            type="button"
                            onclick={() => removeFormData(idx)}
                            class="text-slate-500 hover:text-rose-400 transition-colors"
                            title="Remove field"
                          >
                            ✕
                          </button>
                        </td>
                      </tr>
                    {/each}
                    {#if (!req.body.formData || req.body.formData.length === 0)}
                      <tr>
                        <td colspan="4" class="p-6 text-center text-slate-500 font-sans">
                          No URL-encoded fields defined. Click <button type="button" onclick={() => addFormData('text')} class="text-sky-400 hover:underline">+ Add Field</button> to configure parameters.
                        </td>
                      </tr>
                    {/if}
                  </tbody>
                </table>
              </div>
            </div>

          <!-- MULTIPART FORM DATA (TEXT & FILE UPLOAD) -->
          {:else if req.body.type === 'multipart'}
            <div class="space-y-2 flex-1 flex flex-col">
              <input
                type="file"
                bind:this={multipartFileInput}
                onchange={onMultipartFileInputChange}
                class="hidden"
              />

              <div class="flex items-center justify-between text-[11px] text-slate-400">
                <span>Multipart Form Data (<code class="text-sky-400">multipart/form-data</code>) · Text & File references</span>
                <span class="font-mono text-slate-500">{(req.body.formData || []).filter(f => f.enabled && f.key).length} active parts</span>
              </div>

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
                            onchange={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                            class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
                          />
                        </td>
                        <td class="p-1 border-r border-slate-800">
                          <input
                            type="text"
                            bind:value={item.key}
                            placeholder="field_name"
                            oninput={() => {
                              if (item.key.endsWith('~')) {
                                item.key = item.key.slice(0, -1);
                                item.enabled = false;
                              } else if (item.key.startsWith('~')) {
                                item.key = item.key.slice(1);
                                item.enabled = false;
                              }
                              workspaceStore.updateActiveRequest({ body: req?.body });
                            }}
                            class="w-full bg-transparent px-2 py-1 text-sky-300 focus:outline-none"
                          />
                        </td>
                        <td class="p-1 border-r border-slate-800 text-center font-sans">
                          <button
                            type="button"
                            onclick={() => {
                              item.type = item.type === 'file' ? 'text' : 'file';
                              workspaceStore.updateActiveRequest({ body: req?.body });
                            }}
                            class="px-2 py-0.5 rounded text-[11px] font-semibold transition-colors {item.type === 'file' ? 'bg-purple-950/80 text-purple-300 border border-purple-800/60' : 'bg-slate-800 text-slate-300 border border-slate-700'}"
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
                                  workspaceStore.updateActiveRequest({ body: req?.body });
                                }}
                                placeholder="Relative path e.g. ./fixtures/avatar.png"
                                class="w-full bg-transparent py-1 text-purple-200 focus:outline-none font-mono text-xs"
                              />
                              <button
                                type="button"
                                onclick={() => triggerMultipartFileBrowse(idx)}
                                class="px-2 py-0.5 rounded bg-purple-950/80 hover:bg-purple-900 border border-purple-800/60 text-purple-200 text-[11px] font-sans font-medium whitespace-nowrap flex items-center gap-1 transition-colors cursor-pointer shrink-0"
                                title="Browse local file to anchor relative path"
                              >
                                📁 Browse...
                              </button>
                            </div>
                          {:else}
                            <input
                              type="text"
                              bind:value={item.value}
                              placeholder={multipartValuePlaceholder}
                              oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                              class="w-full bg-transparent px-2 py-1 text-slate-200 focus:outline-none"
                            />
                          {/if}
                        </td>
                        <td class="p-2 text-center">
                          <button
                            type="button"
                            onclick={() => removeFormData(idx)}
                            class="text-slate-500 hover:text-rose-400 transition-colors cursor-pointer"
                            title="Remove field"
                          >
                            ✕
                          </button>
                        </td>
                      </tr>
                    {/each}
                    {#if (!req.body.formData || req.body.formData.length === 0)}
                      <tr>
                        <td colspan="5" class="p-6 text-center text-slate-500 font-sans">
                          No multipart fields defined. Click 
                          <button type="button" onclick={() => addFormData('text')} class="text-sky-400 hover:underline mx-1 cursor-pointer">+ Text Field</button>
                          or 
                          <button type="button" onclick={() => addFormData('file')} class="text-purple-400 hover:underline mx-1 cursor-pointer">📁 + File Upload</button>
                          to configure multipart data.
                        </td>
                      </tr>
                    {/if}
                  </tbody>
                </table>
              </div>
              <p class="text-[11px] text-slate-500 font-sans">
                💡 <span class="font-medium text-slate-400">Relative Path Anchoring:</span> File uploads strictly resolve relative to workspace root (e.g. <code class="text-purple-300 font-mono">./fixtures/avatar.png</code>). Absolute machine paths are prevented from persisting.
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
                    class="text-[11px] text-slate-500 hover:text-rose-400 flex items-center gap-1 transition-colors cursor-pointer"
                    title="Clear selected file"
                  >
                    ✕ Clear File
                  </button>
                {/if}
              </div>

              <!-- Real-world Drag & Drop / File Browser Dropzone -->
              <div
                role="region"
                aria-label="File dropzone"
                ondragover={(e) => { e.preventDefault(); isBinaryDragging = true; }}
                ondragleave={() => { isBinaryDragging = false; }}
                ondrop={handleBinaryDrop}
                class="border-2 border-dashed rounded-lg p-5 transition-all text-center {isBinaryDragging ? 'border-sky-500 bg-sky-500/10' : req.body.binaryFilePath ? 'border-slate-700 bg-slate-950/60' : 'border-slate-800 hover:border-slate-700 bg-slate-950/30'}"
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
                              {formatFileSize(binarySelectedFileInfo.size)}
                            </span>
                          {/if}
                        </div>
                        <p class="text-[11px] text-slate-500 font-mono truncate mt-0.5">
                          {req.body.binaryFilePath}
                        </p>
                      </div>
                    </div>
                    <div class="flex items-center gap-2 shrink-0">
                      <button
                        type="button"
                        onclick={() => binaryFileInput?.click()}
                        class="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium border border-slate-700 transition-colors flex items-center gap-1.5 cursor-pointer"
                      >
                        📁 Change File...
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="space-y-2 py-2">
                    <div class="text-3xl text-slate-600">📁</div>
                    <div>
                      <button
                        type="button"
                        onclick={() => binaryFileInput?.click()}
                        class="px-3 py-1.5 rounded bg-sky-600 hover:bg-sky-500 text-xs font-semibold text-white shadow-sm transition-colors cursor-pointer inline-flex items-center gap-1.5"
                      >
                        📁 Browse File to Upload
                      </button>
                    </div>
                    <p class="text-[11px] text-slate-500">
                      or drag & drop a file here to anchor relative to workspace root
                    </p>
                  </div>
                {/if}
              </div>

              <!-- Relative File Path Input -->
              <div class="space-y-2">
                <label for="binary-file-path-input" class="text-xs font-medium text-slate-300 flex items-center justify-between">
                  <span>Relative File Path</span>
                  {#if req.body.binaryFilePath && (req.body.binaryFilePath.startsWith('C:') || req.body.binaryFilePath.startsWith('/') || req.body.binaryFilePath.startsWith('\\'))}
                    <span class="text-[10px] text-amber-400 font-normal">⚠️ Should be relative to workspace root (e.g. ./fixtures/file)</span>
                  {/if}
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="binary-file-path-input"
                    type="text"
                    bind:value={req.body.binaryFilePath}
                    placeholder="e.g. ./fixtures/data.bin or ./sample.pdf"
                    oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                    class="flex-1 bg-slate-950 border border-slate-700 rounded px-3 py-1.5 font-mono text-xs text-sky-300 focus:outline-none focus:border-sky-500"
                  />
                  <button
                    type="button"
                    onclick={() => binaryFileInput?.click()}
                    class="px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-xs text-slate-300 font-medium whitespace-nowrap border border-slate-700 flex items-center gap-1.5 transition-colors cursor-pointer"
                  >
                    📁 Browse...
                  </button>
                </div>
              </div>

              <div class="space-y-2">
                <label for="binary-content-type-input" class="text-xs font-medium text-slate-300">Content-Type Header</label>
                <input
                  id="binary-content-type-input"
                  type="text"
                  bind:value={req.body.binaryContentType}
                  placeholder="application/octet-stream"
                  oninput={() => workspaceStore.updateActiveRequest({ body: req?.body })}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 font-mono text-xs text-slate-200 focus:outline-none focus:border-sky-500"
                />
                <div class="flex items-center gap-1.5 flex-wrap pt-1">
                  <span class="text-[10px] text-slate-500">Presets:</span>
                  {#each ['application/octet-stream', 'application/pdf', 'image/png', 'application/zip', 'audio/mpeg'] as mime}
                    <button
                      type="button"
                      onclick={() => {
                        if (!req) return;
                        req.body.binaryContentType = mime;
                        workspaceStore.updateActiveRequest({ body: req.body });
                      }}
                      class="px-2 py-0.5 rounded bg-slate-800/80 hover:bg-slate-700 text-[10px] font-mono text-slate-400 hover:text-sky-300 border border-slate-700/60 transition-colors cursor-pointer"
                    >
                      {mime}
                    </button>
                  {/each}
                </div>
              </div>
            </div>

          <!-- NONE -->
          {:else}
            <div class="p-8 text-center text-slate-500 border border-dashed border-slate-800 rounded-lg font-sans space-y-2">
              <span class="text-2xl block text-slate-600">∅</span>
              <p class="text-xs text-slate-400 font-medium">This request does not send a body payload.</p>
              <p class="text-[11px] text-slate-500">
                Choose <button type="button" onclick={() => setBodyType('json')} class="text-sky-400 hover:underline">JSON</button>,
                <button type="button" onclick={() => setBodyType('xml')} class="text-sky-400 hover:underline">XML</button>,
                <button type="button" onclick={() => setBodyType('form-urlencoded')} class="text-sky-400 hover:underline">Form</button>,
                <button type="button" onclick={() => setBodyType('multipart')} class="text-sky-400 hover:underline">Multipart</button>,
                or <button type="button" onclick={() => setBodyType('binary')} class="text-sky-400 hover:underline">Binary</button> above to attach a payload.
              </p>
            </div>
          {/if}
        </div>

      <!-- SCRIPTS TAB (PRE-REQUEST & POST-RESPONSE) -->
      {:else if activeSubTab === 'scripts'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <div class="flex items-center justify-between shrink-0">
            <div class="flex items-center gap-1.5 bg-slate-900 border border-slate-800 p-0.5 rounded-lg">
              <button
                type="button"
                onclick={() => activeScriptType = 'preRequest'}
                class="px-3 py-1 rounded text-xs font-semibold transition-colors {activeScriptType === 'preRequest' ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200'}"
              >
                Pre-request Script
              </button>
              <button
                type="button"
                onclick={() => activeScriptType = 'postResponse'}
                class="px-3 py-1 rounded text-xs font-semibold transition-colors {activeScriptType === 'postResponse' ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200'}"
              >
                Post-response Script
              </button>
            </div>

            <!-- Snippets Quick Bar -->
            <div class="flex items-center gap-1.5 flex-wrap">
              <span class="text-[10px] text-slate-500">Insert Snippet:</span>
              {#if activeScriptType === 'preRequest'}
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("const email = faker.internet.email();\nyarac.setVar('user_email', email);")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Faker Email
                </button>
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("const name = faker.person.fullName();\nyarac.setVar('user_name', name);")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Faker Name
                </button>
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("req.headers['X-Timestamp'] = Date.now().toString();")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Header Timestamp
                </button>
              {:else}
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("const data = res.json();\nif (data?.token) {\n  yarac.setVar('auth_token', data.token);\n}")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Extract Token
                </button>
                <button
                  type="button"
                  onclick={() => insertScriptSnippet("console.log('Response Status:', res.status);")}
                  class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-indigo-300"
                >
                  + Log Status
                </button>
              {/if}
            </div>
          </div>

          <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
            <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
              <span class="font-mono text-indigo-400">
                {activeScriptType === 'preRequest' ? '// Runs before HTTP request is dispatched' : '// Runs immediately after HTTP response is received'}
              </span>
              <span class="text-slate-500 font-mono text-[10px]">Isolated Deno Core Runtime (5000ms max)</span>
            </div>

            {#if activeScriptType === 'preRequest'}
              <textarea
                value={req.scripts?.preRequest || ''}
                oninput={(e) => {
                  if (!req) return;
                  if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
                  req.scripts.preRequest = (e.target as HTMLTextAreaElement).value;
                  workspaceStore.updateActiveRequest({ scripts: req.scripts });
                }}
                placeholder={preScriptPlaceholder}
                class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-indigo-100 resize-none focus:outline-none"
              ></textarea>
            {:else}
              <textarea
                value={req.scripts?.postResponse || ''}
                oninput={(e) => {
                  if (!req) return;
                  if (!req.scripts) req.scripts = { preRequest: '', postResponse: '' };
                  req.scripts.postResponse = (e.target as HTMLTextAreaElement).value;
                  workspaceStore.updateActiveRequest({ scripts: req.scripts });
                }}
                placeholder={postScriptPlaceholder}
                class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-indigo-100 resize-none focus:outline-none"
              ></textarea>
            {/if}
          </div>
        </div>

      <!-- ASSERT TAB -->
      {:else if activeSubTab === 'assert'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <div class="flex items-center justify-between shrink-0">
            <div>
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">JavaScript Assertions</span>
              <p class="text-[11px] text-slate-500">
                Lightweight assertion statements evaluated directly on the response. Test results display in the Response tab.
              </p>
            </div>

            <!-- Snippets -->
            <div class="flex items-center gap-1.5 flex-wrap">
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.status).toBe(200);")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Status 200
              </button>
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.time).toBeLessThan(1000);")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Latency &lt; 1000ms
              </button>
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.headers['content-type']).toContain('application/json');")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Content-Type JSON
              </button>
              <button
                type="button"
                onclick={() => insertAssertSnippet("expect(res.json().id).toBeDefined();")}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] font-mono text-teal-300"
              >
                + Body ID Defined
              </button>
            </div>
          </div>

          <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
            <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
              <span class="font-mono text-teal-400">// Available matchers: toBe, toEqual, toBeDefined, toBeNull, toBeGreaterThan, toContain</span>
              <span class="text-slate-500 font-mono text-[10px]">Response results shown under "Test Results"</span>
            </div>
            <textarea
              value={req.assert || ''}
              oninput={(e) => {
                if (!req) return;
                req.assert = (e.target as HTMLTextAreaElement).value;
                workspaceStore.updateActiveRequest({ assert: req.assert });
              }}
              placeholder={assertPlaceholder}
              class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-teal-100 resize-none focus:outline-none"
            ></textarea>
          </div>
        </div>

      <!-- TESTS TAB (JASMINE / BRUNO BDD) -->
      {:else if activeSubTab === 'tests'}
        <div class="h-full flex flex-col space-y-3 font-sans">
          <div class="flex items-center justify-between shrink-0">
            <div>
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Jasmine / Bruno BDD Test Suite</span>
              <p class="text-[11px] text-slate-500">
                Structure tests with <code class="text-pink-400 font-mono">describe()</code> and <code class="text-pink-400 font-mono">it()</code> blocks. Full Faker library available globally.
              </p>
            </div>

            <div class="flex items-center gap-1.5">
              <button
                type="button"
                onclick={() => insertTestSnippet(`describe('Response Contract', () => {
  it('should return valid status and body', () => {
    expect(res.status).toBe(200);
    const body = res.json();
    expect(body).toBeDefined();
  });
});`)}
                class="px-2.5 py-1 rounded bg-pink-950/80 hover:bg-pink-900 text-pink-300 border border-pink-800 text-[11px] font-mono"
              >
                + BDD Template
              </button>
            </div>
          </div>

          <div class="flex-1 flex flex-col border border-slate-800 rounded-lg overflow-hidden bg-slate-950 min-h-[220px]">
            <div class="bg-slate-900/60 px-3 py-1.5 border-b border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
              <span class="font-mono text-pink-400">// globals: describe, it, test, expect, res, req, faker</span>
              <span class="text-slate-500 font-mono text-[10px]">Jasmine / Chai assertion suite</span>
            </div>
            <textarea
              value={req.tests || ''}
              oninput={(e) => {
                if (!req) return;
                req.tests = (e.target as HTMLTextAreaElement).value;
                workspaceStore.updateActiveRequest({ tests: req.tests });
              }}
              placeholder={testsPlaceholder}
              class="flex-1 w-full bg-slate-950 p-3 font-mono text-xs text-pink-100 resize-none focus:outline-none"
            ></textarea>
          </div>
        </div>

      <!-- SETTINGS TAB -->
      {:else if activeSubTab === 'settings'}
        <div class="max-w-2xl space-y-4 font-sans">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block">
            Request Execution Settings
          </span>

          <div class="p-4 bg-slate-900/60 border border-slate-800 rounded-lg space-y-4">
            <!-- Follow Redirects -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-follow-redirect" class="text-xs font-semibold text-slate-200 block cursor-pointer">Follow Redirects</label>
                <span class="text-[11px] text-slate-500">Automatically follow HTTP 3xx redirection responses</span>
              </div>
              <input
                id="set-follow-redirect"
                type="checkbox"
                checked={req.settings?.followRedirects ?? true}
                onchange={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.followRedirects = (e.target as HTMLInputElement).checked;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
              />
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Max Redirects -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-max-redirects" class="text-xs font-semibold text-slate-200 block">Max Redirects</label>
                <span class="text-[11px] text-slate-500">Maximum number of consecutive redirects before aborting</span>
              </div>
              <input
                id="set-max-redirects"
                type="number"
                min="0"
                max="50"
                value={req.settings?.maxRedirects ?? 5}
                oninput={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.maxRedirects = parseInt((e.target as HTMLInputElement).value || '5', 10);
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-24 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500 text-right"
              />
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Timeout (ms) -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-timeout" class="text-xs font-semibold text-slate-200 block">Timeout (ms)</label>
                <span class="text-[11px] text-slate-500">Maximum execution time before request cancels. <strong>0</strong> = indefinite (no timeout)</span>
              </div>
              <div class="flex items-center gap-2">
                <input
                  id="set-timeout"
                  type="number"
                  min="0"
                  step="500"
                  value={req.settings?.timeoutMs ?? 0}
                  oninput={(e) => {
                    if (!req) return;
                    if (!req.settings) {
                      req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                    }
                    req.settings.timeoutMs = parseInt((e.target as HTMLInputElement).value || '0', 10);
                    workspaceStore.updateActiveRequest({ settings: req.settings });
                  }}
                  class="w-28 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500 text-right"
                />
                <span class="text-[11px] text-slate-400 font-mono">ms</span>
              </div>
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Verify SSL -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-verify-ssl" class="text-xs font-semibold text-slate-200 block cursor-pointer">Verify SSL Certificates</label>
                <span class="text-[11px] text-slate-500">Enforce valid TLS / HTTPS certificates for remote server</span>
              </div>
              <input
                id="set-verify-ssl"
                type="checkbox"
                checked={req.settings?.verifySsl ?? true}
                onchange={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.verifySsl = (e.target as HTMLInputElement).checked;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
              />
            </div>

            <div class="border-t border-slate-800/80"></div>

            <!-- Inherit Auth -->
            <div class="flex items-center justify-between">
              <div>
                <label for="set-inherit-auth" class="text-xs font-semibold text-slate-200 block cursor-pointer">Inherit Collection Auth</label>
                <span class="text-[11px] text-slate-500">Automatically attach parent collection authentication credentials</span>
              </div>
              <input
                id="set-inherit-auth"
                type="checkbox"
                checked={req.settings?.inheritAuth ?? (req.auth.type === 'inherit')}
                onchange={(e) => {
                  if (!req) return;
                  if (!req.settings) {
                    req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
                  }
                  req.settings.inheritAuth = (e.target as HTMLInputElement).checked;
                  workspaceStore.updateActiveRequest({ settings: req.settings });
                }}
                class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0"
              />
            </div>
          </div>
        </div>

      <!-- FLOW TAB -->
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
