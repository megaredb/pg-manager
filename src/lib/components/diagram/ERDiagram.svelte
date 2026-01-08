<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    SvelteFlow,
    Background,
    Controls,
    type Node,
    type Edge,
    BackgroundVariant,
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import ELK from "elkjs/lib/elk.bundled.js";
  import TableNode from "./TableNode.svelte";

  interface Props {
    connectionId: number;
    schemaName?: string;
    initialData?: string;
  }

  let { connectionId, schemaName = "public", initialData }: Props = $props();

  let nodes = $state<Node[]>([]);
  let edges = $state<Edge[]>([]);

  const nodeTypes = {
    table: TableNode,
  };

  let isLoading = $state(true);
  const elk = new ELK();

  // Edge Context Menu State
  let edgeMenu = $state<{ id: string; x: number; y: number } | null>(null);

  export function getDiagramState() {
    return JSON.stringify({ nodes, edges });
  }

  const getLayoutedElements = async (rawNodes: Node[], rawEdges: Edge[]) => {
    const graph = {
      id: "root",
      layoutOptions: {
        "elk.algorithm": "layered",
        "elk.direction": "RIGHT",
        "elk.spacing.nodeNode": "60",
        "elk.layered.spacing.nodeNodeBetweenLayers": "120",
      },
      children: rawNodes.map((node) => ({
        id: node.id,
        width: 240,
        height:
          45 +
          (Array.isArray(node.data.columns) ? node.data.columns.length : 0) *
            28,
      })),
      edges: rawEdges.map((edge) => ({
        id: edge.id,
        sources: [edge.source],
        targets: [edge.target],
      })),
    };

    try {
      const layoutedGraph = await elk.layout(graph);
      return {
        nodes: rawNodes.map((node) => {
          const layoutedNode = layoutedGraph.children?.find(
            (n) => n.id === node.id
          );
          return {
            ...node,
            position: { x: layoutedNode?.x || 0, y: layoutedNode?.y || 0 },
          };
        }),
        edges: rawEdges,
      };
    } catch (e) {
      console.error("ELK Layout error:", e);
      return { nodes: rawNodes, edges: rawEdges };
    }
  };

  async function loadData() {
    isLoading = true;
    try {
      if (initialData) {
        try {
          const parsed = JSON.parse(initialData);
          nodes = parsed.nodes || [];
          edges = parsed.edges || [];

          return;
        } catch (e) {
          console.error("Failed to parse saved diagram", e);
        }
      }

      const [tablesRaw, columnsRaw, fks] = await Promise.all([
        invoke<any[]>("get_tables", { connectionId, schemaName }),
        invoke<any[]>("get_schema_columns", { connectionId, schemaName }),
        invoke<any[]>("get_foreign_keys", { connectionId, schemaName }).catch(
          (err) => []
        ),
      ]);

      const columnsByTable: Record<string, any[]> = {};
      columnsRaw.forEach((col) => {
        if (!columnsByTable[col.table_name])
          columnsByTable[col.table_name] = [];
        const isPk =
          col.column_name === "id" ||
          col.column_name === `${col.table_name}_id`;
        columnsByTable[col.table_name].push({
          name: col.column_name,
          type: col.data_type,
          isPk: isPk,
        });
      });

      const rawNodes: Node[] = [];
      const rawEdges: Edge[] = [];

      for (const t of tablesRaw) {
        rawNodes.push({
          id: t.table_name,
          type: "table",
          position: { x: 0, y: 0 },
          data: {
            label: t.table_name,
            columns: columnsByTable[t.table_name] || [],
          },
        });
      }

      for (const fk of fks) {
        const targetCols = columnsByTable[fk.table_name] || [];
        const sourceCols = columnsByTable[fk.foreign_table_name] || [];
        const hasTargetCol = targetCols.some((c) => c.name === fk.column_name);
        const hasSourceCol = sourceCols.some(
          (c) => c.name === fk.foreign_column_name
        );

        if (hasTargetCol && hasSourceCol) {
          rawEdges.push({
            id: `e-${fk.constraint_name}`,
            source: fk.foreign_table_name,
            target: fk.table_name,
            sourceHandle: `${fk.foreign_column_name}-source`,
            targetHandle: `${fk.column_name}-target`,
            animated: true,
            style: "stroke: #a6adbb; stroke-width: 2;",
            type: "smoothstep",
          });
        }
      }

      const layouted = await getLayoutedElements(rawNodes, rawEdges);
      nodes = layouted.nodes;
      edges = layouted.edges;
    } catch (e) {
      console.error("Failed to load ER diagram data:", e);
    } finally {
      isLoading = false;
    }
  }

  // --- EVENTS ---
  function onEdgeContextMenu({
    event,
    edge,
  }: {
    event: MouseEvent;
    edge: Edge;
  }) {
    event.preventDefault();
    // Position menu using client coordinates
    edgeMenu = {
      id: edge.id,
      x: event.clientX,
      y: event.clientY,
    };
  }

  function deleteEdge() {
    if (edgeMenu) {
      edges = edges.filter((e) => e.id !== edgeMenu!.id);
      edgeMenu = null;
    }
  }

  onMount(() => {
    loadData();
    // Закрываем меню при клике в любом месте
    window.addEventListener("click", () => {
      edgeMenu = null;
    });
  });
</script>

<div class="w-full h-full bg-base-100 relative">
  {#if isLoading}
    <div
      class="absolute inset-0 flex items-center justify-center bg-base-100 z-10 bg-opacity-80"
    >
      <span class="loading loading-spinner loading-lg text-primary"></span>
    </div>
  {/if}

  <!-- @ts-ignore -->
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    class="bg-base-200"
    defaultEdgeOptions={{ type: "smoothstep" }}
    onedgecontextmenu={onEdgeContextMenu}
  >
    <Controls />
    <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
  </SvelteFlow>

  <!-- CUSTOM EDGE CONTEXT MENU -->
  {#if edgeMenu}
    <div
      class="fixed z-50 menu bg-base-100 rounded-box shadow-lg border border-base-300 p-1 w-32"
      style="left: {edgeMenu.x}px; top: {edgeMenu.y}px;"
    >
      <button
        class="btn btn-ghost btn-sm text-error justify-start"
        onclick={deleteEdge}
      >
        Delete Connection
      </button>
    </div>
  {/if}
</div>
