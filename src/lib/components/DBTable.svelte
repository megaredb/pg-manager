<script lang="ts">
  import { DB } from "$lib/values";

  interface Props {
    tableName: string;
  }

  interface Column {
    cid: number;
    name: string;
    type: string;
    notnull: number;
    dflt_value: string | null;
    pk: number;
  }

  interface ForeignKey {
    id: number;
    seq: number;
    table: string;
    from: string;
    to: string;
    on_update: string;
    on_delete: string;
    match: string;
  }

  interface ForeignKeyOptions {
    [columnName: string]: {
      refTable: string;
      refColumn: string;
      options: Array<{ value: any; label: string }>;
    };
  }

  interface TableRow {
    _tempId: string;
    _isNew?: boolean;
    [key: string]: any;
  }

  let { tableName }: Props = $props();

  let columns: Column[] = $state([]);
  let rows: TableRow[] = $state([]);
  let editedRows: Set<string> = $state(new Set());
  let deletedRows: Set<string> = $state(new Set());
  let pkColumn: string | null = $state(null);
  let foreignKeys: ForeignKey[] = $state([]);
  let fkOptions: ForeignKeyOptions = $state({});

  loadData();

  async function loadData(): Promise<void> {
    await loadColumns();
    await loadForeignKeys();
    await loadRows();
    resetChangeTracking();
  }

  async function loadColumns(): Promise<void> {
    columns = (await DB.select(`PRAGMA table_info(${tableName})`)) as Column[];
    const primaryKey = columns.find((col) => col.pk === 1);
    pkColumn = primaryKey ? primaryKey.name : null;
  }

  async function loadForeignKeys(): Promise<void> {
    try {
      foreignKeys = (await DB.select(
        `PRAGMA foreign_key_list(${tableName})`
      )) as ForeignKey[];

      for (const fk of foreignKeys) {
        await loadForeignKeyOptions(fk);
      }
    } catch (error) {
      console.error("Error loading foreign keys:", error);
    }
  }

  async function loadForeignKeyOptions(fk: ForeignKey): Promise<void> {
    const { table: refTable, to: refColumn, from: fromColumn } = fk;

    const refRows = (await DB.select(
      `SELECT ${refColumn}, * FROM ${refTable}`
    )) as any[];

    const refTableInfo = (await DB.select(
      `PRAGMA table_info(${refTable})`
    )) as Column[];

    const displayColumn = findDisplayColumn(refTableInfo, refColumn);

    fkOptions[fromColumn] = {
      refTable,
      refColumn,
      options: refRows.map((row) => ({
        value: row[refColumn],
        label: formatForeignKeyLabel(row, refColumn, displayColumn),
      })),
    };
  }

  function findDisplayColumn(tableInfo: Column[], refColumn: string): string {
    const textColumn = tableInfo.find(
      (col) => !col.pk && col.type.includes("TEXT")
    );
    return textColumn?.name || refColumn;
  }

  function formatForeignKeyLabel(
    row: any,
    refColumn: string,
    displayColumn: string
  ): string {
    if (displayColumn === refColumn) {
      return String(row[refColumn]);
    }
    return `${row[refColumn]} - ${row[displayColumn]}`;
  }

  async function loadRows(): Promise<void> {
    const data = (await DB.select(`SELECT * FROM ${tableName}`)) as any[];
    rows = data.map((row, index) => ({
      ...row,
      _tempId: `existing_${index}`,
    }));
  }

  function resetChangeTracking(): void {
    editedRows = new Set();
    deletedRows = new Set();
  }

  function isForeignKey(columnName: string): boolean {
    return foreignKeys.some((fk) => fk.from === columnName);
  }

  function getFKInfo(columnName: string) {
    return fkOptions[columnName];
  }

  function parseDefaultValue(dflt_value: string | null, type: string): any {
    if (dflt_value === null) return null;

    if (
      (dflt_value.startsWith("'") && dflt_value.endsWith("'")) ||
      (dflt_value.startsWith('"') && dflt_value.endsWith('"'))
    ) {
      return dflt_value.slice(1, -1);
    }

    if (type === "INTEGER" || type === "REAL") {
      const num = parseFloat(dflt_value);
      return isNaN(num) ? null : num;
    }

    return dflt_value;
  }

  function hasUnsavedChanges(): boolean {
    return (
      editedRows.size > 0 ||
      deletedRows.size > 0 ||
      rows.some((row) => row._isNew)
    );
  }

  function getCellClass(row: TableRow): string {
    if (deletedRows.has(row._tempId)) return "bg-error/20 line-through";
    if (row._isNew) return "bg-success/20";
    if (editedRows.has(row._tempId)) return "bg-warning/20";
    return "";
  }

  function createNewRow(): TableRow {
    const newRow: TableRow = {
      _tempId: `new_${Date.now()}`,
      _isNew: true,
    };

    columns.forEach((col) => {
      newRow[col.name] = getDefaultValueForColumn(col);
    });

    return newRow;
  }

  function getDefaultValueForColumn(col: Column): any {
    if (col.pk && col.type === "INTEGER") {
      return null;
    }

    if (col.dflt_value !== null) {
      return parseDefaultValue(col.dflt_value, col.type);
    }

    if (isForeignKey(col.name)) {
      const fkInfo = getFKInfo(col.name);
      return fkInfo?.options[0]?.value ?? null;
    }

    if (col.type === "INTEGER") return 0;
    if (col.type === "REAL") return 0.0;
    return "";
  }

  function markAsEdited(tempId: string): void {
    editedRows = new Set(editedRows.add(tempId));
  }

  function onAddClick(): void {
    const newRow = createNewRow();
    rows = [...rows, newRow];
  }

  function onDeleteClick(row: TableRow): void {
    if (row._isNew) {
      deletedRows.delete(row._tempId);
      rows = rows.filter((r) => r._tempId !== row._tempId);
      deletedRows = new Set(deletedRows);
      return;
    }

    const newSet = new Set(deletedRows);
    if (newSet.has(row._tempId)) {
      newSet.delete(row._tempId);
    } else {
      newSet.add(row._tempId);
    }
    deletedRows = newSet;
  }

  function onRefreshClick(): void {
    if (hasUnsavedChanges()) {
      const confirmed = window.confirm(
        "You have unsaved changes. Are you sure you want to refresh?"
      );
      if (!confirmed) return;
    }

    loadData();
  }

  async function onSaveClick(): Promise<void> {
    try {
      await DB.execute("PRAGMA foreign_keys = ON");

      await deleteRows();
      await insertNewRows();
      await updateEditedRows();

      await loadData();
      alert("Data saved successfully!");
    } catch (error) {
      console.error("Error saving data:", error);
      alert(`Error saving data: ${error}`);
    }
  }

  async function deleteRows(): Promise<void> {
    if (!pkColumn) return;

    for (const row of rows) {
      if (deletedRows.has(row._tempId) && !row._isNew) {
        await DB.execute(`DELETE FROM ${tableName} WHERE ${pkColumn} = ?`, [
          row[pkColumn],
        ]);
      }
    }
  }

  async function insertNewRows(): Promise<void> {
    for (const row of rows) {
      if (row._isNew && !deletedRows.has(row._tempId)) {
        await insertRow(row);
      }
    }
  }

  async function insertRow(row: TableRow): Promise<void> {
    const cols = columns
      .filter((col) => !(col.pk && col.type === "INTEGER"))
      .map((col) => col.name);

    const placeholders = cols.map(() => "?").join(", ");
    const values = cols.map((colName) => row[colName]);

    await DB.execute(
      `INSERT INTO ${tableName} (${cols.join(", ")}) VALUES (${placeholders})`,
      values
    );
  }

  async function updateEditedRows(): Promise<void> {
    if (!pkColumn) return;

    for (const row of rows) {
      if (
        !row._isNew &&
        editedRows.has(row._tempId) &&
        !deletedRows.has(row._tempId)
      ) {
        await updateRow(row);
      }
    }
  }

  async function updateRow(row: TableRow): Promise<void> {
    if (!pkColumn) return;

    const nonPkColumns = columns.filter((col) => !col.pk);
    const setClauses = nonPkColumns.map((col) => `${col.name} = ?`).join(", ");
    const values = [...nonPkColumns.map((col) => row[col.name]), row[pkColumn]];

    await DB.execute(
      `UPDATE ${tableName} SET ${setClauses} WHERE ${pkColumn} = ?`,
      values
    );
  }

  let editingRow: TableRow | null = $state(null);
  let editModal: HTMLDialogElement;

  function onEditClick(row: TableRow): void {
    editingRow = { ...row };
    editModal.showModal();
  }

  function onSaveModal(): void {
    if (!editingRow) return;

    const index = rows.findIndex((r) => r._tempId === editingRow!._tempId);

    if (index !== -1) {
      rows[index] = editingRow;
      markAsEdited(editingRow._tempId);
    }

    editModal.close();
  }
</script>

<div class="flex px-2 items-center gap-2 mb-2">
  <h1 class="text-2xl font-bold">{tableName}</h1>

  <button class="btn btn-primary btn-xs" onclick={onAddClick}> ✚ </button>

  <button class="btn btn-secondary btn-xs" onclick={onSaveClick}> 💾 </button>

  <button class="btn btn-primary btn-xs" onclick={onRefreshClick}> ↺ </button>

  {#if hasUnsavedChanges()}
    <span class="badge badge-warning">Unsaved changes</span>
  {/if}
</div>

<div class="overflow-x-auto pb-2.5">
  <table class="table table-xs table-pin-rows bg-base-200">
    <thead>
      <tr>
        {#each columns as column}
          <th>
            {#if column.pk}
              <span class="badge badge-sm badge-primary">PK</span>
            {/if}

            {#if isForeignKey(column.name)}
              <span class="badge badge-sm badge-accent">FK</span>
            {/if}

            {column.name}
            <span class="text-xs opacity-60">({column.type})</span>

            {#if isForeignKey(column.name)}
              <span class="text-xs opacity-60">
                → {getFKInfo(column.name)?.refTable}
              </span>
            {/if}
          </th>
        {/each}
        <th></th>
        <th></th>
      </tr>
    </thead>

    <tbody>
      {#each rows as row}
        <tr class={getCellClass(row)}>
          {#each columns as column}
            <td>
              {#if column.pk && column.type === "INTEGER"}
                <span class="text-sm opacity-70">
                  {row[column.name] ?? "AUTO"}
                </span>
              {:else if isForeignKey(column.name)}
                <select
                  class="select select-xs select-bordered w-full"
                  bind:value={row[column.name]}
                  onchange={() => markAsEdited(row._tempId)}
                  disabled={deletedRows.has(row._tempId)}
                >
                  {#each getFKInfo(column.name)?.options || [] as option}
                    <option value={option.value}>
                      {option.label}
                    </option>
                  {/each}
                </select>
              {:else if column.type === "INTEGER" || column.type === "REAL"}
                <input
                  type="number"
                  class="input input-xs input-bordered w-full"
                  bind:value={row[column.name]}
                  oninput={() => markAsEdited(row._tempId)}
                  step={column.type === "REAL" ? "0.01" : "1"}
                  disabled={deletedRows.has(row._tempId)}
                />
              {:else}
                <input
                  type="text"
                  class="input input-xs input-bordered w-full"
                  bind:value={row[column.name]}
                  oninput={() => markAsEdited(row._tempId)}
                  disabled={deletedRows.has(row._tempId)}
                />
              {/if}
            </td>
          {/each}

          <td>
            <button
              class="btn btn-xs"
              class:btn-error={!deletedRows.has(row._tempId)}
              class:btn-warning={deletedRows.has(row._tempId)}
              onclick={() => onDeleteClick(row)}
            >
              {deletedRows.has(row._tempId) ? "↶" : "🗑️"}
            </button>
          </td>
          <td>
            <button
              class="btn btn-xs btn-primary"
              onclick={() => onEditClick(row)}
            >
              ✏️
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<dialog id="edit_modal" class="modal" bind:this={editModal}>
  <div class="modal-box w-11/12 max-w-2xl">
    <h3 class="text-lg font-bold mb-4">Edit row</h3>

    {#if editingRow}
      <div class="flex flex-col gap-3">
        {#each columns as column}
          {@const fieldId = `edit_field_${column.name}`}

          <div class="form-control w-full">
            <label class="label py-1" for={fieldId}>
              <span class="label-text font-semibold">
                {column.name}
                {#if column.pk}<span class="badge badge-xs badge-primary ml-1"
                    >PK</span
                  >{/if}
                {#if isForeignKey(column.name)}<span
                    class="badge badge-xs badge-accent ml-1">FK</span
                  >{/if}
              </span>
              <span class="label-text-alt opacity-60">{column.type}</span>
            </label>

            {#if column.pk && column.type === "INTEGER"}
              <input
                id={fieldId}
                type="text"
                class="input input-bordered w-full bg-base-200 text-opacity-70"
                value={editingRow[column.name] ?? "AUTO"}
                readonly
                disabled
              />
            {:else if isForeignKey(column.name)}
              <select
                id={fieldId}
                class="select select-bordered w-full"
                bind:value={editingRow[column.name]}
              >
                {#each getFKInfo(column.name)?.options || [] as option}
                  <option value={option.value}>
                    {option.label}
                  </option>
                {/each}
              </select>
            {:else if column.type === "INTEGER" || column.type === "REAL"}
              <input
                id={fieldId}
                type="number"
                class="input input-bordered w-full"
                bind:value={editingRow[column.name]}
                step={column.type === "REAL" ? "0.01" : "1"}
              />
            {:else if column.type.includes("TEXT") || column.type.includes("BLOB")}
              <textarea
                id={fieldId}
                class="textarea textarea-bordered w-full"
                bind:value={editingRow[column.name]}
                rows="2"
              ></textarea>
            {:else}
              <input
                id={fieldId}
                type="text"
                class="input input-bordered w-full"
                bind:value={editingRow[column.name]}
              />
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <div class="modal-action">
      <button class="btn btn-primary" onclick={onSaveModal}>Save</button>

      <form method="dialog">
        <button class="btn">Cancel</button>
      </form>
    </div>
  </div>
</dialog>
