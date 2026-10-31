<script lang="ts">
  import { workspaceStore, type CookieItem } from '../stores/workspace.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let selectedDomain = $state<string>('');
  let newDomainInput = $state('');
  let isAddingDomain = $state(false);

  let cookies = $derived(workspaceStore.currentWorkspace.cookies || []);

  let domainList = $derived.by(() => {
    const set = new Set<string>();
    for (const ck of cookies) {
      if (ck.domain) set.add(ck.domain);
    }
    if (selectedDomain && !set.has(selectedDomain)) {
      set.add(selectedDomain);
    }
    return Array.from(set);
  });

  $effect(() => {
    if (isOpen && !selectedDomain && domainList.length > 0) {
      selectedDomain = domainList[0];
    }
  });

  let currentDomainCookies = $derived(
    cookies.filter(c => c.domain === selectedDomain)
  );

  function handleAddDomain() {
    const trimmed = newDomainInput.trim().toLowerCase();
    if (!trimmed) return;
    selectedDomain = trimmed;
    newDomainInput = '';
    isAddingDomain = false;
  }

  function handleAddCookie() {
    if (!selectedDomain) return;
    workspaceStore.addCookie({
      domain: selectedDomain,
      path: '/',
      name: 'session_id',
      value: Math.random().toString(36).substring(2, 12),
      httpOnly: true,
      secure: false
    });
  }

  function handleDeleteCookie(cookieId: string) {
    workspaceStore.deleteCookie(cookieId);
  }

  function handleClearDomain() {
    workspaceStore.clearCookies(selectedDomain);
  }

  function handleClearAll() {
    workspaceStore.clearCookies();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 font-sans animate-fade-in">
    <div class="apple-modal-card w-full max-w-4xl h-[560px] flex flex-col text-apple-body-on-dark shadow-2xl">
      <!-- Modal Header -->
      <div class="h-12 border-b border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2.5">
          <span class="text-base text-amber-400">🍪</span>
          <h2 class="apple-body-strong text-apple-body-on-dark tracking-tight">Cookie Jar Manager</h2>
          <span class="apple-micro-legal uppercase font-mono px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark">
            reqwest_cookie_store
          </span>
        </div>
        <button
          type="button"
          aria-label="Close cookie jar modal"
          onclick={onClose}
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
        >
          ✕
        </button>
      </div>

      <!-- Split View Body -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Left: Domain List -->
        <div class="w-56 border-r border-apple-hairline-dark bg-apple-surface-tile-3 p-3 flex flex-col justify-between shrink-0">
          <div class="space-y-2">
            <div class="flex items-center justify-between mb-2">
              <span class="apple-micro-legal font-semibold text-apple-ink-muted-48 uppercase tracking-wider">Domains</span>
              <button
                type="button"
                onclick={() => isAddingDomain = true}
                class="text-xs text-apple-primary-on-dark hover:underline font-medium"
              >
                + Add Domain
              </button>
            </div>

            {#if isAddingDomain}
              <div class="p-2 rounded-apple-sm bg-apple-surface-tile-2 border border-apple-hairline-dark space-y-2">
                <input
                  type="text"
                  bind:value={newDomainInput}
                  placeholder="e.g. 127.0.0.1 or api.dev"
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleAddDomain();
                    if (e.key === 'Escape') isAddingDomain = false;
                  }}
                  class="w-full bg-apple-surface-tile-1 border border-apple-hairline-dark rounded px-2 py-1 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary font-mono"
                />
                <div class="flex gap-1.5 justify-end text-[11px]">
                  <button
                    type="button"
                    onclick={() => isAddingDomain = false}
                    class="px-2 py-0.5 rounded text-apple-body-muted hover:bg-apple-surface-tile-3"
                  >
                    Cancel
                  </button>
                  <button
                    type="button"
                    onclick={handleAddDomain}
                    class="apple-button-primary px-2.5 py-0.5 font-medium"
                  >
                    Add
                  </button>
                </div>
              </div>
            {/if}

            <div class="space-y-1 overflow-auto max-h-[360px]">
              {#each domainList as domain}
                {@const isSelected = selectedDomain === domain}
                {@const count = cookies.filter(c => c.domain === domain).length}
                <div
                  role="button"
                  tabindex="0"
                  onclick={() => selectedDomain = domain}
                  onkeydown={(e) => e.key === 'Enter' && (selectedDomain = domain)}
                  class="w-full text-left p-2 rounded-apple-sm text-xs transition-colors flex items-center justify-between group cursor-pointer font-mono
                    {isSelected
                      ? 'bg-apple-surface-tile-1 text-apple-primary-on-dark border border-apple-primary/40 font-semibold'
                      : 'text-apple-body-muted hover:bg-apple-surface-tile-2 hover:text-white'}"
                >
                  <span class="truncate">{domain}</span>
                  <span class="apple-micro-legal px-1.5 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-ink-muted-48 border border-apple-hairline-dark">
                    {count}
                  </span>
                </div>
              {/each}

              {#if domainList.length === 0}
                <div class="p-4 text-center text-apple-ink-muted-48 apple-fine-print">
                  No domains with cookies yet.
                </div>
              {/if}
            </div>
          </div>

          <div class="pt-2 border-t border-apple-hairline-dark">
            <button
              type="button"
              onclick={handleClearAll}
              class="w-full text-left px-2 py-1 rounded text-xs text-rose-400 hover:bg-apple-surface-tile-2 flex items-center gap-1.5 transition-colors"
            >
              <span>🗑️</span>
              <span>Clear All Cookies</span>
            </button>
          </div>
        </div>

        <!-- Right: Cookies Table for Selected Domain -->
        <div class="flex-1 flex flex-col p-4 overflow-hidden bg-apple-surface-tile-1">
          {#if selectedDomain}
            <div class="flex items-center justify-between mb-3 shrink-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-bold font-mono text-apple-body-on-dark">{selectedDomain}</span>
                <span class="text-xs text-apple-body-muted">({currentDomainCookies.length} cookies)</span>
              </div>

              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={handleAddCookie}
                  class="apple-button-primary text-xs px-3 py-1 font-medium"
                >
                  + Add Cookie
                </button>
                {#if currentDomainCookies.length > 0}
                  <button
                    type="button"
                    onclick={handleClearDomain}
                    class="apple-button-dark-utility text-xs px-2.5 py-1 text-rose-400 hover:text-rose-300"
                  >
                    Clear Domain
                  </button>
                {/if}
              </div>
            </div>

            <!-- Cookie Table -->
            <div class="flex-1 overflow-auto border border-apple-hairline-dark rounded-apple-sm bg-apple-surface-tile-1">
              <table class="w-full border-collapse text-left font-mono text-xs">
                <thead class="sticky top-0 bg-apple-surface-tile-2 border-b border-apple-hairline-dark z-10">
                  <tr class="text-apple-body-muted apple-micro-legal">
                    <th class="w-1/4 p-2 border-r border-apple-hairline-dark">Cookie Name</th>
                    <th class="w-1/3 p-2 border-r border-apple-hairline-dark">Cookie Value</th>
                    <th class="w-20 p-2 border-r border-apple-hairline-dark">Path</th>
                    <th class="w-16 p-2 border-r border-apple-hairline-dark text-center font-sans">HttpOnly</th>
                    <th class="w-16 p-2 border-r border-apple-hairline-dark text-center font-sans">Secure</th>
                    <th class="w-8 p-2 text-center"></th>
                  </tr>
                </thead>
                <tbody>
                  {#each currentDomainCookies as ck (ck.id)}
                    <tr class="border-b border-apple-hairline-dark/60 hover:bg-apple-surface-tile-2/40">
                      <td class="p-1 border-r border-apple-hairline-dark">
                        <input
                          type="text"
                          bind:value={ck.name}
                          placeholder="name"
                          oninput={() => workspaceStore.scheduleDiskSync()}
                          class="w-full bg-transparent px-2 py-1 text-apple-primary-on-dark focus:outline-none"
                        />
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark">
                        <input
                          type="text"
                          bind:value={ck.value}
                          placeholder="value"
                          oninput={() => workspaceStore.scheduleDiskSync()}
                          class="w-full bg-transparent px-2 py-1 text-apple-body-on-dark focus:outline-none"
                        />
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark">
                        <input
                          type="text"
                          bind:value={ck.path}
                          placeholder="/"
                          oninput={() => workspaceStore.scheduleDiskSync()}
                          class="w-full bg-transparent px-2 py-1 text-apple-body-muted focus:outline-none"
                        />
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark text-center">
                        <input
                          type="checkbox"
                          bind:checked={ck.httpOnly}
                          onchange={() => workspaceStore.scheduleDiskSync()}
                          class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark text-apple-primary focus:ring-0"
                        />
                      </td>
                      <td class="p-1 border-r border-apple-hairline-dark text-center">
                        <input
                          type="checkbox"
                          bind:checked={ck.secure}
                          onchange={() => workspaceStore.scheduleDiskSync()}
                          class="rounded-apple-xs bg-apple-surface-tile-3 border-apple-hairline-dark text-apple-primary focus:ring-0"
                        />
                      </td>
                      <td class="p-2 text-center">
                        <button
                          type="button"
                          onclick={() => handleDeleteCookie(ck.id)}
                          class="text-apple-ink-muted-48 hover:text-rose-400"
                        >
                          ✕
                        </button>
                      </td>
                    </tr>
                  {/each}

                  {#if currentDomainCookies.length === 0}
                    <tr>
                      <td colspan="6" class="p-8 text-center text-apple-body-muted font-sans apple-caption">
                        No cookies stored for domain "{selectedDomain}". Click "+ Add Cookie" above.
                      </td>
                    </tr>
                  {/if}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="flex-1 flex items-center justify-center text-apple-body-muted">
              Select or create a domain on the left to manage cookies.
            </div>
          {/if}
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="h-12 border-t border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <span class="apple-fine-print text-apple-ink-muted-48">
          Cookies persist automatically across sessions and are scoped strictly per host and domain.
        </span>
        <button
          type="button"
          onclick={onClose}
          class="apple-button-dark-utility text-xs px-4 py-1.5 font-semibold"
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
