<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  export interface MenuItem {
    label: string;
    icon?: any;
    onClick: () => void;
    className?: string;
  }

  interface Props {
    isOpen: boolean;
    x: number;
    y: number;
    items: MenuItem[];
    onClose: () => void;
  }

  let { isOpen = $bindable(), x, y, items, onClose }: Props = $props();

  let menuElement: HTMLDivElement;

  function handleClickOutside(event: MouseEvent) {
    if (menuElement && !menuElement.contains(event.target as Node)) {
      onClose();
    }
  }

  function handleEscape(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  $effect(() => {
    if (isOpen) {
      document.addEventListener("click", handleClickOutside);
      document.addEventListener("keydown", handleEscape);

      return () => {
        document.removeEventListener("click", handleClickOutside);
        document.removeEventListener("keydown", handleEscape);
      };
    }
  });

  function handleItemClick(item: MenuItem) {
    item.onClick();
    onClose();
  }
</script>

{#if isOpen}
  <div
    bind:this={menuElement}
    class="fixed z-50 menu bg-base-100 rounded-box shadow-lg border border-base-300 min-w-40"
    style="left: {x}px; top: {y}px;"
  >
    <ul>
      {#each items as item}
        <li>
          <button
            onclick={() => handleItemClick(item)}
            class={item.className || ""}
          >
            {#if item.icon}
              <svelte:component this={item.icon} class="h-4 w-4" />
            {/if}
            {item.label}
          </button>
        </li>
      {/each}
    </ul>
  </div>
{/if}
