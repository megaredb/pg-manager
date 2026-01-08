<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeTextFile } from "@tauri-apps/plugin-fs";

  export interface QueryResultItem {
    type: "success" | "error";
    query: string;
    data?: {
      columns: string[];
      rows: any[][];
      execution_time_ms: number;
      row_count: number;
    };
    error?: string;
    timestamp: Date;
  }

  interface Props {
    results: QueryResultItem[];
    isExecuting: boolean;
  }

  let { results = [], isExecuting }: Props = $props();

  async function exportToCSV(
    data: { columns: string[]; rows: any[][] },
    queryIndex: number
  ) {
    const csvContent =
      data.columns.join(",") +
      "\n" +
      data.rows.map((row) => row.join(",")).join("\n");

    try {
      const filePath = await save({
        filters: [{ name: "CSV", extensions: ["csv"] }],
        defaultPath: `query_result_${queryIndex + 1}_${new Date().getTime()}.csv`,
      });
      if (filePath) {
        await writeTextFile(filePath, csvContent);
        alert("CSV saved!");
      }
    } catch (e) {
      console.error(e);
      alert("Failed to save CSV: " + e);
    }
  }
</script>

<div class="flex flex-col h-full w-full bg-base-100 overflow-hidden">
  <!-- STATUS BAR -->
  <div
    class="flex items-center justify-between p-2 border-b border-base-300 bg-base-200 text-xs min-h-[36px]"
  >
    <div class="flex items-center gap-2">
      {#if isExecuting}
        <span class="loading loading-spinner loading-xs text-primary"></span>
        <span class="font-medium">Executing script...</span>
      {:else if results.length > 0}
        <span class="font-bold">Completed</span>
        <span class="opacity-70">({results.length} queries processed)</span>
      {:else}
        <span class="opacity-50">Ready</span>
      {/if}
    </div>
  </div>

  <!-- SCROLLABLE LIST -->
  <div class="flex-1 overflow-y-auto p-2 space-y-2">
    {#if results.length === 0 && !isExecuting}
      <div
        class="flex flex-col items-center justify-center h-full text-base-content/30 gap-2"
      >
        <div class="text-4xl">📊</div>
        <div>Run a query to see results here</div>
      </div>
    {/if}

    {#each results as res, index (index)}
      <!-- COLLAPSE ITEM -->
      <div
        class="collapse collapse-arrow border border-base-300 bg-base-100 rounded-box shadow-sm"
      >
        <input
          type="checkbox"
          checked={res.type === "error" ||
            (res.data && res.data.rows.length > 0)}
        />

        <div
          class="collapse-title text-sm font-medium py-2 min-h-0 flex items-center gap-3 bg-base-200/50"
        >
          <!-- BADGE -->
          {#if res.type === "error"}
            <span class="badge badge-error badge-sm text-white shrink-0"
              >ERROR</span
            >
          {:else if res.data && res.data.columns.length > 0}
            <span class="badge badge-info badge-sm shrink-0">SELECT</span>
            <span class="text-xs opacity-70"
              >{res.data.row_count} rows in {res.data.execution_time_ms}ms</span
            >
          {:else}
            <span class="badge badge-success badge-sm shrink-0">EXECUTE</span>
            <span class="text-xs opacity-70"
              >Success in {res.data?.execution_time_ms}ms</span
            >
          {/if}

          <!-- QUERY PREVIEW -->
          <span class="truncate font-mono opacity-80" title={res.query}>
            {res.query}
          </span>

          <!-- EXPORT BUTTON -->
          {#if res.type === "success" && res.data && res.data.columns.length > 0}
            <button
              class="btn btn-ghost btn-xs ml-auto shrink-0 z-1000"
              title="Export to CSV"
              onclick={(e) => {
                e.stopPropagation();
                exportToCSV(res.data!, index);
              }}
            >
              📤
            </button>
          {/if}
        </div>

        <div class="collapse-content px-0 pb-0">
          <!-- ERROR BODY -->
          {#if res.type === "error"}
            <div
              class="p-4 text-error bg-error/10 font-mono text-xs whitespace-pre-wrap border-t border-error/20"
            >
              {res.error}
            </div>

            <!-- SUCCESS BODY -->
          {:else if res.data}
            {#if res.data.columns.length > 0}
              <!-- TABLE -->
              <div class="overflow-x-auto max-h-80 border-t border-base-300">
                <table class="table table-xs table-pin-rows">
                  <thead>
                    <tr class="bg-base-200">
                      <th class="w-10 bg-base-300/50 text-right opacity-50"
                        >#</th
                      >
                      {#each res.data.columns as col}
                        <th>{col}</th>
                      {/each}
                    </tr>
                  </thead>
                  <tbody>
                    {#each res.data.rows as row, rIndex}
                      <tr class="hover">
                        <td
                          class="text-right opacity-30 select-none bg-base-100"
                          >{rIndex + 1}</td
                        >
                        {#each row as cell}
                          <td
                            class="max-w-[200px] truncate"
                            title={String(cell)}
                          >
                            {#if cell === null}
                              <span class="opacity-30 italic">NULL</span>
                            {:else if typeof cell === "boolean"}
                              <span class={cell ? "text-success" : "text-error"}
                                >{String(cell)}</span
                              >
                            {:else}
                              {String(cell)}
                            {/if}
                          </td>
                        {/each}
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
              {#if res.data.rows.length === 0}
                <div class="p-2 text-xs opacity-50 text-center italic">
                  No rows returned
                </div>
              {/if}
            {:else}
              <!-- NON-SELECT SUCCESS MESSAGE -->
              <div
                class="p-4 text-success bg-success/5 border-t border-success/10 text-xs"
              >
                Query executed successfully. No data returned.
              </div>
            {/if}
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>
