use crate::models::app_user_logs::AppUserLog;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreateAppUserLogRequest {
    pub user_id: i64,
    pub action_type: String,
    pub details: Option<String>,
}

#[derive(serde::Serialize)]
pub struct UserStatistics {
    pub total_connections: i64,
    pub total_queries: i64,
    pub success_queries: i64,
    pub error_queries: i64,
    pub pinned_queries: i64,
    pub last_login: Option<NaiveDateTime>,
    pub user_created_at: Option<NaiveDateTime>,
}

pub async fn log_action_internal(
    pool: &SqlitePool,
    user_id: i64,
    action_type: &str,
    details: Option<&str>,
) -> Result<(), String> {
    sqlx::query("INSERT INTO app_user_logs (user_id, action_type, details) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(action_type)
        .bind(details)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_app_user_log(
    pool: State<'_, SqlitePool>,
    request: CreateAppUserLogRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO app_user_logs (user_id, action_type, details) VALUES (?, ?, ?) RETURNING log_id",
    )
    .bind(request.user_id)
    .bind(request.action_type)
    .bind(request.details)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn get_app_user_logs(
    pool: State<'_, SqlitePool>,
    user_id: i64,
) -> Result<Vec<AppUserLog>, String> {
    sqlx::query_as::<_, AppUserLog>(
        "SELECT * FROM app_user_logs WHERE user_id = ? ORDER BY timestamp DESC LIMIT 100",
    )
    .bind(user_id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_statistics(
    pool: State<'_, SqlitePool>,
    user_id: i64,
) -> Result<UserStatistics, String> {
    let total_connections: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM connections WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&*pool)
            .await
            .unwrap_or(0);

    let total_queries: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM query_history qh 
         JOIN connections c ON qh.connection_id = c.connection_id 
         WHERE c.user_id = ?",
    )
    .bind(user_id)
    .fetch_one(&*pool)
    .await
    .unwrap_or(0);

    let success_queries: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM query_history qh 
         JOIN connections c ON qh.connection_id = c.connection_id 
         WHERE c.user_id = ? AND qh.status = 'success'",
    )
    .bind(user_id)
    .fetch_one(&*pool)
    .await
    .unwrap_or(0);

    let error_queries = total_queries - success_queries;

    let pinned_queries: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pinned_queries pq 
         JOIN connections c ON pq.connection_id = c.connection_id 
         WHERE c.user_id = ?",
    )
    .bind(user_id)
    .fetch_one(&*pool)
    .await
    .unwrap_or(0);

    let last_login: Option<NaiveDateTime> = sqlx::query_scalar(
        "SELECT timestamp FROM app_user_logs WHERE user_id = ? AND action_type = 'LOGIN' ORDER BY timestamp DESC LIMIT 1 OFFSET 1"
    )
    .bind(user_id)
    .fetch_optional(&*pool)
    .await
    .unwrap_or(None);

    let user_created_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT created_at FROM app_users WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&*pool)
            .await
            .unwrap_or(None);

    Ok(UserStatistics {
        total_connections,
        total_queries,
        success_queries,
        error_queries,
        pinned_queries,
        last_login,
        user_created_at,
    })
}
