<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, lineNumbers, keymap, placeholder as cmPlaceholder } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { defaultKeymap } from '@codemirror/commands';
  import { javascript } from '@codemirror/lang-javascript';
  import { json } from '@codemirror/lang-json';

  interface Props {
    content?: string;
    language?: 'javascript' | 'json' | 'text';
    placeholder?: string;
    readOnly?: boolean;
    onchange?: (val: string) => void;
  }

  let {
    content = '',
    language = 'javascript',
    placeholder = '',
    readOnly = false,
    onchange
  }: Props = $props();

  let container: HTMLDivElement | null = $state(null);
  let view: EditorView | null = $state(null);

  onMount(() => {
    if (!container) return;

    const extensions = [
      lineNumbers(),
      keymap.of(defaultKeymap),
      EditorView.lineWrapping,
      EditorView.theme({
        '&': { height: '100%', fontSize: '12px', backgroundColor: '#000000', color: '#f5f5f7' },
        '.cm-content': { fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace', padding: '8px 4px' },
        '.cm-gutters': { backgroundColor: '#1c1c1e', color: '#86868b', borderRight: '1px solid #3a3a3c' },
        '&.cm-focused': { outline: 'none' },
        '.cm-selectionBackground, ::selection': { backgroundColor: 'rgba(41, 151, 255, 0.25)' },
        '.cm-cursor': { borderLeftColor: '#2997ff' },
        '.cm-placeholder': { color: '#636366', fontStyle: 'italic' }
      }),
      EditorView.updateListener.of((update) => {
        if (update.docChanged && onchange) {
          onchange(update.state.doc.toString());
        }
      })
    ];

    if (placeholder) {
      extensions.push(cmPlaceholder(placeholder));
    }

    if (readOnly) {
      extensions.push(EditorState.readOnly.of(true));
      extensions.push(EditorView.editable.of(false));
    }

    if (language === 'javascript') {
      extensions.push(javascript());
    } else if (language === 'json') {
      extensions.push(json());
    }

    const state = EditorState.create({
      doc: content,
      extensions
    });

    view = new EditorView({
      state,
      parent: container
    });

    return () => {
      view?.destroy();
      view = null;
    };
  });

  $effect(() => {
    if (view && content !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: content }
      });
    }
  });
</script>

<div class="h-full w-full overflow-hidden" bind:this={container}></div>
