<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView, hoverTooltip, keymap } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { defaultKeymap } from '@codemirror/commands';
  import { StreamLanguage, HighlightStyle, syntaxHighlighting } from '@codemirror/language';
  import { tags as t } from '@lezer/highlight';
  import { workspaceStore } from '../stores/workspace.svelte';

  interface Props {
    content?: string;
    onchange?: (content: string) => void;
  }

  let { content = '', onchange }: Props = $props();

  let editorElement: HTMLDivElement | null = $state(null);
  let view: EditorView | null = $state(null);

  // Apple dark palette highlight styles
  const yaracHighlightStyle = HighlightStyle.define([
    { tag: t.heading, color: '#2997ff', fontWeight: 'bold' },
    { tag: t.propertyName, color: '#e5e5ea', fontWeight: '600' },
    { tag: t.string, color: '#30d158' },
    { tag: t.number, color: '#ff9f0a' },
    { tag: t.bool, color: '#ff453a', fontWeight: '600' },
    { tag: t.comment, color: '#8e8e93', fontStyle: 'italic' },
    { tag: t.special(t.variableName), color: '#64d2ff', fontWeight: '600' },
    { tag: t.operator, color: '#8e8e93' },
    { tag: t.keyword, color: '#bf5af2', fontWeight: 'bold' },
  ]);

  // CodeMirror StreamLanguage for .yarac format (TOML headers, keys, fenced blocks)
  const yaracLanguage = StreamLanguage.define({
    token(stream) {
      if (stream.eatSpace()) return null;

      // Comment (# ...)
      if (stream.match(/^#.*/)) {
        return 'comment';
      }

      // Section header [section] or [section.sub]
      if (stream.match(/^\[[\w\.-]+\]/)) {
        return 'heading';
      }

      // Fenced code blocks ```...
      if (stream.match(/^```[\w-]*/)) {
        return 'keyword';
      }

      // Dynamic variables {{...}}
      if (stream.match(/^\{\{[^}]+\}\}/)) {
        return 'special(variableName)';
      }

      // String literals ("..." or '...')
      if (stream.match(/^"(?:[^"\\]|\\.)*"/)) {
        return 'string';
      }
      if (stream.match(/^'(?:[^'\\]|\\.)*'/)) {
        return 'string';
      }

      // Numbers
      if (stream.match(/^[0-9]+(\.[0-9]+)?\b/)) {
        return 'number';
      }

      // Booleans
      if (stream.match(/^(true|false)\b/)) {
        return 'bool';
      }

      // HTTP Methods
      if (stream.match(/^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\b/)) {
        return 'keyword';
      }

      // Key names (before =)
      if (stream.match(/^[a-zA-Z0-9_~-]+(?=\s*=)/)) {
        return 'propertyName';
      }

      // Operator
      if (stream.match(/^[=]/)) {
        return 'operator';
      }

      stream.next();
      return null;
    }
  });

  // Dynamic variable resolver across 5-tier scope stack and Bruno faker macros
  function resolveVariableTooltip(varName: string): { value: string; scope: string; isKeyring: boolean } {
    // 1. Dynamic Bruno / Faker Macros
    if (varName.startsWith('$')) {
      const lower = varName.toLowerCase();
      if (lower === '$uuid' || lower === '$guid') {
        return { value: 'e7b1a2c3-4d5e-6f7a-8b9c-0d1e2f3a4b5c', scope: 'Dynamic ($uuid)', isKeyring: false };
      }
      if (lower === '$timestamp') {
        return { value: String(Math.floor(Date.now() / 1000)), scope: 'Dynamic ($timestamp)', isKeyring: false };
      }
      if (lower === '$isotimestamp') {
        return { value: new Date().toISOString(), scope: 'Dynamic ($isoTimestamp)', isKeyring: false };
      }
      if (lower === '$randomint') {
        return { value: '42', scope: 'Dynamic ($randomInt)', isKeyring: false };
      }
      if (lower === '$randomfloat') {
        return { value: '42.50', scope: 'Dynamic ($randomFloat)', isKeyring: false };
      }
      if (lower === '$randomemail') {
        return { value: 'user_dev@example.com', scope: 'Dynamic ($randomEmail)', isKeyring: false };
      }
      if (lower === '$randomfullname') {
        return { value: 'Alex Mercer', scope: 'Dynamic ($randomFullName)', isKeyring: false };
      }
      if (lower === '$randompassword') {
        return { value: '••••••••', scope: 'Dynamic ($randomPassword)', isKeyring: false };
      }
      if (lower === '$randomcity') {
        return { value: 'San Francisco', scope: 'Dynamic ($randomCity)', isKeyring: false };
      }
      if (lower === '$randomcountry') {
        return { value: 'United States', scope: 'Dynamic ($randomCountry)', isKeyring: false };
      }
      if (lower === '$randomprice') {
        return { value: '49.99', scope: 'Dynamic ($randomPrice)', isKeyring: false };
      }
      if (lower === '$randomcompany') {
        return { value: 'Acme Corp', scope: 'Dynamic ($randomCompany)', isKeyring: false };
      }
    }

    // 2. Request Scoped Variables
    const activeReq = workspaceStore.activeRequest;
    if (activeReq?.variables) {
      const match = activeReq.variables.find(v => v.key === varName && v.enabled && !v.key.endsWith('~'));
      if (match) {
        return {
          value: match.isSecret ? '•••••••• [REDACTED]' : match.value,
          scope: `Request: ${activeReq.name}`,
          isKeyring: Boolean(match.isSecret)
        };
      }
    }

    // 3. Active Environment Variables
    const activeEnv = workspaceStore.activeEnvironment;
    if (activeEnv?.variables) {
      const match = activeEnv.variables.find(v => v.key === varName && v.enabled && !v.key.endsWith('~'));
      if (match) {
        return {
          value: match.isSecret ? '•••••••• [REDACTED]' : match.value,
          scope: `Environment: ${activeEnv.name}`,
          isKeyring: Boolean(match.isSecret)
        };
      }
    }

    // 4. Default Base URL & common templates
    if (varName === 'base_url') {
      return { value: 'https://api.yarac.dev', scope: 'Default (Workspace Scope)', isKeyring: false };
    }

    return {
      value: '(unresolved variable)',
      scope: 'Unresolved',
      isKeyring: false
    };
  }

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
        const resolved = resolveVariableTooltip(varName);

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
        yaracLanguage,
        syntaxHighlighting(yaracHighlightStyle),
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
