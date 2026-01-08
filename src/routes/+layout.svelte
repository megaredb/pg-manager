<script lang="ts">
  import { fade } from "svelte/transition";
  import { page } from "$app/state";
  import "../app.css";
  import { authState } from "$lib/stores/auth.svelte";
  import Login from "$lib/components/Login.svelte";

  let { children } = $props();
</script>

<div
  class="flex h-screen w-full flex-col overflow-hidden bg-base-100 text-base-content"
>
  {#if authState.isAuthenticated}
    {#key page.url.pathname}
      <div in:fade={{ duration: 150 }} class="h-full">
        {@render children()}
      </div>
    {/key}
  {:else}
    <main
      class="flex h-full w-full items-center justify-center relative bg-base-200"
    >
      <Login />
    </main>
  {/if}
</div>
