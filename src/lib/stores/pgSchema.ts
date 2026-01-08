import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface Schema {
  schema_name: string;
}

export interface Table {
  table_name: string;
  table_schema: string;
}

export interface View {
  view_name: string;
  view_schema: string;
}

export interface SchemaData {
  schemas: Schema[];
  tables: Record<string, Table[]>; // key: schema_name
  views: Record<string, View[]>; // key: schema_name
}

export const schemaCache = writable<Record<number, SchemaData>>({});

export const testConnection = async (
  connectionId: number
): Promise<boolean> => {
  try {
    const result = await invoke<boolean>("test_connection", { connectionId });
    return result;
  } catch (error) {
    console.error("Connection test failed:", error);
    return false;
  }
};

export const loadSchemas = async (connectionId: number): Promise<Schema[]> => {
  try {
    const schemas = await invoke<Schema[]>("get_schemas", { connectionId });

    // Update cache
    schemaCache.update((cache) => {
      if (!cache[connectionId]) {
        cache[connectionId] = { schemas: [], tables: {}, views: {} };
      }
      cache[connectionId].schemas = schemas;
      return cache;
    });

    return schemas;
  } catch (error) {
    console.error("Failed to load schemas:", error);
    throw error;
  }
};

export const loadTables = async (
  connectionId: number,
  schemaName: string
): Promise<Table[]> => {
  try {
    const tables = await invoke<Table[]>("get_tables", {
      connectionId,
      schemaName,
    });

    // Update cache
    schemaCache.update((cache) => {
      if (!cache[connectionId]) {
        cache[connectionId] = { schemas: [], tables: {}, views: {} };
      }
      cache[connectionId].tables[schemaName] = tables;
      return cache;
    });

    return tables;
  } catch (error) {
    console.error("Failed to load tables:", error);
    throw error;
  }
};

export const loadViews = async (
  connectionId: number,
  schemaName: string
): Promise<View[]> => {
  try {
    const views = await invoke<View[]>("get_views", {
      connectionId,
      schemaName,
    });

    // Update cache
    schemaCache.update((cache) => {
      if (!cache[connectionId]) {
        cache[connectionId] = { schemas: [], tables: {}, views: {} };
      }
      cache[connectionId].views[schemaName] = views;
      return cache;
    });

    return views;
  } catch (error) {
    console.error("Failed to load views:", error);
    throw error;
  }
};
