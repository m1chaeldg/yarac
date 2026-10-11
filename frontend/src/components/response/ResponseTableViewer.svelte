<script lang="ts">
  import type { FlatRow } from '../../utils/responseUtils';

  interface Props {
    columns: string[];
    rows: FlatRow[];
  }

  let { columns = [], rows = [] }: Props = $props();

  let tableSearch = $state('');
  let columnFilter = $state<string>('');
  let sortColumn = $state<string | null>(null);
  let sortDir = $state<'asc' | 'desc' | null>(null);

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
    let result = [...rows];
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
    if (!columnFilter) return columns;
    return columns.filter(c => c === '#' || c === columnFilter);
  });
</script>

<div class="h-full flex flex-col bg-slate-950 overflow-hidden text-xs">
  <!-- Table Search & Filter Bar -->
  <div class="h-9 border-b border-slate-800 px-3 bg-slate-900/60 flex items-center justify-between gap-3 shrink-0">
    <div class="flex items-center gap-2 flex-1 max-w-sm">
      <input
        type="text"
        bind:value={tableSearch}
        placeholder="Filter table rows..."
        class="w-full bg-slate-950 border border-slate-800 rounded px-2 py-1 text-slate-200 text-xs focus:outline-none focus:border-sky-500 font-mono"
      />
    </div>

    <div class="flex items-center gap-2">
      <span class="text-slate-500 text-[11px]">Column:</span>
      <select
        bind:value={columnFilter}
        class="bg-slate-950 border border-slate-800 rounded px-2 py-1 text-slate-300 text-xs focus:outline-none focus:border-sky-500"
      >
        <option value="">All Columns ({columns.length})</option>
        {#each columns.filter(c => c !== '#') as col}
          <option value={col}>{col}</option>
        {/each}
      </select>
      <span class="text-slate-500 text-[11px] font-mono">
        {filteredRows.length} {filteredRows.length === 1 ? 'row' : 'rows'}
      </span>
    </div>
  </div>

  <!-- Table Viewport -->
  <div class="flex-1 overflow-auto">
    {#if filteredRows.length === 0}
      <div class="p-8 text-center text-slate-500 italic">No matching rows found</div>
    {:else}
      <table class="w-full text-left border-collapse">
        <thead class="sticky top-0 bg-slate-900 border-b border-slate-800 text-[11px] font-semibold text-slate-400 z-10 shadow-sm">
          <tr>
            {#each displayColumns as col}
              <th
                class="px-3 py-2 cursor-pointer hover:bg-slate-800 transition-colors whitespace-nowrap select-none"
                onclick={() => toggleSort(col)}
              >
                <div class="flex items-center gap-1.5">
                  <span class="font-mono">{col}</span>
                  {#if sortColumn === col}
                    <span class="text-sky-400">{sortDir === 'asc' ? '▲' : '▼'}</span>
                  {/if}
                </div>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-mono text-[11px]">
          {#each filteredRows as row, i}
            <tr class="hover:bg-slate-900/40 transition-colors {i % 2 === 0 ? 'bg-slate-950' : 'bg-slate-900/20'}">
              {#each displayColumns as col}
                <td class="px-3 py-1.5 text-slate-300 whitespace-nowrap max-w-xs truncate" title={String(row[col] ?? '')}>
                  {#if col === '#'}
                    <span class="text-slate-500 font-semibold">{row[col]}</span>
                  {:else if typeof row[col] === 'boolean'}
                    <span class={row[col] ? 'text-emerald-400 font-semibold' : 'text-rose-400'}>{String(row[col])}</span>
                  {:else if row[col] === null || row[col] === undefined}
                    <span class="text-slate-600 italic">null</span>
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
