import Database from "@tauri-apps/plugin-sql";

export const DB = await Database.load("sqlite:pgmanager.db");
