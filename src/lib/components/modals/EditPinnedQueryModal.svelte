<script lang="ts">
  import {
    updatePinnedQuery,
    type PinnedQuery,
  } from "$lib/stores/pinnedQueries";

  interface Props {
    isOpen: boolean;
    pinnedQuery: PinnedQuery | null;
    onClose: () => void;
  }

  let { isOpen = $bindable(), pinnedQuery, onClose }: Props = $props();

  let name = $state("");
  let description = $state("");
  let isSubmitting = $state(false);

  $effect(() => {
    if (isOpen && pinnedQuery) {
      name = pinnedQuery.query_name;
      description = pinnedQuery.description || "";
    }
  });

  async function handleSave() {
    if (!pinnedQuery || !name.trim()) return;
    isSubmitting = true;
    try {
      await updatePinnedQuery(pinnedQuery.pinned_query_id, {
        query_name: name,
        description: description,
      });
      onClose();
    } catch (e) {
      console.error(e);
      alert("Failed to update query");
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen && pinnedQuery}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Edit Pinned Query</h3>

      <div class="form-control w-full mt-4">
        <label class="label"><span class="label-text">Name</span></label>
        <input
          type="text"
          bind:value={name}
          class="input input-bordered w-full"
        />
      </div>

      <div class="form-control w-full mt-2">
        <label class="label"><span class="label-text">Description</span></label>
        <textarea
          bind:value={description}
          class="textarea textarea-bordered h-24"
        ></textarea>
      </div>

      <div class="modal-action">
        <button class="btn btn-ghost" onclick={onClose}>Cancel</button>
        <button
          class="btn btn-primary"
          onclick={handleSave}
          disabled={isSubmitting || !name}
        >
          Save
        </button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={onClose}></div>
  </div>
{/if}
