import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { authState } from "./auth.svelte";

export interface Diagram {
  diagram_id: number;
  connection_id: number;
  diagram_name: string;
  definition_json: string;
  created_at?: string;
}

export const diagrams = writable<Diagram[]>([]);

export const loadDiagrams = async (userId?: number) => {
  const uid = userId || authState.user?.user_id;
  if (!uid) return;

  try {
    const result = await invoke<Diagram[]>("get_diagrams", { userId: uid });
    diagrams.set(result);
  } catch (error) {
    console.error("Failed to load diagrams:", error);
  }
};

export const createDiagram = async (payload: {
  connection_id: number;
  diagram_name: string;
  definition_json: string;
}) => {
  try {
    await invoke("create_diagram", { request: payload });
    await loadDiagrams();
  } catch (error) {
    console.error("Failed to create diagram:", error);
    throw error;
  }
};

export const updateDiagram = async (
  id: number,
  payload: { diagram_name: string; definition_json: string }
) => {
  try {
    await invoke("update_diagram", {
      request: { diagram_id: id, ...payload },
    });
    await loadDiagrams();
  } catch (error) {
    console.error("Failed to update diagram:", error);
    throw error;
  }
};

export const deleteDiagram = async (id: number) => {
  try {
    await invoke("delete_diagram", { diagramId: id });
    await loadDiagrams();
  } catch (error) {
    console.error("Failed to delete diagram:", error);
    throw error;
  }
};
