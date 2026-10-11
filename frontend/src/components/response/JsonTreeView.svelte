<script lang="ts">
  import JsonTreeNode from './JsonTreeNode.svelte';

  interface Props {
    data: any;
    expandVersion?: number;
    collapseVersion?: number;
  }

  let { data, expandVersion = 0, collapseVersion = 0 }: Props = $props();
</script>

<div class="h-full overflow-auto p-3 font-mono text-xs">
  {#if data === undefined}
    <div class="p-6 text-center text-apple-ink-muted-48 italic">
      No JSON data to display
    </div>
  {:else}
    <div class="space-y-0.5">
      <JsonTreeNode
        value={data}
        depth={0}
        {expandVersion}
        {collapseVersion}
        defaultExpanded={true}
        isLast={true}
      />
    </div>
  {/if}
</div>
