<script lang="ts">
  import { backendApi } from '../services/backendApi';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen, onClose, onSuccess }: Props = $props();

  let branch = $state('main');
  let modifiedFiles = $state<string[]>([]);
  let untrackedFiles = $state<string[]>([]);
  let selectedFiles = $state<Record<string, boolean>>({});
  let commitMessage = $state('');
  let isCommitting = $state(false);
  let isSyncing = $state(false);
  let feedbackMessage = $state<string | null>(null);
  let branches = $state<string[]>(['main']);

  async function loadStatus() {
    try {
      const st = await backendApi.getGitStatus();
      branch = st.branch || 'main';
      modifiedFiles = st.modified_files || [];
      untrackedFiles = st.untracked_files || [];

      // Auto-select all by default
      const sel: Record<string, boolean> = {};
      for (const f of modifiedFiles) sel[f] = true;
      for (const f of untrackedFiles) sel[f] = true;
      selectedFiles = sel;

      branches = await backendApi.getGitBranches();
    } catch (e: any) {
      feedbackMessage = 'Failed to load git status';
    }
  }

  $effect(() => {
    if (isOpen) {
      commitMessage = '';
      feedbackMessage = null;
      loadStatus();
    }
  });

  let totalChanges = $derived(modifiedFiles.length + untrackedFiles.length);

  let selectedCount = $derived(
    Object.values(selectedFiles).filter(Boolean).length
  );

  function toggleAll(selectAll: boolean) {
    const sel: Record<string, boolean> = {};
    for (const f of modifiedFiles) sel[f] = selectAll;
    for (const f of untrackedFiles) sel[f] = selectAll;
    selectedFiles = sel;
  }

  async function handleCommit(andSync = false) {
    if (!commitMessage.trim() || selectedCount === 0 || isCommitting) return;
    isCommitting = true;
    feedbackMessage = null;

    try {
      const res = await backendApi.commitGit(commitMessage.trim());
      if (res.status === 'ok') {
        if (andSync) {
          isSyncing = true;
          const syncRes = await backendApi.syncGit();
          isSyncing = false;
          if (syncRes.status !== 'ok') {
            feedbackMessage = 'Committed, but sync failed: ' + (syncRes.message || 'unknown error');
            isCommitting = false;
            return;
          }
        }
        feedbackMessage = 'Successfully committed changes!';
        setTimeout(() => {
          onSuccess?.();
          onClose();
        }, 1000);
      } else {
        feedbackMessage = 'Commit error: ' + (res.message || 'unknown');
      }
    } catch (e: any) {
      feedbackMessage = 'Error: ' + e.message;
    } finally {
      isCommitting = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      handleCommit(false);
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center apple-frosted-backdrop p-4 font-sans animate-fade-in">
    <div class="apple-modal-card w-full max-w-2xl h-[540px] flex flex-col text-apple-body-on-dark shadow-2xl">
      <!-- Modal Header -->
      <div class="h-12 border-b border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2.5">
          <span class="text-base text-apple-primary-on-dark">🌿</span>
          <h2 class="apple-body-strong text-apple-body-on-dark tracking-tight">Git Changes & Commit</h2>
          <span class="apple-micro-legal font-mono px-2 py-0.5 rounded-apple-pill bg-apple-surface-tile-3 text-apple-primary-on-dark border border-apple-hairline-dark flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-apple-full bg-emerald-400"></span>
            <span>{branch}</span>
          </span>
        </div>

        <button
          type="button"
          aria-label="Close commit modal"
          onclick={onClose}
          class="apple-button-icon-circular w-7 h-7 text-xs font-bold"
        >
          ✕
        </button>
      </div>

      <!-- Main Body -->
      <div class="flex-1 flex flex-col p-4 overflow-hidden bg-apple-surface-tile-1 space-y-3">
        <!-- Changes Header Toolbar -->
        <div class="flex items-center justify-between shrink-0">
          <div class="flex items-center gap-2">
            <span class="text-xs font-semibold text-apple-body-on-dark">Changed Files ({totalChanges})</span>
            <span class="apple-micro-legal text-apple-body-muted font-mono">{selectedCount} selected</span>
          </div>

          <div class="flex items-center gap-2 text-xs">
            <button
              type="button"
              onclick={() => toggleAll(true)}
              class="text-apple-primary-on-dark hover:underline"
            >
              Select All
            </button>
            <span class="text-apple-ink-muted-48">•</span>
            <button
              type="button"
              onclick={() => toggleAll(false)}
              class="text-apple-body-muted hover:text-white"
            >
              Unselect All
            </button>
          </div>
        </div>

        <!-- Files List -->
        <div class="flex-1 overflow-auto border border-apple-hairline-dark rounded-apple-sm bg-apple-surface-tile-2 p-2 space-y-1 font-mono text-xs">
          {#each modifiedFiles as file}
            <label class="flex items-center gap-2 p-1.5 rounded hover:bg-apple-surface-tile-3 cursor-pointer transition-colors">
              <input
                type="checkbox"
                bind:checked={selectedFiles[file]}
                class="rounded-apple-xs bg-apple-surface-tile-1 border-apple-hairline-dark text-apple-primary focus:ring-0"
              />
              <span class="text-amber-400 text-[10px] font-sans font-bold px-1 rounded bg-amber-400/10">M</span>
              <span class="truncate text-apple-body-on-dark">{file}</span>
            </label>
          {/each}

          {#each untrackedFiles as file}
            <label class="flex items-center gap-2 p-1.5 rounded hover:bg-apple-surface-tile-3 cursor-pointer transition-colors">
              <input
                type="checkbox"
                bind:checked={selectedFiles[file]}
                class="rounded-apple-xs bg-apple-surface-tile-1 border-apple-hairline-dark text-apple-primary focus:ring-0"
              />
              <span class="text-emerald-400 text-[10px] font-sans font-bold px-1 rounded bg-emerald-400/10">U</span>
              <span class="truncate text-apple-body-on-dark">{file}</span>
            </label>
          {/each}

          {#if totalChanges === 0}
            <div class="p-8 text-center text-apple-body-muted font-sans">
              Working tree clean. No uncommitted changes detected.
            </div>
          {/if}
        </div>

        <!-- Commit Message Input -->
        <div class="shrink-0 space-y-1.5">
          <label for="commit-msg-input" class="text-xs font-semibold text-apple-body-on-dark block">
            Commit Message
          </label>
          <textarea
            id="commit-msg-input"
            bind:value={commitMessage}
            placeholder="Describe your API request changes... (Ctrl+Enter to commit)"
            rows="3"
            class="w-full bg-apple-surface-tile-2 border border-apple-hairline-dark rounded p-2.5 text-xs text-apple-body-on-dark focus:outline-none focus:border-apple-primary font-sans resize-none"
          ></textarea>
        </div>

        {#if feedbackMessage}
          <div class="text-xs px-3 py-1.5 rounded bg-apple-surface-tile-3 border border-apple-hairline-dark text-apple-primary-on-dark">
            {feedbackMessage}
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="h-12 border-t border-apple-hairline-dark bg-apple-surface-tile-2 px-4 flex items-center justify-between shrink-0">
        <span class="apple-fine-print text-apple-ink-muted-48">
          Hybrid Git tracks collection files directly in workspace root.
        </span>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={onClose}
            class="apple-button-dark-utility text-xs px-3 py-1.5"
          >
            Cancel
          </button>

          <button
            type="button"
            disabled={!commitMessage.trim() || selectedCount === 0 || isCommitting}
            onclick={() => handleCommit(false)}
            class="apple-button-dark-utility text-xs px-3.5 py-1.5 font-semibold text-apple-primary-on-dark disabled:opacity-40"
          >
            {isCommitting && !isSyncing ? 'Committing...' : 'Commit'}
          </button>

          <button
            type="button"
            disabled={!commitMessage.trim() || selectedCount === 0 || isCommitting}
            onclick={() => handleCommit(true)}
            class="apple-button-primary text-xs px-4 py-1.5 font-semibold disabled:opacity-40"
          >
            {isSyncing ? 'Syncing...' : 'Commit & Sync'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
