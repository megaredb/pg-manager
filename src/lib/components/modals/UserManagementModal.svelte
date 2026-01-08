<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { authState, type AppUser } from "$lib/stores/auth.svelte";

  // Icons
  import AddIcon from "$lib/components/icons/AddIcon.svelte";
  import DeleteIcon from "$lib/components/icons/DeleteIcon.svelte";
  import EditIcon from "$lib/components/icons/EditIcon.svelte";
  import SearchIcon from "../icons/SearchIcon.svelte";
  import { highlightMatch } from "$lib/utils/highlight";

  interface Props {
    isOpen: boolean;
  }

  let { isOpen = $bindable() }: Props = $props();

  let users = $state<AppUser[]>([]);
  let isLoading = $state(false);

  // Search State
  let searchQuery = $state("");

  // Form State
  let isEditing = $state(false);
  let showForm = $state(false);
  let formError = $state<string | null>(null);

  let formData = $state({
    user_id: 0,
    username: "",
    password: "",
    role: "user",
  });

  async function loadUsers() {
    isLoading = true;
    try {
      users = await invoke<AppUser[]>("get_app_users", {
        usernameSearch: searchQuery.length > 0 ? searchQuery : null,
      });
    } catch (err) {
      console.error("Failed to load users", err);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      resetForm();
      searchQuery = "";
    }
  });

  $effect(() => {
    if (!isOpen) return;

    const query = searchQuery;

    const timeoutId = setTimeout(
      () => {
        loadUsers();
      },
      query === "" ? 0 : 300
    );

    return () => clearTimeout(timeoutId);
  });

  function resetForm() {
    showForm = false;
    isEditing = false;
    formError = null;
    formData = { user_id: 0, username: "", password: "", role: "user" };
  }

  function handleEdit(user: AppUser) {
    formData = {
      user_id: user.user_id,
      username: user.username,
      password: "",
      role: user.role || "user",
    };
    isEditing = true;
    showForm = true;
    formError = null;
  }

  function handleAdd() {
    resetForm();
    showForm = true;
  }

  async function handleDelete(userId: number) {
    if (userId === authState.user?.user_id) {
      alert("You cannot delete your own account.");
      return;
    }

    if (!(await confirm("Are you sure you want to delete this user?"))) return;

    try {
      await invoke("delete_app_user", { userId });
      await loadUsers();
      if (isEditing && formData.user_id === userId) {
        resetForm();
      }
    } catch (e) {
      console.error(e);
      alert("Failed to delete user");
    }
  }

  async function handleSubmit() {
    formError = null;

    // Validation
    if (!formData.username) {
      formError = "Username is required";
      return;
    }
    if (!isEditing && !formData.password) {
      formError = "Password is required for new users";
      return;
    }

    if (isEditing && formData.user_id === authState.user?.user_id) {
      if (formData.role !== "admin") {
        formError = "You cannot remove your own admin privileges.";
        return;
      }
    }

    try {
      if (isEditing) {
        // Update
        const payload = {
          user_id: formData.user_id,
          username: formData.username,
          role: formData.role,
          password: formData.password ? formData.password : null,
        };
        await invoke("update_app_user", { request: payload });
      } else {
        // Create
        const payload = {
          username: formData.username,
          password: formData.password,
          role: formData.role,
        };
        await invoke("create_app_user", { request: payload });
      }

      await loadUsers();
      resetForm();
    } catch (err) {
      console.error(err);
      formError = typeof err === "string" ? err : "Operation failed";
    }
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div class="modal-box w-11/12 max-w-4xl h-[600px] flex flex-col">
      <div class="flex justify-between items-center mb-4">
        <h3 class="font-bold text-lg">User Management</h3>
        <button
          class="btn btn-sm btn-circle btn-ghost"
          onclick={() => (isOpen = false)}>✕</button
        >
      </div>

      <div class="flex-1 flex gap-4 overflow-hidden">
        <div
          class="flex-1 flex flex-col overflow-hidden border border-base-300 rounded-box"
        >
          <!-- Toolbar with Search -->
          <div
            class="p-2 bg-base-200 border-b border-base-300 flex justify-between items-center gap-2"
          >
            <div class="relative flex-1 max-w-xs">
              <input
                type="text"
                placeholder="Search users..."
                bind:value={searchQuery}
                class="input input-xs input-bordered w-full pl-8"
              />
              <div
                class="absolute left-2 top-1/2 -translate-y-1/2 opacity-50 pointer-events-none"
              >
                <SearchIcon class="w-4 h-4" />
              </div>
            </div>

            <div class="flex items-center gap-2">
              <span class="text-xs opacity-60"
                >{users.length} user{users.length !== 1 ? "s" : ""}</span
              >
              <button class="btn btn-xs btn-primary gap-1" onclick={handleAdd}>
                <AddIcon class="w-3 h-3" /> Add
              </button>
            </div>
          </div>

          <div class="flex-1 overflow-y-auto p-0">
            <table class="table table-pin-rows table-xs">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Username</th>
                  <th>Role</th>
                  <th class="text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each users as u}
                  <tr
                    class="hover {isEditing && formData.user_id === u.user_id
                      ? 'bg-base-200'
                      : ''}"
                  >
                    <td>{u.user_id}</td>
                    <td class="font-medium">
                      {@html highlightMatch(u.username, searchQuery)}
                      {#if u.user_id === authState.user?.user_id}
                        <span class="text-[10px] text-primary ml-1">(You)</span>
                      {/if}
                    </td>
                    <td>
                      <span
                        class="badge badge-xs {u.role === 'admin'
                          ? 'badge-secondary'
                          : 'badge-ghost'}"
                      >
                        {u.role || "user"}
                      </span>
                    </td>
                    <td class="text-right">
                      <button
                        class="btn btn-ghost btn-xs text-primary"
                        onclick={() => handleEdit(u)}
                        title="Edit"
                      >
                        <EditIcon class="w-4 h-4" />
                      </button>
                      {#if u.user_id !== authState.user?.user_id}
                        <button
                          class="btn btn-ghost btn-xs text-error"
                          onclick={() => handleDelete(u.user_id)}
                          title="Delete"
                        >
                          <DeleteIcon class="w-4 h-4" />
                        </button>
                      {:else}
                        <!-- Placeholder to keep layout consistent -->
                        <div
                          class="btn btn-ghost btn-xs opacity-0 cursor-default"
                        >
                          <div class="w-4 h-4"></div>
                        </div>
                      {/if}
                    </td>
                  </tr>
                {/each}
                {#if users.length === 0}
                  <tr>
                    <td colspan="4" class="text-center opacity-50 py-8">
                      No users found matching "{searchQuery}"
                    </td>
                  </tr>
                {/if}
              </tbody>
            </table>
          </div>
        </div>

        {#if showForm}
          <div class="w-80 bg-base-200 p-4 rounded-box overflow-y-auto">
            <h4 class="font-bold mb-4">
              {isEditing ? "Edit User" : "New User"}
            </h4>

            <form
              onsubmit={(e) => {
                e.preventDefault();
                handleSubmit();
              }}
              class="flex flex-col gap-3"
            >
              <div class="form-control">
                <label class="label"
                  ><span class="label-text text-xs">Username</span></label
                >
                <input
                  type="text"
                  bind:value={formData.username}
                  class="input input-sm input-bordered"
                  required
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text text-xs">Password</span>
                  {#if isEditing}
                    <span class="label-text-alt text-xs opacity-50"
                      >(Blank to keep)</span
                    >
                  {/if}
                </label>
                <input
                  type="password"
                  bind:value={formData.password}
                  class="input input-sm input-bordered"
                  placeholder={isEditing ? "••••••••" : ""}
                />
              </div>

              <div class="form-control">
                <label class="label"
                  ><span class="label-text text-xs">Role</span></label
                >
                <select
                  bind:value={formData.role}
                  class="select select-sm select-bordered"
                  disabled={isEditing &&
                    formData.user_id === authState.user?.user_id}
                >
                  <option value="user">User</option>
                  <option value="admin">Admin</option>
                </select>
                {#if isEditing && formData.user_id === authState.user?.user_id}
                  <span class="text-[10px] text-warning mt-1"
                    >You cannot change your own role.</span
                  >
                {/if}
              </div>

              {#if formError}
                <div class="alert alert-error text-xs p-2 mt-2">
                  <span>{formError}</span>
                </div>
              {/if}

              <div class="flex gap-2 mt-4">
                <button
                  type="button"
                  class="btn btn-sm btn-ghost flex-1"
                  onclick={resetForm}>Cancel</button
                >
                <button type="submit" class="btn btn-sm btn-primary flex-1"
                  >Save</button
                >
              </div>
            </form>
          </div>
        {/if}
      </div>
    </div>
    <div class="modal-backdrop" onclick={() => (isOpen = false)}></div>
  </div>
{/if}
