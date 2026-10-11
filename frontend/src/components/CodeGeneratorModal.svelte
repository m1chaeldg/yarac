<script lang="ts">
  import { CODE_LANGUAGES, type CodeLanguage } from '../utils/codeGenerators';
  import type { YaracRequest } from '../stores/workspace.svelte';
  import { workspaceStore } from '../stores/workspace.svelte';

  interface Props {
    isOpen: boolean;
    request: YaracRequest | null;
    onClose: () => void;
  }

  let { isOpen, request, onClose }: Props = $props();

  let selectedLangId = $state<string>('curl');
  let selectedVariantId = $state<string>('bash');
  let interpolateEnv = $state<boolean>(true);
  let isCopied = $state<boolean>(false);

  let currentLang = $derived(
    CODE_LANGUAGES.find(l => l.id === selectedLangId) || CODE_LANGUAGES[0]
  );

  // Sync variant when language changes
  $effect(() => {
    if (currentLang && !currentLang.variants.some(v => v.id === selectedVariantId)) {
      selectedVariantId = currentLang.variants[0]?.id || '';
    }
  });

  let envMap = $derived.by(() => {
    if (!interpolateEnv) return undefined;
    const activeEnv = workspaceStore.activeEnvironment;
    if (!activeEnv) return undefined;
    const map: Record<string, string> = {};
    for (const v of activeEnv.variables) {
      if (v.enabled && v.key.trim()) {
        map[v.key] = v.value;
      }
    }
    return map;
  });

  let generatedCode = $derived.by(() => {
    if (!request || !currentLang) return '// No request selected';
    return currentLang.generate(request, selectedVariantId, envMap);
  });

  async function copyToClipboard() {
    if (!generatedCode) return;
    try {
      await navigator.clipboard.writeText(generatedCode);
      isCopied = true;
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen && request}
<div
    class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 animate-in fade-in duration-150"
    onclick={onClose}
    role="presentation"
  >
    <div
      class="apple-modal-card w-full max-w-4xl h-[78vh] flex flex-col overflow-hidden text-apple-body-on-dark font-sans"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="code-generator-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-5 py-3.5 border-b border-apple-hairline-dark flex items-center justify-between shrink-0 bg-apple-surface-tile-2">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-apple-sm bg-apple-surface-tile-3 text-apple-primary-on-dark flex items-center justify-center font-bold text-sm border border-apple-hairline-dark">
            &lt;/&gt;
          </div>
          <div>
            <h2 id="code-generator-title" class="apple-body-strong text-apple-body-on-dark flex items-center gap-2">
              Generate Code Snippet
              <span class="apple-micro-legal font-mono font-normal text-apple-body-muted px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 border border-apple-hairline-dark">
                {request.method} {request.name}
              </span>
            </h2>
            <p class="apple-fine-print text-apple-body-muted">
              Ready-to-use client implementations across major languages and HTTP frameworks.
            </p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
          title="Close dialog"
        >
          ✕
        </button>
      </div>

      <!-- Main Body -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Sidebar: Language Selection -->
        <div class="w-48 border-r border-apple-hairline-dark bg-apple-surface-tile-3 p-2 overflow-y-auto space-y-1 shrink-0 select-none">
          <div class="px-2 py-1 apple-micro-legal font-bold uppercase tracking-wider text-apple-ink-muted-48">
            Languages ({CODE_LANGUAGES.length})
          </div>
          {#each CODE_LANGUAGES as lang (lang.id)}
            <button
              type="button"
              onclick={() => {
                selectedLangId = lang.id;
                selectedVariantId = lang.variants[0]?.id || '';
              }}
              class="w-full text-left px-2.5 py-1.5 rounded-apple-sm text-xs font-medium flex items-center gap-2 transition-colors
                {selectedLangId === lang.id
                  ? 'bg-apple-surface-tile-1 text-apple-primary-on-dark font-semibold border border-apple-primary/40'
                  : 'text-apple-body-muted hover:bg-apple-surface-tile-2 hover:text-white'}"
            >
              <span>{lang.icon}</span>
              <span class="truncate">{lang.name}</span>
            </button>
          {/each}
        </div>

        <!-- Center Code Display -->
        <div class="flex-1 flex flex-col overflow-hidden bg-apple-surface-black">
          <!-- Options bar -->
          <div class="px-4 py-2 border-b border-apple-hairline-dark bg-apple-surface-tile-2 flex items-center justify-between gap-3 shrink-0">
            <!-- Variant Selector -->
            <div class="flex items-center gap-2">
              <span class="apple-micro-legal text-apple-body-muted font-medium">Library:</span>
              <div class="flex items-center gap-1 bg-apple-surface-tile-3 border border-apple-hairline-dark rounded-apple-pill p-0.5">
                {#each currentLang.variants as variant (variant.id)}
                  <button
                    type="button"
                    onclick={() => selectedVariantId = variant.id}
                    class="px-2.5 py-0.5 rounded-apple-pill text-xs transition-colors
                      {selectedVariantId === variant.id
                        ? 'bg-apple-primary text-white font-semibold'
                        : 'text-apple-body-muted hover:text-white'}"
                  >
                    {variant.name}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Options & Copy Button -->
            <div class="flex items-center gap-3">
              <label class="flex items-center gap-1.5 text-xs text-apple-body-muted cursor-pointer select-none">
                <input
                  type="checkbox"
                  bind:checked={interpolateEnv}
                  class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark text-apple-primary focus:ring-0"
                />
                <span class="apple-micro-legal text-apple-body-muted">Resolve Environment Vars</span>
              </label>

              <button
                type="button"
                onclick={copyToClipboard}
                class="apple-button-primary text-xs px-3.5 py-1"
              >
                {#if isCopied}
                  <span>✓</span>
                  <span>Copied!</span>
                {:else}
                  <span>📋</span>
                  <span>Copy Code</span>
                {/if}
              </button>
            </div>
          </div>

          <!-- Code View Area -->
          <div class="flex-1 overflow-auto p-4 font-mono text-xs bg-apple-surface-black text-apple-body-on-dark leading-relaxed selection:bg-apple-primary/40 selection:text-white">
            <pre class="m-0 font-mono"><code>{generatedCode}</code></pre>
          </div>
        </div>
      </div>

      <!-- Footer Info -->
      <div class="px-5 py-2.5 border-t border-apple-hairline-dark bg-apple-surface-tile-2 flex items-center justify-between apple-micro-legal text-apple-ink-muted-48 shrink-0">
        <div class="flex items-center gap-2">
          <span>🔒 100% Client-side generation</span>
          <span>•</span>
          <span>Zero external telemetry</span>
        </div>
        <div>
          Tip: Press <kbd class="px-1.5 py-0.5 rounded-apple-xs bg-apple-surface-tile-3 border border-apple-hairline-dark text-apple-body-muted font-mono text-[10px]">Esc</kbd> to exit
        </div>
      </div>
    </div>
  </div>
{/if}
