use crate::commands::app_user_logs::log_action_internal;
use crate::models::connections::Connection;
use crate::password::encrypt_data;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreateConnectionRequest {
    pub user_id: i64,
    pub connection_name: String,
    pub host: String,
    pub port: Option<i64>,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub ssl_mode: Option<String>,
    pub folder_id: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct UpdateConnectionRequest {
    pub connection_id: i64,
    pub connection_name: String,
    pub host: String,
    pub port: Option<i64>,
    pub db_name: String,
    pub db_user: String,
    pub db_password: Option<String>,
    pub ssl_mode: Option<String>,
    pub folder_id: Option<i64>,
}

#[tauri::command]
pub async fn create_connection(
    pool: State<'_, SqlitePool>,
    request: CreateConnectionRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let encrypted_pass = encrypt_data(&request.db_password);

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO connections (user_id, connection_name, host, port, db_name, db_user, db_password_encrypted, ssl_mode, folder_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING connection_id",
    )
    .bind(request.user_id)
    .bind(&request.connection_name)
    .bind(request.host)
    .bind(request.port)
    .bind(request.db_name)
    .bind(request.db_user)
    .bind(encrypted_pass)
    .bind(request.ssl_mode)
    .bind(request.folder_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    let _ = log_action_internal(
        &pool,
        request.user_id,
        "CREATE_CONNECTION",
        Some(&request.connection_name),
    )
    .await;

    Ok(id)
}

#[tauri::command]
pub async fn update_connection(
    pool: State<'_, SqlitePool>,
    request: UpdateConnectionRequest,
) -> Result<(), String> {
    let new_password = match &request.db_password {
        Some(p) if !p.is_empty() => Some(encrypt_data(p)),
        _ => None,
    };

    let user_id: Option<i64> = (if let Some(encrypted_pass) = new_password {
        sqlx::query_scalar("UPDATE connections SET db_password_encrypted = ?, connection_name = ?, host = ?, port = ?, db_name = ?, db_user = ?, ssl_mode = ?, folder_id = ? WHERE connection_id = ? RETURNING user_id")
            .bind(encrypted_pass)
    } else {
        sqlx::query_scalar("UPDATE connections SET connection_name = ?, host = ?, port = ?, db_name = ?, db_user = ?, ssl_mode = ?, folder_id = ? WHERE connection_id = ? RETURNING user_id")
    })
    .bind(&request.connection_name)
    .bind(request.host)
    .bind(request.port)
    .bind(request.db_name)
    .bind(request.db_user)
    .bind(request.ssl_mode)
    .bind(request.folder_id)
    .bind(request.connection_id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(uid) = user_id {
        let _ = log_action_internal(
            &pool,
            uid,
            "UPDATE_CONNECTION",
            Some(&request.connection_name),
        )
        .await;
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_connection(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<(), String> {
    let user_id: Option<i64> =
        sqlx::query_scalar("DELETE FROM connections WHERE connection_id = ? RETURNING user_id")
            .bind(connection_id)
            .fetch_optional(&*pool)
            .await
            .map_err(|e| e.to_string())?;

    if let Some(uid) = user_id {
        let _ = log_action_internal(
            &pool,
            uid,
            "DELETE_CONNECTION",
            Some(&format!("ID: {connection_id}")),
        )
        .await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_connections(
    pool: State<'_, SqlitePool>,
    user_id: i64,
    with_tags: Option<Vec<i64>>,
) -> Result<Vec<Connection>, String> {
    let mut sql = String::from("SELECT * FROM connections WHERE user_id = ?");

    let has_tags = with_tags.as_ref().map(|t| !t.is_empty()).unwrap_or(false);

    if has_tags {
        let tags = with_tags.as_ref().unwrap();
        let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
        let in_clause = placeholders.join(", ");

        sql.push_str(&format!(
            " AND connection_id IN (SELECT connection_id FROM connection_tags WHERE tag_id IN ({in_clause}))"
        ));
    }

    let mut query = sqlx::query_as::<_, Connection>(&sql);

    query = query.bind(user_id);

    if has_tags {
        if let Some(tags) = &with_tags {
            for tag_id in tags {
                query = query.bind(tag_id);
            }
        }
    }

    query.fetch_all(&*pool).await.map_err(|e| e.to_string())
}
