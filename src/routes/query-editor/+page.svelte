<script lang="ts">
  import CodeMirror from "svelte-codemirror-editor";
  import { sql } from "@codemirror/lang-sql";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { DB } from "$lib/values";
  import type { QueryResult } from "@tauri-apps/plugin-sql";
  import { browser } from "$app/environment";

  interface SelectResult {
    type: "select";
    query: string;
    cols: string[];
    rows: any[];
  }

  interface ExecuteResult {
    type: "execute";
    query: string;
    message: string;
  }

  interface ErrorResult {
    type: "error";
    query: string;
    message: string;
  }

  type ExecutionResult = SelectResult | ExecuteResult | ErrorResult;

  const EDITOR_STORAGE_KEY = "sqlEditorContent";
  const DEFAULT_SQL_VALUE = `-- Your SQL script here...`;
  const DEFAULT_SQL_SELECT = `SELECT * FROM your_table LIMIT 10;`;
  const DEFAULT_SQL_INSERT = `INSERT INTO your_table (col1, col2) VALUES (1, 2);`;
  const DEFAULT_SQL_UPDATE = `UPDATE your_table SET col1 = 1 WHERE id = 1;`;
  const DEFAULT_SQL_DELETE = `DELETE FROM your_table WHERE id = 1;`;

  const DATA_QUERY_KEYWORDS = ["select", "pragma", "explain", "values", "with"];

  function parseSqlScript(script: string): string[] {
    const noBlockComments = script.replace(/\/\*[\s\S]*?\*\//g, "");
    const noLineComments = noBlockComments.replace(/\-\-.*$/gm, "");

    return noLineComments
      .split(";")
      .map((q) => q.trim())
      .filter((q) => q.length > 0);
  }

  function isDataQuery(query: string): boolean {
    const normalizedQuery = query.toLowerCase().trim();
    return DATA_QUERY_KEYWORDS.some((keyword) =>
      normalizedQuery.startsWith(keyword)
    );
  }

  let sqlEditorValue = $state(
    (browser && localStorage.getItem(EDITOR_STORAGE_KEY)) || DEFAULT_SQL_VALUE
  );
  let executionResults = $state<ExecutionResult[]>([]);
  let isLoading = $state(false);

  $effect(() => {
    if (browser) {
      localStorage.setItem(EDITOR_STORAGE_KEY, sqlEditorValue);
    }
  });

  async function executeSql() {
    isLoading = true;
    executionResults = [];

    const queries = parseSqlScript(sqlEditorValue);

    if (queries.length === 0) {
      executionResults = [
        {
          type: "error",
          query: "N/A",
          message: "Error: No SQL queries found.",
        },
      ];
      isLoading = false;
      return;
    }

    const results: ExecutionResult[] = [];

    for (const query of queries) {
      try {
        if (isDataQuery(query)) {
          const rows: any[] = await DB.select(query);
          const cols = rows.length > 0 ? Object.keys(rows[0]) : [];
          results.push({
            type: "select",
            query,
            cols,
            rows,
          });
        } else {
          const result: QueryResult = await DB.execute(query);
          results.push({
            type: "execute",
            query,
            message: `✅ Success: Query executed. Rows affected: ${result.rowsAffected}`,
          });
        }
      } catch (error: any) {
        console.error("SQL Error:", error);
        results.push({
          type: "error",
          query,
          message: `❌ Error: ${error.message || error}`,
        });

        break;
      }
    }

    executionResults = results;
    isLoading = false;
  }
</script>

<div class="flex flex-col h-screen overflow-hidden bg-base-300">
  <div class="flex m-2 items-center gap-2">
    <button
      class="btn btn-primary btn-xs"
      onclick={executeSql}
      disabled={isLoading}
    >
      ▶ Execute
    </button>
    {#if isLoading}
      <span class="loading loading-spinner loading-sm"></span>
    {/if}
    <button
      class="btn btn-primary btn-xs"
      onclick={() => (sqlEditorValue = DEFAULT_SQL_VALUE)}
    >
      📝 Clear
    </button>
    <div class="divider divider-horizontal"></div>
    <div class="flex flex-wrap gap-2">
      {#each Object.entries( { SELECT: DEFAULT_SQL_SELECT, INSERT: DEFAULT_SQL_INSERT, UPDATE: DEFAULT_SQL_UPDATE, DELETE: DEFAULT_SQL_DELETE } ) as [type, query]}
        <button
          class={`btn btn-soft btn-xs`}
          onclick={() => (sqlEditorValue = query)}
        >
          {type}
        </button>
      {/each}
    </div>
  </div>

  <div class="p-2 flex overflow-hidden w-full h-1/2 gap-2">
    <div
      class="w-full flex flex-col h-full overflow-hidden rounded-lg bg-base-100"
    >
      <h1 class="badge badge-primary badge-xs m-1">SQL-Editor</h1>
      <div class="flex-1 overflow-auto">
        <CodeMirror
          bind:value={sqlEditorValue}
          lang={sql()}
          theme={oneDark}
          class="h-full"
        />
      </div>
    </div>
  </div>

  <div class="flex-1 p-2 overflow-auto space-y-4">
    {#if executionResults.length === 0}
      <div class="p-4 text-center text-base-content/60">No results.</div>
    {/if}

    {#each executionResults as result (result.query + result.type)}
      <div class="collapse collapse-arrow bg-base-100 shadow-md">
        <input
          type="checkbox"
          checked={result.type === "error" ||
            (result.type === "select" && result.rows.length > 0)}
        />

        <div class="collapse-title font-mono text-sm font-medium">
          {#if result.type === "error"}
            <span class="badge badge-error badge-xs mr-2">ERROR</span>
          {:else if result.type === "execute"}
            <span class="badge badge-success badge-xs mr-2">EXECUTE</span>
          {:else if result.type === "select"}
            <span class="badge badge-info badge-xs mr-2">SELECT</span>
          {/if}
          <span class="opacity-70 whitespace-pre-wrap">{result.query}</span>
        </div>

        <div class="collapse-content bg-base-200/50">
          {#if result.type === "error"}
            <div class="alert alert-error">
              <pre class="whitespace-pre-wrap">{result.message}</pre>
            </div>
          {:else if result.type === "execute"}
            <div class="alert alert-success">
              <span>{result.message}</span>
            </div>
          {:else if result.type === "select"}
            {#if result.rows.length > 0}
              <p class="text-xs mb-2">Recieved {result.rows.length} rows.</p>
              <div class="overflow-x-auto max-h-64">
                <table class="table table-xs table-pin-rows table-pin-cols">
                  <thead>
                    <tr>
                      {#each result.cols as colName}
                        <th>{colName}</th>
                      {/each}
                    </tr>
                  </thead>
                  <tbody>
                    {#each result.rows as row}
                      <tr>
                        {#each result.cols as colName}
                          <td>{row[colName]}</td>
                        {/each}
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {:else}
              <div class="alert alert-info">
                <span>ℹ️ No rows found.</span>
              </div>
            {/if}
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>
