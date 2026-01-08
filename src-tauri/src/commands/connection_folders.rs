use crate::models::connection_folders::ConnectionFolder;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreateFolderRequest {
    pub user_id: i64,
    pub folder_name: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateFolderRequest {
    pub folder_id: i64,
    pub folder_name: String,
}

#[tauri::command]
pub async fn create_connection_folder(
    pool: State<'_, SqlitePool>,
    request: CreateFolderRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO connection_folders (user_id, folder_name) VALUES (?, ?) RETURNING folder_id",
    )
    .bind(request.user_id)
    .bind(request.folder_name)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_connection_folder(
    pool: State<'_, SqlitePool>,
    request: UpdateFolderRequest,
) -> Result<(), String> {
    sqlx::query("UPDATE connection_folders SET folder_name = ? WHERE folder_id = ?")
        .bind(request.folder_name)
        .bind(request.folder_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_connection_folder(
    pool: State<'_, SqlitePool>,
    folder_id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM connection_folders WHERE folder_id = ?")
        .bind(folder_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_connection_folders(
    pool: State<'_, SqlitePool>,
    user_id: i64,
) -> Result<Vec<ConnectionFolder>, String> {
    sqlx::query_as::<_, ConnectionFolder>("SELECT * FROM connection_folders WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())
}
