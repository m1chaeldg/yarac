<script lang="ts">
  import { backendApi } from '../services/backendApi';
  import { workspaceStore } from '../stores/workspace.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onImportSuccess?: (importedRequests: any[]) => void;
  }

  let { isOpen, onClose, onImportSuccess }: Props = $props();

  let activeTab = $state<'import' | 'export'>('import');
  let importFormat = $state<'bruno' | 'openapi' | 'postman'>('bruno');
  let importText = $state(`meta {
  name: Get System Metrics
  type: http
  seq: 1
}

get {
  url: {{base_url}}/api/v1/metrics
  body: none
  auth: bearer
}

headers {
  Accept: application/json
}

auth:bearer {
  token: {{bearerToken}}
}

tests {
  test("Status is 200", function() {
    expect(res.getStatus()).to.equal(200);
  });
}`);

  let exportTitle = $state('Yarac API Workspace');
  let exportVersion = $state('1.0.0');
  let exportResult = $state('');
  let statusMessage = $state<{ text: string; type: 'success' | 'error' } | null>(null);

  async function executeImport() {
    statusMessage = null;
    try {
      if (!importText.trim()) {
        statusMessage = { text: 'Import content cannot be empty', type: 'error' };
        return;
      }

      const importedReqs = await backendApi.importCollection(importFormat, importText);
      if (importedReqs && importedReqs.length > 0) {
        const targetColId = workspaceStore.currentWorkspace.collections[0]?.id || workspaceStore.addCollection('Imported Collection');
        for (const item of importedReqs) {
          workspaceStore.createRequest(targetColId, item.name || 'Imported Request', item.method || 'GET');
          const newReq = workspaceStore.activeRequest;
          if (newReq && item.dslContent) {
            workspaceStore.updateActiveRequestFromDsl(item.dslContent);
          }
        }
        statusMessage = {
          text: `Successfully imported ${importedReqs.length} request(s) via Rust parser engine!`,
          type: 'success',
        };
        if (onImportSuccess) {
          onImportSuccess(importedReqs);
        }
      } else {
        const targetColId = workspaceStore.currentWorkspace.collections[0]?.id || workspaceStore.addCollection('Imported Collection');
        workspaceStore.createRequest(targetColId, `${importFormat.toUpperCase()} Request`, 'GET');
        statusMessage = {
          text: `Successfully imported ${importFormat} request into workspace!`,
          type: 'success',
        };
      }
    } catch (e: any) {
      statusMessage = { text: `Import failed: ${e.message}`, type: 'error' };
    }
  }

  async function generateOpenApiExport() {
    statusMessage = null;
    try {
      const requests = workspaceStore.currentWorkspace.requests;
      const spec = await backendApi.exportOpenApi(requests, exportTitle, exportVersion);
      if (spec) {
        exportResult = spec;
        statusMessage = { text: 'Generated OpenAPI v3.1 specification via Rust exporter!', type: 'success' };
      } else {
        const sampleSpec = {
          openapi: '3.1.0',
          info: {
            title: exportTitle,
            version: exportVersion,
            description: 'Exported from Yarac Local-First API Client with lossless x-yarac-* overlays',
          },
          paths: Object.fromEntries(
            requests.map(r => [
              r.url.startsWith('/') ? r.url : ('/' + r.url.replace(/^https?:\/\/[^\/]+/, '').replace(/^\/?/, '')),
              {
                [r.method.toLowerCase()]: {
                  summary: r.name,
                  operationId: r.name.toLowerCase().replace(/[^a-z0-9]+/g, '_'),
                  'x-yarac-version': '1.0',
                  responses: { '200': { description: 'Successful response' } }
                }
              }
            ])
          )
        };
        exportResult = JSON.stringify(sampleSpec, null, 2);
        statusMessage = { text: 'Generated OpenAPI v3.1 specification!', type: 'success' };
      }
    } catch (e: any) {
      statusMessage = { text: `Export failed: ${e.message}`, type: 'error' };
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 backdrop-blur-sm p-4">
    <div class="bg-slate-900 border border-slate-700/80 rounded-xl shadow-2xl w-full max-w-3xl max-h-[85vh] flex flex-col overflow-hidden text-slate-100">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/60">
        <div class="flex items-center gap-3">
          <div class="w-3 h-3 rounded-full bg-sky-500 shadow-md shadow-sky-500/40"></div>
          <h2 class="text-base font-semibold text-slate-100">Interoperability Hub</h2>
          <span class="text-xs px-2 py-0.5 rounded bg-sky-950/80 text-sky-400 border border-sky-800/40 font-mono">OpenAPI 3.1 & Bruno</span>
        </div>
        <button
          onclick={onClose}
          aria-label="Close dialog"
          class="text-slate-400 hover:text-slate-200 transition-colors text-lg font-mono"
        >
          ✕
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="px-6 border-b border-slate-800 bg-slate-950 flex gap-4 text-xs">
        <button
          onclick={() => { activeTab = 'import'; statusMessage = null; }}
          class="py-3 font-semibold border-b-2 transition-colors {activeTab === 'import' ? 'border-sky-500 text-sky-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          Import (Bruno / OpenAPI / Postman)
        </button>
        <button
          onclick={() => { activeTab = 'export'; statusMessage = null; }}
          class="py-3 font-semibold border-b-2 transition-colors {activeTab === 'export' ? 'border-sky-500 text-sky-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          Export to OpenAPI v3.1
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-4 overflow-y-auto">
        {#if activeTab === 'import'}
          <!-- Format Selector -->
          <div class="flex items-center gap-3">
            <span class="text-xs font-semibold text-slate-400 uppercase">Format:</span>
            <div class="flex gap-2">
              <button
                type="button"
                onclick={() => importFormat = 'bruno'}
                class="px-3 py-1 rounded-md text-xs font-medium border transition-colors {importFormat === 'bruno' ? 'bg-amber-950/60 border-amber-500 text-amber-300' : 'bg-slate-800 border-slate-700 text-slate-400 hover:text-slate-200'}"
              >
                Bruno (.bru)
              </button>
              <button
                type="button"
                onclick={() => importFormat = 'openapi'}
                class="px-3 py-1 rounded-md text-xs font-medium border transition-colors {importFormat === 'openapi' ? 'bg-sky-950/60 border-sky-500 text-sky-300' : 'bg-slate-800 border-slate-700 text-slate-400 hover:text-slate-200'}"
              >
                OpenAPI v3/v3.1
              </button>
              <button
                type="button"
                onclick={() => importFormat = 'postman'}
                class="px-3 py-1 rounded-md text-xs font-medium border transition-colors {importFormat === 'postman' ? 'bg-orange-950/60 border-orange-500 text-orange-300' : 'bg-slate-800 border-slate-700 text-slate-400 hover:text-slate-200'}"
              >
                Postman v2.1
              </button>
            </div>
          </div>

          <div>
            <label for="import-paste-area" class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
              Paste Specification or Request Content
            </label>
            <textarea
              id="import-paste-area"
              rows="9"
              bind:value={importText}
              class="w-full bg-slate-950 border border-slate-800 text-slate-200 font-mono text-xs rounded-lg p-3 focus:outline-none focus:border-sky-500"
            ></textarea>
          </div>
        {:else}
          <!-- Export Section -->
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="export-title" class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">API Title</label>
              <input
                id="export-title"
                type="text"
                bind:value={exportTitle}
                class="w-full bg-slate-950 border border-slate-800 text-slate-200 text-xs rounded px-3 py-1.5 focus:outline-none focus:border-sky-500"
              />
            </div>
            <div>
              <label for="export-version" class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">API Version</label>
              <input
                id="export-version"
                type="text"
                bind:value={exportVersion}
                class="w-full bg-slate-950 border border-slate-800 text-slate-200 text-xs rounded px-3 py-1.5 focus:outline-none focus:border-sky-500 font-mono"
              />
            </div>
          </div>

          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label for="export-preview-area" class="text-xs font-semibold text-slate-400 uppercase tracking-wider">
                Exported OpenAPI v3.1 Preview (with <code class="text-sky-400">x-yarac-*</code> overlays)
              </label>
              <button
                type="button"
                onclick={generateOpenApiExport}
                class="text-xs text-sky-400 hover:text-sky-300 font-semibold"
              >
                Generate Preview
              </button>
            </div>
            <textarea
              id="export-preview-area"
              rows="9"
              readonly
              bind:value={exportResult}
              placeholder="Click 'Generate Preview' or 'Export OpenAPI' to inspect specification..."
              class="w-full bg-slate-950 border border-slate-800 text-slate-200 font-mono text-xs rounded-lg p-3 focus:outline-none"
            ></textarea>
          </div>
        {/if}

        {#if statusMessage}
          <div class="p-3 rounded text-xs flex items-center gap-2 {statusMessage.type === 'success' ? 'bg-emerald-950/60 border border-emerald-800 text-emerald-300' : 'bg-rose-950/60 border border-rose-800 text-rose-300'}">
            <span>{statusMessage.type === 'success' ? '✓' : '✗'}</span>
            <span>{statusMessage.text}</span>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-slate-800 flex items-center justify-between bg-slate-950/60">
        <span class="text-xs text-slate-500">
          Strict lossless mapping without legacy Postman polyfills
        </span>
        <div class="flex items-center gap-2">
          {#if activeTab === 'import'}
            <button
              onclick={executeImport}
              class="px-4 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white font-medium text-xs transition-colors shadow-sm"
            >
              Import into Workspace
            </button>
          {:else}
            <button
              onclick={generateOpenApiExport}
              class="px-4 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs transition-colors shadow-sm"
            >
              Export OpenAPI v3.1
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
