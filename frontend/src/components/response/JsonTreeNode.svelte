<script lang="ts">
  import { getJsonType, formatPrimitiveValue, getContainerSummary, type JsonType } from '../../utils/jsonUtils';
  import JsonTreeNode from './JsonTreeNode.svelte';

  interface Props {
    keyName?: string | number | null;
    value: any;
    depth?: number;
    expandVersion: number;
    collapseVersion: number;
    defaultExpanded?: boolean;
    isLast?: boolean;
    searchQuery?: string;
  }

  let {
    keyName = null,
    value,
    depth = 0,
    expandVersion,
    collapseVersion,
    defaultExpanded = true,
    isLast = true,
    searchQuery = ''
  }: Props = $props();

  let valType = $derived<JsonType>(getJsonType(value));
  let isContainer = $derived(valType === 'object' || valType === 'array');
  let containerSummary = $derived(getContainerSummary(value, valType));

  let isExpanded = $state(true);

  // Track parent expand all / collapse all broadcast counters
  let lastSeenExpand = 0;
  let lastSeenCollapse = 0;

  $effect(() => {
    if (expandVersion > lastSeenExpand) {
      lastSeenExpand = expandVersion;
      isExpanded = true;
    }
  });

  $effect(() => {
    if (collapseVersion > lastSeenCollapse) {
      lastSeenCollapse = collapseVersion;
      isExpanded = false;
    }
  });

  $effect(() => {
    if (searchQuery && isContainer) {
      try {
        if (JSON.stringify(value).toLowerCase().includes(searchQuery.toLowerCase())) {
          isExpanded = true;
        }
      } catch {}
    }
  });

  function toggle() {
    if (isContainer) {
      isExpanded = !isExpanded;
    }
  }

  function getHighlightParts(text: string, query: string) {
    if (!query) return null;
    const idx = text.toLowerCase().indexOf(query.toLowerCase());
    if (idx === -1) return null;
    return {
      before: text.substring(0, idx),
      match: text.substring(idx, idx + query.length),
      after: text.substring(idx + query.length)
    };
  }

  let keyParts = $derived(keyName !== null && searchQuery ? getHighlightParts(String(keyName), searchQuery) : null);
  let valParts = $derived(!isContainer && searchQuery ? getHighlightParts(formatPrimitiveValue(value, valType), searchQuery) : null);
</script>

<div class="font-mono text-xs leading-relaxed select-text">
  {#if isContainer}
    <!-- Container Row (Object or Array) -->
    <div
      class="flex items-center gap-1 group py-0.5 px-1 rounded-apple-xs hover:bg-apple-surface-tile-2 transition-colors cursor-pointer"
      onclick={toggle}
      role="button"
      tabindex="0"
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggle(); } }}
    >
      <!-- Expand/Collapse Chevron -->
      <span
        class="inline-flex items-center justify-center w-3.5 h-3.5 text-apple-ink-muted-48 group-hover:text-apple-primary-on-dark transition-transform duration-150 {isExpanded ? 'rotate-90' : 'rotate-0'}"
        aria-hidden="true"
      >
        ▶
      </span>

      <!-- Key Name if applicable -->
      {#if keyName !== null && keyName !== undefined}
        {#if keyParts}
          <span class="text-apple-primary-on-dark font-medium font-mono">{keyParts.before}<mark class="bg-amber-400/40 text-amber-200 px-0.5 rounded">{keyParts.match}</mark>{keyParts.after}:</span>
        {:else}
          <span class="text-apple-primary-on-dark font-medium font-mono">{keyName}:</span>
        {/if}
      {/if}

      <!-- Open Bracket / Summary -->
      {#if valType === 'array'}
        <span class="text-apple-body-muted font-bold">[</span>
        {#if !isExpanded}
          <span class="text-[11px] px-1 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-ink-muted-48 border border-apple-hairline-dark/60 font-sans">
            {containerSummary}
          </span>
          <span class="text-apple-body-muted font-bold">]</span>
        {/if}
      {:else}
        <span class="text-apple-body-muted font-bold">&#123;</span>
        {#if !isExpanded}
          <span class="text-[11px] px-1 py-0.2 rounded-apple-pill bg-apple-surface-tile-3 text-apple-ink-muted-48 border border-apple-hairline-dark/60 font-sans">
            {containerSummary}
          </span>
          <span class="text-apple-body-muted font-bold">&#125;</span>
        {/if}
      {/if}

      {#if !isLast && !isExpanded}
        <span class="text-apple-ink-muted-48">,</span>
      {/if}
    </div>

    <!-- Nested Items Viewport -->
    {#if isExpanded}
      <div class="pl-4 ml-1.5 border-l border-apple-hairline-dark/40 flex flex-col space-y-0.5">
        {#if valType === 'array'}
          {#each value as item, idx (idx)}
            <JsonTreeNode
              keyName={idx}
              value={item}
              depth={depth + 1}
              {expandVersion}
              {collapseVersion}
              {defaultExpanded}
              isLast={idx === value.length - 1}
              {searchQuery}
            />
          {/each}
        {:else if valType === 'object' && value}
          {#each Object.entries(value) as [k, v], idx (k)}
            <JsonTreeNode
              keyName={k}
              value={v}
              depth={depth + 1}
              {expandVersion}
              {collapseVersion}
              {defaultExpanded}
              isLast={idx === Object.keys(value).length - 1}
              {searchQuery}
            />
          {/each}
        {/if}
      </div>

      <!-- Closing Bracket -->
      <div class="pl-4 py-0.5 text-apple-body-muted font-bold">
        {valType === 'array' ? ']' : '}'}{#if !isLast}<span class="text-apple-ink-muted-48">,</span>{/if}
      </div>
    {/if}

  {:else}
    <!-- Primitive Row (String, Number, Boolean, Null) -->
    <div class="flex items-center gap-1.5 py-0.5 px-1 hover:bg-apple-surface-tile-2 rounded-apple-xs transition-colors">
      <!-- Spacer for align with containers -->
      <span class="w-3.5 inline-block"></span>

      {#if keyName !== null && keyName !== undefined}
        {#if keyParts}
          <span class="text-apple-primary-on-dark font-medium font-mono">{keyParts.before}<mark class="bg-amber-400/40 text-amber-200 px-0.5 rounded">{keyParts.match}</mark>{keyParts.after}:</span>
        {:else}
          <span class="text-apple-primary-on-dark font-medium font-mono">{keyName}:</span>
        {/if}
      {/if}

      {#if valParts}
        <span class="break-all {valType === 'string' ? 'text-emerald-400' : valType === 'number' ? 'text-sky-400' : valType === 'boolean' ? 'text-amber-400' : 'text-apple-ink-muted-48'}">
          {valParts.before}<mark class="bg-amber-400/40 text-amber-200 px-0.5 rounded">{valParts.match}</mark>{valParts.after}
        </span>
      {:else}
        {#if valType === 'string'}
          <span class="text-emerald-400 break-all">{formatPrimitiveValue(value, valType)}</span>
        {:else if valType === 'number'}
          <span class="text-sky-400 font-semibold">{formatPrimitiveValue(value, valType)}</span>
        {:else if valType === 'boolean'}
          <span class="text-amber-400 font-bold">{formatPrimitiveValue(value, valType)}</span>
        {:else if valType === 'null'}
          <span class="text-apple-ink-muted-48 italic font-semibold">{formatPrimitiveValue(value, valType)}</span>
        {:else}
          <span class="text-apple-body-on-dark">{formatPrimitiveValue(value, valType)}</span>
        {/if}
      {/if}

      {#if !isLast}
        <span class="text-apple-ink-muted-48">,</span>
      {/if}
    </div>
  {/if}
</div>
