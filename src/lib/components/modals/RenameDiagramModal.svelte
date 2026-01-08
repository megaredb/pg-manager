<script lang="ts">
  import { updateDiagram, type Diagram } from "$lib/stores/diagrams";
  import { tabsState } from "$lib/stores/tabs.svelte";

  interface Props {
    isOpen: boolean;
    diagram: Diagram | null;
    onClose: () => void;
  }

  let { isOpen = $bindable(), diagram, onClose }: Props = $props();

  let name = $state("");
  let isSubmitting = $state(false);

  $effect(() => {
    if (isOpen && diagram) {
      name = diagram.diagram_name;
    }
  });

  async function handleSave() {
    if (!diagram || !name.trim()) return;
    isSubmitting = true;
    try {
      await updateDiagram(diagram.diagram_id, {
        diagram_name: name,
        definition_json: diagram.definition_json,
      });

      const openTab = tabsState.tabs.find(
        (t) => t.diagramId === diagram.diagram_id
      );
      if (openTab) {
        tabsState.markAsSaved(openTab.id, name, diagram.diagram_id);
      }

      onClose();
    } catch (e) {
      console.error(e);
      alert("Failed to rename diagram");
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen && diagram}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Rename Diagram</h3>

      <div class="form-control w-full mt-4">
        <label class="label"><span class="label-text">New Name</span></label>
        <input
          type="text"
          bind:value={name}
          class="input input-bordered w-full"
          autofocus
        />
      </div>

      <div class="modal-action">
        <button class="btn btn-ghost" onclick={onClose}>Cancel</button>
        <button
          class="btn btn-primary"
          onclick={handleSave}
          disabled={isSubmitting || !name.trim()}
        >
          {isSubmitting ? "Saving..." : "Rename"}
        </button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={onClose}></div>
  </div>
{/if}
