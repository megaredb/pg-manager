import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { authState } from "./auth.svelte";

export interface QueryHistoryItem {
  history_id: number;
  connection_id: number;
  query_text: string;
  status: "success" | "error";
  execution_time_ms?: number;
  error_message?: string;
  executed_at: string;
}

export const queryHistory = writable<QueryHistoryItem[]>([]);
export const historyPage = writable(0);
export const historyPageSize = 20;

export const historySearch = writable<string>("");
export const historyStatus = writable<string>("all");
export const historySortDesc = writable<boolean>(true);
export const historyStartDate = writable<string | null>(null);
export const historyEndDate = writable<string | null>(null);

function formatForSqlite(isoString: string): string | null {
  if (!isoString) return null;
  return isoString.replace("T", " ");
}

export const loadQueryHistory = async (page: number = 0, userId?: number) => {
  const uid = userId || authState.user?.user_id;
  if (!uid) return;

  const limit = 20;
  const offset = page * limit;
  const search = get(historySearch);
  const status = get(historyStatus);
  const sortDesc = get(historySortDesc);

  const rawStart = get(historyStartDate);
  const rawEnd = get(historyEndDate);

  try {
    const result = await invoke<any[]>("get_query_history", {
      request: {
        user_id: uid,
        limit: limit,
        offset: offset,
        search_query: search || null,
        status_filter: status,
        start_date: rawStart ? formatForSqlite(rawStart) : null,
        end_date: rawEnd ? formatForSqlite(rawEnd) : null,
        sort_desc: sortDesc,
      },
    });

    queryHistory.set(result);
    historyPage.set(page);
  } catch (error) {
    console.error("Failed to load query history:", error);
  }
};
