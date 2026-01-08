import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { authState } from "./auth.svelte";

export interface Connection {
  connection_id: number;
  folder_id?: number;
  user_id: number;
  connection_name: string;
  host: string;
  port?: number;
  db_name: string;
  db_user: string;
  db_password_encrypted: string;
  ssl_mode?: string;
}

export const connections = writable<Connection[]>([]);

export const activeConnectionTags = writable<number[] | null>(null);

export const loadConnections = async (userId?: number) => {
  const uid = userId || authState.user?.user_id;
  if (!uid) return;

  const tags = get(activeConnectionTags);

  try {
    const result = await invoke<Connection[]>("get_connections", {
      userId: uid,
      withTags: tags && tags.length > 0 ? tags : null,
    });
    connections.set(result);
  } catch (error) {
    console.error("Failed to load connections:", error);
  }
};

export const createConnection = async (payload: any) => {
  try {
    payload.db_password = payload.db_password_encrypted;
    delete payload.db_password_encrypted;
    await invoke("create_connection", { request: payload });
    await loadConnections();
  } catch (error) {
    console.error("Failed to create connection:", error);
    throw error;
  }
};

export const updateConnection = async (connectionId: number, payload: any) => {
  try {
    payload.connection_id = connectionId;
    payload.db_password = payload.db_password_encrypted;
    delete payload.db_password_encrypted;
    await invoke("update_connection", { request: payload });
    await loadConnections();
  } catch (error) {
    console.error("Failed to update connection:", error);
    throw error;
  }
};

export const deleteConnection = async (connectionId: number) => {
  try {
    await invoke("delete_connection", { connectionId });
    await loadConnections();
  } catch (error) {
    console.error("Failed to delete connection:", error);
    throw error;
  }
};
