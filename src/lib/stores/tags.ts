import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { authState } from "./auth.svelte";

export interface Tag {
  tag_id: number;
  user_id: number;
  tag_name: string;
  color_hex: string | null;
}

export interface ConnectionTag {
  tag_id: number;
  connection_id: number;
}

export const tags = writable<Tag[]>([]);
export const connectionTags = writable<Record<number, number[]>>({});

export const loadTags = async (userId?: number) => {
  const uid = userId || authState.user?.user_id;
  if (!uid) return;

  try {
    const result = await invoke<Tag[]>("get_tags", { userId: uid });
    tags.set(result);

    const relations = await invoke<ConnectionTag[]>("get_all_connection_tags");
    const map: Record<number, number[]> = {};
    relations.forEach((r) => {
      if (!map[r.connection_id]) map[r.connection_id] = [];
      map[r.connection_id].push(r.tag_id);
    });
    connectionTags.set(map);
  } catch (error) {
    console.error("Failed to load tags:", error);
  }
};

export const createTag = async (name: string, color: string | null) => {
  const uid = authState.user?.user_id;
  if (!uid) return;

  await invoke("create_tag", {
    request: { user_id: uid, tag_name: name, color_hex: color },
  });
  await loadTags(uid);
};

export const deleteTag = async (id: number) => {
  await invoke("delete_tag", { tagId: id });
  await loadTags();
};

export const addTagToConnection = async (tagId: number, connId: number) => {
  await invoke("add_connection_tag", {
    request: { tag_id: tagId, connection_id: connId },
  });
  await loadTags();
};

export const removeTagFromConnection = async (
  tagId: number,
  connId: number
) => {
  await invoke("remove_connection_tag", { tagId, connectionId: connId });
  await loadTags();
};
