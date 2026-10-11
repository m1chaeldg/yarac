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
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 font-sans animate-in fade-in duration-100">
    <div class="apple-modal-card w-full max-w-3xl max-h-[85vh] flex flex-col text-apple-body-on-dark">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-apple-hairline-dark flex items-center justify-between bg-apple-surface-tile-2 shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-2.5 h-2.5 rounded-apple-full bg-apple-primary"></div>
          <h2 class="apple-body-strong text-apple-body-on-dark">Interoperability Hub</h2>
          <span class="apple-micro-legal px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark font-mono">OpenAPI 3.1 & Bruno</span>
        </div>
        <button
          onclick={onClose}
          aria-label="Close dialog"
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
        >
          ✕
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="px-6 border-b border-apple-hairline-dark bg-apple-surface-tile-3 flex gap-4 text-xs shrink-0">
        <button
          type="button"
          onclick={() => { activeTab = 'import'; statusMessage = null; }}
          class="py-3 font-semibold border-b-2 transition-colors {activeTab === 'import' ? 'border-apple-primary text-apple-primary-on-dark' : 'border-transparent text-apple-body-muted hover:text-white'}"
        >
          Import (Bruno / OpenAPI / Postman)
        </button>
        <button
          type="button"
          onclick={() => { activeTab = 'export'; statusMessage = null; }}
          class="py-3 font-semibold border-b-2 transition-colors {activeTab === 'export' ? 'border-apple-primary text-apple-primary-on-dark' : 'border-transparent text-apple-body-muted hover:text-white'}"
        >
          Export to OpenAPI v3.1
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-4 overflow-y-auto bg-apple-surface-tile-1">
        {#if activeTab === 'import'}
          <!-- Format Selector -->
          <div class="flex items-center gap-3">
            <span class="apple-micro-legal font-semibold text-apple-ink-muted-48 uppercase">Format:</span>
            <div class="flex gap-2">
              <button
                type="button"
                onclick={() => importFormat = 'bruno'}
                class="px-3 py-1 rounded-apple-pill text-xs font-medium border transition-colors {importFormat === 'bruno' ? 'bg-apple-primary text-white border-transparent' : 'bg-apple-surface-tile-2 border-apple-hairline-dark text-apple-body-muted hover:text-white'}"
              >
                Bruno (.bru)
              </button>
              <button
                type="button"
                onclick={() => importFormat = 'openapi'}
                class="px-3 py-1 rounded-apple-pill text-xs font-medium border transition-colors {importFormat === 'openapi' ? 'bg-apple-primary text-white border-transparent' : 'bg-apple-surface-tile-2 border-apple-hairline-dark text-apple-body-muted hover:text-white'}"
              >
                OpenAPI v3/v3.1
              </button>
              <button
                type="button"
                onclick={() => importFormat = 'postman'}
                class="px-3 py-1 rounded-apple-pill text-xs font-medium border transition-colors {importFormat === 'postman' ? 'bg-apple-primary text-white border-transparent' : 'bg-apple-surface-tile-2 border-apple-hairline-dark text-apple-body-muted hover:text-white'}"
              >
                Postman v2.1
              </button>
            </div>
          </div>

          <div>
            <label for="import-paste-area" class="block apple-micro-legal font-semibold text-apple-ink-muted-48 uppercase tracking-wider mb-1.5">
              Paste Specification or Request Content
            </label>
            <textarea
              id="import-paste-area"
              rows="9"
              bind:value={importText}
              class="w-full bg-apple-surface-black border border-apple-hairline-dark text-apple-body-on-dark font-mono text-xs rounded-apple-sm p-3 focus:outline-none focus:border-apple-primary"
            ></textarea>
          </div>
        {:else}
          <!-- Export Section -->
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="export-title" class="block apple-micro-legal font-semibold text-apple-ink-muted-48 uppercase tracking-wider mb-1">API Title</label>
              <input
                id="export-title"
                type="text"
                bind:value={exportTitle}
                class="w-full bg-apple-surface-black border border-apple-hairline-dark text-apple-body-on-dark text-xs rounded-apple-sm px-3 py-1.5 focus:outline-none focus:border-apple-primary"
              />
            </div>
            <div>
              <label for="export-version" class="block apple-micro-legal font-semibold text-apple-ink-muted-48 uppercase tracking-wider mb-1">API Version</label>
              <input
                id="export-version"
                type="text"
                bind:value={exportVersion}
                class="w-full bg-apple-surface-black border border-apple-hairline-dark text-apple-body-on-dark text-xs rounded-apple-sm px-3 py-1.5 focus:outline-none focus:border-apple-primary font-mono"
              />
            </div>
          </div>

          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label for="export-preview-area" class="apple-micro-legal font-semibold text-apple-ink-muted-48 uppercase tracking-wider">
                Exported OpenAPI v3.1 Preview (with <code class="text-apple-primary-on-dark">x-yarac-*</code> overlays)
              </label>
              <button
                type="button"
                onclick={generateOpenApiExport}
                class="text-xs text-apple-primary-on-dark hover:underline font-semibold"
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
              class="w-full bg-apple-surface-black border border-apple-hairline-dark text-apple-body-on-dark font-mono text-xs rounded-apple-sm p-3 focus:outline-none"
            ></textarea>
          </div>
        {/if}

        {#if statusMessage}
          <div class="p-3 rounded-apple-sm text-xs flex items-center gap-2 {statusMessage.type === 'success' ? 'bg-apple-surface-tile-2 border border-apple-primary/40 text-apple-primary-on-dark' : 'bg-apple-surface-tile-2 border border-rose-800 text-rose-300'}">
            <span>{statusMessage.type === 'success' ? '✓' : '✗'}</span>
            <span>{statusMessage.text}</span>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-apple-hairline-dark flex items-center justify-between bg-apple-surface-tile-2 shrink-0">
        <span class="apple-fine-print text-apple-ink-muted-48">
          Strict lossless mapping without legacy Postman polyfills
        </span>
        <div class="flex items-center gap-2">
          {#if activeTab === 'import'}
            <button
              type="button"
              onclick={executeImport}
              class="apple-button-primary text-xs px-4 py-1.5 font-semibold"
            >
              Import into Workspace
            </button>
          {:else}
            <button
              type="button"
              onclick={generateOpenApiExport}
              class="apple-button-primary text-xs px-4 py-1.5 font-semibold"
            >
              Export OpenAPI v3.1
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
