import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { authState } from "./auth.svelte";

export interface Bookmark {
  bookmark_id: number;
  connection_id: number;
  schema_name: string;
  object_name: string;
  object_type: string;
}

export const bookmarks = writable<Bookmark[]>([]);

export const loadBookmarks = async (userId?: number) => {
  const uid = userId || authState.user?.user_id;
  if (!uid) return;

  try {
    const result = await invoke<Bookmark[]>("get_bookmarks", { userId: uid });
    bookmarks.set(result);
  } catch (error) {
    console.error("Failed to load bookmarks:", error);
  }
};

export const toggleBookmark = async (
  connId: number,
  schema: string,
  objName: string,
  type: string
) => {
  await invoke("toggle_bookmark", {
    request: {
      connection_id: connId,
      schema_name: schema,
      object_name: objName,
      object_type: type,
    },
  });
  await loadBookmarks(authState.user?.user_id);
};

export const isBookmarked = (
  bms: Bookmark[],
  connId: number,
  schema: string,
  objName: string,
  type: string
) => {
  return bms.some(
    (b) =>
      b.connection_id === connId &&
      b.schema_name === schema &&
      b.object_name === objName &&
      b.object_type === type
  );
};
