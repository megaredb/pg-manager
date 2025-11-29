<script lang="ts">
  import { fade } from "svelte/transition";
  import { page } from "$app/state";
  import "../app.css";

  let { children } = $props();

  let tabs = [
    { id: 0, label: "Main", href: "/" },
    { id: 1, label: "App Users", href: "/app-users" },
    { id: 2, label: "Connections", href: "/connections" },
    { id: 3, label: "Connections Folders", href: "/connections-folders" },
    { id: 4, label: "Query Editor", href: "/query-editor" },
  ];
</script>

<div role="tablist" class="tabs tabs-box">
  {#each tabs as tab}
    <a
      href={tab.href}
      class={tab.href === page.url.pathname ? "tab tab-active" : "tab"}
    >
      {tab.label}
    </a>
  {/each}
</div>

{#key page.url.pathname}
  <div in:fade={{ duration: 150 }} style="position: relative; width: 100%;">
    {@render children()}
  </div>
{/key}
