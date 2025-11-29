<script lang="ts">
  import { DB } from "$lib/values";
  import { onMount } from "svelte";

  interface Folder {
    connections_folder_id: number;
    user_id: number;
    folder_name: string;
  }

  interface Connection {
    connection_id: number;
    connection_name: string;
    host: string;
    port: number;
    db_name: string;
    db_user: string;
    db_password_encrypted: string;
    ssl_mode: string;
  }

  let folders: Folder[] = $state([]);
  let connections: Connection[] = $state([]);
  let selectedFolderId: number | null = $state(null);
  let isLoadingConnections = $state(false);

  onMount(async () => {
    folders = (await DB.select("SELECT * FROM connection_folders")) as Folder[];

    if (folders.length > 0) {
      await selectFolder(folders[0].connections_folder_id);
    }
  });

  async function selectFolder(folderId: number) {
    if (selectedFolderId === folderId) return;

    selectedFolderId = folderId;
    isLoadingConnections = true;

    try {
      connections = (await DB.select(
        "SELECT connection_id, connection_name, host, port, db_name, db_user, db_password_encrypted, ssl_mode FROM connections WHERE connections_folder_id = ?",
        [folderId]
      )) as Connection[];
    } catch (e) {
      console.error("Error loading connections:", e);
      connections = [];
    } finally {
      isLoadingConnections = false;
    }
  }
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Folders & Connections</h1>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <div class="md:col-span-1">
      <h2 class="text-xl font-semibold mb-2">Folders</h2>
      <ul class="menu bg-base-200 rounded-box">
        {#each folders as folder}
          <li>
            <!-- svelte-ignore a11y_invalid_attribute -->
            <a
              href="#"
              class:active={selectedFolderId === folder.connections_folder_id}
              onclick={(event) => {
                event.preventDefault();
                selectFolder(folder.connections_folder_id);
              }}
            >
              {folder.folder_name}
              <span class="badge badge-sm">User {folder.user_id}</span>
            </a>
          </li>
        {/each}
      </ul>
    </div>

    <div class="md:col-span-2">
      <h2 class="text-xl font-semibold mb-2">Connections</h2>
      {#if isLoadingConnections}
        <div class="flex justify-center items-center h-32">
          <span class="loading loading-lg loading-spinner"></span>
        </div>
      {:else if !selectedFolderId}
        <div class="alert alert-info">
          <span>Select a folder.</span>
        </div>
      {:else if connections.length === 0}
        <div class="alert">
          <span>No connections found.</span>
        </div>
      {:else}
        <div class="overflow-x-auto">
          <table class="table table-sm table-zebra bg-base-200">
            <thead>
              <tr>
                <th>ID</th>
                <th>Name</th>
                <th>Host</th>
                <th>Port</th>
                <th>DB Name</th>
                <th>DB User</th>
                <th>Encrypted Password</th>
                <th>SSL Mode</th>
              </tr>
            </thead>
            <tbody>
              {#each connections as conn}
                <tr>
                  <td>{conn.connection_id}</td>
                  <td>{conn.connection_name}</td>
                  <td>{conn.host}</td>
                  <td>{conn.port}</td>
                  <td>{conn.db_name}</td>
                  <td>{conn.db_user}</td>
                  <td>{conn.db_password_encrypted}</td>
                  <td>{conn.ssl_mode}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>
</div>
