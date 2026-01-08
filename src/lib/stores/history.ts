import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface QueryHistoryItem {
  history_id: number;
  connection_id: number;
  connection_name: string;
  query_text: string;
  status: string;
  execution_time_ms?: number;
  error_message?: string;
  executed_at?: string;
}

export const queryHistory = writable<QueryHistoryItem[]>([]);

export const loadQueryHistory = async (
  search: string = "",
  fromDate: string | null = null,
  toDate: string | null = null,
  status: string | null = null
) => {
  try {
    // Convert empty strings to null for optional params if needed,
    // but Rust side handles Option<String> which is null or string.
    // In JS, passing null explicitly is safer for Option types.
    const result = await invoke<QueryHistoryItem[]>("get_query_history", {
      search: search || null,
      fromDate: fromDate || null,
      toDate: toDate || null,
      status: status || null,
    });
    queryHistory.set(result);
  } catch (error) {
    console.error("Failed to load query history:", error);
  }
};
