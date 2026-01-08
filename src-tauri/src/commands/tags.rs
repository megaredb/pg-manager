use crate::models::tags::Tag;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreateTagRequest {
    pub user_id: i64,
    pub tag_name: String,
    pub color_hex: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateTagRequest {
    pub tag_id: i64,
    pub tag_name: String,
    pub color_hex: Option<String>,
}

#[tauri::command]
pub async fn create_tag(
    pool: State<'_, SqlitePool>,
    request: CreateTagRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO tags (user_id, tag_name, color_hex) VALUES (?, ?, ?) RETURNING tag_id",
    )
    .bind(request.user_id)
    .bind(request.tag_name)
    .bind(request.color_hex)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_tag(
    pool: State<'_, SqlitePool>,
    request: UpdateTagRequest,
) -> Result<(), String> {
    sqlx::query("UPDATE tags SET tag_name = ?, color_hex = ? WHERE tag_id = ?")
        .bind(request.tag_name)
        .bind(request.color_hex)
        .bind(request.tag_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_tag(pool: State<'_, SqlitePool>, tag_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM connection_tags WHERE tag_id = ?")
        .bind(tag_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM tags WHERE tag_id = ?")
        .bind(tag_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_tags(pool: State<'_, SqlitePool>, user_id: i64) -> Result<Vec<Tag>, String> {
    sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())
}
