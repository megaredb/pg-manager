use crate::models::query_history::QueryHistory;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct AddQueryHistoryRequest {
    pub connection_id: i64,
    pub query_text: String,
    pub status: String,
    pub execution_time_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct GetQueryHistoryRequest {
    pub user_id: i64,
    pub limit: i64,
    pub offset: i64,
    pub search_query: Option<String>,
    pub status_filter: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub sort_desc: Option<bool>,
}

#[tauri::command]
pub async fn add_query_history(
    pool: State<'_, SqlitePool>,
    request: AddQueryHistoryRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO query_history (connection_id, query_text, status, execution_time_ms, error_message) VALUES (?, ?, ?, ?, ?) RETURNING history_id",
    )
    .bind(request.connection_id)
    .bind(request.query_text)
    .bind(request.status)
    .bind(request.execution_time_ms)
    .bind(request.error_message)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn get_query_history(
    pool: State<'_, SqlitePool>,
    request: GetQueryHistoryRequest,
) -> Result<Vec<QueryHistory>, String> {
    let mut sql = String::from(
        "SELECT qh.* FROM query_history qh
         JOIN connections c ON qh.connection_id = c.connection_id
         WHERE c.user_id = ?",
    );

    let has_status = request
        .status_filter
        .as_ref()
        .map(|s| !s.is_empty() && s != "all")
        .unwrap_or(false);

    if has_status {
        sql.push_str(" AND qh.status = ?");
    }

    if request.start_date.is_some() {
        sql.push_str(" AND qh.executed_at >= ?");
    }

    if request.end_date.is_some() {
        sql.push_str(" AND qh.executed_at <= ?");
    }
    let mut order_clauses = Vec::new();

    let has_search = request
        .search_query
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    if has_search {
        order_clauses.push("(qh.query_text LIKE ?) DESC");
    }

    let is_desc = request.sort_desc.unwrap_or(true);
    if is_desc {
        order_clauses.push("qh.executed_at DESC");
    } else {
        order_clauses.push("qh.executed_at ASC");
    }

    if !order_clauses.is_empty() {
        sql.push_str(" ORDER BY ");
        sql.push_str(&order_clauses.join(", "));
    }

    sql.push_str(" LIMIT ? OFFSET ?");

    let mut query = sqlx::query_as::<_, QueryHistory>(&sql).bind(request.user_id);

    if has_status {
        query = query.bind(request.status_filter.unwrap());
    }

    if let Some(start) = request.start_date {
        query = query.bind(start);
    }

    if let Some(end) = request.end_date {
        query = query.bind(end);
    }

    if has_search {
        if let Some(search) = &request.search_query {
            let search_pattern = format!("%{search}%");
            query = query.bind(search_pattern);
        }
    }

    query = query.bind(request.limit).bind(request.offset);

    query.fetch_all(&*pool).await.map_err(|e| e.to_string())
}
