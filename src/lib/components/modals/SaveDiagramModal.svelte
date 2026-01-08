<script lang="ts">
  import { createDiagram, updateDiagram } from "$lib/stores/diagrams";
  import { tabsState } from "$lib/stores/tabs.svelte";

  interface Props {
    isOpen: boolean;
    connectionId: number;
    diagramData: string; // JSON
    tabId: string;
    existingDiagramId?: number;
    currentName?: string;
    onClose: () => void;
  }

  let {
    isOpen = $bindable(),
    connectionId,
    diagramData,
    tabId,
    existingDiagramId,
    currentName = "",
    onClose,
  }: Props = $props();

  let name = $state(currentName);
  let isSubmitting = $state(false);

  // Обновляем имя, если открываем для существующей диаграммы
  $effect(() => {
    if (isOpen) name = currentName || "";
  });

  async function handleSave() {
    if (!name) return;
    isSubmitting = true;
    try {
      if (existingDiagramId) {
        await updateDiagram(existingDiagramId, {
          diagram_name: name,
          definition_json: diagramData,
        });
        tabsState.markAsSaved(tabId, name);
      } else {
        // В реальном приложении createDiagram должен возвращать ID, чтобы мы могли обновить таб
        // Пока просто сохраним
        await createDiagram({
          connection_id: connectionId,
          diagram_name: name,
          definition_json: diagramData,
        });
        // Для простоты: закрываем и обновляем список.
        // В идеале: получить ID и сделать tabsState.markAsSaved(tabId, name, newId);
        tabsState.markAsSaved(tabId, name);
      }

      onClose();
    } catch (e) {
      console.error(e);
      alert("Failed to save diagram");
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">
        {existingDiagramId ? "Update Diagram" : "Save Diagram"}
      </h3>

      <div class="form-control w-full mt-4">
        <label class="label"><span class="label-text">Diagram Name</span></label
        >
        <input
          type="text"
          bind:value={name}
          placeholder="e.g. User Relations"
          class="input input-bordered w-full"
        />
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
