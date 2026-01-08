use crate::models::diagrams::Diagram;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreateDiagramRequest {
    pub connection_id: i64,
    pub diagram_name: String,
    pub definition_json: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateDiagramRequest {
    pub diagram_id: i64,
    pub diagram_name: String,
    pub definition_json: String,
}

#[tauri::command]
pub async fn create_diagram(
    pool: State<'_, SqlitePool>,
    request: CreateDiagramRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO diagrams (connection_id, diagram_name, definition_json) VALUES (?, ?, ?) RETURNING diagram_id",
    )
    .bind(request.connection_id)
    .bind(request.diagram_name)
    .bind(request.definition_json)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_diagram(
    pool: State<'_, SqlitePool>,
    request: UpdateDiagramRequest,
) -> Result<(), String> {
    sqlx::query("UPDATE diagrams SET diagram_name = ?, definition_json = ? WHERE diagram_id = ?")
        .bind(request.diagram_name)
        .bind(request.definition_json)
        .bind(request.diagram_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_diagram(pool: State<'_, SqlitePool>, diagram_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM diagrams WHERE diagram_id = ?")
        .bind(diagram_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_diagrams(
    pool: State<'_, SqlitePool>,
    user_id: i64,
) -> Result<Vec<Diagram>, String> {
    sqlx::query_as::<_, Diagram>(
        "SELECT d.* FROM diagrams d
         JOIN connections c ON d.connection_id = c.connection_id
         WHERE c.user_id = ?
         ORDER BY d.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())
}
