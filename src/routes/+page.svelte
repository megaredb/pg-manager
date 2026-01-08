<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import { authState } from "$lib/stores/auth.svelte"; // Import authState

  // Components
  import Sidebar from "$lib/components/Sidebar.svelte";
  import QueryEditor from "$lib/components/QueryEditor.svelte";
  import ResultsTable, {
    type QueryResultItem,
  } from "$lib/components/ResultsTable.svelte";
  import SaveQueryModal from "$lib/components/modals/SaveQueryModal.svelte";
  import SaveDiagramModal from "$lib/components/modals/SaveDiagramModal.svelte";

  import ERDiagram from "$lib/components/diagram/ERDiagram.svelte";

  // Utils
  import { parseSqlScript } from "$lib/utils/sqlHelpers";
  import { tabsState } from "$lib/stores/tabs.svelte";
  import { loadQueryHistory } from "$lib/stores/queryHistory";

  // State
  let isExecuting = $state(false);
  let tabResults = $state(new Map<string, QueryResultItem[]>());

  // Modals
  let isSaveModalOpen = $state(false);
  let isSaveDiagramModalOpen = $state(false);

  // References to diagram components (Map: tabId -> component instance)
  let diagramRefs = $state<Record<string, any>>({});

  let activeTab = $derived(
    tabsState.tabs.find((t) => t.id === tabsState.activeTabId)
  );

  function handleGlobalKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      // SQL Save
      if (activeTab && activeTab.type === "query" && activeTab.content) {
        isSaveModalOpen = true;
      }
      // Diagram Save
      if (activeTab && activeTab.type === "diagram") {
        isSaveDiagramModalOpen = true;
      }
    }
  }

  async function runQuery() {
    if (!activeTab || !activeTab.content || isExecuting) return;

    const queries = parseSqlScript(activeTab.content);

    if (queries.length === 0) {
      console.warn("No queries found.");
      return;
    }

    isExecuting = true;
    const newResultsMap = new Map(tabResults);
    newResultsMap.set(activeTab.id, []);
    tabResults = newResultsMap;

    const currentTabId = activeTab.id;
    const connId = activeTab.connectionId;
    const localResults: QueryResultItem[] = [];

    for (const query of queries) {
      if (tabsState.activeTabId !== currentTabId) break;

      try {
        const res: any = await invoke("execute_query", {
          connectionId: connId,
          queryText: query,
        });

        localResults.push({
          type: "success",
          query: query,
          data: res,
          timestamp: new Date(),
        });
      } catch (e: any) {
        const errorMsg =
          typeof e === "string" ? e : e.message || "Unknown error";
        localResults.push({
          type: "error",
          query: query,
          error: errorMsg,
          timestamp: new Date(),
        });
        break;
      }
    }
    const updateMap = new Map(tabResults);
    updateMap.set(currentTabId, [...localResults]);
    tabResults = updateMap;

    await loadQueryHistory(0, authState.user?.user_id); // Pass user ID
    isExecuting = false;
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

{#if activeTab}
  <!-- SQL Save Modal -->
  <SaveQueryModal
    bind:isOpen={isSaveModalOpen}
    connectionId={activeTab.connectionId}
    queryText={activeTab.content || ""}
    tabId={activeTab.id}
    onClose={() => (isSaveModalOpen = false)}
  />

  <!-- Diagram Save Modal -->
  <SaveDiagramModal
    bind:isOpen={isSaveDiagramModalOpen}
    connectionId={activeTab.connectionId}
    diagramData={diagramRefs[activeTab.id]
      ? diagramRefs[activeTab.id].getDiagramState()
      : ""}
    tabId={activeTab.id}
    existingDiagramId={activeTab.diagramId}
    currentName={activeTab.title.replace("Diagram - ", "")}
    onClose={() => (isSaveDiagramModalOpen = false)}
  />
{/if}

<div class="h-screen w-screen flex bg-base-100 overflow-hidden text-sm">
  <!-- ГЛАВНЫЙ СПЛИТ: Сайдбар слева, Контент справа -->
  <PaneGroup direction="horizontal">
    <!-- LEFT SIDEBAR PANE -->
    <Pane
      defaultSize={20}
      minSize={15}
      maxSize={40}
      class="border-r border-base-300"
    >
      <Sidebar />
    </Pane>

    <PaneResizer
      class="w-1 bg-base-300 hover:bg-primary/50 transition-colors cursor-col-resize"
    />

    <!-- MAIN CONTENT PANE -->
    <Pane>
      <div class="flex flex-col h-full w-full">
        <!-- TABS HEADER -->
        <div
          class="flex items-center bg-base-200 border-b border-base-300 h-10 overflow-x-auto no-scrollbar shrink-0"
        >
          {#each tabsState.tabs as tab (tab.id)}
            <button
              class="flex items-center gap-2 px-4 h-full border-r border-base-300 select-none hover:bg-base-100/50 min-w-[120px] max-w-[200px]
                     {tabsState.activeTabId === tab.id
                ? 'bg-base-100 font-medium border-t-2 border-t-primary'
                : 'text-base-content/70'}"
              onclick={() => tabsState.setActive(tab.id)}
              title={tab.title}
            >
              {#if tab.type === "diagram"}
                <span class="text-secondary" title="ER Diagram">🕸️</span>
              {:else}
                <span class="text-info" title="SQL Query">SQL</span>
              {/if}
              <span class="truncate flex-1 text-left">{tab.title}</span>
              {#if tab.isDirty}<span
                  class="w-2 h-2 rounded-full bg-warning shrink-0"
                ></span>{/if}
              <span
                role="button"
                tabindex="0"
                class="hover:text-error ml-1 opacity-50 hover:opacity-100 shrink-0 font-bold px-1"
                onclick={(e) => {
                  e.stopPropagation();
                  tabsState.closeTab(tab.id);
                }}
                onkeydown={(e) =>
                  e.key === "Enter" ? tabsState.closeTab(tab.id) : null}>×</span
              >
            </button>
          {/each}
          {#if tabsState.tabs.length === 0}
            <div
              class="px-4 text-base-content/40 italic flex items-center gap-2"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                class="w-4 h-4"
                ><path
                  fill-rule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM6.75 9.25a.75.75 0 000 1.5h4.59l-2.1 1.95a.75.75 0 001.02 1.1l3.5-3.25a.75.75 0 000-1.1l-3.5-3.25a.75.75 0 10-1.02 1.1l2.1 1.95H6.75z"
                  clip-rule="evenodd"
                /></svg
              >
              <span>Select connection > Right Click > New Query / Diagram</span>
            </div>
          {/if}
        </div>

        <!-- TAB CONTENT (Render ALL tabs, toggle visibility) -->
        <div class="flex-1 relative overflow-hidden">
          {#each tabsState.tabs as tab (tab.id)}
            <!-- Используем display: none для скрытия неактивных вкладок, чтобы сохранить их состояние (DOM и JS) -->
            <div
              class="absolute inset-0 flex flex-col bg-base-100"
              style:display={tabsState.activeTabId === tab.id ? "flex" : "none"}
            >
              <!-- (A) ER DIAGRAM VIEW -->
              {#if tab.type === "diagram"}
                <div class="flex flex-col h-full">
                  <!-- DIAGRAM TOOLBAR -->
                  <div
                    class="h-10 border-b border-base-300 flex items-center px-2 gap-2 bg-base-100 shrink-0"
                  >
                    <button
                      class="btn btn-sm btn-ghost gap-2 font-normal"
                      onclick={() => (isSaveDiagramModalOpen = true)}
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-4 h-4"
                        ><path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          d="M17.593 3.322c1.1.128 1.907 1.077 1.907 2.185V21L12 17.25 4.5 21V5.507c0-1.108.806-2.057 1.907-2.185a48.507 48.507 0 0111.186 0z"
                        /></svg
                      >
                      Save Diagram
                    </button>
                    <div
                      class="divider divider-horizontal m-0 h-6 self-center"
                    ></div>
                    <span class="text-xs text-base-content/50 font-mono"
                      >Conn: {tab.connectionId}</span
                    >
                  </div>

                  <div class="flex-1 relative">
                    <ERDiagram
                      bind:this={diagramRefs[tab.id]}
                      connectionId={tab.connectionId}
                      schemaName={tab.content || "public"}
                      initialData={tab.diagramData}
                    />
                  </div>
                </div>

                <!-- (B) SQL EDITOR VIEW -->
              {:else}
                <div class="flex flex-col h-full">
                  <!-- SQL TOOLBAR -->
                  <div
                    class="h-10 border-b border-base-300 flex items-center px-2 gap-2 bg-base-100 shrink-0"
                  >
                    <button
                      class="btn btn-sm btn-primary gap-2 font-normal"
                      disabled={isExecuting && tabsState.activeTabId === tab.id}
                      onclick={runQuery}
                    >
                      {#if isExecuting && tabsState.activeTabId === tab.id}
                        <span class="loading loading-spinner loading-xs"></span>
                      {:else}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          viewBox="0 0 24 24"
                          fill="currentColor"
                          class="w-4 h-4"
                          ><path
                            fill-rule="evenodd"
                            d="M4.5 5.653c0-1.426 1.529-2.33 2.779-1.643l11.54 6.348c1.295.712 1.295 2.573 0 3.285L7.28 19.991c-1.25.687-2.779-.217-2.779-1.643V5.653z"
                            clip-rule="evenodd"
                          /></svg
                        >
                      {/if}
                      Run
                    </button>
                    <button
                      class="btn btn-sm btn-ghost gap-2 font-normal"
                      onclick={() => (isSaveModalOpen = true)}
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-4 h-4"
                        ><path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          d="M17.593 3.322c1.1.128 1.907 1.077 1.907 2.185V21L12 17.25 4.5 21V5.507c0-1.108.806-2.057 1.907-2.185a48.507 48.507 0 0111.186 0z"
                        /></svg
                      > Save
                    </button>
                    <div
                      class="divider divider-horizontal m-0 h-6 self-center"
                    ></div>
                    <span class="text-xs text-base-content/50 font-mono"
                      >Conn: {tab.connectionId}</span
                    >
                  </div>

                  <!-- EDITOR / RESULTS SPLIT -->
                  <PaneGroup direction="vertical">
                    <Pane defaultSize={40} minSize={20} class="bg-base-100">
                      <!-- Используем onChange для обновления стора, так как bind:value на tab.content внутри each иногда требует явного обновления -->
                      <QueryEditor
                        value={tab.content || ""}
                        onChange={(val) =>
                          tabsState.updateTabContent(tab.id, val)}
                        onRun={runQuery}
                      />
                    </Pane>
                    <PaneResizer
                      class="h-1 bg-base-300 hover:bg-primary/50 transition-colors cursor-row-resize"
                    />
                    <Pane defaultSize={60} minSize={20} class="bg-base-200/50">
                      <ResultsTable
                        results={tabResults.get(tab.id) || []}
                        {isExecuting}
                      />
                    </Pane>
                  </PaneGroup>
                </div>
              {/if}
            </div>
          {/each}

          {#if tabsState.tabs.length === 0}
            <!-- EMPTY STATE -->
            <div
              class="flex flex-col items-center justify-center h-full text-base-content/30 gap-4 bg-base-200/30"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="w-20 h-20 opacity-20"
                ><path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125"
                /></svg
              >
              <div class="text-center">
                <p class="font-medium">No Active Query</p>
                <p class="text-sm opacity-70">
                  Open a connection from the sidebar to start
                </p>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </Pane>
  </PaneGroup>
</div>
