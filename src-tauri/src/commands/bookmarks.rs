use crate::models::bookmarks::Bookmark;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct BookmarkRequest {
    pub connection_id: i64,
    pub schema_name: String,
    pub object_name: String,
    pub object_type: String,
}

#[tauri::command]
pub async fn toggle_bookmark(
    pool: State<'_, SqlitePool>,
    request: BookmarkRequest,
) -> Result<bool, String> {
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT bookmark_id FROM bookmarks WHERE connection_id = ? AND schema_name = ? AND object_name = ? AND object_type = ?"
    )
    .bind(request.connection_id)
    .bind(&request.schema_name)
    .bind(&request.object_name)
    .bind(&request.object_type)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(id) = existing {
        sqlx::query("DELETE FROM bookmarks WHERE bookmark_id = ?")
            .bind(id)
            .execute(&*pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        sqlx::query(
            "INSERT INTO bookmarks (connection_id, schema_name, object_name, object_type) VALUES (?, ?, ?, ?)"
        )
        .bind(request.connection_id)
        .bind(request.schema_name)
        .bind(request.object_name)
        .bind(request.object_type)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn get_bookmarks(
    pool: State<'_, SqlitePool>,
    user_id: i64,
) -> Result<Vec<Bookmark>, String> {
    sqlx::query_as::<_, Bookmark>(
        "SELECT b.* FROM bookmarks b
         JOIN connections c ON b.connection_id = c.connection_id
         WHERE c.user_id = ?",
    )
    .bind(user_id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())
}
