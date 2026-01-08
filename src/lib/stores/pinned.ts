import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface PinnedQueryItem {
  pinned_query_id: number;
  connection_id: number;
  connection_name: string;
  query_name: string;
  query_text: string;
  description?: string;
}

export const pinnedQueries = writable<PinnedQueryItem[]>([]);

export const loadPinnedQueries = async (search: string = "") => {
  try {
    const result = await invoke<PinnedQueryItem[]>("get_pinned_queries", {
      search: search || null,
    });
    pinnedQueries.set(result);
  } catch (error) {
    console.error("Failed to load pinned queries:", error);
  }
};
