<script lang="ts">
  import { onMount } from "svelte";
  import { authState } from "$lib/stores/auth.svelte";
  import {
    connections,
    loadConnections,
    deleteConnection,
    type Connection,
    activeConnectionTags,
  } from "$lib/stores/connections";
  import {
    folders,
    loadFolders,
    deleteFolder,
    type ConnectionFolder,
  } from "$lib/stores/folders";
  import {
    schemaCache,
    loadSchemas,
    loadTables,
    loadViews,
  } from "$lib/stores/pgSchema";
  import {
    queryHistory,
    loadQueryHistory,
    historyPage,
    historySearch,
    historyStatus,
    historySortDesc,
    historyStartDate,
    historyEndDate,
  } from "$lib/stores/queryHistory";
  import {
    pinnedQueries,
    loadPinnedQueries,
    deletePinnedQuery,
    pinnedSearch,
    type PinnedQuery,
    pinnedSortAsc,
  } from "$lib/stores/pinnedQueries";
  import {
    diagrams,
    loadDiagrams,
    deleteDiagram,
    type Diagram,
  } from "$lib/stores/diagrams";
  import { tags, connectionTags, loadTags, type Tag } from "$lib/stores/tags";
  import {
    bookmarks,
    loadBookmarks,
    toggleBookmark,
    type Bookmark,
  } from "$lib/stores/bookmarks";
  import { tabsState } from "$lib/stores/tabs.svelte";

  // Components
  import ContextMenu, {
    type MenuItem,
  } from "$lib/components/ContextMenu.svelte";
  import ConnectionFormModal from "$lib/components/modals/ConnectionFormModal.svelte";
  import DeleteConnectionModal from "$lib/components/modals/DeleteConnectionModal.svelte";
  import UserManagementModal from "$lib/components/modals/UserManagementModal.svelte";
  import FolderModal from "$lib/components/modals/FolderModal.svelte";
  import EditPinnedQueryModal from "$lib/components/modals/EditPinnedQueryModal.svelte";
  import RenameDiagramModal from "$lib/components/modals/RenameDiagramModal.svelte";
  import TagsManagerModal from "$lib/components/modals/TagsManagerModal.svelte";
  import DashboardModal from "$lib/components/modals/DashboardModal.svelte";

  // Icons
  import AddIcon from "$lib/components/icons/AddIcon.svelte";
  import DatabaseIcon from "$lib/components/icons/DatabaseIcon.svelte";
  import DeleteIcon from "$lib/components/icons/DeleteIcon.svelte";
  import EditIcon from "$lib/components/icons/EditIcon.svelte";
  import FolderIcon from "$lib/components/icons/FolderIcon.svelte";
  import LogoutIcon from "$lib/components/icons/LogoutIcon.svelte";
  import SchemaIcon from "$lib/components/icons/SchemaIcon.svelte";
  import TableIcon from "$lib/components/icons/TableIcon.svelte";
  import ViewIcon from "$lib/components/icons/ViewIcon.svelte";
  import SettingsIcon from "$lib/components/icons/SettingsIcon.svelte";
  import { highlightMatch } from "$lib/utils/highlight";

  // --- STATE ---
  type SidebarTab = "explorer" | "pinned" | "history" | "diagrams";
  let activeSidebarTab = $state<SidebarTab>("explorer");

  let expandedNodes = $state(new Set<string>());
  let loadingConnections = $state(new Set<number>());
  let failedConnections = $state(new Set<number>());

  // Modals
  let isConnectionModalOpen = $state(false);
  let isDeleteModalOpen = $state(false);
  let isUserManagementOpen = $state(false);
  let isFolderModalOpen = $state(false);
  let isEditPinnedModalOpen = $state(false);
  let isRenameDiagramModalOpen = $state(false);
  let isTagsModalOpen = $state(false);
  let isDashboardOpen = $state(false);

  let targetConnection = $state<Connection | null>(null);
  let targetFolder = $state<ConnectionFolder | null>(null);
  let targetPinnedQuery = $state<PinnedQuery | null>(null);
  let targetDiagram = $state<Diagram | null>(null);

  // UI
  let isUserDropdownOpen = $state(false);
  let contextMenuOpen = $state(false);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let contextMenuItems = $state<MenuItem[]>([]);
  let errorToast = $state<string | null>(null);
  let toastTimeout: any;

  // Debounce timers
  let historySearchTimer: any;
  let pinnedSearchTimer: any;

  // --- ACTIONS ---
  function togglePinnedSort() {
    $pinnedSortAsc = !$pinnedSortAsc;
    loadPinnedQueries(authState.user?.user_id);
  }

  function openPinnedQuery(pq: PinnedQuery) {
    tabsState.addTab({
      title: pq.query_name,
      type: "query",
      connectionId: pq.connection_id,
      content: pq.query_text,
    });
  }

  function openSavedDiagram(d: Diagram) {
    tabsState.addTab({
      title: d.diagram_name,
      type: "diagram",
      connectionId: d.connection_id,
      content: "public",
      diagramId: d.diagram_id,
      diagramData: d.definition_json,
    });
  }

  function restoreHistoryQuery(item: any) {
    tabsState.addTab({
      title: `History ${new Date(item.executed_at).toLocaleTimeString()}`,
      type: "query",
      connectionId: item.connection_id,
      content: item.query_text,
    });
  }

  function openBookmark(bm: Bookmark) {
    if (bm.object_type === "connection") {
      // Connection закладки удалены по запросу
    } else if (bm.object_type === "schema") {
      tabsState.addTab({
        title: `Diagram - ${bm.schema_name}`,
        type: "diagram",
        connectionId: bm.connection_id,
        content: bm.schema_name,
      });
    } else {
      tabsState.addTab({
        title: bm.object_name,
        type: "query",
        connectionId: bm.connection_id,
        content: `SELECT * FROM "${bm.schema_name}"."${bm.object_name}" LIMIT 100;`,
      });
    }
  }

  function toggleStar(
    e: MouseEvent,
    connId: number,
    schema: string,
    obj: string,
    type: string
  ) {
    e.stopPropagation();
    toggleBookmark(connId, schema, obj, type);
  }

  function handleTableClick(connId: number, schema: string, table: string) {
    tabsState.addTab({
      title: table,
      type: "query",
      connectionId: connId,
      content: `SELECT * FROM "${schema}"."${table}" LIMIT 100;`,
    });
  }

  // --- FILTER HANDLERS ---
  function handleHistorySearch(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    $historySearch = val;
    clearTimeout(historySearchTimer);
    historySearchTimer = setTimeout(() => {
      loadQueryHistory(0, authState.user?.user_id);
    }, 300);
  }

  function handleHistoryStatusChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    $historyStatus = val;
    loadQueryHistory(0, authState.user?.user_id);
  }

  function toggleHistorySort() {
    $historySortDesc = !$historySortDesc;
    loadQueryHistory(0, authState.user?.user_id);
  }

  function handlePinnedSearch(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    $pinnedSearch = val;
    clearTimeout(pinnedSearchTimer);
    pinnedSearchTimer = setTimeout(() => {
      loadPinnedQueries(authState.user?.user_id);
    }, 300);
  }

  // --- HELPERS ---
  function toggleNode(key: string) {
    const newSet = new Set(expandedNodes);
    if (newSet.has(key)) newSet.delete(key);
    else newSet.add(key);
    expandedNodes = newSet;
  }
  function showError(message: unknown) {
    const msg =
      typeof message === "string"
        ? message
        : message instanceof Error
          ? message.message
          : "Unknown error";
    errorToast = msg;
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
      errorToast = null;
    }, 4000);
  }

  async function handleConnectionClick(conn: Connection) {
    const key = `conn:${conn.connection_id}`;
    if (expandedNodes.has(key)) {
      toggleNode(key);
      return;
    }
    loadingConnections = new Set(loadingConnections).add(conn.connection_id);
    if (failedConnections.has(conn.connection_id))
      failedConnections = new Set(
        [...failedConnections].filter((id) => id !== conn.connection_id)
      );

    try {
      await loadSchemas(conn.connection_id);
      toggleNode(key);
    } catch (e) {
      failedConnections = new Set(failedConnections).add(conn.connection_id);
      showError(e);
    } finally {
      loadingConnections = new Set(
        [...loadingConnections].filter((id) => id !== conn.connection_id)
      );
    }
  }

  async function handleSchemaClick(connId: number, schemaName: string) {
    const key = `schema:${connId}:${schemaName}`;
    if (!expandedNodes.has(key)) {
      try {
        await Promise.all([
          loadTables(connId, schemaName),
          loadViews(connId, schemaName),
        ]);
        toggleNode(key);
      } catch (e) {
        showError(e);
      }
    } else {
      toggleNode(key);
    }
  }

  // --- CONTEXT MENUS ---
  // (Оставляем как есть, код меню не меняется)
  function handleConnectionContextMenu(e: MouseEvent, conn: Connection) {
    e.preventDefault();
    e.stopPropagation();
    targetConnection = conn;
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuItems = [
      {
        label: "New Query",
        icon: AddIcon,
        onClick: () => {
          tabsState.addTab({
            title: `Query - ${conn.connection_name}`,
            type: "query",
            connectionId: conn.connection_id,
            content: "",
          });
        },
      },
      {
        label: "View ER Diagram (Public)",
        icon: SchemaIcon,
        onClick: () => {
          tabsState.addTab({
            title: `Diagram - ${conn.connection_name}`,
            type: "diagram",
            connectionId: conn.connection_id,
            content: "public",
          });
        },
      },
      {
        label: "Manage Tags...",
        icon: EditIcon,
        onClick: () => {
          isTagsModalOpen = true;
        },
      },
      {
        label: "Edit Connection",
        icon: EditIcon,
        onClick: () => {
          isConnectionModalOpen = true;
        },
      },
      {
        label: "Delete Connection",
        icon: DeleteIcon,
        className: "text-error",
        onClick: () => {
          isDeleteModalOpen = true;
        },
      },
    ];
    contextMenuOpen = true;
  }

  function handleSchemaContextMenu(
    e: MouseEvent,
    connId: number,
    schemaName: string
  ) {
    e.preventDefault();
    e.stopPropagation();
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuItems = [
      {
        label: `View Diagram (${schemaName})`,
        icon: SchemaIcon,
        onClick: () => {
          tabsState.addTab({
            title: `Diagram - ${schemaName}`,
            type: "diagram",
            connectionId: connId,
            content: schemaName,
          });
        },
      },
    ];
    contextMenuOpen = true;
  }

  function handleFolderContextMenu(e: MouseEvent, folder: ConnectionFolder) {
    e.preventDefault();
    e.stopPropagation();
    targetFolder = folder;
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuItems = [
      {
        label: "Rename Folder",
        icon: EditIcon,
        onClick: () => {
          isFolderModalOpen = true;
        },
      },
      {
        label: "Delete Folder",
        icon: DeleteIcon,
        className: "text-error",
        onClick: async () => {
          if (await confirm(`Delete folder "${folder.folder_name}"?`)) {
            await deleteFolder(folder.folder_id);
            await loadConnections(authState.user?.user_id);
          }
        },
      },
    ];
    contextMenuOpen = true;
  }

  function handlePinnedQueryContextMenu(e: MouseEvent, pq: PinnedQuery) {
    e.preventDefault();
    e.stopPropagation();
    targetPinnedQuery = pq;
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuItems = [
      {
        label: "Edit",
        icon: EditIcon,
        onClick: () => {
          isEditPinnedModalOpen = true;
        },
      },
      {
        label: "Delete",
        icon: DeleteIcon,
        className: "text-error",
        onClick: async () => {
          if (await confirm(`Delete saved query?`))
            await deletePinnedQuery(pq.pinned_query_id);
        },
      },
    ];
    contextMenuOpen = true;
  }

  function handleDiagramContextMenu(e: MouseEvent, d: Diagram) {
    e.preventDefault();
    e.stopPropagation();
    targetDiagram = d;
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuItems = [
      {
        label: "Rename Diagram",
        icon: EditIcon,
        onClick: () => {
          isRenameDiagramModalOpen = true;
        },
      },
      {
        label: "Delete Diagram",
        icon: DeleteIcon,
        className: "text-error",
        onClick: async () => {
          if (await confirm(`Delete diagram "${d.diagram_name}"?`)) {
            await deleteDiagram(d.diagram_id);
          }
        },
      },
    ];
    contextMenuOpen = true;
  }

  function handleAddConnection() {
    targetConnection = null;
    isConnectionModalOpen = true;
  }
  function handleAddFolder() {
    targetFolder = null;
    isFolderModalOpen = true;
  }
  function handleConfirmDelete() {
    if (targetConnection) {
      deleteConnection(targetConnection.connection_id).then(() => {
        isDeleteModalOpen = false;
        targetConnection = null;
      });
    }
  }
  function handleOpenSettings() {
    isUserManagementOpen = true;
    isUserDropdownOpen = false;
  }
  function handleOpenDashboard() {
    isDashboardOpen = true;
    isUserDropdownOpen = false;
  }
  function handleLogout() {
    authState.logout();
    isUserDropdownOpen = false;
  }

  async function nextHistoryPage() {
    await loadQueryHistory($historyPage + 1, authState.user?.user_id);
  }
  async function prevHistoryPage() {
    if ($historyPage > 0)
      await loadQueryHistory($historyPage - 1, authState.user?.user_id);
  }

  let groupedConnections = $derived.by(() => {
    const withFolder: Record<number, Connection[]> = {};
    const withoutFolder: Connection[] = [];

    const source = $connections;

    for (const conn of source) {
      if (conn.folder_id) {
        if (!withFolder[conn.folder_id]) withFolder[conn.folder_id] = [];
        withFolder[conn.folder_id].push(conn);
      } else {
        withoutFolder.push(conn);
      }
    }
    return { withFolder, withoutFolder };
  });

  function getConnectionTags(connId: number) {
    const tagIds = $connectionTags[connId] || [];
    return tagIds
      .map((id) => $tags.find((t) => t.tag_id === id))
      .filter(Boolean) as Tag[];
  }
  function isStarred(
    connId: number,
    schema: string,
    obj: string,
    type: string
  ) {
    return $bookmarks.some(
      (b) =>
        b.connection_id === connId &&
        b.schema_name === schema &&
        b.object_name === obj &&
        b.object_type === type
    );
  }

  $effect(() => {
    if (authState.user?.user_id) {
      loadConnections(authState.user.user_id);
      loadFolders(authState.user.user_id);
      loadPinnedQueries(authState.user.user_id);
      loadDiagrams(authState.user.user_id);
      loadQueryHistory(0, authState.user.user_id);
      loadTags(authState.user.user_id);
      loadBookmarks(authState.user.user_id);
    }
  });

  onMount(() => {
    if (authState.user?.user_id) {
      loadConnections(authState.user.user_id);
      loadFolders(authState.user.user_id);
      loadPinnedQueries(authState.user.user_id);
      loadDiagrams(authState.user.user_id);
      loadQueryHistory(0, authState.user.user_id);
      loadTags(authState.user.user_id);
      loadBookmarks(authState.user.user_id);
    }
  });

  function handleDateChange() {
    loadQueryHistory(0, authState.user?.user_id);
  }
</script>

<!-- MODALS -->
<DashboardModal bind:isOpen={isDashboardOpen} />
<ConnectionFormModal
  bind:isOpen={isConnectionModalOpen}
  connection={targetConnection}
  onClose={() => {
    isConnectionModalOpen = false;
    targetConnection = null;
  }}
  onSuccess={() => {
    loadConnections(authState.user?.user_id);
  }}
/>
<DeleteConnectionModal
  bind:isOpen={isDeleteModalOpen}
  connection={targetConnection}
  onConfirm={handleConfirmDelete}
  onCancel={() => {
    isDeleteModalOpen = false;
    targetConnection = null;
  }}
/>
<UserManagementModal bind:isOpen={isUserManagementOpen} />
<FolderModal
  bind:isOpen={isFolderModalOpen}
  folder={targetFolder}
  onClose={() => {
    isFolderModalOpen = false;
    targetFolder = null;
  }}
/>
<EditPinnedQueryModal
  bind:isOpen={isEditPinnedModalOpen}
  pinnedQuery={targetPinnedQuery}
  onClose={() => {
    isEditPinnedModalOpen = false;
    targetPinnedQuery = null;
  }}
/>
<RenameDiagramModal
  bind:isOpen={isRenameDiagramModalOpen}
  diagram={targetDiagram}
  onClose={() => {
    isRenameDiagramModalOpen = false;
    targetDiagram = null;
  }}
/>
<TagsManagerModal
  bind:isOpen={isTagsModalOpen}
  connectionId={targetConnection?.connection_id}
  onClose={() => {
    isTagsModalOpen = false;
    targetConnection = null;
  }}
/>
<ContextMenu
  bind:isOpen={contextMenuOpen}
  x={contextMenuX}
  y={contextMenuY}
  items={contextMenuItems}
  onClose={() => (contextMenuOpen = false)}
/>

{#if errorToast}
  <div class="toast toast-bottom toast-end z-9999">
    <div class="alert alert-error text-white shadow-lg p-2 min-h-0 text-sm">
      <span>{errorToast}</span>
    </div>
  </div>
{/if}

<div class="h-full bg-base-200 border-r border-base-300 flex flex-col w-full">
  <!-- TOP TABS -->
  <div class="flex items-center border-b border-base-300">
    <button
      class="flex-1 py-3 hover:bg-base-300 transition-colors {activeSidebarTab ===
      'explorer'
        ? 'border-b-2 border-primary text-primary'
        : 'text-base-content/60'}"
      onclick={() => (activeSidebarTab = "explorer")}
      title="Explorer"
    >
      <DatabaseIcon class="w-5 h-5 mx-auto" />
    </button>
    <button
      class="flex-1 py-3 hover:bg-base-300 transition-colors {activeSidebarTab ===
      'pinned'
        ? 'border-b-2 border-primary text-primary'
        : 'text-base-content/60'}"
      onclick={() => (activeSidebarTab = "pinned")}
      title="Saved Queries"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="w-5 h-5 mx-auto"
        ><path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M11.48 3.499a.562.562 0 011.04 0l2.125 5.111a.563.563 0 00.475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 00-.182.557l1.285 5.385a.562.562 0 01-.84.61l-4.725-2.885a.563.563 0 00-.586 0L6.982 20.54a.562.562 0 01-.84-.61l1.285-5.386a.562.562 0 00-.182-.557l-4.204-3.602a.563.563 0 01.321-.988l5.518-.442a.563.563 0 00.475-.345L11.48 3.5z"
        /></svg
      >
    </button>
    <button
      class="flex-1 py-3 hover:bg-base-300 transition-colors {activeSidebarTab ===
      'diagrams'
        ? 'border-b-2 border-primary text-primary'
        : 'text-base-content/60'}"
      onclick={() => (activeSidebarTab = "diagrams")}
      title="Diagrams"
    >
      <SchemaIcon class="w-5 h-5 mx-auto" />
    </button>
    <button
      class="flex-1 py-3 hover:bg-base-300 transition-colors {activeSidebarTab ===
      'history'
        ? 'border-b-2 border-primary text-primary'
        : 'text-base-content/60'}"
      onclick={() => (activeSidebarTab = "history")}
      title="History"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="w-5 h-5 mx-auto"
        ><path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z"
        /></svg
      >
    </button>
  </div>

  <div class="flex-1 overflow-y-auto px-2 py-2 text-sm select-none">
    {#if activeSidebarTab === "explorer"}
      <div class="flex flex-col mb-2">
        <div class="flex items-center justify-between px-2 mb-2">
          <span class="text-xs font-bold opacity-50 uppercase">Connections</span
          >
          <div class="flex gap-1">
            <button
              class="btn btn-ghost btn-xs btn-square"
              onclick={handleAddFolder}
              title="New Folder"><FolderIcon class="h-4 w-4" /></button
            >
            <button
              class="btn btn-ghost btn-xs btn-square"
              onclick={handleAddConnection}
              title="New Connection"><AddIcon class="h-4 w-4" /></button
            >
          </div>
        </div>
        {#if $tags.length > 0}
          <div class="flex gap-1 overflow-x-auto px-2 pb-2 no-scrollbar">
            <button
              class="badge badge-xs cursor-pointer hover:opacity-80 transition-opacity
      {$activeConnectionTags === null || $activeConnectionTags.length === 0
                ? 'badge-neutral'
                : 'badge-ghost border-base-300'}"
              onclick={() => {
                $activeConnectionTags = null;
                loadConnections(authState.user?.user_id);
              }}>All</button
            >

            {#each $tags as tag}
              {@const isSelected = $activeConnectionTags?.includes(tag.tag_id)}
              <button
                class="badge badge-xs cursor-pointer hover:opacity-80 transition-opacity border"
                style:background-color={isSelected
                  ? tag.color_hex
                  : "transparent"}
                style:border-color={tag.color_hex || "#ccc"}
                style:color={isSelected ? "#fff" : "inherit"}
                onclick={() => {
                  if (isSelected) {
                    $activeConnectionTags = $activeConnectionTags!.filter(
                      (id) => id !== tag.tag_id
                    );
                    if ($activeConnectionTags.length === 0) {
                      $activeConnectionTags = null;
                    }
                  } else {
                    if ($activeConnectionTags === null) {
                      $activeConnectionTags = [tag.tag_id];
                    } else {
                      $activeConnectionTags = [
                        ...$activeConnectionTags,
                        tag.tag_id,
                      ];
                    }
                  }
                  loadConnections(authState.user?.user_id);
                }}
              >
                {tag.tag_name}
              </button>
            {/each}
          </div>
        {/if}
        {#if $bookmarks.length > 0}
          <div
            class="px-2 mb-1 text-[10px] font-bold opacity-50 uppercase mt-2"
          >
            Starred
          </div>
          <ul class="menu menu-xs bg-base-200 w-full p-0 gap-1 mb-4">
            {#each $bookmarks as bm}
              <li>
                <div class="flex items-center gap-1 group/item">
                  <button
                    class="flex flex-col items-start text-left py-2 h-auto gap-1 border-l-2 border-warning flex-1 bg-base-100 hover:bg-base-200"
                    onclick={() => openBookmark(bm)}
                  >
                    <div
                      class="font-bold truncate w-full flex items-center gap-2"
                    >
                      <span class="text-warning">★</span>
                      {bm.object_name || bm.schema_name || "Connection"}
                    </div>
                    <div
                      class="text-[10px] opacity-60 truncate w-full text-left"
                    >
                      {bm.schema_name} • {bm.object_type}
                    </div>
                  </button>
                  <button
                    class="btn btn-ghost btn-xs btn-square text-error opacity-0 group-hover/item:opacity-100 transition-opacity"
                    onclick={() =>
                      toggleStar(
                        new MouseEvent("click"),
                        bm.connection_id,
                        bm.schema_name,
                        bm.object_name,
                        bm.object_type
                      )}
                  >
                    ✕
                  </button>
                </div>
              </li>
            {/each}
          </ul>
          <div class="divider my-0 h-px opacity-20"></div>
        {/if}
      </div>
      <ul class="menu menu-xs bg-base-200 w-full p-0 gap-1">
        {#each $folders as folder (folder.folder_id)}
          {@const isFolderExpanded = expandedNodes.has(
            `folder:${folder.folder_id}`
          )}
          <li>
            <button
              class="flex items-center gap-2 font-medium"
              onclick={() => toggleNode(`folder:${folder.folder_id}`)}
              oncontextmenu={(e) => handleFolderContextMenu(e, folder)}
            >
              <FolderIcon
                class="h-4 w-4 text-warning {isFolderExpanded
                  ? 'rotate-90'
                  : ''} transition-transform"
              /> <span class="truncate">{folder.folder_name}</span>
            </button>
            {#if isFolderExpanded}
              <ul class="ml-2 border-l border-base-content/10">
                {#if groupedConnections.withFolder[folder.folder_id]}
                  {#each groupedConnections.withFolder[folder.folder_id] as connection (connection.connection_id)}
                    {@render connectionNode(connection)}
                  {/each}
                {/if}
              </ul>
            {/if}
          </li>
        {/each}
        {#each groupedConnections.withoutFolder as connection (connection.connection_id)}
          {@render connectionNode(connection)}
        {/each}
      </ul>
    {:else if activeSidebarTab === "pinned"}
      <div class="px-2 mb-2 flex flex-col gap-2">
        <div class="flex justify-between items-center">
          <span class="text-xs font-bold opacity-50 uppercase"
            >Saved Queries</span
          >
          <button
            class="btn btn-ghost btn-xs btn-square"
            onclick={togglePinnedSort}
            title="Toggle Sort Date"
          >
            {#if $pinnedSortAsc}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="w-4 h-4"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M3 4.5h14.25M3 9h9.75M3 13.5h9.75m4.5-4.5v12m0 0l-3.75-3.75M17.25 21L21 17.25"
                />
              </svg>
            {:else}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="w-4 h-4"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M3 4.5h14.25M3 9h9.75M3 13.5h5.25m5.25-.75L17.25 9m0 0L21 12.75M17.25 9v12"
                />
              </svg>
            {/if}
          </button>
        </div>

        <input
          type="text"
          placeholder="Search saved..."
          class="input input-xs input-bordered w-full"
          value={$pinnedSearch}
          oninput={handlePinnedSearch}
        />
      </div>

      {#if $pinnedQueries.length === 0}
        <div class="text-center p-4 opacity-50 text-xs">
          No saved queries found.
        </div>
      {:else}
        <ul class="menu menu-xs w-full p-0">
          {#each $pinnedQueries as pq (pq.pinned_query_id)}
            <li>
              <button
                class="flex flex-col items-start py-2 h-auto gap-1"
                onclick={() => openPinnedQuery(pq)}
                oncontextmenu={(e) => handlePinnedQueryContextMenu(e, pq)}
              >
                <div class="font-bold truncate w-full flex items-center gap-2">
                  <span class="text-warning">★</span>
                  <span
                    >{@html highlightMatch(pq.query_name, $pinnedSearch)}
                  </span>
                  <span class="text-[10px] opacity-50"
                    >{new Date(pq.created_at).toLocaleString()}</span
                  >
                </div>
                <div class="text-[10px] opacity-60 truncate w-full font-mono">
                  {pq.query_text}
                </div>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {:else if activeSidebarTab === "diagrams"}
      <div class="px-2 mb-2 text-xs font-bold opacity-50 uppercase">
        Saved Diagrams
      </div>
      {#if $diagrams.length === 0}
        <div class="text-center p-4 opacity-50 text-xs">No saved diagrams.</div>
      {:else}
        <ul class="menu menu-xs w-full p-0 gap-1">
          {#each $diagrams as d}
            <li>
              <button
                class="flex items-center gap-2 py-2"
                onclick={() => openSavedDiagram(d)}
                oncontextmenu={(e) => handleDiagramContextMenu(e, d)}
                ><SchemaIcon class="h-4 w-4 text-secondary" />
                <div class="flex flex-col items-start overflow-hidden">
                  <span class="font-bold truncate w-full">{d.diagram_name}</span
                  ><span class="text-[10px] opacity-50"
                    >ID: {d.connection_id}</span
                  >
                </div></button
              >
            </li>
          {/each}
        </ul>
      {/if}
    {:else if activeSidebarTab === "history"}
      <div class="px-2 mb-2 flex flex-col gap-2">
        <div class="flex justify-between items-center">
          <span class="text-xs font-bold opacity-50 uppercase"
            >Query History</span
          >
          <button
            class="btn btn-ghost btn-xs btn-square"
            onclick={toggleHistorySort}
            title="Toggle Sort By Date"
          >
            {#if $historySortDesc}
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
                  d="M3 4.5h14.25M3 9h9.75M3 13.5h9.75m4.5-4.5v12m0 0l-3.75-3.75M17.25 21L21 17.25"
                /></svg
              >
            {:else}
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
                  d="M3 4.5h14.25M3 9h9.75M3 13.5h5.25m5.25-.75L17.25 9m0 0L21 12.75M17.25 9v12"
                /></svg
              >
            {/if}
          </button>
        </div>

        <!-- SEARCH & FILTER -->
        <input
          type="text"
          placeholder="Search SQL..."
          class="input input-xs input-bordered w-full"
          value={$historySearch}
          oninput={handleHistorySearch}
        />
        <select
          class="select select-xs select-bordered w-full"
          value={$historyStatus}
          onchange={handleHistoryStatusChange}
        >
          <option value="all">All Statuses</option>
          <option value="success">Success</option>
          <option value="error">Error</option>
        </select>

        <div
          class="flex flex-col gap-1 border border-base-300 rounded p-2 bg-base-200/50"
        >
          <div class="flex justify-between items-center">
            <span class="text-[10px] font-bold opacity-50 uppercase"
              >Time Range</span
            >
            {#if $historyStartDate || $historyEndDate}
              <button
                class="btn btn-ghost btn-xs h-4 min-h-0 px-1 text-error"
                onclick={() => {
                  $historyStartDate = "";
                  $historyEndDate = "";
                  handleDateChange();
                }}
                title="Clear dates"
              >
                Clear
              </button>
            {/if}
          </div>

          <div class="flex flex-col">
            <span class="text-[9px] opacity-60 ml-1">From:</span>
            <input
              type="datetime-local"
              class="input input-xs input-bordered w-full px-1 text-[10px]"
              bind:value={$historyStartDate}
              onchange={handleDateChange}
            />
          </div>

          <div class="flex flex-col">
            <span class="text-[9px] opacity-60 ml-1">To:</span>
            <input
              type="datetime-local"
              class="input input-xs input-bordered w-full px-1 text-[10px]"
              bind:value={$historyEndDate}
              onchange={handleDateChange}
            />
          </div>
        </div>
      </div>

      <div class="flex justify-center gap-2 p-2 mt-2 border-t border-base-300">
        <button
          class="btn btn-xs btn-ghost"
          disabled={$historyPage === 0}
          onclick={prevHistoryPage}>Prev</button
        >
        <span class="text-xs self-center">Page {$historyPage + 1}</span>
        <button
          class="btn btn-xs btn-ghost"
          disabled={$queryHistory.length === 0}
          onclick={nextHistoryPage}>Next</button
        >
      </div>
      {#if $queryHistory.length === 0}
        <div class="text-center p-4 opacity-50 text-xs">No history found.</div>
      {:else}
        <ul class="menu menu-xs w-full p-0 gap-1">
          {#each $queryHistory as item}
            <li>
              <button
                class="flex flex-col items-start py-2 h-auto gap-1 border-b border-base-300/50"
                onclick={() => restoreHistoryQuery(item)}
                ><div class="flex items-center gap-2 w-full">
                  <div
                    class="w-2 h-2 rounded-full {item.status === 'success'
                      ? 'bg-success'
                      : 'bg-error'} shrink-0"
                  ></div>
                  <span class="text-[10px] font-mono opacity-50"
                    >{new Date(item.executed_at).toLocaleString()}</span
                  >
                </div>
                <div class="text-xs truncate w-full font-mono opacity-80 pl-4">
                  {@html highlightMatch(item.query_text, $historySearch)}
                </div></button
              >
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>

  <div class="border-t border-base-300 p-2">
    <div class="dropdown dropdown-top w-full">
      <button
        class="btn btn-ghost btn-sm w-full justify-start gap-2 px-1"
        onclick={() => (isUserDropdownOpen = !isUserDropdownOpen)}
      >
        <div class="avatar placeholder">
          <div
            class="bg-neutral text-neutral-content rounded-full w-6 flex items-center justify-center"
          >
            <span class="text-xs"
              >{authState.user?.username?.charAt(0) || "U"}</span
            >
          </div>
        </div>
        <div class="truncate flex-1 text-left">
          {authState.user?.username || "User"}
        </div>
      </button>
      {#if isUserDropdownOpen}
        <ul
          class="dropdown-content z-100 menu p-2 shadow bg-base-100 rounded-box w-52 mb-2"
        >
          <li>
            <button onclick={handleOpenDashboard}>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="w-4 h-4"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M3.75 6A2.25 2.25 0 016 3.75h2.25A2.25 2.25 0 0110.5 6v2.25a2.25 2.25 0 01-2.25 2.25H6a2.25 2.25 0 01-2.25-2.25V6zM3.75 15.75A2.25 2.25 0 016 13.5h2.25a2.25 2.25 0 012.25 2.25V18a2.25 2.25 0 01-2.25 2.25H6A2.25 2.25 0 013.75 18v-2.25zM13.5 6a2.25 2.25 0 012.25-2.25H18A2.25 2.25 0 0120.25 6v2.25A2.25 2.25 0 0118 10.5h-2.25a2.25 2.25 0 01-2.25-2.25V6zM13.5 15.75a2.25 2.25 0 012.25-2.25H18a2.25 2.25 0 012.25 2.25V18A2.25 2.25 0 0118 20.25h-2.25A2.25 2.25 0 0113.5 18v-2.25z"
                />
              </svg>
              Statistics / Dashboard
            </button>
          </li>
          {#if authState.user?.role === "admin"}
            <li>
              <button onclick={handleOpenSettings}
                ><SettingsIcon class="h-4 w-4" /> Manage Users</button
              >
            </li>
            <div class="divider my-1"></div>
          {/if}
          <li>
            <button onclick={handleLogout} class="text-error"
              ><LogoutIcon class="h-4 w-4" /> Logout</button
            >
          </li>
        </ul>
      {/if}
    </div>
  </div>
</div>

{#snippet connectionNode(connection: Connection)}
  {@const connKey = `conn:${connection.connection_id}`}
  {@const isConnExpanded = expandedNodes.has(connKey)}
  {@const cachedData = $schemaCache[connection.connection_id]}
  {@const isLoading = loadingConnections.has(connection.connection_id)}
  {@const isFailed = failedConnections.has(connection.connection_id)}
  {@const connTags = getConnectionTags(connection.connection_id)}
  <!-- Removed Bookmark Check for Connection -->

  <li>
    <div
      role="button"
      tabindex="0"
      class="flex items-center justify-between group pr-2"
      onclick={() => handleConnectionClick(connection)}
      onkeydown={(e) => e.key === "Enter" && handleConnectionClick(connection)}
      oncontextmenu={(e) => handleConnectionContextMenu(e, connection)}
    >
      <div class="flex items-center gap-2 overflow-hidden flex-1">
        {#if isLoading}
          <span class="loading loading-spinner loading-xs text-primary"></span>
        {:else}
          <DatabaseIcon
            class="h-4 w-4 {isFailed
              ? 'text-error'
              : isConnExpanded
                ? 'text-primary'
                : 'text-base-content/70'}"
          />
        {/if}
        <span
          class="truncate {isFailed ? 'text-error' : ''} flex-1"
          title={connection.connection_name}
        >
          <span class="opacity-50 text-[10px] mr-1"
            >({connection.connection_id})</span
          >
          {connection.connection_name}
        </span>
      </div>
      {#if connTags.length > 0}
        <div class="flex -space-x-1 shrink-0 mr-2">
          {#each connTags.slice(0, 3) as tag}
            <div
              class="w-2 h-2 rounded-full border border-base-200"
              style:background-color={tag.color_hex || "#ccc"}
              title={tag.tag_name}
            ></div>
          {/each}
        </div>
      {/if}
    </div>
    {#if isConnExpanded}
      <ul class="ml-2 border-l border-base-content/10 mt-0! pt-0!">
        {#if cachedData?.schemas}
          {#each cachedData.schemas as schema}
            {@render schemaNode(connection.connection_id, schema.schema_name)}
          {/each}
        {:else}
          <li class="disabled pl-4 py-1 opacity-50">
            <span class="loading loading-dots loading-xs"></span>
          </li>
        {/if}
      </ul>
    {/if}
  </li>
{/snippet}

{#snippet schemaNode(connId: number, schemaName: string)}
  {@const schemaKey = `schema:${connId}:${schemaName}`}
  {@const isSchemaExpanded = expandedNodes.has(schemaKey)}
  {@const cachedData = $schemaCache[connId]}
  {@const isSchemaBookmarked = isStarred(connId, schemaName, "", "schema")}

  <li>
    <div class="flex items-center justify-between group pr-2">
      <button
        class="flex items-center gap-2 py-1 flex-1 text-left"
        onclick={() => handleSchemaClick(connId, schemaName)}
        oncontextmenu={(e) => handleSchemaContextMenu(e, connId, schemaName)}
      >
        <SchemaIcon class="h-3 w-3 text-secondary" />
        <span class="truncate">{schemaName}</span>
      </button>
      <button
        class="{isSchemaBookmarked
          ? 'block text-warning'
          : 'hidden group-hover:block text-base-content/20 hover:text-warning'} transition-colors"
        onclick={(e) => toggleStar(e, connId, schemaName, "", "schema")}
        >★</button
      >
    </div>

    {#if isSchemaExpanded}
      <ul class="ml-2 border-l border-base-content/10 mt-0! pt-0!">
        {#if cachedData?.tables?.[schemaName]}
          {#each cachedData.tables[schemaName] as table}
            {@const isTableBookmarked = isStarred(
              connId,
              schemaName,
              table.table_name,
              "table"
            )}
            <li>
              <div class="flex items-center justify-between group/item pr-2">
                <button
                  class="flex items-center gap-2 py-1 text-xs flex-1 text-left"
                  onclick={() =>
                    handleTableClick(connId, schemaName, table.table_name)}
                >
                  <TableIcon class="h-3 w-3 text-info" /><span
                    class="truncate opacity-90">{table.table_name}</span
                  >
                </button>
                <button
                  class="{isTableBookmarked
                    ? 'block text-warning'
                    : 'hidden group-hover/item:block text-base-content/20 hover:text-warning'} transition-colors"
                  onclick={(e) =>
                    toggleStar(
                      e,
                      connId,
                      schemaName,
                      table.table_name,
                      "table"
                    )}>★</button
                >
              </div>
            </li>
          {/each}
        {/if}
        {#if cachedData?.views?.[schemaName]}
          {#each cachedData.views[schemaName] as view}
            {@const isViewBookmarked = isStarred(
              connId,
              schemaName,
              view.view_name,
              "view"
            )}
            <li>
              <div class="flex items-center justify-between group/item pr-2">
                <button
                  class="flex items-center gap-2 py-1 text-xs flex-1 text-left"
                  onclick={() =>
                    handleTableClick(connId, schemaName, view.view_name)}
                >
                  <ViewIcon class="h-3 w-3 text-success" /><span
                    class="truncate opacity-90">{view.view_name}</span
                  >
                </button>
                <button
                  class="{isViewBookmarked
                    ? 'block text-warning'
                    : 'hidden group-hover/item:block text-base-content/20 hover:text-warning'} transition-colors"
                  onclick={(e) =>
                    toggleStar(e, connId, schemaName, view.view_name, "view")}
                  >★</button
                >
              </div>
            </li>
          {/each}
        {/if}
      </ul>
    {/if}
  </li>
{/snippet}
