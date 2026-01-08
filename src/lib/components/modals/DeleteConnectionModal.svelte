<script lang="ts">
  import type { Connection } from "$lib/stores/connections";
  import DeleteIcon from "$lib/components/icons/DeleteIcon.svelte";

  interface Props {
    isOpen: boolean;
    connection: Connection | null;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    isOpen = $bindable(),
    connection,
    onConfirm,
    onCancel,
  }: Props = $props();
</script>

{#if isOpen && connection}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg flex items-center gap-2">
        <DeleteIcon class="h-5 w-5 text-error" />
        Delete Connection
      </h3>
      <p class="py-4">
        Are you sure you want to delete the connection
        <span class="font-semibold">"{connection.connection_name}"</span>? This
        action cannot be undone.
      </p>
      <div class="modal-action">
        <button class="btn btn-ghost" onclick={onCancel}>Cancel</button>
        <button class="btn btn-error" onclick={onConfirm}>
          <DeleteIcon class="h-4 w-4" />
          Delete
        </button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={onCancel}></div>
  </div>
{/if}
