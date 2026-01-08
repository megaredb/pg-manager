<script lang="ts">
  import {
    createFolder,
    updateFolder,
    type ConnectionFolder,
  } from "$lib/stores/folders";
  import { authState } from "$lib/stores/auth.svelte";
  import FolderIcon from "$lib/components/icons/FolderIcon.svelte";

  interface Props {
    isOpen: boolean;
    folder?: ConnectionFolder | null;
    onClose: () => void;
  }

  let { isOpen = $bindable(), folder = null, onClose }: Props = $props();

  let folderName = $state("");
  let isSubmitting = $state(false);

  // Когда меняется папка для редактирования, обновляем имя
  $effect(() => {
    if (isOpen) {
      folderName = folder ? folder.folder_name : "";
    }
  });

  async function handleSubmit() {
    if (!folderName.trim()) return;
    isSubmitting = true;
    try {
      if (folder) {
        await updateFolder(folder.folder_id, folderName);
      } else {
        // Fallback user ID 1 if not auth
        const userId = authState.user?.user_id || 1;
        await createFolder(folderName, userId);
      }
      onClose();
    } catch (e) {
      console.error(e);
      alert("Failed to save folder");
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg flex items-center gap-2">
        <FolderIcon class="h-5 w-5" />
        {folder ? "Edit Folder" : "New Folder"}
      </h3>

      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleSubmit();
        }}
      >
        <div class="form-control w-full mt-4">
          <label class="label"
            ><span class="label-text">Folder Name</span></label
          >
          <input
            type="text"
            bind:value={folderName}
            placeholder="Production DBs"
            class="input input-bordered w-full"
            autofocus
          />
        </div>

        <div class="modal-action">
          <button type="button" class="btn btn-ghost" onclick={onClose}
            >Cancel</button
          >
          <button
            type="submit"
            class="btn btn-primary"
            disabled={isSubmitting || !folderName.trim()}
          >
            {isSubmitting ? "Saving..." : "Save"}
          </button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" onclick={onClose}></div>
  </div>
{/if}
