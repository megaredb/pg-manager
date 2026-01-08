<script lang="ts">
  import { createPinnedQuery } from "$lib/stores/pinnedQueries";
  import { tabsState } from "$lib/stores/tabs.svelte";

  interface Props {
    isOpen: boolean;
    connectionId: number;
    queryText: string;
    tabId: string;
    onClose: () => void;
  }

  let {
    isOpen = $bindable(),
    connectionId,
    queryText,
    tabId,
    onClose,
  }: Props = $props();

  let name = $state("");
  let description = $state("");
  let isSubmitting = $state(false);

  async function handleSave() {
    if (!name) return;
    isSubmitting = true;
    try {
      await createPinnedQuery({
        connection_id: connectionId,
        query_name: name,
        query_text: queryText,
        description: description,
      });

      tabsState.markAsSaved(tabId, name);

      onClose();
      name = "";
      description = "";
    } catch (e) {
      console.error(e);
      alert("Failed to save query");
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Save Query</h3>

      <div class="w-full mt-4 flex flex-col gap-2">
        <p class="text-sm">Query Name</p>
        <input
          type="text"
          bind:value={name}
          placeholder="e.g. Monthly Report"
          class="input input-bordered w-full"
        />
      </div>

      <div class="w-full mt-2 flex flex-col gap-2">
        <p class="text-sm">Description (Optional)</p>
        <textarea
          bind:value={description}
          class="textarea textarea-bordered h-24 w-full"
          placeholder="What does this query do?"
        ></textarea>
      </div>

      <div class="modal-action">
        <button class="btn btn-ghost" onclick={onClose}>Cancel</button>
        <button
          class="btn btn-primary"
          onclick={handleSave}
          disabled={isSubmitting || !name}
        >
          {isSubmitting ? "Saving..." : "Save"}
        </button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={onClose}></div>
  </div>
{/if}
