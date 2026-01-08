import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { authState } from "./auth.svelte";

export interface ConnectionFolder {
  folder_id: number;
  user_id: number;
  folder_name: string;
}

export const folders = writable<ConnectionFolder[]>([]);

export const loadFolders = async (userId?: number) => {
  const uid = userId || authState.user?.user_id;
  if (!uid) return;

  try {
    const result = await invoke<ConnectionFolder[]>("get_connection_folders", {
      userId: uid,
    });
    folders.set(result);
  } catch (error) {
    console.error("Failed to load folders:", error);
  }
};

export const createFolder = async (folderName: string, userId: number) => {
  try {
    await invoke("create_connection_folder", {
      request: {
        user_id: userId,
        folder_name: folderName,
      },
    });
    await loadFolders(userId);
  } catch (error) {
    console.error("Failed to create folder:", error);
    throw error;
  }
};

export const updateFolder = async (folderId: number, folderName: string) => {
  try {
    await invoke("update_connection_folder", {
      request: {
        folder_id: folderId,
        folder_name: folderName,
      },
    });
    await loadFolders();
  } catch (error) {
    console.error("Failed to update folder:", error);
    throw error;
  }
};

export const deleteFolder = async (folderId: number) => {
  try {
    await invoke("delete_connection_folder", { folderId });
    await loadFolders();
  } catch (error) {
    console.error("Failed to delete folder:", error);
    throw error;
  }
};
