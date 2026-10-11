<script lang="ts">
  import {
    parseTableData,
    type FlatRow,
    type ObjectViewMode
  } from '../../utils/responseUtils';

  interface Props {
    parsedJson?: any;
    columns?: string[];
    rows?: FlatRow[];
  }

  let { parsedJson = null, columns = [], rows = [] }: Props = $props();

  let selectedCollectionId = $state('root');
  let objectMode = $state<ObjectViewMode>('properties');
  let tableSearch = $state('');
  let columnFilter = $state<string>('');
  let sortColumn = $state<string | null>(null);
  let sortDir = $state<'asc' | 'desc' | null>(null);

  // Derive full table computation based on user choice
  let computedData = $derived.by(() => {
    if (parsedJson) {
      return parseTableData(parsedJson, selectedCollectionId, objectMode);
    }
    return {
      columns,
      rows,
      collections: [],
      isObject: false,
      selectedCollectionId: 'root'
    };
  });

  let activeColumns = $derived(computedData.columns);
  let activeRows = $derived(computedData.rows);
  let availableCollections = $derived(computedData.collections);
  let isTargetObject = $derived(computedData.isObject);

  function toggleSort(col: string) {
    if (sortColumn === col) {
      if (sortDir === 'asc') sortDir = 'desc';
      else if (sortDir === 'desc') {
        sortColumn = null;
        sortDir = null;
      }
    } else {
      sortColumn = col;
      sortDir = 'asc';
    }
  }

  let filteredRows = $derived.by(() => {
    let result = [...activeRows];
    if (tableSearch.trim()) {
      const q = tableSearch.toLowerCase();
      result = result.filter(r =>
        Object.values(r).some(val => String(val).toLowerCase().includes(q))
      );
    }
    if (sortColumn && sortDir) {
      result.sort((a, b) => {
        const valA = a[sortColumn!] ?? '';
        const valB = b[sortColumn!] ?? '';
        if (typeof valA === 'number' && typeof valB === 'number') {
          return sortDir === 'asc' ? valA - valB : valB - valA;
        }
        const strA = String(valA);
        const strB = String(valB);
        return sortDir === 'asc' ? strA.localeCompare(strB) : strB.localeCompare(strA);
      });
    }
    return result;
  });

  let displayColumns = $derived.by(() => {
    if (!columnFilter) return activeColumns;
    return activeColumns.filter(c => c === '#' || c === columnFilter);
  });

  function getTypeBadgeColor(type: string) {
    switch (type) {
      case 'string':
        return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
      case 'number':
        return 'bg-sky-500/10 text-sky-400 border-sky-500/20';
      case 'boolean':
        return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
      case 'array':
        return 'bg-purple-500/10 text-purple-400 border-purple-500/20';
      case 'object':
        return 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20';
      case 'null':
        return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
      default:
        return 'bg-apple-surface-tile-3 text-apple-body-muted border-apple-hairline-dark/60';
    }
  }
</script>

<div class="h-full flex flex-col bg-apple-surface-black overflow-hidden text-xs">
  <!-- Table Controls Toolbar -->
  <div class="border-b border-apple-hairline-dark/60 px-3 py-1.5 bg-apple-surface-tile-2 flex flex-wrap items-center justify-between gap-2.5 shrink-0">
    <div class="flex items-center gap-2 flex-1 min-w-[220px] max-w-sm">
      <input
        type="text"
        bind:value={tableSearch}
        placeholder="Filter table rows..."
        class="w-full bg-apple-surface-black border border-apple-hairline-dark/80 rounded-apple-pill px-3 py-1 text-apple-body-on-dark text-xs focus:outline-none focus:border-apple-primary font-mono placeholder:text-apple-ink-muted-48"
      />
    </div>

    <div class="flex items-center flex-wrap gap-2">
      <!-- Collection / Dataset Selector (if payload has embedded arrays or multiple options) -->
      {#if availableCollections.length > 1}
        <div class="flex items-center gap-1.5 bg-apple-surface-tile-3 px-2 py-0.5 rounded-apple-sm border border-apple-hairline-dark/60">
          <span class="text-apple-body-muted text-[11px] font-medium">Dataset:</span>
          <select
            bind:value={selectedCollectionId}
            class="bg-transparent text-apple-body-on-dark text-xs font-mono focus:outline-none cursor-pointer"
          >
            {#each availableCollections as col}
              <option value={col.id} class="bg-apple-surface-tile-2 text-white">
                {col.label}
              </option>
            {/each}
          </select>
        </div>
      {/if}

      <!-- Object View Mode Toggle (Properties vs Single Record Row) -->
      {#if isTargetObject}
        <div class="flex items-center bg-apple-surface-tile-3 rounded-apple-sm border border-apple-hairline-dark/60 p-0.5">
          <button
            type="button"
            onclick={() => objectMode = 'properties'}
            class="px-2 py-0.5 rounded-apple-xs text-[11px] transition-colors cursor-pointer {objectMode === 'properties' ? 'bg-apple-primary text-apple-primary-on-dark font-semibold shadow-apple-inset' : 'text-apple-body-muted hover:text-white'}"
            title="Show as Key, Type, and Value list"
          >
            Properties
          </button>
          <button
            type="button"
            onclick={() => objectMode = 'record'}
            class="px-2 py-0.5 rounded-apple-xs text-[11px] transition-colors cursor-pointer {objectMode === 'record' ? 'bg-apple-primary text-apple-primary-on-dark font-semibold shadow-apple-inset' : 'text-apple-body-muted hover:text-white'}"
            title="Show as a single multi-column row"
          >
            Record Row
          </button>
        </div>
      {/if}

      <!-- Column Filter -->
      <div class="flex items-center gap-1.5">
        <span class="text-apple-body-muted text-[11px]">Column:</span>
        <select
          bind:value={columnFilter}
          class="bg-apple-surface-tile-3 border border-apple-hairline-dark/80 rounded-apple-sm px-2 py-1 text-apple-body-on-dark text-xs focus:outline-none focus:border-apple-primary"
        >
          <option value="" class="bg-apple-surface-tile-2 text-white">All Columns ({activeColumns.length})</option>
          {#each activeColumns.filter(c => c !== '#') as col}
            <option value={col} class="bg-apple-surface-tile-2 text-white">{col}</option>
          {/each}
        </select>
      </div>

      <span class="text-apple-ink-muted-48 text-[11px] font-mono whitespace-nowrap">
        {filteredRows.length} {filteredRows.length === 1 ? 'row' : 'rows'}
      </span>
    </div>
  </div>

  <!-- Table Viewport -->
  <div class="flex-1 overflow-auto">
    {#if filteredRows.length === 0}
      <div class="p-8 text-center text-apple-ink-muted-48 italic">No matching rows found</div>
    {:else}
      <table class="w-full text-left border-collapse">
        <thead class="sticky top-0 bg-apple-surface-tile-2 border-b border-apple-hairline-dark/60 text-[11px] font-semibold text-apple-body-muted z-10 shadow-none">
          <tr>
            {#each displayColumns as col}
              <th
                class="px-3 py-2 cursor-pointer hover:bg-apple-surface-tile-3 transition-colors whitespace-nowrap select-none"
                onclick={() => toggleSort(col)}
              >
                <div class="flex items-center gap-1.5">
                  <span class="font-mono">{col}</span>
                  {#if sortColumn === col}
                    <span class="text-apple-primary-on-dark">{sortDir === 'asc' ? '▲' : '▼'}</span>
                  {/if}
                </div>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody class="divide-y divide-apple-hairline-dark/40 font-mono text-[11px]">
          {#each filteredRows as row, i}
            <tr class="hover:bg-apple-surface-tile-1/50 transition-colors {i % 2 === 0 ? 'bg-apple-surface-black' : 'bg-apple-surface-tile-1/30'}">
              {#each displayColumns as col}
                <td class="px-3 py-1.5 text-apple-body-on-dark whitespace-nowrap max-w-md truncate" title={String(row[col] ?? '')}>
                  {#if col === '#'}
                    <span class="text-apple-body-muted font-semibold">{row[col]}</span>
                  {:else if col === 'Type' && isTargetObject && objectMode === 'properties'}
                    <span class="px-1.5 py-0.5 rounded-apple-pill text-[10px] font-semibold border {getTypeBadgeColor(String(row[col]))}">
                      {row[col]}
                    </span>
                  {:else if typeof row[col] === 'boolean'}
                    <span class={row[col] ? 'text-emerald-400 font-semibold' : 'text-rose-400'}>{String(row[col])}</span>
                  {:else if row[col] === null || row[col] === undefined}
                    <span class="text-apple-ink-muted-48 italic">null</span>
                  {:else if typeof row[col] === 'string' && row[col].startsWith('[') && row[col].endsWith('items]')}
                    <span class="px-1.5 py-0.5 rounded-apple-pill bg-purple-500/10 text-purple-400 border border-purple-500/20 text-[10px]">
                      {row[col]}
                    </span>
                  {:else}
                    {row[col]}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

