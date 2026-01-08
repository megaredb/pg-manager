use crate::models::connection_tags::ConnectionTag;
use crate::models::tags::Tag;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct AddConnectionTagRequest {
    pub tag_id: i64,
    pub connection_id: i64,
}

#[tauri::command]
pub async fn add_connection_tag(
    pool: State<'_, SqlitePool>,
    request: AddConnectionTagRequest,
) -> Result<(), String> {
    sqlx::query("INSERT OR IGNORE INTO connection_tags (tag_id, connection_id) VALUES (?, ?)")
        .bind(request.tag_id)
        .bind(request.connection_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_connection_tag(
    pool: State<'_, SqlitePool>,
    tag_id: i64,
    connection_id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM connection_tags WHERE tag_id = ? AND connection_id = ?")
        .bind(tag_id)
        .bind(connection_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_tags_for_connection(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<Vec<Tag>, String> {
    sqlx::query_as::<_, Tag>(
        "SELECT t.* FROM tags t
         JOIN connection_tags ct ON t.tag_id = ct.tag_id
         WHERE ct.connection_id = ?",
    )
    .bind(connection_id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_connection_tags(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<ConnectionTag>, String> {
    sqlx::query_as::<_, ConnectionTag>("SELECT * FROM connection_tags")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())
}
