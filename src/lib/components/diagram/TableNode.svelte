<script lang="ts">
  import { Handle, Position, type NodeProps } from "@xyflow/svelte";

  type $$Props = NodeProps & {
    data: {
      label: string;
      columns: { name: string; type: string; isPk: boolean }[];
    };
  };

  export let data: {
    label: string;
    columns: { name: string; type: string; isPk: boolean }[];
  };
</script>

<div
  class="card bg-base-100 shadow-xl border border-base-300 min-w-[200px] text-sm overflow-hidden"
>
  <div
    class="bg-primary/10 p-2 font-bold text-center border-b border-base-300 flex justify-between items-center"
  >
    <span>{data.label}</span>
    <span class="badge badge-xs badge-ghost">{data.columns.length}</span>
  </div>

  <div class="flex flex-col p-0">
    {#each data.columns as col}
      <div
        class="flex justify-between items-center px-3 py-1 border-b border-base-200 last:border-0 hover:bg-base-200/50 relative group"
      >
        <div class="flex items-center gap-2">
          {#if col.isPk}
            <span class="text-warning text-[10px]">🔑</span>
          {/if}
          <span class="font-medium">{col.name}</span>
        </div>
        <span class="text-[10px] opacity-50 font-mono">{col.type}</span>

        <Handle
          type="target"
          position={Position.Left}
          id={`${col.name}-target`}
          class="w-2! h-2! bg-primary/50!"
        />
        <Handle
          type="source"
          position={Position.Right}
          id={`${col.name}-source`}
          class="w-2! h-2! bg-secondary/50!"
        />
      </div>
    {/each}
  </div>
</div>
