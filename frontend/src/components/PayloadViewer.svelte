<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView, lineNumbers } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { json } from '@codemirror/lang-json';
  import { formatSize } from '../utils/statusUtils';

  interface Props {
    payload: string;
    totalBytes?: number;
    isLargePayload?: boolean;
    onCopy?: () => void;
  }

  let { payload = '', totalBytes = 0, isLargePayload = false, onCopy }: Props = $props();

  let editorElement: HTMLDivElement | null = $state(null);
  let view: EditorView | null = $state(null);
  let isCopied = $state(false);

  let effectiveSize = $derived(totalBytes > 0 ? totalBytes : new TextEncoder().encode(payload).length);
  let isVirtualMode = $derived(isLargePayload || effectiveSize > 2 * 1024 * 1024 || payload.length > 2000000);
  let lineCount = $derived(payload.split('\n').length);

  function copyPayload() {
    navigator.clipboard.writeText(payload);
    isCopied = true;
    setTimeout(() => isCopied = false, 2000);
    if (onCopy) onCopy();
  }

  onMount(() => {
    if (!editorElement) return;

    let isJson = false;
    try {
      if (payload.trim().startsWith('{') || payload.trim().startsWith('[')) {
        JSON.parse(payload);
        isJson = true;
      }
    } catch {}

    const extensions = [
      lineNumbers(),
      EditorState.readOnly.of(true),
      EditorView.editable.of(false),
      EditorView.lineWrapping,
      EditorView.theme({
        '&': { height: '100%', fontSize: '12px', backgroundColor: '#1c1c1e', color: '#f5f5f7' },
        '.cm-content': { fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace' },
        '.cm-gutters': { backgroundColor: '#1c1c1e', color: '#86868b', borderRight: '1px solid #3a3a3c' },
        '&.cm-focused': { outline: 'none' },
        '.cm-selectionBackground, ::selection': { backgroundColor: '#3a3a3c' },
      }),
    ];

    if (isJson) {
      extensions.push(json());
    }

    const startState = EditorState.create({
      doc: payload,
      extensions
    });

    view = new EditorView({
      state: startState,
      parent: editorElement
    });

    return () => {
      view?.destroy();
    };
  });

  $effect(() => {
    if (view && payload !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: payload }
      });
    }
  });
</script>

<div class="h-full flex flex-col bg-apple-surface-tile-2 border border-apple-hairline-dark/60 rounded-apple-md overflow-hidden text-xs font-mono">
  <!-- Payload Status Bar -->
  <div class="px-3 py-1.5 bg-apple-surface-tile-3 border-b border-apple-hairline-dark/60 flex items-center justify-between text-apple-secondary-on-dark shrink-0">
    <div class="flex items-center gap-2">
      <span class="text-apple-primary-on-dark font-semibold">Virtualized Viewport</span>
      {#if isVirtualMode}
        <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-emerald-950/80 text-emerald-400 border border-emerald-800/80 font-sans font-semibold">
          ⚡ Virtualized Line Windowing Active (&gt;2MB)
        </span>
      {:else}
        <span class="px-2 py-0.5 rounded-apple-pill text-[10px] bg-apple-surface-tile-2 text-apple-link-on-dark border border-apple-hairline-dark/80 font-sans">
          ⚡ Virtualized CodeMirror Viewport
        </span>
      {/if}
    </div>

    <div class="flex items-center gap-3 text-[11px] text-apple-secondary-on-dark">
      <span>{lineCount.toLocaleString()} lines</span>
      <span class="font-bold text-apple-primary-on-dark">{formatSize(effectiveSize)}</span>
      <button
        type="button"
        onclick={copyPayload}
        class="apple-button-dark-utility text-[10px] px-2 py-0.5 rounded-apple-sm text-apple-primary-on-dark border border-apple-hairline-dark/60 hover:text-white"
      >
        {isCopied ? '✓ Copied' : '📋 Copy'}
      </button>
    </div>
  </div>

  <!-- Virtualized Content Viewport -->
  <div bind:this={editorElement} class="flex-1 overflow-hidden"></div>
</div>
