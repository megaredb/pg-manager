use crate::models::pinned_queries::PinnedQuery;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreatePinnedQueryRequest {
    pub connection_id: i64,
    pub query_name: String,
    pub query_text: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdatePinnedQueryRequest {
    pub pinned_query_id: i64,
    pub query_name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn create_pinned_query(
    pool: State<'_, SqlitePool>,
    request: CreatePinnedQueryRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO pinned_queries (connection_id, query_name, query_text, description) VALUES (?, ?, ?, ?) RETURNING pinned_query_id",
    )
    .bind(request.connection_id)
    .bind(request.query_name)
    .bind(request.query_text)
    .bind(request.description)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_pinned_query(
    pool: State<'_, SqlitePool>,
    request: UpdatePinnedQueryRequest,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE pinned_queries SET query_name = ?, description = ? WHERE pinned_query_id = ?",
    )
    .bind(request.query_name)
    .bind(request.description)
    .bind(request.pinned_query_id)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_pinned_query(
    pool: State<'_, SqlitePool>,
    pinned_query_id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM pinned_queries WHERE pinned_query_id = ?")
        .bind(pinned_query_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_pinned_queries(
    pool: State<'_, SqlitePool>,
    user_id: i64,
    search_query: Option<String>,
    sort_asc: Option<bool>,
) -> Result<Vec<PinnedQuery>, String> {
    let mut sql = String::from(
        "SELECT pq.* FROM pinned_queries pq
         JOIN connections c ON pq.connection_id = c.connection_id
         WHERE c.user_id = ?",
    );

    let mut order_clauses = Vec::new();

    let has_search = search_query
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    if has_search {
        order_clauses.push("(pq.query_name LIKE ?) DESC");
    }

    let is_asc = sort_asc.unwrap_or(true);
    if is_asc {
        order_clauses.push("pq.created_at ASC");
    } else {
        order_clauses.push("pq.created_at DESC");
    }

    if !order_clauses.is_empty() {
        sql.push_str(" ORDER BY ");
        sql.push_str(&order_clauses.join(", "));
    }

    let mut query = sqlx::query_as::<_, PinnedQuery>(&sql).bind(user_id);

    if has_search {
        if let Some(search) = &search_query {
            let search_pattern = format!("%{search}%");
            query = query.bind(search_pattern);
        }
    }

    query.fetch_all(&*pool).await.map_err(|e| e.to_string())
}
