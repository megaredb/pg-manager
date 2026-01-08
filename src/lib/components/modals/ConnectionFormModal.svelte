<script lang="ts">
  import { onMount } from "svelte";
  import type { Connection } from "$lib/stores/connections";
  import { createConnection, updateConnection } from "$lib/stores/connections";
  import { folders, type ConnectionFolder } from "$lib/stores/folders";
  import { authState } from "$lib/stores/auth.svelte";
  import DatabaseIcon from "$lib/components/icons/DatabaseIcon.svelte";

  interface Props {
    isOpen: boolean;
    connection: Connection | null;
    onClose: () => void;
    onSuccess: () => void;
  }

  let {
    isOpen = $bindable(),
    connection,
    onClose,
    onSuccess,
  }: Props = $props();

  let formData = $state({
    connection_name: "",
    host: "",
    port: 5432,
    db_name: "",
    db_user: "",
    db_password_encrypted: "",
    ssl_mode: "prefer",
    folder_id: null as number | null,
  });

  let isSubmitting = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (connection) {
      formData = {
        connection_name: connection.connection_name,
        host: connection.host,
        port: connection.port || 5432,
        db_name: connection.db_name,
        db_user: connection.db_user,
        db_password_encrypted: "",
        ssl_mode: connection.ssl_mode || "prefer",
        folder_id: connection.folder_id || null,
      };
    } else {
      formData = {
        connection_name: "",
        host: "",
        port: 5432,
        db_name: "",
        db_user: "",
        db_password_encrypted: "",
        ssl_mode: "prefer",
        folder_id: null,
      };
    }
    error = null;
  });

  async function handleSubmit() {
    if (isSubmitting) return;

    if (!connection && !formData.db_password_encrypted) {
      error = "Password is required for new connections";
      return;
    }

    error = null;
    isSubmitting = true;

    try {
      const basePayload = {
        ...formData,
        user_id: authState.user?.user_id || 1,
      };

      if (connection) {
        await updateConnection(connection.connection_id, basePayload);
      } else {
        await createConnection(basePayload);
      }

      onSuccess();
      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to save connection";
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div class="modal-box max-w-2xl">
      <h3 class="font-bold text-lg flex items-center gap-2 mb-4">
        <DatabaseIcon class="h-5 w-5" />
        {connection ? "Edit Connection" : "New Connection"}
      </h3>

      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleSubmit();
        }}
      >
        <div class="grid grid-cols-2 gap-4">
          <!-- Connection Name -->
          <div class="form-control col-span-2">
            <label class="label">
              <span class="label-text">Connection Name *</span>
            </label>
            <input
              type="text"
              bind:value={formData.connection_name}
              class="input input-bordered"
              required
              placeholder="My PostgreSQL Server"
            />
          </div>

          <!-- Host -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">Host *</span>
            </label>
            <input
              type="text"
              bind:value={formData.host}
              class="input input-bordered"
              required
              placeholder="localhost"
            />
          </div>

          <!-- Port -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">Port</span>
            </label>
            <input
              type="number"
              bind:value={formData.port}
              class="input input-bordered"
              placeholder="5432"
            />
          </div>

          <!-- Database Name -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">Database *</span>
            </label>
            <input
              type="text"
              bind:value={formData.db_name}
              class="input input-bordered"
              required
              placeholder="postgres"
            />
          </div>

          <!-- User -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">User *</span>
            </label>
            <input
              type="text"
              bind:value={formData.db_user}
              class="input input-bordered"
              required
              placeholder="postgres"
            />
          </div>

          <!-- Password -->
          <div class="form-control col-span-2">
            <label class="label">
              <span class="label-text">Password {connection ? "" : "*"}</span>
              {#if connection}
                <span class="label-text-alt text-base-content/50"
                  >(Leave blank for current)</span
                >
              {/if}
            </label>
            <input
              type="password"
              bind:value={formData.db_password_encrypted}
              class="input input-bordered"
              required={!connection}
              placeholder={connection ? "••••••••" : "••••••••"}
            />
          </div>

          <!-- SSL Mode -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">SSL Mode</span>
            </label>
            <select
              bind:value={formData.ssl_mode}
              class="select select-bordered"
            >
              <option value="disable">Disable</option>
              <option value="prefer">Prefer</option>
              <option value="require">Require</option>
            </select>
          </div>

          <!-- Folder -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">Folder</span>
            </label>
            <select
              bind:value={formData.folder_id}
              class="select select-bordered"
            >
              <option value={null}>No Folder</option>
              {#each $folders as folder}
                <option value={folder.folder_id}>{folder.folder_name}</option>
              {/each}
            </select>
          </div>
        </div>

        {#if error}
          <div class="alert alert-error mt-4">
            <span>{error}</span>
          </div>
        {/if}

        <div class="modal-action">
          <button type="button" class="btn btn-ghost" onclick={onClose}>
            Cancel
          </button>
          <button type="submit" class="btn btn-primary" disabled={isSubmitting}>
            {isSubmitting ? "Saving..." : connection ? "Update" : "Create"}
          </button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" onclick={onClose}></div>
  </div>
{/if}
