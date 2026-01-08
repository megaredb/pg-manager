mod commands;
mod models;
mod password;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::fs;
use std::str::FromStr;
use tauri::Manager;

use commands::prelude as cmds;
use password::hash_password;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let app_dir = app_handle
                    .path()
                    .app_local_data_dir()
                    .expect("failed to get app data dir");
                if !app_dir.exists() {
                    fs::create_dir_all(&app_dir).expect("failed to create app data dir");
                }
                let db_path = app_dir.join("app.db");
                let db_url = format!("sqlite://{}", db_path.to_string_lossy());

                if !db_path.exists() {
                    fs::File::create(&db_path).expect("failed to create db file");
                }

                let options = SqliteConnectOptions::from_str(&db_url)
                    .expect("failed to parse db url")
                    .create_if_missing(true);

                let pool = SqlitePoolOptions::new()
                    .connect_with(options)
                    .await
                    .expect("failed to connect to db");

                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("failed to run migrations");

                let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_users")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);

                if user_count == 0 {
                    sqlx::query(
                        "INSERT INTO app_users (username, password_hash, role) VALUES ('admin', ?, 'admin')",
                    )
                    .bind(hash_password("admin"))
                    .execute(&pool)
                    .await
                    .expect("failed to create default user");
                }

                app_handle.manage(pool);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmds::add_connection_tag,
            cmds::add_query_history,
            cmds::create_app_user,
            cmds::create_app_user_log,
            cmds::toggle_bookmark,
            cmds::create_connection,
            cmds::create_connection_folder,
            cmds::create_diagram,
            cmds::create_pinned_query,
            cmds::create_tag,
            cmds::delete_app_user,
            cmds::delete_connection,
            cmds::delete_pinned_query,
            cmds::delete_connection_folder,
            cmds::delete_tag,
            cmds::execute_query,
            cmds::get_app_user_logs,
            cmds::get_app_users,
            cmds::get_all_connection_tags,
            cmds::get_bookmarks,
            cmds::get_connection_folders,
            cmds::get_tags_for_connection,
            cmds::get_connections,
            cmds::get_diagrams,
            cmds::get_schema_columns,
            cmds::get_pinned_queries,
            cmds::get_query_history,
            cmds::get_schemas,
            cmds::get_tables,
            cmds::get_tags,
            cmds::get_views,
            cmds::test_connection,
            cmds::update_app_user,
            cmds::update_connection,
            cmds::update_pinned_query,
            cmds::update_connection_folder,
            cmds::verify_user_credentials,
            cmds::remove_connection_tag,
            cmds::get_foreign_keys,
            cmds::create_diagram,
            cmds::update_diagram,
            cmds::delete_diagram,
            cmds::get_user_statistics,
            cmds::update_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
