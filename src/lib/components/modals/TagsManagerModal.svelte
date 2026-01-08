<script lang="ts">
  import {
    tags,
    connectionTags,
    createTag,
    deleteTag,
    addTagToConnection,
    removeTagFromConnection,
    type Tag,
  } from "$lib/stores/tags";

  interface Props {
    isOpen: boolean;
    connectionId?: number | null;
    onClose: () => void;
  }

  let { isOpen = $bindable(), connectionId, onClose }: Props = $props();

  let newTagName = $state("");
  let newTagColor = $state("#3b82f6");

  let assignedTagIds = $derived(
    connectionId && $connectionTags[connectionId]
      ? new Set($connectionTags[connectionId])
      : new Set<number>()
  );

  async function handleCreate() {
    if (!newTagName.trim()) return;
    await createTag(newTagName, newTagColor);
    newTagName = "";
  }

  async function toggleAssignment(tag: Tag) {
    if (!connectionId) return;
    if (assignedTagIds.has(tag.tag_id)) {
      await removeTagFromConnection(tag.tag_id, connectionId);
    } else {
      await addTagToConnection(tag.tag_id, connectionId);
    }
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg mb-4">
        {connectionId ? "Manage Tags for Connection" : "Manage Tags"}
      </h3>

      <!-- Create New -->
      <div class="flex gap-2 mb-6 items-end">
        <div class="form-control flex-1">
          <label class="label text-xs pb-1">New Tag Name</label>
          <input
            type="text"
            bind:value={newTagName}
            class="input input-sm input-bordered"
            placeholder="e.g. Production"
          />
        </div>
        <div class="form-control">
          <label class="label text-xs pb-1">Color</label>
          <input
            type="color"
            bind:value={newTagColor}
            class="input input-sm p-0 w-12 h-8"
          />
        </div>
        <button
          class="btn btn-sm btn-primary"
          onclick={handleCreate}
          disabled={!newTagName}>Add</button
        >
      </div>

      <!-- List -->
      <div class="space-y-2 max-h-60 overflow-y-auto">
        {#if $tags.length === 0}
          <div class="text-center text-sm opacity-50 py-4">
            No tags created yet.
          </div>
        {/if}

        {#each $tags as tag}
          <div
            class="flex items-center justify-between p-2 bg-base-200 rounded-box"
          >
            <div class="flex items-center gap-2">
              {#if connectionId}
                <input
                  type="checkbox"
                  class="checkbox checkbox-xs"
                  checked={assignedTagIds.has(tag.tag_id)}
                  onchange={() => toggleAssignment(tag)}
                />
              {/if}
              <div
                class="w-3 h-3 rounded-full"
                style:background-color={tag.color_hex || "#ccc"}
              ></div>
              <span class="font-medium">{tag.tag_name}</span>
            </div>

            <button
              class="btn btn-ghost btn-xs text-error"
              onclick={() => deleteTag(tag.tag_id)}
              title="Delete Tag">✕</button
            >
          </div>
        {/each}
      </div>

      <div class="modal-action">
        <button class="btn" onclick={onClose}>Close</button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={onClose}></div>
  </div>
{/if}
