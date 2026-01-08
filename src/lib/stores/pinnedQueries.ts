import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { authState } from "./auth.svelte";

export interface PinnedQuery {
  pinned_query_id: number;
  connection_id: number;
  query_name: string;
  query_text: string;
  description?: string;
  created_at: string;
}

export const pinnedQueries = writable<PinnedQuery[]>([]);
export const pinnedSearch = writable<string>("");
export const pinnedSortAsc = writable<boolean>(false);

export const loadPinnedQueries = async (userId?: number) => {
  const uid = userId || authState.user?.user_id;
  if (!uid) return;

  const searchQuery = get(pinnedSearch);
  const sortAsc = get(pinnedSortAsc);

  try {
    const result = await invoke<PinnedQuery[]>("get_pinned_queries", {
      userId: uid,
      searchQuery: searchQuery || null,
      sortAsc: sortAsc,
    });
    pinnedQueries.set(result);
  } catch (error) {
    console.error("Failed to load pinned queries:", error);
  }
};

export const createPinnedQuery = async (payload: {
  connection_id: number;
  query_name: string;
  query_text: string;
  description?: string;
}) => {
  try {
    await invoke("create_pinned_query", { request: payload });
    await loadPinnedQueries();
  } catch (error) {
    console.error("Failed to create pinned query:", error);
    throw error;
  }
};

export const updatePinnedQuery = async (
  id: number,
  payload: { query_name: string; description?: string }
) => {
  try {
    await invoke("update_pinned_query", {
      request: { pinned_query_id: id, ...payload },
    });
    await loadPinnedQueries();
  } catch (error) {
    console.error("Failed to update pinned query:", error);
    throw error;
  }
};

export const deletePinnedQuery = async (id: number) => {
  try {
    await invoke("delete_pinned_query", { pinnedQueryId: id });
    await loadPinnedQueries();
  } catch (error) {
    console.error("Failed to delete pinned query:", error);
    throw error;
  }
};
