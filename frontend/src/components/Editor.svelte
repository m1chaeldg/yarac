<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView, hoverTooltip, keymap } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { defaultKeymap } from '@codemirror/commands';

  interface Props {
    content?: string;
    onchange?: (content: string) => void;
  }

  let { content = '', onchange }: Props = $props();

  let editorElement: HTMLDivElement | null = $state(null);
  let view: EditorView | null = $state(null);

  // Mock variable resolution for Hover Tooltip extension
  const variableRegistry: Record<string, { value: string; scope: string; isKeyring: boolean }> = {
    base_url: { value: 'https://api.yarac.dev', scope: 'Environment: Production', isKeyring: false },
    api_bearer_token: { value: 'ey...[REDACTED]', scope: 'Local', isKeyring: true },
    signing_key: { value: 'sec_99482910385', scope: 'Global (OS Keyring)', isKeyring: true },
    uuid: { value: 'e7b1a2c3-4d5e-6f7a-8b9c-0d1e2f3a4b5c', scope: 'Dynamic ($uuid)', isKeyring: false },
  };

  // CodeMirror 6 Hover Tooltip Extension per design.md & tasks.md
  const variableHoverTooltip = hoverTooltip((view, pos, side) => {
    const { from, to, text } = view.state.doc.lineAt(pos);
    const lineText = text;
    const lineOffset = pos - from;

    // Search for {{variable_name}} boundaries
    const regex = /\{\{([^}]+)\}\}/g;
    let match;
    while ((match = regex.exec(lineText)) !== null) {
      const start = match.index;
      const end = start + match[0].length;

      if (lineOffset >= start && lineOffset <= end) {
        const varName = match[1].trim();
        const resolved = variableRegistry[varName] || {
          value: '(unresolved variable)',
          scope: 'Unknown',
          isKeyring: false
        };

        return {
          pos: from + start,
          end: from + end,
          above: true,
          create() {
            const dom = document.createElement('div');
            dom.className = 'cm-yarac-tooltip bg-apple-surface-tile-2 border border-apple-hairline-dark/80 rounded-apple-md p-2.5 text-xs text-apple-primary-on-dark shadow-none z-50 max-w-sm';
            dom.innerHTML = `
              <div class="flex items-center gap-1.5 pb-1 border-b border-apple-hairline-dark/60 font-mono font-semibold text-apple-link-on-dark">
                <span>{{${varName}}}</span>
                ${resolved.isKeyring ? '<span class="px-1.5 py-0.5 rounded-apple-sm text-[10px] bg-apple-surface-tile-3 text-apple-link-on-dark border border-apple-hairline-dark/80">🔒 Keyring</span>' : ''}
              </div>
              <div class="pt-1.5 space-y-1">
                <div class="text-apple-secondary-on-dark">Value: <span class="font-mono text-apple-primary-on-dark">${resolved.value}</span></div>
                <div class="text-[11px] text-apple-secondary-on-dark">Scope: <span class="text-apple-primary-on-dark">${resolved.scope}</span></div>
              </div>
            `;
            return { dom };
          }
        };
      }
    }
    return null;
  });

  onMount(() => {
    if (!editorElement) return;

    const startState = EditorState.create({
      doc: content,
      extensions: [
        keymap.of(defaultKeymap),
        variableHoverTooltip,
        EditorView.theme({
          '&': { height: '100%', fontSize: '13px', backgroundColor: '#1c1c1e', color: '#f5f5f7' },
          '.cm-content': { fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace' },
          '.cm-gutters': { backgroundColor: '#1c1c1e', color: '#86868b', borderRight: '1px solid #3a3a3c' },
          '&.cm-focused .cm-cursor': { borderLeftColor: '#2997ff' },
          '&.cm-focused .cm-selectionBackground, ::selection': { backgroundColor: '#3a3a3c' },
        }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged && onchange) {
            onchange(update.state.doc.toString());
          }
        })
      ]
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
    if (view && content !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: content }
      });
    }
  });
</script>

<div class="w-full h-full relative" bind:this={editorElement}></div>
