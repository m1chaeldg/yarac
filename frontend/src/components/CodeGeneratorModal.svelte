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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4 animate-in fade-in duration-150"
    onclick={onClose}
    role="presentation"
  >
    <div
      class="bg-slate-900 border border-slate-800 rounded-xl shadow-2xl shadow-black/90 w-full max-w-4xl h-[78vh] flex flex-col overflow-hidden text-slate-100"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="code-generator-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-5 py-3.5 border-b border-slate-800 flex items-center justify-between shrink-0 bg-slate-950/60">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-sky-600/20 text-sky-400 flex items-center justify-center font-bold text-sm border border-sky-500/30">
            &lt;/&gt;
          </div>
          <div>
            <h2 id="code-generator-title" class="text-sm font-bold text-slate-100 flex items-center gap-2">
              Generate Code Snippet
              <span class="text-xs font-mono font-normal text-slate-400 px-2 py-0.5 rounded bg-slate-800/80">
                {request.method} {request.name}
              </span>
            </h2>
            <p class="text-[11px] text-slate-400">
              Ready-to-use client implementations across major languages and HTTP frameworks.
            </p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="w-7 h-7 rounded-lg text-slate-400 hover:text-slate-100 hover:bg-slate-800 flex items-center justify-center text-sm font-bold transition-colors"
          title="Close dialog"
        >
          ✕
        </button>
      </div>

      <!-- Main Body -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Sidebar: Language Selection -->
        <div class="w-48 border-r border-slate-800 bg-slate-950/40 p-2 overflow-y-auto space-y-1 shrink-0 select-none">
          <div class="px-2 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-500">
            Languages ({CODE_LANGUAGES.length})
          </div>
          {#each CODE_LANGUAGES as lang (lang.id)}
            <button
              type="button"
              onclick={() => {
                selectedLangId = lang.id;
                selectedVariantId = lang.variants[0]?.id || '';
              }}
              class="w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-medium flex items-center gap-2 transition-colors
                {selectedLangId === lang.id
                  ? 'bg-sky-600/20 text-sky-300 font-semibold border border-sky-500/30 shadow-sm'
                  : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
            >
              <span>{lang.icon}</span>
              <span class="truncate">{lang.name}</span>
            </button>
          {/each}
        </div>

        <!-- Center Code Display -->
        <div class="flex-1 flex flex-col overflow-hidden bg-slate-950/80">
          <!-- Options bar -->
          <div class="px-4 py-2 border-b border-slate-800/80 bg-slate-900/40 flex items-center justify-between gap-3 shrink-0">
            <!-- Variant Selector -->
            <div class="flex items-center gap-2">
              <span class="text-[11px] text-slate-400 font-medium">Library:</span>
              <div class="flex items-center gap-1 bg-slate-900 border border-slate-700/80 rounded-lg p-0.5">
                {#each currentLang.variants as variant (variant.id)}
                  <button
                    type="button"
                    onclick={() => selectedVariantId = variant.id}
                    class="px-2 py-1 rounded text-xs transition-colors
                      {selectedVariantId === variant.id
                        ? 'bg-sky-600 text-white font-medium shadow-xs'
                        : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
                  >
                    {variant.name}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Options & Copy Button -->
            <div class="flex items-center gap-3">
              <label class="flex items-center gap-1.5 text-xs text-slate-300 cursor-pointer select-none">
                <input
                  type="checkbox"
                  bind:checked={interpolateEnv}
                  class="rounded bg-slate-900 border-slate-700 text-sky-600 focus:ring-0 focus:ring-offset-0"
                />
                <span class="text-[11px] text-slate-400">Resolve Environment Vars</span>
              </label>

              <button
                type="button"
                onclick={copyToClipboard}
                class="px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all
                  {isCopied
                    ? 'bg-emerald-600 text-white shadow-lg shadow-emerald-900/40'
                    : 'bg-sky-600 hover:bg-sky-500 text-white shadow-lg shadow-sky-900/30'}"
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
          <div class="flex-1 overflow-auto p-4 font-mono text-xs bg-slate-950 text-slate-200 leading-relaxed selection:bg-sky-900 selection:text-white">
            <pre class="m-0 font-mono"><code>{generatedCode}</code></pre>
          </div>
        </div>
      </div>

      <!-- Footer Info -->
      <div class="px-5 py-2.5 border-t border-slate-800 bg-slate-950/60 flex items-center justify-between text-[11px] text-slate-500 shrink-0">
        <div class="flex items-center gap-2">
          <span>🔒 100% Client-side generation</span>
          <span>•</span>
          <span>Zero external telemetry</span>
        </div>
        <div>
          Tip: Press <kbd class="px-1 py-0.5 rounded bg-slate-800 text-slate-300 font-mono text-[10px]">Esc</kbd> to exit
        </div>
      </div>
    </div>
  </div>
{/if}
